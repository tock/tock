//! Interface for configuring a watchdog

/// A trait for implementing a watchdog in the kernel.
/// This trait is called from the `kernel_loop()` code to setup
/// and maintain the watchdog timer.
/// It is up to the specific `Chip` how it will handle watchdog interrupts.
pub trait WatchDog {
    /// This function must enable the watchdog timer and configure it to
    /// trigger regulary. The period of the timer is left to the implementation
    /// to decide. The implementation must ensure that it doesn't trigger too
    /// early (when we haven't hung for example) or too late as to not catch
    /// faults.
    /// After calling this function the watchdog must be running.
    fn setup(&self) {}

    /// This function must tickle the watchdog to reset the timer.
    /// If the watchdog was previously suspended then this should also
    /// resume the timer.
    fn tickle(&self) {}

    /// Suspends the watchdog timer. After calling this the timer should not
    /// fire until after `tickle()` has been called. This function is called
    /// before sleeping.
    fn suspend(&self) {}

    /// Resumes the watchdog timer. After calling this the timer should be
    /// running again. This is called after returning from sleep, after
    /// `suspend()` was called.
    fn resume(&self) {
        self.tickle();
    }
}

pub trait WatchdogClient {
    fn reset_happened(&self) {}
}

/// Implement default WatchDog trait for unit.
impl WatchDog for () {}
