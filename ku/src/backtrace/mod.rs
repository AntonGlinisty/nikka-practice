/// Отладочная информация [`Callsite`] о точке вызова некоторой функции.
pub mod callsite;

use core::{
    arch::asm,
    fmt,
    mem,
    num::NonZeroUsize,
};

use crate::{
    error::Result,
    memory::{
        Block,
        Virt,
        addr::{
            Tag,
            VirtTag,
        },
    },
};

pub use callsite::Callsite;

/// Поддержка печати трассировок стека.
///
/// Трассировку стека можно расшифровать командой `llvm-symbolizer`, например
/// ```console
/// ~/nikka$ cd kernel
/// ~/nikka/kernel$ cargo test --test 6-um-3-eager-fork
/// ...
/// 23:02:48.579 0 E panicked at 'I did my best, it wasn't much', user/eager_fork/src/main.rs:47:5; backtrace = 0x10008593 0x10008479 0x100085B8 0x10008CD9; pid = 2:1
/// ~/nikka/kernel$ echo '0x10008593 0x10008479 0x100085B8 0x10008CD9' | tr ' ' '\n' | llvm-symbolizer --exe ../target/kernel/user/eager_fork
/// eager_fork::fork_tree::h6af1cf920637510d
/// .../nikka/user/eager_fork/src/main.rs:47:5
///
/// eager_fork::main::h7eeb3104124da585
/// .../nikka/user/eager_fork/src/main.rs:40:1
///
/// main
/// .../nikka/user/lib/src/lib.rs:88:10
///
/// _start
/// .../nikka/user/lib/src/lib.rs:60:19
/// ```
///
/// Требует от компилятора
///   - генерации указателей фреймов (force-frame-pointers=yes) и
///   - расположения кода по фиксированным адресам (relocation-model=dynamic-no-pic).
///
/// Для этого нужно добавить в `.cargo/config.toml` опции
/// ```toml
/// [build]
///     rustflags = [
///         "--codegen", "force-frame-pointers=yes",
///         "--codegen", "relocation-model=dynamic-no-pic",
///     ]
/// ```
#[derive(Clone, Copy)]
pub struct Backtrace {
    /// Адрес, ниже которого не может быть расположен внешний фрейм, --- стек растёт вниз.
    /// Снижает вероятность некорректного обращения к памяти
    /// при поиске конца списка стековых фреймов.
    lower_limit: Virt,

    /// Стек, знание которого позволяет найти конец списка стековых фреймов.
    /// Радикально снижает вероятность некорректного обращения к памяти
    /// при поиске конца списка стековых фреймов.
    /// Если не известен, устанавливается равным одной странице памяти,
    /// в которую указывает регистр `RBP`.
    stack: Block<Virt>,

    /// Текущий стековый фрейм.
    stack_frame: StackFrame,
}

impl Backtrace {
    /// Возвращает трассировку текущего стека.
    ///
    /// Мы указываем компилятору выполнить встраивание этой функции,
    /// чтобы не порождать дополнительный стековый фрейм под её вызов и не захламлять трассировку стека.
    /// В результате, функция вызвавшая [`Backtrace::current()`], в него не попадёт.
    #[inline(always)]
    pub fn current() -> Result<Self> {
        let lower_limit: usize;
        unsafe {
            asm!(
                "mov {0}, rsp",
                out(reg) lower_limit,
                options(nostack, nomem),
            );
        }

        let stack_frame = rbp();
        let stack = Block::from_index(stack_frame, stack_frame + 1)?.enclosing().into();

        Ok(Self {
            lower_limit: Virt::new(lower_limit)?,
            stack,
            stack_frame: StackFrame::current()?,
        })
    }

    /// Возвращает трассировку текущего стека.
    ///
    /// `stack` --- текущий стек, знание которого позволяет найти конец списка стековых фреймов.
    ///
    /// Мы указываем компилятору выполнить встраивание этой функции,
    /// чтобы не порождать дополнительный стековый фрейм под её вызов и не захламлять трассировку стека.
    /// В результате, функция вызвавшая [`Backtrace::with_stack()`], в него не попадёт.
    #[inline(always)]
    pub fn with_stack(stack: Block<Virt>) -> Result<Self> {
        let mut backtrace = Self::current()?;
        backtrace.stack = stack;
        Ok(backtrace)
    }
}

impl Iterator for Backtrace {
    type Item = StackFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.stack_frame.outer(&mut self.lower_limit, self.stack)?;

        Some(mem::replace(&mut self.stack_frame, next))
    }
}

impl fmt::Debug for Backtrace {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "Backtrace:")?;

        for stack_frame in *self {
            write!(formatter, "\n  {stack_frame}")?;
        }

        Ok(())
    }
}

impl fmt::Display for Backtrace {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        let mut separator = "";

        write!(formatter, "[")?;

        for stack_frame in *self {
            write!(formatter, "{separator}{stack_frame}")?;
            separator = " ";
        }

        write!(formatter, "]")
    }
}

/// Узел списка стековых фреймов.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct StackFrame {
    /// Адрес внешнего стекового фрейма --- фрейма вызвавшей функции.
    outer: Option<NonZeroUsize>,

    /// Адрес возврата в вызвавшую функцию.
    return_address: Option<NonZeroUsize>,
}

impl StackFrame {
    /// Возвращает текущий стековый фрейм.
    ///
    /// Мы указываем компилятору выполнить встраивание этой функции,
    /// чтобы не порождать дополнительный стековый фрейм под её вызов и не захламлять трассировку стека.
    #[inline(always)]
    pub fn current() -> Result<Self> {
        let stack_frame = rbp();
        Self::validate(stack_frame)
    }

    /// Адрес возврата в вызвавшую функцию.
    pub fn return_address(&self) -> Option<Virt> {
        Some(Virt::new(self.return_address?.get()).expect("incorrect stack frame"))
    }

    /// Возвращает внешний стековый фрейма или `None`, если текущий фрейм самый внешний.
    fn outer(
        &self,
        lower_limit: &mut Virt,
        stack: Block<Virt>,
    ) -> Option<Self> {
        let outer_address = self.outer?.get();
        let outer_start = Virt::new(outer_address).expect("incorrect stack frame");

        if !VirtTag::is_same_half(outer_start, *lower_limit) || outer_start < *lower_limit {
            return None;
        }

        let outer_end = (outer_start + mem::size_of::<StackFrame>()).ok()?;
        let outer = Block::new(outer_start, outer_end).ok()?;

        if let Ok(new_lower_limit) = outer.start_address() + mem::size_of::<StackFrame>() {
            *lower_limit = new_lower_limit;
        }

        if stack.contains_block(outer) {
            Self::validate(outer_address).ok()
        } else {
            None
        }
    }

    /// Проверяет, что `stack_frame` является валидным адресом стекового фрейма и
    /// по нему расположен валидный стековый фрейм.
    /// В случае успеха возвращает копию этого стекового фрейма.
    ///
    /// # Note
    ///
    /// В случае успешного результата нет абсолютной гарантии,
    /// что по адресу `stack_frame` расположен именно стековый фрейм.
    /// Так как выполняемые проверки могут пройти успешно случайно,
    /// если по адресу `stack_frame` находится не стековый фрейм,
    /// а удовлетворяющий критериям проверок мусор.
    fn validate(stack_frame: usize) -> Result<Self> {
        let stack_frame = unsafe { *Virt::new(stack_frame)?.try_into_ref::<Self>()? };

        if let Some(outer) = stack_frame.outer {
            Virt::new(outer.get())?.try_into_ptr::<Self>()?;
        }
        if let Some(return_address) = stack_frame.return_address {
            Virt::new(return_address.get())?;
        }

        Ok(stack_frame)
    }
}

impl fmt::Display for StackFrame {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{:#X}",
            self.return_address.map(NonZeroUsize::get).unwrap_or(0),
        )
    }
}

/// Возвращает содержимое регистра `RBP`.
///
/// Мы указываем компилятору выполнить встраивание этой функции,
/// чтобы не порождать дополнительный стековый фрейм под её вызов и не захламлять трассировку стека.
#[inline(always)]
fn rbp() -> usize {
    let rbp: usize;
    unsafe {
        asm!(
            "mov {0}, rbp",
            out(reg) rbp,
            options(nostack, nomem),
        );
    }

    rbp
}
