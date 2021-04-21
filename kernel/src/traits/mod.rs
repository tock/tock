//! Traits for implementing various layers and components in Tock.
//!
//! Implementations of these traits are used by the core kernel.

pub mod mpu;
pub mod watchdog;

pub(crate) mod chip;
pub(crate) mod platform;
pub(crate) mod scheduler_timer;
