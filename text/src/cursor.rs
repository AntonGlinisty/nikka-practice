use ku::memory::{
    IndexDataPair,
    IndexDataPortPair,
};

// Used in docs.
#[allow(unused)]
use super::Grid;

/// Типаж для управления курсором в текстовом режиме графического контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array).
pub trait Cursor {
    #[allow(rustdoc::private_intra_doc_links)]
    /// Возвращает позицию курсора.
    ///
    /// Эта позиция соответствует индексу в массиве [`Grid`].
    /// Так же как и этот массив, считается в позициях текстового режима с нуля,
    /// из левого верхнего угла по направлению слева направо.
    /// При достижении правой границы осуществляется переход в самую левую позицию следующей строки.
    fn get(&mut self) -> usize;

    #[allow(rustdoc::private_intra_doc_links)]
    /// Устанавливает позицию курсора.
    ///
    /// Эта позиция соответствует индексу в массиве [`Grid`].
    /// Так же как и этот массив, считается в позициях текстового режима с нуля,
    /// из левого верхнего угла по направлению слева направо.
    /// При достижении правой границы осуществляется переход в самую левую позицию следующей строки.
    ///
    /// # Panics
    ///
    /// Паникует, если `position` превышает максимальную возможную позицию курсора.
    fn set(
        &mut self,
        position: usize,
    );

    /// Включает или отключает отображение курсора.
    fn set_disable(
        &mut self,
        disable: bool,
    );

    #[allow(rustdoc::private_intra_doc_links)]
    /// Устанавливает размер курсора.
    /// Задавать можно только высоту курсора от `0` до [`MAX_LINES`] включительно,
    /// большие значения заменяются на `MAX_LINES`.
    fn set_height(
        &mut self,
        height: u8,
    );
}

/// Структура для управления курсором в текстовом режиме графического контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array).
///
/// Для тестов может быть создана поверх эмулируемых
/// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O),
/// см. [`IndexDataPair`].
pub struct VgaCursor<T: IndexDataPair<u8>>(T);

impl<T: IndexDataPair<u8>> VgaCursor<T> {
    /// Создаёт структуру для управления курсором в текстовом режиме графического контроллера
    /// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array).
    /// Аргумент `port_pair` пару индекс--данные
    /// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O).
    ///
    /// Для создания настоящего VGA-курсора предназначена дополнительная функция
    /// [`create_vga_cursor()`],
    /// которая вызывает [`VgaCursor::new()`] с аргументом, задающим правильные номера портов.
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

/// Создаёт структуру для управления курсором в текстовом режиме графического контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array).
pub(super) fn create_vga_cursor() -> VgaCursor<IndexDataPortPair<u8>> {
    const INDEX_PORT: u16 = 0x03D4;

    unsafe {
        VgaCursor::new(
            IndexDataPortPair::from_index_port(INDEX_PORT).expect("invalid VGA cursor index port"),
        )
    }
}

/// Вес бита выключения курсора в регистре [`START_LINE`] контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array).
pub(super) const CURSOR_DISABLE: u8 = 1 << 5;

/// Маска битов начальной и конечной линий курсора в регистрах [`START_LINE`] и [`END_LINE`]
/// контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array).
pub(super) const CURSOR_LINE_MASK: u8 = (1 << 5) - 1;

/// Индекс регистра контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array),
/// хранящий начальную линию текстового курсора.
/// Вместе с [`END_LINE`] задаёт размер курсора.
pub(super) const START_LINE: u8 = 0x0A;

/// Индекс регистра контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array),
/// хранящий начальную линию текстового курсора.
/// Вместе с [`START_LINE`] задаёт размер курсора.
pub(super) const END_LINE: u8 = 0x0B;

/// Максимальное количество горизонтальных линий текстового курсора контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array),
pub(super) const MAX_LINES: u8 = 1 << 4;

/// Индекс регистра контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array),
/// хранящего старший байт позиции текстового курсора.
/// Вместе с [`POSITION_LOW`] задаёт положение курсора на экране.
pub(super) const POSITION_HIGH: u8 = 0x0E;

/// Индекс регистра контроллера
/// [Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array),
/// хранящего младший байт позиции курсора на экране.
/// Вместе с [`POSITION_HIGH`] задаёт положение курсора на экране.
pub(super) const POSITION_LOW: u8 = 0x0F;
