use core::{
    fmt,
    ops::{
        Deref,
        DerefMut,
    },
};

use lazy_static::lazy_static;

use ku::sync::{
    IrqSpinlock,
    PanicStrategy,
};

use serial::{
    Com,
    Serial as _,
};

/// Последовательный порт.
pub struct Serial(Com);

impl Deref for Serial {
    type Target = Com;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Serial {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Write for Serial {
    fn write_str(
        &mut self,
        s: &str,
    ) -> fmt::Result {
        for octet in s.as_bytes() {
            self.0.send_octet(*octet);
        }

        Ok(())
    }
}

lazy_static! {
    /// Последовательный порт.
    pub static ref SERIAL: IrqSpinlock<Serial, { PanicStrategy::KnockDown }> = {
        IrqSpinlock::new(Serial(Com::new()))
    };
}

/// Печать в последовательный порт.
/// Этот макрос аналогичен [`std::print!()`](https://doc.rust-lang.org/std/macro.print.html).
/// ```ignore
/// print!("hello world");
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        {
            use core::fmt::Write as _;
            let serial = &mut *$crate::serial::SERIAL.lock();
            if serial.write_fmt(format_args!($($arg)*)).is_err() {
                serial.write_str("Err").unwrap();
            }
        }
    );
}

/// Печать в последовательный порт.
/// Этот макрос аналогичен [`std::println!()`](https://doc.rust-lang.org/std/macro.println.html).
/// ```ignore
/// println!("hello world");
/// ```
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
