// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for a priority scheduler.
//!
//! This provides one Component, PriorityComponent.
//!
//! Usage
//! -----
//! ```rust
//! let scheduler =
//!     components::priority::PriorityComponent::new(board_kernel)
//!         .finalize(components::priority_component_static!());
//! ```

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::scheduler::priority::PrioritySched;

#[macro_export]
macro_rules! priority_component_static {
    () => {{
        kernel::static_buf!(kernel::scheduler::priority::PrioritySched)
    };};
}

pub struct PriorityComponent {
    board_kernel: &'static kernel::Kernel,
}

impl PriorityComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> PriorityComponent {
        PriorityComponent { board_kernel }
    }
}

impl Component for PriorityComponent {
    type StaticInput = &'static mut MaybeUninit<PrioritySched>;
    type Output = &'static mut PrioritySched;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        static_buffer.write(PrioritySched::new(self.board_kernel))
    }
}
