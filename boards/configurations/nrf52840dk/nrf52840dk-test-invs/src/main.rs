// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::component::Component;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::{capabilities, create_capability, static_init};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

mod invs_permissions;

const SPI_MOSI: Pin = Pin::P0_20;
const SPI_MISO: Pin = Pin::P0_21;
const SPI_CLK: Pin = Pin::P0_19;

const SPI_MX25R6435F_CHIP_SELECT: Pin = Pin::P0_17;
const SPI_MX25R6435F_WRITE_PROTECT_PIN: Pin = Pin::P0_22;
const SPI_MX25R6435F_HOLD_PIN: Pin = Pin::P0_23;

type ChipHw = nrf52840dk_test_base_lib::ChipHw;

const APP_STORAGE_REGION_SIZE: usize = 4096;

//------------------------------------------------------------------------------
// SYSCALL DRIVER TYPE DEFINITIONS
//------------------------------------------------------------------------------

type Mx25r6435f = components::mx25r6435f::Mx25r6435fComponentType<
    nrf52840::spi::SPIM<'static>,
    nrf52840::gpio::GPIOPin<'static>,
    nrf52840::rtc::Rtc<'static>,
>;
type InvsDriver = components::isolated_nonvolatile_storage::IsolatedNonvolatileStorageComponentType<
    APP_STORAGE_REGION_SIZE,
>;

/// Supported drivers by the platform
pub struct Platform {
    base: nrf52840dk_test_base_lib::Platform,
    invs: &'static InvsDriver,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM => f(Some(self.invs)),
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
    let base_peripherals = &nrf52840_peripherals.nrf52;

    //--------------------------------------------------------------------------
    // ONBOARD EXTERNAL FLASH
    //--------------------------------------------------------------------------

    let mux_spi = components::spi::SpiMuxComponent::new(&base_peripherals.spim0)
        .finalize(components::spi_mux_component_static!(nrf52840::spi::SPIM));

    base_peripherals.spim0.configure(
        nrf52840::pinmux::Pinmux::new(SPI_MOSI),
        nrf52840::pinmux::Pinmux::new(SPI_MISO),
        nrf52840::pinmux::Pinmux::new(SPI_CLK),
    );

    let mx25r6435f = components::mx25r6435f::Mx25r6435fComponent::new(
        Some(&nrf52840_peripherals.gpio_port[SPI_MX25R6435F_WRITE_PROTECT_PIN]),
        Some(&nrf52840_peripherals.gpio_port[SPI_MX25R6435F_HOLD_PIN]),
        &nrf52840_peripherals.gpio_port[SPI_MX25R6435F_CHIP_SELECT],
        mux_alarm,
        mux_spi,
    )
    .finalize(components::mx25r6435f_component_static!(
        nrf52840::spi::SPIM,
        nrf52840::gpio::GPIOPin,
        nrf52840::rtc::Rtc
    ));

    //--------------------------------------------------------------------------
    // NONVOLATILE STORAGE
    //--------------------------------------------------------------------------

    let invs = components::isolated_nonvolatile_storage::IsolatedNonvolatileStorageComponent::new(
        board_kernel,
        capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM,
        mx25r6435f,
        0x40000,  // start address
        0x100000, // length
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::isolated_nonvolatile_storage_component_static!(
        Mx25r6435f,
        APP_STORAGE_REGION_SIZE
    ));

    //--------------------------------------------------------------------------
    // Credential Checking
    //--------------------------------------------------------------------------

    // Create the software-based SHA engine.
    let sha = components::sha::ShaSoftware256Component::new()
        .finalize(components::sha_software_256_component_static!());

    // Create the credential checker.
    let checking_policy = components::appid::checker_sha::AppCheckerSha256Component::new(sha)
        .finalize(components::app_checker_sha256_component_static!());

    // Create the AppID assigner.
    let assigner = components::appid::assigner_name::AppIdAssignerNamesComponent::new()
        .finalize(components::appid_assigner_names_component_static!());

    // Create the process checking machine.
    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());

    //--------------------------------------------------------------------------
    // STORAGE PERMISSIONS
    //--------------------------------------------------------------------------

    // We use a custom storage permissions assigner that is based on the TBF
    // header if present, and otherwise defaults to allowing apps to access
    // their own state.

    #[derive(Clone)]
    pub struct AppStoreCapability;
    unsafe impl capabilities::ApplicationStorageCapability for AppStoreCapability {}

    let storage_permissions_policy = static_init!(
        invs_permissions::InvsStoragePermissions<
            nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
            kernel::process::ProcessStandardDebugFull,
            AppStoreCapability,
        >,
        invs_permissions::InvsStoragePermissions::new(AppStoreCapability)
    );

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

    // Create and start the asynchronous process loader.
    let _loader = components::loader::sequential::ProcessLoaderSequentialComponent::new(
        checker,
        board_kernel,
        chip,
        &nrf52840dk_test_base_lib::FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
        app_flash,
        app_memory,
        create_capability!(capabilities::ProcessManagementCapability),
    )
    .finalize(components::process_loader_sequential_component_static!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        kernel::process::ProcessStandardDebugFull,
        nrf52840dk_test_base_lib::NUM_PROCS
    ));

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let platform = Platform {
        base: base_platform,
        invs,
    };

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
