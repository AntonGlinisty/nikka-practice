use core::{
    fmt::{
        Display,
        Formatter,
        Result,
    },
    num::NonZeroU64,
};

use number_prefix::NumberPrefix;

#[repr(transparent)]
pub struct Hz(NonZeroU64);

impl Hz {
    pub fn new(hz: u64) -> Option<Self> {
        NonZeroU64::new(hz).map(Hz)
    }
}

impl Display for Hz {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
        let hz = self.0.get();
        match NumberPrefix::decimal(hz as f64) {
            NumberPrefix::Standalone(_) => {
                write!(formatter, "{hz} Hz")
            },
            NumberPrefix::Prefixed(prefix, value) => {
                write!(formatter, "{value:.3} {}Hz", prefix.symbol())
            },
        }
    }
}
