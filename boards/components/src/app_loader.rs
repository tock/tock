// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for Dynamic App Loading Drivers.
//!
//! This provides one component, AppLoaderComponent, which provides
//! a system call interface to the dynamic app loading capsule.
//!
//! Example instantiation:
//!
//! ```rust
//! # use kernel::static_init;
//!
//! type NonVolatilePages = components::dynamic_binary_storage::NVPages<nrf52840::nvmc::Nvmc>;
//! type DynamicBinaryStorage<'a> = kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage<
//! 'static,
//! nrf52840::chip::NRF52<'a, Nrf52840DefaultPeripherals<'a>>,
//! kernel::process::ProcessStandardDebugFull,
//! NonVolatilePages,
//! >;
//!
//! let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
//!     board_kernel,
//!     capsules_extra::app_loader::DRIVER_NUM,
//!     dynamic_binary_storage,
//!     dynamic_binary_storage,
//!     ).finalize(components::app_loader_component_static!(
//!     DynamicBinaryStorage<'static>,
//!     DynamicBinaryStorage<'static>,
//!     ));
//! ```

use capsules_extra::app_loader::AppLoader;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::dynamic_binary_storage;

// Setup static space for the objects.
#[macro_export]
macro_rules! app_loader_component_static {
    ($S:ty, $L:ty $(,)?) => {{
        let al = kernel::static_buf!(capsules_extra::app_loader::AppLoader<$S, $L>);
        let buffer = kernel::static_buf!([u8; capsules_extra::app_loader::BUF_LEN]);

        (al, buffer)
    };};
}

pub struct AppLoaderComponent<
    S: dynamic_binary_storage::DynamicBinaryStore + 'static,
    L: dynamic_binary_storage::DynamicProcessLoad + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    storage_driver: &'static S,
    load_driver: &'static L,
}

impl<
        S: dynamic_binary_storage::DynamicBinaryStore + 'static,
        L: dynamic_binary_storage::DynamicProcessLoad + 'static,
    > AppLoaderComponent<S, L>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        storage_driver: &'static S,
        load_driver: &'static L,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            storage_driver,
            load_driver,
        }
    }
}

impl<
        S: dynamic_binary_storage::DynamicBinaryStore + 'static,
        L: dynamic_binary_storage::DynamicProcessLoad + 'static,
    > Component for AppLoaderComponent<S, L>
{
    type StaticInput = (
        &'static mut MaybeUninit<AppLoader<S, L>>,
        &'static mut MaybeUninit<[u8; capsules_extra::app_loader::BUF_LEN]>,
    );
    type Output = &'static AppLoader<S, L>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let buffer = static_buffer
            .1
            .write([0; capsules_extra::app_loader::BUF_LEN]);

        let dynamic_app_loader = static_buffer.0.write(AppLoader::new(
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.storage_driver,
            self.load_driver,
            buffer,
        ));
        dynamic_binary_storage::DynamicBinaryStore::set_storage_client(
            self.storage_driver,
            dynamic_app_loader,
        );
        dynamic_binary_storage::DynamicProcessLoad::set_load_client(
            self.load_driver,
            dynamic_app_loader,
        );
        dynamic_app_loader
    }
}
