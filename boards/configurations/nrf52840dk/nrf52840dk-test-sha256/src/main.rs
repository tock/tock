// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::platform::{KernelResources, SyscallDriverLookup};

type Sha = components::sha::ShaSoftware256ComponentType;
const SHA_DIGEST_LEN: usize = 32;
type ShaDriver = components::sha::ShaDriverComponentType<Sha, SHA_DIGEST_LEN>;

type ChipHw = nrf52840dk_test_base_lib::ChipHw;

/// Supported drivers by the platform
pub struct Platform {
    base: nrf52840dk_test_base_lib::Platform,
    sha_driver: &'static ShaDriver,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::sha256_driver::DRIVER_NUM => f(Some(self.sha_driver)),
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter =
        <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::SyscallFilter;
    type ProcessFault =
        <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::ProcessFault;
    type Scheduler = <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::Scheduler;
    type SchedulerTimer =
        <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::SchedulerTimer;
    type WatchDog = <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::WatchDog;
    type ContextSwitchCallback =
        <nrf52840dk_test_base_lib::Platform as KernelResources<ChipHw>>::ContextSwitchCallback;

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        self.base.syscall_filter()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        self.base.process_fault()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.base.scheduler()
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        self.base.scheduler_timer()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        self.base.watchdog()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        self.base.context_switch_callback()
    }
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let (board_kernel, base_platform, chip, _default_peripherals, _mux_uart, _mux_alarm) =
        nrf52840dk_test_base_lib::start();

    //--------------------------------------------------------------------------
    // SHA256
    //--------------------------------------------------------------------------

    let sha = components::sha::ShaSoftware256Component::new()
        .finalize(components::sha_software_256_component_static!());

    let sha_driver = components::sha::ShaDriverComponent::new(
        board_kernel,
        capsules_extra::sha256_driver::DRIVER_NUM,
        sha,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::sha_driver_component_static!(
        Sha,
        SHA_DIGEST_LEN
    ));

    //--------------------------------------------------------------------------
    // PLATFORM
    //--------------------------------------------------------------------------

    let platform = Platform {
        base: base_platform,
        sha_driver,
    };

    //--------------------------------------------------------------------------
    // PROCESS LOADING
    //--------------------------------------------------------------------------

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

    let app_flash = core::slice::from_raw_parts(
        core::ptr::addr_of!(_sapps),
        core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
    );
    let app_memory = core::slice::from_raw_parts_mut(
        core::ptr::addr_of_mut!(_sappmem),
        core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
    );

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    kernel::process::load_processes(
        board_kernel,
        chip,
        app_flash,
        app_memory,
        &nrf52840dk_test_base_lib::FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        kernel::debug!("Error loading processes!");
        kernel::debug!("{:?}", err);
    });

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let main_loop_capability = create_capability!(kernel::capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
