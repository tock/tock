// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::component::Component;
use kernel::hil::usb::Client;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::process::ProcessLoadingAsync;
use kernel::static_init;
use kernel::{capabilities, create_capability};
use nrf52840::gpio::Pin;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Screen
type ScreenDriver = components::screen::ScreenComponentType;

// USB Keyboard HID - for nRF52840dk
type UsbHw = nrf52840::usbd::Usbd<'static>; // For any nRF52840 board.
type KeyboardHidDriver = components::keyboard_hid::KeyboardHidComponentType<UsbHw>;

// HMAC
type HmacSha256Software = components::hmac::HmacSha256SoftwareComponentType<
    capsules_extra::sha256::Sha256Software<'static>,
>;
type HmacDriver = components::hmac::HmacComponentType<HmacSha256Software, 32>;

struct Platform {
    keyboard_hid_driver: &'static KeyboardHidDriver,
    hmac: &'static HmacDriver,
    screen: &'static ScreenDriver,
    base: nrf52840dk_lib::Platform,
}

const KEYBOARD_HID_DRIVER_NUM: usize = capsules_core::driver::NUM::KeyboardHid as usize;

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::hmac::DRIVER_NUM => f(Some(self.hmac)),
            KEYBOARD_HID_DRIVER_NUM => f(Some(self.keyboard_hid_driver)),
            capsules_extra::screen::screen::DRIVER_NUM => f(Some(self.screen)),
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

// Called by the process loader when the board boots.
impl kernel::process::ProcessLoadingAsyncClient for Platform {
    fn process_loaded(&self, _result: Result<(), kernel::process::ProcessLoadError>) {}

    fn process_loading_finished(&self) {}
}

type ChipHw = nrf52840dk_lib::ChipHw;

impl KernelResources<ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::SyscallFilter;
    type ProcessFault = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::ProcessFault;
    type Scheduler = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::Scheduler;
    type SchedulerTimer = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::SchedulerTimer;
    type WatchDog = <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::WatchDog;
    type ContextSwitchCallback =
        <nrf52840dk_lib::Platform as KernelResources<ChipHw>>::ContextSwitchCallback;

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
    // HMAC-SHA256
    //--------------------------------------------------------------------------

    let sha256_sw = components::sha::ShaSoftware256Component::new()
        .finalize(components::sha_software_256_component_static!());

    let hmac_sha256_sw = components::hmac::HmacSha256SoftwareComponent::new(sha256_sw).finalize(
        components::hmac_sha256_software_component_static!(capsules_extra::sha256::Sha256Software),
    );

    let hmac = components::hmac::HmacComponent::new(
        board_kernel,
        capsules_extra::hmac::DRIVER_NUM,
        hmac_sha256_sw,
    )
    .finalize(components::hmac_component_static!(HmacSha256Software, 32));

    //--------------------------------------------------------------------------
    // CREDENTIALS CHECKING POLICY
    //--------------------------------------------------------------------------

    // Create the credential checker.
    let checking_policy = components::appid::checker_null::AppCheckerNullComponent::new()
        .finalize(components::app_checker_null_component_static!());

    // Create the AppID assigner.
    let assigner = components::appid::assigner_tbf::AppIdAssignerTbfHeaderComponent::new()
        .finalize(components::appid_assigner_tbf_header_component_static!());

    // Create the process checking machine.
    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());

    //--------------------------------------------------------------------------
    // SCREEN
    //--------------------------------------------------------------------------

    const SCREEN_I2C_SDA_PIN: Pin = Pin::P1_10;
    const SCREEN_I2C_SCL_PIN: Pin = Pin::P1_11;

    let i2c_bus = components::i2c::I2CMuxComponent::new(&nrf52840_peripherals.nrf52.twi1, None)
        .finalize(components::i2c_mux_component_static!(nrf52840::i2c::TWI));
    nrf52840_peripherals.nrf52.twi1.configure(
        nrf52840::pinmux::Pinmux::new(SCREEN_I2C_SCL_PIN),
        nrf52840::pinmux::Pinmux::new(SCREEN_I2C_SDA_PIN),
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
        capsules_extra::screen::screen::DRIVER_NUM,
        ssd1306_sh1106,
        None,
    )
    .finalize(components::screen_component_static!(1032));

    ssd1306_sh1106.init_screen();

    //--------------------------------------------------------------------------
    // KEYBOARD
    //--------------------------------------------------------------------------

    // Create the strings we include in the USB descriptor.
    let strings = static_init!(
        [&str; 3],
        [
            "Nordic Semiconductor", // Manufacturer
            "nRF52840dk - TockOS",  // Product
            "serial0001",           // Serial number
        ]
    );

    let usb_device = &nrf52840_peripherals.usbd;

    // Generic HID Keyboard component usage
    let (keyboard_hid, keyboard_hid_driver) = components::keyboard_hid::KeyboardHidComponent::new(
        board_kernel,
        capsules_core::driver::NUM::KeyboardHid as usize,
        usb_device,
        0x1915, // Nordic Semiconductor
        0x503a,
        strings,
    )
    .finalize(components::keyboard_hid_component_static!(UsbHw));

    keyboard_hid.enable();
    keyboard_hid.attach();

    //--------------------------------------------------------------------------
    // STORAGE PERMISSIONS
    //--------------------------------------------------------------------------

    let storage_permissions_policy =
        components::storage_permissions::tbf_header::StoragePermissionsTbfHeaderComponent::new()
            .finalize(
                components::storage_permissions_tbf_header_component_static!(
                    nrf52840dk_lib::ChipHw,
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
    let loader = components::loader::sequential::ProcessLoaderSequentialComponent::new(
        checker,
        board_kernel,
        chip,
        &FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
        app_flash,
        app_memory,
    )
    .finalize(components::process_loader_sequential_component_static!(
        nrf52840dk_lib::ChipHw,
        kernel::process::ProcessStandardDebugFull,
        nrf52840dk_lib::NUM_PROCS
    ));

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let platform = static_init!(
        Platform,
        Platform {
            base: base_platform,
            keyboard_hid_driver,
            hmac,
            screen,
        }
    );

    loader.set_client(platform);

    board_kernel.kernel_loop(
        platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
