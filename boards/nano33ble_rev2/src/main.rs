// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Tock kernel for the Arduino Nano 33 BLE Sense Rev2.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE + IEEE 802.15.4 transceiver).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::addr_of;
use core::ptr::addr_of_mut;

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::Output;
use kernel::hil::led::LedLow;
use kernel::hil::time::Counter;
use kernel::hil::usb::Client;
use kernel::platform::chip::Chip;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

// Three-color LED.
const LED_RED_PIN: Pin = Pin::P0_24;
const LED_GREEN_PIN: Pin = Pin::P0_16;
const LED_BLUE_PIN: Pin = Pin::P0_06;

const LED_KERNEL_PIN: Pin = Pin::P0_13;

const _BUTTON_RST_PIN: Pin = Pin::P0_18;

const GPIO_D2: Pin = Pin::P1_11;
const GPIO_D3: Pin = Pin::P1_12;
const GPIO_D4: Pin = Pin::P1_15;
const GPIO_D5: Pin = Pin::P1_13;
const GPIO_D6: Pin = Pin::P1_14;
const GPIO_D7: Pin = Pin::P0_23;
const GPIO_D8: Pin = Pin::P0_21;
const GPIO_D9: Pin = Pin::P0_27;
const GPIO_D10: Pin = Pin::P1_02;

const _UART_TX_PIN: Pin = Pin::P1_03;
const _UART_RX_PIN: Pin = Pin::P1_10;

/// I2C pins for all of the sensors.
const I2C_SDA_PIN: Pin = Pin::P0_14;
const I2C_SCL_PIN: Pin = Pin::P0_15;

/// GPIO tied to the VCC of the I2C pullup resistors.
const I2C_PULLUP_PIN: Pin = Pin::P1_00;

/// Interrupt pin for the APDS9960 sensor.
const APDS9960_PIN: Pin = Pin::P0_19;

// Constants related to the configuration of the 15.4 network stack
/// Personal Area Network ID for the IEEE 802.15.4 radio
const PAN_ID: u16 = 0xABCD;
/// Gateway (or next hop) MAC Address
const DST_MAC_ADDR: capsules_extra::net::ieee802154::MacAddress =
    capsules_extra::net::ieee802154::MacAddress::Short(49138);
const DEFAULT_CTX_PREFIX_LEN: u8 = 8; //Length of context for 6LoWPAN compression
const DEFAULT_CTX_PREFIX: [u8; 16] = [0x0_u8; 16]; //Context for 6LoWPAN Compression

/// UART Writer for panic!()s.
pub mod io;

// How should the kernel respond when a process faults. For this board we choose
// to stop the app and print a notice, but not immediately panic. This allows
// users to debug their apps, but avoids issues with using the USB/CDC stack
// synchronously for panic! too early after the board boots.
const FAULT_RESPONSE: capsules_system::process_policies::StopWithDebugFaultPolicy =
    capsules_system::process_policies::StopWithDebugFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

// State for loading and holding applications.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>> = None;
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;
static mut CDC_REF_FOR_PANIC: Option<
    &'static capsules_extra::usb::cdc::CdcAcm<
        'static,
        nrf52::usbd::Usbd,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
    >,
> = None;
static mut NRF52_POWER: Option<&'static nrf52840::power::Power> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

// Function for the CDC/USB stack to use to enter the bootloader.
fn baud_rate_reset_bootloader_enter() {
    unsafe {
        // 0x90 is the magic value the bootloader expects
        NRF52_POWER.unwrap().set_gpregret(0x90);
        cortexm4::scb::reset();
    }
}

type HS3003Sensor = components::hs3003::Hs3003ComponentType<
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, nrf52840::i2c::TWI<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<HS3003Sensor>;
type HumidityDriver = components::humidity::HumidityComponentType<HS3003Sensor>;
type Ieee802154MacDevice = components::ieee802154::Ieee802154ComponentMacDeviceType<
    nrf52840::ieee802154_radio::Radio<'static>,
    nrf52840::aes::AesECB<'static>,
>;
type Ieee802154Driver = components::ieee802154::Ieee802154ComponentType<
    nrf52840::ieee802154_radio::Radio<'static>,
    nrf52840::aes::AesECB<'static>,
>;
type RngDriver = components::rng::RngComponentType<nrf52840::trng::Trng<'static>>;

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules_extra::ble_advertising_driver::BLE<
        'static,
        nrf52::ble_radio::Radio<'static>,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            nrf52::rtc::Rtc<'static>,
        >,
    >,
    ieee802154_radio: &'static Ieee802154Driver,
    console: &'static capsules_core::console::Console<'static>,
    pconsole: &'static capsules_core::process_console::ProcessConsole<
        'static,
        { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN },
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            nrf52::rtc::Rtc<'static>,
        >,
        components::process_console::Capability,
    >,
    proximity: &'static capsules_extra::proximity::ProximitySensor<'static>,
    pressure: &'static capsules_extra::pressure::PressureSensor<
        'static,
        capsules_extra::lps22hb::Lps22hb<
            'static,
            capsules_core::virtualizers::virtual_i2c::I2CDevice<
                'static,
                nrf52840::i2c::TWI<'static>,
            >,
        >,
    >,
    temperature: &'static TemperatureDriver,
    humidity: &'static HumidityDriver,
    gpio: &'static capsules_core::gpio::GPIO<'static, nrf52::gpio::GPIOPin<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedLow<'static, nrf52::gpio::GPIOPin<'static>>,
        3,
    >,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    rng: &'static RngDriver,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            nrf52::rtc::Rtc<'static>,
        >,
    >,
    udp_driver: &'static capsules_extra::net::udp::UDPDriver<'static>,
    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_extra::proximity::DRIVER_NUM => f(Some(self.proximity)),
            capsules_extra::pressure::DRIVER_NUM => f(Some(self.pressure)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_extra::humidity::DRIVER_NUM => f(Some(self.humidity)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_extra::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules_extra::ieee802154::DRIVER_NUM => f(Some(self.ieee802154_radio)),
            capsules_extra::net::udp::DRIVER_NUM => f(Some(self.udp_driver)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<nrf52::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>>
    for Platform
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = RoundRobinSched<'static>;
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
        self.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.systick
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    Platform,
    &'static nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>,
) {
    nrf52840::init();

    let ieee802154_ack_buf = static_init!(
        [u8; nrf52840::ieee802154_radio::ACK_BUF_SIZE],
        [0; nrf52840::ieee802154_radio::ACK_BUF_SIZE]
    );

    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new(ieee802154_ack_buf)
    );

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52;

    // Save a reference to the power module for resetting the board into the
    // bootloader.
    NRF52_POWER = Some(&base_peripherals.pwr_clk);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    //--------------------------------------------------------------------------
    // CAPABILITIES
    //--------------------------------------------------------------------------

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    //--------------------------------------------------------------------------
    // DEBUG GPIO
    //--------------------------------------------------------------------------

    // Configure kernel debug GPIOs as early as possible. These are used by the
    // `debug_gpio!(0, toggle)` macro. We configure these early so that the
    // macro is available during most of the setup code and kernel execution.
    kernel::debug::assign_gpios(
        Some(&nrf52840_peripherals.gpio_port[LED_KERNEL_PIN]),
        None,
        None,
    );

    //--------------------------------------------------------------------------
    // GPIO
    //--------------------------------------------------------------------------

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            2 => &nrf52840_peripherals.gpio_port[GPIO_D2],
            3 => &nrf52840_peripherals.gpio_port[GPIO_D3],
            4 => &nrf52840_peripherals.gpio_port[GPIO_D4],
            5 => &nrf52840_peripherals.gpio_port[GPIO_D5],
            6 => &nrf52840_peripherals.gpio_port[GPIO_D6],
            7 => &nrf52840_peripherals.gpio_port[GPIO_D7],
            8 => &nrf52840_peripherals.gpio_port[GPIO_D8],
            9 => &nrf52840_peripherals.gpio_port[GPIO_D9],
            10 => &nrf52840_peripherals.gpio_port[GPIO_D10]
        ),
    )
    .finalize(components::gpio_component_static!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, nrf52840::gpio::GPIOPin>,
        LedLow::new(&nrf52840_peripherals.gpio_port[LED_RED_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED_GREEN_PIN]),
        LedLow::new(&nrf52840_peripherals.gpio_port[LED_BLUE_PIN]),
    ));

    //--------------------------------------------------------------------------
    // ALARM & TIMER
    //--------------------------------------------------------------------------

    let rtc = &base_peripherals.rtc;
    let _ = rtc.start();

    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_static!(nrf52::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(nrf52::rtc::Rtc));

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    // Setup the CDC-ACM over USB driver that we will use for UART.
    // We use the Arduino Vendor ID and Product ID since the device is the same.

    // Create the strings we include in the USB descriptor. We use the hardcoded
    // DEVICEADDR register on the nRF52 to set the serial number.
    let serial_number_buf = static_init!([u8; 17], [0; 17]);
    let serial_number_string: &'static str =
        (*addr_of!(nrf52::ficr::FICR_INSTANCE)).address_str(serial_number_buf);
    let strings = static_init!(
        [&str; 3],
        [
            "Arduino",                         // Manufacturer
            "Nano 33 BLE Sense Rev2 - TockOS", // Product
            serial_number_string,              // Serial number
        ]
    );

    let cdc = components::cdc::CdcAcmComponent::new(
        &nrf52840_peripherals.usbd,
        capsules_extra::usb::cdc::MAX_CTRL_PACKET_SIZE_NRF52840,
        0x2341,
        0x005a,
        strings,
        mux_alarm,
        Some(&baud_rate_reset_bootloader_enter),
    )
    .finalize(components::cdc_acm_component_static!(
        nrf52::usbd::Usbd,
        nrf52::rtc::Rtc
    ));
    CDC_REF_FOR_PANIC = Some(cdc); //for use by panic handler

    // Process Printer for displaying process information.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(cdc, 115200)
        .finalize(components::uart_mux_component_static!());

    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm4::support::reset),
    )
    .finalize(components::process_console_component_static!(
        nrf52::rtc::Rtc<'static>
    ));

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    //--------------------------------------------------------------------------
    // RANDOM NUMBERS
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
    base_peripherals.adc.calibrate();

    let adc_mux = components::adc::AdcMuxComponent::new(&base_peripherals.adc)
        .finalize(components::adc_mux_component_static!(nrf52840::adc::Adc));

    let adc_syscall =
        components::adc::AdcVirtualComponent::new(board_kernel, capsules_core::adc::DRIVER_NUM)
            .finalize(components::adc_syscall_component_helper!(
                // A0
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput2)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A1
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput3)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A2
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput6)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A3
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput5)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A4
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput7)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A5
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput0)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A6
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput4)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A7
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput1)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
            ));

    //--------------------------------------------------------------------------
    // SENSORS
    //--------------------------------------------------------------------------

    let sensors_i2c_bus = components::i2c::I2CMuxComponent::new(&base_peripherals.twi1, None)
        .finalize(components::i2c_mux_component_static!(nrf52840::i2c::TWI));
    base_peripherals.twi1.configure(
        nrf52840::pinmux::Pinmux::new(I2C_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(I2C_SDA_PIN as u32),
    );

    let _ = &nrf52840_peripherals.gpio_port[I2C_PULLUP_PIN].make_output();
    nrf52840_peripherals.gpio_port[I2C_PULLUP_PIN].set();

    let apds9960 = components::apds9960::Apds9960Component::new(
        sensors_i2c_bus,
        0x39,
        &nrf52840_peripherals.gpio_port[APDS9960_PIN],
    )
    .finalize(components::apds9960_component_static!(nrf52840::i2c::TWI));
    let proximity = components::proximity::ProximityComponent::new(
        apds9960,
        board_kernel,
        capsules_extra::proximity::DRIVER_NUM,
    )
    .finalize(components::proximity_component_static!());

    let lps22hb = components::lps22hb::Lps22hbComponent::new(sensors_i2c_bus, 0x5C)
        .finalize(components::lps22hb_component_static!(nrf52840::i2c::TWI));
    let pressure = components::pressure::PressureComponent::new(
        board_kernel,
        capsules_extra::pressure::DRIVER_NUM,
        lps22hb,
    )
    .finalize(components::pressure_component_static!(
        capsules_extra::lps22hb::Lps22hb<
            'static,
            capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, nrf52840::i2c::TWI>,
        >
    ));

    let hs3003 = components::hs3003::Hs3003Component::new(sensors_i2c_bus, 0x44)
        .finalize(components::hs3003_component_static!(nrf52840::i2c::TWI));
    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        hs3003,
    )
    .finalize(components::temperature_component_static!(HS3003Sensor));
    let humidity = components::humidity::HumidityComponent::new(
        board_kernel,
        capsules_extra::humidity::DRIVER_NUM,
        hs3003,
    )
    .finalize(components::humidity_component_static!(HS3003Sensor));

    //--------------------------------------------------------------------------
    // WIRELESS
    //--------------------------------------------------------------------------

    let ble_radio = components::ble::BLEComponent::new(
        board_kernel,
        capsules_extra::ble_advertising_driver::DRIVER_NUM,
        &base_peripherals.ble_radio,
        mux_alarm,
    )
    .finalize(components::ble_component_static!(
        nrf52840::rtc::Rtc,
        nrf52840::ble_radio::Radio
    ));

    use capsules_extra::net::ieee802154::MacAddress;

    let aes_mux = components::ieee802154::MuxAes128ccmComponent::new(&base_peripherals.ecb)
        .finalize(components::mux_aes128ccm_component_static!(
            nrf52840::aes::AesECB
        ));

    let device_id = (*addr_of!(nrf52840::ficr::FICR_INSTANCE)).id();
    let device_id_bottom_16 = u16::from_le_bytes([device_id[0], device_id[1]]);
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
    use capsules_extra::net::ipv6::ip_utils::IPAddr;

    let local_ip_ifaces = static_init!(
        [IPAddr; 3],
        [
            IPAddr([
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f,
            ]),
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
        MacAddress::Short(device_id_bottom_16),
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
        capsules_extra::net::udp::DRIVER_NUM,
        udp_send_mux,
        udp_recv_mux,
        udp_port_table,
        local_ip_ifaces,
    )
    .finalize(components::udp_driver_component_static!(nrf52840::rtc::Rtc));

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52_components::NrfClockComponent::new(&base_peripherals.clock).finalize(());

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        ble_radio,
        ieee802154_radio,
        console,
        pconsole,
        proximity,
        pressure,
        temperature,
        humidity,
        adc: adc_syscall,
        led,
        gpio,
        rng,
        alarm,
        udp_driver,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
    };

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );
    CHIP = Some(chip);

    // Need to disable the MPU because the bootloader seems to set it up.
    chip.mpu().clear_mpu();

    // Configure the USB stack to enable a serial port over CDC-ACM.
    cdc.enable();
    cdc.attach();

    //--------------------------------------------------------------------------
    // TESTS
    //--------------------------------------------------------------------------
    // test::linear_log_test::run(
    //     mux_alarm,
    //     &nrf52840_peripherals.nrf52.nvmc,
    // );
    // test::log_test::run(
    //     mux_alarm,
    //     &nrf52840_peripherals.nrf52.nvmc,
    // );

    debug!("Initialization complete. Entering main loop.");
    let _ = platform.pconsole.start();

    //--------------------------------------------------------------------------
    // PROCESSES AND MAIN LOOP
    //--------------------------------------------------------------------------

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
        &mut *addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, platform, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
