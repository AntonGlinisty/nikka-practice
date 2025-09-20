use x86::io;

pub trait Cursor {
    fn get(&self) -> usize;

    fn set(
        &mut self,
        position: usize,
    );

    fn set_height(
        &mut self,
        height: u8,
    );
}

pub struct VgaCursor {}

impl VgaCursor {
    const INDEX_REGISTER: u16 = 0x03D4;
    const DATA_REGISTER: u16 = 0x03D5;

    const POSITION_HIGH: u8 = 0x0E;
    const POSITION_LOW: u8 = 0x0F;

    pub(super) const fn new() -> Self {
        Self {}
    }

    fn register(
        &self,
        register: u8,
    ) -> u8 {
        // TODO: your code here.
        0 // TODO: remove before flight.
    }

    fn set_register(
        &mut self,
        register: u8,
        value: u8,
    ) {
        // TODO: your code here.
    }
}

impl Cursor for VgaCursor {
    fn get(&self) -> usize {
        // TODO: your code here.
        0 // TODO: remove before flight.
    }

    fn set(
        &mut self,
        position: usize,
    ) {
        // TODO: your code here.
    }

    fn set_height(
        &mut self,
        height: u8,
    ) {
        const START_LINE: u8 = 0x0A;
        const END_LINE: u8 = 0x0B;

        const MAX_LINES: u8 = 1 << 4;
        const LAST_LINE: u8 = MAX_LINES.checked_sub(1).expect("MAX_LINES should be positive");

        // TODO: your code here.
    }
}
