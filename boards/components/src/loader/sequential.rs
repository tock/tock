// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for creating a sequential process loader.
//!
//! `ProcessLoaderSequentialComponent` uses the standard Tock assumptions about
//! where processes are stored in flash and what RAM is allocated for process
//! use.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::platform::chip::Chip;
use kernel::process::ProcessLoadingAsync;

#[macro_export]
macro_rules! process_loader_sequential_component_static {
    ($C:ty, $NUMPROCS:expr $(,)?) => {{
        let loader = kernel::static_buf!(kernel::process::SequentialProcessLoaderMachine<
            $C,
        >);
        let process_binary_array = kernel::static_buf!(
            [Option<kernel::process::ProcessBinary>; $NUMPROCS]
        );

       (loader, process_binary_array)
    };};
}

pub type ProcessLoaderSequentialComponentType<C> =
    kernel::process::SequentialProcessLoaderMachine<'static, C>;

pub struct ProcessLoaderSequentialComponent<C: Chip + 'static, const NUM_PROCS: usize> {
    checker: &'static kernel::process::ProcessCheckerMachine,
    processes: &'static mut [Option<&'static dyn kernel::process::Process>],
    kernel: &'static kernel::Kernel,
    chip: &'static C,
    fault_policy: &'static dyn kernel::process::ProcessFaultPolicy,
    appid_policy: &'static dyn kernel::process_checker::AppIdPolicy,
}

impl<C: Chip, const NUM_PROCS: usize> ProcessLoaderSequentialComponent<C, NUM_PROCS> {
    pub fn new(
        checker: &'static kernel::process::ProcessCheckerMachine,
        processes: &'static mut [Option<&'static dyn kernel::process::Process>],
        kernel: &'static kernel::Kernel,
        chip: &'static C,
        fault_policy: &'static dyn kernel::process::ProcessFaultPolicy,
        appid_policy: &'static dyn kernel::process_checker::AppIdPolicy,
    ) -> Self {
        Self {
            checker,
            processes,
            kernel,
            chip,
            fault_policy,
            appid_policy,
        }
    }
}

impl<C: Chip, const NUM_PROCS: usize> Component for ProcessLoaderSequentialComponent<C, NUM_PROCS> {
    type StaticInput = (
        &'static mut MaybeUninit<kernel::process::SequentialProcessLoaderMachine<'static, C>>,
        &'static mut MaybeUninit<[Option<kernel::process::ProcessBinary>; NUM_PROCS]>,
    );

    type Output = &'static kernel::process::SequentialProcessLoaderMachine<'static, C>;

    fn finalize(mut self, s: Self::StaticInput) -> Self::Output {
        let proc_manage_cap =
            kernel::create_capability!(kernel::capabilities::ProcessManagementCapability);

        const ARRAY_REPEAT_VALUE: Option<kernel::process::ProcessBinary> = None;
        let process_binary_array = s.1.write([ARRAY_REPEAT_VALUE; NUM_PROCS]);

        // These symbols are defined in the standard Tock linker script.
        extern "C" {
            /// Beginning of the ROM region containing app images.
            static _sapps: u8;
            /// End of the ROM region containing app images.
            static _eapps: u8;
            /// Beginning of the RAM region for app memory.
            static mut _sappmem: u8;
            /// End of the RAM region for app memory.
            static _eappmem: u8;
        }

        let loader = unsafe {
            s.0.write(kernel::process::SequentialProcessLoaderMachine::new(
                self.checker,
                *core::ptr::addr_of_mut!(self.processes),
                process_binary_array,
                self.kernel,
                self.chip,
                core::slice::from_raw_parts(
                    core::ptr::addr_of!(_sapps),
                    core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
                ),
                core::slice::from_raw_parts_mut(
                    core::ptr::addr_of_mut!(_sappmem),
                    core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
                ),
                self.fault_policy,
                self.appid_policy,
                &proc_manage_cap,
            ))
        };
        self.checker.set_client(loader);
        loader.register();
        loader.start();
        loader
    }
}
