use core::{
    cell::Cell,
    convert::TryFrom,
    fmt,
    fmt::Write,
    result,
};

use heapless::String;
use postcard::flavors::SerFlavor;
use scopeguard::defer;
use serde::{
    Deserialize,
    Serialize,
    Serializer,
    ser::SerializeTuple,
};
use tracing::{
    Collect,
    Event,
    Id,
    Level,
    Metadata,
    field::{
        Field,
        Visit,
    },
    span::{
        Attributes,
        Record,
    },
};
use tracing_core::span::Current;

use super::{
    RingBufferWriteTx,
    error::{
        Error,
        Error::InvalidArgument,
        Result,
    },
    pipe,
    time::{
        Tsc,
        datetime,
    },
};

pub use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct LogMetadata<'a> {
    file: Option<&'a str>,
    line: Option<u32>,
    level: char,
    target: &'a str,
    timestamp: Tsc,
}

impl<'a> LogMetadata<'a> {
    pub fn new(
        metadata: &Metadata<'a>,
        timestamp: Tsc,
    ) -> Self {
        Self {
            file: metadata.file(),
            line: metadata.line(),
            level: level_into_symbol(metadata.level()),
            target: metadata.target(),
            timestamp,
        }
    }

    pub fn file(&self) -> Option<&'a str> {
        self.file
    }

    pub fn line(&self) -> Option<u32> {
        self.line
    }

    pub fn level(&self) -> Result<Level> {
        level_try_from_symbol(self.level)
    }

    pub fn target(&self) -> &'a str {
        self.target
    }

    pub fn timestamp(&self) -> Tsc {
        self.timestamp
    }
}

#[derive(Debug, Deserialize)]
pub struct LogField<'a>(#[serde(borrow)] &'a str, #[serde(borrow)] LogFieldValue<'a>);

impl LogField<'_> {
    pub fn name(&self) -> &str {
        self.0
    }

    pub fn value(&self) -> &LogFieldValue<'_> {
        &self.1
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LogFieldValue<'a> {
    Bool(bool),
    I64(i64),
    Str(&'a str),
    U64(u64),
    VecStr,
}

struct LogEvent<'a> {
    serializer: postcard::Serializer<LogBuffer<'a>>,
    result: Result<()>,
}

impl LogEvent<'_> {
    fn record_event(
        event: &Event<'_>,
        timestamp: Tsc,
    ) -> Result<()> {
        let mut log_event = Self {
            serializer: postcard::Serializer {
                output: LogBuffer::new(),
            },
            result: Ok(()),
        };

        log_event.result = log_event.record_header(event, timestamp);
        log_event.is_ok_so_far()?;

        event.record(&mut log_event);
        log_event.is_ok_so_far()?;

        log_event.serializer.output.buffer.commit();

        Ok(())
    }

    fn record_header(
        &mut self,
        event: &Event<'_>,
        timestamp: Tsc,
    ) -> Result<()> {
        let metadata = LogMetadata::new(event.metadata(), timestamp);
        metadata.serialize(&mut self.serializer)?;

        let field_count = u8::try_from(event.fields().count())?;
        field_count.serialize(&mut self.serializer)?;

        Ok(())
    }

    fn record_field(
        &mut self,
        field: &Field,
        value: &LogFieldValue<'_>,
    ) -> Result<()> {
        self.is_ok_so_far()?;

        let mut s = self.serializer.serialize_tuple(2)?;
        s.serialize_element(field.name())?;
        s.serialize_element(&value)?;
        s.end()?;

        Ok(())
    }

    fn record_vec_str(
        &mut self,
        value: &dyn fmt::Debug,
    ) -> Result<()> {
        self.is_ok_so_far()?;

        self.write_fmt(format_args!("{value:?}"))?;

        Option::<&str>::serialize(&None, &mut self.serializer)?;

        Ok(())
    }

    fn record_vec_element(
        &mut self,
        text: &str,
    ) -> Result<()> {
        self.is_ok_so_far()?;

        Some(text).serialize(&mut self.serializer)?;

        Ok(())
    }

    fn set_result(
        &mut self,
        result: Result<()>,
    ) {
        if self.result.is_ok() && result.is_err() {
            self.result = result;
        }
    }

    fn is_ok_so_far(&self) -> Result<()> {
        if self.serializer.output.result.is_err() {
            self.serializer.output.result?;
        }

        if self.result.is_err() {
            self.result.clone()?;
        }

        Ok(())
    }
}

impl Visit for LogEvent<'_> {
    fn record_debug(
        &mut self,
        field: &Field,
        value: &dyn fmt::Debug,
    ) {
        let result = self.record_field(field, &LogFieldValue::VecStr);
        self.set_result(result);

        let result = self.record_vec_str(value);
        self.set_result(result);
    }

    fn record_bool(
        &mut self,
        field: &Field,
        value: bool,
    ) {
        let result = self.record_field(field, &LogFieldValue::Bool(value));
        self.set_result(result);
    }

    fn record_i64(
        &mut self,
        field: &Field,
        value: i64,
    ) {
        let result = self.record_field(field, &LogFieldValue::I64(value));
        self.set_result(result);
    }

    fn record_str(
        &mut self,
        field: &Field,
        value: &str,
    ) {
        let result = self.record_field(field, &LogFieldValue::Str(value));
        self.set_result(result);
    }

    fn record_u64(
        &mut self,
        field: &Field,
        value: u64,
    ) {
        let result = self.record_field(field, &LogFieldValue::U64(value));
        self.set_result(result);
    }
}

impl Write for LogEvent<'_> {
    fn write_str(
        &mut self,
        text: &str,
    ) -> fmt::Result {
        let result = self.record_vec_element(text);
        if result.is_ok() {
            Ok(())
        } else {
            self.set_result(result);
            Err(fmt::Error)
        }
    }
}

struct PlanB<const N: usize> {
    log_message: String<N>,
    overflow: bool,
    separator: bool,
}

impl<const N: usize> PlanB<N> {
    fn record_event(event: &Event<'_>) -> String<N> {
        let mut plan_b = Self {
            log_message: String::new(),
            overflow: false,
            separator: false,
        };

        event.record(&mut plan_b);

        plan_b.log_message
    }

    fn record_field(
        &mut self,
        field: &Field,
        value: &dyn fmt::Debug,
    ) -> result::Result<(), ()> {
        if self.separator {
            self.log_message.push_str("; ")?;
        }
        self.separator = true;

        if field.name() != "message" {
            self.log_message.push_str(field.name())?;
            self.log_message.push_str(" = ")?;
        }

        self.write_fmt(format_args!("{value:?}")).map_err(|_| ())
    }
}

impl<const N: usize> Visit for PlanB<N> {
    fn record_debug(
        &mut self,
        field: &Field,
        value: &dyn fmt::Debug,
    ) {
        if !self.overflow {
            self.overflow = self.record_field(field, value).is_err();
        }
    }
}

impl<const N: usize> Write for PlanB<N> {
    fn write_str(
        &mut self,
        text: &str,
    ) -> fmt::Result {
        for ch in text.chars() {
            self.log_message.push(ch).map_err(|_| fmt::Error)?;
        }
        Ok(())
    }
}

pub struct LogCollector {
    flush: Cell<Option<fn()>>,
    level: Level,
    lost_recently: Cell<usize>,
    lost_totally: Cell<usize>,
    plan_b_failures: Cell<usize>,
    plan_c_failures: Cell<usize>,
    recursion: Cell<usize>,
    recursive_failure: Cell<usize>,
}

impl LogCollector {
    const fn new(level: Level) -> Self {
        LogCollector {
            flush: Cell::new(None),
            level,
            lost_recently: Cell::new(0),
            lost_totally: Cell::new(0),
            plan_b_failures: Cell::new(0),
            plan_c_failures: Cell::new(0),
            recursion: Cell::new(0),
            recursive_failure: Cell::new(0),
        }
    }

    pub fn set_flush(
        &self,
        flush: fn(),
    ) {
        self.flush.set(Some(flush));
    }

    fn flush(&self) -> bool {
        if let Some(flush) = self.flush.get() {
            (flush)();
            true
        } else {
            false
        }
    }

    fn report_lost_messages_statistics(&self) {
        let recent = self.lost_recently.get();
        let recursive_failure = self.recursive_failure.get();

        if recent > 0 {
            let plan_b_failures = self.plan_b_failures.get();
            let plan_c_failures = self.plan_c_failures.get();

            let total = self.lost_totally.update(|x| x + recent);
            self.lost_recently.update(|x| x - recent);

            error!(
                recent,
                total, plan_b_failures, plan_c_failures, "lost some log messages",
            );

            if self.recursive_failure.get() > recursive_failure {
                // The error message about lost log messages seem to be lost also.
                // Rollback the statistics to trigger the same error message next time.
                self.lost_recently.update(|x| x + recent);
                self.lost_totally.update(|x| x - recent);
            }
        }
    }

    fn report_lost_message(
        &self,
        event: &Event<'_>,
        timestamp: Tsc,
        error: &Error,
    ) {
        self.lost_recently.update(|x| x + 1);
        let recursive_failure = self.recursive_failure.get();

        let error_message = "failed to log an event";
        let message_prefix = PlanB::<PLAN_B_MAX_MESSAGE_SIZE>::record_event(event);
        error!(
            ?error,
            timestamp = %datetime(timestamp),
            metadata = ?event.metadata(),
            ?message_prefix,
            "{}",
            error_message,
        );

        if self.recursive_failure.get() > recursive_failure {
            self.plan_b_failures.update(|x| x + 1);
            error!(
                ?error,
                timestamp = %datetime(timestamp),
                metadata = ?event.metadata(),
                "{}",
                error_message,
            );
        }

        if self.recursive_failure.get() > recursive_failure + 1 {
            self.plan_c_failures.update(|x| x + 1);
        }
    }
}

// This is safe as long as user processes are single threaded.
unsafe impl Sync for LogCollector {
}

impl Collect for LogCollector {
    fn new_span(
        &self,
        _span: &Attributes<'_>,
    ) -> Id {
        Id::from_u64(0)
    }

    fn event(
        &self,
        event: &Event<'_>,
    ) {
        let timestamp = Tsc::now();

        let recursion = self.recursion.update(|x| x + 1);
        let is_recursive = recursion > 1;
        defer! {
            self.recursion.update(|x| x - 1);
        }
        const RECURSION_LIMIT: usize = 3;
        if recursion > RECURSION_LIMIT {
            return;
        }

        const TRY_COUNT: i32 = 2;

        for tries_left in (0 .. TRY_COUNT).rev() {
            if let Err(error) = LogEvent::record_event(event, timestamp) {
                if !self.flush() || tries_left == 0 {
                    if is_recursive {
                        self.recursive_failure.update(|x| x + 1);
                    } else {
                        self.report_lost_message(event, timestamp, &error);
                    }
                }
            } else {
                if !is_recursive {
                    self.recursive_failure.set(0);
                    self.report_lost_messages_statistics();
                }
                return;
            }
        }
    }

    fn record(
        &self,
        _span: &Id,
        _values: &Record<'_>,
    ) {
    }

    fn record_follows_from(
        &self,
        _span: &Id,
        _follows: &Id,
    ) {
    }

    fn enabled(
        &self,
        metadata: &Metadata<'_>,
    ) -> bool {
        metadata.level() <= &self.level
    }

    fn enter(
        &self,
        _span: &Id,
    ) {
    }

    fn exit(
        &self,
        _span: &Id,
    ) {
    }

    fn current_span(&self) -> Current {
        Current::unknown()
    }
}

struct LogBuffer<'a> {
    buffer: RingBufferWriteTx<'a>,
    result: pipe::Result<()>,
}

impl LogBuffer<'_> {
    fn new() -> Self {
        Self {
            buffer: crate::process_info().log().write_tx(),
            result: Ok(()),
        }
    }
}

impl SerFlavor for LogBuffer<'_> {
    type Output = ();

    fn try_extend(
        &mut self,
        data: &[u8],
    ) -> result::Result<(), ()> {
        self.result = self.buffer.write(data);
        self.result.map_err(|_| ())
    }

    fn try_push(
        &mut self,
        data: u8,
    ) -> result::Result<(), ()> {
        self.result = self.buffer.write(&[data; 1]);
        self.result.map_err(|_| ())
    }

    fn release(self) -> result::Result<Self::Output, ()> {
        self.result.map_err(|_| ())
    }
}

pub const fn level_into_symbol(level: &Level) -> char {
    match *level {
        Level::ERROR => 'E',
        Level::WARN => 'W',
        Level::INFO => 'I',
        Level::DEBUG => 'D',
        Level::TRACE => 'T',
    }
}

const fn level_try_from_symbol(level: char) -> Result<Level> {
    match level {
        'E' => Ok(Level::ERROR),
        'W' => Ok(Level::WARN),
        'I' => Ok(Level::INFO),
        'D' => Ok(Level::DEBUG),
        'T' => Ok(Level::TRACE),
        _ => Err(InvalidArgument),
    }
}

pub static LOG_COLLECTOR: LogCollector = LogCollector::new(Level::DEBUG);

const PLAN_B_MAX_MESSAGE_SIZE: usize = 128;
