pub trait Watchdog {
    /// Enable the watchdog timer. Period is the time in milliseconds
    /// the watchdog will timeout if not serviced.
    fn start(&self, period: usize);

    /// Disable the watchdog timer.
    fn stop(&self);

    /// Service the watchdog to let the hardware know the application
    /// is still executing.
    fn tickle(&self);
}
