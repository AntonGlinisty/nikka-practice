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

pub mod test_scaffolding {
    pub use super::pit8254::test_scaffolding::TICKS_PER_SECOND;
}
