mod pit8254;
mod rtc;

pub use ku::{
    Hz,
    Tsc,
    TscDuration,
    now,
    now_ms,
    timer,
};

use crate::log::info;

pub use pit8254::Pit;
pub use rtc::Rtc;

pub(super) fn init() {
    pit8254::init();
    rtc::init();

    info!("time init");
}
