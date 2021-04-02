Design of Kernel Hardware Interface Layers (HILs)
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Best Current Practice<br/>
**Status:** Draft <br/>
**Author:** Philip Levis <br/>
**Draft-Created:** April 1, 2021<br/>
**Draft-Modified:** April 2, 2021<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes design rules 
hardware interface layers (HILs) in the Tock operating system. HILs
are Rust traits that provide a standard interface to a hardware
resource, such as a sensor, a flash chip, a cryptographic accelerator,
a bus, or a radio. Developers adding new HILs to Tock should read this
document and verify they have followed these guidelines.

Introduction
===============================

In Tock, a hardware interface layer (HIL) is a collection of Rust
traits and types that provide a standardized API to a hardware
resource such as a sensor, flash chip, cryptographic accelerator, bus,
or a radio. Capsules typically use HILs to provide their
functionality. For example, a system call driver capsule that gives
processes access to a temperature sensor relies on having a reference
to an implementation of the `kernel::hil::sensors::TemperatureDriver`
trait. This allows the system call driver capsule to work on top of
any implemeentation of the `TemperatureDriver` trait, whether it is a
local, on-chip sensor, an analog sensor connected to an ADC, or a
digital sensor over a bus.

HILs are used for many purposes within the kernel. They can be directly accessed
by kernel services, such as the in-kernel process console using the UART HIL. They
can be exposed to processes with system driver capsules, such as with GPIO. They
can be virtualized to allow multiple clients to share a single resource, such as with
the virtual timer capsule.

This variety of use cases place a complex set of requirements on how a HIL must
behave. For example, Tock expects that every HIL is virtualizable: it is possible
to take one instance of the trait and allow multiple clients to use it simultaneously
through queueing, such that each one thinks it has its own, independent instance of
the trait. Because virtualization means requests can be queued, all HILs must be
nonblocking and so have a callback for completion. This has implications to buffer
management and ownership.

This document describes these requirements and provides a set of design rules
for HILs. They are:

1. Do not issue synchronous callbacks.
2. Split-phase operations return a synchronous `Result` type which includes
   an error code in its `Err` value.
3. Split-phase operations with a buffer parameter return a tuple in their error 
   result, which includes the passed buffer as an element.
4. Split-phase operrations with a buffer parameter take a mutable reference even 
   if their access is read-only.
5. Split-phase completion callbacks include an `Option<ErrorCode>` as a parameter; 
   these errors are a superset of the synchronous errors.
6. Split-phase completion callbacks for an operation with a buffer parameter return 
   the buffer.
7. Separate control and datapath operations into separate traits.
8. Use fine-grained traits that separate out different use cases.
9. Blocking APIs are not general: use them sparingly, if at all.

The rest of this document describes each of these rules and their
reasoning.

While these are design rules, they are not sarosanct. There are of
course reasons or edge cases why a particular HIL might need to break
one (or more) of them. In such cases, it's usually good to read and
understand the reasoning behind the rule; if those considerations
don't apply in your use case, then it might be OK to break the
rules. But it's important that this exception is true for *all*
implementations of the HIL, not just yours; a HIL is intended to be a
general, reusable API, not a specific implementation.

A key recurring point in these guidelines is that a HIL should
encapsulate a wide range of possible implementations and use cases. It
might be that the hardware you are using or designing a HIL for has
particular properties or behavior. That does not mean all hardware
does. For example, writing to on-chip flash often halts execution, as
the core cannot read instructions while the flash is writing. This
would suggest that the flash HIL should be blocking. But if the chip
has two flash banks, it can be possible that you write to one bank
while you execute from the other. Or, if the flash is off-chip (e.g.,
a SPI device), then operations are over a bus, which is not a blocking
interface.

Rule 2: Return Synchronous Errors
===============================

Methods that invoke hardware can fail. It could be that the hardware is not
configured as expected, it is powered down, or it has been disabled. Generally
speaking, every HIL operation should return a Rust `Result` type, whose `Err` 
variant includes an error code. The Tock kernel provides a standard set of
error codes, oriented towards system calls, in the `kernel::ErrorCode` enum.
Sometimes, however, these error codes don't quite fit the use case and so
a HIL defines its own error codes. The I2C HIL, for example, defines an 
`i2c::Error` enumeration for cases such as address and data negative
acknowledgements, which can occur in I2C.

If a method doesn't return a synchronous error, there is no way for a caller
to know if the operation succeeded. This is especially problematic for
split-phase calls: whether the operation succeeded typically indicates whether
there will be a callback.

Rule 3: Return Passed Buffers in Error Results
===============================

Consider this method:

```rust
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

Rust's ownership rules mean that the caller can't still hold the reference: 
it passed the reference to the implementer of `send`. But since the 
operation did not succeed, the caller does not expect a callback. Forcing
the callee to issue a callback on a failed operation typically forces it 
to include an alarm or other timer. Following Rule 1 means it can't do
so synchronously, so it needs an asynchronous event to invoke the callback
from. This leads to every implementer of the HIL requiring an alarm or
timer, which use RAM, has more complex logic, and makes initialization more
complex.

As a result, in the above interface, if there is an error on `send`, the buffer 
is lost. It's passed into the callee, but the callee
has no way to pass it back.

If a split-phase operation takes a reference to a buffer as a parameter, it
should return a reference to a buffer in the `Err` case:

```rust
fn send(&self, buf: &'static mut [u8]) -> Result<(), (ErrorCode, &'static mut [u8])>;
```

Before Tock transitioned to using `Result`, this calling pattern was typically
implemented with an `Option`:


```rust
fn send(&self, buf: &'static mut [u8]) -> (ReturnCode, Option<&'static mut [u8]>);
```

In this approach, when the `ReturnCode` is `SUCCESS`, the `Option` is always supposed
to be `None`; it the `ReturnCode` has an error value, the `Option` contains the passed
buffer. This invariant, however, cannot be checked. Transitioning to using `Result`
both makes Tock more in line with standard Rust code and enforces the invariant.


Rule 3: Always Pass a Mutable Reference to Buffers
===============================

Rule 4: Include an `Option<ErrorCode>` in Completion Callbacks
===============================

Rule 5: Always Return the Passed Buffer in a Completion Callback
===============================

Rule 6: Separate Control and Datapath Operations into Separate Traits
===============================

Rule 7: Use Fine-grained Traits That Separate Different Use Cases
===============================

Rule 8: Avoid Blocking APIs
===============================

Author Address
=================================
```
email - Philip Levis <pal@cs.stanford.edu>
```
