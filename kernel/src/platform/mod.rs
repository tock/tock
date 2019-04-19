//! Interface for chips and boards.

use crate::driver::Driver;
use crate::syscall;

pub mod mpu;
crate mod systick;

/// Interface for individual boards.
pub trait Platform {
    /// Platform-specific mapping of syscall numbers to objects that implement
    /// the Driver methods for that syscall
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&Driver>) -> R;

    // Default implementation is a no-op that simply calls with_driver()
    // Generally, a board can implement with_driver_permissions() by passing
    // `permissions` and a driver num to the `permissions` capsule, which will
    // verify that the process is allowed access to that specific driver
    fn with_driver_permissions<F, R>(&self, _permissions: &[u32], driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&Driver>) -> R,
    {
        self.with_driver(driver_num, f)
    }
}

/// Interface for individual MCUs.
pub trait Chip {
    type MPU: mpu::MPU;
    type UserspaceKernelBoundary: syscall::UserspaceKernelBoundary;
    type SysTick: systick::SysTick;

    fn service_pending_interrupts(&self);
    fn has_pending_interrupts(&self) -> bool;
    fn mpu(&self) -> &Self::MPU;
    fn systick(&self) -> &Self::SysTick;
    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary;
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
