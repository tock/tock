//! Hardware agnostic interfaces for counter-like resources.
//!
//! This HIL includes three main interfaces for interacting with time-related
//! resources:
//!
//! - `Counter`
//! - `Alarm<'a>`
//! - `Timer<'a>`
//!
//! Each is based on the underlying time source of a counter and provides a
//! different abstraction to users:
//!
//! - Counters are a simple register that increments at a set rate and can be
//!   started and stopped. This can be used to time the duration of operations.
//! - Alarms provide an interrupt when the underlying counter reaches a certain value.
//!   This can be used to wait for a specified amount of time.
//! - Timers provide "oneshot" and "repeating" interfaces for receiving an
//!   interrupt after a certain amount of time, either once or continuously.
//!   This can be used for scheduling repeated events.
//!
//! All three interfaces also implement the `Time` trait which encapsulates
//! basic parameters of the underlying timing hardware, such as the number of
//! bits used for the hardware counter and the frequency at which the counter
//! increments.
//!
//! ```txt
//! The three interfaces:
//!
//!               +-------------+  +-------------+
//! +----------+  | AlarmClient |  | TimerClient |
//! |          |  +-------------+  +-------------+
//! | Counter  |  +-------------+  +-------------+
//! |          |  |             |  |             |
//! | [ Time ] |  |    Alarm    |  |    Timer    |
//! |          |  |             |  |             |
//! +----------+  |  [ Time ]   |  |  [ Time ]   |
//!               |             |  |             |
//!               +-------------+  +-------------+
//! ```

use crate::ReturnCode;

/// `Time` is a base trait that captures information about the underlying time
/// keeping source used for the higher-level abstractions in this HIL.
///
/// The details of the time keeping are based on two properties:
/// 1. The number of bits used by the increasing counter in hardware.
/// 2. How quickly the hardware increments the value in that counter.
///
/// The `Time` trait captures these two properties in the `Ticks` and
/// `Frequency` associated types. Each chip that supports time keeping must
/// implement the `Time` trait and provide suitable types for these types that
/// capture what the underlying hardware is doing. Typically, there are
/// pre-defined structs that implement the relevant traits (`Ticks` and
/// `Frequency`) for common values that chips tend to use. However, it is also
/// possible for chips to provide their own versions as needed.
///
/// The `Time` trait also exposes several functions that are useful for
/// interacting with the hardware counters.
pub trait Time {
    /// A type that encodes how many bits are used to keep track of the current
    /// time in the hardware counter. This is equivalent to the number of
    /// hardware ticks that can be counted until the counter overflows.
    ///
    /// Commonly, hardware platforms use 32 bit or 24 bit wide counters for
    /// their time keeping.
    type Ticks: Ticks;

    /// A type that encodes how quickly the hardware increments the underlying
    /// counter.
    ///
    /// This is related to the underlying clock used to drive the counter in
    /// hardware.
    type Frequency: Frequency;

    /// Returns the current time in hardware clock units.
    fn now(&self) -> Self::Ticks;

    /// Convert a value in seconds to the number of ticks that have to be
    /// counted in the hardware counter before `s` seconds have elapsed.
    fn ticks_from_seconds(s: u32) -> Self::Ticks {
        Self::Ticks::from(s * Self::Frequency::frequency())
    }

    /// Convert a value in milliseconds to the number of ticks that have to be
    /// counted in the hardware counter before `ms` milliseconds have elapsed.
    fn ticks_from_ms(ms: u32) -> Self::Ticks {
        Self::Ticks::from(((ms as u64 * Self::Frequency::frequency() as u64) / 1000) as u32)
    }

    /// Convert a value in microseconds to the number of ticks that have to be
    /// counted in the hardware counter before `us` microseconds have elapsed.
    fn ticks_from_us(us: u32) -> Self::Ticks {
        Self::Ticks::from(((us as u64 * Self::Frequency::frequency() as u64) / 1_000_000) as u32)
    }
}

/// `Counter` exposes a very basic time keeper: a single increasing counter that
/// can be started and stopped. This provides a thin layer on top of the
/// underlying hardware counter used for time keeping.
///
/// Because `Counter` also implements `Time`, users of `Counter` can query the
/// current value of the counter using the `now()` function.
///
/// The counter has no rollover protection. That is, the value returned by
/// `now()` can be smaller after calling `start()` if the counter overflows
/// while it is running.
pub trait Counter: Time {
    /// Start the counter.
    ///
    /// This counter will increase at the frequency specified by the `Frequency`
    /// type in the `Time` trait.
    ///
    /// This can return an error if the counter could not be started for some
    /// reason. This could happen if the underlying time source does not support
    /// stopping and starting the counter.
    fn start(&self) -> ReturnCode;

    /// Stop the counter.
    ///
    /// This will return an error if the underlying hardware counter cannot be
    /// stopped.
    fn stop(&self) -> ReturnCode;

    /// Check if the counter is currently running. Returns `true` if the
    /// underlying counter is incrementing.
    fn is_running(&self) -> bool;
}

/// Trait to represent clock frequency in Hz.
///
/// This trait is used as an associated type for `Time` so clients can portably
/// convert native cycles to real-time values.
///
/// The time HIL includes several default implementations of the `Frequency`
/// trait commonly found on hardware platforms.
pub trait Frequency {
    /// Return the frequency in Hz that a hardware counter increments its count.
    fn frequency() -> u32;
}

/// 16 MHz `Frequency` type.
#[derive(Debug)]
pub struct Freq16MHz;
impl Frequency for Freq16MHz {
    fn frequency() -> u32 {
        16000000
    }
}

/// 32 KHz `Frequency` type.
#[derive(Debug)]
pub struct Freq32KHz;
impl Frequency for Freq32KHz {
    fn frequency() -> u32 {
        32768
    }
}

/// 16 KHz `Frequency` type.
#[derive(Debug)]
pub struct Freq16KHz;
impl Frequency for Freq16KHz {
    fn frequency() -> u32 {
        16000
    }
}

/// 1 KHz `Frequency` type.
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
    fn set_alarm(&self, tics: Self::Ticks);

    /// Sets a one-shot alarm to fire when the clock reaches `duration` ticks
    /// from now.
    ///
    /// [`Client#fired`](trait.Client.html#tymethod.fired) is signaled when the
    /// alarm is elapsed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let delta = 1337;
    /// alarm.set_alarm_from_now(delta);
    /// ```
    fn set_alarm_from_now(&self, duration: Self::Ticks) {
        self.set_alarm(self.now().wrapping_add(duration));
    }

    /// Returns the value set in [`set_alarm`](#tymethod.set_alarm). This is the
    /// value (in ticks) when the alarm will fire.
    fn get_alarm(&self) -> Self::Ticks;

    /// Set the client for when the counter reaches the set tics value and the
    /// alarm fires.
    fn set_client(&'a self, client: &'a dyn AlarmClient);

    /// Returns whether this alarm is currently active (that is it will
    /// eventually trigger a callback if there is a client).
    fn is_enabled(&self) -> bool;

    /// Enables the alarm using the previously set `tics` for the alarm.
    ///
    /// Most implementations should use the default implementation which calls
    /// `set_alarm` with the value returned by `get_alarm` unless there is a
    /// more efficient way to achieve the same semantics.
    fn enable(&self) {
        self.set_alarm(self.get_alarm())
    }

    /// Disables the alarm.
    ///
    /// The implementation will _always_ disable the alarm and prevent events
    /// related to previously set alarms from being delivered to the client.
    fn disable(&self);
}

/// A client of an implementer of the [`Alarm`](trait.Alarm.html) trait.
///
/// This is for alarm users to receive a callback when the alarm fires.
pub trait AlarmClient {
    /// Callback signaled when the alarm's clock reaches the value set in
    /// [`Alarm#set_alarm`](trait.Alarm.html#tymethod.set_alarm).
    fn fired(&self);
}

/// The `Timer` trait models a timer that can notify when a particular interval
/// has elapsed. This can be a single notification or a repeating notification.
pub trait Timer<'a>: Time {
    /// Set the client for interrupt events.
    fn set_client(&'a self, client: &'a dyn TimerClient);

    /// Sets a one-shot timer to fire in `interval` clock-tics.
    ///
    /// Calling this method will override any existing oneshot or repeating timer.
    fn oneshot(&self, interval: Self::Ticks);

    /// Sets repeating timer to fire every `interval` clock-tics.
    ///
    /// Calling this method will override any existing oneshot or repeating timer.
    fn repeat(&self, interval: Self::Ticks);

    /// Returns the interval for a repeating timer.
    ///
    /// Returns `None` if the timer is disabled or in oneshot mode and
    /// `Some(interval)` if it is repeating.
    fn interval(&self) -> Option<Self::Ticks>;

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
    fn time_remaining(&self) -> Option<Self::Ticks>;

    /// Returns whether this timer is currently active (has time remaining).
    fn is_enabled(&self) -> bool {
        self.time_remaining().is_some()
    }

    /// Cancels an outstanding timer.
    ///
    /// The implementation will _always_ cancel the timer. No future callbacks
    /// will be delivered.
    fn cancel(&self);
}

/// A client of an implementer of the [`Timer`](trait.Timer.html) trait.
///
/// This is for timer users to receive callbacks when the timer fires.
pub trait TimerClient {
    /// Callback signaled when the timer's clock reaches the specified interval.
    fn fired(&self);
}

/// Trait to represent the width of the hardware counter used in the underlying
/// time source. This encodes the number of bits the counter uses, which
/// specifies how many ticks can occur before the counter wraps around.
///
/// This trait is used as an associate type for the `Time` trait so the clients
/// can portably convert real-world time to ticks and manage wrap-around of the
/// underlying counter.
pub trait Ticks: Clone + Copy + From<u32> {
    fn into_usize(self) -> usize;
    fn into_u32(self) -> u32;

    fn wrapping_add(self, other: Self) -> Self;
    fn wrapping_sub(self, other: Self) -> Self;

    /// Check whether `now` is after or equal to `when`, w.r.t. a `reference` time point.
    fn expired(reference: Self, now: Self, when: Self) -> bool;

    /// Returns the wrap-around value of the clock.
    ///
    /// The maximum value of the clock, at which `now` will wrap around. I.e., this should return
    /// `core::u32::MAX` on a 32-bit-clock, or `(1 << 24) - 1` for a 24-bit clock.
    fn max_value() -> Self;
}

#[derive(Clone, Copy)]
pub struct Ticks32Bits(u32);

impl Ticks32Bits {
    pub const MAX_VALUE: u32 = 0xFFFFFFFF;
}

impl From<u32> for Ticks32Bits {
    fn from(x: u32) -> Self {
        Ticks32Bits(x)
    }
}

impl Ticks for Ticks32Bits {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn into_u32(self) -> u32 {
        self.0
    }

    fn wrapping_add(self, other: Self) -> Self {
        Ticks32Bits(self.0.wrapping_add(other.0))
    }

    fn wrapping_sub(self, other: Self) -> Self {
        Ticks32Bits(self.0.wrapping_sub(other.0))
    }

    fn expired(reference: Self, now: Self, when: Self) -> bool {
        now.wrapping_sub(reference).0 >= when.wrapping_sub(reference).0
    }

    fn max_value() -> Self {
        Self(Self::MAX_VALUE)
    }
}

#[derive(Clone, Copy)]
pub struct Ticks24Bits(u32);

impl Ticks24Bits {
    pub const MAX_VALUE: u32 = 0x00FFFFFF;
}

impl From<u32> for Ticks24Bits {
    fn from(x: u32) -> Self {
        Ticks24Bits(x & Self::MAX_VALUE)
    }
}

impl Ticks for Ticks24Bits {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn into_u32(self) -> u32 {
        self.0
    }

    fn wrapping_add(self, other: Self) -> Self {
        Ticks24Bits((self.0.wrapping_add(other.0)) & Self::MAX_VALUE)
    }

    fn wrapping_sub(self, other: Self) -> Self {
        Ticks24Bits((self.0.wrapping_sub(other.0)) & Self::MAX_VALUE)
    }

    fn expired(reference: Self, now: Self, when: Self) -> bool {
        now.wrapping_sub(reference).0 >= when.wrapping_sub(reference).0
    }

    fn max_value() -> Self {
        Self(Self::MAX_VALUE)
    }
}

#[cfg(test)]
mod test {
    use super::{Freq16MHz, Freq32KHz, Ticks, Ticks24Bits, Ticks32Bits, Time};

    struct Time32Bits16MHz;
    impl Time for Time32Bits16MHz {
        type Ticks = Ticks32Bits;
        type Frequency = Freq16MHz;

        fn now(&self) -> Self::Ticks {
            Self::Ticks::from(0)
        }
    }

    struct Time32Bits32KHz;
    impl Time for Time32Bits32KHz {
        type Ticks = Ticks32Bits;
        type Frequency = Freq32KHz;

        fn now(&self) -> Self::Ticks {
            Self::Ticks::from(0)
        }
    }

    #[test]
    fn test_ticks_from_seconds() {
        assert_eq!(
            Time32Bits16MHz::ticks_from_seconds(1).into_u32(),
            16_000_000
        );
        assert_eq!(
            Time32Bits16MHz::ticks_from_seconds(10).into_u32(),
            160_000_000
        );
        assert_eq!(Time32Bits32KHz::ticks_from_seconds(1).into_u32(), 32_768);
        assert_eq!(Time32Bits32KHz::ticks_from_seconds(10).into_u32(), 327_680);
    }

    #[test]
    fn test_ticks_from_ms() {
        assert_eq!(Time32Bits16MHz::ticks_from_ms(1).into_u32(), 16_000);
        assert_eq!(Time32Bits16MHz::ticks_from_ms(10).into_u32(), 160_000);
        assert_eq!(Time32Bits16MHz::ticks_from_ms(100).into_u32(), 1_600_000);
        assert_eq!(Time32Bits32KHz::ticks_from_ms(1).into_u32(), 32);
        assert_eq!(Time32Bits32KHz::ticks_from_ms(10).into_u32(), 327);
        assert_eq!(Time32Bits32KHz::ticks_from_ms(100).into_u32(), 3_276);
    }

    #[test]
    fn test_ticks_from_us() {
        assert_eq!(Time32Bits16MHz::ticks_from_us(1).into_u32(), 16);
        assert_eq!(Time32Bits16MHz::ticks_from_us(10).into_u32(), 160);
        assert_eq!(Time32Bits16MHz::ticks_from_us(100).into_u32(), 1_600);
        assert_eq!(Time32Bits32KHz::ticks_from_us(1).into_u32(), 0);
        assert_eq!(Time32Bits32KHz::ticks_from_us(10).into_u32(), 0);
        assert_eq!(Time32Bits32KHz::ticks_from_us(100).into_u32(), 3);
    }

    #[test]
    fn test_expired_reference_zero() {
        assert_eq!(
            Ticks32Bits::expired(
                Ticks32Bits::from(0),
                Ticks32Bits::from(1),
                Ticks32Bits::from(0)
            ),
            true
        );
        assert_eq!(
            Ticks32Bits::expired(
                Ticks32Bits::from(0),
                Ticks32Bits::from(0xFFFFFFFF),
                Ticks32Bits::from(0)
            ),
            true
        );
        assert_eq!(
            Ticks32Bits::expired(
                Ticks32Bits::from(0),
                Ticks32Bits::from(1),
                Ticks32Bits::from(1)
            ),
            true
        );
        assert_eq!(
            Ticks32Bits::expired(
                Ticks32Bits::from(0),
                Ticks32Bits::from(0),
                Ticks32Bits::from(1)
            ),
            false
        );
        assert_eq!(
            Ticks32Bits::expired(
                Ticks32Bits::from(0),
                Ticks32Bits::from(0),
                Ticks32Bits::from(0xFFFFFFFF)
            ),
            false
        );
    }

    #[test]
    fn test_24bit_from() {
        assert_eq!(Ticks24Bits::from(1).into_u32(), 1);
        assert_eq!(Ticks24Bits::from(0x00FFFFFF).into_u32(), 0xFFFFFF);
        assert_eq!(Ticks24Bits::from(0x01000000).into_u32(), 0);
        assert_eq!(Ticks24Bits::from(0x12345678).into_u32(), 0x345678);
        assert_eq!(Ticks24Bits::from(0xFFFFFFFF).into_u32(), 0xFFFFFF);
    }

    #[test]
    fn test_24bit_add() {
        assert_eq!(
            Ticks24Bits::from(1)
                .wrapping_add(Ticks24Bits::from(2))
                .into_u32(),
            3
        );
        assert_eq!(
            Ticks24Bits::from(0xFFFFFF)
                .wrapping_add(Ticks24Bits::from(1))
                .into_u32(),
            0
        );
        assert_eq!(
            Ticks24Bits::from(0xFFFFFF)
                .wrapping_add(Ticks24Bits::from(123))
                .into_u32(),
            122
        );
    }

    #[test]
    fn test_24bit_sub() {
        assert_eq!(
            Ticks24Bits::from(1)
                .wrapping_sub(Ticks24Bits::from(2))
                .into_u32(),
            0xFFFFFF
        );
        assert_eq!(
            Ticks24Bits::from(0xFFFFFF)
                .wrapping_sub(Ticks24Bits::from(1))
                .into_u32(),
            0xFFFFFE
        );
        assert_eq!(
            Ticks24Bits::from(123)
                .wrapping_sub(Ticks24Bits::from(122))
                .into_u32(),
            1
        );
    }
}
