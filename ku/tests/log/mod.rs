#![deny(warnings)]

use spin::Lazy;

use tracing::Level;
use tracing_subscriber::{
    self,
    fmt,
};

pub fn init() {
    *INIT;
}

static INIT: Lazy<()> = Lazy::new(|| {
    let format = fmt::format()
        .with_level(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(true)
        .compact();

    tracing_subscriber::fmt()
        .with_ansi(false)
        .event_format(format)
        .with_max_level(Level::DEBUG)
        .init();
});
