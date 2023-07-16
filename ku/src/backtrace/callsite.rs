use core::{
    fmt,
    num::NonZeroUsize,
    panic::Location,
};

use super::Backtrace;

/// Отладочная информация о точке вызова некоторой функции.
#[derive(Clone, Copy)]
pub struct Callsite {
    /// Backtrace --- список адресов возврата от самого внутреннего стекового фрейма наружу.
    backtrace: [Option<NonZeroUsize>; Self::BACKTRACE_LEN],

    /// Точка исходного кода с именем исходного файла и строкой.
    location: &'static Location<'static>,
}

impl Callsite {
    /// Создаёт отладочную информация о точке вызова.
    /// В качестве `location` можно передавать [`Location::caller()`].
    /// Для использования которой вызывающая функция должна быть помечена
    /// [атрибутом `#[track_caller]`](https://doc.rust-lang.org/reference/attributes/codegen.html#the-track_caller-attribute).
    #[inline(never)]
    pub fn new(location: &'static Location<'static>) -> Self {
        let mut backtrace = [None; Self::BACKTRACE_LEN];

        if let Ok(current_backtrace) = Backtrace::current() {
            for (return_address, stack_frame) in backtrace.iter_mut().zip(current_backtrace) {
                *return_address =
                    stack_frame.return_address().and_then(|x| NonZeroUsize::new(x.into_usize()));
            }
        }

        Self {
            backtrace,
            location,
        }
    }

    /// Создаёт пустой объект информации о точке вызова.
    #[track_caller]
    pub const fn zero() -> Self {
        Self {
            backtrace: [None; Self::BACKTRACE_LEN],
            location: Location::caller(),
        }
    }

    /// Максимальная глубина хранимого backtrace.
    const BACKTRACE_LEN: usize = 16;
}

impl fmt::Debug for Callsite {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        formatter
            .debug_struct("Callsite")
            .field("location", &format_args!("{self}"))
            .finish()
    }
}

impl fmt::Display for Callsite {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{}", self.location)?;

        let mut separator = ", backtrace: [";
        for return_address in self.backtrace.iter().map_while(|&x| Some(x?.get())) {
            write!(formatter, "{}{:#X}", separator, return_address)?;
            separator = " ";
        }
        if separator == " " {
            write!(formatter, "]")?;
        }

        Ok(())
    }
}
