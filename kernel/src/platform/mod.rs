// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Traits for implementing various layers and components in Tock.
//!
//! Implementations of these traits are used by the core kernel.

pub mod chip;
pub mod mpu;
pub mod scheduler_timer;
pub mod watchdog;

pub(crate) mod platform;

pub use self::platform::ContextSwitchCallback;
pub use self::platform::KernelResources;
pub use self::platform::ProcessFault;
pub use self::platform::SyscallDriverLookup;
pub use self::platform::SyscallFilter;
pub use self::platform::TbfHeaderFilterDefaultAllow;
