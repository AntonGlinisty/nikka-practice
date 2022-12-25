use core::fmt;

use bitflags::bitflags;

bitflags! {
    /// [Причина некорректности](https://wiki.osdev.org/Exceptions#Error_code)
    /// обращения к странице виртуальной памяти.
    pub struct PageFaultInfo: usize {
        /// Страница отображена, но нарушены флаги доступа к ней.
        const PRESENT = 1 << 0;

        /// Обращение на запись (не обязательно недопустимое).
        const WRITE = 1 << 1;

        /// Обращение из пространства пользователя (не обязательно недопустимое).
        const USER = 1 << 2;

        /// Таблица страниц содержит некорректные данные --- есть `1` в зарезервированном бите.
        const RESERVED = 1 << 3;

        /// Недопустимое выполнение кода.
        const EXECUTE = 1 << 4;
    }
}

impl fmt::Display for PageFaultInfo {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{:#b} = {} | {} | {}",
            self.bits(),
            if self.contains(Self::RESERVED) {
                "malformed page table (a reserved bit set)"
            } else if self.contains(Self::PRESENT) {
                "access mode violation"
            } else {
                "non-present page"
            },
            if self.contains(Self::EXECUTE) {
                "execute"
            } else if self.contains(Self::WRITE) {
                "write"
            } else {
                "read"
            },
            if self.contains(Self::USER) {
                "user"
            } else {
                "kernel"
            },
        )
    }
}
