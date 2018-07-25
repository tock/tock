//! Interface for chips and boards.

use driver::Driver;

pub mod mpu;
crate mod systick;

/// Interface for individual boards.
pub trait Platform {
    /// Platform-specific mapping of syscall numbers to objects that implement
    /// the Driver methods for that syscall
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&Driver>) -> R;
}

/// Interface for individual MCUs.
pub trait Chip {
    type MPU: mpu::MPU;
    type SysTick: systick::SysTick;

    fn service_pending_interrupts(&mut self);
    fn has_pending_interrupts(&self) -> bool;
    fn mpu(&self) -> &Self::MPU;
    fn systick(&self) -> &Self::SysTick;
    fn sleep(&self);
    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R;
}

/// Generic operations that clock-like things are expected to support.
pub trait ClockInterface {
    fn is_enabled(&self) -> bool;
    fn enable(&self);
    fn disable(&self);
}

/// Helper struct for interfaces that expect clocks, but have no clock control
pub struct NoClockControl {}
impl ClockInterface for NoClockControl {
    fn is_enabled(&self) -> bool {
        true
    }
    fn enable(&self) {}
    fn disable(&self) {}
}

/// Instance of NoClockControl for things that need references to `ClockInterface` objects
pub static mut NO_CLOCK_CONTROL: NoClockControl = NoClockControl {};
