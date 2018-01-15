//! Interface and default implementation for the system tick timer.

/// Interface for the system tick timer.
pub trait SysTick {
    /// Sets the timer as close as possible to the given interval in
    /// microseconds.  The clock is 24-bits wide and specific timing is
    /// dependent on the driving clock. Increments of 10ms are most accurate
    /// and, in practice 466ms is the approximate maximum.
    fn set_timer(&self, us: u32);

    /// Returns the time left in approximate microseconds
    fn value(&self) -> u32;

    fn overflowed(&self) -> bool;

    fn reset(&self);

    fn enable(&self, with_interrupt: bool);

    fn overflow_fired() -> bool;
}

impl SysTick for () {
    fn reset(&self) {}

    fn set_timer(&self, _: u32) {}

    fn enable(&self, _: bool) {}

    fn overflowed(&self) -> bool {
        false
    }

    fn value(&self) -> u32 {
        !0
    }

    fn overflow_fired() -> bool {
        false
    }
}
