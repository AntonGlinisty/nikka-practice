use x86::io;

pub trait Cursor {
    fn new() -> Self;
    fn get(&self) -> usize;
    fn set(
        &mut self,
        position: usize,
    );
}

pub struct VgaCursor {}

impl VgaCursor {
    const INDEX_REGISTER: u16 = 0x03D4;
    const DATA_REGISTER: u16 = 0x03D5;

    const POSITION_HIGH: u8 = 0x0E;
    const POSITION_LOW: u8 = 0x0F;

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

    fn set_height(
        &mut self,
        height: u8,
    ) {
        const CURSOR_BEGIN_LINE: u8 = 0x0A;
        const CURSOR_END_LINE: u8 = 0x0B;

        const MAX_LINES: u8 = 1 << 4;
        const LAST_LINE: u8 = MAX_LINES.checked_sub(1).expect("MAX_LINES should be positive");

        // TODO: your code here.
    }
}

impl Cursor for VgaCursor {
    fn new() -> Self {
        let mut cursor = Self {};
        cursor.set_height(2);

        cursor
    }

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
}
