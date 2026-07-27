// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for any App Flash Driver.
//!
//! Usage
//! -----
//! ```rust
//! let app_flash =
//!     components::app_flash_driver::AppFlashComponent::new(board_kernel, &base_peripherals.nvmc)
//!         .finalize(components::app_flash_component_static!(
//!             nrf52833::nvmc::Nvmc,
//!             512
//!     ));
//! ```

use capsules_extra::app_flash_driver::AppFlash;
use capsules_extra::nonvolatile_to_pages::NonvolatileToPages;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::nonvolatile_storage::NonvolatileStorage;

#[macro_export]
macro_rules! app_flash_component_static {
    ($F:ty, $buffer_size: literal) => {{
        let buffer = kernel::static_buf!([u8; $buffer_size]);
        let page_buffer = kernel::static_buf!(<$F as kernel::hil::flash::Flash>::Page);
        let nv_to_page = kernel::static_buf!(
            capsules_extra::nonvolatile_to_pages::NonvolatileToPages<'static, $F>
        );
        let app_flash = kernel::static_buf!(capsules_extra::app_flash_driver::AppFlash<'static>);
        (buffer, page_buffer, nv_to_page, app_flash)
    };};
}

pub type AppFlashComponentType = capsules_extra::app_flash_driver::AppFlash<'static>;

pub struct AppFlashComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    const BUF_LEN: usize,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    storage: &'static F,
    mem_cap: CAP,
}

impl<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    const BUF_LEN: usize,
    CAP: MemoryAllocationCapability + 'static,
> AppFlashComponent<F, BUF_LEN, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        storage: &'static F,
        mem_cap: CAP,
    ) -> AppFlashComponent<F, BUF_LEN, CAP> {
        AppFlashComponent {
            board_kernel,
            driver_num,
            storage,
            mem_cap,
        }
    }
}

impl<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, NonvolatileToPages<'static, F>>,
    const BUF_LEN: usize,
    CAP: MemoryAllocationCapability + 'static,
> Component for AppFlashComponent<F, BUF_LEN, CAP>
{
    type StaticInput = (
        &'static mut MaybeUninit<[u8; BUF_LEN]>,
        &'static mut MaybeUninit<<F as hil::flash::Flash>::Page>,
        &'static mut MaybeUninit<NonvolatileToPages<'static, F>>,
        &'static mut MaybeUninit<AppFlash<'static>>,
    );
    type Output = &'static AppFlash<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer.0.write([0; BUF_LEN]);

        let flash_pagebuffer = static_buffer
            .1
            .write(<F as hil::flash::Flash>::Page::default());

        let nv_to_page = static_buffer
            .2
            .write(NonvolatileToPages::new(self.storage, flash_pagebuffer));
        self.storage.set_client(nv_to_page);

        let app_flash = static_buffer
            .3
            .write(capsules_extra::app_flash_driver::AppFlash::new(
                nv_to_page,
                self.board_kernel
                    .create_grant(self.driver_num, &self.mem_cap),
                buffer,
            ));

        nv_to_page.set_client(app_flash);

        app_flash
    }
}
