// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for Dynamic App Loading Drivers.
//!
//! This provides one component, NonvolatileStorageComponent, which provides
//! a system call interface to non-volatile storage.
//!
//! Example instantiation:
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
//!     board_kernel,
//!     capsules_extra::app_loader::DRIVER_NUM,
//!     dynamic_process_loader,
//!     ).finalize(components::app_loader_component_static!());
//! ```

use capsules_extra::app_loader::AppLoader;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::dynamic_process_loading;

// Setup static space for the objects.
#[macro_export]
macro_rules! app_loader_component_static {
    () => {{
        let al = kernel::static_buf!(capsules_extra::app_loader::AppLoader<'static>);
        let buffer = kernel::static_buf!([u8; capsules_extra::app_loader::BUF_LEN]);

        (al, buffer)
    };};
}

pub struct AppLoaderComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    driver: &'static dyn dynamic_process_loading::DynamicProcessLoading,
}

impl AppLoaderComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        driver: &'static dyn dynamic_process_loading::DynamicProcessLoading,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            driver,
        }
    }
}

impl Component for AppLoaderComponent {
    type StaticInput = (
        &'static mut MaybeUninit<AppLoader<'static>>,
        &'static mut MaybeUninit<[u8; capsules_extra::app_loader::BUF_LEN]>,
    );
    type Output = &'static AppLoader<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let buffer = static_buffer
            .1
            .write([0; capsules_extra::app_loader::BUF_LEN]);

        let dynamic_app_loader = static_buffer.0.write(AppLoader::new(
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.driver,
            buffer,
        ));
        kernel::dynamic_process_loading::DynamicProcessLoading::set_client(
            self.driver,
            dynamic_app_loader,
        );
        dynamic_app_loader
    }
}
