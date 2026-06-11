// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).
//! Intended for demonstration / testing of the app software watchdog capsule.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities::{ProcessManagementCapability, ProcessRestartCapability};
use kernel::component::Component;
use kernel::debug;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::{capabilities, create_capability};
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

/// Capability for Restarting Processes needed for the app software watchdog.
struct PRCapability;
unsafe impl ProcessRestartCapability for PRCapability {}

type AppSoftwareWatchdog = capsules_extra::app_software_watchdog::AppSoftwareWatchdog<
    'static,
    VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
    PRCapability,
>;

/// Supported drivers by the platform
pub struct Platform {
    base_platform: nrf52840dk_test_base_lib::Platform,
    app_software_watchdog: &'static AppSoftwareWatchdog,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::app_software_watchdog::DRIVER_NUM => {
                f(Some(self.app_software_watchdog))
            }
            _ => self.base_platform.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>>
    for Platform
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = nrf52840dk_test_base_lib::SchedulerInUse;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.base_platform.scheduler()
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        self.base_platform.scheduler_timer()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let (board_kernel, base_platform, chip, _default_peripherals, _mux_uart, mux_alarm) =
        nrf52840dk_test_base_lib::start();

    //--------------------------------------------------------------------------
    // App Software Watchdog
    //--------------------------------------------------------------------------

    // Capability for managing processes that is owned/used by ProcessRestarter
    // to restart
    struct PMCapability;
    unsafe impl ProcessManagementCapability for PMCapability {}

    let app_software_watchdog =
        components::app_software_watchdog::AppSoftwareWatchdogComponent::new(
            mux_alarm,
            board_kernel,
            PRCapability,
            PMCapability,
        )
        .finalize(components::app_softare_watchdog_component_static!(
            nrf52840::rtc::Rtc,
            PRCapability,
            PMCapability,
        ));

    //--------------------------------------------------------------------------
    // Load Processes
    //--------------------------------------------------------------------------

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    // These symbols are defined in the linker script.
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

    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        ),
        &nrf52840dk_test_base_lib::FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    //--------------------------------------------------------------------------
    // KERNEL LOOP
    //--------------------------------------------------------------------------

    let platform = Platform {
        base_platform,
        app_software_watchdog,
    };

    let main_loop_capability = create_capability!(kernel::capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
