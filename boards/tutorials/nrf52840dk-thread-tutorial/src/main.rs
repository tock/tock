// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use kernel::component::Component;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::{capabilities, create_capability};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;
use nrf52840dk_lib::{self, NUM_PROCS, PROCESSES};

type ScreenDriver = components::screen::ScreenComponentType;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

type Ieee802154RawDriver =
    components::ieee802154::Ieee802154RawComponentType<nrf52840::ieee802154_radio::Radio<'static>>;

struct Platform {
    base: nrf52840dk_lib::Platform,
    ieee802154: &'static Ieee802154RawDriver,
    eui64: &'static nrf52840dk_lib::Eui64Driver,
    screen: &'static ScreenDriver,
    nonvolatile_storage:
        &'static capsules_extra::isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage<
            'static,
            {
                components::isolated_nonvolatile_storage::ISOLATED_NONVOLATILE_STORAGE_APP_REGION_SIZE_DEFAULT
            },
        >,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::eui64::DRIVER_NUM => f(Some(self.eui64)),
            capsules_extra::ieee802154::DRIVER_NUM => f(Some(self.ieee802154)),
            capsules_extra::screen::DRIVER_NUM => f(Some(self.screen)),
            capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM => {
                f(Some(self.nonvolatile_storage))
            }
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

type Chip = nrf52840dk_lib::Chip;

impl KernelResources<Chip> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <nrf52840dk_lib::Platform as KernelResources<Chip>>::SyscallFilter;
    type ProcessFault = <nrf52840dk_lib::Platform as KernelResources<Chip>>::ProcessFault;
    type Scheduler = <nrf52840dk_lib::Platform as KernelResources<Chip>>::Scheduler;
    type SchedulerTimer = <nrf52840dk_lib::Platform as KernelResources<Chip>>::SchedulerTimer;
    type WatchDog = <nrf52840dk_lib::Platform as KernelResources<Chip>>::WatchDog;
    type ContextSwitchCallback =
        <nrf52840dk_lib::Platform as KernelResources<Chip>>::ContextSwitchCallback;

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
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    // Create the base board:
    let (board_kernel, base_platform, chip, nrf52840_peripherals, _mux_alarm) =
        nrf52840dk_lib::start();

    //--------------------------------------------------------------------------
    // RAW 802.15.4
    //--------------------------------------------------------------------------

    let device_id = (*addr_of!(nrf52840::ficr::FICR_INSTANCE)).id();

    let eui64 = components::eui64::Eui64Component::new(u64::from_le_bytes(device_id))
        .finalize(components::eui64_component_static!());

    let ieee802154 = components::ieee802154::Ieee802154RawComponent::new(
        board_kernel,
        capsules_extra::ieee802154::DRIVER_NUM,
        &nrf52840_peripherals.ieee802154_radio,
    )
    .finalize(components::ieee802154_raw_component_static!(
        nrf52840::ieee802154_radio::Radio,
    ));

    //--------------------------------------------------------------------------
    // SCREEN
    //--------------------------------------------------------------------------

    const SCREEN_I2C_SDA_PIN: Pin = Pin::P1_10;
    const SCREEN_I2C_SCL_PIN: Pin = Pin::P1_11;

    let i2c_bus = components::i2c::I2CMuxComponent::new(&nrf52840_peripherals.nrf52.twi1, None)
        .finalize(components::i2c_mux_component_static!(nrf52840::i2c::TWI));
    nrf52840_peripherals.nrf52.twi1.configure(
        nrf52840::pinmux::Pinmux::new(SCREEN_I2C_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(SCREEN_I2C_SDA_PIN as u32),
    );
    nrf52840_peripherals
        .nrf52
        .twi1
        .set_speed(nrf52840::i2c::Speed::K400);

    // I2C address is b011110X, and on this board D/C̅ is GND.
    let ssd1306_sh1106_i2c = components::i2c::I2CComponent::new(i2c_bus, 0x3c)
        .finalize(components::i2c_component_static!(nrf52840::i2c::TWI));

    // Create the ssd1306 object for the actual screen driver.
    #[cfg(feature = "screen_ssd1306")]
    let ssd1306_sh1106 = components::ssd1306::Ssd1306Component::new(ssd1306_sh1106_i2c, true)
        .finalize(components::ssd1306_component_static!(nrf52840::i2c::TWI));

    #[cfg(feature = "screen_sh1106")]
    let ssd1306_sh1106 = components::sh1106::Sh1106Component::new(ssd1306_sh1106_i2c, true)
        .finalize(components::sh1106_component_static!(nrf52840::i2c::TWI));

    let screen = components::screen::ScreenComponent::new(
        board_kernel,
        capsules_extra::screen::DRIVER_NUM,
        ssd1306_sh1106,
        None,
    )
    .finalize(components::screen_component_static!(1032));

    ssd1306_sh1106.init_screen();

    //--------------------------------------------------------------------------
    // NONVOLATILE STORAGE
    //--------------------------------------------------------------------------

    // 32kB of userspace-accessible storage, page aligned:
    kernel::storage_volume!(APP_STORAGE, 32);

    let nonvolatile_storage = components::isolated_nonvolatile_storage::IsolatedNonvolatileStorageComponent::new(
        board_kernel,
        capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM,
        &nrf52840_peripherals.nrf52.nvmc,
        core::ptr::addr_of!(APP_STORAGE) as usize,
        APP_STORAGE.len()
    )
    .finalize(components::isolated_nonvolatile_storage_component_static!(
        nrf52840::nvmc::Nvmc,
        { components::isolated_nonvolatile_storage::ISOLATED_NONVOLATILE_STORAGE_APP_REGION_SIZE_DEFAULT }
    ));

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let platform = Platform {
        base: base_platform,
        eui64,
        ieee802154,
        screen,
        nonvolatile_storage,
    };

    //--------------------------------------------------------------------------
    // CREDENTIAL CHECKING
    //--------------------------------------------------------------------------

    // Create the credential checker.
    let checking_policy = components::appid::checker_null::AppCheckerNullComponent::new()
        .finalize(components::app_checker_null_component_static!());

    // Create the AppID assigner.
    let assigner = components::appid::assigner_name::AppIdAssignerNamesComponent::new()
        .finalize(components::appid_assigner_names_component_static!());

    // Create the process checking machine.
    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());

    //--------------------------------------------------------------------------
    // STORAGE PERMISSIONS
    //--------------------------------------------------------------------------

    let storage_permissions_policy =
        components::storage_permissions::individual::StoragePermissionsIndividualComponent::new()
            .finalize(
                components::storage_permissions_individual_component_static!(
                    nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
                    kernel::process::ProcessStandardDebugFull,
                ),
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
        &mut *addr_of_mut!(PROCESSES),
        board_kernel,
        chip,
        &FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
        app_flash,
        app_memory,
    )
    .finalize(components::process_loader_sequential_component_static!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        kernel::process::ProcessStandardDebugFull,
        NUM_PROCS
    ));

    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
