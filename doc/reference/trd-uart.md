Universal Asynchronous Receiver Transmitter (UART)  HIL
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis <br/>

Abstract
-------------------------------

This document describes the hardware independent layer interface (HIL)
for UARTs (serial ports) in the Tock operating system kernel. It
describes the Rust traits and other definitions for this service as
well as the reasoning behind them. This document is in full compliance
with [TRD1](./trd1-trds.md).


1 Introduction
===============================

A serial port (UART) is a basic communication interface that Tock
relies on for debugging and interactive applications. Unlike the SPI
and I2C buses, which have a clock line, UART communication is
asynchronous. This allows it to require only one pin for each
direction of communication, but limits its speed as clock drift
between the two sides can cause bits to be read incorrectly.

The UART HIL is in the kernel crate, in module `hil::uart`. It provides five
main traits:

  * `kernel::hil::uart::Configuration`: allows a client to query how a UART is configured.
  * `kernel::hil::uart::Configure`: allows a client to configure a UART, setting its speed, data width, parity, and stop bit configuration.
  * `kernel::hil::uart::Transmit`: is for transmitting data.
  * `kernel::hil::uart::TransmitClient`: is for handling callbacks indicating a data transmission is complete.
  * `kernel::hil::uart::Receive`: is for receiving data.
  * `kernel::hil::time::ReceiveClient`: handles a callback when data is received.

There are also a collection of traits that combine these into more
complete abstractions. For example, the `Uart` trait represents a
complete UART, extending `Transmit`, `Receive`, and `Configure`.

To provide a level of minimal platform independence, a port of Tock to
a given microcontoller is expected to implement certain instances of
these traits. This allows, for example, debug output and panic dumps
to work across chips and platforms.

This document describes these traits, their semantics, and the
instances that a Tock chip is expected to implement. It also describes
how the `virtual_uart` capsule allows multiple clients to share a
UART.  This document assumes familiarity with serial ports and their
framing: [Wikipedia's article on asynchronous serial
communication](https://en.wikipedia.org/wiki/Asynchronous_serial_communication)
is a good reference.

2 `Configuration` and `Configure` 
===============================

The `Configuration` trait allows a client to query how a UART is
configured. The `Configure` trait allows a client to configure a UART,
by setting is baud date, data width, parity, stop bits, and whether
hardware flow control is enabled.

These two traits are separate because there are cases when clients
need to know the configuration but cannot set it. For example, when a UART
is virtualized across multiple clients (e.g., so multiple sources can 
write to the console), individual clients may want to check the baud rate.
However, they cannot set the baud rate, because that is fixed and shared
across all of them. Similarly, some services may need to be able to set
the UART configuration but do not need to check it.

Most devices using serial ports today use 8-bit data, but some older
devices use more or fewer bits, and hardware implementations support
this. If the data width of a UART is set to less than 8 bits, data is
still partitioned into bytes, and the UART sends the least significant
bits of each byte. Suppose a UART is configured to send 7-bit
words. If a client sends 5 bytes, the UART will send 35 bits,
transmitting the bottom 7 bits of each byte. The most significant bit
of each byte is ignored.

```rust
pub enum StopBits {
    One = 1,
    Two = 2,
}

pub enum Parity {
    None = 0,
    Odd = 1,
    Even = 2,
}

pub enum Width {
    Six = 6,
    Seven = 7,
    Eight = 8,
}

pub struct Parameters {
    pub baud_rate: u32, // baud rate in bit/s
    pub width: Width,
    pub parity: Parity,
    pub stop_bits: StopBits,
    pub hw_flow_control: bool,
}

pub trait Configuration {
	fn get_baud_rate(&self) -> u32;
    fn get_width(&self) -> Width;
	fn get_parity(&self) -> Parity;
    fn get_stop_bits(&self) -> StopBits;	
    fn get_flow_control(&self) -> bool;
	fn get_configuration(&self) -> Configuration;
}

pub trait Configure {
	fn set_baud_rate(&self, rate: u32) -> Result<u32, ErrorCode);
    fn set_width(&self, width: Width) -> Result<(), ErrorCode);
    fn set_parity(&self, parity: Parity) -> Result<(), ErrorCode);
    fn set_stop_bits(&self, stop: StopBits) -> Result<(), ErrorCode);
    fn set_flow_control(&self, on: bool) -> Result<(), ErrorCode);
	fn configure(&self, params: Parameters) -> Result<(), ErrorCode>;
}
```

Methods in `Configure` can return the following error conditions:
- OFF: The underlying hardware is currently not available, perhaps
  because it has not been initialized or in the case of a shared
  hardware USART controller because it is set up for SPI.
- INVAL: Baud rate was set to 0.
- ENOSUPPORT: The underlying UART cannot satisfy this configuration.
- FAIL: Other failure condition.


The UART may be unable to set the precise baud rate specified. For
example, the UART may be driven off a fixed clock with integer
prescalar. A call to `configure` MUST set the baud rate to the closest
possible value to the `baud_rate` field of the `params` argument and a
call to `set_baud_rate` MUST set the baud rate to the closest possible
value to the `rate` argument. The `Ok` result of `set_baud_rate`
includes the actual rate set, while an `Err(INVAL)` result means the
requested rate is well outside the operating speed of the UART (e.g., 4MHz).


3 `Transmit` and `TransmitClient`
===============================

The `Transmit` and `TransmitClient` traits 

```rust
pub trait Transmit<'a> {
    fn set_transmit_client(&self, client: &'a dyn TransmitClient);

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode>;
    fn transmit_abort(&self) -> Result<(), ErrorCode>;
}

pub trait TransmitClient {
    fn transmitted_word(&self, _rval: Result<(), ErrorCode>) {}
    fn transmitted_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        rval: Result<(), ErrorCode>,
    );
}
```

The `Transmit` trait has two data paths: `transmit_word` and `transmit_buffer`.
The `transmit_word` method is used in narrow use cases where the cost and complexity
of buffer management is not needed. Generally, software should use the `transmit_buffer`
method. Most software implementations use DMA, such that a call to `transmit_buffer` triggers
a single interrupt when the transfer completes; this saves energy and CPU cycles over per-byte
transfers and also improves transfer speeds because hardware can keep the UART busy.


    /// A call to `Transmit::transmit_word` completed. The `Result<(), ErrorCode>`
    /// indicates whether the word was successfully transmitted. A call
    /// to `transmit_word` or `transmit_buffer` made within this callback
    /// SHOULD NOT return BUSY: when this callback is made the UART should
    /// be ready to receive another call.
    ///
    /// `rval` is Ok(()) if the word was successfully transmitted, or
    ///   - CANCEL if the call to `transmit_word` was cancelled and
    ///     the word was not transmitted.
    ///   - FAIL if the transmission failed in some way.
	
    /// A call to `Transmit::transmit_buffer` completed. The `Result<(), ErrorCode>`
    /// indicates whether the buffer was successfully transmitted. A call
    /// to `transmit_word` or `transmit_buffer` made within this callback
    /// SHOULD NOT return BUSY: when this callback is made the UART should
    /// be ready to receive another call.
    ///
    /// The `tx_len` argument specifies how many words were transmitted.
    /// An `rval` of Ok(()) indicates that every requested word was
    /// transmitted: `tx_len` in the callback should be the same as
    /// `tx_len` in the initiating call.
    ///
    /// `rval` is Ok(()) if the full buffer was successfully transmitted, or
    ///   - CANCEL if the call to `transmit_buffer` was cancelled and
    ///     the buffer was not fully transmitted. `tx_len` contains
    ///     how many words were transmitted.
    ///   - SIZE if the buffer could only be partially transmitted. `tx_len`
    ///     contains how many words were transmitted.
    ///   - FAIL if the transmission failed in some way.



    /// Transmit a buffer of data. On completion, `transmitted_buffer`
    /// in the `TransmitClient` will be called.  If the `Result<(), ErrorCode>`
    /// returned by `transmit` is an `Ok(())`, the struct will issue a `transmitted_buffer`
    /// callback in the future. If the value of the `Result<(), ErrorCode>` is
    /// `Err(), then the `tx_buffer` argument is returned in the
    /// `Err()`, along with the `ErrorCode`.
    ///  Valid `ErrorCode` values are:
    ///  - OFF: The underlying hardware is not available, perhaps because
    ///          it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - BUSY: the UART is already transmitting and has not made a
    ///           transmission callback yet.
    ///  - SIZE : `tx_len` is larger than the passed slice.
    ///  - FAIL: some other error.
    ///
    /// Each byte in `tx_buffer` is a UART transfer word of 8 or fewer
    /// bits.  The word width is determined by the UART configuration,
    /// truncating any more significant bits. E.g., 0x18f transmitted in
    /// 8N1 will be sent as 0x8f and in 7N1 will be sent as 0x0f. Clients
    /// that need to transfer 9-bit words should use `transmit_word`.
    ///
    /// Calling `transmit_buffer` while there is an outstanding
    /// `transmit_buffer` or `transmit_word` operation will return BUSY.

    /// Transmit a single word of data asynchronously. The word length is
    /// determined by the UART configuration: it can be 6, 7, 8, or 9 bits long.
    /// If the `Result<(), ErrorCode>` is Ok(()), on completion,
    /// `transmitted_word` will be called on the `TransmitClient`.
    /// Other valid `Result<(), ErrorCode>` values are:
    ///  - OFF: The underlying hardware is not available, perhaps because
    ///          it has not been initialized or in the case of a shared
    ///          hardware USART controller because it is set up for SPI.
    ///  - BUSY: the UART is already transmitting and has not made a
    ///           transmission callback yet.
    ///  - FAIL: not supported, or some other error.
    /// If the `Result<(), ErrorCode>` is not Ok(()), no callback will be made.
    /// Calling `transmit_word` while there is an outstanding
    /// `transmit_buffer` or `transmit_word` operation will return
    /// BUSY.
	
	    /// Abort an outstanding call to `transmit_word` or `transmit_buffer`.
    /// The return code indicates whether the call has fully terminated or
    /// there will be a callback. Cancelled calls to `transmit_buffer` MUST
    /// always make a callback, to return the passed buffer back to the caller.
    ///
    /// If abort_transmit returns Ok(()), there will be no future
    /// callback and the client may retransmit immediately. If
    /// abort_transmit returns any other `Result<(), ErrorCode>` there will be a
    /// callback. This means that if there is no outstanding call to
    /// `transmit_word` or `transmit_buffer` then a call to
    /// `abort_transmit` returns Ok(()). If there was a `transmit`
    /// outstanding and is cancelled successfully then `BUSY` will
    /// be returned and there will be a callback with a `Result<(), ErrorCode>`
    /// of `CANCEL`. If there was a reception outstanding, which is
    /// not cancelled successfully, then `FAIL` will be returned and
    /// there will be a later callback.
    ///
    /// Returns Ok(()) or
    ///  - FAIL if the outstanding call to either transmit operation could
    ///    not be synchronously cancelled. A callback will be made on the
    ///    client indicating whether the call was successfully cancelled.




4 `Alarm` and `AlarmClient` traits
===============================

Instances of the `Alarm` trait track an incrementing clock and can
trigger callbacks when the clock reaches a specific value as well as
when it overflows. The trait is derived from `Time` trait and
therefore has associated `Time::Frequency` and `Ticks` types.

The `AlarmClient` trait handles callbacks from an instance of `Alarm`.
The trait derives from `OverflowClient` and adds an additional callback
denoting that the time specified to the `Alarm` has been reached.

`Alarm` and `Timer` (presented below) differ in their level of
abstraction. An `Alarm` presents the abstraction of receiving a
callback when a point in time is reached or on an overflow. In
contrast, `Timer` allows one to request callbacks at some interval in
the future, either once or periodically. `Alarm` requests a callback
at an absolute moment while `Timer` requests a callback at a point
relative to now.

```rust
pub trait AlarmClient {
  fn alarm(&self);
}

pub trait Alarm: Time {
  fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks);
  fn get_alarm(&self) -> Self::Ticks;
  fn disarm(&self) -> Result<(), ErrorCode>;
  fn set_alarm_client(&'a self, client: &'a dyn AlarmClient);
}
```

`Alarm` has a `disable` in order to cancel an existing alarm. Calling
`set_alarm` enables an alarm. If there is currently no alarm set, this
sets a new alarm. If there is an alarm set, calling `set_alarm` cancels
the previous alarm and replaces the it with the new one. It cancels the
previous alarm so a client does not have to disambiguate which alarm it
is handling, the previous or current one.

The `reference` parameter of `set_alarm` is typically a sample of
`Time::now` just before `set_alarm` is called, but it can also be a
stored value from a previous call. The `reference` parameter follows
the invariant that it is in the past: its value is by definition equal
to or less than a call to `Time::now`.

The `set_alarm` method takes a `reference` and a `dt` parameter to
handle edge cases in which it can be impossible distinguish between
alarms for the very near past and alarms for the very far future. The
edge case occurs when the underlying counter increments past the
compare value between when the call was made and the compare register
is actually set. Because the counter has moved past the intended
compare value, it will have to wrap around before the alarm will
fire. However, one cannot assume that the counter has moved past the
intended compare and issue a callback: the software may have requested
an alarm very far in the future, close to the width of the counter.

Having a `reference` and `dt` parameters disambiguates these two
cases. Suppose the current counter value is `current`.  If `current`
is not within the range [`reference`, `reference + dt`) (considering
unsigned wraparound), then this means the requested firing time has
passed and the callback should be issued immediately (e.g., with a
deferred procedure call, or setting the alarm very short in the
future).


5 `Timer` and `TimerClient` traits
===============================

The `Timer` trait presents the abstraction of a timer. The
timer can either be one-shot or periodic with a fixed
interval. `Timer` derives from `Time`, therefore has associated
`Time::Frequency` and `Ticks` types.

The `TimerClient` trait handles callbacks from an instance of `Timer`.
The trait has a single callback, denoting that the timer has fired.

```rust
pub trait TimerClient {
  fn timer(&self);
}

pub trait Timer<'a>: Time {
  fn set_timer_client(&'a self, &'a dyn TimerClient);
  fn oneshot(&self, interval: Self::Ticks) -> Self::Ticks;
  fn repeating(&self, interval: Self::Ticks) -> Self::Ticks;

  fn interval(&self) -> Option<Self::Ticks>;
  fn is_oneshot(&self) -> bool;
  fn is_repeating(&self) -> bool;

  fn time_remaining(&self) -> Option<Self::Ticks>;
  fn is_enabled(&self) -> bool;

  fn cancel(&self) -> Result<(), ErrorCode>;
}
```

The `oneshot` method causes the timer to issue the `TimerClient`'s
`fired` method exactly once when `interval` clock ticks have elapsed.
Calling `oneshot` MUST invalidate and replace any previous calls to
`oneshot` or `repeating`. The method returns the actual number of
ticks in the future that the callback will execute. This value MAY be
greater than `interval` to prevent certain timer race conditions
(e.g., that require a compare be set at least N ticks in the future)
but MUST NOT be less than `interval`.

The `repeating` method causes the timer to call the `Client`'s `fired`
method periodically, every `interval` clock ticks. Calling `oneshot`
MUST invalidate and replace any previous calls to `oneshot` or
`repeat`. The method returns the actual number of ticks in the future
that the first callback will execute. This value MAY be greater than
`interval` to prevent certain timer race conditions (e.g., that
require a compare be set at least N ticks in the future) but MUST NOT
be less than `interval`.


6 `Frequency` and `Ticks` Implementations
=================================

The time HIL provides four standard implementations of `Frequency`:

```rust
pub struct Freq16MHz;
pub struct Freq1MHz;
pub struct Freq32KHz;
pub struct Freq16KHz;
pub struct Freq1KHz;
```

The time HIL provides three standard implementaitons of `Ticks`:

```rust
pub struct Ticks24Bits(u32);
pub struct Ticks32Bits(u32);
pub struct Ticks64Bits(u64);
```

The 24 bits implementation is to support some Nordic Semiconductor
nRF platforms (e.g. nRF52840) that only support a 24-bit counter.


7 Capsules
===============================

The Tock kernel provides three standard capsules:

  * `capsules::alarm::AlarmDriver` provides a system call driver for
    an `Alarm`.
  * `capsules::virtual_alarm` provides a set of
    abstractions for virtualizing a single `Alarm` into many.
  * `capsules::virtual_timer` provides a set of abstractions for
    virtualizing a single `Alarm` into many `Timer` instances.

8 Required Modules
===============================

A chip MUST provide an instance of `Alarm` with a `Frequency` of `Freq32KHz`
and a `Ticks` of `Ticks32Bits`.

A chip MUST provide an instance of `Time` with a `Frequency` of `Freq32KHz` and
a `Ticks` of `Ticks64Bits`.

A chip SHOULD provide an Alarm with a `Frequency` of `Freq1MHz` and a `Ticks`
of `Ticks32Bits`.


9 Implementation Considerations
===============================

This section describes implementation considerations for hardware
implementations.

The trickiest aspects of implementing the traits in this document relate
to the `Alarm` trait and the semantics of how and when callbacks
are triggered. In particular, if `set_alarm` indicates a time that has
already passed, then the implementation should adjust it so that it
will trigger very soon (rather than wait for a wrap-around).

This is complicated by the fact that as the code is executing, the
underlying counter continues to tick. Therefore an implementation must
also be careful that this "very soon" time does not fall into the
past. Furthermore, many instances of timer hardware requires that a
compare value be some minimum number of ticks in the future. In
practice, this means setting "very soon" to be a safe number of ticks
in the future is a better implementation approach than trying to be
extremely precise and inadvertently choosing too soon and then waiting
for a wraparound.

Pseudocode to handle these cases is as follows:

```
set_alarm(self, reference, dt):
  now = now()
  expires = reference.wrapping_add(dt)
  if !now.within_range(reference, expired):
    expires = now

  if expires.wrapping_sub(now) < MIN_DELAY:
    expires = now.wrapping_add(MIN_DELAY)

  clear_alarm()
  set_compare(expires)
  enable_alarm()
```

10 Acknowledgements
===============================

The traits and abstractions in this document draw from contributions
and ideas from Patrick Mooney and Guillaume Endignoux as well as
others.


11 Authors' Address
=================================
```
Amit Levy
amit@amitlevy.com

Philip Levis
409 Gates Hall
Stanford University
Stanford, CA 94305
USA
pal@cs.stanford.edu

Guillaume Endignoux
guillaumee@google.com
```
