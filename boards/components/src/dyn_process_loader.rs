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
//!     &mut *addr_of_mut!(PROCESSES),
//!     loader,
//! )
//! .finalize(components::process_loader_component_static!(
//!     nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
//!     kernel::process::ProcessStandardDebugFull,
//! ));
//! ```

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::dynamic_process_loading::DynamicProcessLoader;
use kernel::process;
use kernel::platform::chip::Chip;
use kernel::process::ProcessStandardDebug;
use kernel::process::SequentialProcessLoaderMachine;
use kernel::process::ProcessLoadingAsync;

// Setup static space for the objects.
#[macro_export]
macro_rules! process_loader_component_static {
    ($C:ty, $D:ty $(,)?) => {{
        kernel::static_buf!(kernel::dynamic_process_loading::DynamicProcessLoader<'static, $C, $D>)
    };};
}

pub struct ProcessLoaderComponent <
C: Chip + 'static,
D: ProcessStandardDebug + 'static,
>{
    processes: &'static mut [Option<&'static dyn process::Process>],
    loader_driver:  &'static SequentialProcessLoaderMachine<'static, C, D>,
}

impl  <
C: Chip + 'static,
D: ProcessStandardDebug + 'static,
>ProcessLoaderComponent <C, D>
{
    pub fn new(
        processes: &'static mut [Option<&'static dyn process::Process>],
        loader_driver:  &'static SequentialProcessLoaderMachine<'static, C, D>,
    ) -> Self {
        Self {
            processes,
            loader_driver,
        }
    }
}

impl  <
C: Chip + 'static,
D: ProcessStandardDebug + 'static,
>Component for ProcessLoaderComponent <C, D> {
    type StaticInput = &'static mut MaybeUninit<DynamicProcessLoader<'static, C, D>>;
    type Output = &'static DynamicProcessLoader<'static, C, D>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let dynamic_process_loader = static_buffer.write(DynamicProcessLoader::new(
            self.processes,
            self.loader_driver,
        ));
    self.loader_driver
        .set_runtime_client(dynamic_process_loader);
    dynamic_process_loader
    }
}
