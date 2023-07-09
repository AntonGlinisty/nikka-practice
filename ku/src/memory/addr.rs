use core::{
    fmt,
    marker::PhantomData,
    mem,
    ops::{
        Add,
        Sub,
    },
    slice,
};

use static_assertions::const_assert_eq;
use x86_64::addr::VirtAddr;

use crate::error::{
    Error::{
        InvalidAlignment,
        InvalidArgument,
        Null,
        Overflow,
    },
    Result,
};

use super::{
    mmu::{
        PAGE_OFFSET_BITS,
        PAGE_TABLE_INDEX_BITS,
        PAGE_TABLE_ROOT_LEVEL,
    },
    size::{
        self,
        SizeOf,
    },
};

// Used in docs.
#[allow(unused)]
use crate::{
    self as ku,
    error::Error,
};

/// Базовый тип для адресов [архитектуры x86-64](https://wiki.osdev.org/X86-64), как виртуальных, так и физических.
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Addr<T>(usize, PhantomData<T>);

const_assert_eq!(mem::size_of::<Addr<()>>(), mem::size_of::<usize>());
const_assert_eq!(mem::size_of::<Addr<()>>(), mem::size_of::<u64>());

impl<T: Tag> Addr<T> {
    /// Создаёт [`Addr`] --- [`Phys`] или [`Virt`] --- по его битовому представлению `addr`.
    ///
    /// Возвращает ошибку [`Error::InvalidArgument`] если битовое представление `addr`
    /// не является корректным для адреса целевого типа.
    pub fn new(addr: usize) -> Result<Self> {
        T::new(addr)
    }

    /// Создаёт [`Addr`] --- [`Phys`] или [`Virt`] --- по его битовому представлению `addr`.
    ///
    /// Возвращает ошибку [`Error::InvalidArgument`] если битовое представление `addr`
    /// не является корректным для адреса целевого типа.
    pub fn new_u64(addr: u64) -> Result<Self> {
        Self::new(size::from(addr))
    }

    /// Возвращает нулевой [`Addr`] --- [`Phys`] или [`Virt`].
    /// В отличие от `Addr::default()` доступна в константном контексте,
    /// поэтому не может использовать `T::new()`.
    pub const fn zero() -> Self {
        Self(0, PhantomData)
    }

    /// Возвращает битовое представление адреса.
    pub fn into_usize(self) -> usize {
        self.0
    }

    /// Возвращает битовое представление адреса.
    pub fn into_u64(self) -> u64 {
        size::into_u64(self.0)
    }

    /// Возвращает битовое представление адреса в виде целого типа,
    /// для которого есть [`TryFrom<u64>`], например в виде [`u32`].
    /// Возвращает ошибку [`Error::Int`], если адрес не помещается в выбранный тип.
    pub fn try_into<Q: TryFrom<u64>>(self) -> Result<Q>
    where
        Error: From<<Q as TryFrom<u64>>::Error>,
    {
        size::try_into(self.0)
    }
}

impl<T: Tag> Add<usize> for Addr<T> {
    type Output = Result<Self>;

    fn add(
        self,
        rhs: usize,
    ) -> Self::Output {
        self.0.checked_add(rhs).ok_or(Overflow).and_then(Self::new).and_then(|addr| {
            if T::is_same_half(self, addr) {
                Ok(addr)
            } else {
                Err(Overflow)
            }
        })
    }
}

impl<T: Tag> Sub<usize> for Addr<T> {
    type Output = Result<Self>;

    fn sub(
        self,
        rhs: usize,
    ) -> Self::Output {
        self.0.checked_sub(rhs).ok_or(Overflow).and_then(Self::new).and_then(|addr| {
            if T::is_same_half(self, addr) {
                Ok(addr)
            } else {
                Err(Overflow)
            }
        })
    }
}

impl<T: Tag> Sub<Self> for Addr<T> {
    type Output = Result<usize>;

    fn sub(
        self,
        rhs: Self,
    ) -> Self::Output {
        if T::is_same_half(self, rhs) {
            self.0.checked_sub(rhs.0).ok_or(Overflow)
        } else {
            Err(Overflow)
        }
    }
}

impl<T: Tag> fmt::Debug for Addr<T> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{}({}{:X})", T::ADDR_NAME, T::HEX_PREFIX, self.0)
    }
}

impl<T: Tag> fmt::Display for Addr<T> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{}{:X}", T::HEX_PREFIX, self.0)
    }
}

/// Тег, позволяющий различать
/// виртуальные адреса [`ku::memory::Virt`] и физические [`ku::memory::Phys`],
/// а также (виртуальные) страницы [`ku::memory::Page`] и (физические) фреймы [`ku::memory::Frame`].
pub trait Tag: Clone + Copy + Default + Eq + Ord {
    /// Создаёт [`Addr`] --- [`Phys`] или [`Virt`] --- по его битовому представлению `addr`.
    ///
    /// Возвращает ошибку [`Error::InvalidArgument`] если битовое представление `addr`
    /// не является корректным для адреса целевого типа.
    fn new(addr: usize) -> Result<Addr<Self>>;

    /// Возвращает `true` если:
    /// - Адреса `x` и `y` виртуальные и находятся в одной и той же
    /// [половине виртуального адресного пространства](https://en.wikipedia.org/wiki/X86-64#Virtual_address_space_details).
    /// - Адреса `x` и `y` физические.
    fn is_same_half(
        x: Addr<Self>,
        y: Addr<Self>,
    ) -> bool;

    /// Задаёт имя типа при печати:
    /// - `"Phys"` для физических адресов;
    /// - `"Virt"` для виртуальных адресов.
    const ADDR_NAME: &'static str;

    /// Задаёт имя типа при печати:
    /// - `"Frame"` для физических фреймов;
    /// - `"Page"` для виртуальных страниц.
    const FRAGE_NAME: &'static str;

    /// Заменяет префикс `0x` при печати на:
    /// - `0p` (**p**hysical) для физических адресов;
    /// - `0v` (**v**irtual) для виртуальных адресов.
    const HEX_PREFIX: &'static str;
}

/// Тег физических адресов [`ku::memory::Phys`] и фреймов [`ku::memory::Frame`].
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysTag;

impl Tag for PhysTag {
    fn new(addr: usize) -> Result<Phys> {
        Phys::new_impl(addr)
    }

    fn is_same_half(
        _x: Phys,
        _y: Phys,
    ) -> bool {
        true
    }

    const ADDR_NAME: &'static str = "Phys";
    const FRAGE_NAME: &'static str = "Frame";
    const HEX_PREFIX: &'static str = "0p";
}

/// Тег виртуальных адресов и страниц.
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct VirtTag;

impl Tag for VirtTag {
    fn new(addr: usize) -> Result<Virt> {
        Virt::new_impl(addr)
    }

    fn is_same_half(
        x: Virt,
        y: Virt,
    ) -> bool {
        x.is_higher_half() == y.is_higher_half()
    }

    const ADDR_NAME: &'static str = "Virt";
    const FRAGE_NAME: &'static str = "Page";
    const HEX_PREFIX: &'static str = "0v";
}

/// Используется для [`ku::memory::Block<Virt>`] и [`ku::memory::Block<Page>`].
#[doc(hidden)]
pub trait IsVirt {
    fn from_ptr<T: ?Sized>(ptr: *const T) -> Self;
    fn from_ref<T: ?Sized>(x: &T) -> Self;
    fn into_ptr_u8(self) -> *const u8;
    fn into_mut_ptr_u8(self) -> *mut u8;
    fn try_into_ptr<T>(self) -> Result<*const T>;
    fn try_into_mut_ptr<T>(self) -> Result<*mut T>;
    unsafe fn try_into_ref<'a, T>(self) -> Result<&'a T>;
    unsafe fn try_into_mut<'a, T>(self) -> Result<&'a mut T>;
    unsafe fn try_into_slice<'a, T>(
        self,
        len: usize,
    ) -> Result<&'a [T]>;
    unsafe fn try_into_mut_slice<'a, T>(
        self,
        len: usize,
    ) -> Result<&'a mut [T]>;
}

/// Физический адрес [архитектуры x86-64](https://wiki.osdev.org/X86-64).
///
/// # Examples
///
/// ## Преобразования между `Phys` и базовыми типами
/// Преобразования между [`Phys`] и [`usize`](https://doc.rust-lang.org/core/primitive.usize.html)/[`u64`](https://doc.rust-lang.org/core/primitive.u64.html).
/// ```rust
/// # use ku::{
/// #     error::Result,
/// #     memory::Phys,
/// # };
/// #
/// # fn f() -> Result<()> {
/// let phys = Phys::new(0x123ABC)?;
/// assert_eq!(phys, Phys::new_u64(0x123ABC)?);
/// assert_eq!(phys.into_u64(), 0x123ABC_u64);
/// assert_eq!(phys.into_usize(), 0x123ABC_usize);
/// # Ok(())
/// # }
/// #
/// # assert!(f().is_ok());
/// ```
///
/// ## Некорректные адреса
/// [`Phys::new()`] возвращает ошибку [`ku::error::Error::InvalidArgument`]
/// при попытке задать некорректный с точки зрения [архитектуры x86-64](https://wiki.osdev.org/X86-64) физический адрес.
/// ```rust
/// # use ku::{
/// #     error::Error::InvalidArgument,
/// #     memory::Phys,
/// # };
/// #
/// let bad_address = 1 << 63;
/// assert_eq!(Phys::new(bad_address), Err(InvalidArgument));
/// ```
pub type Phys = Addr<PhysTag>;

/// Виртуальный адрес [архитектуры x86-64](https://wiki.osdev.org/X86-64).
///
/// # Examples
///
/// ## Преобразования между `Virt` и базовыми типами
/// Преобразования между [`Virt`] и
/// [`usize`](https://doc.rust-lang.org/core/primitive.usize.html)/[`u64`](https://doc.rust-lang.org/core/primitive.u64.html).
/// ```rust
/// # use ku::{
/// #     error::Result,
/// #     memory::Virt,
/// # };
/// #
/// # fn f() -> Result<()> {
/// let virt = Virt::new(0x123ABC)?;
/// assert_eq!(virt, Virt::new_u64(0x123ABC)?);
/// assert_eq!(virt.into_u64(), 0x123ABC_u64);
/// assert_eq!(virt.into_usize(), 0x123ABC_usize);
/// # Ok(())
/// # }
/// #
/// # assert!(f().is_ok());
/// ```
///
/// ## Некорректные адреса
/// [`Virt::new()`] возвращает ошибку [`ku::error::Error::InvalidArgument`]
/// при попытке задать некорректный с точки зрения
/// [архитектуры x86-64](https://wiki.osdev.org/X86-64) виртуальный адрес.
/// ```rust
/// # use ku::{
/// #     error::Error::InvalidArgument,
/// #     memory::Virt,
/// # };
/// #
/// let bad_address = 1 << 63;
/// assert_eq!(Virt::new(bad_address), Err(InvalidArgument));
/// ```
///
/// ## Преобразования между `Virt` и `VirtAddr`
/// [`Virt`] поддерживает преобразования в/из [`x86_64::addr::VirtAddr`].
/// Это нужно для использования структур из [`x86_64`],
/// например [`x86_64::structures::tss::TaskStateSegment`] и
/// [`x86_64::structures::idt::Entry::set_handler_addr()`].
/// ```rust
/// # use x86_64::VirtAddr;
/// #
/// # use ku::{
/// #     error::Result,
/// #     memory::Virt,
/// # };
/// #
/// fn f(virt_addr: VirtAddr) -> Virt {
///     virt_addr.into()
/// }
///
/// # fn g() -> Result<()> {
/// let virt = Virt::new(0x123ABC)?;
/// let virt_addr: VirtAddr = virt.into();
/// let virt2: Virt = virt_addr.into();
/// assert_eq!(virt, virt2);
/// assert_eq!(virt, f(virt.into()));
/// # Ok(())
/// # }
/// #
/// # assert!(g().is_ok());
/// ```
pub type Virt = Addr<VirtTag>;

impl Phys {
    /// Создаёт физический адрес по его битовому представлению `addr`.
    ///
    /// Возвращает ошибку [`Error::InvalidArgument`] если битовое представление `addr`
    /// не является корректным для физического адреса.
    fn new_impl(addr: usize) -> Result<Self> {
        let zeros = addr.leading_zeros();
        if zeros >= Self::UNUSED_BITS {
            Ok(Self(addr, PhantomData))
        } else {
            Err(InvalidArgument)
        }
    }

    /// Количество используемых битов в физическом адресе.
    pub(super) const BITS: u32 = 52;

    /// Количество неиспользуемых битов в физическом адресе.
    const UNUSED_BITS: u32 = usize::BITS - Self::BITS;
}

impl Virt {
    /// Создаёт виртуальный адрес по его битовому представлению `addr`.
    ///
    /// Возвращает ошибку [`Error::InvalidArgument`] если битовое представление `addr`
    /// не является корректным для виртуального адреса.
    fn new_impl(addr: usize) -> Result<Self> {
        if addr.leading_zeros() > Self::UNUSED_BITS || addr.leading_ones() > Self::UNUSED_BITS {
            Ok(Self(addr, PhantomData))
        } else {
            Err(InvalidArgument)
        }
    }

    /// Возвращает `true` если виртуальный адрес относится к
    /// [верхней половине](https://en.wikipedia.org/wiki/X86-64#Virtual_address_space_details)
    /// адресного пространства.
    pub fn is_higher_half(&self) -> bool {
        self.into_usize() >> Self::BITS != 0
    }

    /// Преобразует указатель на `T` в [`Virt`].
    ///
    /// # Panics
    ///
    /// Паникует, если указатель не корректен.
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        const_assert_eq!(mem::size_of::<*const ()>(), mem::size_of::<usize>());
        Self::new(ptr as *const () as usize).expect("bad pointer")
    }

    /// Преобразует ссылку на `T` в [`Virt`].
    ///
    /// # Panics
    ///
    /// Паникует, если ссылка не корректна.
    pub fn from_ref<T: ?Sized>(x: &T) -> Self {
        Self::from_ptr(x as *const T)
    }

    /// Преобразует [`Virt`] в указатель на константный [`u8`].
    pub fn into_ptr_u8(self) -> *const u8 {
        const_assert_eq!(mem::align_of::<u8>(), 1);
        const_assert_eq!(mem::size_of::<*const u8>(), mem::size_of::<usize>());
        self.0 as *const u8
    }

    /// Преобразует [`Virt`] в указатель на изменяемый [`u8`].
    pub fn into_mut_ptr_u8(self) -> *mut u8 {
        const_assert_eq!(mem::align_of::<u8>(), 1);
        const_assert_eq!(mem::size_of::<*mut u8>(), mem::size_of::<usize>());
        self.0 as *mut u8
    }

    /// Преобразует [`Virt`] в указатель на константный `T`.
    ///
    /// Возвращает ошибку [`Error::InvalidAlignment`],
    /// если адрес не соответствует типу `T` по выравниванию.
    pub fn try_into_ptr<T>(self) -> Result<*const T> {
        const_assert_eq!(mem::size_of::<*const ()>(), mem::size_of::<usize>());
        if self.0 % mem::align_of::<T>() == 0 {
            Ok(self.0 as *const T)
        } else {
            Err(InvalidAlignment)
        }
    }

    /// Преобразует [`Virt`] в указатель на изменяемый `T`.
    ///
    /// Возвращает ошибку [`Error::InvalidAlignment`],
    /// если адрес не соответствует типу `T` по выравниванию.
    pub fn try_into_mut_ptr<T>(self) -> Result<*mut T> {
        const_assert_eq!(mem::size_of::<*mut ()>(), mem::size_of::<usize>());
        if self.0 % mem::align_of::<T>() == 0 {
            Ok(self.0 as *mut T)
        } else {
            Err(InvalidAlignment)
        }
    }

    /// Преобразует [`Virt`] в неизменяемую ссылку на `T`.
    ///
    /// Возвращает ошибки:
    /// - [`Error::InvalidAlignment`] если адрес не соответствует типу `Q` по выравниванию.
    /// - [`Error::Null`] если адрес нулевой.
    ///
    /// # Safety
    ///
    /// Вызывающая сторона должна гарантировать,
    /// что не будут нарушены инварианты управления памятью в Rust,
    /// которые не относятся к возвращаемым ошибкам.
    pub unsafe fn try_into_ref<'a, T>(self) -> Result<&'a T> {
        let ptr = self.try_into_ptr::<T>()?;
        if ptr.is_null() {
            Err(Null)
        } else {
            Ok(&*ptr)
        }
    }

    /// Преобразует [`Virt`] в изменяемую ссылку на `T`.
    ///
    /// Возвращает ошибки:
    /// - [`Error::InvalidAlignment`] если адрес не соответствует типу `Q` по выравниванию.
    /// - [`Error::Null`] если адрес нулевой.
    ///
    /// # Safety
    ///
    /// Вызывающая сторона должна гарантировать,
    /// что не будут нарушены инварианты управления памятью в Rust,
    /// которые не относятся к возвращаемым ошибкам.
    pub unsafe fn try_into_mut<'a, T>(self) -> Result<&'a mut T> {
        let ptr = self.try_into_mut_ptr::<T>()?;
        if ptr.is_null() {
            Err(Null)
        } else {
            Ok(&mut *ptr)
        }
    }

    /// Преобразует [`Virt`] в срез из `len` неизменяемых элементов типа `Q`.
    ///
    /// Возвращает ошибки:
    /// - [`Error::InvalidAlignment`] если адрес не соответствует типу `Q` по выравниванию.
    /// - [`Error::Null`] если адрес нулевой.
    ///
    /// # Safety
    ///
    /// Вызывающая сторона должна гарантировать,
    /// что не будут нарушены инварианты управления памятью в Rust,
    /// которые не относятся к возвращаемым ошибкам.
    pub unsafe fn try_into_slice<'a, T>(
        self,
        len: usize,
    ) -> Result<&'a [T]> {
        let ptr = self.try_into_ptr::<T>()?;
        if ptr.is_null() {
            Err(Null)
        } else {
            Ok(slice::from_raw_parts(ptr, len))
        }
    }

    /// Преобразует [`Virt`] в срез из `len` изменяемых элементов типа `Q`.
    ///
    /// Возвращает ошибки:
    /// - [`Error::InvalidAlignment`] если адрес не соответствует типу `Q` по выравниванию.
    /// - [`Error::Null`] если адрес нулевой.
    ///
    /// # Safety
    ///
    /// Вызывающая сторона должна гарантировать,
    /// что не будут нарушены инварианты управления памятью в Rust,
    /// которые не относятся к возвращаемым ошибкам.
    pub unsafe fn try_into_mut_slice<'a, T>(
        self,
        len: usize,
    ) -> Result<&'a mut [T]> {
        let ptr = self.try_into_mut_ptr::<T>()?;
        if ptr.is_null() {
            Err(Null)
        } else {
            Ok(slice::from_raw_parts_mut(ptr, len))
        }
    }

    /// Маска для
    /// [знакового расширения](https://en.wikipedia.org/wiki/X86-64#Virtual_address_space_details)
    /// битового представления виртуального адреса.
    pub const NEGATIVE_SIGN_EXTEND: usize = !0 << Self::BITS;

    /// Количество используемых битов в физическом адресе.
    const BITS: u32 = PAGE_TABLE_INDEX_BITS * (PAGE_TABLE_ROOT_LEVEL + 1) + PAGE_OFFSET_BITS;

    /// Количество неиспользуемых битов в физическом адресе.
    const UNUSED_BITS: u32 = usize::BITS - Self::BITS;
}

impl IsVirt for Virt {
    fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        Virt::from_ptr::<T>(ptr)
    }

    fn from_ref<T: ?Sized>(x: &T) -> Self {
        Virt::from_ref::<T>(x)
    }

    fn into_ptr_u8(self) -> *const u8 {
        Virt::into_ptr_u8(self)
    }

    fn into_mut_ptr_u8(self) -> *mut u8 {
        Virt::into_mut_ptr_u8(self)
    }

    fn try_into_ptr<T>(self) -> Result<*const T> {
        Virt::try_into_ptr::<T>(self)
    }

    fn try_into_mut_ptr<T>(self) -> Result<*mut T> {
        Virt::try_into_mut_ptr::<T>(self)
    }

    unsafe fn try_into_ref<'a, T>(self) -> Result<&'a T> {
        Virt::try_into_ref::<'a, T>(self)
    }

    unsafe fn try_into_mut<'a, T>(self) -> Result<&'a mut T> {
        Virt::try_into_mut::<'a, T>(self)
    }

    unsafe fn try_into_slice<'a, T>(
        self,
        len: usize,
    ) -> Result<&'a [T]> {
        Virt::try_into_slice::<T>(self, len)
    }

    unsafe fn try_into_mut_slice<'a, T>(
        self,
        len: usize,
    ) -> Result<&'a mut [T]> {
        Virt::try_into_mut_slice::<T>(self, len)
    }
}

impl From<VirtAddr> for Virt {
    fn from(addr: VirtAddr) -> Self {
        Self::new_u64(addr.as_u64()).expect("bad VirtAddr")
    }
}

impl From<Virt> for VirtAddr {
    fn from(addr: Virt) -> Self {
        VirtAddr::new(addr.into_u64())
    }
}

impl<T: Tag> SizeOf for Addr<T> {
    const SIZE_OF: usize = 1;
}

#[cfg(test)]
mod test {
    use super::{
        Tag,
        Virt,
        VirtTag,
    };

    #[test]
    fn two_separate_halves_of_virt() {
        let lower_half_first = Virt::new(0).unwrap();
        let lower_half_last = Virt::new(0x0000_7FFF_FFFF_FFFF).unwrap();
        let higher_half_first = Virt::new(0xFFFF_8000_0000_0000).unwrap();
        let higher_half_last = Virt::new(0xFFFF_FFFF_FFFF_FFFF).unwrap();

        assert!(VirtTag::is_same_half(lower_half_first, lower_half_last));
        assert!(VirtTag::is_same_half(higher_half_first, higher_half_last));
        assert!(!VirtTag::is_same_half(lower_half_first, higher_half_first));
        assert!(!VirtTag::is_same_half(lower_half_first, higher_half_last));
        assert!(!VirtTag::is_same_half(lower_half_last, higher_half_first));
        assert!(!VirtTag::is_same_half(lower_half_last, higher_half_last));

        assert!(
            (lower_half_last + (higher_half_first.into_usize() - lower_half_last.into_usize()))
                .is_err()
        );

        for shift in [0, 1, 2, 10, 20, 30, 40, 50, 63] {
            let mut delta = (1usize << shift).wrapping_sub(0x10);
            for _ in 0 ..= 0x20 {
                if delta != 0 {
                    assert!((lower_half_first - delta).is_err());
                    assert!((lower_half_last + delta).is_err());
                    assert!((higher_half_first - delta).is_err());
                    assert!((higher_half_last + delta).is_err());
                }
                delta = delta.wrapping_add(1);
            }
        }
    }
}
