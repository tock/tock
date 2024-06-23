// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use core::ptr::addr_of_mut;

use kernel::debug;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::{capabilities, create_capability};
use nrf52840dk_lib::{self, PROCESSES};

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

struct Platform {
    base: nrf52840dk_lib::Platform,
    eui64_driver: &'static nrf52840dk_lib::Eui64Driver,
    ieee802154_driver: &'static nrf52840dk_lib::Ieee802154Driver,
    udp_driver: &'static capsules_extra::net::udp::UDPDriver<'static>,
    thread_driver: &'static capsules_extra::net::thread::driver::ThreadNetworkDriver<
        'static,
        VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
    >,
    i2c_master_slave: &'static capsules_core::i2c_master_slave_driver::I2CMasterSlaveDriver<
        'static,
        nrf52840::i2c::TWI<'static>,
    >,
    spi_controller: &'static capsules_core::spi_controller::Spi<
        'static,
        capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
            'static,
            nrf52840::spi::SPIM<'static>,
        >,
    >,
    dynamic_app_loader: &'static capsules_extra::app_loader::AppLoader<'static>,
    kv_driver: &'static KVDriver,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::eui64::DRIVER_NUM => f(Some(self.eui64_driver)),
            capsules_extra::net::udp::DRIVER_NUM => f(Some(self.udp_driver)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_core::i2c_master_slave_driver::DRIVER_NUM => f(Some(self.i2c_master_slave)),
            capsules_core::spi_controller::DRIVER_NUM => f(Some(self.spi_controller)),
            capsules_extra::app_loader::DRIVER_NUM => f(Some(self.dynamic_app_loader)),
            capsules_extra::net::thread::driver::DRIVER_NUM => f(Some(self.thread_driver)),
            capsules_extra::kv_driver::DRIVER_NUM => f(Some(self.kv_driver)),
            _ => f(None),
            capsules_extra::ieee802154::DRIVER_NUM => f(Some(self.ieee802154_driver)),
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
    let (board_kernel, base_platform, chip, default_peripherals, mux_alarm) =
        nrf52840dk_lib::start();

    //--------------------------------------------------------------------------
    // IEEE 802.15.4 and UDP
    //--------------------------------------------------------------------------

    let device_id = nrf52840::ficr::FICR_INSTANCE.id();
    let device_id_bottom_16: u16 = u16::from_le_bytes([device_id[0], device_id[1]]);
    let (ieee802154_radio, mux_mac) = components::ieee802154::Ieee802154Component::new(
        board_kernel,
        capsules_extra::ieee802154::DRIVER_NUM,
        &nrf52840_peripherals.ieee802154_radio,
        aes_mux,
        PAN_ID,
        device_id_bottom_16,
        device_id,
    )
    .finalize(components::ieee802154_component_static!(
        nrf52840::ieee802154_radio::Radio,
        nrf52840::aes::AesECB<'static>
    ));

    let local_ip_ifaces = static_init!(
        [IPAddr; 3],
        [
            IPAddr::generate_from_mac(capsules_extra::net::ieee802154::MacAddress::Long(device_id)),
            IPAddr([
                0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
                0x1e, 0x1f,
            ]),
            IPAddr::generate_from_mac(capsules_extra::net::ieee802154::MacAddress::Short(
                device_id_bottom_16
            )),
        ]
    );

    let (udp_send_mux, udp_recv_mux, udp_port_table) = components::udp_mux::UDPMuxComponent::new(
        mux_mac,
        DEFAULT_CTX_PREFIX_LEN,
        DEFAULT_CTX_PREFIX,
        DST_MAC_ADDR,
        MacAddress::Long(device_id),
        local_ip_ifaces,
        mux_alarm,
    )
    .finalize(components::udp_mux_component_static!(
        nrf52840::rtc::Rtc,
        Ieee802154MacDevice
    ));

    // UDP driver initialization happens here
    let udp_driver = components::udp_driver::UDPDriverComponent::new(
        board_kernel,
        capsules_extra::net::udp::driver::DRIVER_NUM,
        udp_send_mux,
        udp_recv_mux,
        udp_port_table,
        local_ip_ifaces,
    )
    .finalize(components::udp_driver_component_static!(nrf52840::rtc::Rtc));

    let thread_driver = components::thread_network::ThreadNetworkComponent::new(
        board_kernel,
        capsules_extra::net::thread::driver::DRIVER_NUM,
        udp_send_mux,
        udp_recv_mux,
        udp_port_table,
        aes_mux,
        device_id,
        mux_alarm,
    )
    .finalize(components::thread_network_component_static!(
        nrf52840::rtc::Rtc,
        nrf52840::aes::AesECB<'static>
    ));

    ieee802154_radio.set_key_procedure(thread_driver);
    ieee802154_radio.set_device_procedure(thread_driver);

    //--------------------------------------------------------------------------
    // TEMPERATURE (internal)
    //--------------------------------------------------------------------------

    let temp = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        &base_peripherals.temp,
    )
    .finalize(components::temperature_component_static!(
        nrf52840::temperature::Temp
    ));

    //--------------------------------------------------------------------------
    // RANDOM NUMBER GENERATOR
    //--------------------------------------------------------------------------

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &base_peripherals.trng,
    )
    .finalize(components::rng_component_static!(nrf52840::trng::Trng));

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
        &base_peripherals.adc,
        adc_channels,
        board_kernel,
        capsules_core::adc::DRIVER_NUM,
    )
    .finalize(components::adc_dedicated_component_static!(
        nrf52840::adc::Adc
    ));

    //--------------------------------------------------------------------------
    // SPI
    //--------------------------------------------------------------------------

    let mux_spi = components::spi::SpiMuxComponent::new(&base_peripherals.spim0)
        .finalize(components::spi_mux_component_static!(nrf52840::spi::SPIM));

    // Create the SPI system call capsule.
    let spi_controller = components::spi::SpiSyscallComponent::new(
        board_kernel,
        mux_spi,
        &gpio_port[SPI_CS],
        capsules_core::spi_controller::DRIVER_NUM,
    )
    .finalize(components::spi_syscall_component_static!(
        nrf52840::spi::SPIM
    ));

    base_peripherals.spim0.configure(
        nrf52840::pinmux::Pinmux::new(SPI_MOSI as u32),
        nrf52840::pinmux::Pinmux::new(SPI_MISO as u32),
        nrf52840::pinmux::Pinmux::new(SPI_CLK as u32),
    );

    //--------------------------------------------------------------------------
    // ONBOARD EXTERNAL FLASH
    //--------------------------------------------------------------------------

    let mx25r6435f = components::mx25r6435f::Mx25r6435fComponent::new(
        Some(&gpio_port[SPI_MX25R6435F_WRITE_PROTECT_PIN]),
        Some(&gpio_port[SPI_MX25R6435F_HOLD_PIN]),
        &gpio_port[SPI_MX25R6435F_CHIP_SELECT] as &dyn kernel::hil::gpio::Pin,
        mux_alarm,
        mux_spi,
    )
    .finalize(components::mx25r6435f_component_static!(
        nrf52840::spi::SPIM,
        nrf52840::gpio::GPIOPin,
        nrf52840::rtc::Rtc
    ));

    //--------------------------------------------------------------------------
    // TICKV
    //--------------------------------------------------------------------------

    // Static buffer to use when reading/writing flash for TicKV.
    let page_buffer = static_init!(
        <Mx25r6435f as kernel::hil::flash::Flash>::Page,
        <Mx25r6435f as kernel::hil::flash::Flash>::Page::default()
    );

    // SipHash for creating TicKV hashed keys.
    let sip_hash = components::siphash::Siphasher24Component::new()
        .finalize(components::siphasher24_component_static!());

    // TicKV with Tock wrapper/interface.
    let tickv = components::tickv::TicKVDedicatedFlashComponent::new(
        sip_hash,
        mx25r6435f,
        0, // start at the beginning of the flash chip
        (capsules_extra::mx25r6435f::SECTOR_SIZE as usize) * 32, // arbitrary size of 32 pages
        page_buffer,
    )
    .finalize(components::tickv_dedicated_flash_component_static!(
        Mx25r6435f,
        Siphasher24,
        TICKV_PAGE_SIZE,
    ));

    // KVSystem interface to KV (built on TicKV).
    let tickv_kv_store = components::kv::TicKVKVStoreComponent::new(tickv).finalize(
        components::tickv_kv_store_component_static!(
            TicKVDedicatedFlash,
            capsules_extra::tickv::TicKVKeyType,
        ),
    );

    let kv_store_permissions = components::kv::KVStorePermissionsComponent::new(tickv_kv_store)
        .finalize(components::kv_store_permissions_component_static!(
            TicKVKVStore
        ));

    // Share the KV stack with a mux.
    let mux_kv = components::kv::KVPermissionsMuxComponent::new(kv_store_permissions).finalize(
        components::kv_permissions_mux_component_static!(KVStorePermissions),
    );

    // Create a virtual component for the userspace driver.
    let virtual_kv_driver = components::kv::VirtualKVPermissionsComponent::new(mux_kv).finalize(
        components::virtual_kv_permissions_component_static!(KVStorePermissions),
    );

    // Userspace driver for KV.
    let kv_driver = components::kv::KVDriverComponent::new(
        virtual_kv_driver,
        board_kernel,
        capsules_extra::kv_driver::DRIVER_NUM,
    )
    .finalize(components::kv_driver_component_static!(
        VirtualKVPermissions
    ));

    //--------------------------------------------------------------------------
    // I2C CONTROLLER/TARGET
    //--------------------------------------------------------------------------

    let i2c_master_slave = components::i2c::I2CMasterSlaveDriverComponent::new(
        board_kernel,
        capsules_core::i2c_master_slave_driver::DRIVER_NUM,
        &base_peripherals.twi1,
    )
    .finalize(components::i2c_master_slave_component_static!(
        nrf52840::i2c::TWI
    ));

    base_peripherals.twi1.configure(
        nrf52840::pinmux::Pinmux::new(I2C_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(I2C_SDA_PIN as u32),
    );
    base_peripherals.twi1.set_speed(nrf52840::i2c::Speed::K400);

    //--------------------------------------------------------------------------
    // ANALOG COMPARATOR
    //--------------------------------------------------------------------------

    // Initialize AC using AIN5 (P0.29) as VIN+ and VIN- as AIN0 (P0.02)
    // These are hardcoded pin assignments specified in the driver
    let analog_comparator = components::analog_comparator::AnalogComparatorComponent::new(
        &base_peripherals.acomp,
        components::analog_comparator_component_helper!(
            nrf52840::acomp::Channel,
            &nrf52840::acomp::CHANNEL_AC0
        ),
        board_kernel,
        capsules_extra::analog_comparator::DRIVER_NUM,
    )
    .finalize(components::analog_comparator_component_static!(
        nrf52840::acomp::Comparator
    ));

    //--------------------------------------------------------------------------
    // Dynamic App Load (OTA)
    //-------------------------------------------------------------------------

    let dynamic_process_loader = components::dyn_process_loader::ProcessLoaderComponent::new(
        &mut PROCESSES,
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        &base_peripherals.nvmc,
        &FAULT_RESPONSE,
    )
    .finalize(components::process_loader_component_static!(
        nrf52840::nvmc::Nvmc,
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
    ));

    let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
        board_kernel,
        capsules_extra::app_loader::DRIVER_NUM,
        dynamic_process_loader,
    )
    .finalize(components::app_loader_component_static!());

    //--------------------------------------------------------------------------
    // NRF CLOCK SETUP
    //--------------------------------------------------------------------------

    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    //--------------------------------------------------------------------------
    // USB EXAMPLES
    //--------------------------------------------------------------------------
    // Uncomment to experiment with this.

    // // Create the strings we include in the USB descriptor.
    // let strings = static_init!(
    //     [&str; 3],
    //     [
    //         "Nordic Semiconductor", // Manufacturer
    //         "nRF52840dk - TockOS",  // Product
    //         "serial0001",           // Serial number
    //     ]
    // );

    // CTAP Example
    //
    // let (ctap, _ctap_driver) = components::ctap::CtapComponent::new(
    //     board_kernel,
    //     capsules_extra::ctap::DRIVER_NUM,
    //     &nrf52840_peripherals.usbd,
    //     0x1915, // Nordic Semiconductor
    //     0x503a, // lowRISC generic FS USB
    //     strings,
    // )
    // .finalize(components::ctap_component_static!(nrf52840::usbd::Usbd));

    // ctap.enable();
    // ctap.attach();

    // // Keyboard HID Example
    // type UsbHw = nrf52840::usbd::Usbd<'static>;
    // let usb_device = &nrf52840_peripherals.usbd;

    // let (keyboard_hid, keyboard_hid_driver) = components::keyboard_hid::KeyboardHidComponent::new(
    //     board_kernel,
    //     capsules_core::driver::NUM::KeyboardHid as usize,
    //     usb_device,
    //     0x1915, // Nordic Semiconductor
    //     0x503a,
    //     strings,
    // )
    // .finalize(components::keyboard_hid_component_static!(UsbHw));

    // keyboard_hid.enable();
    // keyboard_hid.attach();

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::round_robin_component_static!(NUM_PROCS));
    let (eui64_driver, ieee802154_driver, udp_driver) =
        nrf52840dk_lib::ieee802154_udp(board_kernel, default_peripherals, mux_alarm);

    let platform = Platform {
        base: base_platform,
        eui64_driver,
        ieee802154_driver,
        udp_driver,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        i2c_master_slave,
        spi_controller,
        dynamic_app_loader,
        kv_driver,
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
    };

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

    let remaining_memory = kernel::process::load_processes_return_memory(
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
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
        &mut *addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|(err, remaining_memory)| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
        remaining_memory
    });

    dynamic_process_loader.set_memory(remaining_memory);

    (board_kernel, platform, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
