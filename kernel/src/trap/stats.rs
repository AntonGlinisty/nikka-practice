use core::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use super::COUNT;

/// Информация о прерывании с некоторым номером.
pub struct Stats {
    /// Сколько раз сработало это исключение или прерывание.
    count: AtomicUsize,

    /// Короткая мнемоника исключения или прерывания.
    mnemonic: &'static str,

    /// Имя исключения или прерывания.
    name: &'static str,
}

impl Stats {
    /// Создаёт информацию об исключении или прерывании
    /// с именем `name` и короткой мнемоникой `mnemonic`.
    const fn new(
        name: &'static str,
        mnemonic: &'static str,
    ) -> Stats {
        Stats {
            name,
            mnemonic,
            count: AtomicUsize::new(0),
        }
    }

    /// Сколько раз сработало это исключение или прерывание.
    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Короткая мнемоника исключения или прерывания.
    pub fn mnemonic(&self) -> &'static str {
        self.mnemonic
    }

    /// Имя исключения или прерывания.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Инкрементирует счётчик срабатывания исключения или прерывания.
    pub(super) fn inc(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Информация обо всех исключениях и прерываниях.
type InterruptStats = [Stats; COUNT];

/// Информация обо всех исключениях и прерываниях.
pub static TRAP_STATS: InterruptStats = [
    Stats::new("Divide Error", "#DE"),
    Stats::new("Debug", "#DB"),
    Stats::new("Non-maskable Interrupt", "#NM"),
    Stats::new("Breakpoint", "#BP"),
    Stats::new("Overflow", "#OF"),
    Stats::new("Bound Range Exceeded", "#BR"),
    Stats::new("Invalid Opcode", "#UD"),
    Stats::new("Device Not Available", "#NA"),
    Stats::new("Double Fault", "#DF"),
    Stats::new("Coprocessor Segment Overrun", "#CS"),
    Stats::new("Invalid TSS", "#TS"),
    Stats::new("Segment Not Present", "#NP"),
    Stats::new("Stack-Segment Fault", "#SS"),
    Stats::new("General Protection Fault", "#GP"),
    Stats::new("Page Fault", "#PF"),
    Stats::new("Reserved 0x0F", "#0F"),
    Stats::new("x87 Floating-Point Exception", "#MF"),
    Stats::new("Alignment Check", "#AC"),
    Stats::new("Machine Check", "#MC"),
    Stats::new("SIMD Floating-Point Exception", "#XF"),
    Stats::new("Virtualization Exception", "#VE"),
    Stats::new("Reserved 0x15", "#15"),
    Stats::new("Reserved 0x16", "#16"),
    Stats::new("Reserved 0x17", "#17"),
    Stats::new("Reserved 0x18", "#18"),
    Stats::new("Reserved 0x19", "#19"),
    Stats::new("Reserved 0x1A", "#1A"),
    Stats::new("Reserved 0x1B", "#1B"),
    Stats::new("Reserved 0x1C", "#1C"),
    Stats::new("Reserved 0x1D", "#1D"),
    Stats::new("Security Exception", "#SX"),
    Stats::new("Reserved 0x1F", "#1F"),
    Stats::new("PIT", "#IT"),
    Stats::new("Keyboard", "#KB"),
    Stats::new("Cascade", "#CA"),
    Stats::new("COM2", "#C2"),
    Stats::new("COM1", "#C1"),
    Stats::new("LPT2", "#L2"),
    Stats::new("Floppy Disk", "#FD"),
    Stats::new("LPT1", "#L1"),
    Stats::new("RTC", "#RT"),
    Stats::new("Free 0x29", "#29"),
    Stats::new("Free 0x2A", "#2A"),
    Stats::new("Free 0x2B", "#2B"),
    Stats::new("PS2 Mouse", "#MS"),
    Stats::new("Coprocessor", "#CP"),
    Stats::new("Primary ATA Hard Disk", "#PD"),
    Stats::new("Secondary ATA Hard Disk", "#SD"),
    Stats::new("Timer", "#TI"),
    Stats::new("Spurious", "#SP"),
];
