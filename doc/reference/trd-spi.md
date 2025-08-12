Kernel Serial Peripheral Interface (SPI) HIL
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis, Alexandru Radovici<br/>
**Draft-Created:** 2021/08/13 <br/>
**Draft-Modified:** 2021/08/13 <br/>
**Draft-Version:** 2 <br/>
**Draft-Discuss:** devel@lists.tockos.org<br/>

Abstract
-------------------------------

This document proposes hardware independent layer interface (HIL) for 
a serial peripheral interface (SPI) bus in the Tock operating system 
kernel. It describes the Rust traits and other
definitions for this service as well as the reasoning behind them. This
document is in full compliance with [TRD1](./trd1-trds.md).

Note that this HIL has not been implemented yet in the master branch
of Tock -- this is a working document as the HIL is designed.

1 Introduction
===============================

The serial peripheral interface (SPI) is a standard bus design for
processors and microcontrollers to exchange data with sensors, I/O
devices, and other off-chip compoments. The bus is clocked. The device
driving the clock is called a "master" or "controller" and the device
whose clock is driven is called a "slave" or "peripheral". A SPI bus
has three data lines: the clock (CLK), data from the controller to the
peripheral (MOSI) and data from the peripheral to the controller
(MISO). A SPI bus does not have addressing. Instead, peripherals have
a chip select (CS) pin. When a peripheral's chip select line is
brought low, it receives data on MOSI and sends data on MISO. A
controller can connect to CS pins on many different devices and share
the bus between them by explicitly controlling which ones are active.

The SPI HIL is in the kernel crate, in module `hil::spi`. It provides seven main
traits:

  * `kernel::hil::spi::Configure`: provides an abstraction of
    configuring a SPI bus by setting its data rate, phase, and
    polarity.
  * `kernel::hil::spi::Controller`: allows a client for a SPI in
    controller mode to send and receive data.
  * `kernel::hil::spi::ControllerDevice`: combines `Configure` and
    `Controller` to provide an abstraction of a SPI bus in controller
    mode for a client that is bound to a specific chip select (e.g., a
    sensor driver). It allows a client to send and receive data as
    well as configure the bus for (only) its own operations.
  * `kernel::hil::spi::ChipSelect`: allows a client to change which
    chip select is active on a SPI bus incontroller mode.
  * `kernel::hil::spi::ControllerBus`: combines `ControllerDevice` and
    `ChipSelect` to allow a client to issue SPI operations on any chip
    select. It also supports initializing the bus hardware. This trait
    is intended to be implemented by a chip implementation.
  * `kernel::hil::spi::PeripheralDevice`: extends `Configure` and
    provides an abstraction of a SPI bus in peripheral mode. It allows
    a client to learn when it is selected, to send and receive data,
    and configure the bus for its own operations.
  * `kernel::hil::spi::PeripheralBus`: extends `PeripheralDevice` to
    support initializing the bus hardware. This trait is intended to
    be implemented by a chip peripheral implementation.
  * `kernel::hil::spi::Bus`: represents a SPI bus that can be
    dynamically changed between controller and peripheral modes. This
    trait is intended to be implemented by a chip implementation.

A given board MUST NOT include an implementation of more than one of
the `ControllerBus`, `PeripheralBus`, and `Bus` traits for a given SPI
bus. these traits are mutually exclusive.

This document describes these traits and their semantics.

2 `Configure` trait
===============================

The `Configure` trait allows a client to set the data rate (clock
frequency) of the SPI bus as well as its polarity and phase. Polarity
controls whether the clock line is high or low when the bus is
idle. Phase controls on which clock edges the bus clocks data in and
out. It also allows configuring whether data is sent most significant
bit first or least significant bit first.

```rust
pub enum DataOrder {
    MSBFirst,
    LSBFirst,
}

pub enum ClockPolarity {
    IdleLow,
    IdleHigh,
}

pub enum ClockPhase {
    SampleLeading,
    SampleTrailing,
}

pub trait Configure {
    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode>;
    fn get_rate(&self) -> u32;

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode>;
    fn get_polarity(&self) -> ClockPolarity;

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode>;
    fn get_phase(&self) -> ClockPhase;

    fn set_data_order(&self, order: DataOrder) -> Result<(), ErrorCode>;
    fn get_data_order(&self) -> DataOrder;
}
```

All of the set methods in `Configure` can return an error. Valid errors are:
  - INVAL (`set_rate` only): the parameter is outside the allowed range
  - NOSUPPORT (`set_polarity`, `set_phase`, `set_data_order`): the
    parameter provided cannot be supported. For example, a SPI bus
    that cannot have an `IdleHigh` polarity returns NOSUPPORT if a
    client tries to set it to have this polarity.
  - OFF (all): the bus is currently powered down in a state that does
    not allow configuring it.
  - BUSY (all): the bus is in the midst of an operation and cannot
    currently change its configuration.
  - FAIL (all): some other error occurred.

The `set_rate` method returns a `u32` in its success case. This is the
actual data rate set, which may differ from the one passed, e.g., due
to clock precision or prescalars. The actual rate rate set MUST be
less than the `rate` passed. If no rate can be set (e.g., the `rate`
is too small), `set_rate` MUST return `Err(INVAL)`.

The relationship of phase and polarity follows the standard SPI
specification[1]:

|  Polarity  |      Phase       |  Idle Level |    Data Out    |     Data In    |
|------------|------------------|-------------|----------------|----------------|
|  IdleLow   |  SampleLeading   |     Low     |  Rising Edge   |  Falling Edge  |
|  IdleLow   |  SampleTrailing  |     Low     |  Falling Edge  |  Rising Edge   |
|  IdleHigh  |  SampleLeading   |     High    |  Rising Edge   |  Rising Edge   |
|  IdleHigh  |  SampleTrailing  |     High    |  Falling Edge  |  Falling Edge  |

If the SPI bus is in the middle an outstanding operation
(`Controller::read_write_bytes` or `Peripheral::read_write_bytes`),
calls to `Configure` to set values MUST return BUSY.

3 `Controller`, `ControllerDevice`, and `ControllerClient` traits
===============================

The `Controller` trait allows a client to send and receive data on a SPI bus
in controller mode:

```rust
pub trait Controller<'a> {
    fn set_client(&self, client: &'a dyn ControllerClient);
    fn read_write_bytes(
        &self,
        write_buffer: &'a mut [u8],
        read_buffer: Option<&'a mut [u8]>,
        len: usize,
    ) -> Result<(), (ErrorCode, &'a mut [u8], Option<&'a mut [u8]>)>;
}

pub trait ControllerClient<'a> {
    fn read_write_done(
        &self,
        write_buffer: &'a mut [u8],
        read_buffer: Option<&'a mut [u8]>,
        len: usize,
        status: Result<(), ErrorCode>,
    );
}
```

The `read_write_bytes` method always takes a buffer to write and has
an optional buffer to read into. For operations that do not need to
read from the SPI peripheral, the `read_buffer` can be `None`.

If the call to `read_write_bytes` returns `Ok(())`, the implementation
MUST issue a callback to the `SpiControllerClient` when it
completes. If the call returns an `Err`, the implementation MUST NOT
issue a callback, except if the `ErrorCode` is `BUSY`. In this case,
the implementation issues a callback for the outstanding operation but
does not issue a callback for the failed one. If it returns `Err`, the
implementation MUST return the buffers passed in the call. Valid
`ErrorCode` values for an `Err` result are:
  - BUSY: the SPI is busy with another call to `read_write_bytes` and
    so cannot complete the request.
  - OFF: the SPI is off and cannot accept a request.
  - INVAL: the length value is 0, or one of the buffers passed has length 0.
  - RESERVE: there is no client for a callback.
  - `SIZE`: one of the buffers passed is smaller than `len`: `len` bytes
  cannot be transferred.
  - FAIL: some other failure condition.

The `set_client` method sets which callback to invoke when a
`read_write_bytes` call completes. The `read_write_done` callback MUST
return the buffers passed in the call to `read_write_bytes`. The `len`
argument is the number of bytes read/written. The `status` argument
indicates whether the SPI operation completed successfully. It may
return any of the `ErrorCode` values that can be returned by
`read_write_bytes`: these represent asynchronous errors (e.g., due to
queueing).

The `ControllerDevice` trait combines `Controller` and `Configure`
traits.  It provides the abstraction of being able to read/write to
the bus and adjust its configuration.

```rust
pub trait ControllerDevice<'a>: Controller<'a> + Configure<'a> {}
```

4 `ChipSelect` and `ControllerBus`
===============================

The `ChipSelect` trait allows a client to change which chip select is
active on the SPI bus. Because different SPI hardware can provide
different numbers of chip selects, the actual chip select value is an
associated type. This associated type is typically an `enum` so a chip
implementation can statically verify that clients pass only valid chip
select values.

```rust
pub trait ChipSelect {
  type Value: Copy;
  fn set_chip_select(&self, cs: Self::Value) -> Result<(), ErrorCode>;
  fn get_chip_select(&self) -> Self::Value);
}
```

The `ControllerBus` trait combines `ControllerDevice` and `ChipSelect`
to provide the full abstraction of a SPI bus. It is the trait that
chip SPI implementations provide. In addition to `ControllerDevice`
and `ChipSelect`, `ControllerBus` includes an `init` method. This
`init` method initializes the hardware to be a SPI controller and is
typically called at boot.

```rust
pub trait ControllerBus<'a>: ControllerDevice<'a> + ChipSelect {
  fn init(&self) -> Result<(), ErrorCode>;
}
```

The `Err` result of `init` can return the following `ErrorCode` values:
  - OFF: not currently powered so can't be initialized.
  - RESERVE: no clock is configured yet.
  - FAIL: other failure condition.

A client using a `ControllerBus` can exchange data with multiple SPI
peripherals, switching between them with `ChipSelect`. Calls to
`Configure` modify the configuration of the current chip select, which
are stateful.  Changing the chip select uses the last configuration
set for *that* chip select.  For example,

```rust
bus.set_chip_select(1);
bus.set_phase(SampleLeading);
bus.set_chip_select(2);
bus.set_phase(SampleTrailing);
bus.set_chip_select(1);
bus.read_write_bytes(...); // Uses SampleLeading
```

will have a SampleLeading phase in the final `write_byte_bytes` call,
because the configuration of chip select 1 is saved, and restored when
chip select is set back to 1.

5 `Peripheral` and `PeripheralClient` traits
===============================

When a chip acts as a SPI peripheral, it does not drive the
clock. Instead, it response to the clock of the controller. In some
cases, the peripheral must be able to respond with a bit of data
before it has even received one (e.g., if phase is set to
`SampleLeading`). As a result, a peripheral read/write request may
never complete if the controller never issues a request of its
own. The peripheral has to provide read and write buffers in
anticipation of a controller request. Unlike a controller, which must
always write data, a peripheral can only read, only write, or read and
write.

```rust
pub trait Peripheral {
    fn set_client(&self, client: &'static dyn PeripheralClient);

    fn read_write_bytes(&self, write_buffer: Option<&'static mut [u8]>, read_buffer: Option<&'static mut [u8]>, len: usize,) -> Result<
        (),
        (ErrorCode, Option<&'static mut [u8]>, Option<&'static mut [u8]>,
        ),
    >;
    fn set_write_byte(&self, write_byte: u8);
}

pub trait PeripheralClient {
    fn chip_selected(&self);
    fn read_write_done(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
        status: Result<(), ErrorCode>,
    );
}
```

The `Peripheral` API differs from the `Controller` in three ways:
  - `read_write_bytes` has an optional write buffer,
  - clients have a `chip_selected` callback, and
  - a peripheral can set its write as a single-byte value.

When a controller brings the chip select line low, the implementation
calls the `chip_selected` callback to inform the peripheral that an
operation is starting.  Because a controller may begin clocking data
almost immediately after the chip select is brought low (e.g., a SPI
clock tick, so in some cases a few hundred nanoseconds. Because this
is faster than the `chip_selected` callback can typically be issued,
the client SHOULD have already made a `read_write_bytes` or
`set_write_byte` call, so the SPI hardware has a byte ready to send.

The `set_write_byte` call sets the byte that the SPI peripheral should
write to the controller. The peripheral will write this byte on each
SPI byte operation until the next call to `set_write_byte` or
`read_write_bytes` with a write buffer argument.

The `read_write_bytes` method takes two `Option` types: one for the
write buffer and one for the read buffer. The SPI peripheral will read
bytes written by the controller into the read buffer, and will write
out the bytes in the write buffer to the controller. If no write
buffer is provided, the bytes the peripheral will write are
undefined. If `read_write_bytes` returns `Ok(())`, the request was
accepted and the implementation MUST issue a callback when the request
completes or has an error. The valid `ErrorCode` values for
`read_write_bytes` are:
 
  - BUSY: the SPI is busy with another call to `read_write_bytes` and
    so cannot complete the request.
  - OFF: the SPI is off and cannot accept a request.
  - INVAL: the `len` parameter was 0 or both buffers were `None`.
  - RESERVE: there is no client for a callback.
  - SIZE: one of the passed buffers is smaller than `len`.

The `read_write_done` callback is called when the outstanding
`read_write_bytes` request completes. The `len` argument is how many
bytes were read/written. It may differ from the `len` passed to
`read_write_bytes` if one of the buffers is is shorter, or if an error
occured. It may return any of the `ErrorCode` values that can be
returned by `read_write_bytes`: these represent asynchronous errors
(e.g., due to arbitration).

6 `PeripheralDevice` and `PeripheralBus` traits
===============================

The `PeripheralDevice` trait represents the standard client
abstraction of a SPI peripheral. It combines `Peripheral` and
`Configure`:

```rust
pub trait PeripheralDevice<'a>: Peripheral<'a> + Configure {}
```

PeripheralBus represents the lowest-level hardware abstraction of a
SPI peripheral.  It is the trait that chip implementations typically
implement. It is `PeripheralDevice` plus an `init()` method for
initializing hardware to be a SPI peripheral:

```rust
pub trait PeripheralBus<'a>: PeripheralDevice<'a> {
  fn init(&self) -> Result<(), ErrorCode>;
}
```
The `Err` result of `init` can return the following `ErrorCode` values:
  - OFF: not currently powered so can't be initialized.
  - FAIL: other failure condition.

7 `Bus` trait
===============================

The `ControllerBus` and `PeripheralBus` traits are intended for use
cases when a given SPI block is always used as either or a controller
or always used as a peripheral. Some systems, however, require the bus
to change between these roles.  For example, a board might export the
bus over an expansion header, and whether it behaves as a peripheral
or controller depends on what it's plugged into and which userspace
processes run.

The `Bus` trait allows software to dynamically change a SPI bus
between controller and peripheral mode.

```rust
pub trait Bus<'a>: PeripheralDevice<'a> + ControllerBus<'a> {
    fn make_controller(&self) -> Result<(), ErrorCode>;
    fn make_peripheral(&self) -> Result<(), ErrorCode>;
    fn is_controller(&self) -> bool;
    fn is_peripheral(&self) -> bool;
}
```

If software invokes a `Peripheral` operation while the bus is in
controller mode, the method MUST return OFF. If software invokes a
`Controller` operation while the bus is in peripheral mode, the method
MUST return off. Changing the controller chip select while the device
is in peripheral mode changes the chip select configuration of the
controller but MUST NOT have an effect on peripheral mode.

When a `Bus` first starts and is initialized, it MUST be in controller
mode, as the `init()` method is part of the `ControllerBus` trait.

8 Capsules
===============================

This section describes the standard Tock capsules for SPI communication.

9 Implementation Considerations
===============================

10 Authors' Address
=================================
```
Philip Levis
409 Gates Hall
Stanford University
Stanford, CA 94305
USA
pal@cs.stanford.edu

Alexandru Radovici <msg4alex@gmail.com>
```
