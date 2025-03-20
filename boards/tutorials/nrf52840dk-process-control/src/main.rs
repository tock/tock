// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use kernel::component::Component;
// use kernel::debug;
use kernel::hil::led::LedLow;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::{capabilities, create_capability, static_init};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;
use nrf52840dk_lib::{self, PROCESSES};

// The nRF52840DK LEDs (see back of board)
const LED1_PIN: Pin = Pin::P0_13;
const LED2_PIN: Pin = Pin::P0_14;
const LED3_PIN: Pin = Pin::P0_15;
const LED4_PIN: Pin = Pin::P0_16;

type ScreenDriver = components::screen::ScreenComponentType;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>> = None;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

type Ieee802154RawDriver =
    components::ieee802154::Ieee802154RawComponentType<nrf52840::ieee802154_radio::Radio<'static>>;

/// Needed for process info capsule.
pub struct PMCapability;
unsafe impl capabilities::ProcessManagementCapability for PMCapability {}
unsafe impl capabilities::ProcessStartCapability for PMCapability {}

struct Platform {
    base: nrf52840dk_lib::Platform,
    ieee802154: &'static Ieee802154RawDriver,
    eui64: &'static nrf52840dk_lib::Eui64Driver,
    screen: &'static ScreenDriver,
    adc: &'static capsules_core::adc::AdcDedicated<'static, nrf52840::adc::Adc<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        kernel::hil::led::LedLow<'static, nrf52840::gpio::GPIOPin<'static>>,
        4,
    >,
    nonvolatile_storage:
        &'static capsules_extra::nonvolatile_storage_driver::NonvolatileStorage<'static>,
    process_info: &'static capsules_extra::process_info_driver::ProcessInfo<PMCapability>,
    processes: &'static [Option<&'static dyn kernel::process::Process>],
    dynamic_app_loader: &'static capsules_extra::app_loader::AppLoader<'static>,
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
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_extra::process_info_driver::DRIVER_NUM => f(Some(self.process_info)),
            capsules_extra::nonvolatile_storage_driver::DRIVER_NUM => {
                f(Some(self.nonvolatile_storage))
            }
            capsules_extra::app_loader::DRIVER_NUM => f(Some(self.dynamic_app_loader)),
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

impl kernel::process::ProcessLoadingAsyncClient for Platform {
    fn process_loaded(&self, _result: Result<(), kernel::process::ProcessLoadError>) {}

    fn process_loading_finished(&self) {
        kernel::debug!("Processes Loaded at Main:");

        for (i, proc) in self.processes.iter().enumerate() {
            proc.map(|p| {
                kernel::debug!("[{}] {}", i, p.get_process_name());
                kernel::debug!("    ShortId: {}", p.short_app_id());
            });
        }
    }
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let processes = &*addr_of!(PROCESSES);

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
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, nrf52840::gpio::GPIOPin>,
        LedLow::new(&nrf52840_peripherals.gpio_port[LED1_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED2_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED3_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED4_PIN]),
    ));

    //--------------------------------------------------------------------------
    // ADC
    //--------------------------------------------------------------------------

    let adc_channels = static_init!(
        [nrf52840::adc::AdcChannelSetup; 6],
        [
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput1),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput2),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput4),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput5),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput6),
            nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput7),
        ]
    );
    let adc = components::adc::AdcDedicatedComponent::new(
        &nrf52840_peripherals.nrf52.adc,
        adc_channels,
        board_kernel,
        capsules_core::adc::DRIVER_NUM,
    )
    .finalize(components::adc_dedicated_component_static!(
        nrf52840::adc::Adc
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
    // PROCESS INFO FOR USERSPACE
    //--------------------------------------------------------------------------

    let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let process_info = kernel::static_init!(
        capsules_extra::process_info_driver::ProcessInfo<PMCapability>,
        capsules_extra::process_info_driver::ProcessInfo::new(
            board_kernel,
            board_kernel.create_grant(capsules_extra::process_info_driver::DRIVER_NUM, &grant_cap),
            PMCapability
        )
    );

    CHIP = Some(chip);

    let mux_flash = components::flash::FlashMuxComponent::new(&nrf52840_peripherals.nrf52.nvmc)
        .finalize(components::flash_mux_component_static!(
            nrf52840::nvmc::Nvmc
        ));

    // Create a virtual flash user for dynamic binary storage
    let virtual_flash_dbs = components::flash::FlashUserComponent::new(mux_flash).finalize(
        components::flash_user_component_static!(nrf52840::nvmc::Nvmc),
    );

    // Create a virtual flash user for nonvolatile
    let virtual_flash_nvm = components::flash::FlashUserComponent::new(mux_flash).finalize(
        components::flash_user_component_static!(nrf52840::nvmc::Nvmc),
    );
    //--------------------------------------------------------------------------
    // Credential Checking
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
    // STORAGE PERMISSIONS
    //--------------------------------------------------------------------------

    let storage_permissions_policy =
        components::storage_permissions::null::StoragePermissionsNullComponent::new().finalize(
            components::storage_permissions_null_component_static!(
                nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
                kernel::process::ProcessStandardDebugFull,
            ),
        );

    // Create and start the asynchronous process loader.
    let loader = components::loader::sequential::ProcessLoaderSequentialComponent::new(
        checker,
        &mut *addr_of_mut!(PROCESSES),
        board_kernel,
        chip,
        &FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
    )
    .finalize(components::process_loader_sequential_component_static!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        kernel::process::ProcessStandardDebugFull,
        NUM_PROCS
    ));

    //--------------------------------------------------------------------------
    // Dynamic App Loading
    //--------------------------------------------------------------------------

    // Create the dynamic binary flasher.
    let dynamic_binary_storage =
        components::dynamic_binary_storage::SequentialBinaryStorageComponent::new(
            virtual_flash_dbs,
            loader,
        )
        .finalize(components::sequential_binary_storage_component_static!(
            capsules_core::virtualizers::virtual_flash::FlashUser<'static, nrf52840::nvmc::Nvmc>,
            nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
            kernel::process::ProcessStandardDebugFull,
        ));

    // Create the dynamic app loader capsule.
    let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
        board_kernel,
        capsules_extra::app_loader::DRIVER_NUM,
        dynamic_binary_storage,
        dynamic_binary_storage,
    )
    .finalize(components::app_loader_component_static!());

    //--------------------------------------------------------------------------
    // NONVOLATILE STORAGE
    //--------------------------------------------------------------------------

    // 32kB of userspace-accessible storage, page aligned:
    kernel::storage_volume!(APP_STORAGE, 32);

    let nonvolatile_storage = components::nonvolatile_storage::NonvolatileStorageComponent::new(
        board_kernel,
        capsules_extra::nonvolatile_storage_driver::DRIVER_NUM,
        virtual_flash_nvm,
        core::ptr::addr_of!(APP_STORAGE) as usize,
        APP_STORAGE.len(),
        // No kernel-writeable flash:
        core::ptr::null::<()>() as usize,
        0,
    )
    .finalize(components::nonvolatile_storage_component_static!(
        capsules_core::virtualizers::virtual_flash::FlashUser<'static, nrf52840::nvmc::Nvmc>
    ));

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let platform = Platform {
        base: base_platform,
        eui64,
        ieee802154,
        screen,
        adc,
        led,
        nonvolatile_storage,
        process_info,
        processes,
        dynamic_app_loader,
    };

    // let process_management_capability =
    //     create_capability!(capabilities::ProcessManagementCapability);

    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
