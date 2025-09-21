use ku::memory::{
    IndexDataPair,
    IndexDataPortPair,
};

pub trait Cursor {
    fn get(&mut self) -> usize;

    fn set(
        &mut self,
        position: usize,
    );

    fn set_disable(
        &mut self,
        disable: bool,
    );

    fn set_height(
        &mut self,
        height: u8,
    );
}

pub struct VgaCursor<T: IndexDataPair<u8>>(T);

impl<T: IndexDataPair<u8>> VgaCursor<T> {
    pub(super) unsafe fn new(port_pair: T) -> Self {
        Self(port_pair)
    }

    // TODO: your code here.
}

impl<T: IndexDataPair<u8>> Cursor for VgaCursor<T> {
    fn get(&mut self) -> usize {
        // TODO: your code here.
        0 // TODO: remove before flight.
    }

    fn set(
        &mut self,
        position: usize,
    ) {
        // TODO: your code here.
    }

    fn set_disable(
        &mut self,
        disable: bool,
    ) {
        // TODO: your code here.
    }

    fn set_height(
        &mut self,
        height: u8,
    ) {
        // TODO: your code here.
    }
}

pub(super) fn create_vga_cursor() -> VgaCursor<IndexDataPortPair<u8>> {
    const INDEX_PORT: u16 = 0x03D4;

    unsafe {
        VgaCursor::new(
            IndexDataPortPair::from_index_port(INDEX_PORT).expect("invalid VGA cursor index port"),
        )
    }
}

pub(super) const CURSOR_DISABLE: u8 = 1 << 5;
pub(super) const CURSOR_LINE_MASK: u8 = (1 << 5) - 1;
pub(super) const START_LINE: u8 = 0x0A;
pub(super) const END_LINE: u8 = 0x0B;
pub(super) const MAX_LINES: u8 = 1 << 4;
pub(super) const POSITION_HIGH: u8 = 0x0E;
pub(super) const POSITION_LOW: u8 = 0x0F;
