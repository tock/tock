//! Traits for implementing various layers and components in Tock.
//!
//! Implementations of these traits are used by the core kernel.

pub mod chip;
pub mod mpu;
pub mod platform;
pub mod scheduler_timer;
pub mod watchdog;
