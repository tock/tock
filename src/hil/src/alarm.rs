//! Hardware agnostic interfaces for counter-like resources (e.g. an AST).

pub trait Frequency {
    fn frequency() -> u32;
}

pub struct Freq16MHz;
impl Frequency for Freq16MHz {
    fn frequency() -> u32 {
        16000000
    }
}

pub struct Freq16KHz;
impl Frequency for Freq16KHz {
    fn frequency() -> u32 {
        16000
    }
}

pub struct Freq1KHz;
impl Frequency for Freq1KHz {
    fn frequency() -> u32 {
        1000
    }
}

/// The `Alarm` trait keeps track of a counter such as a hardware AST.
///
/// Alarms represent a resource that keeps track of time in some fixed unit
/// (usually clock tics). Implementors should use the
/// [`AlarmClient`](trait.AlarmClient.html) trait to signal when the counter has
/// reached a pre-specified value set in [`set_alarm`](#tymethod.set_alarm).
pub trait Alarm {
    type Frequency: Frequency;

    /// Returns the current time in hardware clock units.
    fn now(&self) -> u32;

    /// Sets a one-shot alarm fire when the clock reaches `tics`.
    ///
    /// [`AlarmClient#fired`](trait.AlarmClient.html#tymethod.fired) is signaled
    /// when `tics` is reached.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let delta = 1337;
    /// let tics = alarm.now().wrapping_add(delta);
    /// alarm.set_alarm(tics);
    /// ```
    fn set_alarm(&self, tics: u32);

    /// Disables the alarm.
    fn disable_alarm(&self);

    /// Returns true if the alarm is armed
    fn is_armed(&self) -> bool;

    /// Returns the value set in [`set_alarm`](#tymethod.set_alarm)
    fn get_alarm(&self) -> u32;
}

/// A client of an implementor of the [`Alarm`](trait.Alarm.html) trait.
pub trait AlarmClient {
    /// Callback signaled when the alarm's clock reaches the value set in
    /// [`Alarm#set_alarm`](trait.Alarm.html#tymethod.set_alarm).
    fn fired(&self);
}
