//! LiteX register abstraction types
//!
//! LiteX is able to generate vastly different SoC with different
//! buswidths, CSR widths and configurations
//!
//! This module defines interfaces very similar to `tock_registers`
//! (and based on `tock_registers`) for various register- and
//! bus-width configurations
//!
//! Essentially, the bus data width (default 32 bit), the CSR data
//! width, the CSR byte ordering and naturally the desired register
//! width can change. This module defines generic traits for accessing
//! registers and register abstraction structs.
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
//! ```text
//! <AccessTypes><RegisterWidth>C<CSRDataWidth>B<BaseWidth>
//! ```
//!
//! where `AccessType` in `{ ReadOnly, WriteOnly, ReadWrite }`,
//! `RegisterWidth` in `{ 8, 16, 32, 64 }`, `CSRDataWidth` in `{ 8, 32
//! }`, `BaseWidth` in `{ 32 }`.

use core::marker::PhantomData;
use tock_registers::fields::{Field, FieldValue, TryFromValue};
use tock_registers::interfaces::{Readable, Writeable};
pub use tock_registers::register_bitfields;
use tock_registers::registers::{
    ReadOnly as TRReadOnly, ReadWrite as TRReadWrite, WriteOnly as TRWriteOnly,
};
use tock_registers::{LocalRegisterCopy, RegisterLongName, UIntLike as TRUIntLike};

/// Extension of the `tock_registers` `UIntLike` trait.
///
/// This extends the `UIntLike` trait of `tock_registers` to also
/// provide the `1` and maximum (all bits set) values of the
/// respective integer type
///
/// This allows for peripherals to be written generic over the
/// underlying CSR width (as in the case of event managers, LEDs,
/// etc.), manipulating bitmaps
pub trait UIntLike: TRUIntLike {
    fn one() -> Self;
    fn max() -> Self {
        !Self::zero()
    }
}

// Implement the custom UIntLike trait for all required base integer
// types
impl UIntLike for u8 {
    fn one() -> Self {
        1
    }
}
impl UIntLike for u16 {
    fn one() -> Self {
        1
    }
}
impl UIntLike for u32 {
    fn one() -> Self {
        1
    }
}
impl UIntLike for u64 {
    fn one() -> Self {
        1
    }
}
impl UIntLike for u128 {
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
pub trait BaseReadableRegister<T: UIntLike> {
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
pub trait BaseWriteableRegister<T: UIntLike> {
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
pub trait Read<T: UIntLike> {
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
pub trait Write<T: UIntLike> {
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
pub trait ReadWrite<T: UIntLike>: Read<T> + Write<T> {
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

// Implement the [`Read`] trait (providing high-level methods to read
// specific fields of a register) for every type implementing the
// [`BaseReadableRegister`] trait.
impl<R, T: UIntLike> Read<T> for R
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

// Implement the [`Write`] trait (providing high-level methods to set
// specific fields of a register) for every type implementing the
// [`BaseWritableRegister`] trait.
impl<R, T: UIntLike> Write<T> for R
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

// Implement the [`ReadWrite`] trait (providing high-level methods to
// update specific fields of a register) for every type implementing
// both the [`Read`] and [`Write`] trait.
impl<R, T: UIntLike> ReadWrite<T> for R
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

// ---------- COLLECTION OF REGISTER TYPES FOR A SPECIFIC LITEX CONFIGURATION ----------

/// Register abstraction types collection
///
/// This trait defines a collection of types for a certain set of
/// LiteX configuration options. It provides types with all
/// accessibility constraints ([`Read`], [`Write`], [`ReadWrite`]) for
/// every defined register width.
///
/// All types must be over a common register layout configuration, having identical
///
/// - base integer width
/// - CSR data width
/// - endianness
///
/// ## Generic Register Type Arguments
///
/// Usually registers are generic over a [`RegisterLongName`] type
/// arguments, such that the [`Field`] and [`FieldValue`] arguments of
/// the various methods make sense.
///
/// Unfortunately, those generic type arguments cannot be passed
/// through associated types in traits until [generic associated
/// types](https://github.com/rust-lang/rust/issues/44265) stabilize.
///
/// In the meantime, the types [`ReadRegWrapper`], [`WriteRegWrapper`]
/// and [`ReadWriteRegWrapper`] can be used to access fields in a
/// register as commonly done in tock-registers:
///
/// ```rust
/// # // This is a dummy setup to make the doctests pass
/// # use tock_registers::register_bitfields;
/// # use kernel::utilities::StaticRef;
/// # use litex::litex_registers::{
/// #   LiteXSoCRegisterConfiguration,
/// #   LiteXSoCRegistersC8B32,
/// #   Read,
/// #   ReadRegWrapper,
/// # };
/// #
/// pub struct LiteXUartRegisters<R: LiteXSoCRegisterConfiguration> {
///   txfull: R::ReadOnly8,
/// }
///
/// register_bitfields![u8,
///   txfull [
///     full OFFSET(0) NUMBITS(1) []
///   ],
/// ];
///
/// # static mut testregs: u32 = 0;
/// #
/// # fn main() {
/// #   let regs = unsafe {
/// #     StaticRef::new(&mut testregs as *mut u32 as *mut LiteXUartRegisters<LiteXSoCRegistersC8B32>)
/// #   };
/// #   let _ =
/// ReadRegWrapper::wrap(&regs.txfull).is_set(txfull::full)
/// #   ;
/// # }
/// ```
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

/// Collection of LiteX register abstraction types
///
/// This collection of LiteX registers has the following configuration:
///
/// - base integer width: 32 bit
/// - csr data width: 8 bit
/// - endianness: big
///
/// For documentation on the usage of these types, refer to the
/// [`LiteXSoCRegisterConfiguration`] trait documentation.
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

/// Collection of LiteX register abstraction types
///
/// This collection of LiteX registers has the following configuration:
///
/// - base integer width: 32 bit
/// - csr data width: 32 bit
/// - endianness: big
///
/// For documentation on the usage of these types, refer to the
/// [`LiteXSoCRegisterConfiguration`] trait documentation.
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

// ---------- WRAPPERS AROUND READ,WRITE,READWRITE TRAITS WITH GENERIC REGISTER NAMES ----------
/// Workaround-wrapper for readable LiteX registers
///
/// This workaround-wrapper is required to make an associated type of
/// [`LiteXSoCRegisterConfiguration`] generic over the
/// [`RegisterLongName`] until generic associated types stabilize in
/// Rust. Please see the [`LiteXSoCRegisterConfiguration`]
/// documentation for more information.
pub struct ReadRegWrapper<'a, T: UIntLike, N: RegisterLongName, R: BaseReadableRegister<T>>(
    &'a R,
    PhantomData<T>,
    PhantomData<N>,
);
impl<'a, T: UIntLike, N: RegisterLongName, R: BaseReadableRegister<T>> ReadRegWrapper<'a, T, N, R> {
    #[inline]
    pub fn wrap(reg: &'a R) -> Self {
        ReadRegWrapper(reg, PhantomData, PhantomData)
    }
}

impl<T: UIntLike, N: RegisterLongName, R: BaseReadableRegister<T>> BaseReadableRegister<T>
    for ReadRegWrapper<'_, T, N, R>
{
    type Reg = N;
    const REG_WIDTH: usize = R::REG_WIDTH;

    #[inline]
    fn base_get(&self) -> T {
        self.0.base_get()
    }
}

/// Workaround-wrapper for writable LiteX registers
///
/// This workaround-wrapper is required to make an associated type of
/// [`LiteXSoCRegisterConfiguration`] generic over the
/// [`RegisterLongName`] until generic associated types stabilize in
/// Rust. Please see the [`LiteXSoCRegisterConfiguration`]
/// documentation for more information.
pub struct WriteRegWrapper<'a, T: UIntLike, N: RegisterLongName, R: BaseWriteableRegister<T>>(
    &'a R,
    PhantomData<T>,
    PhantomData<N>,
);
impl<'a, T: UIntLike, N: RegisterLongName, R: BaseWriteableRegister<T>>
    WriteRegWrapper<'a, T, N, R>
{
    #[inline]
    pub fn wrap(reg: &'a R) -> Self {
        WriteRegWrapper(reg, PhantomData, PhantomData)
    }
}

impl<T: UIntLike, N: RegisterLongName, R: BaseWriteableRegister<T>> BaseWriteableRegister<T>
    for WriteRegWrapper<'_, T, N, R>
{
    type Reg = N;
    const REG_WIDTH: usize = R::REG_WIDTH;

    #[inline]
    fn base_set(&self, value: T) {
        self.0.base_set(value)
    }
}

/// Workaround-wrapper for read- and writable LiteX registers
///
/// This workaround-wrapper is required to make an associated type of
/// [`LiteXSoCRegisterConfiguration`] generic over the
/// [`RegisterLongName`] until generic associated types stabilize in
/// Rust. Please see the [`LiteXSoCRegisterConfiguration`]
/// documentation for more information.
pub struct ReadWriteRegWrapper<
    'a,
    T: UIntLike,
    N: RegisterLongName,
    R: BaseReadableRegister<T> + BaseWriteableRegister<T>,
>(&'a R, PhantomData<T>, PhantomData<N>);
impl<
        'a,
        T: UIntLike,
        N: RegisterLongName,
        R: BaseReadableRegister<T> + BaseWriteableRegister<T>,
    > ReadWriteRegWrapper<'a, T, N, R>
{
    #[inline]
    pub fn wrap(reg: &'a R) -> Self {
        ReadWriteRegWrapper(reg, PhantomData, PhantomData)
    }
}

impl<T: UIntLike, N: RegisterLongName, R: BaseReadableRegister<T> + BaseWriteableRegister<T>>
    BaseReadableRegister<T> for ReadWriteRegWrapper<'_, T, N, R>
{
    type Reg = N;
    const REG_WIDTH: usize = <R as BaseReadableRegister<T>>::REG_WIDTH;

    #[inline]
    fn base_get(&self) -> T {
        self.0.base_get()
    }
}

impl<T: UIntLike, N: RegisterLongName, R: BaseReadableRegister<T> + BaseWriteableRegister<T>>
    BaseWriteableRegister<T> for ReadWriteRegWrapper<'_, T, N, R>
{
    type Reg = N;
    const REG_WIDTH: usize = <R as BaseWriteableRegister<T>>::REG_WIDTH;

    #[inline]
    fn base_set(&self, value: T) {
        self.0.base_set(value)
    }
}

// ---------- AUTOMATICALLY GENERATED CODE ----------
//
// The following code has been gerated by the `litex-register-gen`
// procedural macro, which is not yet included with Tock.
//
// The following arguments to the macro produced the results below:
//
// litex_register_abstraction!(ReadOnly8C8B32 {
//     access_type: "read_only",
//     value_width: 8,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(WriteOnly8C8B32 {
//     access_type: "write_only",
//     value_width: 8,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadWrite8C8B32 {
//     access_type: "read_write",
//     value_width: 8,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadOnly16C8B32 {
//     access_type: "read_only",
//     value_width: 16,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(WriteOnly16C8B32 {
//     access_type: "write_only",
//     value_width: 16,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadWrite16C8B32 {
//     access_type: "read_write",
//     value_width: 16,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadOnly32C8B32 {
//     access_type: "read_only",
//     value_width: 32,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(WriteOnly32C8B32 {
//     access_type: "write_only",
//     value_width: 32,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadWrite32C8B32 {
//     access_type: "read_write",
//     value_width: 32,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadOnly64C8B32 {
//     access_type: "read_only",
//     value_width: 64,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(WriteOnly64C8B32 {
//     access_type: "write_only",
//     value_width: 64,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadWrite64C8B32 {
//     access_type: "read_write",
//     value_width: 64,
//     wishbone_data_width: 8,
//     base_width: 32,
//     endianness: "big",
// });
//
//
// litex_register_abstraction!(ReadOnly8C32B32 {
//     access_type: "read_only",
//     value_width: 8,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(WriteOnly8C32B32 {
//     access_type: "write_only",
//     value_width: 8,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadWrite8C32B32 {
//     access_type: "read_write",
//     value_width: 8,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadOnly16C32B32 {
//     access_type: "read_only",
//     value_width: 16,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(WriteOnly16C32B32 {
//     access_type: "write_only",
//     value_width: 16,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadWrite16C32B32 {
//     access_type: "read_write",
//     value_width: 16,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadOnly32C32B32 {
//     access_type: "read_only",
//     value_width: 32,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(WriteOnly32C32B32 {
//     access_type: "write_only",
//     value_width: 32,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadWrite32C32B32 {
//     access_type: "read_write",
//     value_width: 32,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadOnly64C32B32 {
//     access_type: "read_only",
//     value_width: 64,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(WriteOnly64C32B32 {
//     access_type: "write_only",
//     value_width: 64,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });
//
// litex_register_abstraction!(ReadWrite64C32B32 {
//     access_type: "read_write",
//     value_width: 64,
//     wishbone_data_width: 32,
//     base_width: 32,
//     endianness: "big",
// });

#[repr(C)]
pub struct ReadOnly8C8B32<N: RegisterLongName = ()> {
    reg_p0: TRReadOnly<u8>,
    _reserved_0: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u8> for ReadOnly8C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 8usize;
    #[inline]
    fn base_get(&self) -> u8 {
        let reg_p0_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p0.get());
        u8::from_be_bytes([reg_p0_val[0usize]])
    }
}
#[repr(C)]
pub struct WriteOnly8C8B32<N: RegisterLongName = ()> {
    reg_p0: TRWriteOnly<u8>,
    _reserved_0: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseWriteableRegister<u8> for WriteOnly8C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 8usize;
    #[inline]
    fn base_set(&self, value: u8) {
        let bytes: [u8; 1usize] = u8::to_be_bytes(value);
        self.reg_p0.set(u8::from_be_bytes([bytes[0usize]]));
    }
}
#[repr(C)]
pub struct ReadWrite8C8B32<N: RegisterLongName = ()> {
    reg_p0: TRReadWrite<u8>,
    _reserved_0: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u8> for ReadWrite8C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 8usize;
    #[inline]
    fn base_get(&self) -> u8 {
        let reg_p0_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p0.get());
        u8::from_be_bytes([reg_p0_val[0usize]])
    }
}
impl<N: RegisterLongName> BaseWriteableRegister<u8> for ReadWrite8C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 8usize;
    #[inline]
    fn base_set(&self, value: u8) {
        let bytes: [u8; 1usize] = u8::to_be_bytes(value);
        self.reg_p0.set(u8::from_be_bytes([bytes[0usize]]));
    }
}
#[repr(C)]
pub struct ReadOnly16C8B32<N: RegisterLongName = ()> {
    reg_p0: TRReadOnly<u8>,
    _reserved_0: [u8; 3usize],
    reg_p1: TRReadOnly<u8>,
    _reserved_1: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u16> for ReadOnly16C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 16usize;
    #[inline]
    fn base_get(&self) -> u16 {
        let reg_p0_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p0.get());
        let reg_p1_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p1.get());
        u16::from_be_bytes([reg_p0_val[0usize], reg_p1_val[0usize]])
    }
}
#[repr(C)]
pub struct WriteOnly16C8B32<N: RegisterLongName = ()> {
    reg_p0: TRWriteOnly<u8>,
    _reserved_0: [u8; 3usize],
    reg_p1: TRWriteOnly<u8>,
    _reserved_1: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseWriteableRegister<u16> for WriteOnly16C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 16usize;
    #[inline]
    fn base_set(&self, value: u16) {
        let bytes: [u8; 2usize] = u16::to_be_bytes(value);
        self.reg_p0.set(u8::from_be_bytes([bytes[0usize]]));
        self.reg_p1.set(u8::from_be_bytes([bytes[1usize]]));
    }
}
#[repr(C)]
pub struct ReadWrite16C8B32<N: RegisterLongName = ()> {
    reg_p0: TRReadWrite<u8>,
    _reserved_0: [u8; 3usize],
    reg_p1: TRReadWrite<u8>,
    _reserved_1: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u16> for ReadWrite16C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 16usize;
    #[inline]
    fn base_get(&self) -> u16 {
        let reg_p0_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p0.get());
        let reg_p1_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p1.get());
        u16::from_be_bytes([reg_p0_val[0usize], reg_p1_val[0usize]])
    }
}
impl<N: RegisterLongName> BaseWriteableRegister<u16> for ReadWrite16C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 16usize;
    #[inline]
    fn base_set(&self, value: u16) {
        let bytes: [u8; 2usize] = u16::to_be_bytes(value);
        self.reg_p0.set(u8::from_be_bytes([bytes[0usize]]));
        self.reg_p1.set(u8::from_be_bytes([bytes[1usize]]));
    }
}
#[repr(C)]
pub struct ReadOnly32C8B32<N: RegisterLongName = ()> {
    reg_p0: TRReadOnly<u8>,
    _reserved_0: [u8; 3usize],
    reg_p1: TRReadOnly<u8>,
    _reserved_1: [u8; 3usize],
    reg_p2: TRReadOnly<u8>,
    _reserved_2: [u8; 3usize],
    reg_p3: TRReadOnly<u8>,
    _reserved_3: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u32> for ReadOnly32C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 32usize;
    #[inline]
    fn base_get(&self) -> u32 {
        let reg_p0_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p0.get());
        let reg_p1_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p1.get());
        let reg_p2_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p2.get());
        let reg_p3_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p3.get());
        u32::from_be_bytes([
            reg_p0_val[0usize],
            reg_p1_val[0usize],
            reg_p2_val[0usize],
            reg_p3_val[0usize],
        ])
    }
}
#[repr(C)]
pub struct WriteOnly32C8B32<N: RegisterLongName = ()> {
    reg_p0: TRWriteOnly<u8>,
    _reserved_0: [u8; 3usize],
    reg_p1: TRWriteOnly<u8>,
    _reserved_1: [u8; 3usize],
    reg_p2: TRWriteOnly<u8>,
    _reserved_2: [u8; 3usize],
    reg_p3: TRWriteOnly<u8>,
    _reserved_3: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseWriteableRegister<u32> for WriteOnly32C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 32usize;
    #[inline]
    fn base_set(&self, value: u32) {
        let bytes: [u8; 4usize] = u32::to_be_bytes(value);
        self.reg_p0.set(u8::from_be_bytes([bytes[0usize]]));
        self.reg_p1.set(u8::from_be_bytes([bytes[1usize]]));
        self.reg_p2.set(u8::from_be_bytes([bytes[2usize]]));
        self.reg_p3.set(u8::from_be_bytes([bytes[3usize]]));
    }
}
#[repr(C)]
pub struct ReadWrite32C8B32<N: RegisterLongName = ()> {
    reg_p0: TRReadWrite<u8>,
    _reserved_0: [u8; 3usize],
    reg_p1: TRReadWrite<u8>,
    _reserved_1: [u8; 3usize],
    reg_p2: TRReadWrite<u8>,
    _reserved_2: [u8; 3usize],
    reg_p3: TRReadWrite<u8>,
    _reserved_3: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u32> for ReadWrite32C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 32usize;
    #[inline]
    fn base_get(&self) -> u32 {
        let reg_p0_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p0.get());
        let reg_p1_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p1.get());
        let reg_p2_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p2.get());
        let reg_p3_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p3.get());
        u32::from_be_bytes([
            reg_p0_val[0usize],
            reg_p1_val[0usize],
            reg_p2_val[0usize],
            reg_p3_val[0usize],
        ])
    }
}
impl<N: RegisterLongName> BaseWriteableRegister<u32> for ReadWrite32C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 32usize;
    #[inline]
    fn base_set(&self, value: u32) {
        let bytes: [u8; 4usize] = u32::to_be_bytes(value);
        self.reg_p0.set(u8::from_be_bytes([bytes[0usize]]));
        self.reg_p1.set(u8::from_be_bytes([bytes[1usize]]));
        self.reg_p2.set(u8::from_be_bytes([bytes[2usize]]));
        self.reg_p3.set(u8::from_be_bytes([bytes[3usize]]));
    }
}
#[repr(C)]
pub struct ReadOnly64C8B32<N: RegisterLongName = ()> {
    reg_p0: TRReadOnly<u8>,
    _reserved_0: [u8; 3usize],
    reg_p1: TRReadOnly<u8>,
    _reserved_1: [u8; 3usize],
    reg_p2: TRReadOnly<u8>,
    _reserved_2: [u8; 3usize],
    reg_p3: TRReadOnly<u8>,
    _reserved_3: [u8; 3usize],
    reg_p4: TRReadOnly<u8>,
    _reserved_4: [u8; 3usize],
    reg_p5: TRReadOnly<u8>,
    _reserved_5: [u8; 3usize],
    reg_p6: TRReadOnly<u8>,
    _reserved_6: [u8; 3usize],
    reg_p7: TRReadOnly<u8>,
    _reserved_7: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u64> for ReadOnly64C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 64usize;
    #[inline]
    fn base_get(&self) -> u64 {
        let reg_p0_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p0.get());
        let reg_p1_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p1.get());
        let reg_p2_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p2.get());
        let reg_p3_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p3.get());
        let reg_p4_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p4.get());
        let reg_p5_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p5.get());
        let reg_p6_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p6.get());
        let reg_p7_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p7.get());
        u64::from_be_bytes([
            reg_p0_val[0usize],
            reg_p1_val[0usize],
            reg_p2_val[0usize],
            reg_p3_val[0usize],
            reg_p4_val[0usize],
            reg_p5_val[0usize],
            reg_p6_val[0usize],
            reg_p7_val[0usize],
        ])
    }
}
#[repr(C)]
pub struct WriteOnly64C8B32<N: RegisterLongName = ()> {
    reg_p0: TRWriteOnly<u8>,
    _reserved_0: [u8; 3usize],
    reg_p1: TRWriteOnly<u8>,
    _reserved_1: [u8; 3usize],
    reg_p2: TRWriteOnly<u8>,
    _reserved_2: [u8; 3usize],
    reg_p3: TRWriteOnly<u8>,
    _reserved_3: [u8; 3usize],
    reg_p4: TRWriteOnly<u8>,
    _reserved_4: [u8; 3usize],
    reg_p5: TRWriteOnly<u8>,
    _reserved_5: [u8; 3usize],
    reg_p6: TRWriteOnly<u8>,
    _reserved_6: [u8; 3usize],
    reg_p7: TRWriteOnly<u8>,
    _reserved_7: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseWriteableRegister<u64> for WriteOnly64C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 64usize;
    #[inline]
    fn base_set(&self, value: u64) {
        let bytes: [u8; 8usize] = u64::to_be_bytes(value);
        self.reg_p0.set(u8::from_be_bytes([bytes[0usize]]));
        self.reg_p1.set(u8::from_be_bytes([bytes[1usize]]));
        self.reg_p2.set(u8::from_be_bytes([bytes[2usize]]));
        self.reg_p3.set(u8::from_be_bytes([bytes[3usize]]));
        self.reg_p4.set(u8::from_be_bytes([bytes[4usize]]));
        self.reg_p5.set(u8::from_be_bytes([bytes[5usize]]));
        self.reg_p6.set(u8::from_be_bytes([bytes[6usize]]));
        self.reg_p7.set(u8::from_be_bytes([bytes[7usize]]));
    }
}
#[repr(C)]
pub struct ReadWrite64C8B32<N: RegisterLongName = ()> {
    reg_p0: TRReadWrite<u8>,
    _reserved_0: [u8; 3usize],
    reg_p1: TRReadWrite<u8>,
    _reserved_1: [u8; 3usize],
    reg_p2: TRReadWrite<u8>,
    _reserved_2: [u8; 3usize],
    reg_p3: TRReadWrite<u8>,
    _reserved_3: [u8; 3usize],
    reg_p4: TRReadWrite<u8>,
    _reserved_4: [u8; 3usize],
    reg_p5: TRReadWrite<u8>,
    _reserved_5: [u8; 3usize],
    reg_p6: TRReadWrite<u8>,
    _reserved_6: [u8; 3usize],
    reg_p7: TRReadWrite<u8>,
    _reserved_7: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u64> for ReadWrite64C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 64usize;
    #[inline]
    fn base_get(&self) -> u64 {
        let reg_p0_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p0.get());
        let reg_p1_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p1.get());
        let reg_p2_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p2.get());
        let reg_p3_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p3.get());
        let reg_p4_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p4.get());
        let reg_p5_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p5.get());
        let reg_p6_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p6.get());
        let reg_p7_val: [u8; 1usize] = u8::to_be_bytes(self.reg_p7.get());
        u64::from_be_bytes([
            reg_p0_val[0usize],
            reg_p1_val[0usize],
            reg_p2_val[0usize],
            reg_p3_val[0usize],
            reg_p4_val[0usize],
            reg_p5_val[0usize],
            reg_p6_val[0usize],
            reg_p7_val[0usize],
        ])
    }
}
impl<N: RegisterLongName> BaseWriteableRegister<u64> for ReadWrite64C8B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 64usize;
    #[inline]
    fn base_set(&self, value: u64) {
        let bytes: [u8; 8usize] = u64::to_be_bytes(value);
        self.reg_p0.set(u8::from_be_bytes([bytes[0usize]]));
        self.reg_p1.set(u8::from_be_bytes([bytes[1usize]]));
        self.reg_p2.set(u8::from_be_bytes([bytes[2usize]]));
        self.reg_p3.set(u8::from_be_bytes([bytes[3usize]]));
        self.reg_p4.set(u8::from_be_bytes([bytes[4usize]]));
        self.reg_p5.set(u8::from_be_bytes([bytes[5usize]]));
        self.reg_p6.set(u8::from_be_bytes([bytes[6usize]]));
        self.reg_p7.set(u8::from_be_bytes([bytes[7usize]]));
    }
}
#[repr(C)]
pub struct ReadOnly8C32B32<N: RegisterLongName = ()> {
    reg_p0: TRReadOnly<u8>,
    _reserved_0: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u8> for ReadOnly8C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 8usize;
    fn base_get(&self) -> u8 {
        self.reg_p0.get()
    }
}
#[repr(C)]
pub struct WriteOnly8C32B32<N: RegisterLongName = ()> {
    reg_p0: TRWriteOnly<u8>,
    _reserved_0: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseWriteableRegister<u8> for WriteOnly8C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 8usize;
    #[inline]
    fn base_set(&self, value: u8) {
        self.reg_p0.set(value)
    }
}
#[repr(C)]
pub struct ReadWrite8C32B32<N: RegisterLongName = ()> {
    reg_p0: TRReadWrite<u8>,
    _reserved_0: [u8; 3usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u8> for ReadWrite8C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 8usize;
    fn base_get(&self) -> u8 {
        self.reg_p0.get()
    }
}
impl<N: RegisterLongName> BaseWriteableRegister<u8> for ReadWrite8C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 8usize;
    #[inline]
    fn base_set(&self, value: u8) {
        self.reg_p0.set(value)
    }
}
#[repr(C)]
pub struct ReadOnly16C32B32<N: RegisterLongName = ()> {
    reg_p0: TRReadOnly<u16>,
    _reserved_0: [u8; 2usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u16> for ReadOnly16C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 16usize;
    fn base_get(&self) -> u16 {
        self.reg_p0.get()
    }
}
#[repr(C)]
pub struct WriteOnly16C32B32<N: RegisterLongName = ()> {
    reg_p0: TRWriteOnly<u16>,
    _reserved_0: [u8; 2usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseWriteableRegister<u16> for WriteOnly16C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 16usize;
    #[inline]
    fn base_set(&self, value: u16) {
        self.reg_p0.set(value)
    }
}
#[repr(C)]
pub struct ReadWrite16C32B32<N: RegisterLongName = ()> {
    reg_p0: TRReadWrite<u16>,
    _reserved_0: [u8; 2usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u16> for ReadWrite16C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 16usize;
    fn base_get(&self) -> u16 {
        self.reg_p0.get()
    }
}
impl<N: RegisterLongName> BaseWriteableRegister<u16> for ReadWrite16C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 16usize;
    #[inline]
    fn base_set(&self, value: u16) {
        self.reg_p0.set(value)
    }
}
#[repr(C)]
pub struct ReadOnly32C32B32<N: RegisterLongName = ()> {
    reg_p0: TRReadOnly<u32>,
    _reserved_0: [u8; 0usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u32> for ReadOnly32C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 32usize;
    #[inline]
    fn base_get(&self) -> u32 {
        let reg_p0_val: [u8; 4usize] = u32::to_be_bytes(self.reg_p0.get());
        u32::from_be_bytes([
            reg_p0_val[0usize],
            reg_p0_val[1usize],
            reg_p0_val[2usize],
            reg_p0_val[3usize],
        ])
    }
}
#[repr(C)]
pub struct WriteOnly32C32B32<N: RegisterLongName = ()> {
    reg_p0: TRWriteOnly<u32>,
    _reserved_0: [u8; 0usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseWriteableRegister<u32> for WriteOnly32C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 32usize;
    #[inline]
    fn base_set(&self, value: u32) {
        let bytes: [u8; 4usize] = u32::to_be_bytes(value);
        self.reg_p0.set(u32::from_be_bytes([
            bytes[0usize],
            bytes[1usize],
            bytes[2usize],
            bytes[3usize],
        ]));
    }
}
#[repr(C)]
pub struct ReadWrite32C32B32<N: RegisterLongName = ()> {
    reg_p0: TRReadWrite<u32>,
    _reserved_0: [u8; 0usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u32> for ReadWrite32C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 32usize;
    #[inline]
    fn base_get(&self) -> u32 {
        let reg_p0_val: [u8; 4usize] = u32::to_be_bytes(self.reg_p0.get());
        u32::from_be_bytes([
            reg_p0_val[0usize],
            reg_p0_val[1usize],
            reg_p0_val[2usize],
            reg_p0_val[3usize],
        ])
    }
}
impl<N: RegisterLongName> BaseWriteableRegister<u32> for ReadWrite32C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 32usize;
    #[inline]
    fn base_set(&self, value: u32) {
        let bytes: [u8; 4usize] = u32::to_be_bytes(value);
        self.reg_p0.set(u32::from_be_bytes([
            bytes[0usize],
            bytes[1usize],
            bytes[2usize],
            bytes[3usize],
        ]));
    }
}
#[repr(C)]
pub struct ReadOnly64C32B32<N: RegisterLongName = ()> {
    reg_p0: TRReadOnly<u32>,
    _reserved_0: [u8; 0usize],
    reg_p1: TRReadOnly<u32>,
    _reserved_1: [u8; 0usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u64> for ReadOnly64C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 64usize;
    #[inline]
    fn base_get(&self) -> u64 {
        let reg_p0_val: [u8; 4usize] = u32::to_be_bytes(self.reg_p0.get());
        let reg_p1_val: [u8; 4usize] = u32::to_be_bytes(self.reg_p1.get());
        u64::from_be_bytes([
            reg_p0_val[0usize],
            reg_p0_val[1usize],
            reg_p0_val[2usize],
            reg_p0_val[3usize],
            reg_p1_val[0usize],
            reg_p1_val[1usize],
            reg_p1_val[2usize],
            reg_p1_val[3usize],
        ])
    }
}
#[repr(C)]
pub struct WriteOnly64C32B32<N: RegisterLongName = ()> {
    reg_p0: TRWriteOnly<u32>,
    _reserved_0: [u8; 0usize],
    reg_p1: TRWriteOnly<u32>,
    _reserved_1: [u8; 0usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseWriteableRegister<u64> for WriteOnly64C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 64usize;
    #[inline]
    fn base_set(&self, value: u64) {
        let bytes: [u8; 8usize] = u64::to_be_bytes(value);
        self.reg_p0.set(u32::from_be_bytes([
            bytes[0usize],
            bytes[1usize],
            bytes[2usize],
            bytes[3usize],
        ]));
        self.reg_p1.set(u32::from_be_bytes([
            bytes[4usize],
            bytes[5usize],
            bytes[6usize],
            bytes[7usize],
        ]));
    }
}
#[repr(C)]
pub struct ReadWrite64C32B32<N: RegisterLongName = ()> {
    reg_p0: TRReadWrite<u32>,
    _reserved_0: [u8; 0usize],
    reg_p1: TRReadWrite<u32>,
    _reserved_1: [u8; 0usize],
    _regname: PhantomData<N>,
}
impl<N: RegisterLongName> BaseReadableRegister<u64> for ReadWrite64C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 64usize;
    #[inline]
    fn base_get(&self) -> u64 {
        let reg_p0_val: [u8; 4usize] = u32::to_be_bytes(self.reg_p0.get());
        let reg_p1_val: [u8; 4usize] = u32::to_be_bytes(self.reg_p1.get());
        u64::from_be_bytes([
            reg_p0_val[0usize],
            reg_p0_val[1usize],
            reg_p0_val[2usize],
            reg_p0_val[3usize],
            reg_p1_val[0usize],
            reg_p1_val[1usize],
            reg_p1_val[2usize],
            reg_p1_val[3usize],
        ])
    }
}
impl<N: RegisterLongName> BaseWriteableRegister<u64> for ReadWrite64C32B32<N> {
    type Reg = N;
    const REG_WIDTH: usize = 64usize;
    #[inline]
    fn base_set(&self, value: u64) {
        let bytes: [u8; 8usize] = u64::to_be_bytes(value);
        self.reg_p0.set(u32::from_be_bytes([
            bytes[0usize],
            bytes[1usize],
            bytes[2usize],
            bytes[3usize],
        ]));
        self.reg_p1.set(u32::from_be_bytes([
            bytes[4usize],
            bytes[5usize],
            bytes[6usize],
            bytes[7usize],
        ]));
    }
}
