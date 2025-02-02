// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for dynamic process loading.
//!
//! This provides one component, ProcessLoaderComponent, which provides
//! a system call interface to DynamicProcessLoader.
//!
//!```rust, ignore
//! # use kernel::static_init;
//!
//! let dynamic_process_loader = components::dyn_process_loader::ProcessLoaderComponent::new(
//!         &mut *addr_of_mut!(PROCESSES),
//!         loader,
//!     ).finalize(components::process_loader_component_static!());
//! ```

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::dynamic_process_loading::DynamicProcessLoader;
use kernel::process;

// Setup static space for the objects.
#[macro_export]
macro_rules! process_loader_component_static {
    () => {{
        kernel::static_buf!(kernel::dynamic_process_loading::DynamicProcessLoader<'static>)
    };};
}

pub struct ProcessLoaderComponent {
    processes: &'static mut [Option<&'static dyn process::Process>],
    loader_driver: &'static dyn process::ProcessLoadingAsync<'static>,
}

impl ProcessLoaderComponent {
    pub fn new(
        processes: &'static mut [Option<&'static dyn process::Process>],
        loader_driver: &'static dyn process::ProcessLoadingAsync<'static>,
    ) -> Self {
        Self {
            processes,
            loader_driver,
        }
    }
}

impl Component for ProcessLoaderComponent {
    type StaticInput = &'static mut MaybeUninit<DynamicProcessLoader<'static>>;
    type Output = &'static DynamicProcessLoader<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        static_buffer.write(DynamicProcessLoader::new(
            self.processes,
            self.loader_driver,
        ))
    }
}
