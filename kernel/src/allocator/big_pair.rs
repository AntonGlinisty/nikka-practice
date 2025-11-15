use duplicate::duplicate_item;

use ku::{
    allocator::{
        BigAllocator,
        BigAllocatorPair,
    },
    error::{
        Error::InvalidArgument,
        Result,
    },
    memory::{
        Block,
        Page,
        mmu::PageTableFlags,
    },
};

use crate::memory::{
    AddressSpace,
    FrameGuard,
    Translate,
};

/// Пара аллокаторов, связанная с одним или с двумя разными адресными пространствами.
/// Реализует типаж [`BigAllocatorPair`].
/// В частности, позволяет скопировать
/// отображение страниц из текущего адресного пространства [`BigPair::src()`]
/// в целевое адресное пространство [`BigPair::dst()`] методом [`BigPair::copy_mapping()`].
///
/// Требуется, когда нужно использовать [`BigAllocator`]
/// либо для текущего адресного пространства, в котором работает код,
/// либо вообще говоря для другого адресного пространства.
///
/// Например:
///   - Из одного адресного пространства загрузить в другое код процесса.
///   - Из одного адресного пространства создать разделяемую
///     с другим адресным пространством область памяти.
pub(crate) struct BigPair<'a> {
    /// Одно или два разных адресных пространства.
    address_spaces: AddressSpacePair<'a>,

    /// Флаги доступа к памяти, выделяемой аллокатором [`BigPair::dst()`].
    dst_flags: PageTableFlags,

    /// Флаги доступа к памяти, выделяемой аллокатором [`BigPair::src()`].
    src_flags: PageTableFlags,
}

impl<'a> BigPair<'a> {
    /// Создаёт пару [`BigAllocator`], связанную с двумя разными адресными пространствами.
    /// Параметры `src` и `src_flags` задают текущее адресное пространство и
    /// флаги по умолчанию в нём.
    /// Параметры `dst` и `dst_flags` задают второе адресное пространство и
    /// флаги по умолчанию в нём.
    #[allow(unused)]
    pub(crate) fn new_pair(
        src: &'a mut AddressSpace,
        src_flags: PageTableFlags,
        dst: &'a mut AddressSpace,
        dst_flags: PageTableFlags,
    ) -> Self {
        Self {
            address_spaces: AddressSpacePair::Different { dst, src },
            dst_flags,
            src_flags,
        }
    }

    /// Создаёт пару [`BigAllocator`], связанную с одним адресным пространством.
    /// Параметр `src_dst` задаёт адресное пространство.
    /// Параметры `src_flags` и `dst_flags` задают флаги доступа
    /// при работе с ним как с текущим и как с целевым соответственно.
    pub(crate) fn new_single(
        src_dst: &'a mut AddressSpace,
        src_flags: PageTableFlags,
        dst_flags: PageTableFlags,
    ) -> Self {
        Self {
            address_spaces: AddressSpacePair::Same { src_dst },
            dst_flags,
            src_flags,
        }
    }

    /// Возвращает целевое адресное пространство.
    fn dst_address_space(&mut self) -> &mut AddressSpace {
        match &mut self.address_spaces {
            AddressSpacePair::Different { dst, .. } => dst,
            AddressSpacePair::Same { src_dst } => src_dst,
        }
    }

    /// Возвращает текущее адресное пространство.
    fn src_address_space(&mut self) -> &mut AddressSpace {
        match &mut self.address_spaces {
            AddressSpacePair::Different { src, .. } => src,
            AddressSpacePair::Same { src_dst } => src_dst,
        }
    }
}

impl BigAllocatorPair for BigPair<'_> {
    #[duplicate_item(
        accessor address_space_accessor flags;
        [dst] [dst_address_space] [dst_flags];
        [src] [src_address_space] [src_flags];
    )]
    fn accessor(&mut self) -> impl BigAllocator {
        let flags = self.flags;
        self.address_space_accessor().allocator(flags)
    }

    fn is_same(&self) -> bool {
        matches!(self.address_spaces, AddressSpacePair::Same { .. })
    }

    unsafe fn copy_mapping(
        &mut self,
        src_block: Block<Page>,
        dst_block: Block<Page>,
        flags: Option<PageTableFlags>,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }
}

/// Одно или пара двух разных адресных пространств.
enum AddressSpacePair<'a> {
    /// Пара двух разных адресных пространств.
    Different {
        /// Адресное пространство, над которым производятся действия.
        dst: &'a mut AddressSpace,

        /// Текущее адресное пространство, в котором работает код.
        src: &'a mut AddressSpace,
    },

    /// Одно адресное пространство.
    Same {
        /// Текущее адресное пространство, в котором работает код и
        /// над ним же он производит действия.
        src_dst: &'a mut AddressSpace,
    },
}
