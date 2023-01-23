Design of Kernel Hardware Interface Layers (HILs)
========================================

**TRD: 2** <br/>
**Working Group:** Kernel<br/>
**Type:** Best Current Practice<br/>
**Status:** Final<br/>
**Author:** Brad Campbell, Philip Levis <br/>

Abstract
-------------------------------

This document describes design rules of hardware interface layers (HILs)
in the Tock operating system. HILs are Rust traits that provide a
standard interface to a hardware resource, such as a sensor, a flash
chip, a cryptographic accelerator, a bus, or a radio. Developers
adding new HILs to Tock should read this document and verify they have
followed these guidelines.

Introduction
===============================

In Tock, a hardware interface layer (HIL) is a collection of Rust
traits and types that provide a standardized API to a hardware
resource such as a sensor, flash chip, cryptographic accelerator, bus,
or a radio. 

Capsules use HILs to implement their functionality. For example, a
system call driver capsule that gives processes access to a
temperature sensor relies on having a reference to an implementation
of the `kernel::hil::sensors::TemperatureDriver` trait. This allows
the system call driver capsule to work on top of any implementation of
the `TemperatureDriver` trait, whether it is a local, on-chip sensor,
an analog sensor connected to an ADC, or a digital sensor over a bus.

Capsules use HILs in many different ways. They can be directly
accessed by kernel services, such as the in-kernel process console
using the UART HIL. They can be exposed to processes with system
driver capsules, such as with GPIO. They can be virtualized to allow
multiple clients to share a single resource, such as with the virtual
timer capsule.

This variety of use cases places a complex set of requirements on how
a HIL must behave. For example, Tock expects that every HIL is
virtualizable: it is possible to take one instance of the trait and
allow multiple clients to use it simultaneously, such that each one
thinks it has its own, independent instance of the trait. Because
virtualization often means requests can be queued and the Tock kernel
has a single stack, all HILs must be nonblocking and so have a
callback for completion. This has implications to buffer management
and ownership.

This document describes these requirements and describes a set of
design rules for HILs. They are:

1. Don't make synchronous callbacks.
2. Split-phase operations return a synchronous `Result` type which
   includes an error code in its `Err` value.
3. For split-phase operations, `Ok` means a callback will occur
   while `Err` with an error code besides BUSY means one won't.
4. Error results of split-phase operations with a buffer parameter
   include a reference to passed buffer. This returns the buffer
   to the caller.
5. Split-phase operations with a buffer parameter take a mutable reference
   even if their access is read-only.
6. Split-phase completion callbacks include a `Result` parameter whose
   `Err` contains  an error code; these errors are a superset of the
   synchronous errors.
7. Split-phase completion callbacks for an operation with a buffer
   parameter return the buffer.
8. Use fine-grained traits that separate out different use cases.
9. Separate control and datapath operations into separate traits.
10. Blocking APIs are not general: use them sparingly, if at all.
11. `initialize()` methods, when needed, should be in a separate
    trait and invoked in an instantiating Component.
12. Traits that can trigger callbacks should have a `set_client`
    method.
13. Use generic lifetimes where possible, except for buffers used in
    split-phase operations, which should be `'static`.

The rest of this document describes each of these rules and their
reasoning.

While these are design rules, they are not sacrosanct. There are
reasons or edge cases why a particular HIL might need to break one (or
more) of them. In such cases, be sure to understand the reasoning
behind the rule; if those considerations don't apply in your use case,
then it might be acceptable to break it. But it's important to realize
the exception is true for *all* implementations of the HIL, not just
yours; a HIL is intended to be a general, reusable API, not a specific
implementation.

The key recurring point in these guidelines is that a HIL should
encapsulate a wide range of possible implementations and use cases. It
might be that the hardware you are using or designing a HIL for has
particular properties or behavior. That does not mean all hardware
does. For example, a software pseudo-random generator can
synchronously return random numbers. However, a hardware-based one
typically cannot (without blocking). If you write a blocking random
number HIL because you are working with a software one, you are
precluding hardware implementations from using your HIL. This means
that a developer must decide to use either the blocking HIL (which in
some cases can't exist) or the non-blocking one, making software less
reusable.

Rule 1: Don't Make Synchronous Callbacks
===============================

Consider the following API for requesting 32 bits of randomness:

```rust
trait Random {
  fn random(&self) -> Result<(), ErrorCode>;
  fn set_client(&self, client: &'static Client);
}

trait Client {
  fn random_ready(&self, result: Result<u32, ErrorCode>);
}
```

If `Random` is implemented on top of a hardware random number
generator, the random bits might not be ready until an interrupt
is issued. E.g., if the implementation generates random numbers
by running AES128 in counter mode on a hidden seed[HCG], then
generating random bits may require an interrupt.

But AES128 computes *4* 32-bit values of randomness. So a smart
implementation will compute 128 bits, and call back with 32 of them.
The next 3 calls to `random` can produce data from the remaining
data. The simple implementation for this algorithm is to call `random_ready`
inside the call to `random` if cached results are ready: the values
are ready, so issue the callback immediately.

Making the `random_ready` callback from inside `random` is a bad idea for
two reasons: call loops and client code complexity.

The first issue that arises is it can create call loops. Suppose that
the client wants 1024 bits (so 32 words) or randomness. It needs to
invoke `random` 32 times. The standard call pattern is to call `random`,
then in the `random_ready` callback, store the new random bits and call
`random` again. This repeats 32 times.

If the implementation uses an interrupt every 4 calls, then this call
pattern isn't terrible: it would result in 8 stack frames. But suppose
that the implementation chooses to generate not 128 bits at a time, but
rather 1024 bits (e.g., runs counter mode on 32 words). Then one could have
up to 64 stack frames. It might be that the compiler inlines this, but it
also might not. Assuming the compiler always does a specific optimization
for you is dangerous: there all sorts of edge cases and heuristics, and
trying to adjust source code to coax it to do what you want (which can
change with each compiler release) is brittle.

The second, and more dangerously, client logic becomes much more complex.
For example, consider this client code:

```rust
  ...
  if self.state.get() == State::Idle {
    let result = random.random();
    match result {
      Ok(()) => self.state.set(State::Waiting),
      Err(e) => self.state.set(State::Error),
    }
  }
  ...

fn random_ready(&self, bits: u32, result: Result<(), ErrorCode>) {
  match result {
    Ok(()) => {
      // Use the random bits
      self.state.set(State::Idle);
    }
    Err(e) => {
      self.state.set(State::Error);
    }
  }
}
```

The result of starting a split-phase call indicates whether
there will be a callback: `Ok` means there will be a callback, while
`Err` means there will not. If the implementation of `Random` issues
a synchronous callback, then the `state` variable of the client will be
in an incorrect state. Before the call to `random` returns, the callback
executes and sets `state` to `State::Idle`. Then, the call to `random`
returns, and sets `state` to `State::Waiting`. If the callback checks
whether it's in the `Waiting` state (e.g., to guard against spurious/buggy
callbacks), this check will fail. The problem is that the callback occurs
*before* the caller even knows that it will occur.

There are ways to guard against this. The caller can optimistically assume
that `random` will succeed:

```rust
  ...
  if self.state.get() == State::Idle {
    self.state.set(State::Waiting);
    let result = random.random();
    match result {
      Err(e) => self.state.set(State::Error),
      Ok(()) => {} // Do nothing
    }
  }
  ...

fn random_ready(&self, bits: u32, result: Result<(), ErrorCode>) {
  match result {
    Ok(()) => {
      // Use the random bits
      self.state.set(State::Idle);
    }
    Err(e) => {
      self.state.set(State::Error);
    }
  }
}
```

After the first match (where `random` is called), `self.state` can be in
3 states:

  1. `State::Waiting`, if the call succeeded but the callback is asynchronous.
  2. `State::Error`, if the call or callback failed.
  3. `State::Idle`, if it received a synchronous callback.

This progresses up the call stack. The client that invoked this module might
receive a callback invoked from within the `random_ready` callback.

Expert programmers who are fully prepared for a re-entrant callback
might realize this and program accordingly, but most programmers
aren't. Some of the Tock developers who have been writing event-driven
embedded for code decades have mistakenly handled this case. Having
synchronous callbacks makes all code need to be as carefully written
as interrupt handling code, since from the caller's standpoint the
callback can preempt execution.

Issuing an asynchronous callback requires that the module be invoked
again later: it needs to return now, and then after that call stack is
popped, invoke the callback. For callbacks that will be triggered by
interrupts, this occurs naturally. However, if, such as in the random
number generation example, the callback is purely from software, the
module needs a way to make itself be invoked later, but as quickly as
possible. The standard mechanism to achieve this in Tock is through
deferred procedure calls. This mechanism allows a module to tell the
Tock scheduler to call again later, from the main scheduling loop. For
example, a caching implementation of `Random` might look like this:

```rust
impl Random for CachingRNG {
  fn random(&self) -> Result<(), ErrorCode> {
    if self.busy.get() {
      return Err(ErrorCode::BUSY);
    }

    self.busy.set(true);
    if self.cached_words.get() > 0 {
      // This tells the scheduler to issue a deferred procedure call,
      self.deferred_call.set();
    } else {
      self.request_more_randomness();
    }
  }
  ...
}

impl<'a> DeferredCallClient for CachingRNG<'a> {
  fn handle_deferred_call(&self) {
    let rbits = self.pop_cached_word();
    self.client.random_ready(rbits, Ok(()));
  }

  // This function must be called during board initialization.
  fn register(&'static self) {
      self.deferred_call.register(self);
  }
}
```

Rule 2: Return Synchronous Errors
===============================

Methods that invoke hardware can fail. It could be that the hardware is not
configured as expected, it is powered down, or it has been disabled. Generally
speaking, every HIL operation should return a Rust `Result` type, whose `Err`
variant includes an error code. The Tock kernel provides a standard set of
error codes, oriented towards system calls, in the `kernel::ErrorCode` enum.

HILs SHOULD return `ErrorCode`. Sometimes, however, these error codes don't
quite fit the use case and so in those cases a HIL may defines its own error
codes. The I2C HIL, for example, defines an `i2c::Error` enumeration for cases
such as address and data negative acknowledgments, which can occur in I2C.
In cases when a HIL returns its own error code type, this error code type
should also be able to represent all of the errors returned in a callback
(see Rule 6 below).

If a method doesn't return a synchronous error, there is no way for a caller
to know if the operation succeeded. This is especially problematic for
split-phase calls: whether the operation succeeds indicates whether
there will be a callback.

Rule 3: Split-phase `Result` Values Indicate Whether a Callback Will Occur
===============================

Suppose you have a split-phase call, such as for a SPI read/write operation:

```rust
pub trait SpiMasterClient {
  /// Called when a read/write operation finishes
  fn read_write_done(
    &self,
    write_buffer: &'static mut [u8],
    read_buffer: Option<&'static mut [u8]>,
    len: usize,
  );
}
pub trait SpiMaster {
  fn read_write_bytes(
    &self,
    write_buffer: &'static mut [u8],
    read_buffer: Option<&'static mut [u8]>,
    len: usize,
  ) -> Result<(), ErrorCode>;
}
```

One issue that arises is whether a client calling
`SpiMaster::read_write_bytes` should expect a callback invocation of
`SpiMasterClient::read_write_done`.  Often, when writing event-driven
code, modules are state machines. If the client is waiting for an
operation to complete, then it shouldn't call `read_write_bytes`
again.  Similarly, if it calls `read_write_bytes` and the operation
doesn't start (so there won't be a callback), then it can try to call
`read_write_bytes` again later.

It's very important to a caller to know whether a callback will be issued.
If there will be a callback, then it knows that it will be invoked again:
it can use this invocation to dequeue a request, issue its own callbacks,
or perform other operations. If there won't be a callback, then it might
never be invoked again, and can be in a stuck state.

For this reason, the standard calling convention in Tock is that an
`Ok` result means there will be a callback in response to this call,
and an `Err` result means there will not be a callback in response to
*this* call. Note that it is possible for an `Err` result to be
returned yet there will be a callback in the future.  This depends on
which `ErrorCode` is passed.  A common calling pattern is for a trait
to return `ErrorCode::BUSY` if there is already an operation pending
and a callback will be issued. This error code is unique in this way:
the general rule is that `Ok` means there will be a callback in
response to this call, `Err` with `ErrorCode::BUSY` means this call
did not start a new operation but there will be a callback in response
to a prior call, and any other `Err` means there will not be a
callback.

Cancellation calls are a slight variation on this approach. A call to
a `cancel` method (such as `uart::Transmit::transmit_abort`) also
returns a `Result` type, for which an `Ok` value means there will be a
callback in the future while an `Err` value means there will not be a
callback. In this way the return type reflects the original call.
The `Ok` value of a cancel method, however, needs to distinguish between
two cases:
  1. There was an outstanding operation, so there will be a callback, but
  it was not cancelled.
  2. There was an outstanding operation, so there will be a callback, and
  it was cancelled.
  
  
The `Result::Ok` type for cancel calls therefore often contains
information that signals whether the operation was successfully
cancelled.

Rule 4: Return Passed Buffers in Error Results
===============================

Consider this method:

```rust
// Anti-pattern: caller cannot regain buf on an error
fn send(&self, buf: &'static mut [u8]) -> Result<(), ErrorCode>;
```

This method is for a split-phase call: there is a corresponding
completion callback that passes the buffer back:

```rust
fn send_done(&self, buf: &'static mut[u8]);
```


The `send` method follows Rule 2: it returns a synchronous error. But
suppose that calling it returns an `Err(ErrorCode)`: what happens to
the buffer?

Rust's ownership rules mean that the caller can't still hold the
reference: it passed the reference to the implementer of `send`. But
since the operation did not succeed, the caller does not expect a
callback. Forcing the callee to issue a callback on a failed operation
typically forces it to include an alarm or other timer. Following Rule
1 means it can't do so synchronously, so it needs an asynchronous
event to invoke the callback from. This leads to every implementer of
the HIL requiring an alarm or timer, which use RAM, has more complex
logic, and makes initialization more complex.

As a result, in the above interface, if there is an error on `send`,
the buffer is lost. It's passed into the callee, but the callee has no
way to pass it back.

If a split-phase operation takes a reference to a buffer as a
parameter, it should return a reference to a buffer in the `Err` case:

```rust
fn send(&self, buf: &'static mut [u8]) -> Result<(), (ErrorCode, &'static mut [u8])>;
```

Before Tock transitioned to using `Result`, this calling pattern was
typically implemented with an `Option`:


```rust
fn send(&self, buf: &'static mut [u8]) -> (ReturnCode, Option<&'static mut [u8]>);
```

In this approach, when the `ReturnCode` is `SUCCESS`, the `Option` is
always supposed to be `None`; it the `ReturnCode` has an error value,
the `Option` contains the passed buffer. This invariant, however,
cannot be checked. Transitioning to using `Result` both makes Tock
more in line with standard Rust code and enforces the invariant.


Rule 5: Always Pass a Mutable Reference to Buffers
===============================

Suppose you are designing a trait to write some text to an LCD screen. The trait
takes a buffer of ASCII characters, which it puts on the LCD:

```rust
// Anti-pattern: caller is forced to discard mutability
trait LcdTextDisplay {
  // This is an anti-pattern: the `text` buffer should be `mut`, for reasons explained below
  fn display_text(&self, text: &'static [u8]) -> Result<(), ErrorCode>;
  fn set_client(&self, client: &'static Client);
}

trait Client {
  fn text_displayed(&self, text: &'static [u8], result: Result<(), ErrorCode>);
}
```

Because the text display only needs to read the provided text, the reference
to the buffer is not mutable.

This is a mistake.

The issue that arises is that because the caller passes the reference to the
LCD screen, it loses access to it. Suppose that the caller has a mutable
reference to a buffer, which it uses to read in data typed from a user before
displaying it on the screen. Or, more generally, it has a mutable reference
so it can create new text to display to the screen.

```rust
enum State {
  Idle,
  Reading,
  Writing,
}

struct TypeToText {
  buffer: Option<&'static mut [u8]>,
  uart: &'static dyn uart::Receive<'static>,
  lcd: &'static dyn LcdTextDisplay,
  state: Cell<State>,
}

impl TypeToText {
  fn display_more(&self) -> Result<(), ErrorCode> {
    if self.state.get() != State::Idle || self.buffer.is_none() {
      return Err(ErrorCode::BUSY);
    }
    let buffer = self.buffer.take();
    let uresult = uart.receive_buffer(buffer, buffer.len());
    match uresult {
      Ok(()) => {
        self.state.set(State::Reading);
        return Ok(());
      }
      Err(e) => return Err(e),
    }
  }
}

impl uart::ReceiveClient<'static> for TypeToText {
  fn received_buffer(&self, buf: &'static mut [u8]) {
    self.lcd.display_text(buf); // discards mut
  }
}
```

The problem is in this last line. `TypeToText` needs a mutable
reference so it can read into it. But once it passes the reference
to `LcdTextDisplay`, it discards mutability and cannot get it back:
`text_displayed` provides an immutable reference, which then cannot
be put back into the `buffer` field of `TypeToText`.

For this reason, split phase operations that take references should
generally take mutable references, even if they only need read-only
access. Because the reference will not be returned back until the
callback, the caller cannot rely on the call stack and scoping to
retain mutability.

Rule 6: Include a `Result` in Completion Callbacks That Includes an Error Code in its `Err`
===============================

Any error that can occur synchronously can usually occur asynchronously too.
Therefore, callbacks need to indicate that an error occurred and pass that
back to the caller. Callbacks therefore should include a `Result` type,
whose `Err` variant includes an error code. This error code type SHOULD
be the same type that is returned synchronously, to simplify error
processing and returning errors to userspace when needed.

The common case for this is virtualization, where a capsule turns one
instance of a trait into a set of instances that can be used by many
clients, each with their own callback. A typical virtualizer queues
requests. When a request comes in, if the underlying resource is idle,
the virtualizer forwards the request and marks itself busy. If the
request on the underlying resource returns an error, the virtualizer
returns this error to the client immediately and marks itself idle
again.

If the underlying resource is busy, then the virtualizer returns an
`Ok` to the caller and queues the request. Later, when the request is
dequeued, the virtualizer invokes the underlying resource. If this
operation returns an error, then the virtualizer issues a callback to
the client, passing the error. Because virtualizers queue and delay
operations, they also delay errors. If a HIL does not pass a `Result`
in its callback, then there is no way for the virtualizer inform the
client that the operation failed.

Note that abstractions which can be virtualized concurrently may not
need to pass a `Result` in their callback. `Alarm`, for example, can
be virtualized into many alarms. These alarms, however, are not queued
in a way that implies future failure.  A call to `Alarm::set_alarm`
cannot fail, so there is no need to return a `Result` in the callback.

Rule 7: Always Return the Passed Buffer in a Completion Callback
===============================

If a client passes a buffer to a module for an operation, it needs to
be able to reclaim it when the operation completes. Rust ownership
(and the fact that passed references must be mutable, see Rule 5
above) means that the caller must pass the reference to the HIL
implementation. The HIL needs to pass it back.

Rule 8: Use Fine-grained Traits That Separate Different Use Cases
===============================

Access to a trait gives access to functionality. If several pieces of
functionality are coupled into a single trait, then a client that
needs access to only some of them gets all of them. HILs should
therefore decompose their abstractions into fine-grained traits that
separate different use cases. For clients that need multiple pieces of
functionality, the HIL can also define composite traits, such that a
single reference can provide multiple traits.

Consider, for example, an early version of the `Alarm` trait:

```rust
pub trait Alarm: Time {
  fn now(&self) -> u32;
  fn set_alarm(&self, tics: u32);
  fn get_alarm(&self) -> u32;
}
```

This trait coupled two operations: setting an alarm for a callback and
being able to get the current time. A module that only needs to be able
to get the current time (e.g., for a timestamp) must also be able to
set an alarm, which implies RAM/state allocation somewhere.

The modern versions of the traits look like this:

```rust
pub trait Time {
  type Frequency: Frequency; // The number of ticks per second
  type Ticks: Ticks; // The width of a time value
  fn now(&self) -> Self::Ticks;
}

pub trait Alarm<'a>: Time {
  fn set_alarm_client(&'a self, client: &'a dyn AlarmClient);
  fn set_alarm(&self, reference: Self::Ticks, dt: Self::Ticks);
  fn get_alarm(&self) -> Self::Ticks;

  fn disarm(&self) -> ReturnCode;
  fn is_armed(&self) -> bool;
  fn minimum_dt(&self) -> Self::Ticks;
}
```

They decouple getting a timestamp (the `Time` trait) from an alarm
that issues callbacks at a particular timestamp (the `Alarm` trait).

Separating a HIL into fine-grained traits allows Tock to follow
the security principle of least privilege. In the case of GPIO, for
example, being able to read a pin does not mean a client should be
able to reconfigure or write it. Similarly, for a UART, being able
to transmit data does not mean that a client should always also be
able to read data, or reconfigure the UART parameters.

Rule 9: Separate Control and Datapath Operations into Separate Traits
===============================

This rule is a direct corollary for Rule 8, but has some specific
considerations that make it a rather hard and fast rule. Rule 8
(separate HILs into fine-grained traits) has a lot of flexibility
in design sensibility in terms of what operations *can* be coupled
together. This rule, however, is more precise and strict.

Many abstractions combine data operations and control operations.
For example, a SPI bus has data operations for sending and receiving
data, but it also has control operations for setting its speed,
polarity, and chip select. An ADC has data operations for sampling a
pin, but also has control operations for setting the bit width of
a sample, the reference voltage, and the sampling clock used. Finally,
a radio has data operations to send and receive packets, but also
control operations for setting transmit power, frequencies, and
local addresses.

HILs should separate these operations: control and data operations
should (almost) never be in the same trait. The major reason is
security: allowing a capsule to send packets should not also allow
it to set the local node address. The second major reason is virtualization.
For example, a UART virtualizer that allows multiple concurrent readers
cannot allow them to change the speed or UART configuration, as it is
shared among all of them. A capsule that can read a GPIO pin should
not always be able to reconfigure the pin (what if other capsules need
to be able to read it too?).

For example, returning to the UART example, this is an early version
of the UART trait (v1.3):

```rust
// Anti-pattern: combining data and control operations makes this
// trait unvirtualizable, as multiple clients cannot configure a
// shared UART. It also requires every client to handle both
// receive and transmit callbacks.
pub trait UART {
  fn set_client(&self, client: &'static Client);
  fn configure(&self, params: UARTParameters) -> ReturnCode;
  fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize);
  fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize);
  fn abort_receive(&self);
}
```

It breaks both Rule 8 and Rule 9. It couples reception and
transmission (Rule 8).  It also couples configuration with data (Rule
9). This HIL was fine when there was only a single user of the
UART. However, once the UART was virtualized, `configure` could not
work for virtualized clients. There were two options: have `configure`
always return an error for virtual clients, or write a new trait for
virtual clients that did not have `configure`. Neither is a good
solution. The first pushes failures to runtime: a capsule that needs
to adjust the configuration of the UART can be connected to a virtual
UART and compile fine, but then fails when it tries to call
`configure`. If that occurs rarely, then it might be a long time until
the problem is discovered. The second solution (a new trait) breaks
the idea of virtualization: a client has to be bound to either a
physical UART or a virtual one, and can't be swapped between them even
if it never calls `configure`.

The modern UART HIL looks like this:

```rust
pub trait Configure {
  fn configure(&self, params: Parameters) -> ReturnCode;
}
pub trait Transmit<'a> {
  fn set_transmit_client(&self, client: &'a dyn TransmitClient);
  fn transmit_buffer(
    &self,
    tx_buffer: &'static mut [u8],
    tx_len: usize,
  ) -> (ReturnCode, Option<&'static mut [u8]>);
  fn transmit_word(&self, word: u32) -> ReturnCode;
  fn transmit_abort(&self) -> ReturnCode;
}
pub trait Receive<'a> {
  fn set_receive_client(&self, client: &'a dyn ReceiveClient);
  fn receive_buffer(
    &self,
    rx_buffer: &'static mut [u8],
    rx_len: usize,
  ) -> (ReturnCode, Option<&'static mut [u8]>);
  fn receive_word(&self) -> ReturnCode;
  fn receive_abort(&self) -> ReturnCode;
}
pub trait Uart<'a>: Configure + Transmit<'a> + Receive<'a> {}
pub trait UartData<'a>: Transmit<'a> + Receive<'a> {}
```

Rule 10: Avoid Blocking APIs
===============================

The Tock kernel is non-blocking: I/O operations are split-phase and
have a completion callback. If an operation blocks, it blocks the
entire system.

There are cases when operations are synchronous *sometimes*. The
random number generator in Rule 1 is an example. If random bits are
cached, then a call to request random bits can sometimes return those
bits synchronously.  If the random number generator needs to engage
the underlying AES engine, then the random bits have to be
asynchronous. As Rule 1 goes into, even operations that *could* be
synchronous should have a callback that executes asynchronously.

Having a conditional synchronous operation and an asynchronous backup
is a poor solution. While it might seem to make the synchronous cases
simpler, a caller still needs to handle the asynchronous ones. The
code ends up being more complex and larger/longer, as it is now
conditional: a caller has to handle both cases.

The more attractive case is when a particular implementation of a HIL
seems like it can always be synchronous, therefore its HIL is
synchronous. For example, writes to flash are typically asynchronous:
the chip issues an interrupt once the bits are written. However, if
the flash chip being written is the same as the one code is fetched
from, then the chip may block reads while the write completes. From
the perspective of the caller, writing to flash is blocking, as the
core stops fetching instructions.  A synchronous flash HIL allows
implementations to be simpler, straight-line code.

Capsules implemented on a synchronous HIL only work for
implementations with synchronous behavior. Such a HIL limits
reuse. For example, a storage system built on top of this synchronous
API can only work on the same flash bank instructions are stored on:
otherwise, the operations will be split-phase.

There are use cases when splitting HILs in this way is worth it. For
example, straight-line code can often be shorter and simpler than
event-driven systems.  By providing a synchronous API for the subset of
devices that can support it, one can reduce code size and produce more
light-weight implementations.  For this reason, the rule is to *avoid*
blocking APIs, not to never implement them.  They can and should at
times exist, but their uses cases should be narrow and constrained as
they are fundamentally not as reusable.

Rule 11: `initialize()` methods, when needed, should be in a separate trait and invoked in an instantiating Component
===============================

Occasionally, HIL implementations need an `initialize` method to set up
state or configure hardware before their first use. When one-time 
initialization is needed, doing it deterministically at boot is preferable
than doing it dynamically on the first operation (e.g., by having a 
`is_initialized` field and calling `initialize` if it is false, then setting
it true). Doing at boot has two advantages. First, it is fail-fast:
if the HIL cannot initialize, this will be detected immediately at boot
instead of potentially non-deterministically on the first operation. Second,
it makes operations more deterministic in their execution time, which is
useful for precise applications.

Because one-time initializations should only be invoked at boot, they
should not be part of standard HIL traits, as those traits are used by
clients and services. Instead, they should either be in a separate
trait or part of a structure's implementation.

Because forgetting to initialize a module is a common source of errors,
modules that require one-time initialization should, if at all possible,
put this in an instantiable `Component` for that module. The `Component`
can handle all of the setup needed for the module, including invoking
the call to initialize.

Rule 12: Traits that can trigger callbacks should have a `set_client` method
===============================

If a HIL trait can trigger callbacks it should include a method for
setting the client that handles the callbacks. There are two
reasons. First, it is generally important to be able to change
callbacks at runtime, e.g., in response to requests, virtualization,
or other dynamic runtime behavior. Second, a client that can trigger a
callback should also be able to control what method the callback
invokes.  This gives the client flexibility if it needs to change
dispatch based on internal state, or perform some form of proxying. It
also allows the client to disable callbacks (by passing an
implementation of the trait that does nothing).

Rule 13: Use generic lifetimes, except for buffers in split-phase operations, which should be `'static`
===============================

HIL implementations should use generic lifetimes whenever possible. This has
two major advantages. First, it leaves open the possibilty that the kernel
might, in the future, support loadable modules which have a finite lifetime.
Second, an explicit `` `static`` lifetime brings safety and borrow-checker
limitations, because mutably accessing `` `static `` variables is generally
considered unsafe. 

If possible, use a single lifetime unless there are compelling reasons or
requirements otherwise. The standard lifetime name in Tock code is `` `a``,
although code can use other ones if they make sense.
In practice today, these `` `a`` lifetimes are all bound to `` `static``. 
However, by not using `` `static`` explicitly these HILs can be used with
arbitrary lifetimes.

Buffers used in split-phase operations are the one exception to generic
lifetimes. In most cases, these buffers will be used by hardware in DMA
or other operations. In that way, their lifetime is not bound to program
execution (i.e., lifetimes or stack frames) in a simple way. For example,
a buffer passed to a DMA engine may be held by the engine indefinitely if
it is never started. For this reason, buffers that touch hardware usually
must be `` `static``. If their lifetime is not `` `static``, then they
must be copied into a static buffer to be used (this is usually what happens
when application `AppSlice` buffers are passed to hardware). To avoid
unnecessary copies, HILs should use `` `static`` lifetimes for buffers
used in split-phase operations.

Author Addresses
=================================
```
Philip Levis
414 Gates Hall
Stanford University
Stanford, CA 94305

email: Philip Levis <pal@cs.stanford.edu>
phone: +1 650 725 9046

Brad Campbell 
Computer Science	
241 Olsson	
P.O. Box 400336
Charlottesville, Virginia 22904 

email: Brad Campbell <bradjc@virginia.edu>
```
