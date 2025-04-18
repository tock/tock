// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for dynamic binary storage.
//!
//! This provides one component, SequentialBinaryStorageComponent, which provides
//! a system call interface to SequentialDynamicBinaryStorage.
//!
//!```rust, ignore
//! # use kernel::static_init;
//!
//! let dynamic_binary_storage = components::dyn_binary_storage::SequentialBinaryStorageComponent::new(
//!     &base_peripherals.nvmc,
//!     &loader,
//! )
//! .finalize(components::sequential_binary_storage_component_static!(
//!     nrf52840::nvmc::Nvmc,
//!     nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
//!     kernel::process::ProcessStandardDebugFull,
//! ));
//! ```

use capsules_extra::nonvolatile_to_pages::NonvolatileToPages;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage;
use kernel::hil;
use kernel::platform::chip::Chip;
use kernel::process::ProcessStandardDebug;
use kernel::process::SequentialProcessLoaderMachine;

pub type NVPages<F> = capsules_extra::nonvolatile_to_pages::NonvolatileToPages<'static, F>;

// Setup static space for the objects.
#[macro_export]
macro_rules! sequential_binary_storage_component_static {
    ($F:ty, $C:ty, $D:ty $(,)?) => {{
        let page = kernel::static_buf!(<$F as kernel::hil::flash::Flash>::Page);
        let ntp = kernel::static_buf!(
            capsules_extra::nonvolatile_to_pages::NonvolatileToPages<'static, $F>
        );
        let pl = kernel::static_buf!(
            kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage<
                'static,
                'static,
                $C,
                $D,
                capsules_extra::nonvolatile_to_pages::NonvolatileToPages<'static, $F>,
            >
        );
        let buffer = kernel::static_buf!([u8; kernel::dynamic_binary_storage::BUF_LEN]);

        (page, ntp, pl, buffer)
    };};
}

pub struct SequentialBinaryStorageComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    C: Chip + 'static,
    D: ProcessStandardDebug + 'static,
> {
    nv_flash: &'static F,
    loader_driver: &'static SequentialProcessLoaderMachine<'static, C, D>,
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
        C: 'static + Chip,
        D: 'static + ProcessStandardDebug,
    > SequentialBinaryStorageComponent<F, C, D>
{
    pub fn new(
        nv_flash: &'static F,
        loader_driver: &'static SequentialProcessLoaderMachine<'static, C, D>,
    ) -> Self {
        Self {
            nv_flash,
            loader_driver,
        }
    }
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
        C: 'static + Chip,
        D: 'static + ProcessStandardDebug,
    > Component for SequentialBinaryStorageComponent<F, C, D>
{
    type StaticInput = (
        &'static mut MaybeUninit<<F as hil::flash::Flash>::Page>,
        &'static mut MaybeUninit<NonvolatileToPages<'static, F>>,
        &'static mut MaybeUninit<
            SequentialDynamicBinaryStorage<'static, 'static, C, D, NonvolatileToPages<'static, F>>,
        >,
        &'static mut MaybeUninit<[u8; kernel::dynamic_binary_storage::BUF_LEN]>,
    );
    type Output = &'static SequentialDynamicBinaryStorage<
        'static,
        'static,
        C,
        D,
        NonvolatileToPages<'static, F>,
    >;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer
            .3
            .write([0; kernel::dynamic_binary_storage::BUF_LEN]);

        let flash_pagebuffer = static_buffer
            .0
            .write(<F as hil::flash::Flash>::Page::default());

        let nv_to_page = static_buffer
            .1
            .write(NonvolatileToPages::new(self.nv_flash, flash_pagebuffer));
        hil::flash::HasClient::set_client(self.nv_flash, nv_to_page);

        let dynamic_binary_storage = static_buffer.2.write(SequentialDynamicBinaryStorage::new(
            nv_to_page,
            self.loader_driver,
            buffer,
        ));
        hil::nonvolatile_storage::NonvolatileStorage::set_client(
            nv_to_page,
            dynamic_binary_storage,
        );
        self.loader_driver
            .set_runtime_client(dynamic_binary_storage);
        dynamic_binary_storage.register();
        dynamic_binary_storage
    }
}
