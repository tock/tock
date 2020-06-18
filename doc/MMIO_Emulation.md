# RFC: Tock interface for MMIO device emulation

## Motivation

As part of the "On-Host Testing Framework" requirement from the [Tock on
OpenTitan
Roadmap](https://github.com/tock/tock/blob/f4779c078f3ead673d10cf3a1fd8437a61702f96/doc/wg/opentitan/roadmap.md),
we would like to provide a way for drivers to use emulated or mock
implementations of MMIO devices for host-side testing.

Host-side testing is a broad category that could include booting TockOS in
verilator, a qemu-like emulator, or running TockOS applications on the host's
native architecture. This document focuses on changes to the MMIO register
interface that would unblock unit testing in drivers that use MMIO.

## Background

TockOS has built a [register
abstraction](https://github.com/tock/tock/tree/db843ee15bfc6beceef367c4ade3e6cd24adebc6/libraries/tock-register-interface/README.md)
that provides a level of safety on top of performing volatile reads and writes
to arbitrary memory addresses.

[`libraries/tock-register-interface/src/registers.rs`](https://github.com/tock/tock/blob/master/libraries/tock-register-interface/src/registers.rs)

``` rust
/// Read/Write registers.
// To successfully alias this structure onto hardware registers in memory, this
// struct must be exactly the size of the `T` .
#[repr(transparent)]
pub struct ReadWrite<T: IntLike, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}
...snip...

impl<T: IntLike, R: RegisterLongName> ReadWrite<T, R> {
    #[inline]
    /// Get the raw register value
    pub fn get(&self) -> T {
        unsafe { ::core::ptr::read_volatile(&self.value) }
    }

    #[inline]
    /// Set the raw register value
    pub fn set(&self, value: T) {
        unsafe { ::core::ptr::write_volatile(&self.value as *const T as *mut T, value) }
    }
    ...snip...
}
```

Here's what it looks like in practice:

[`chips/sifive/src/uart.rs`](https://github.com/tock/tock/blob/7efc61a96fe38d908d03221d5d567da32d91ada1/chips/sifive/src/uart.rs#L13)

``` rust
#[repr(C)]
pub struct UartRegisters {
    /// Transmit Data Register
    txdata: ReadWrite<u32, txdata::Register>,
    /// Receive Data Register
    rxdata: ReadWrite<u32, rxdata::Register>,
    /// Transmit Control Register
    txctrl: ReadWrite<u32, txctrl::Register>,
    /// Receive Control Register
    rxctrl: ReadWrite<u32, rxctrl::Register>,
    /// Interrupt Enable Register
    ie: ReadWrite<u32, interrupt::Register>,
    /// Interrupt Pending Register
    ip: ReadOnly<u32, interrupt::Register>,
    /// Baud Rate Divisor Register
    div: ReadWrite<u32, div::Register>,
}

register_bitfields![u32,
    txdata [
        full OFFSET(31) NUMBITS(1) [],
        data OFFSET(0) NUMBITS(8) []
    ],
    rxdata [
        empty OFFSET(31) NUMBITS(1) [],
        data OFFSET(0) NUMBITS(8) []
    ],
    txctrl [
        txcnt OFFSET(16) NUMBITS(3) [],
        nstop OFFSET(1) NUMBITS(1) [
            OneStopBit = 0,
            TwoStopBits = 1
        ],
        txen OFFSET(0) NUMBITS(1) []
    ],
    rxctrl [
        counter OFFSET(16) NUMBITS(3) [],
        enable OFFSET(0) NUMBITS(1) []
    ],
    interrupt [
        rxwm OFFSET(1) NUMBITS(1) [],
        txwm OFFSET(0) NUMBITS(1) []
    ],
    div [
        div OFFSET(0) NUMBITS(16) []
    ]
];

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
    clock_frequency: u32,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    stop_bits: Cell<hil::uart::StopBits>,
    buffer: TakeCell<'static, [u8]>,
    len: Cell<usize>,
    index: Cell<usize>,
}
```

[`chips/e310x/src/uart.rs`](https://github.com/tock/tock/blob/7efc61a96fe38d908d03221d5d567da32d91ada1/chips/e310x/src/uart.rs)

``` rust
//! UART instantiation.

use kernel::common::StaticRef;
use sifive::uart::{Uart, UartRegisters};

pub static mut UART0: Uart = Uart::new(UART0_BASE, 18_000_000);

const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x1001_3000 as *const UartRegisters) };
```

The TockOS abstraction aliases a struct on top of MMIO memory. This fits MMIO
registers into Rust's memory safety model. From the client's perspective, only
the aliasing of the struct over an existing MMIO region is considered unsafe.
The client must be certain that the struct layout matches the MMIO region
exactly.

Not every driver uses the `tock-registers` crate. Many use [`VolatileCell`](https://github.com/tock/tock/blob/27d6bd11f9a618d75bcdc0edd9993c218111932a/doc/Mutable_References.md#volatilecell)
in their own `#[repr(C)]` structs, sometimes in conjunction with the
`tock-registers` crates. Supporting drivers that use other inte

## Proposal

### Global MMIO device registration; add `mmio_emu` feature to the `tock-registers` crate.

With this approach, we keep the interface for existing capsules the same. Rather
than invoking `ptr::{read,write}_volatile` directly, register implementations
will invoke a wrapper function.

``` rust
pub fn get(&self) -> T {
    unsafe { mmio::read_volatile(&self.value) }
}

pub fn set(&self, value: T) {
    unsafe { mmio::write_volatile(&self.value as *const T as *mut T, value) }
}
```

The `mmio` module behavior can be configured via cargo features to enable or
disable MMIO emulation:

``` rust
#[cfg_attr(not(feature = "mmio_emu"), path = "mmio.rs")]
#[cfg_attr(feature = "mmio_emu", path = "mmio_emu.rs")]
pub mod mmio;
```

Boards running TockOS would have no need for the `mmio_emu` feature and would
end up invoking the same `ptr::{read,write}_volatile` functions that they do
now.

Unit tests on the host can enable `mmio_emu` to allow emulation of MMIO
registers.

Proof-of-concept implementation at
https://github.com/smibarber/tock/blob/mmio-emulation/libraries/tock-register-interface/src/mmio_emu.rs

Pros:

* No changes required to register interface, so existing drivers that use the `tock-registers` interface can work with emulation.

* Ease of implementation. No large refactoring or API migration needed, unless migrating drivers from direct `VolatileCell` usage to `tock-registers` .

Cons:

* Requires MMIO regions to be registered globally. This creates more overhead in unit testing since an MMIO access must be translated back from a global address into its device and offset.

* Awkward interface for users creating a fake MMIO device. Users must declare a `static` item for each emulated device to reserve its address range and to be able to create a `StaticRef<Registers>` for capsules.

* Selecting between passthrough/emulation with a cargo feature is a bit kludgy.

Example use:

``` rust
#[test]
fn counter_device() {
    // Declare a global instance of CounterRegisters to reserve an address
    // range for testing.
    static FAKE_REGS: MaybeUninit<CounterRegisters> = MaybeUninit::uninit();

    // Take a reference to the CounterRegisters struct.
    // Technically this is UB since it's uninitialized, but we never actually
    // access this memory.
    let regs = unsafe { &*FAKE_REGS.as_ptr() };

    let device = Arc::new(Mutex::new(CounterDevice::new()));
    register_mmio_device(device, &FAKE_REGS).unwrap();

    assert_eq!(regs.counter.get(), 0);
    regs.increment.set(5);
    assert_eq!(regs.counter.get(), 5);
    regs.increment.set(1);
    assert_eq!(regs.counter.get(), 6);
}
```

## Alternatives considered

### Traitify register interface

Existing TockOS capsules usually hold a reference directly to the MMIO region, 
such as `StaticRef<FooRegisters>` . The capsules then access struct fields
directly and invoke methods on them.

Instead, we want to hide the implementation details of accessing registers
behind traits. A driver can then be insulated from the implementation details of
accessing the registers, in particular the `StaticRef<FooRegisters>` which
requires a static MMIO region.

We'll ignore the problem of bitfields within a register for now, and think about
a simple device `Foo` with one read/write `u32` control register. A minimal
`FooRegisters` trait to start with might look like this.

``` rust
pub trait FooRegisters {
    fn set_control(&self, val: u32);
    fn control(&self) -> u32;
}
```

This works at the most basic level, but has lost the additional helper methods
that the `ReadWrite` struct has, in particular being able to read or write an
enum. We could return a proxy object instead that implements a `ReadWrite` 
trait.

``` rust
pub trait ReadWrite<T: IntLike> {...}
pub trait FooRegisters {
    fn control(&self) -> &ReadWrite<u32>;
}
```

However, this returns a trait object. We want to avoid dynamic dispatch, which
would add unnecessary code bloat and overhead to each MMIO register access.
Static dispatch is preferable, since it gives the compiler and LTO maximum
flexibility to optimize.

An associated type would allow impls of `FooRegisters` to pick either "real" or
"fake" register implementations, and still allows for static dispatch. But since
register wrappers require a type parameter, this depends on [generic associated types (GATs)](https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md#associated-type-constructors-of-type-arguments)
which have not yet been fully implemented.

``` rust
pub trait ReadWrite<T: IntLike> {...}

pub trait FooRegisters {
    type RW<T>: ReadWrite<T: Intlike>;
    fn control(&self) -> Self::RW<u32>;
}

struct FooRegistersWrapper;
impl FooRegisters for FooRegistersWrapper {
    type RW<T> = RealReadWrite<T>;
    fn control(&self) -> Self::RW<u32> {...}
}
```

Pros:

* Drivers are abstracted away from the implementation details of the `tock-registers` crate.

* Easy to create fake or emulated instances for testing. No `StaticRef` is required.

Cons:

* A clean implementation would require [generic associated types](https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md), which are not yet fully implemented or stable.

* Changes API substantially for existing capsules.

* Additional layers make it more difficult to reason that register access overhead compiles down to the same optimal loads and stores.

### Traitify register implementations

We keep the existing `#[repr(C)]` structs, but add a type parameter that allows
substituting in a fake or real register implementation. This keeps the interface
for existing drivers largely the same.

``` rust
trait RegType<T: IntLike> {
    fn set(&self, value: T);
    fn get(&self) -> T;
}

#[repr(transparent)]
pub struct RealRegister<T: IntLike> {
    value: T,
}

impl<T: IntLike> RegType for RealRegister<T> {...}

#[repr(transparent)]
pub struct ReadWrite<T: IntLike, RT: RegType<T>> {
    value: RT<T>,
    associated_register: PhantomData<R>,
}
// And the same for ReadOnly, WriteOnly, etc.

// The above works fine, but how do we pass in a RegType as a type parameter
// to a FooRegisters struct?

// Here's one option, but the limitation is that T here isn't generic within
// the struct! The caller picks a concrete T and all uses of RegType inside
// the struct must use the same concrete T.
struct FooRegisters<T: IntLike, RegType<T>> {
    // These fields won't work since u32 and u64 may not be the same type as T.
    control: ReadWrite<u32, RT>,
    status:  ReadOnly<u64, RT>,
}

// This version allows RT to be generic over any T: IntLike.
// However, this requires higher rank trait bounds to work for types, not just
// lifetimes as they do now.
struct FooRegisters<RT>
where for<T: IntLike> RT: RegType<T> {
    // These fields work, since u32 and u64 both have IntLike impls.
    control: ReadWrite<u32, RT>,
    status:  ReadOnly<u64, RT>,
}
```

We cannot pass in a generic `RegType` as a type parameter to `FooRegisters` in current Rust. This requires some form of [higher kinded
types](https://github.com/rust-lang/rfcs/issues/324). The syntax above was borrowed from [Higher-Rank Trait Bounds](https://doc.rust-lang.org/stable/nomicon/hrtb.html).

### Use conditional compilation to replace register structs

The `ReadWrite` , `ReadOnly` , and other register structs could conditionally be 
replaced with versions that call into MMIO emulation functions.

``` rust
#[cfg(feature = "mmio_emu")]
pub struct ReadWrite<T: IntLike, R: RegisterLongName = ()> {
    device: Arc<Mutex<dyn MmioDevice>>,
    reg_offset: usize,
    value: PhantomData<T>,
    associated_register: PhantomData<R>,
}
```

This avoids the need to register devices globally. It does not eliminate the
need to create a static item for each test, since drivers still use `StaticRef` .

One difficulty with writing this is that the register offsets are implicitly encoded
in the layout of the overall register block struct. Changing the size of the struct
(formerly `#[repr(transparent)]` for `T` ) means that the register offsets must
instead be initialized by some constructor.

The `register_structs!` macro does have offset information, so it could generate the
required constructor.
