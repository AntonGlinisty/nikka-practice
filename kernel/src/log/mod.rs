use core::fmt::{
    Debug,
    Write,
};

use serde::Deserialize;
use spin::Mutex;
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
use tracing_core::{
    dispatch,
    dispatch::Dispatch,
    span::Current,
};

use ku::{
    RingBuffer,
    log::{
        LogField,
        LogFieldValue,
        LogMetadata,
        level_into_symbol,
    },
    time::{
        Tsc,
        datetime_ms,
    },
};
use text::{
    Color,
    print,
    println,
};

use crate::{
    error::{
        Error::Unimplemented,
        Result,
    },
    process::Pid,
    smp::LocalApic,
};

pub use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};

pub(super) fn init() {
    dispatch::set_global_default(Dispatch::from_static(&LOG_COLLECTOR)).unwrap();
}

pub(super) fn user_events(
    pid: Pid,
    log_buffer: &mut RingBuffer,
) {
    LOG_COLLECTOR.log.lock().user_events(pid, log_buffer);
}

struct LogEvent {
    separator: bool,
    color: Color,
}

impl LogEvent {
    const MESSAGE: Color = Color::WHITE;
    const VALUE: Color = Color::LIGHT_CYAN;

    fn new() -> Self {
        Self {
            separator: false,
            color: Self::VALUE,
        }
    }

    fn field(
        &mut self,
        name: &str,
    ) {
        if self.separator {
            print!("; ");
        } else {
            self.separator = true
        }

        if name == "message" {
            self.color = Self::MESSAGE;
        } else {
            print!("{} = ", name);
            self.color = Self::VALUE;
        }
    }

    fn str_value_part(
        &mut self,
        value_part: &str,
    ) {
        print!(color(self.color), "{}", value_part);
    }

    fn debug(
        &mut self,
        name: &str,
        value: &dyn Debug,
    ) {
        self.field(name);
        print!(color(self.color), "{:?}", value);
    }
}

impl Visit for LogEvent {
    fn record_debug(
        &mut self,
        field: &Field,
        value: &dyn Debug,
    ) {
        self.debug(field.name(), value);
    }
}

#[allow(unused)]
#[derive(Eq, PartialEq)]
enum Format {
    Compact,
    Full,
    Timeless,
}

struct Log {
    format: Format,
}

impl Log {
    const CPU: Color = Color::DARK_GRAY;
    const LOCATION: Color = Color::DARK_GRAY;

    const fn new(format: Format) -> Self {
        Self { format }
    }

    const fn level_color(level: &Level) -> Color {
        match *level {
            Level::ERROR => Color::LIGHT_RED,
            Level::WARN => Color::LIGHT_YELLOW,
            Level::INFO => Color::WHITE,
            Level::DEBUG => Color::LIGHT_BLUE,
            Level::TRACE => Color::DARK_GRAY,
        }
    }

    fn log_event(
        &self,
        event: &Event<'_>,
        tsc: Tsc,
    ) {
        self.log_metadata(
            event.metadata().level(),
            LogMetadata::new(event.metadata(), tsc),
        );
        event.record(&mut LogEvent::new());
        println!();
    }

    fn log_metadata(
        &self,
        level: &Level,
        metadata: LogMetadata,
    ) {
        if self.format != Format::Timeless {
            let timestamp = datetime_ms(metadata.timestamp());
            print!("{:?} ", timestamp.time());
        }

        print!(color(Self::CPU), "{} ", LocalApic::id());

        match self.format {
            Format::Compact | Format::Timeless => {
                print!(
                    color(Self::level_color(level)),
                    "{} ",
                    level_into_symbol(level),
                );
            },
            Format::Full => {
                print!(color(Self::level_color(level)), "{} ", level);
                print!(
                    color(Self::LOCATION),
                    "{} {}:{} ",
                    metadata.target(),
                    metadata.file().unwrap_or("?"),
                    metadata.line().unwrap_or(0),
                );
            },
        }
    }

    fn user_events(
        &self,
        pid: Pid,
        log_buffer: &mut RingBuffer,
    ) {
        let mut tx = log_buffer.read_tx();
        let mut deserializer = postcard::Deserializer::from_bytes(tx.read());

        while self.user_event(pid, &mut deserializer).is_ok() {}

        tx.commit();
    }

    fn user_event<'a>(
        &self,
        pid: Pid,
        deserializer: &mut postcard::Deserializer<'a, postcard::de_flavors::Slice<'a>>,
    ) -> Result<()> {
        let metadata = LogMetadata::deserialize(&mut *deserializer)?;
        let level = metadata.level().map_err(|_| Unimplemented)?;
        self.log_metadata(&level, metadata);

        let count = u8::deserialize(&mut *deserializer)?;
        let mut event = LogEvent::new();
        for _ in 0 .. count {
            let field = LogField::deserialize(&mut *deserializer)?;
            match field.value() {
                LogFieldValue::VecStr => {
                    event.field(field.name());
                    while let Some(value) = Option::<&str>::deserialize(&mut *deserializer)? {
                        event.str_value_part(value);
                    }
                },
                LogFieldValue::I64(value) => event.debug(field.name(), &value as &dyn Debug),
                LogFieldValue::U64(value) => event.debug(field.name(), &value as &dyn Debug),
                LogFieldValue::Bool(value) => event.debug(field.name(), &value as &dyn Debug),
                LogFieldValue::Str(value) => event.debug(field.name(), &value as &dyn Debug),
            }
        }
        event.debug("pid", &pid as &dyn Debug);

        println!();

        Ok(())
    }
}

struct LogCollector {
    level: Level,
    log: Mutex<Log>,
}

impl LogCollector {
    const fn new(
        format: Format,
        level: Level,
    ) -> Self {
        Self {
            level,
            log: Mutex::new(Log::new(format)),
        }
    }
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
        let now = Tsc::now();
        self.log.lock().log_event(event, now);
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

static LOG_COLLECTOR: LogCollector = LogCollector::new(Format::Compact, Level::DEBUG);
