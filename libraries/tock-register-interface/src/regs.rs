//! Implementation of registers and bitfields.
//!
//! Allows register maps to be specified like this:
//!
//! ```rust
//! use common::regs::{ReadOnly, ReadWrite, WriteOnly};
//!
//! #[repr(C)]
//! struct Registers {
//!     // Control register: read-write
//!     cr: ReadWrite<u32, Control::Register>,
//!     // Status register: read-only
//!     s: ReadOnly<u32, Status::Register>,
//! }
//! ```
//!
//! and register fields and definitions to look like:
//!
//! ```rust
//! register_bitfields![u32,
//!     Status [
//!         TXCOMPLETE  OFFSET(0) NUMBITS(1) [],
//!         TXINTERRUPT OFFSET(1) NUMBITS(1) [],
//!         RXCOMPLETE  OFFSET(2) NUMBITS(1) [],
//!         RXINTERRUPT OFFSET(3) NUMBITS(1) [],
//!         MODE        OFFSET(4) NUMBITS(3) [
//!             FullDuplex = 0,
//!             HalfDuplex = 1,
//!             Loopback = 2,
//!             Disabled = 3
//!         ],
//!         ERRORCOUNT OFFSET(6) NUMBITS(3) []
//!     ]
//! ];
//! ```
//!
//! Author
//! ------
//! - Shane Leonard <shanel@stanford.edu>

use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, BitAnd, BitOr, Not, Shl, Shr};

/// IntLike properties needed to read/write/modify a register.
pub trait IntLike:
    BitAnd<Output = Self>
    + BitOr<Output = Self>
    + Not<Output = Self>
    + Eq
    + Shr<u32, Output = Self>
    + Shl<u32, Output = Self>
    + Copy
    + Clone
{
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

/// Descriptive name for each register.
pub trait RegisterLongName {}

impl RegisterLongName for () {}

/// Conversion of raw register value into enumerated values member.
/// Implemented inside register_bitfields![] macro for each bit field.
pub trait TryFromValue<V> {
    type EnumType;

    fn try_from(v: V) -> Option<Self::EnumType>;
}

/// Read/Write registers.
pub struct ReadWrite<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

/// Read-only registers.
pub struct ReadOnly<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

/// Write-only registers.
pub struct WriteOnly<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

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
    pub fn read_as_enum<E: TryFromValue<T, EnumType=E>>(&self, field: Field<T, R>) -> Option<E> {
        let val: T = self.read(field);

        E::try_from(val)
    }
    
    #[inline]
    pub fn extract(&self) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.get())
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
    pub fn modify_no_read(&self, original: LocalRegisterCopy<T, R>, field: FieldValue<T, R>) {
        self.set((original.get() & !field.mask) | field.value);
    }

    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        self.read(field) != T::zero()
    }

    #[inline]
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask != T::zero()
    }

    #[inline]
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask == field.value
    }
}

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
    pub fn read_as_enum<E: TryFromValue<T, EnumType=E>>(&self, field: Field<T, R>) -> Option<E> {
        let val: T = self.read(field);

        E::try_from(val)
    }

    #[inline]
    pub fn extract(&self) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.get())
    }

    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        self.read(field) != T::zero()
    }

    #[inline]
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask != T::zero()
    }

    #[inline]
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask == field.value
    }
}

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

/// This behaves very similarly to a read-only register, but instead of doing a
/// volatile read to MMIO to get the value for each function call, a copy of the
/// register contents are stored locally in memory. This allows a peripheral
/// to do a single read on a register, and then check which bits are set without
/// having to do a full MMIO read each time. It also allows the value of the
/// register to be "cached" in case the peripheral driver needs to clear the
/// register in hardware yet still be able to check the bits.
#[derive(Copy, Clone)]
pub struct LocalRegisterCopy<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

impl<T: IntLike, R: RegisterLongName> LocalRegisterCopy<T, R> {
    pub const fn new(value: T) -> Self {
        LocalRegisterCopy {
            value: value,
            associated_register: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self) -> T {
        self.value
    }

    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        (self.value & (field.mask << field.shift)) >> field.shift
    }

    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        self.read(field) != T::zero()
    }

    #[inline]
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        self.value & field.mask != T::zero()
    }

    #[inline]
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        self.value & field.mask == field.value
    }

    /// Do a bitwise AND operation of the stored value and the passed in value
    /// and return a new LocalRegisterCopy.
    #[inline]
    pub fn bitand(&self, rhs: T) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.value & rhs)
    }
}

impl<T: IntLike + fmt::Debug, R: RegisterLongName> fmt::Debug for LocalRegisterCopy<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl<R: RegisterLongName> From<LocalRegisterCopy<u8, R>> for u8 {
    fn from(r: LocalRegisterCopy<u8, R>) -> u8 {
        r.value
    }
}

impl<R: RegisterLongName> From<LocalRegisterCopy<u16, R>> for u16 {
    fn from(r: LocalRegisterCopy<u16, R>) -> u16 {
        r.value
    }
}

impl<R: RegisterLongName> From<LocalRegisterCopy<u32, R>> for u32 {
    fn from(r: LocalRegisterCopy<u32, R>) -> u32 {
        r.value
    }
}

/// Specific section of a register.
#[derive(Copy, Clone)]
pub struct Field<T: IntLike, R: RegisterLongName> {
    pub mask: T,
    pub shift: u32,
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

/// Values for the specific register fields.
// For the FieldValue, the masks and values are shifted into their actual
// location in the register.
#[derive(Copy, Clone)]
pub struct FieldValue<T: IntLike, R: RegisterLongName> {
    pub mask: T,
    pub value: T,
    associated_register: PhantomData<R>,
}

// Necessary to split the implementation of u8 and u32 out because the bitwise
// math isn't treated as const when the type is generic.
impl<R: RegisterLongName> FieldValue<u8, R> {
    pub const fn new(mask: u8, shift: u32, value: u8) -> Self {
        FieldValue {
            mask: mask << shift,
            value: (value << shift) & (mask << shift),
            associated_register: PhantomData,
        }
    }

    /// Get the raw bitmask represented by this FieldValue.
    pub fn mask(self) -> u8 {
        self.mask as u8
    }
}

impl<R: RegisterLongName> From<FieldValue<u8, R>> for u8 {
    fn from(val: FieldValue<u8, R>) -> u8 {
        val.value
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

impl<R: RegisterLongName> From<FieldValue<u16, R>> for u16 {
    fn from(val: FieldValue<u16, R>) -> u16 {
        val.value
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

    /// Get the raw bitmask represented by this FieldValue.
    pub fn mask(self) -> u32 {
        self.mask as u32
    }
}

impl<R: RegisterLongName> From<FieldValue<u32, R>> for u32 {
    fn from(val: FieldValue<u32, R>) -> u32 {
        val.value
    }
}

impl<T: IntLike, R: RegisterLongName> FieldValue<T, R> {
    // Modify fields in a register value
    pub fn modify(self, val: T) -> T {
        (val & !self.mask) | self.value
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

// Combine two fields with the += operator
impl<T: IntLike, R: RegisterLongName> AddAssign for FieldValue<T, R> {
    fn add_assign(&mut self, rhs: FieldValue<T, R>) {
        *self = FieldValue {
            mask: self.mask | rhs.mask,
            value: self.value | rhs.value,
            associated_register: PhantomData,
        };
    }
}
