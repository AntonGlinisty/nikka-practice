use core::fmt::{
    Debug,
    Write,
};

use tracing::{
    Collect,
    Event,
    Id,
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
    print,
    println,
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

        if field.name() != "message" {
            print!("{} = ", field.name());
        }

        print!("{:?}", value);
    }
}

struct LogCollector;

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
        let metadata = &event.metadata();
        print!(
            "{} {} {}:{} ",
            metadata.level(),
            metadata.target(),
            metadata.file().unwrap_or("?"),
            metadata.line().unwrap_or(0),
        );
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

static LOG_COLLECTOR: LogCollector = LogCollector;
