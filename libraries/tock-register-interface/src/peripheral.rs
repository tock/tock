// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

/// Macro that allows for the definition of memory-mapped I/O (MMIO)
/// peripherals. See `README.md` for a brief summary; this documentation is a
/// more in-depth explanation of `peripheral!`'s capabilities.
///
/// # Basic Registers
/// The following `peripheral!` invocation:
/// ```
/// tock_registers::peripheral! {
///     /// Documentation for `Foo` goes here.
///     Foo {
///         // A very basic register, named status. It is located at offset 0x0
///         // (it is the first register in the peripheral), and contains a u16
///         // that can be safely read but not written to.
///         0x0 => status: u16 { Read },
///
///         // A read-write register, located at offset 0x2, named ctrl. This
///         // register contains a u16.
///         0x2 => ctrl: u16 { Read, Write },
///
///         // Fields can have doc comments, e.g.:
///         /// A write-only register.
///         0x4 => transmit: u8 { Write },
///
///         // A write-only array register named leds. This register has the same
///         // total size as ctrl (2 bytes). The difference between leds and
///         // ctrl is that leds is read or written one byte at a time, whereas
///         // ctrl is entirely read or written in a single operation.
///         0x5 => leds: [u8; 2] { Read, Write },
///     }
/// }
/// ```
/// generates a trait that looks like the following:
/// ```
/// #[allow(non_camel_case_types)]
/// trait Foo {
///     type status<'s>: tock_registers::Register<DataType = u16>
///         + tock_registers::Read<LongName = ()>
///     where
///         Self: 's;
///     fn status(&self) -> Self::status<'_>;
///
///     type ctrl<'s>: tock_registers::Register<DataType = u16>
///         + tock_registers::Read<LongName = ()>
///         + tock_registers::Write<LongName = ()>
///     where
///         Self: 's;
///     fn ctrl(&self) -> Self::ctrl<'_>;
///
///     type transmit<'s>: tock_registers::Register<DataType = u8>
///         + tock_registers::Write<LongName = ()>
///     where
///         Self: 's;
///     fn transmit(&self) -> Self::transmit<'_>;
///
///     type leds<'s>: tock_registers::Register<DataType = [u8; 2]>
///         + tock_registers::Read<LongName = ()>
///         + tock_registers::Write<LongName = ()>
///     where
///         Self: 's;
///     fn leds(&self) -> Self::leds<'_>;
/// }
/// ```
/// and an implementation of the trait for use on real systems (as opposed to
/// unit test environments):
/// ```ignore
/// #[derive(Clone, Copy)]
/// struct RealFoo<P: tock_registers::MmioPointer> { ... }
/// impl<P: tock_registers::MmioPointer> Foo for RealFoo<P> { ... }
/// ```
///
/// # Registers with bitfields (LongNames)
/// Registers can have LongNames, which specify the values of bitfields they
/// contain:
/// ```ignore
/// tock_registers::register_bitfields![u32,
///     Control [ ... ]
///     Pin [ ... ]
///     Status [ ... ]
///     Trigger [ ... ]
/// ];
/// tock_registers::peripheral! {
///     Foo {
///         // A bitfield is specified by putting the bitfield type in parenthesis
///         // after the register's data type:
///         0x0 => control: u32(Control::Register) { Write },
///
///         // Array registers can have bitfields as well. The bitfield applies to
///         // each register in the array.
///         0x4 => pins: [u8; 4](Pin::Register) { Read, Write },
///
///         // Registers can have different bitfields for read operations than for
///         // write operations. In that case, specify the register's LongName on
///         // the operation rather than the
///         0x8 => aliased: u32 { Read(Status::Register), Write(Trigger::Register) },
///     }
/// }
/// ```
/// Specifying bitfields changes the LongNames in the generated trait:
/// ```ignore
/// trait Foo {
///     type control<'s> = tock_registers::Register<DataType = u32>
///         + tock_registers::Write<LongName = Control::Register>
///     where
///         Self: 's;
///     fn control(&self) -> Self::control<'_>;
///
///     type pins<'s> = tock_registers::Register<DataType = [u8; 4]>
///         + tock_registers::Read<LongName = Pins::Register>
///         + tock_registers::Write<LongName = Pins::Register>
///     where
///         Self: 's;
///     fn pins(&self) -> Self::pins<'_>;
///
///     type aliased<'s> = tock_registers::Register<DataType = u32>
///         + tock_registers::Read<LongName = Status::Register>
///         + tock_registers::Write<LongName = Trigger::Register>
///     where
///         Self: 's;
///     fn aliased(&self) -> Self::aliased<'_>;
/// }
/// ```
///
/// # Padding
/// If there is a gap between registers in a peripheral, you must declare that
/// gap as a padding field:
/// ```
/// tock_registers::peripheral! {
///     Foo {
///         0x0 => register_a: u8 {},
///
///         // The padding field; specified by writing _ in place of the name
///         // and omitting the data type and operation list.
///         0x1 => _,
///
///         // The offset of the next field determines the size of the padding
///         // (in this case, the padding is two bytes).
///         0x3 => register_b: u8 {},
///     }
/// }
/// ```
/// Padding fields are ommitted from the generated trait.
///
/// # Blanket Deref impl
/// `tock_registers::peripheral!` emits a blanket implementation of the
/// peripheral trait for types that `Deref` to an implementation of the
/// peripheral:
/// ```ignore
/// impl<T: core::ops::Deref> Foo for T where T::Target: Foo {
///     ...
/// }
/// ```
/// Using this blanket impl, a driver that looks like:
/// ```ignore
/// struct FooDriver<F: Foo> {
///     foo: F,
/// }
/// ```
/// Can be used as either a `FooDriver<&FakeFoo>` or a `FooDriver<Rc<FakeFoo>>`
/// in unit tests.
///
/// # Bus Adapters
/// If you add the `#[allow_bus_adapter]` attribute to a peripheral:
/// ```
/// tock_registers::peripheral! {
///     #[allow_bus_adapter]
///     Foo {
///         0x0 => ctrl: u16 { Write },
///         // Since the offset of register changes depending on the passed bus
///         // adapter, you can have the offset be inferred by specifying _ as
///         // the offset:
///         _ => status: u8 { Read },
///     }
/// }
/// ```
/// the generated `RealFoo` structure will gain a
/// `B: tock_registers::BusAdapter` generic argument:
/// ```ignore
/// #[derive(Clone, Copy)]
/// struct RealFoo<
///     P: tock_registers::MmioPointer,
///     B: tock_registers::BusAdapter<u8> = tock_registers::DirectBus>
///     { ... }
/// ```
/// This argument is used to support unusual chip designs (mainly some LiteX
/// designs).
///
/// The above example also showed inferred offsets, which are valid in other
/// contexts as well. Note that a padding field cannot be followed by a field
/// with an inferred offset.
///
/// # Generated `macro_rules!` macro
/// In addition to the `Foo` trait and `RealFoo` trait, `peripheral!` also
/// generates a `macro_rules!` macro with the same name as the trait:
/// ```ignore
/// #[macro_export]
/// macro_rules! Foo { ... }
/// ```
/// This is currently unused, but is emitted because we may want to use it in
/// the future, and adding it later would be a breaking change.
///
/// # Unsafe operations
/// Registers can be unsafe to use:
/// ```
/// tock_registers::peripheral! {
///     Foo {
///         // A write-only register that is unsafe to write.
///         0x0 => a: u8 { UnsafeWrite },
///
///         // A register that can safely be read from or unsafely be written to
///         0x1 => b: u8 { Read, UnsafeWrite },
///
///         // A register can also be unsafe to read (for use if reading it
///         // triggers a memory-unsafe operation).
///         0x2 => c: u8 { UnsafeRead },
///
///         // Other possible combinations:
///         0x3 => d: u8 { UnsafeRead, Write },
///         0x4 => e: u8 { UnsafeRead, UnsafeWrite },
///
///         // Note that a register cannot be both UnsafeRead and Read; there is
///         // a maximum of one read-type operation allowed per register. The
///         // same applies to UnsafeWrite and Write.
///     }
/// }
/// ```
/// These registers implement the [`UnsafeRead`] and [`UnsafeWrite`] traits
/// instead of their safe equivalents.
///
/// # Doc comments and `cfg` attributes
/// Peripheral declarations can have documentation comments and `cfg` attributes:
/// ```
/// tock_registers::peripheral! {
///     /// This doc comment will be applied to the generated `Foo` trait, so it
///     /// will show up in `Foo`'s Rustdoc.
///     // This peripheral is only present if the has_foo feature is enabled.
///     #[cfg(feature = "has_foo")]
///     Foo {
///         /// This doc comment will be applied to the `Foo::ctrl` accessor
///         /// method, and show up in its Rustdoc.
///         0x0 => ctrl: u8 { Write },
///
///         // This register is only present if the bounce feature is enabled.
///         #[cfg(feature = "bounce")]
///         0x1 => bounce: u8 { Read, Write },
///
///         // Often, you will need to place a register with an inferred offset
///         // (or padding) after a register with a `cfg` attribute, because the
///         // offset may change depending on the crate's enabled features.
///         _ => next_register: u8 {},
///     }
/// }
/// ```
///
/// # `#[real_name]` attribute
/// The `Real*` type can be renamed, in case it collides with another name in
/// scope:
/// ```ignore
/// tock_registers::peripheral! {
///     // This will generate trait Foo and struct ActualFoo, rather than
///     // generating struct RealFoo.
///     #[real(ActualFoo)]
///     Foo { ... }
/// }
/// ```

#[macro_export]
macro_rules! peripheral {
    [$(#[$attr:meta])* $visibility:vis $name:ident {$($fields:tt)*}] => {
        $crate::reexport::peripheral!($crate; $(#[$attr])* $visibility $name { $($fields)* });
    }
}
