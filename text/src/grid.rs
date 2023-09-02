use core::{
    cmp,
    ops::{
        Deref,
        DerefMut,
        Range,
    },
};

use static_assertions::{
    const_assert,
    const_assert_eq,
};
use volatile::Volatile;

use serial::Serial;

use super::Attribute;

pub fn is_graphical(octet: u8) -> bool {
    (b' ' + 1 .. 0x7F).contains(&octet)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Glyph {
    pub character: u8,
    attribute: Attribute,
}

impl Deref for Glyph {
    type Target = Self;

    fn deref(&self) -> &Self {
        self
    }
}

impl DerefMut for Glyph {
    fn deref_mut(&mut self) -> &mut Self {
        self
    }
}

pub const COLUMN_COUNT: usize = 80;
const ROW_COUNT: usize = 25;
pub const LEN: usize = COLUMN_COUNT * ROW_COUNT;

pub const TAB_WIDTH: usize = 8;

const_assert!(COLUMN_COUNT > 0);
const_assert_eq!(COLUMN_COUNT % TAB_WIDTH, 0);
const_assert_eq!(LEN % COLUMN_COUNT, 0);
const_assert!(LEN >= 3 * COLUMN_COUNT);

pub type Buffer = [Volatile<Glyph>; LEN];

pub struct Grid<'a> {
    buffer: &'a mut Buffer,

    row_start: usize,
    column: usize,

    attribute: Attribute,
}

impl<'a> Grid<'a> {
    pub fn new(
        buffer: &'a mut Buffer,
        attribute: Attribute,
    ) -> Self {
        Self {
            buffer,
            row_start: 0,
            column: 0,
            attribute,
        }
    }

    pub fn init<S: Serial>(
        &mut self,
        cursor_position: usize,
        serial: &mut S,
    ) -> usize {
        let position = self.printed_data_end(cursor_position);

        self.log_printed_data(serial, 0 .. position);

        if self.row_start >= LEN {
            self.scroll();
        } else {
            self.clear(position .. LEN);
        }

        position
    }

    pub fn attribute(&mut self) -> Attribute {
        self.attribute
    }

    pub fn set_attribute(
        &mut self,
        attribute: Attribute,
    ) {
        self.attribute = attribute;
    }

    pub fn position(&self) -> usize {
        self.row_start + self.column
    }

    pub fn is_newline(&self) -> bool {
        self.column == 0
    }

    pub fn clear(
        &mut self,
        range: Range<usize>,
    ) {
        // TODO: your code here.
    }

    pub fn print_character(
        &mut self,
        ch: char,
    ) {
        // TODO: your code here.
    }

    pub fn scroll(&mut self) {
        // TODO: your code here.
    }

    fn character(
        &mut self,
        ch: char,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    fn newline(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    fn tab(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    fn adjust_position(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    fn log_printed_data<S: Serial>(
        &mut self,
        serial: &mut S,
        range: Range<usize>,
    ) {
        self.row_start = 0;
        self.column = 0;

        let mut prev_graphical = 0;

        for i in range {
            if is_graphical(self.buffer[i].read().character) {
                while prev_graphical <= i {
                    serial.send_octet(self.buffer[prev_graphical].read().character);
                    prev_graphical += 1;
                }
            }

            self.column += 1;

            if self.column >= COLUMN_COUNT {
                self.row_start += COLUMN_COUNT;
                self.column = 0;
                prev_graphical = i + 1;
                serial.send_octet(b'\n');
            }
        }
    }

    fn printed_data_end(
        &self,
        cursor_position: usize,
    ) -> usize {
        let position = cmp::min(cursor_position, LEN);
        let mut printed_data_end = position;

        for i in position .. LEN {
            if is_graphical(self.buffer[i].read().character) {
                printed_data_end = i + 1;
            }
        }

        printed_data_end
    }
}
