// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Mock implementations of hardware devices.
//!
//! The capsules in this module pretend to be real peripherals so that the
//! rest of the kernel can be exercised without the actual hardware being
//! present. They implement the same HIL traits a driver would use to talk to
//! the real chip (for example [`kernel::hil::i2c::I2CDevice`]) and use
//! [`kernel::deferred_call::DeferredCall`]s to asynchronously deliver the
//! callbacks that hardware interrupts would normally trigger.

pub mod i2c_bus;
pub mod sensors;
