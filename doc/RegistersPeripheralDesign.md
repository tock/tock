`tock_registers::peripheral!` Macro and Trait Interface Design
==============================================================

## Basic terminology

A `peripheral!` invocation looks like the following:

```rust
peripheral! {
    foo {
        0x00 => ctrl: u32 { read + write },
        0x04 => received: u8 { read },
    }
}
```

This invocation specifies two *registers*, `ctrl` and `received`. `ctrl`
implements two *operations*, `read` and `write`. `received` only implements
`read`.

## Core `tock_registers` structs

`tock_registers` exports the following structs:

```rust
#[repr(transparent)]
pub struct Register<const REL_ADDR: usize, Value, Peripheral, Accessor> {
    pub accessor: Accessor,
    _phantom: core::marker::PhantomData<(Value, Peripheral)>,
}

impl<Accessor> Register<_, _, _, Accessor> {
    fn new(accessor: Accessor) -> Self {
        Self {
            accessor,
            _phantom: Default::default(),
        }
    }
}

pub struct Real { _noconstruct: () }
```

## Operation implementation

An *operation* is implemented as a module that contains three traits:

```rust
pub mod read {
    pub trait Access<const REL_ADDR: usize, Value> {
        fn read(&self) -> Value;
    }

    pub trait Has<const REL_ADDR: usize, Value> {}

    pub trait Register<Value> {
        fn read(&self) -> Value;
    }

    impl<
            const REL_ADDR: usize,
            Value,
            Peripheral: Has<REL_ADDR, Value>,
            Accessor: Access<REL_ADDR, Value>,
        > Register<Value> for crate::tock_registers::Register<REL_ADDR, Peripheral, Accessor>
    {
        fn read(&self) -> Value {
            self.accessor.read()
        }
    }

    impl<const REL_ADDR: usize, Value> Access<REL_ADDR, Value> for tock_registers::Real {
        fn read(&self) -> Value {
            unsafe {
                core::ptr::read_volatile((self as *const Self as usize + REL_ADDR) as *const Value)
            }
        }
    }
}
```

## `peripheral!` expansion

Our example peripheral:

```rust
peripheral! {
    foo {
        0x00 => ctrl: u32 { read + write },
        0x04 => received: u8 { read },
    }
}
```

expands to:

```rust
mod foo {
    trait Accessor: Copy +
                    read::Access<0, u32> +
                    write::Access<0, u32> +
                    read::Access<4, u8> {}

    impl<A: Copy +
            read::Access<0, u32> +
            write::Access<0, u32> +
            read::Access<4, u8>>
    Accessor for A {}

    #[repr(C)]
    struct Registers<Accessor> {
        pub ctrl: tock_registers::Register<0, Self, Accessor>,
        pub received: tock_registers::Register<4, Self, Accessor>,
    }

    impl read::Has<0, u32> for Registers<_> {}
    impl write::Has<0, u32> for Registers<_> {}
    impl read::Has<4, u8> for Registers<_> {}

    impl<Accessor> Registers<Accessor> {
        // Used in unit tests
        pub fn new(accessor: Accessor) -> Self {
            Self {
                ctrl: Register::new(accessor),
                received: Register::new(accessor),
            }
        }
    }
}
```

## How do you use the generated registers?

```rust
peripheral! {
    foo {
        0x00 => ctrl: u32 { read + write },
        0x04 => received: u8 { read },
    }
}

// Called with A == tock_registers::Real in the real kernel, and a fake version
// of the Foo peripheral in unit tests.
fn use_foo<A: foo::Accessor>(instance: &'static foo::Registers<A>) -> u32 {
    use read::Register;

    foo.ctrl.read()
    // foo.ctrl.read() invokes read::Register::<u32>::read(), which calls
    // read::Access<0, u32> on foo.ctrl.accessor. If A is tock_registers::Real,
    // this performs a volatile memory read.
}
```

## Properties of this design

1. Allows unit testing -- a Foo test can implement `read::Access<0, u32>`,
   `write::Access<0, u32>`, and `read::Access<4, u8>` on a fake version of
   `Foo` and use that to test `use_foo`'s functionality.
2. Resolves the unsoundness with pointers pointing into MMIO memory.
   `foo::Registers::<tock_registers::Real>` is a zero-sized type, so a
   reference to it does not point to any data, so the compiler cannot insert
   arbitrary deferences to it.
3. Operations can be defined outside `tock_registers`. This design allows the
   `riscv-csr` crate to define its operations and retain the full functionality
   of `tock_registers` (including unit test functionality).
4. Hideously complex and hard to explain.

## Example driver implementation: rp2040 watchdog

Ported over from
[`chips/rp2040/src/watchdog.rs`](https://github.com/tock/tock/blob/master/chips/rp2040/src/watchdog.rs):

```rust
peripheral! {
    watchdog {
        /// Watchdog control
        /// The rst_wdsel register determines which subsystems are reset when th
        /// The watchdog can be triggered in software.
        0x000 => ctrl: u32 { read<CTRL::Register> + write<CTRL::Register> },
        /// Load the watchdog timer. The maximum setting is 0xffffff which corresponds to 0x
        0x004 => load: u32 { read + write },
        /// Logs the reason for the last reset. Both bits are zero for the case of a hardwar
        0x008 => reason: u32 { read<REASON::Register> + write<REASON::Register> },
        /// Scratch register. Information persists through soft reset of the chip.
        0x00C => scratch0: u32 { read<SCRATCH0::Register> + write<SCRATCH0::Register> },
        /// Scratch register. Information persists through soft reset of the chip.
        0x010 => scratch1: u32 { read<SCRATCH1::Register> + write<SCRATCH1::Register> },
        /// Scratch register. Information persists through soft reset of the chip.
        0x014 => scratch2: u32 { read<SCRATCH2::Register> + write<SCRATCH2::Register> },
        /// Scratch register. Information persists through soft reset of the chip.
        0x018 => scratch3: u32 { read<SCRATCH3::Register> + write<SCRATCH3::Register> },
        /// Scratch register. Information persists through soft reset of the chip.
        0x01C => scratch4: u32 { read<SCRATCH4::Register> + write<SCRATCH4::Register> },
        /// Scratch register. Information persists through soft reset of the chip.
        0x020 => scratch5: u32 { read<SCRATCH5::Register> + write<SCRATCH5::Register> },
        /// Scratch register. Information persists through soft reset of the chip.
        0x024 => scratch6: u32 { read<SCRATCH6::Register> + write<SCRATCH6::Register> },
        /// Scratch register. Information persists through soft reset of the chip.
        0x028 => scratch7: u32 { READ<SCRATCH7::Register> + write<SCRATCH7::Register> },
        /// Controls the tick generator
        0x02C => tick: u32 { read<TICK::Register> + write<TICK::Register> },
    }
}

register_bitfields![u32,
    CTRL [
        /// Trigger a watchdog reset
        TRIGGER OFFSET(31) NUMBITS(1) [],
        /// When not enabled the watchdog timer is paused
        ENABLE OFFSET(30) NUMBITS(1) [],
        /// Pause the watchdog timer when processor 1 is in debug mode
        PAUSE_DBG1 OFFSET(26) NUMBITS(1) [],
        /// Pause the watchdog timer when processor 0 is in debug mode
        PAUSE_DBG0 OFFSET(25) NUMBITS(1) [],
        /// Pause the watchdog timer when JTAG is accessing the bus fabric
        PAUSE_JTAG OFFSET(24) NUMBITS(1) [],
        /// Indicates the number of ticks / 2 (see errata RP2040-E1) before a watchdog reset
        TIME OFFSET(0) NUMBITS(24) []
    ],
    LOAD [

        LOAD OFFSET(0) NUMBITS(24) []
    ],
    REASON [

        FORCE OFFSET(1) NUMBITS(1) [],

        TIMER OFFSET(0) NUMBITS(1) []
    ],
    SCRATCH0 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH1 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH2 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH3 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH4 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH5 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH6 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    SCRATCH7 [
        VALUE OFFSET (0) NUMBITS (32) []
    ],
    TICK [
        /// Count down timer: the remaining number clk_tick cycles before the next tick is g
        COUNT OFFSET(11) NUMBITS(9) [],
        /// Is the tick generator running?
        RUNNING OFFSET(10) NUMBITS(1) [],
        /// start / stop tick generation
        ENABLE OFFSET(9) NUMBITS(1) [],
        /// Total number of clk_tick cycles before the next tick.
        CYCLES OFFSET(0) NUMBITS(9) []
    ]
];

pub struct Watchdog<'a, Accessor> {
    registers: &'a watchdog::Registers<Accessor>,
    resets: OptionalCell<&'a resets::Resets>,
}

impl<'a, Accessor: watchdog::Accessor> Watchdog<'a, Accessor> {
    use read::Register;
    use write::Register;

    // modify::Register is a not-yet-described trait that is implemented on all
    // register types that implement both read and write. It provides the
    // `.modify` method used by start_tick.
    use modify::Register;

    pub const fn with_registers(registers: &'a watchdog::Registers<Accessor>) -> Watchdog<'a, Accessor> {
        Watchdog {
            registers,
            resets: OptionalCell::empty(),
        }
    }

    pub fn resolve_dependencies(&self, resets: &'a resets::Resets) {
        self.resets.set(resets);
    }

    pub fn start_tick(&self, cycles_in_mhz: u32) {
        self.registers
            .tick
            .modify(TICK::CYCLES.val(cycles_in_mhz) + TICK::ENABLE::SET);
    }

    pub fn reboot(&self) {
        self.resets
            .map(|resets| resets.watchdog_reset_all_except(&[]));
        self.registers.ctrl.write(CTRL::TRIGGER::SET);
    }
}

#[cfg(/* Only in the real kernel */)]
impl<'a> Watchdog<'a, tock_registers::Real> {
    // Returns a new instance of the watchdog driver.
    pub const fn new() -> Watchdog<'a, tock_registers::Real> {
        // Safety: 0x40058000 is the correct address for the Watchdog MMIO
        // region, and the peripheral! invocation is a correct description of
        // the registers therein.
        let registers = unsafe { &*(0x40058000 as *const tock_registers::Real) };
        Self::with_registers(registers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_tick() {
        let fake = FakeWatchdog::new();
        let driver = Watchdog::with_registers(&fake);
        driver.start_tick(100_000);
        assert_eq!(fake.get_current_tick(), 100_000);
    }

    struct FakeWatchdog {
        ctrl: Cell<u32>,
        // ...
    }

    impl FakeWatchdog {
        // ...
    }

    impl read::Access<0, u32, CTRL::Register> for &FakeWatchdog {
        fn read(&self) -> u32 {
            self.ctrl.get()
        }
    }

    impl write::Access<0, u32, CTRL::Register> for &FakeWatchdog {
        // ...
    }

    // And many more impls -- one per (register, op) combination.

    // Should probably have peripheral! generate a macro_rules! macro to
    // simplify the implementation, something like:
    watchdog::fake! { FakeWatchdog;
        impl ctrl.read<CTRL::Register> {
            fn read(&self) -> u32 {
                self.ctrl.get()
            }
        }

        impl ctrl.write<CTRL::Register> {
            // ...
        }

        // ...
    }
}
```
