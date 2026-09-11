// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for dynamic binary storage backed by RAM.
//!
//! This is the same role as `components::dynamic_binary_storage`, but for
//! platforms with no real flash: the backing store is a
//! [`crate::ram_nonvolatile_storage::RamNonvolatileStorage`] rather than a
//! `hil::flash::Flash` device, so no `NonvolatileToPages` adapter is
//! needed -- `SequentialDynamicBinaryStorage` already speaks the
//! nonvolatile-storage HIL directly.
//!
//! Board-local, like [`crate::ram_nonvolatile_storage`]: this isn't
//! published from `boards/components` for other boards to depend on.

use crate::ram_nonvolatile_storage::RamNonvolatileStorage;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage;
use kernel::hil;
use kernel::platform::chip::Chip;
use kernel::process::{ProcessStandardDebug, SequentialProcessLoaderMachine};

// Setup static space for the objects.
macro_rules! ram_dynamic_binary_storage_component_static {
    ($C:ty, $D:ty $(,)?) => {{
        let pl = kernel::static_buf!(
            kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage<
                'static,
                'static,
                $C,
                $D,
                crate::ram_nonvolatile_storage::RamNonvolatileStorage<'static>,
            >
        );
        let buffer = kernel::static_buf!([u8; kernel::dynamic_binary_storage::BUF_LEN]);

        (pl, buffer)
    }};
}
pub(crate) use ram_dynamic_binary_storage_component_static;

pub struct RamDynamicBinaryStorageComponent<C: Chip + 'static, D: ProcessStandardDebug + 'static> {
    board_kernel: &'static kernel::Kernel,
    nv_storage: &'static RamNonvolatileStorage<'static>,
    loader_driver: &'static SequentialProcessLoaderMachine<'static, C, D>,
}

impl<C: Chip + 'static, D: ProcessStandardDebug + 'static> RamDynamicBinaryStorageComponent<C, D> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        nv_storage: &'static RamNonvolatileStorage<'static>,
        loader_driver: &'static SequentialProcessLoaderMachine<'static, C, D>,
    ) -> Self {
        Self {
            board_kernel,
            nv_storage,
            loader_driver,
        }
    }
}

impl<C: Chip + 'static, D: ProcessStandardDebug + 'static> Component
    for RamDynamicBinaryStorageComponent<C, D>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            SequentialDynamicBinaryStorage<'static, 'static, C, D, RamNonvolatileStorage<'static>>,
        >,
        &'static mut MaybeUninit<[u8; kernel::dynamic_binary_storage::BUF_LEN]>,
    );
    type Output = &'static SequentialDynamicBinaryStorage<
        'static,
        'static,
        C,
        D,
        RamNonvolatileStorage<'static>,
    >;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer
            .1
            .write([0; kernel::dynamic_binary_storage::BUF_LEN]);

        let dynamic_binary_storage = static_buffer.0.write(SequentialDynamicBinaryStorage::new(
            self.board_kernel,
            self.nv_storage,
            self.loader_driver,
            buffer,
        ));
        hil::nonvolatile_storage::NonvolatileStorage::set_client(
            self.nv_storage,
            dynamic_binary_storage,
        );
        self.loader_driver
            .set_runtime_client(dynamic_binary_storage);
        dynamic_binary_storage.register();
        dynamic_binary_storage
    }
}
