use core::fmt::{
    Debug,
    Write,
};

use chrono::{
    DateTime,
    Utc,
};
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

use text::{
    Color,
    print,
    println,
};

use crate::{
    smp::LocalApic,
    time,
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

struct LogEvent {
    separator: bool,
}

impl LogEvent {
    const MESSAGE: Color = Color::WHITE;
    const VALUE: Color = Color::LIGHT_CYAN;

    fn new() -> Self {
        Self { separator: false }
    }
}

impl Visit for LogEvent {
    fn record_debug(
        &mut self,
        field: &Field,
        value: &dyn Debug,
    ) {
        if self.separator {
            print!("; ");
        } else {
            self.separator = true
        }

        if field.name() == "message" {
            print!(color(Self::MESSAGE), "{:?}", value);
        } else {
            print!("{} = ", field.name());
            print!(color(Self::VALUE), "{:?}", value);
        }
    }
}

#[allow(unused)]
enum Format {
    Compact,
    Full,
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

    const fn level_symbol(level: &Level) -> char {
        match *level {
            Level::ERROR => 'E',
            Level::WARN => 'W',
            Level::INFO => 'I',
            Level::DEBUG => 'D',
            Level::TRACE => 'T',
        }
    }

    fn log_event(
        &self,
        event: &Event<'_>,
        timestamp: DateTime<Utc>,
    ) {
        self.log_metadata(event.metadata(), timestamp);
        event.record(&mut LogEvent::new());
        println!();
    }

    fn log_metadata(
        &self,
        metadata: &Metadata<'_>,
        timestamp: DateTime<Utc>,
    ) {
        match self.format {
            Format::Compact => {
                print!("{:?} ", timestamp.time());
                print!(color(Self::CPU), "{} ", LocalApic::id());
                print!(
                    color(Self::level_color(metadata.level())),
                    "{} ",
                    Self::level_symbol(metadata.level()),
                );
            },
            Format::Full => {
                print!("{:?} ", timestamp);
                print!(color(Self::CPU), "cpu={} ", LocalApic::id());
                print!(
                    color(Self::level_color(metadata.level())),
                    "{} ",
                    metadata.level(),
                );
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
        let now = time::now_ms();
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
