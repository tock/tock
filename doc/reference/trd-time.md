Kernel Time HIL
========================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Amit Levy and Philip Levis <br/>
**Draft-Created:** Feb 06, 2017<br/>
**Draft-Modified:** March 18, 2020<br/>
**Draft-Version:** 2<br/>
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

The time HIL is in the kernel crate, in module `hil::time`. It provides six
main traits:


  * `kernel::hil::time::Time`: provides an abstraction of a moment in time. It has two associated types. One describes the width and maximum value of a time value. The other specifies the frequency of the ticks of the time value.
  * `kernel::hil::time::Counter`: derives from `Time` and provides an abstraction of a free-running counter that can be started or stopped. A `Counter`'s moment in time is the current value of the counter.
  * `kernel::hil::time::Alarm`: derives from `Time`, and provides an abstraction of being able to receive a callback at a future moment in time. 
  * `kernel::hil::time::Timer`: derives from `Time`, and provides an abstraction of being able to receive a callback at some amount of time in the future, or a series of callbacks at a given period.
  * `kernel::hil::time::OverflowClient`: handles an overflow callback from a `Counter`.
  * `kernel::hil::time::AlarmClient`: handles the callback from an `Alarm`.
  * `kernel::hil::time::TimerClient`: handles the callback from a `Timer`.

In addition, to provide a level of minimal platform independence, a
port of Tock to a given microcontoller is expected to implement
certain instances of these traits. This allows, for example, system
call capsules for alarm callbacks to work across boards and chips.

This document describes these traits, their semantics, and the
instances that a Tock chip is expected to implement.

2 `Time`, `Frequency`, and `Ticks` traits
===============================

The `Time` trait represents a moment in time, which is obtained by
calling `now`.

The trait has two associated types. The first, `Frequency`, is an
implementation of the `Frequency` trait which describes how many ticks
there are in a second. The inverse of the frequency defines the time
interval between two ticks of time.

The second associated type, `Ticks`, defines the width of the time
value. This is an associated type because different microcontrollers
represent time with different bit widths: most Cortex-M
microcontrollers, for example, use 32 bits, while RISC-V uses 64 bits
and the Nordic nRF51822 provides only a 24-bit counter. The `Ticks`
associated type defines this, such that users of the `Time` trait can
know when wraparound will occur.


```rust
pub trait Ticks: Clone + Copy + From<u32> {
    fn into_usize(self) -> usize;
    fn into_u32(self) -> u32;

    fn wrapping_add(self, other: Self) -> Self;
    fn wrapping_sub(self, other: Self) -> Self;

    fn expired(reference: Self, now: Self, when: Self) -> bool;
    fn max_value() -> Self;
}

pub trait Frequency {
    fn frequency() -> u32; // Represented in Hz
}

pub trait Time {
    type Frequency: Frequency;
    type Ticks: Ticks;

    fn now(&self) -> Self::Ticks;

    fn ticks_from_seconds(s: u32) -> Self::Ticks;
    fn ticks_from_ms(s: u32) -> Self::Ticks;
    fn ticks_from_us(s: u32) -> Self::Ticks;
}
```

Frequency is defined with an [associated type] of the `Time` trait
(`Time::Frequencey`). It MUST implement the `Frequency` trait, which
has a single method, `frequency`. `frequency` returns the frequency in
Hz, e.g. 1MHz is 1000000. Clients can use this to write code that is
independent of the underlying frequency. However, at the same time,
`Frequency` allows a user of `Time` to know the granularity of ticks
and so avoid quantization error when two different times map to the
same time tick. For example, if a user of `Time` needs microsecond
precision, then the associated type can be used to statically check
that is is not put on top of an implementation with 32kHz precision.

The three `ticks_from` methods are helper functions to convert values
in seconds, milliseconds, or microseconds to a number of ticks.


[associated type]: https://doc.rust-lang.org/book/associated-types.html


3 `Counter` and `OverflowClient` traits
===============================

The `Counter` trait is the abstraction of a free-running counter that
can be started and stopped. This trait derives from the `Time` trait, so
it has associated `Frequency` and `Tick` types. The `Counter` trait
allows a client to register for callbacks when the counter overflows.

```rust
pub trait OverflowClient {
  fn overflow(&self);
}

pub trait Counter<'a>: Time {
  fn start(&self) -> ReturnCode;
  fn stop(&self) -> ReturnCode;
  fn is_running(&self) -> bool;
  fn set_client(&'a self, &'a dyn OverflowClient);
}
```

The `OverflowClient` trait is separated from the `AlarmClient` trait
because there are cases when software simply wants a free-running
counter to keep track of time, but does not need triggers at a
particular time. For hardware that has a limited number of
compare registers, allocating one of them when the compare itself
isn't needed would be wasteful.



4 `Alarm` and `AlarmClient` traits
===============================

Instances of the `Alarm` trait track an incrementing clock and can
trigger callbacks when the clock reaches a specific value and when it
overflows. The trait is derived from `Time` trait and, as a result,
has associated `Time::Frequency` and `Ticks` types.

The `AlarmClient` trait handles callbacks from an instance of `Alarm`.
The trait derives from `OverflowClient` and adds an additional callback
denoting that the time specified to the `Alarm` has been reached.

`Alarm` and `Timer` (presented below) differ in their level of
abstraction. An `Alarm` presents the abstraction of receiving a
callback when a point in time is reached or on an overflow. In
contrast, `Timer` allows one to set up callbacks that occur
regularly at a fixed interval.


```rust
pub trait AlarmClient: OverflowClient {
  fn alarm(&self);
}

pub trait Alarm: Time {
  fn set_alarm(&self, now: Self::Ticks, dt: Self::Ticks);
  fn get_alarm(&self) -> Self::Ticks;
  fn disable(&self) -> ReturnCode;
  fn set_client(&'a self, client: &'a dyn AlarmClient);    
}
```

`Alarm` has a `disable` in order to cancel an existing alarm. Calling
`set_alarm` enables an alarm.

The `set_alarm` method takes a `now` and a `dt` parameter to handle
edge cases in which it can be impossible distinguish between alarms
for the very near future and alarms for the very far future. The edge
case occurs when the underlying counter increments past the compare
value between when the call was made and the compare register is
actually set. Because the counter has moved past the intended compare
value, it will have to wrap around before the alarm will
fire. However, one cannot assume that it was supposed to fire because
it could have been that the software did request an alarm very far in
the future, close to the width of the counter.



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

