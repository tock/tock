// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for Flash
//!
//! Provides `FlashMux` and `FlashUser` (virtual flash).
//!
//! Usage
//! -----
//! ```rust
//!    let mux_flash = components::flash::FlashMuxComponent::new(&base_peripherals.nvmc).finalize(
//!       components::flash_mux_component_static!(nrf52833::nvmc::Nvmc),
//!    );
//!
//!    let virtual_app_flash = components::flash::FlashUserComponent::new(mux_flash).finalize(
//!       components::flash_user_component_static!(nrf52833::nvmc::Nvmc),
//!    );
//! ```

use capsules_core::virtualizers::virtual_flash::FlashUser;
use capsules_core::virtualizers::virtual_flash::MuxFlash;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::flash::{Flash, HasClient};

// Setup static space for the objects.
#[macro_export]
macro_rules! flash_user_component_static {
    ($F:ty) => {{
        kernel::static_buf!(capsules_core::virtualizers::virtual_flash::FlashUser<'static, $F>)
    };};
}

#[macro_export]
macro_rules! flash_mux_component_static {
    ($F:ty) => {{
        kernel::static_buf!(capsules_core::virtualizers::virtual_flash::MuxFlash<'static, $F>)
    };};
}

pub struct FlashMuxComponent<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> {
    flash: &'static F,
}

impl<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> FlashMuxComponent<F> {
    pub fn new(flash: &'static F) -> FlashMuxComponent<F> {
        FlashMuxComponent { flash }
    }
}

impl<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> Component
    for FlashMuxComponent<F>
{
    type StaticInput = &'static mut MaybeUninit<MuxFlash<'static, F>>;
    type Output = &'static MuxFlash<'static, F>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_flash = s.write(MuxFlash::new(self.flash));
        HasClient::set_client(self.flash, mux_flash);

        mux_flash
    }
}

pub struct FlashUserComponent<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> {
    mux_flash: &'static MuxFlash<'static, F>,
}

impl<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> FlashUserComponent<F> {
    pub fn new(mux_flash: &'static MuxFlash<'static, F>) -> Self {
        Self { mux_flash }
    }
}

impl<F: 'static + Flash + HasClient<'static, MuxFlash<'static, F>>> Component
    for FlashUserComponent<F>
{
    type StaticInput = &'static mut MaybeUninit<FlashUser<'static, F>>;
    type Output = &'static FlashUser<'static, F>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(FlashUser::new(self.mux_flash))
    }
}
