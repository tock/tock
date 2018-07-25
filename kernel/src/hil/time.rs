//! Hardware agnostic interfaces for counter-like resources.

pub trait Time {
    type Frequency: Frequency;

    /// Disable any outstanding alarm or timer
    fn disable(&self);

    /// Returns whether a timer or alarm is currently armed
    fn is_armed(&self) -> bool;
}

/// Trait to represent clock frequency in Hz
///
/// This trait is used as an associated type for `Alarm` so clients can portably
/// convert native cycles to real-time values.
pub trait Frequency {
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

/// The `Alarm` trait models a wrapping counter capapable of notifying when the
/// counter reaches a certain value.
///
/// Alarms represent a resource that keeps track of time in some fixed unit
/// (usually clock tics). Implementors should use the
/// [`Client`](trait.Client.html) trait to signal when the counter has
/// reached a pre-specified value set in [`set_alarm`](#tymethod.set_alarm).
pub trait Alarm: Time {
    /// Returns the current time in hardware clock units.
    fn now(&self) -> u32;

    /// Sets a one-shot alarm fire when the clock reaches `tics`.
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
}

/// A client of an implementor of the [`Alarm`](trait.Alarm.html) trait.
pub trait Client {
    /// Callback signaled when the alarm's clock reaches the value set in
    /// [`Alarm#set_alarm`](trait.Alarm.html#tymethod.set_alarm).
    fn fired(&self);
}

/// The `Timer` trait models a timer that can notify when a particular interval
/// has elapsed.
pub trait Timer: Time {
    /// Sets a one-shot timer to fire in `interval` clock-tics.
    fn oneshot(&self, interval: u32);
    /// Sets repeating timer to fire every `interval` clock-tics.
    fn repeat(&self, interval: u32);
}
