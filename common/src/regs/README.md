# Register/Bitfield Interface for Tock Teensy Port

This module provides an interface for defining and manipulating memory mapped 
registers and bitfields. 

## Defining registers

The module provides three types for working with memory mapped registers: `RW`,
`RO`, and `WO`, providing read-write, read-only, and write-only functionality,
respectively.

Defining the registers is similar to the C-style approach, where each register
is a field in a packed struct:

```rust
use common::regs::{RO, RW, WO};


#[repr(C, packed)]
struct Registers {
    // Control register: read-write
    // The 'Control' parameter constrains this register to only use fields from
    // a certain group (defined below in the bitfields section).
    cr: RW<u8, Control>,

    // Status register: read-only
    s: RO<u8, Status>

    // Registers can be bytes, halfwords, or words:
    // Note that the second type parameter can be omitted, meaning that there
    // are no bitfields defined for these registers.
    byte0: RW<u8>,
    byte1: RW<u8>,
    short: RW<u16>,
    word: RW<u32>

    // Etc.
}
```

## Defining bitfields

Bitfields are defined through the `bitfields!` macro:

```rust
bitfields! [ 
    // First parameter is the register width for the bitfields. Can be u8, u16,
    // or u32.
    u8,

    // Each subsequent parameter is a register abbreviation, its descriptive
    // name, and its associated bitfields.
    // The descriptive name defines this 'group' of bitfields. Only registers
    // defined as RW<_, Control> can use these bitfields.
    CR Control [
        // Bitfields are defined as:
        // name (Mask(mask), shift) [ /* optional named values */ ]

        // This is a two-bit field which includes bits 4 and 5
        RANGE (Mask(0b11), 4) [
            // Each of these defines a name for a value that the bitfield can be 
            // written with or matched against. Note that this set is not exclusive--
            // the field can still be written with arbitrary constants.
            VeryHigh = 0,
            High = 1,
            Low = 2
        ],

        // A common case is single-bit bitfields, which usually just mean
        // 'enable' or 'disable' something. This syntax is a shorthand, defining
        // EN as a field over bit 3, and INT as a field over bit 2.
        EN 3 [],
        INT 2 []
    ],

    // Without the explanatory comments like above, bitfield definition is quite compact:
    // Status register
    S Status [
        TXCOMPLETE 0 [],
        TXINTERRUPT 1 [],
        RXCOMPLETE 2 [],
        RXINTERRUPT 3 [],
        MODE (Mask(0b11), 4) [
            FullDuplex = 0,
            HalfDuplex = 1,
            Loopback = 2,
            Disabled = 3
        ],
        ERRORCOUNT (Mask(0b11), 6) []
    ]
]
```


## Example: Using registers and bitfields

Assuming we have defined a `Registers` struct and the corresponding bitfields as
in the previous two sections. We also have an immutable reference to the 
`Registers` struct, named `regs`.

```rust
// -----------------------------------------------------------------------------
// RAW ACCESS
// -----------------------------------------------------------------------------

// Get or set the raw value of the register directly. Nothing fancy:
regs.cr.set(regs.cr.get() + 1);


// -----------------------------------------------------------------------------
// READ
// -----------------------------------------------------------------------------

// `range` will contain the unshifted value of the RANGE field, e.g. 0, 1, 2, or 3.
// The type annotation is not necessary, but provided for clarity here.
let range: u8 = regs.cr.read(CR::RANGE);

// `en` will be 0 or 1
let en: u8 = regs.cr.read(CR::EN);


// -----------------------------------------------------------------------------
// MODIFY
// -----------------------------------------------------------------------------

// Write a value to a bitfield without altering the values in other fields:
regs.cr.modify(CR::RANGE.val(2)); // Leaves EN, INT unchanged

// Named constants can be used instead of the raw values:
regs.cr.modify(CR::RANGE::VeryHigh);

// Another example of writing a field with a raw value:
regs.cr.modify(CR::EN.val(0)); // Leaves RANGE, INT unchanged

// For one-bit fields, the named values SET and CLEAR are automatically
// defined:
regs.cr.modify(CR::EN::SET);

// Write multiple values at once, without altering other fields:
regs.cr.modify(CR::EN::CLEAR + CR::RANGE::Low); // INT unchanged

// Any number of non-overlapping fields can be combined:
regs.cr.modify(CR::EN::CLEAR + CR::RANGE::High + CR::INT::SET);


// -----------------------------------------------------------------------------
// WRITE
// -----------------------------------------------------------------------------

// Same interface as modify, except that all unspecified fields are overwritten to zero.
regs.cr.write(CR::RANGE.val(1)); // implictly sets all other bits to zero

// -----------------------------------------------------------------------------
// BITFLAGS
// -----------------------------------------------------------------------------

// For one-bit fields, easily check if they are set or clear:
let txcomplete: bool = regs.s.is_set(S::TXCOMPLETE);

// -----------------------------------------------------------------------------
// MATCHING
// -----------------------------------------------------------------------------

// You can also query a specific register state easily with `matches`:

// Doesn't care about the state of any field except TXCOMPLETE and MODE:
let ready: bool = regs.s.matches(S::TXCOMPLETE:SET + 
                                 S::MODE::FullDuplex);

// This is very useful for awaiting for a specific condition:
while !regs.s.matches(S::TXCOMPLETE::SET + 
                      S::RXCOMPLETE::SET +
                      S::TXINTERRUPT::CLEAR) {}
```

Note that `modify` performs exactly one volatile load and one volatile store,
`write` performs exactly one volatile store, and `read` performs exactly one
volatile load. Thus, you are ensured that a single call will set or query all
fields simultaneously.

## Performance

TODO: specific study

Examining the binaries while testing this interface, everything compiles
down to the optimal inlined bit twiddling instructions--in other words, there is
zero runtime cost, as far as my informal preliminary study has found. I will
eventually be writing a more rigorous test to confirm this.

## Nice type checking

This interface helps the compiler catch some common types of bugs via type checking.

If you define the bitfields for eg a control register, you can give them a
descriptive group name like `Control`. This group of bitfields will only work with a 
register of the type `RW<_, Control>` (or `RO/WO`, etc). For instance, if we have 
the bitfields and registers as defined above,

```rust
// This line compiles, because CR and regs.cr are both associated with the
// Control group of bitfields.
regs.cr.modify(CR::RANGE.val(1));

// This line will not compile, because CR is associated with the Control group,
// while regs.s is associated with the Status group.
regs.s.modify(CR::RANGE.val(1));

```

## Usage in Tock Teensy port

All of the Teensy registers are defined in `boards/teensy/src/regs`. The
registers are used in almost every module in `boards/teensy/src/`
