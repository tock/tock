// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK)
//! with a second, USB CDC-ACM based serial console.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::usb::Client;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::static_init;

type ChipHw = nrf52840dk_test_base_lib::ChipHw;

type Console2Driver = components::console::ConsoleComponentType;

const fn dup_driver_num(driver_num: usize, instance: usize) -> usize {
    (instance << 24) | driver_num
}

const CONSOLE2_DRIVER_NUM: usize = dup_driver_num(capsules_core::console::DRIVER_NUM, 1);

/// Supported drivers by the platform
pub struct Platform {
    base: nrf52840dk_test_base_lib::Platform,
    console2: &'static Console2Driver,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            CONSOLE2_DRIVER_NUM => f(Some(self.console2)),

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
    let (board_kernel, base_platform, chip, nrf52840_peripherals, _mux_uart, mux_alarm) =
        nrf52840dk_test_base_lib::start();

    //--------------------------------------------------------------------------
    // USB CDC-ACM (SECOND SERIAL CONSOLE)
    //--------------------------------------------------------------------------

    // Create the strings we include in the USB descriptor. We use the
    // hardcoded DEVICEADDR register on the nRF52 to set the serial number.
    let ficr = nrf52840::ficr::Ficr::new();
    let serial_number_buf = static_init!([u8; 17], [0; 17]);
    let serial_number_string: &'static str = ficr.address_str(serial_number_buf);
    let strings = static_init!(
        [&str; 3],
        [
            "Tock",                // Manufacturer
            "nRF52840DK - TockOS", // Product
            serial_number_string,  // Serial number
        ]
    );

    let cdc = components::cdc::CdcAcmComponent::new(
        &nrf52840_peripherals.usbd,
        capsules_extra::usb::cdc::MAX_CTRL_PACKET_SIZE_NRF52840,
        0x2341,
        0x005a,
        strings,
        mux_alarm,
        None,
    )
    .finalize(components::cdc_acm_component_static!(
        nrf52840::usbd::Usbd,
        nrf52840::rtc::Rtc
    ));

    // Virtualize the CDC-ACM connection so it can be used as a UART for the
    // second console, independent of the UART console set up by
    // `nrf52840dk-test-base`.
    let mux_uart2 = components::console::UartMuxComponent::new(cdc, 115200)
        .finalize(components::uart_mux_component_static!());

    // Setup a second serial console for userspace, this time over the
    // USB CDC-ACM channel, registered at a different driver number than the
    // standard UART console.
    let console2 = components::console::ConsoleComponent::new(
        board_kernel,
        CONSOLE2_DRIVER_NUM,
        mux_uart2,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!());

    // Configure the USB stack to enable the CDC-ACM serial port.
    cdc.enable();
    cdc.attach();

    //--------------------------------------------------------------------------
    // PLATFORM
    //--------------------------------------------------------------------------

    let platform = Platform {
        base: base_platform,
        console2,
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
