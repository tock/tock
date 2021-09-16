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
with [TRD1](./trd1-trds.md). The UART HIL in this document also adheres
to the rules in the [HIL Design Guide](./trd-hil-design.md), which requires
all callbacks to be asynchronous -- even if they could be synchronous.


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

  * `kernel::hil::uart::Configuration`: allows a client to query how a
    UART is configured.
  * `kernel::hil::uart::Configure`: allows a client to configure a UART,
    setting its speed, data width, parity, and stop bit configuration.
  * `kernel::hil::uart::Transmit`: is for transmitting data.
  * `kernel::hil::uart::TransmitClient`: is for handling callbacks
    when a data transmission is complete.
  * `kernel::hil::uart::Receive`: is for receiving data.
  * `kernel::hil::time::ReceiveClient`: handles callbacks when data is
    received.

There are also collections of traits that combine these into more
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
    fn set_baud_rate(&self, rate: u32) -> Result<u32, ErrorCode>;
    fn set_width(&self, width: Width) -> Result<(), ErrorCode>;
    fn set_parity(&self, parity: Parity) -> Result<(), ErrorCode>;
    fn set_stop_bits(&self, stop: StopBits) -> Result<(), ErrorCode>;
    fn set_flow_control(&self, on: bool) -> Result<(), ErrorCode>;
    fn configure(&self, params: Parameters) -> Result<(), ErrorCode>;
}
```

Methods in `Configure` can return the following error conditions:
  - `OFF`: The underlying hardware is currently not available, perhaps
    because it has not been initialized or in the case of a shared
    hardware USART controller because it is set up for SPI.
  - `INVAL`: Baud rate was set to 0.
  - `ENOSUPPORT`: The underlying UART cannot satisfy this configuration.
  - `FAIL`: Other failure condition.


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

The `Transmit` and `TransmitClient` traits allow a client to transmit
bytes over the UART.

```rust
enum AbortResult {
    Failure,
    Success,
}

pub trait Transmit<'a> {
    fn set_transmit_client(&self, client: &'a dyn TransmitClient);

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode>;
    fn transmit_abort(&self) -> Result<AbortResult, ()>;
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

The `Transmit` trait has two data paths: `transmit_word` and
`transmit_buffer`.  The `transmit_word` method is used in narrow use
cases where the cost and complexity of buffer management is not
needed. Generally, software should use the `transmit_buffer`
method. Most software implementations use DMA, such that a call to
`transmit_buffer` triggers a single interrupt when the transfer
completes; this saves energy and CPU cycles over per-byte transfers
and also improves transfer speeds because hardware can keep the UART
busy.

Each byte transmitted is a data word for the UART. If the UART is
using 8-bit data words, each data word is a byte. If the UART is using
smaller data words, it MUST ignore the high order bits of the data
values. For example, if the UART is using 6-bit data words and is told
to transmit `0xff`, it will transmit `0x3f`, ignoring the first two
bits. If a client needs to transmit data words larger than 8 bits, it
should use `transmit_word`, as `transmit_buffer` is a buffer of 8-bit
bytes and cannot store 9-bit values.

There can be a single transmit operation ongoing at any
time. Successfully calling either `transmit_buffer` or
`transmit_word` causes the UART to become busy until it issues the
callback corresponding to the outstanding operation.

3.1 `transmit_buffer` and `transmitted_buffer`
===============================

`Transmit::transmit_buffer` sends a buffer of data. The result
returned by `transmit_buffer` indicates whether there will be a
callback in the future. If `transmit_buffer` returns `Ok(())`,
implementation MUST call the `TransmitClient::transmitted_buffer`
callback in the future when the transmission completes or fails. If
`transmit_buffer` returns `Err` it MUST NOT issue a callback in the
future in response to this call. If the error is `BUSY`, this is
because there is an outstanding call to `transmit_buffer` or
`transmit_word`: the implementation handles these calls normally and
issues a callback for them. However, it does not issue a callback for
the call to `transmit_buffer` that returned `Err`.

The valid error codes for `transmit_buffer` are:
  - `OFF`: the underlying hardware is not available, perhaps because it has
    not been initialized or has been initialized into a different mode
    (e.g., a USART has been configured to be a SPI).
  - `BUSY`: the UART is already transmitting and has not made a transmission
    callback yet.
  - `SIZE`: `tx_len` is larger than the passed slice.
  - `FAIL`: some other failure.

Calling `transmit_buffer` while there is an outstanding
`transmit_buffer` or `transmit_word` operation MUST return `Err(BUSY)`.

The `TransmitClient::transmitted_buffer` callback indicates completion
of a buffer transmission.  The `Result` indicates whether the buffer
was successfully transmitted.  The `tx_len` argument specifies how
many data words (defined by `Configure`) were transmitted. If the
`rval` of `transmitted_buffer` is `Ok(())`, `tx_len` MUST be equal to
the size of the transmission started by `transmit_buffer`, defined
above.  A call to `transmit_word` or `transmit_buffer` made within
this callback MUST NOT return `Err(BUSY)` unless it is because this is
not the first call to one of these methods in the callback.  When this
callback is made, the UART MUST be ready to receive another call. The
valid `ErrorCode` values for `transmitted_buffer` are all of those
returned by `transmit_buffer` plus:
  - `CANCEL` if the call to `transmit_buffer` was cancelled by a call
    to `abort` and the entire buffer was not transmitted.
  - `SIZE` if the buffer could only be partially transmitted.

3.2 `transmit_word` and `transmitted_word`
===============================

The `transmit_word` method transmits a single data word of data
asynchronously.  The word length is determined by the UART
configuration.  A UART implementation MAY choose to not implement
`transmit_word` and `transmitted_word`.  There is a default
implementation of `transmitted_word` so clients that do not use
`receive_word` do not have to implement a callback.

If `transmit_word` returns `Ok(())`, the implementation MUST call the
`transmitted_word` callback in the future. If a call to
`transmit_word` returns `Err`, the implementation MUST NOT issue a
callback for this call, although if the it is `Err(BUSY)` is will
issue a callback for the outstanding operation.  Valid `ErrorCode`
results for `transmit_word` are:
  - `OFF`: The underlying hardware is not available, perhaps because
    it has not been initialized or in the case of a shared
    hardware USART controller because it is set up for SPI.
  - `BUSY`: the UART is already transmitting and has not made a
    transmission callback yet.
  - `NOSUPPORT`: the implementation does not support `transmit_word`
    operations.
  - `FAIL`: some other error.

The `TransmitClient::transmitted_word` method indicates that a single
word transmission completed.  The `Result` indicates whether the word
was successfully transmitted. A call to `transmit_word` or
`transmit_buffer` made within this callback MUST NOT return BUSY
unless it is because this is not the first call to one of these
methods in the callback.  When this callback is made, the UART MUST be
ready to receive another call. The valid `ErrorCode` values for
`transmitted_word` are all of those returned by `transmit_word` plus:
  - `CANCEL` if the call to `transmit_word` was cancelled by a call to
    `abort` and the word was not transmitted.

3.3 `transmit_abort`
===============================

The `transmit_abort` method allows a UART implementation to terminate
an outstanding call to `transmit_word` or `transmit_buffer` early. The
result of `transmit_abort` indicates whether the abort was
successful. Cancelled calls to `transmit_buffer` MUST always make a
callback, to return the transmit buffer to the caller. Cancelled calls
to `transmit_word` MAY issue a callback.

If `transmit_abort` returns `Ok`, there will be be a future callback
for the completion of the outstanding request. The value of `AbortResult`
denotes whether it will be cancelled:
  - `Ok(AbortResult::Success)`: there was an outstanding operation, which
    has been cancelled.  A callback will be made for that operation with an
    `ErrorCode` of `CANCEL`.
  - `Ok(AbortResult::Failure)`: there was an outstanding operation, which
    has not been cancelled.  A callback will be made for that operation with
    a result other than `Err(CANCEL)`.
  - `Err(()):` there was no outstanding request and there will be no future
    callback.

If there is no outstanding call to `transmit_word` or
`transmit_buffer`, `transmit_abort` MUST return `Err(())`.

4 `Receive` and `ReceiveClient` traits
===============================

The `Receive` and `ReceiveClient` traits are used to receive data from the
UART. They support both single-word and buffer reception. Buffer-based
reception is more efficient, as it allows an MCU to handle only one
interrupt for many data words. However, buffer-based reception only supports
data words of 6, 7, and 8 bits, so clients using 9-bit words need to use
word operations.

Each byte received is a data word for the UART. If the UART is using
8-bit data words, each data word is a byte. If the UART is using
smaller data words, it MUST zero the high order bits of the data
values. For example, if the UART is using 6-bit data words and
receives `0x1f`, it must store `0x1f` in a byte and not set high order
bits.  If the UART is using 9-bit words and receives `0x1ea`, it
stores this in a 32-bit value for `receive_word` as `0x000001ea`.

`Receive` supports a single outstanding receive request. A successful
call to `receive_buffer` or `receive_word` causes the UART to be busy
until the callback for the outstanding operation is issued.

If the UART returns `Ok` to a call to `receive_buffer` or
`receive_word`, it MUST return `Err(BUSY)` to subsequent calls to
those methods until it issues the callback corresponding to the
outstanding operation. The first call to `receive_buffer` or
`receive_word` from within a receive callback MUST NOT return
`Err(BUSY)`: when it makes a callback, a UART must be ready to handle
another reception request.


```rust
enum AbortResult {
    Failure,
    Success,
}

pub trait Receive<'a> {
    fn set_receive_client(&self, client: &'a dyn ReceiveClient);
    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
    fn receive_word(&self) -> Result<(), ErrorCode>;
    fn receive_abort(&self) -> Result<AbortResult, ()>;
}

pub trait ReceiveClient {
    fn received_word(&self, _word: u32, _rval: Result<(), ErrorCode>, _error: Error) {}

    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rval: Result<(), ErrorCode>,
        error: Error,
    );
}
```

4.1 `receive_buffer`, `received_buffer` and `receive_abort`
===============================

The `receive_buffer` method receives from the UART into the passed
buffer.  It receives up to `rx_len` bytes. When `rx_len` bytes has
been received, the implementation MUST call the `received_buffer`
callback to signal reception completion with an `rval` of
`Ok(())`. The implementation MAY call the `received_buffer` callback
before all `rx_len` bytes have been received.  If it calls the
`received_buffer` callback before all `rx_len` bytes have been
received, `rval` MUST be `Err`. Valid return values for
`receive_buffer` are:
  - `OFF`: the underlying hardware is not available, because it has not
    been initialized or is configured in a way that does not allow
    UART communication (e.g., a USART is configured to be SPI).
  - `BUSY`: the UART is already receiving (a buffer or a word)
    and has not made a reception `received` callback yet.
  - `SIZE`: `rx_len` is larger than the passed slice.


The `receive_abort` method can be used to cancel an outstanding buffer
reception call. If there is an outstanding buffer reception, calling
`receive_abort` MUST terminate the reception as early as possible,
possibly completing it before all of the requested bytes have been
read. In this case, the implementation MUST issue a `received_buffer`
callback reporting the number of bytes actually read and with an
`rval` of `Err(CANCEL)`.

Reception early termination is necessary for UART virtualization. For
example, suppose there are two UART clients. The first issues a read
of 80 bytes.  After 20 bytes have been read, the second client issues
a read of 40 bytes.  At this point, the virtualizer has to reduce the
length of its outstanding read, from 60 (80-20) to 40 bytes. It needs
to copy the 20 bytes read into the first client's buffer, the next 40
bytes into both of their buffers, and the last 20 bytes read into the
first client's buffer. It accomplishes this by calling `receive_abort`
to terminate the 100-byte read, copying the bytes read from the
resulting callback, then issuing a `receive_buffer` of 40 bytes.

The valid return values for `receive_abort` are:
  - `Ok(AbortResult::Success)`: there was a reception outstanding and
     it has been cancelled.  A callback with `Err(CANCEL)` will be called.
  - `Err(AbortResult::Failure)`: there was a reception outstanding but it
     was not cancelled.  A callback will be called with an `rval` other than
     `Err(CANCEL)`.
  - `Err(())`: there was no reception outstanding and the implementation will
    not issue a callback.

If there is no outstanding call to `receive_buffer` or
`receive_word`, `receive_abort` MUST return `Err(())`.

4.2 `receive_word` and `received_word`
===============================

The `receive_word` method and `received_word` callback allow a client
to perform data word operations without buffer management. They
receive a single UART data word, where the word length is defined by
the UART configuration and can be wider than 8 bits.

A UART implementation MAY choose to not implement `receive_word` and
`received_word`.  There is a default implementation of `received_word`
so clients that do not use `receive_word` do not have to implement a
callback.

If the UART returns `Ok(())` to a call to `receive_word`, it MUST make
a `received_word` callback in the future, when it receives a data word
or some error occurs. Valid `Err` values of `receive_word` are:
  - `BUSY`: the UART is busy with an outstanding call to
    `receive_buffer` or `receive_word`.
  - `OFF`: the UART is powered down or in a configuration that does
    not allow UART reception (e.g., it is a USART in SPI mode).
  - `NOSUPPORT`: `receive_word` operations are not supported.
  - `FAIL`: some other error.

5 `ReceiveXXX` trait
===============================

This is the section for what's currently called `ReceiveAdvanced`.


6 Composite Traits
===============================

In addition to the 6 basic traits, the UART HIL defines several traits
that use these basic traits as supertraits. These composite traits allow
structures to refer to multiple pieces of UART functionality with a
single reference and ensure that their implementations are coupled.

```rust
pub trait Uart<'a>: Configure + Configuration + Transmit<'a> + Receive<'a> {}
pub trait UartData<'a>: Configuration + Transmit<'a> + Receive<'a> {}
pub trait UartAdvanced<'a>: Configure + Configuration + Transmit<'a> + ReceiveAdvanced<'a> {}
pub trait Client: Configuration+ ReceiveClient + TransmitClient {}
```

The HIL provides blanket implementations of these four traits: any
structure that implements the supertraits of a composite trait will
automatically implement the composite trait.

7 Capsules
===============================

The Tock kernel provides two standard capsules for UARTs:

  * `capsules::console::Console` provides a userspace abstraction of a console.
    It allows userspace to print to and read from a serial port through a
    system call API.
  * `capsules::virtual_uart` provides a set of abstractions for virtualizing
    a single UART into many UARTs.

The structures in `capsules::virtual_uart` allow multiple clients to
read from and write to a serial port. Write operations are interleaved
at the granularity of `transmit_buffer` calls: each client's
`transmit_buffer` call is printed contiguously, but consecutive calls
to `transmit_buffer` from a single client may have other data inserted
between them. When a client calls `receive_buffer`, it starts reading
data from the serial port at that point in time, for the length of its
request. If multiple clients make `receive_buffer` calls that overlap
with one another, they each receive copies of the received data.

Suppose, for example, that there are two clients. One of them calls
`receive_buffer` for 8 bytes. A user starts typing "1234567890"
at the console. After the third byte, another client calls `receive_buffer`
for 4 bytes. After the user types "7", the second client will
receive a `received_buffer` callback with a buffer containing "4567".
After the user types "8", the first client will receive a callback
with a buffer containing "12345678".


8 Authors' Address
=================================
```
Philip Levis
409 Gates Hall
Stanford University
Stanford, CA 94305
USA
pal@cs.stanford.edu
```
