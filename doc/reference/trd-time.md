Kernel Time HIL
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Amit Levy <br/>
**Draft-Created:** Feb 06, 2017<br/>
**Draft-Modified:** May 24, 2017<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes the hardware independent layer interface (HIL) for time
in the Tock operating system kernel. It describes the Rust traits and other
definitions for this service as well as the reasoning behind them. This
document is in full compliance with [TRD1].

1 Introduction
===============================

Microcontrollers provide a variety of hardware controllers that keep track of
time. The Tock kernel organizes these various types of controllers into two
broad categories: alarms and timers. Alarms continuously increment a clock and
can fire an event when the clock reaches a specific value. Timers can fire an
event after a certain number of clock tics have elapsed.

The time HIL is in the kernel crate, in module `hil::time`. It provides four
main traits:

  * `kernel::hil::time::Time`: The base trait for all time-based controllers.
  * `kernel::hil::time::Alarm`: Presents an abstract time controller that can
    fire when the underlying clock reaches a certain value.
  * `kernel::hil::time::Timer`: Presents an abstract time controller that can
    fire at given intervals.
  * `kernel::hil::gpio::Client`: handles the callback either an `Alarm` or
    `Timer` fire.

Most hardware time controllers can implement both the `Timer` and `Alarm`
traits, however some are more natural for one rather than the other. Moreover,
it is possible to implement a `Timer` in terms of an `Alarm` and vica versa,
however in general there is some loss of fidelity and memory overhead
associated with going from a `Timer` to an `Alarm`. Therefore, it is advisable
to use the trait that most honestly represents the underlying hardware as high
up the stack as possible.

The rest of this document discusses each trait in turn.

2 `Time` trait
===============================

The `Time` trait is the base-type for controllers that keep track of time. The
`Alarm` and `Timer` traits (below) are subtypes of this trait. Most
significantly, the `Time` trait defines the frequency for a particular
controller allowing clients to write code that is not reliant on a particular
clock frequency.

```rust
pub trait Frequency {
    fn frequency() -> u32;
}

pub trait Time {
    type Frequency: Frequency;

    fn disable(&self);

    fn is_armed(&self) -> bool;
}
```

Frequency is defined with an [associated type] of the `Time` trait
(`Time::Frequencey`). It MUST implement the `Frequency` trait, which has a
single method, `frequency`. `frequency` returns the frequency in Hz, e.g. 1MHz
is 1000000. Clients can use this to write clock-independent code. For example,
to convert from seconds to clock tics:

```rust
fn seconds_to_tics<T: Time>(seconds: u32) -> u32 {
    seconds * <T::Frequency>::frequency()
}
```

The `disable` method disables the time controller, stopping any pending alarms
or timers. It MAY disable the underlying clock to save power.

The `is_armed` methods indicates whether the time controller has an event
queued (i.e. an alarm or timer).

[associated type]: https://doc.rust-lang.org/book/associated-types.html

3 `Alarm` trait
===============================

Instances of the `Alarm` trait track a continuously incrementing clock and can
fire an event when the clock reaches a specific value. The trait is a subtype
of the `Time` trait and, as a result, has access to the `Time::Frequency`
associated type. Whenever the trait referes to "tics", they are interpreted in
native clock tics.

```rust
pub trait Alarm: Time {
    fn now(&self) -> u32;

    fn set_alarm(&self, tics: u32);

    fn get_alarm(&self) -> u32;
}
```

The `now` method returns the current value of the clock in tics.

The `set_alarm` method causes the alarm to call the `Client`'s `fired` method
(below) when the clock reaches the given value. Calling `set_alarm` MUST
invalidate and replace any previous calls to `set_alarm`.

The `get_alarm` method returns the value passed to the previous call to
`set_alarm`. `get_alarm` is only meaningful if `Time#is_armed` returns true.

4 `Timer` trait
===============================

Instances of the `Timer` trait counts underlying clock tics and trigger an
event when a certain number of tics has elapsed. The trait is a subtype of the
`Time` trait and, as a result, has access to the `Time::Frequency` associated
type. Whenever the trait referes to "tics", they are interpreted in native
clock tics.

```rust
pub trait Timer: Time {
    fn oneshot(&self, interval: u32);
    fn repeat(&self, interval: u32);
}
```

The `oneshot` method causes the alarm to call the `Client`'s `fired` method
(below) exactly once when the the given number of clock tics have elapsed.
Calling `oneshot` MUST invalidate and replace any previous calls to
`oneshot` or `repeat`.

The `repeat` method causes the alarm to call the `Client`'s `fired` method
(below) each time the given number of clock tics have elapsed, continuously. An
implementation MAY incur negligable delay between firing an event and reseting
the timer. Many hardware timers can be configured to automatically reset the
timer, though, and implementations SHOULD use this facility when available.
Calling `oneshot` MUST invalidate and replace any previous calls to `oneshot`
or `repeat`.

5 `Client` trait
===============================

The `Client` trait is how a caller provides a callback to a `Timer` or `Alarm`
implementation. Using a function defined outside these traits, it registers a
reference implementing the `Client` trait.

```rust
pub trait Client {
    fn fired(&self);
}
```

Whenever an alarm or timer event occures, the implementation MUST call the
`Client`'s `fired` method.

6 Example Implementation
=================================

7 Authors' Address
=================================

email - amit@amitlevy.com

