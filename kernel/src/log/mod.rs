use core::fmt::{
    Debug,
    Write,
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

use crate::time;

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

struct LogCollector {
    format: Format,
}

impl LogCollector {
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
        let metadata = &event.metadata();

        match self.format {
            Format::Compact => {
                print!("{:?} ", now.time());
                print!(
                    color(Self::level_color(metadata.level())),
                    "{} ",
                    Self::level_symbol(metadata.level()),
                );
            },
            Format::Full => {
                print!("{:?} ", now);
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

        event.record(&mut LogEvent::new());
        println!();
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
        _metadata: &Metadata<'_>,
    ) -> bool {
        true
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

static LOG_COLLECTOR: LogCollector = LogCollector::new(Format::Compact);
