// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for TicKV KV System Driver.
//!
//! This provides one component, TicKVComponent, which provides
//! a system call inteface to non-volatile storage.
//!
//! Usage
//! -----
//! ```rust
//!    let flash_ctrl_read_buf = static_init!(
//!        [u8; lowrisc::flash_ctrl::PAGE_SIZE],
//!        [0; lowrisc::flash_ctrl::PAGE_SIZE]
//!    );
//!    let page_buffer = static_init!(
//!        lowrisc::flash_ctrl::LowRiscPage,
//!        lowrisc::flash_ctrl::LowRiscPage::default()
//!    );
//!
//!    let mux_flash = components::tickv::FlashMuxComponent::new(&peripherals.flash_ctrl).finalize(
//!        components::flash_user_component_static!(lowrisc::flash_ctrl::FlashCtrl),
//!    );
//!
//!    // SipHash
//!    let sip_hash = static_init!(
//!        capsules_extra::sip_hash::SipHasher24,
//!        capsules_extra::sip_hash::SipHasher24::new()
//!    );
//!    sip_hash.register();
//!
//!    let tickv = components::tickv::TicKVComponent::new(
//!        sip_hash,
//!        &mux_flash,
//!        0x20040000 / lowrisc::flash_ctrl::PAGE_SIZE,
//!        0x40000,
//!        flash_ctrl_read_buf,
//!        page_buffer,
//!    )
//!    .finalize(components::tickv_component_static!(
//!        lowrisc::flash_ctrl::FlashCtrl,
//!        capsules_extra::sip_hash::SipHasher24
//!    ));
//!    hil::flash::HasClient::set_client(&peripherals.flash_ctrl, mux_flash);
//! ```

use capsules_core::virtualizers::virtual_flash::FlashUser;
use capsules_core::virtualizers::virtual_flash::MuxFlash;
use capsules_extra::tickv::TicKVStore;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::flash::HasClient;
use kernel::hil::hasher::Hasher;

// Setup static space for the objects.
#[macro_export]
macro_rules! tickv_component_static {
    ($F:ty, $H:ty, $PAGE_SIZE:expr $(,)?) => {{
        let flash =
            kernel::static_buf!(capsules_core::virtualizers::virtual_flash::FlashUser<'static, $F>);
        let tickv = kernel::static_buf!(
            capsules_extra::tickv::TicKVStore<
                'static,
                capsules_core::virtualizers::virtual_flash::FlashUser<'static, $F>,
                $H,
                $PAGE_SIZE,
            >
        );

        (flash, tickv)
    };};
}

#[macro_export]
macro_rules! tickv_dedicated_flash_component_static {
    ($F:ty, $H:ty, $PAGE_SIZE:expr $(,)?) => {{
        let tickfs_read_buffer = kernel::static_buf!([u8; $PAGE_SIZE]);
        let tickv =
            kernel::static_buf!(capsules_extra::tickv::TicKVStore<'static, $F, $H, $PAGE_SIZE>);

        (tickv, tickfs_read_buffer)
    };};
}

pub struct TicKVComponent<
    F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, MuxFlash<'static, F>>,
    H: 'static + Hasher<'static, 8>,
    const PAGE_SIZE: usize,
> {
    mux_flash: &'static MuxFlash<'static, F>,
    hasher: &'static H,
    region_offset: usize,
    flash_size: usize,
    tickfs_read_buf: &'static mut [u8; PAGE_SIZE],
    flash_read_buffer: &'static mut F::Page,
}

impl<
        F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, MuxFlash<'static, F>>,
        H: Hasher<'static, 8>,
        const PAGE_SIZE: usize,
    > TicKVComponent<F, H, PAGE_SIZE>
{
    pub fn new(
        hasher: &'static H,
        mux_flash: &'static MuxFlash<'static, F>,
        region_offset: usize,
        flash_size: usize,
        tickfs_read_buf: &'static mut [u8; PAGE_SIZE],
        flash_read_buffer: &'static mut F::Page,
    ) -> Self {
        Self {
            hasher,
            mux_flash,
            region_offset,
            flash_size,
            tickfs_read_buf,
            flash_read_buffer,
        }
    }
}

impl<
        F: 'static + hil::flash::Flash + hil::flash::HasClient<'static, MuxFlash<'static, F>>,
        H: 'static + Hasher<'static, 8>,
        const PAGE_SIZE: usize,
    > Component for TicKVComponent<F, H, PAGE_SIZE>
{
    type StaticInput = (
        &'static mut MaybeUninit<FlashUser<'static, F>>,
        &'static mut MaybeUninit<TicKVStore<'static, FlashUser<'static, F>, H, PAGE_SIZE>>,
    );
    type Output = &'static TicKVStore<'static, FlashUser<'static, F>, H, PAGE_SIZE>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let _grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_flash = static_buffer.0.write(FlashUser::new(self.mux_flash));

        let driver = static_buffer.1.write(TicKVStore::new(
            virtual_flash,
            self.hasher,
            self.tickfs_read_buf,
            self.flash_read_buffer,
            self.region_offset,
            self.flash_size,
        ));
        virtual_flash.set_client(driver);
        driver.initialise();
        driver
    }
}

pub struct TicKVDedicatedFlashComponent<
    F: 'static
        + hil::flash::Flash
        + hil::flash::HasClient<'static, TicKVStore<'static, F, H, PAGE_SIZE>>,
    H: 'static + Hasher<'static, 8>,
    const PAGE_SIZE: usize,
> {
    flash: &'static F,
    hasher: &'static H,
    region_offset: usize,
    flash_size: usize,
    flash_read_buffer: &'static mut F::Page,
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, TicKVStore<'static, F, H, PAGE_SIZE>>,
        H: Hasher<'static, 8>,
        const PAGE_SIZE: usize,
    > TicKVDedicatedFlashComponent<F, H, PAGE_SIZE>
{
    pub fn new(
        hasher: &'static H,
        flash: &'static F,
        region_offset: usize,
        flash_size: usize,
        flash_read_buffer: &'static mut F::Page,
    ) -> Self {
        Self {
            hasher,
            flash,
            region_offset,
            flash_size,
            flash_read_buffer,
        }
    }
}

impl<
        F: 'static
            + hil::flash::Flash
            + hil::flash::HasClient<'static, TicKVStore<'static, F, H, PAGE_SIZE>>,
        H: 'static + Hasher<'static, 8>,
        const PAGE_SIZE: usize,
    > Component for TicKVDedicatedFlashComponent<F, H, PAGE_SIZE>
{
    type StaticInput = (
        &'static mut MaybeUninit<TicKVStore<'static, F, H, PAGE_SIZE>>,
        &'static mut MaybeUninit<[u8; PAGE_SIZE]>,
    );
    type Output = &'static TicKVStore<'static, F, H, PAGE_SIZE>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let _grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let tickfs_read_buf = static_buffer.1.write([0; PAGE_SIZE]);

        let tickv = static_buffer.0.write(TicKVStore::new(
            self.flash,
            self.hasher,
            tickfs_read_buf,
            self.flash_read_buffer,
            self.region_offset,
            self.flash_size,
        ));
        self.flash.set_client(tickv);
        self.hasher.set_client(tickv);
        tickv.initialise();
        tickv
    }
}
