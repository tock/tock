//! Interface system tick timer.

/// Interface for the system tick timer.
///
/// A system tick timer provides a countdown timer to enforce process scheduling
/// quantums.  Implementations should have consistent timing while the CPU is
/// active, but need not operate during sleep.
///
/// On most chips, this will be implemented by the core (e.g. the ARM core), but
/// some chips lack this optional peripheral, in which case it might be
/// implemented by another timer or alarm controller.
pub trait SysTick {
    /// Sets the timer as close as possible to the given interval in
    /// microseconds.
    ///
    /// Callers can assume at least a 24-bit wide clock. Specific timing is
    /// dependent on the driving clock. In practice, increments of 10ms are most
    /// accurate and values up to 400ms are valid.
    fn set_timer(&self, us: u32);

    /// Returns if there is at least `us` microseconds left
    fn greater_than(&self, us: u32) -> bool;

    /// Returns true if the timer has expired
    fn overflowed(&self) -> bool;

    /// Resets the timer
    ///
    /// Resets the timer to 0 and disables it
    fn reset(&self);

    /// Enables the timer
    ///
    /// Enabling the timer will begin a count down from the value set with
    /// `set_timer`.
    ///
    ///   * `with_interrupt` - if set, an expiring timer will fire an interrupt.
    fn enable(&self, with_interrupt: bool);
}

/// A dummy `SysTick` implementation in which the timer never expires.
///
/// Using this implementation is functional, but will mean the scheduler cannot
/// interrupt non-yielding processes.
impl SysTick for () {
    fn reset(&self) {}

    fn set_timer(&self, _: u32) {}

    fn enable(&self, _: bool) {}

    fn overflowed(&self) -> bool {
        false
    }

    fn greater_than(&self, _: u32) -> bool {
        true
    }
}
