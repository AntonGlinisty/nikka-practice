use alloc::{
    vec,
    vec::Vec,
};
use core::{
    arch,
    arch::{
        asm,
        naked_asm,
    },
    fmt,
    mem,
};

use chrono::Duration;
use memoffset::offset_of;
use static_assertions::const_assert;
use x86::msr::IA32_EFER;
use x86_64::{
    instructions::interrupts,
    registers::{
        control::{
            Cr0,
            Cr4,
        },
        model_specific::Efer,
    },
    structures::gdt::SegmentSelector,
};

#[cfg(not(feature = "conservative-backtraces"))]
use sentinel_frame::with_sentinel_frame;

use crate::{
    error::Result,
    log::{
        debug,
        error,
        info,
        warn,
    },
    memory::{
        BASE_ADDRESS_SPACE,
        Block,
        Frame,
        GDT,
        KERNEL_RW,
        KERNEL_RX,
        Page,
        Phys,
        Phys2Virt,
        RealModePseudoDescriptor,
        SmallGdt,
        Translate,
        Virt,
        mmu::PageTableFlags,
        size,
    },
    process::{
        Scheduler,
        syscall,
    },
    time,
    trap::IDT,
};

use super::{
    LocalApic,
    cpu::Cpu,
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Запуск Application Processor с Bootstrap Processor.
/// Аргумент [`phys2virt`][Phys2Virt] описывает линейное отображение
/// физической памяти в виртуальную внутри этого страничного отображения.
/// Аргумент `cpu` задаёт структуру [`Cpu`] для запускаемого Application Processor.
///
/// MultiProcessor Specification 4.1 part B.4 "Application Processor Startup"
pub(super) fn boot_ap(
    phys2virt: Phys2Virt,
    cpu: &mut Cpu,
) -> Result<()> {
    let cpu_id = cpu.id();
    let code_size = switch_mode_code_size();
    let code_size_is_unrealistic = code_size < 8;
    if code_size_is_unrealistic {
        debug!(cpu = %cpu_id, %code_size, "AP init does not seem to be implemented, skipping it");
        return Ok(());
    }

    let timer = time::timer();

    let boot_memory = prepare_boot_code(phys2virt, cpu)?;

    LocalApic::send_init(cpu_id, boot_memory.code_frame())?;

    let message = "boot Application Processor";

    if let Err(error) = cpu.wait_initialized(Duration::seconds(1), boot_memory) {
        error!(cpu = %cpu_id, duration = %timer.elapsed(), ?error, message);
    } else {
        info!(cpu = %cpu_id, duration = %timer.elapsed(), message);
    }

    Ok(())
}

/// Подготовка кода и стека начальной инициализации Application Processor.
/// Возвращает физический адрес, куда сохранён код и стек инициализации.
/// А также гард, который при своём удалении восстановит исходное содержимое этой памяти.
/// Аргумент [`phys2virt`][Phys2Virt] описывает линейное отображение
/// физической памяти в виртуальную внутри этого страничного отображения.
/// Аргумент `cpu` задаёт структуру [`Cpu`] для запускаемого Application Processor.
/// Через [`Cpu::initialized`] запускаемый Application Processor сигнализирует
/// запускающему Bootstrap Processor, что инициализация AP завершена.
fn prepare_boot_code(
    phys2virt: Phys2Virt,
    cpu: &mut Cpu,
) -> Result<BootMemory> {
    let boot_memory = BootMemory::new(phys2virt)?;

    copy_switch_mode_code(boot_memory.code_data())?;

    let boot_stack_phys = real_mode_address(BOOT_STACK, "boot stack");
    let boot_stack = BootStack::new(boot_stack_phys, cpu)?;

    debug!(?boot_stack);

    unsafe {
        phys2virt
            .map(boot_stack_phys)?
            .try_into_mut_ptr::<BootStack>()
            .expect("BOOT_STACK address is not suitable for BootStack type")
            .write_volatile(boot_stack);
        arch::x86_64::_mm_mfence();
    }

    Ok(boot_memory)
}

/// Адрес, куда релоцируется код функции [`switch_from_real_mode_to_long_mode()`].
/// Чтобы он был доступен из реального режима работы, в котором стартует Application Processor.
const BOOT_CODE: usize = 7 * Page::SIZE;

/// Адрес структуры [`BootStack`] с параметрами для кода инициализации
/// [`switch_from_real_mode_to_long_mode()`].
const BOOT_STACK: usize = BOOT_CODE + BOOT_CODE_PLUS_STACK_SIZE - mem::size_of::<BootStack>();

/// Размер в памяти, который должны занимать код инициализации
/// [`switch_from_real_mode_to_long_mode()`] и необходимые ему параметры [`BootStack`].
const BOOT_CODE_PLUS_STACK_SIZE: usize = Page::SIZE;

/// Стек с дополнительной информацией для загрузки Application Processor.
///
/// Через этот стек в [`switch_from_real_mode_to_long_mode()`] передаётся дополнительная информация,
/// необходимая для загрузки AP.
/// Например, дескриптор описывающий GDT, который можно передать инструкции `lgdt` как есть.
/// Или адрес корневой таблицы страниц, который можно как есть записать в `CR3`.
/// Это сделано, чтобы:
///   - Не нужно было в функции [`switch_from_real_mode_to_long_mode()`]
///     на 16-битном ассемблере строить GDT, таблицы страниц и т.д.
///     То есть для максимального упрощения ассемблерной части кода.
///   - Содержимое системных регистров процессора на всех AP совпадало с аналогичными регистрами BSP.
///     То есть, чтобы можно было один раз правильно настроить системные регистры BSP,
///     и автоматически получить правильные и согласованные по системе в целом
///     настройки всех процессоров.
///
/// Поля в [`BootStack`] организованы в удобном для
/// [`switch_from_real_mode_to_long_mode()`] порядке.
/// В частности, поля с селектором кода [`BootStack::kernel_code`] и релоцированным адресом
/// [`BootStack::set_cs_rip_to_64bit`] метки `set_cs_rip_to_64bit:` лежат так, что образуют
/// [far pointer](https://en.wikipedia.org/wiki/Far_pointer) `kernel_code:set_cs_rip_to_64bit`,
/// который может быть использован инструкцией `far ret` как есть.
#[repr(C)]
struct BootStack {
    /// <https://wiki.osdev.org/CPU_Registers_x86-64#IA32_EFER>
    efer: u32,

    /// <https://wiki.osdev.org/CPU_Registers_x86-64#CR4>
    cr4: u32,

    /// <https://wiki.osdev.org/CPU_Registers_x86-64#CR3>
    cr3: u32,

    /// <https://wiki.osdev.org/CPU_Registers_x86-64#CR0>
    cr0: u32,

    /// Дескриптор, описывающий адрес и размер промежуточной
    /// [Global Descriptor Table](https://en.wikipedia.org/wiki/Global_Descriptor_Table) (GDT),
    /// которую использует ассемблерный код.
    /// Сама промежуточная GDT хранится тут же, в поле [`BootStack::gdt`].
    gdt_pseudo_descriptor: RealModePseudoDescriptor,

    /// Селектор сегмента данных ядра.
    kernel_data: SegmentSelector,

    /// Релоцированный адрес метки `set_cs_rip_to_64bit:`.
    ///
    /// [`BootStack::set_cs_rip_to_64bit`] и [`BootStack::kernel_code`] лежат так, что образуют
    /// [far pointer](https://en.wikipedia.org/wiki/Far_pointer)
    /// `kernel_code:set_cs_rip_to_64bit`,
    /// который может быть использован инструкцией `retf` как есть.
    set_cs_rip_to_64bit: u16,

    /// Селектор сегмента кода ядра.
    kernel_code: SegmentSelector,

    /// Выравнивание для следующего поля --- [`BootStack::cpu`].
    _padding: u32,

    /// Указатель на структуру [`Cpu`] запускаемого Application Processor.
    cpu: Virt,

    /// Стек ядра для запускаемого Application Processor.
    kernel_stack: Virt,

    /// Промежуточная
    /// [Global Descriptor Table](https://en.wikipedia.org/wiki/Global_Descriptor_Table) (GDT),
    /// которую использует ассемблерный код.
    gdt: SmallGdt,
}

const_assert!(offset_of!(BootStack, cpu).is_multiple_of(mem::size_of::<Virt>()));

impl BootStack {
    /// Подготавливает временный стек для функции [`switch_from_real_mode_to_long_mode()`].
    /// Аргумент `boot_stack_phys` указывает физический адрес, куда будет помещён этот стек.
    /// Аргумент `cpu` задаёт структуру [`Cpu`] для запускаемого Application Processor.
    /// Через [`Cpu::initialized`] запускаемый Application Processor сигнализирует
    /// запускающему Bootstrap Processor, что инициализация AP завершена.
    fn new(
        boot_stack_phys: Phys,
        cpu: &mut Cpu,
    ) -> Result<Self> {
        let switch_mode_start: usize;
        let set_cs_rip_to_64bit: usize;
        unsafe {
            asm!(
                "
                mov {switch_mode_start}, OFFSET switch_mode_start
                mov {set_cs_rip_to_64bit}, OFFSET set_cs_rip_to_64bit
                ",
                switch_mode_start = out(reg) switch_mode_start,
                set_cs_rip_to_64bit = out(reg) set_cs_rip_to_64bit,
            )
        }

        let efer = Efer::read().bits().try_into()?;
        let cr4 = Cr4::read_raw().try_into()?;
        let cr3 = BASE_ADDRESS_SPACE.lock().page_table_root().address().try_into()?;
        let cr0 = Cr0::read_raw().try_into()?;
        let gdt = SmallGdt::new();
        let gdt_pseudo_descriptor =
            SmallGdt::real_mode_pseudo_descriptor((boot_stack_phys + offset_of!(BootStack, gdt))?)?;
        let set_cs_rip_to_64bit: u16 =
            size::into_u64(BOOT_CODE + (set_cs_rip_to_64bit - switch_mode_start))
                .try_into()
                .expect(
                    "set_cs_rip_to_64bit label address should fit into 16 bit for Application \
                     Processors to be able to ret to it",
                );

        let kernel_stack = cpu.kernel_stack().pointer();

        Ok(Self {
            efer,
            cr4,
            cr3,
            cr0,
            gdt_pseudo_descriptor,
            kernel_data: SmallGdt::kernel_data(),
            set_cs_rip_to_64bit,
            kernel_code: SmallGdt::kernel_code(),
            _padding: 0,
            cpu: Virt::from_ref(cpu),
            kernel_stack,
            gdt,
        })
    }
}

impl fmt::Debug for BootStack {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            concat!(
                "{{ ",
                "efer: {:#X}, cr4: {:#X}, cr3: {:#X}, cr0: {:#X}, ",
                "gdt: {:?}, kernel_code: {:?}, kernel_data: {:?}, set_cs_rip_to_64bit: 0p{:04X}, ",
                "kernel_stack: {}, cpu: {}",
                "}}",
            ),
            self.efer,
            self.cr4,
            self.cr3,
            self.cr0,
            self.gdt_pseudo_descriptor,
            self.kernel_code,
            self.kernel_data,
            self.set_cs_rip_to_64bit,
            self.kernel_stack,
            self.cpu,
        )
    }
}

/// Переключает Application Processor из [реального режима](https://en.wikipedia.org/wiki/Real_mode) в [64-битный режим](https://en.wikipedia.org/wiki/Long_mode) и выполняет начальную инициализацию процессора.
///
/// Переключение режима происходит напрямую, минуя [32-х битный защищённый режим](https://en.wikipedia.org/wiki/Protected_mode).
/// Все нужные структуры и значения регистров копируются с BSP.
#[allow(named_asm_labels)]
#[cold]
#[unsafe(link_section = ".switch_from_real_mode_to_long_mode")]
#[unsafe(naked)]
extern "C" fn switch_from_real_mode_to_long_mode() -> ! {
    naked_asm!(
        "
        .code16

        switch_mode_start:

            // Ваш код для 16-битного режима работы процессора.
            // Он должен использовать только 16-битные и 32-битные регистры ---
            // AX, EAX, SP, ESP, DS, SS, ..., CR*.
            // 64-битные регистры --- RAX--R15 пока недоступны.
            // Используйте только инструкции ассемблера,
            // макрокоманды вроде .code16 и .code64 не добавляйте и не двигайте.

            // TODO: your code here.

        .code64

            // Не пишите тут свой код, скорее всего это будет неверно.

            // Perform a far return to `set_cs_rip_to_64bit` in order to set `CS:RIP` right.
            retf

        set_cs_rip_to_64bit:

            // Теперь процессор полностью настроен для 64-битного режима и работает в нём.
            // Можно использовать 64-битные регистры RAX--R15.
            // Используйте только инструкции ассемблера,
            // макрокоманды вроде .code16 и .code64 не добавляйте и не двигайте.

            // TODO: your code here.

            // Здесь нужно перейти на `ap_kernel_main()` по абсолютному адресу.
            // Так как этот код будет перемещён в другие адреса памяти, а `ap_kernel_main()` нет,
            // то разность адресов этой команды и `ap_kernel_main()` поменяется.
            // И обычные `jmp ap_kernel_main` и `call ap_kernel_main`,
            // работающие с относительными адресами,
            // после перемещения кода содержали бы неверные смещения.
            // Можно воспользоваться косвенным переходом на абсолютный адрес, записанный в регистр.
            mov rax, OFFSET {ap_kernel_main}
            jmp rax

            // Метки `switch_mode_start` и `switch_mode_end` отмечают границы
            // загрузочного кода, который должен быть скопирован.
            // Поэтому пишите код загрузки AP строго внутри них.
        switch_mode_end:
        ",

        // TODO: your code here.
        ap_kernel_main = sym ap_kernel_main,
    );
}

/// Завершает инициализацию Application Processor.
/// Аргумент `cpu` задаёт структуру [`Cpu`] для запускаемого Application Processor.
/// Через [`Cpu::initialized`] запускаемый Application Processor сигнализирует
/// запускающему Bootstrap Processor, что инициализация AP завершена.
#[cfg_attr(not(feature = "conservative-backtraces"), with_sentinel_frame)]
#[cold]
#[inline(never)]
extern "C" fn ap_kernel_main(cpu: &mut Cpu, // rdi
) -> ! {
    GDT.lock().load();

    if let Err(error) = LocalApic::init() {
        warn!(?error, "failed to init local APIC");
    }

    cpu.set_gs();
    cpu.set_tss();

    IDT.load();
    interrupts::enable();

    syscall::init();

    cpu.signal_initialized();

    info!(cpu = %cpu.id(), "report for duty");

    Scheduler::run();
}

/// Гард, который:
///   - При создании запоминает данные, записанные в блоке [`BootMemory::data`].
///   - При своём удалении восстановит исходное содержимое в этом блоке памяти.
pub(super) struct BootMemory {
    /// Физический адрес кода загрузки Application Processor.
    code_frame: Frame,

    /// Блок памяти, из которого будет исполняться код загрузки.
    /// Гарантированно отображается с доступом на исполнение
    /// перед загрузкой Application Processor.
    code_pages: Block<Page>,

    /// Блок памяти, содержимое которого нужно сохранить и в последствии восстановить.
    /// Соответствует блоку [`BootMemory::code_pages`], но гарантированно отображён с
    /// доступом на запись.
    /// Чтобы одни и те же страницы не были отображены одновременно на запись и на исполнение,
    /// имеет другой виртуальный адрес, чем [`BootMemory::code_pages`].
    /// Но отображена на те же физические фреймы.
    data: &'static mut [usize],

    /// Оригинальные данные блока [`BootMemory::data`],
    /// которые содержались в нём до записи в него кода загрузки Application Processor.
    /// Их нужно будет восстановить после загрузки Application Processor.
    original_data: &'static mut [usize],

    /// Оригинальные флаги страниц блока [`BootMemory::code_pages`],
    /// с которыми он был отображён до записи в него кода загрузки Application Processor.
    /// Их нужно будет восстановить после загрузки Application Processor.
    original_flags: Vec<PageTableFlags>,
}

impl BootMemory {
    /// Возвращает гард [`BootMemory`], который:
    ///   - При создании запоминает данные, записанные в блоке [`BootMemory::data`].
    ///   - Изменяет флаги этого блока так, чтобы можно было исполнять код в нём.
    ///   - При своём удалении восстановит исходное содержимое в этом блоке памяти и
    ///     его исходные флаги.
    fn new(phys2virt: Phys2Virt) -> Result<Self> {
        let message = "invalid boot code_pages block";

        let code_start = Virt::new(BOOT_CODE).expect(message);
        let code_pages = Self::code_block(code_start).expect(message);
        let mut original_flags = vec![PageTableFlags::default(); code_pages.count()];

        let mut address_space = BASE_ADDRESS_SPACE.lock();

        for (flags, page) in original_flags.iter_mut().zip(code_pages) {
            *flags = address_space.path(page.address()).flags()?;
        }
        unsafe {
            address_space.remap_block(code_pages, KERNEL_RX)?;
        }

        let code_frame = Frame::new(real_mode_address(BOOT_CODE, "boot code_pages"))
            .expect("boot code should be aligned on frame boundary");
        let data = unsafe {
            Self::code_block(phys2virt.map(code_frame.address())?)
                .expect(message)
                .try_into_mut_slice()?
        };

        let original_data = unsafe { address_space.map_slice_zeroed(data.len(), KERNEL_RW)? };

        original_data.clone_from_slice(data);

        Ok(Self {
            code_frame,
            code_pages,
            data,
            original_data,
            original_flags,
        })
    }

    /// Возвращает блок страниц, содержащих адреса с `virt` до `virt + BOOT_CODE_PLUS_STACK_SIZE`.
    fn code_block(virt: Virt) -> Result<Block<Page>> {
        Ok(Block::new(virt, (virt + BOOT_CODE_PLUS_STACK_SIZE)?)?.enclosing())
    }

    /// Возвращает виртуальный адрес кода загрузки Application Processor.
    /// Гарантируется, что по этому адресу можно записывать данные.
    fn code_data(&self) -> Virt {
        Virt::from_ref(self.data)
    }

    /// Возвращает физический адрес кода загрузки Application Processor.
    fn code_frame(&self) -> Frame {
        self.code_frame
    }
}

impl Drop for BootMemory {
    fn drop(&mut self) {
        let mut address_space = BASE_ADDRESS_SPACE.lock();

        for (flags, page) in self.original_flags.iter().zip(self.code_pages) {
            address_space
                .path(page.address())
                .set_flags(*flags)
                .expect("the page should be mapped");
        }

        self.data.clone_from_slice(self.original_data);

        unsafe {
            address_space
                .unmap_slice(self.original_data)
                .expect("the block should be mapped in new");
        }
    }
}

/// Релоцирует код начальной загрузки Application Processor,
/// расположенный в функции [`switch_from_real_mode_to_long_mode()`]
/// в заданный адрес `boot_code_address`.
fn copy_switch_mode_code(boot_code_address: Virt) -> Result<()> {
    let switch_mode_code_address = Virt::from_ptr(switch_from_real_mode_to_long_mode as *const ());
    let switch_mode_code_slice =
        unsafe { switch_mode_code_address.try_into_mut_slice::<u8>(switch_mode_code_size())? };
    let boot_code_slice =
        unsafe { boot_code_address.try_into_mut_slice::<u8>(switch_mode_code_slice.len())? };

    boot_code_slice.clone_from_slice(switch_mode_code_slice);

    Ok(())
}

/// Вычисляет размер кода начальной загрузки Application Processor,
/// расположенного в функции [`switch_from_real_mode_to_long_mode()`].
fn switch_mode_code_size() -> usize {
    let switch_mode_start: usize;
    let switch_mode_end: usize;
    unsafe {
        asm!(
            "
            mov {switch_mode_start}, OFFSET switch_mode_start
            mov {switch_mode_end}, OFFSET switch_mode_end
            ",
            switch_mode_start = out(reg) switch_mode_start,
            switch_mode_end = out(reg) switch_mode_end,
        )
    }

    let switch_mode_code_size = switch_mode_end - switch_mode_start;
    assert!(switch_mode_code_size + mem::size_of::<BootStack>() <= BOOT_CODE_PLUS_STACK_SIZE);

    switch_mode_code_size
}

/// Преобразует заданный физический адрес `address` в [`Phys`].
///
/// # Panics
///
/// Паникует, если адрес не влезает в 32 бита и потому не будет доступен в 32-битном режиме.
/// Добавляет `what` в сообщение паники.
fn real_mode_address(
    address: usize,
    what: &'static str,
) -> Phys {
    let address = address.try_into().unwrap_or_else(|_| {
        panic!(
            "{} address {:#X} should fit into 32 bit for Application Processors to be able to \
             boot from it",
            what, address,
        )
    });

    Phys::new_u32(address)
}
