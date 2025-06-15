#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_std]

mod cursor;
mod grid;

#[cfg(test)]
mod test;

use core::fmt::{
    Result,
    Write,
};

use bitflags::bitflags;
use lazy_static::lazy_static;

use serial::{
    Com,
    Serial,
};

use ku::sync::{
    PanicStrategy,
    Spinlock,
};

use cursor::{
    Cursor,
    VgaCursor,
};
use grid::Buffer;

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Color: u8 {
        const BLACK = 0;

        const RED = 1 << 2;
        const GREEN = 1 << 1;
        const BLUE = 1 << 0;

        const CYAN = Self::GREEN.bits() | Self::BLUE.bits();
        const MAGENTA = Self::RED.bits() | Self::BLUE.bits();
        const YELLOW = Self::RED.bits() | Self::GREEN.bits();

        const GRAY = Self::RED.bits() | Self::GREEN.bits() | Self::BLUE.bits();

        const LIGHT = 1 << 3;

        const DARK_GRAY = Self::LIGHT.bits() | Self::BLACK.bits();

        const LIGHT_RED = Self::LIGHT.bits() | Self::RED.bits();
        const LIGHT_GREEN = Self::LIGHT.bits() | Self::GREEN.bits();
        const LIGHT_BLUE = Self::LIGHT.bits() | Self::BLUE.bits();

        const LIGHT_CYAN = Self::LIGHT.bits() | Self::CYAN.bits();
        const LIGHT_MAGENTA = Self::LIGHT.bits() | Self::MAGENTA.bits();
        const LIGHT_YELLOW = Self::LIGHT.bits() | Self::YELLOW.bits();

        const WHITE = Self::LIGHT.bits() | Self::GRAY.bits();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Attribute(u8);

impl Attribute {
    pub const NORMAL: Attribute = Attribute::new(Color::GRAY, Color::BLACK);
    pub const PANIC: Attribute = Attribute::new(Color::WHITE, Color::RED);

    pub const fn new(
        foreground: Color,
        background: Color,
    ) -> Attribute {
        Attribute(background.bits() << Self::BACKGROUND_SHIFT | foreground.bits())
    }

    pub const fn background(&self) -> Color {
        Color::from_bits(self.0 >> Self::BACKGROUND_SHIFT).expect("undefined color")
    }

    const BACKGROUND_SHIFT: u8 = 4;
}

pub struct Text<'a, C: Cursor, S: Serial> {
    grid: grid::Grid<'a>,
    cursor: C,
    serial: S,
}

impl<'a, C: Cursor, S: Serial> Text<'a, C, S> {
    fn new(
        buffer: &'a mut Buffer,
        attribute: Attribute,
    ) -> Self {
        Self {
            grid: grid::Grid::new(buffer, attribute),
            cursor: C::new(),
            serial: S::new(),
        }
    }

    pub fn init(&mut self) {
        let position = self.grid.init(self.cursor.get(), &mut self.serial);

        let is_screen_clear = position == 0;
        if !is_screen_clear {
            if !self.grid.is_newline() {
                writeln!(self).unwrap();
            }

            writeln!(self).unwrap();
        }
    }

    pub fn attribute(&mut self) -> Attribute {
        self.grid.attribute()
    }

    pub fn set_attribute(
        &mut self,
        attribute: Attribute,
    ) {
        self.grid.set_attribute(attribute);
    }
}

impl<'a, C: Cursor, S: Serial> Write for Text<'a, C, S> {
    fn write_str(
        &mut self,
        text: &str,
    ) -> Result {
        for ch in text.chars() {
            self.grid.print_character(ch)
        }
        for octet in text.as_bytes() {
            self.serial.send_octet(*octet);
        }

        self.cursor.set(self.grid.position());

        Ok(())
    }
}

fn text_buffer() -> &'static mut Buffer {
    const TEXT_BUFFER_ADDRESS: usize = 0xB_8000;

    let buffer = TEXT_BUFFER_ADDRESS as *mut Buffer;

    unsafe { &mut *buffer }
}

type VgaText = Text<'static, VgaCursor, Com>;

lazy_static! {
    pub static ref TEXT: Spinlock<VgaText, { PanicStrategy::KnockDown }> =
        Spinlock::new(Text::<VgaCursor, Com>::new(
            text_buffer(),
            Attribute::NORMAL,
        ));
}

#[macro_export]
macro_rules! make_attribute {
    ($foreground:expr, $background:expr) => {
        $crate::Attribute::new($foreground, $background)
    };
    ($base:expr; $foreground:expr, $background:expr) => {
        $crate::make_attribute!($foreground, $background)
    };
    ($base:expr; $foreground:expr) => {
        $crate::make_attribute!($foreground, $base.background())
    };
}

#[macro_export]
macro_rules! print {
    (color($($colors:tt)*), $($arg:tt)*) => (
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut text = $crate::TEXT.lock();
            let old_attribute = text.attribute();
            let new_attribute = $crate::make_attribute!(old_attribute; $($colors)*);
            text.set_attribute(new_attribute);
            if text.write_fmt(format_args!($($arg)*)).is_err() {
                text.write_str("Err").unwrap();
            }
            text.set_attribute(old_attribute);
        })
    );
    ($($arg:tt)*) => (
        x86_64::instructions::interrupts::without_interrupts(|| {
            $crate::TEXT.lock().write_fmt(format_args!($($arg)*)).unwrap()
        })
    );
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    (color($($colors:tt)*), $($arg:tt)*) => (
        $crate::print!(
            color($($colors)*),
            "{}\n",
            format_args!($($arg)*),
        )
    );
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
