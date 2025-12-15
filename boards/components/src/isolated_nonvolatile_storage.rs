// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for isolated non-volatile storage Drivers.
//!
//! This provides one component, IsolatedNonvolatileStorageComponent, which provides
//! a system call interface to isolated non-volatile storage.
//!
//! This differs from NonvolatileStorageComponent in that it provides isolation
//! between apps. Each app has it's own storage address space (that starts at 0)
//! which doesn't interfere with other apps.
//!
//! Usage
//! -----
//! ```rust
//! let nonvolatile_storage = components::isolated_nonvolatile_storage::IsolatedNonvolatileStorageComponent::new(
//!     board_kernel,
//!     &sam4l::flashcalw::FLASH_CONTROLLER,
//!     0x60000,
//!     0x20000,
//! )
//! .finalize(components::isolated_nonvolatile_storage_component_static!(
//!     sam4l::flashcalw::FLASHCALW
//! ));
//! ```

use capsules_extra::isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage;
use capsules_extra::nonvolatile_to_pages::NonvolatileToPages;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::DriverNumber;

// How much storage space to allocate per-app. Currently, regions are not
// growable, so this will be all the space the app gets.
// Only affects newly allocated regions. Old regions can remain the same size.
pub const ISOLATED_NONVOLATILE_STORAGE_APP_REGION_SIZE_DEFAULT: usize = 2048;

// Setup static space for the objects.
#[macro_export]
macro_rules! isolated_nonvolatile_storage_component_static {
    ($F:ty, $APP_REGION_SIZE:expr $(,)?) => {{
        let page = kernel::static_buf!(<$F as kernel::hil::flash::Flash>::Page);
        let ntp = kernel::static_buf!(
            capsules_extra::nonvolatile_to_pages::NonvolatileToPages<'static, $F>
        );
        let ns = kernel::static_buf!(
            capsules_extra::isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage<
                'static,
                $APP_REGION_SIZE,
            >
        );
        let buffer =
            kernel::static_buf!([u8; capsules_extra::isolated_nonvolatile_storage_driver::BUF_LEN]);

        (page, ntp, ns, buffer)
    };};
}

pub type IsolatedNonvolatileStorageComponentType<const APP_REGION_SIZE: usize> =
    IsolatedNonvolatileStorage<'static, APP_REGION_SIZE>;

pub struct IsolatedNonvolatileStorageComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    const APP_REGION_SIZE: usize,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: DriverNumber,
    flash: &'static F,
    userspace_start: usize,
    userspace_length: usize,
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
        const APP_REGION_SIZE: usize,
    > IsolatedNonvolatileStorageComponent<F, APP_REGION_SIZE>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: DriverNumber,
        flash: &'static F,
        userspace_start: usize,
        userspace_length: usize,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            flash,
            userspace_start,
            userspace_length,
        }
    }
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
        const APP_REGION_SIZE: usize,
    > Component for IsolatedNonvolatileStorageComponent<F, APP_REGION_SIZE>
{
    type StaticInput = (
        &'static mut MaybeUninit<<F as hil::flash::Flash>::Page>,
        &'static mut MaybeUninit<NonvolatileToPages<'static, F>>,
        &'static mut MaybeUninit<IsolatedNonvolatileStorage<'static, APP_REGION_SIZE>>,
        &'static mut MaybeUninit<
            [u8; capsules_extra::isolated_nonvolatile_storage_driver::BUF_LEN],
        >,
    );
    type Output = &'static IsolatedNonvolatileStorage<'static, APP_REGION_SIZE>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let buffer = static_buffer
            .3
            .write([0; capsules_extra::isolated_nonvolatile_storage_driver::BUF_LEN]);

        let flash_pagebuffer = static_buffer
            .0
            .write(<F as hil::flash::Flash>::Page::default());

        let nv_to_page = static_buffer
            .1
            .write(NonvolatileToPages::new(self.flash, flash_pagebuffer));
        hil::flash::HasClient::set_client(self.flash, nv_to_page);

        let nonvolatile_storage = static_buffer.2.write(IsolatedNonvolatileStorage::new(
            nv_to_page,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.userspace_start, // Start address for userspace accessible region
            self.userspace_length, // Length of userspace accessible region
            buffer,
        ));
        hil::nonvolatile_storage::NonvolatileStorage::set_client(nv_to_page, nonvolatile_storage);
        nonvolatile_storage
    }
}
