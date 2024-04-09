# Tock Register Interface

This crate provides an interface and types for defining and
manipulating registers and bitfields.

## Defining peripherals

Note: This README introduces the `peripheral!` macro and describes its
most-commonly-used features. `peripheral!` has several more pieces of
functionality that are not described in this document; see its Rustdoc for
complete documentation.

An MMIO peripheral is defined using the `peripheral!` macro:

```rust
tock_registers::peripheral! {
    /// Documentation for `Registers`.
    Registers {
        // Control register: read-write. `u8` specifies the data type of the
        // memory-mapped register, and the `Control` parameter specifies how
        // bitfields within the register should be accessed.
        0x000 => cr: u8(Control::Register) { Read, Write },

        // Status register: read-only. Registers can have documentation
        // comments, like:
        /// Status register
        0x001 => s: u8(Status::Register) { Read },

        // Registers can have different sizes. The bitfield parameter is
        // optional:
        0x002 => byte0: u8 { Read, Write },
        0x003 => byte1: u8 { Read, Write },
        0x004 => short: u16 { Read, Write },

        // Empty space between registers must be marked with a padding field,
        // specified by naming the field _ and omitting the rest of the
        // specification. The length of the padding field will be inferred. In
        // practice, padding fields remove the check that two successive
        // registers are adjacent in the address space.
        0x006 => _,

        // If you have several adjacent registers, you can specify an array
        // register:
        0x00C => gpio_pins: [u32; 4] { Read, Write },

        // Array registers can have bitfields as well
        0x01C => port_ctrl: [u8; 4](PortCtrl::Register) { Read, Write },
    }
}
```

The `peripheral!` macro tests that you specified the fields (registers and
padding) in increasing order, with no gaps.

The macro generates a trait representing the peripheral with one method for each
register:

```rust
trait Registers {
    // Type aliases (cr, s, byte0, ...) omitted for brevity, they will be
    // explained later.
    fn cr(&self) -> Self::cr<'_> { ... }
    fn s(&self) -> Self::s<'_> { ... }
    fn byte0(&self) -> Self::byte0<'_> { ... }
    fn byte1(&self) -> Self::byte1<'_> { ... }
    fn short(&self) -> Self::short <'_>{ ... }
    fn gpio_pins(&self) -> Self::gpio_pins<'_> { ... }
    fn port_ctrl(&self) -> Self::port_ctrl<'_> { ... }
}
```

The types returned by the `Registers` methods implement traits defined in the
`tock_registers::register_traits` module. `peripheral!` also generates an
implementation of the trait for use on the real chip:

```rust
struct RealRegisters { ... }

impl Registers for RealRegisters { ... }
```

Drivers can use the trait to work with both the real hardware and with fake
hardware for unit testing:

```rust
struct Driver<R: Registers> {
    registers: R,
}

// Real systems will use Driver<RealRegisters>

// Unit tests will provide:
struct Fake { ... }
impl Registers for Fake {
    type cr<'s> = tock_registers::FakeRegister<'s, u8, Control::Register,
            tock_registers::Safe, tock_registers::Safe> where Self: 's;
    fn cr(&self) => Self::cr<'_> {
        tock_registers::FakeRegister::with_data(self)
            .on_read(|this| ...)
            .on_write(|this| ...)
    }

    /* One type and fn for each register */
}
// then use Driver<Fake>.
```

It also generates a `macro_rules!` macro with the same name as the trait:

```rust
#[macro_export]
macro_rules! Registers { ... }
```

This macro does not currently do anything, but is generated for future
compatibility.

## Defining bitfields

Bitfields are defined through the `register_bitfields!` macro:

```rust
register_bitfields! [
    // First parameter is the register width. Can be u8, u16, u32, or u64.
    u32,

    // Each subsequent parameter is a register abbreviation, its descriptive
    // name, and its associated bitfields.
    // The descriptive name defines this 'group' of bitfields. Only registers
    // defined as ReadWrite<_, Control::Register> can use these bitfields.
    Control [
        // Bitfields are defined as:
        // name OFFSET(shift) NUMBITS(num) [ /* optional values */ ]

        // This is a two-bit field which includes bits 4 and 5
        RANGE OFFSET(4) NUMBITS(2) [
            // Each of these defines a name for a value that the bitfield can be
            // written with or matched against. Note that this set is not exclusive--
            // the field can still be written with arbitrary constants.
            VeryHigh = 0,
            High = 1,
            Low = 2
        ],

        // A common case is single-bit bitfields, which usually just mean
        // 'enable' or 'disable' something.
        EN  OFFSET(3) NUMBITS(1) [],
        INT OFFSET(2) NUMBITS(1) []
    ],

    // Another example:
    // Status register
    Status [
        TXCOMPLETE  OFFSET(0) NUMBITS(1) [],
        TXINTERRUPT OFFSET(1) NUMBITS(1) [],
        RXCOMPLETE  OFFSET(2) NUMBITS(1) [],
        RXINTERRUPT OFFSET(3) NUMBITS(1) [],
        MODE        OFFSET(4) NUMBITS(3) [
            FullDuplex = 0,
            HalfDuplex = 1,
            Loopback = 2,
            Disabled = 3
        ],
        ERRORCOUNT OFFSET(6) NUMBITS(3) []
    ],

    // In a simple case, offset can just be a number, and the number of bits
    // is set to 1:
    InterruptFlags [
        UNDES   10,
        TXEMPTY  9,
        NSSR     8,
        OVRES    3,
        MODF     2,
        TDRE     1,
        RDRF     0
    ]
]
```

## Register Trait Bounds

Earlier, we omitted the bounds on the register types. For the following
peripheral definition:

```rust
tock_registers::peripheral! {
    Registers {
        0x0 => a: u8 { Read, Write },
        0x1 => b: u32(Ctrl::Register) { Read, Write },
    }
}
```

`peripheral!` generates:

```rust
trait Registers {
    type a<'s>: tock_registers::Register<DataType = u8> +
            tock_registers::Read<LongName = ()> +
            tock_registers::Write<LongName = ()> where Self: 's;
    fn a(&self) -> Self::a<'_>;

    type b<'s>: tock_registers::Register<DataType = u32> +
            tock_registers::Read<LongName = Ctrl::Register> +
            tock_registers::Write<LongName = Ctrl::Register> where Self: 's;
    fn b(&self) -> Self::b<'_>;
}

struct RealRegisters<M: ...> { ... }

impl<M: ...> Registers for RealRegisters<M> {
    type a<'s> = tock_registers::RealRegister<
        's,
        M,
        u8,
        (),
        tock_registers::Safe,
        tock_registers::Safe,
    >;
    fn a(&self) -> Self::a<'_> { ... }

    type b<'s> = tock_registers::RealRegister<
        's,
        M,
        u32,
        Ctrl::Register,
        tock_registers::Safe,
        tock_registers::Safe,
    >;
    fn b(&self) -> Self::b<'_> { ... }
}
```

In short: all registers have a `Register<...>` bound. If a register as `Read`,
`Write`, `UnsafeRead`, and/or `UnsafeWrite`, then it will have corresponding
trait bounds.

## Example: Using registers and bitfields

Assuming we have defined bitfields as in the previous section, and the following `peripheral!` invocation:

```rust
tock_registers::peripheral! {
    Registers {
        // Bitfields are specified by putting them in parenthesis after the
        // register's data type.
        0x000 => cr: u8(Control::Register) { Read, Write },
        0x001 => s: u8(Status::Register) { Read },
        0x002 => byte0: u8 { Read, Write },
        0x003 => byte1: u8 { Read, Write },
        0x004 => short: u16 { Read, Write },
        0x006 => _,
        0x00C => array: [u32; 4] { Read, Write },
    }
}
```

and `registers` is a `&RealRegisters`:

```rust
// -----------------------------------------------------------------------------
// RAW ACCESS
// -----------------------------------------------------------------------------

// Get or set the raw value of the register directly. Nothing fancy:
registers.cr().set(registers.cr().get() + 1);


// -----------------------------------------------------------------------------
// READ
// -----------------------------------------------------------------------------

// `range` will contain the value of the RANGE field, e.g. 0, 1, 2, or 3.
// The type annotation is not necessary, but provided for clarity here.
let range: u8 = registers.cr().read(Control::RANGE);

// Or one can read `range` as a enum and `match` over it.
let range = registers.cr().read_as_enum(Control::RANGE);
match range {
    Some(Control::RANGE::Value::VeryHigh) => { /* ... */ }
    Some(Control::RANGE::Value::High) => { /* ... */ }
    Some(Control::RANGE::Value::Low) => { /* ... */ }

    None => unreachable!("invalid value")
}

// `en` will be 0 or 1
let en: u8 = registers.cr().read(Control::EN);


// -----------------------------------------------------------------------------
// MODIFY
// -----------------------------------------------------------------------------

// Write a value to a bitfield without altering the values in other fields:
registers.cr().modify(Control::RANGE.val(2)); // Leaves EN, INT unchanged

// Named constants can be used instead of the raw values:
registers.cr().modify(Control::RANGE::VeryHigh);

// Enum values can also be used:
registers.cr().modify(Control::RANGE::Value::VeryHigh.into())

// Another example of writing a field with a raw value:
registers.cr().modify(Control::EN.val(0)); // Leaves RANGE, INT unchanged

// For one-bit fields, the named values SET and CLEAR are automatically
// defined:
registers.cr().modify(Control::EN::SET);

// Write multiple values at once, without altering other fields:
registers.cr().modify(Control::EN::CLEAR + Control::RANGE::Low); // INT unchanged

// Any number of non-overlapping fields can be combined:
registers.cr().modify(Control::EN::CLEAR + Control::RANGE::High + CR::INT::SET);

// In some cases (such as a protected register) .modify() may not be appropriate.
// To enable updating a register without coupling the read and write, use
// modify_no_read():
let original = registers.cr().extract();
registers.cr().modify_no_read(original, Control::EN::CLEAR);


// -----------------------------------------------------------------------------
// WRITE
// -----------------------------------------------------------------------------

// Same interface as modify, except that all unspecified fields are overwritten to zero.
registers.cr().write(Control::RANGE.val(1)); // implictly sets all other bits to zero

// -----------------------------------------------------------------------------
// BITFLAGS
// -----------------------------------------------------------------------------

// For one-bit fields, easily check if they are set or clear:
let txcomplete: bool = registers.s().is_set(Status::TXCOMPLETE);

// -----------------------------------------------------------------------------
// MATCHING
// -----------------------------------------------------------------------------

// You can also query a specific register state easily with `matches_all` or
// `any_matching_bits_set` or `matches_any`:

// Doesn't care about the state of any field except TXCOMPLETE and MODE:
let ready: bool = registers.s().matches_all(Status::TXCOMPLETE:SET +
                                            Status::MODE::FullDuplex);

// This is very useful for awaiting for a specific condition:
while !registers.s().matches_all(Status::TXCOMPLETE::SET +
                                 Status::RXCOMPLETE::SET +
                                 Status::TXINTERRUPT::CLEAR) {}

// Or for checking whether any interrupts are enabled:
let any_ints = registers.s().any_matching_bits_set(Status::TXINTERRUPT + Status::RXINTERRUPT);

// Or for checking whether any completion states are cleared:
let any_cleared = registers.s().matches_any(
    &[Status::TXCOMPLETE::CLEAR, Status::RXCOMPLETE::CLEAR]);

// Or for checking if a multi-bit field matches one of several modes:
let sub_word_size = registers.s().matches_any(&[Size::Halfword, Size::Word]);

// Or for checking if any of several fields exactly match in the register:
let not_supported_mode = registers.s().matches_any(
    &[Status::Mode::HalfDuplex, Status::Mode::VARSYNC, Status::MODE::NOPARITY]);

// Also you can read a register with set of enumerated values as a enum and `match` over it:
let mode = registers.cr().read_as_enum(Status::MODE);

match mode {
    Some(Status::MODE::Value::FullDuplex) => { /* ... */ }
    Some(Status::MODE::Value::HalfDuplex) => { /* ... */ }

    None => unreachable!("invalid value")
}

// -----------------------------------------------------------------------------
// LOCAL COPY
// -----------------------------------------------------------------------------

// More complex code may want to read a register value once and then keep it in
// a local variable before using the normal register interface functions on the
// local copy.

// Create a copy of the register value as a local variable.
let local = registers.cr().extract();

// Now all the functions for a ReadOnly register work.
let txcomplete: bool = local.is_set(Status::TXCOMPLETE);

// -----------------------------------------------------------------------------
// In-Memory Registers
// -----------------------------------------------------------------------------

// In some cases, code may want to edit a memory location with all of the
// register features described above, but the actual memory location is not a
// fixed MMIO register but instead an arbitrary location in memory. If this
// location is then shared with the hardware (i.e. via DMA) then the code
// must do volatile reads and writes since the value may change without the
// software knowing. To support this, the library includes an `InMemoryRegister`
// type.

let control: InMemoryRegister<u32, Control::Register> = InMemoryRegister::new(0)
control.write(Contol::BYTE_COUNT.val(0) +
              Contol::ENABLE::Yes +
              Contol::LENGTH.val(10));
```

Note that `modify` performs exactly one volatile load and one volatile store,
`write` performs exactly one volatile store, and `read` performs exactly one
volatile load. Thus, you are ensured that a single call will set or query all
fields simultaneously.

## Performance

Examining the binaries while testing this interface, everything compiles
down to the optimal inlined bit twiddling instructions--in other words, there is
zero runtime cost, as far as an informal preliminary study has found.

## Nice type checking

This interface helps the compiler catch some common types of bugs via type checking.

If you define the bitfields for e.g. a control register, you can give them a
descriptive group name like `Control`. This group of bitfields will only work
with a register of the type `ReadWrite<_, Control>` (or `ReadOnly/WriteOnly`,
etc). For instance, if we have the bitfields and registers as defined above,

```rust
// This line compiles, because registers.cr is associated with the Control group
// of bitfields.
registers.cr().modify(Control::RANGE.val(1));

// This line will not compile, because registers.s is associated with the Status
// group, not the Control group.
let range = registers.s().read(Control::RANGE);
```

## Naming conventions

There are several related names in the register definitions. Below is a
description of the naming convention for each:

```rust
use tock_registers::registers::ReadWrite;

#[repr(C)]
struct Registers {
tock_registers::peripheral! {
    Registers {
        // The register name in the struct should be a lowercase version of the
        // register abbreviation, as written in the datasheet:
        0x0 => cr: u8(Control::Register) { Read, Write },
    }
}

register_bitfields! [
    u8,

    // The name should be the long descriptive register name,
    // camelcase, without the word 'register'.
    Control [
        // The field name should be the capitalized abbreviated
        // field name, as given in the datasheet.
        RANGE OFFSET(4) NUMBITS(3) [
            // Each of the field values should be camelcase,
            // as descriptive of their value as possible.
            VeryHigh = 0,
            High = 1,
            Low = 2
        ]
    ]
]
```

## Debug trait

By default, if you print the value of a register, you will get the raw value as a number.

How ever, you can use the `debug` method to get a more human readable output.

This is implemented in `LocalRegisterCopy` and in using `Debuggable` registers which is auto implemented with `Read`.

Example:

```rust
// Create a copy of the register value as a local variable.
let local = registers.cr().extract();

println!("cr: {:#?}", local.debug());
```

For example, if the value of the `Control` register is `0b0000_0100`, the output will be:

```rust
cr: Control {
    RANGE: VeryHigh,
    EN: 0,
    INT: 1
}
```

Similarly it works directly on the register:

```rust
// require `Debuggable` trait
use tock_registers::interfaces::Debuggable;

println!("cr: {:#?}", registers.cr().debug());
```
> Do note this will issue a read to the register once.
