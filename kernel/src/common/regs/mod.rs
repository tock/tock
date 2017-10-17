//! Implementation of registers and bitfields

#[macro_use]
pub mod macros;

use core::marker::PhantomData;
use core::ops::{BitAnd, BitOr, Not, Shr, Shl, Add};

pub trait IntLike: BitAnd<Output=Self> +
                   BitOr<Output=Self> +
                   Not<Output=Self> +
                   Eq +
                   Shr<u32, Output=Self> +
                   Shl<u32, Output=Self> + Copy + Clone {
    fn zero() -> Self;
}

impl IntLike for u8 {
    fn zero() -> Self {
        0
    }
}
impl IntLike for u16 {
    fn zero() -> Self {
        0
    }
}
impl IntLike for u32 {
    fn zero() -> Self {
        0
    }
}

pub trait RegisterLongName {}

impl RegisterLongName for () {}

pub struct ReadWrite<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

pub struct ReadOnly<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

pub struct WriteOnly<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

#[allow(dead_code)]
impl<T: IntLike, R: RegisterLongName> ReadWrite<T, R> {
    pub const fn new(value: T) -> Self {
        ReadWrite {
            value: value,
            associated_register: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self) -> T {
        unsafe { ::core::ptr::read_volatile(&self.value) }
    }

    #[inline]
    pub fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(&self.value as *const T as *mut T, value) }
    }

    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        (self.get() & (field.mask << field.shift)) >> field.shift
    }

    #[inline]
    pub fn write(&self, field: FieldValue<T, R>) {
        self.set(field.value);
    }

    #[inline]
    pub fn modify(&self, field: FieldValue<T, R>) {
        let reg: T = self.get();
        self.set((reg & !field.mask) | field.value);
    }

    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        self.read(field) != T::zero()
    }

    #[inline]
    pub fn matches(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask == field.value
    }
}

#[allow(dead_code)]
impl<T: IntLike, R: RegisterLongName> ReadOnly<T, R> {
    pub const fn new(value: T) -> Self {
        ReadOnly {
            value: value,
            associated_register: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self) -> T {
        unsafe { ::core::ptr::read_volatile(&self.value) }
    }

    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        (self.get() & (field.mask << field.shift)) >> field.shift
    }

    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        self.read(field) != T::zero()
    }

    #[inline]
    pub fn matches(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask == field.value
    }
}

#[allow(dead_code)]
impl<T: IntLike, R: RegisterLongName> WriteOnly<T, R> {
    pub const fn new(value: T) -> Self {
        WriteOnly {
            value: value,
            associated_register: PhantomData,
        }
    }

    #[inline]
    pub fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(&self.value as *const T as *mut T, value) }
    }

    #[inline]
    pub fn write(&self, field: FieldValue<T, R>) {
        self.set(field.value);
    }
}

#[derive(Copy, Clone)]
pub struct Field<T: IntLike, R: RegisterLongName> {
    mask: T,
    shift: u32,
    associated_register: PhantomData<R>,
}

// For the Field, the mask is unshifted, ie. the LSB should always be set
impl<R: RegisterLongName> Field<u8, R> {
    pub const fn new(mask: u8, shift: u32) -> Field<u8, R> {
        Field {
            mask: mask,
            shift: shift,
            associated_register: PhantomData,
        }
    }

    pub fn val(&self, value: u8) -> FieldValue<u8, R> {
        FieldValue::<u8, R>::new(self.mask, self.shift, value)
    }
}

impl<R: RegisterLongName> Field<u16, R> {
    pub const fn new(mask: u16, shift: u32) -> Field<u16, R> {
        Field {
            mask: mask,
            shift: shift,
            associated_register: PhantomData,
        }
    }

    pub fn val(&self, value: u16) -> FieldValue<u16, R> {
        FieldValue::<u16, R>::new(self.mask, self.shift, value)
    }
}

impl<R: RegisterLongName> Field<u32, R> {
    pub const fn new(mask: u32, shift: u32) -> Field<u32, R> {
        Field {
            mask: mask,
            shift: shift,
            associated_register: PhantomData,
        }
    }

    pub fn val(&self, value: u32) -> FieldValue<u32, R> {
        FieldValue::<u32, R>::new(self.mask, self.shift, value)
    }
}


// For the FieldValue, the masks and values are shifted into their actual location in the register
#[derive(Copy, Clone)]
pub struct FieldValue<T: IntLike, R: RegisterLongName> {
    mask: T,
    value: T,
    associated_register: PhantomData<R>,
}

// Necessary to split the implementation of u8 and u32 out because the bitwise math isn't treated
// as const when the type is generic
impl<R: RegisterLongName> FieldValue<u8, R> {
    pub const fn new(mask: u8, shift: u32, value: u8) -> Self {
        FieldValue {
            mask: mask << shift,
            value: (value << shift) & (mask << shift),
            associated_register: PhantomData,
        }
    }
}

impl<R: RegisterLongName> FieldValue<u16, R> {
    pub const fn new(mask: u16, shift: u32, value: u16) -> Self {
        FieldValue {
            mask: mask << shift,
            value: (value << shift) & (mask << shift),
            associated_register: PhantomData,
        }
    }
}

impl<R: RegisterLongName> FieldValue<u32, R> {
    pub const fn new(mask: u32, shift: u32, value: u32) -> Self {
        FieldValue {
            mask: mask << shift,
            value: (value << shift) & (mask << shift),
            associated_register: PhantomData,
        }
    }
}

// Combine two fields with the addition operator
impl<T: IntLike, R: RegisterLongName> Add for FieldValue<T, R> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        FieldValue {
            mask: self.mask | rhs.mask,
            value: self.value | rhs.value,
            associated_register: PhantomData,
        }
    }
}
