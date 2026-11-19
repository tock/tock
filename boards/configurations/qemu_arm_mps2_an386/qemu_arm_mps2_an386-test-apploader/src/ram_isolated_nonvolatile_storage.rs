// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for isolated non-volatile storage backed by RAM.
//!
//! This is the same userspace-facing stack as
//! `components::isolated_nonvolatile_storage`, but for platforms with no
//! real nonvolatile storage peripheral: the backing store is
//! [`crate::ram_nonvolatile_storage::RamNonvolatileStorage`] over a plain
//! `&'static mut [u8]` buffer rather than flash. Contents do not survive a
//! reset.
//!
//! Board-local, like [`crate::ram_nonvolatile_storage`]: this isn't
//! published from `boards/components` for other boards to depend on.

use crate::ram_nonvolatile_storage::RamNonvolatileStorage;
use capsules_extra::isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage;
use core::mem::MaybeUninit;
use kernel::capabilities::MemoryAllocationCapability;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil;

// Setup static space for the objects.
macro_rules! ram_isolated_nonvolatile_storage_component_static {
    ($APP_REGION_SIZE:expr $(,)?) => {{
        let storage =
            kernel::static_buf!(crate::ram_nonvolatile_storage::RamNonvolatileStorage<'static>);
        let ns = kernel::static_buf!(
            capsules_extra::isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage<
                'static,
                $APP_REGION_SIZE,
            >
        );
        let buffer =
            kernel::static_buf!([u8; capsules_extra::isolated_nonvolatile_storage_driver::BUF_LEN]);

        (storage, ns, buffer)
    }};
}
pub(crate) use ram_isolated_nonvolatile_storage_component_static;

pub type RamIsolatedNonvolatileStorageComponentType<const APP_REGION_SIZE: usize> =
    IsolatedNonvolatileStorage<'static, APP_REGION_SIZE>;

pub struct RamIsolatedNonvolatileStorageComponent<
    const APP_REGION_SIZE: usize,
    CAP: MemoryAllocationCapability + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    memory: &'static mut [u8],
    mem_cap: CAP,
}

impl<const APP_REGION_SIZE: usize, CAP: MemoryAllocationCapability + 'static>
    RamIsolatedNonvolatileStorageComponent<APP_REGION_SIZE, CAP>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        memory: &'static mut [u8],
        mem_cap: CAP,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            memory,
            mem_cap,
        }
    }
}

impl<const APP_REGION_SIZE: usize, CAP: MemoryAllocationCapability + 'static> Component
    for RamIsolatedNonvolatileStorageComponent<APP_REGION_SIZE, CAP>
{
    type StaticInput = (
        &'static mut MaybeUninit<RamNonvolatileStorage<'static>>,
        &'static mut MaybeUninit<IsolatedNonvolatileStorage<'static, APP_REGION_SIZE>>,
        &'static mut MaybeUninit<
            [u8; capsules_extra::isolated_nonvolatile_storage_driver::BUF_LEN],
        >,
    );
    type Output = &'static IsolatedNonvolatileStorage<'static, APP_REGION_SIZE>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer
            .2
            .write([0; capsules_extra::isolated_nonvolatile_storage_driver::BUF_LEN]);

        let memory_len = self.memory.len();
        let storage = static_buffer
            .0
            .write(RamNonvolatileStorage::new(self.memory));
        storage.register();

        let nonvolatile_storage = static_buffer.1.write(IsolatedNonvolatileStorage::new(
            storage,
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
            0,          // Start address for userspace accessible region
            memory_len, // Length of userspace accessible region
            buffer,
        ));
        hil::nonvolatile_storage::NonvolatileStorage::set_client(storage, nonvolatile_storage);
        nonvolatile_storage
    }
}
