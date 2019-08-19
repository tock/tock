//! Hardware agnostic interfaces for counter-like resources.

use crate::ReturnCode;

pub trait Time {
    type Frequency: Frequency;

    /// Returns the current time in hardware clock units.
    fn now(&self) -> u32;

    /// Returns the wrap-around value of the clock.
    ///
    /// The maximum value of the clock, at which `now` will wrap around. I.e., this should return
    /// `core::u32::MAX` on a 32-bit-clock, or `1 << 24` for a 24-bit clock.
    fn max_tics(&self) -> u32;
}

pub trait Counter: Time {
    fn start(&self) -> ReturnCode;
    fn stop(&self) -> ReturnCode;
    fn is_running(&self) -> bool;
}

/// Trait to represent clock frequency in Hz
///
/// This trait is used as an associated type for `Alarm` so clients can portably
/// convert native cycles to real-time values.
pub trait Frequency {
    /// Returns frequency in Hz.
    fn frequency() -> u32;
}

/// 16MHz `Frequency`
#[derive(Debug)]
pub struct Freq16MHz;
impl Frequency for Freq16MHz {
    fn frequency() -> u32 {
        16000000
    }
}

/// 32KHz `Frequency`
#[derive(Debug)]
pub struct Freq32KHz;
impl Frequency for Freq32KHz {
    fn frequency() -> u32 {
        32768
    }
}

/// 16KHz `Frequency`
#[derive(Debug)]
pub struct Freq16KHz;
impl Frequency for Freq16KHz {
    fn frequency() -> u32 {
        16000
    }
}

/// 1KHz `Frequency`
#[derive(Debug)]
pub struct Freq1KHz;
impl Frequency for Freq1KHz {
    fn frequency() -> u32 {
        1000
    }
}

/// The `Alarm` trait models a wrapping counter capable of notifying when the
/// counter reaches a certain value.
///
/// Alarms represent a resource that keeps track of time in some fixed unit
/// (usually clock tics). Implementers should use the
/// [`Client`](trait.Client.html) trait to signal when the counter has
/// reached a pre-specified value set in [`set_alarm`](#tymethod.set_alarm).
pub trait Alarm<'a>: Time {
    /// Sets a one-shot alarm to fire when the clock reaches `tics`.
    ///
    /// [`Client#fired`](trait.Client.html#tymethod.fired) is signaled
    /// when `tics` is reached.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let delta = 1337;
    /// let tics = alarm.now().wrapping_add(delta);
    /// alarm.set_alarm(tics);
    /// ```
    fn set_alarm(&self, tics: u32);

    /// Returns the value set in [`set_alarm`](#tymethod.set_alarm)
    fn get_alarm(&self) -> u32;

    /// Set the client for interrupt events.
    fn set_client(&self, client: &'a AlarmClient);

    /// Returns whether this alarm is currently active (will eventually trigger
    /// a callback if there is a client).
    fn is_enabled(&self) -> bool;

    /// Enables the alarm using the previously set `tics` for the alarm.
    ///
    /// Most implementations should use the default implementation which calls `set_alarm` with the
    /// value returned by `get_alarm` unless there is a more efficient way to achieve the same
    /// semantics.
    fn enable(&self) {
        self.set_alarm(self.get_alarm())
    }

    /// Disables the alarm.
    ///
    /// The implementation will _always_ disable the alarm, however, it may be possible for an
    /// alarm to have already expired but the event not delivered to the client. In this case, the
    /// implementation must return [`FAIL`] letting the caller know that an event for the alarm
    /// will still be delivered.
    fn disable(&self) -> ReturnCode;
}

/// A client of an implementer of the [`Alarm`](trait.Alarm.html) trait.
pub trait AlarmClient {
    /// Callback signaled when the alarm's clock reaches the value set in
    /// [`Alarm#set_alarm`](trait.Alarm.html#tymethod.set_alarm).
    fn fired(&self);
}

/// The `Timer` trait models a timer that can notify when a particular interval
/// has elapsed.
pub trait Timer<'a>: Time {
    /// Set the client for interrupt events.
    fn set_client(&self, client: &'a TimerClient);

    /// Sets a one-shot timer to fire in `interval` clock-tics.
    ///
    /// Calling this method will override any existing oneshot or repeating timer.
    fn oneshot(&self, interval: u32);

    /// Sets repeating timer to fire every `interval` clock-tics.
    ///
    /// Calling this method will override any existing oneshot or repeating timer.
    fn repeat(&self, interval: u32);

    /// Returns the interval for a repeating timer.
    ///
    /// Returns `None` if the timer is disabled or in oneshot mode and `Some(interval)` if it is
    /// repeating.
    fn interval(&self) -> Option<u32>;

    /// Returns whether this is a oneshot (rather than repeating) timer.
    fn is_oneshot(&self) -> bool {
        self.interval().is_none()
    }

    /// Returns whether this is a repeating (rather than oneshot) timer.
    fn is_repeating(&self) -> bool {
        self.interval().is_some()
    }

    /// Returns the remaining time in clock tics for a oneshot or repeating timer.
    ///
    /// Returns `None` if the timer is disabled.
    fn time_remaining(&self) -> Option<u32>;

    /// Returns whether this timer is currently active (has time remaining).
    fn is_enabled(&self) -> bool {
        self.time_remaining().is_some()
    }

    /// Cancels an outstanding timer.
    ///
    /// The implementation will _always_ cancel the timer, however, it may be possible for a timer
    /// to have already expired but not delivered to the client. In this case, the implementation
    /// must return [`FAIL`] letting the caller know that an event for the timer will still be
    /// delivered.
    // Q:(alevy) would it be simpler semantics to require the implementation to handle this?
    // Specifically, if there is a pending interrupt when `cancel` is called, the implementation
    // should ensure that a callback for the timer event is _not_ delivered. On the SAM4L, at
    // least, that would be nearly as easy to implement and seems easier to use.
    fn cancel(&self) -> ReturnCode;
}

/// A client of an implementer of the [`Timer`](trait.Timer.html) trait.
pub trait TimerClient {
    /// Callback signaled when the timer's clock reaches the specified interval.
    fn fired(&self);
}
