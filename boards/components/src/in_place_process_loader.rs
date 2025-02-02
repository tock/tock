// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for In Place Process Loading Drivers.
//!
//! Example instantiation:
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let in_place_process_loader = components::in_place_process_loader::InPlaceProcessLoaderComponent::new(
//!     board_kernel,
//!     capsules_extra::in_place_process_loader::DRIVER_NUM,
//!     dynamic_process_loader,
//!     ).finalize(components::in_place_process_loader_component_static!());
//! ```

use capsules_extra::in_place_process_loader::InPlaceProcessLoader;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::dynamic_process_loading;

// Setup static space for the objects.
#[macro_export]
macro_rules! in_place_process_loader_component_static {
    () => {{
        kernel::static_buf!(capsules_extra::in_place_process_loader::InPlaceProcessLoader<'static>)
    };};
}

pub struct InPlaceProcessLoaderComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    loading_driver: &'static dyn dynamic_process_loading::DynamicProcessLoading,
}

impl InPlaceProcessLoaderComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        loading_driver: &'static dyn dynamic_process_loading::DynamicProcessLoading,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            loading_driver,
        }
    }
}

impl Component for InPlaceProcessLoaderComponent {
    type StaticInput = &'static mut MaybeUninit<InPlaceProcessLoader<'static>>;
    type Output = &'static InPlaceProcessLoader<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        static_buffer.write(InPlaceProcessLoader::new(
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.loading_driver,
        ))
    }
}
