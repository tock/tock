//! LiteX is able to generate vastly different SoC with different
//! buswidths, CSR widths and configurations
//!
//! This module defines interfaces very similar to `tock_registers`
//! (and based on `tock_registers`) for various register- and
//! bus-width configurations
//!
//! Essentially, the bus data width (default 32 bit), the CSR data
//! width, the CSR byte ordering and naturally the desired register
//! width can change. This module defines generic traits and uses a
//! procedural macro to generate combinations of these settings, to
//! then be used in register structs.
//!
//! The different register types of a specific SoC configuration are
//! combined using a
//! [`LiteXSoCRegisterConfiguration`](crate::litex_registers::LiteXSoCRegisterConfiguration)
//! structure, which can be used to adapt the register interfaces of
//! peripherals to the different configurations.
//!
//! ## Naming Scheme
//!
//! The generated register abstractions follow the naming scheme
//!
//! ```
//! <AccessTypes><RegisterWidth>C<CSRDataWidth>B<BaseWidth>
//! ```
//!
//! where `AccessType` in `{ ReadOnly, WriteOnly, ReadWrite }`,
//! `RegisterWidth` in `{ 8, 16, 32, 64 }`, `CSRDataWidth` in `{ 8, 32
//! }`, `BaseWidth` in `{ 32 }`.

use core::marker::PhantomData;
pub use tock_registers::register_bitfields;
use tock_registers::registers::{
    Field, FieldValue, IntLike as TRIntLike, LocalRegisterCopy, ReadOnly as TRReadOnly,
    ReadWrite as TRReadWrite, RegisterLongName, TryFromValue, WriteOnly as TRWriteOnly,
};

/// Extend the `tock_registers` `IntLike` trait to also provide the
/// `1` and maximum (all bits set) values of the respective integer
/// type
///
/// This allows for peripherals to be written generic over the
/// underlying CSR width (as in the case of event managers, LEDs,
/// etc.), manipulating bitmaps
pub trait IntLike: TRIntLike {
    fn one() -> Self;
    fn max() -> Self {
        !Self::zero()
    }
}

// Implement the custom IntLike trait for all required base integer
// types
impl IntLike for u8 {
    fn one() -> Self {
        1
    }
}
impl IntLike for u16 {
    fn one() -> Self {
        1
    }
}
impl IntLike for u32 {
    fn one() -> Self {
        1
    }
}
impl IntLike for u64 {
    fn one() -> Self {
        1
    }
}

/// Trait to be implemented by custom register structs that support
/// reading the current value
///
/// # DO NOT USE DIRECTLY
///
/// This must be public to prevent generating a `private_in_public`
/// but really should not be used directly. Use `Read` instead, which
/// also incorporates this functionality.
pub trait BaseReadableRegister<T: IntLike> {
    type Reg: RegisterLongName;
    const REG_WIDTH: usize;

    /// Get the raw register value
    fn base_get(&self) -> T;
}

/// Trait to be implemented by custom register structs that support
/// writing to them
///
/// ## DO NOT USE DIRECTLY
///
/// This must be public to prevent generating a `private_in_public`
/// but really should not be used directly. Use `Write` instead, which
/// also incorporates this functionality.
pub trait BaseWriteableRegister<T: IntLike> {
    type Reg: RegisterLongName;
    const REG_WIDTH: usize;

    /// Set the raw register value
    fn base_set(&self, value: T);
}

/// Readable register
///
/// This interface is analogous to the methods supported on
/// `tock_registers::registers::{ReadOnly, ReadWrite}` types
///
/// It is automatically implemented for all `BaseReadableRegister`s
pub trait Read<T: IntLike> {
    type Reg: RegisterLongName;
    const REG_WIDTH: usize;

    /// Get the raw register value
    fn get(&self) -> T;

    /// Read the value of the given field
    fn read(&self, field: Field<T, Self::Reg>) -> T;

    /// Read value of the given field as an enum member
    fn read_as_enum<E: TryFromValue<T, EnumType = E>>(
        &self,
        field: Field<T, Self::Reg>,
    ) -> Option<E>;

    /// Make a local copy of the register
    fn extract(&self) -> LocalRegisterCopy<T, Self::Reg>;

    /// Check if one or more bits in a field are set
    fn is_set(&self, field: Field<T, Self::Reg>) -> bool;

    /// Check if any specified parts of a field match
    fn matches_any(&self, field: FieldValue<T, Self::Reg>) -> bool;

    /// Check if all specified parts of a field match
    fn matches_all(&self, field: FieldValue<T, Self::Reg>) -> bool;
}

/// Writeable register
///
/// This interface is analogous to the methods supported on
/// `tock_registers::registers::{WriteOnly, ReadWrite}` types
///
/// It is automatically implemented for all `BaseWriteableRegister`s
pub trait Write<T: IntLike> {
    type Reg: RegisterLongName;
    const REG_WIDTH: usize;

    /// Set the raw register value
    fn set(&self, value: T);

    /// Write the value of one or more fields, overwriting the other
    /// fields with zero
    fn write(&self, field: FieldValue<T, Self::Reg>);
}

/// Readable and writable register
///
/// This interface is analogous to the methods supported on
/// `tock_registers::registers::ReadWrite` types
///
/// It is automatically implemented for all types that are both
/// `BaseReadableRegister` and `BaseWriteableRegister`s
pub trait ReadWrite<T: IntLike>: Read<T> + Write<T> {
    const REG_WIDTH: usize;

    /// Write the value of one or more fields, leaving the other
    /// fields unchanged
    fn modify(&self, field: FieldValue<T, <Self as Read<T>>::Reg>);

    /// Write the value of one or more fields, maintaining the value
    /// of unchanged fields via a provided original value, rather than
    /// a register read.
    fn modify_no_read(
        &self,
        original: LocalRegisterCopy<T, <Self as Read<T>>::Reg>,
        field: FieldValue<T, <Self as Read<T>>::Reg>,
    );
}

impl<R, T: IntLike> Read<T> for R
where
    R: BaseReadableRegister<T>,
{
    type Reg = <Self as BaseReadableRegister<T>>::Reg;
    const REG_WIDTH: usize = R::REG_WIDTH;

    #[inline]
    fn get(&self) -> T {
        self.base_get()
    }

    #[inline]
    fn read(&self, field: Field<T, Self::Reg>) -> T {
        field.read(self.get())
    }

    #[inline]
    fn read_as_enum<E: TryFromValue<T, EnumType = E>>(
        &self,
        field: Field<T, Self::Reg>,
    ) -> Option<E> {
        field.read_as_enum(self.get())
    }

    #[inline]
    fn extract(&self) -> LocalRegisterCopy<T, Self::Reg> {
        LocalRegisterCopy::new(self.get())
    }

    #[inline]
    fn is_set(&self, field: Field<T, Self::Reg>) -> bool {
        field.is_set(self.get())
    }

    #[inline]
    fn matches_any(&self, field: FieldValue<T, Self::Reg>) -> bool {
        field.matches_any(self.get())
    }

    #[inline]
    fn matches_all(&self, field: FieldValue<T, Self::Reg>) -> bool {
        field.matches_all(self.get())
    }
}

impl<R, T: IntLike> Write<T> for R
where
    R: BaseWriteableRegister<T>,
{
    type Reg = <Self as BaseWriteableRegister<T>>::Reg;
    const REG_WIDTH: usize = R::REG_WIDTH;

    #[inline]
    fn set(&self, value: T) {
        self.base_set(value)
    }

    #[inline]
    fn write(&self, field: FieldValue<T, Self::Reg>) {
        self.set(field.value)
    }
}

impl<R, T: IntLike> ReadWrite<T> for R
where
    R: Read<T> + Write<T>,
{
    const REG_WIDTH: usize = <R as Read<T>>::REG_WIDTH;

    #[inline]
    fn modify(&self, field: FieldValue<T, <Self as Read<T>>::Reg>) {
        self.set(field.modify(self.get()));
    }

    #[inline]
    fn modify_no_read(
        &self,
        original: LocalRegisterCopy<T, <Self as Read<T>>::Reg>,
        field: FieldValue<T, <Self as Read<T>>::Reg>,
    ) {
        self.set(field.modify(original.get()));
    }
}

pub trait LiteXSoCRegisterConfiguration {
    type ReadOnly8: BaseReadableRegister<u8, Reg = ()>;
    type WriteOnly8: BaseWriteableRegister<u8, Reg = ()>;
    type ReadWrite8: BaseReadableRegister<u8, Reg = ()> + BaseWriteableRegister<u8, Reg = ()>;

    type ReadOnly16: BaseReadableRegister<u16, Reg = ()>;
    type WriteOnly16: BaseWriteableRegister<u16, Reg = ()>;
    type ReadWrite16: BaseReadableRegister<u16, Reg = ()> + BaseWriteableRegister<u16, Reg = ()>;

    type ReadOnly32: BaseReadableRegister<u32, Reg = ()>;
    type WriteOnly32: BaseWriteableRegister<u32, Reg = ()>;
    type ReadWrite32: BaseReadableRegister<u32, Reg = ()> + BaseWriteableRegister<u32, Reg = ()>;

    type ReadOnly64: BaseReadableRegister<u64, Reg = ()>;
    type WriteOnly64: BaseWriteableRegister<u64, Reg = ()>;
    type ReadWrite64: BaseReadableRegister<u64, Reg = ()> + BaseWriteableRegister<u64, Reg = ()>;
}

litex_register_abstraction!(ReadOnly8C8B32 {
    access_type: "read_only",
    value_width: 8,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(WriteOnly8C8B32 {
    access_type: "write_only",
    value_width: 8,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadWrite8C8B32 {
    access_type: "read_write",
    value_width: 8,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadOnly16C8B32 {
    access_type: "read_only",
    value_width: 16,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(WriteOnly16C8B32 {
    access_type: "write_only",
    value_width: 16,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadWrite16C8B32 {
    access_type: "read_write",
    value_width: 16,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadOnly32C8B32 {
    access_type: "read_only",
    value_width: 32,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(WriteOnly32C8B32 {
    access_type: "write_only",
    value_width: 32,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadWrite32C8B32 {
    access_type: "read_write",
    value_width: 32,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadOnly64C8B32 {
    access_type: "read_only",
    value_width: 64,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(WriteOnly64C8B32 {
    access_type: "write_only",
    value_width: 64,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadWrite64C8B32 {
    access_type: "read_write",
    value_width: 64,
    wishbone_data_width: 8,
    base_width: 32,
    endianess: "big",
});

pub enum LiteXSoCRegistersC8B32 {}
impl LiteXSoCRegisterConfiguration for LiteXSoCRegistersC8B32 {
    type ReadOnly8 = ReadOnly8C8B32;
    type WriteOnly8 = WriteOnly8C8B32;
    type ReadWrite8 = ReadWrite8C8B32;

    type ReadOnly16 = ReadOnly16C8B32;
    type WriteOnly16 = WriteOnly16C8B32;
    type ReadWrite16 = ReadWrite16C8B32;

    type ReadOnly32 = ReadOnly32C8B32;
    type WriteOnly32 = WriteOnly32C8B32;
    type ReadWrite32 = ReadWrite32C8B32;

    type ReadOnly64 = ReadOnly64C8B32;
    type WriteOnly64 = WriteOnly64C8B32;
    type ReadWrite64 = ReadWrite64C8B32;
}

litex_register_abstraction!(ReadOnly8C32B32 {
    access_type: "read_only",
    value_width: 8,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(WriteOnly8C32B32 {
    access_type: "write_only",
    value_width: 8,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadWrite8C32B32 {
    access_type: "read_write",
    value_width: 8,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadOnly16C32B32 {
    access_type: "read_only",
    value_width: 16,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(WriteOnly16C32B32 {
    access_type: "write_only",
    value_width: 16,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadWrite16C32B32 {
    access_type: "read_write",
    value_width: 16,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadOnly32C32B32 {
    access_type: "read_only",
    value_width: 32,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(WriteOnly32C32B32 {
    access_type: "write_only",
    value_width: 32,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadWrite32C32B32 {
    access_type: "read_write",
    value_width: 32,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadOnly64C32B32 {
    access_type: "read_only",
    value_width: 64,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(WriteOnly64C32B32 {
    access_type: "write_only",
    value_width: 64,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

litex_register_abstraction!(ReadWrite64C32B32 {
    access_type: "read_write",
    value_width: 64,
    wishbone_data_width: 32,
    base_width: 32,
    endianess: "big",
});

pub enum LiteXSoCRegistersC32B32 {}
impl LiteXSoCRegisterConfiguration for LiteXSoCRegistersC32B32 {
    type ReadOnly8 = ReadOnly8C32B32;
    type WriteOnly8 = WriteOnly8C32B32;
    type ReadWrite8 = ReadWrite8C32B32;

    type ReadOnly16 = ReadOnly16C32B32;
    type WriteOnly16 = WriteOnly16C32B32;
    type ReadWrite16 = ReadWrite16C32B32;

    type ReadOnly32 = ReadOnly32C32B32;
    type WriteOnly32 = WriteOnly32C32B32;
    type ReadWrite32 = ReadWrite32C32B32;

    type ReadOnly64 = ReadOnly64C32B32;
    type WriteOnly64 = WriteOnly64C32B32;
    type ReadWrite64 = ReadWrite64C32B32;
}

pub struct ReadRegWrapper<'a, T: IntLike, N: RegisterLongName, R: BaseReadableRegister<T>>(
    &'a R,
    PhantomData<T>,
    PhantomData<N>,
);
impl<'a, T: IntLike, N: RegisterLongName, R: BaseReadableRegister<T>> ReadRegWrapper<'a, T, N, R> {
    #[inline]
    pub fn wrap(reg: &'a R) -> Self {
        ReadRegWrapper(reg, PhantomData, PhantomData)
    }
}

impl<T: IntLike, N: RegisterLongName, R: BaseReadableRegister<T>> BaseReadableRegister<T>
    for ReadRegWrapper<'_, T, N, R>
{
    type Reg = N;
    const REG_WIDTH: usize = R::REG_WIDTH;

    #[inline]
    fn base_get(&self) -> T {
        self.0.base_get()
    }
}

pub struct WriteRegWrapper<'a, T: IntLike, N: RegisterLongName, R: BaseWriteableRegister<T>>(
    &'a R,
    PhantomData<T>,
    PhantomData<N>,
);
impl<'a, T: IntLike, N: RegisterLongName, R: BaseWriteableRegister<T>>
    WriteRegWrapper<'a, T, N, R>
{
    #[inline]
    pub fn wrap(reg: &'a R) -> Self {
        WriteRegWrapper(reg, PhantomData, PhantomData)
    }
}

impl<T: IntLike, N: RegisterLongName, R: BaseWriteableRegister<T>> BaseWriteableRegister<T>
    for WriteRegWrapper<'_, T, N, R>
{
    type Reg = N;
    const REG_WIDTH: usize = R::REG_WIDTH;

    #[inline]
    fn base_set(&self, value: T) {
        self.0.base_set(value)
    }
}

pub struct ReadWriteRegWrapper<
    'a,
    T: IntLike,
    N: RegisterLongName,
    R: BaseReadableRegister<T> + BaseWriteableRegister<T>,
>(&'a R, PhantomData<T>, PhantomData<N>);
impl<
        'a,
        T: IntLike,
        N: RegisterLongName,
        R: BaseReadableRegister<T> + BaseWriteableRegister<T>,
    > ReadWriteRegWrapper<'a, T, N, R>
{
    #[inline]
    pub fn wrap(reg: &'a R) -> Self {
        ReadWriteRegWrapper(reg, PhantomData, PhantomData)
    }
}

impl<T: IntLike, N: RegisterLongName, R: BaseReadableRegister<T> + BaseWriteableRegister<T>>
    BaseReadableRegister<T> for ReadWriteRegWrapper<'_, T, N, R>
{
    type Reg = N;
    const REG_WIDTH: usize = <R as BaseReadableRegister<T>>::REG_WIDTH;

    #[inline]
    fn base_get(&self) -> T {
        self.0.base_get()
    }
}

impl<T: IntLike, N: RegisterLongName, R: BaseReadableRegister<T> + BaseWriteableRegister<T>>
    BaseWriteableRegister<T> for ReadWriteRegWrapper<'_, T, N, R>
{
    type Reg = N;
    const REG_WIDTH: usize = <R as BaseWriteableRegister<T>>::REG_WIDTH;

    #[inline]
    fn base_set(&self, value: T) {
        self.0.base_set(value)
    }
}
