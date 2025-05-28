// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Adafruit CLUE nRF52480 Express.
//!
//! It is based on nRF52840 Express SoC (Cortex M4 core with a BLE + IEEE 802.15.4 transceiver).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::addr_of;
use core::ptr::addr_of_mut;

use capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM;

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::buzzer::Buzzer;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedHigh;
use kernel::hil::symmetric_encryption::AES128;
use kernel::hil::time::Alarm;
use kernel::hil::time::Counter;
use kernel::hil::usb::Client;
use kernel::platform::chip::Chip;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

// LEDs.
const LED_RED_PIN: Pin = Pin::P1_01;
const LED_WHITE_PIN: Pin = Pin::P0_10;

const LED_KERNEL_PIN: Pin = Pin::P1_01;

// Speaker
const SPEAKER_PIN: Pin = Pin::P1_00;

// Buttons
const BUTTON_LEFT: Pin = Pin::P1_02;
const BUTTON_RIGHT: Pin = Pin::P1_10;

#[allow(dead_code)]
const GPIO_D0: Pin = Pin::P0_04;
#[allow(dead_code)]
const GPIO_D1: Pin = Pin::P0_05;
#[allow(dead_code)]
const GPIO_D2: Pin = Pin::P0_03;
#[allow(dead_code)]
const GPIO_D3: Pin = Pin::P0_28;
#[allow(dead_code)]
const GPIO_D4: Pin = Pin::P0_02;

const GPIO_D6: Pin = Pin::P1_09;
const GPIO_D7: Pin = Pin::P0_07;
const GPIO_D8: Pin = Pin::P1_07;
const GPIO_D9: Pin = Pin::P0_27;

#[allow(dead_code)]
const GPIO_D10: Pin = Pin::P0_30;
#[allow(dead_code)]
const GPIO_D12: Pin = Pin::P0_31;

const GPIO_D13: Pin = Pin::P0_08;
const GPIO_D14: Pin = Pin::P0_06;
const GPIO_D15: Pin = Pin::P0_26;
const GPIO_D16: Pin = Pin::P0_29;

const _UART_TX_PIN: Pin = Pin::P0_05;
const _UART_RX_PIN: Pin = Pin::P0_04;

/// I2C pins for all of the sensors.
const I2C_SDA_PIN: Pin = Pin::P0_24;
const I2C_SCL_PIN: Pin = Pin::P0_25;

/// Interrupt pin for the APDS9960 sensor.
const APDS9960_PIN: Pin = Pin::P0_09;

/// Personal Area Network ID for the IEEE 802.15.4 radio
const PAN_ID: u16 = 0xABCD;

/// TFT ST7789H2
const ST7789H2_SCK: Pin = Pin::P0_14;
const ST7789H2_MOSI: Pin = Pin::P0_15;
const ST7789H2_MISO: Pin = Pin::P0_26; // ST7789H2 has no MISO Pin, but SPI requires a MISO Pin
const ST7789H2_CS: Pin = Pin::P0_12;
const ST7789H2_DC: Pin = Pin::P0_13;
const ST7789H2_RESET: Pin = Pin::P1_03;

/// TFT backlight
const _ST7789H2_LITE: Pin = Pin::P1_05;

/// UART Writer for panic!()s.
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::StopWithDebugFaultPolicy =
    capsules_system::process_policies::StopWithDebugFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

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

// Function for the CDC/USB stack to use to enter the Adafruit nRF52 Bootloader
fn baud_rate_reset_bootloader_enter() {
    unsafe {
        // 0x4e is the magic value the Adafruit nRF52 Bootloader expects
        // as defined by https://github.com/adafruit/Adafruit_nRF52_Bootloader/blob/master/src/main.c
        NRF52_POWER.unwrap().set_gpregret(0x90);
        // uncomment to use with Adafruit nRF52 Bootloader
        // NRF52_POWER.unwrap().set_gpregret(0x4e);
        cortexm4::scb::reset();
    }
}

type SHT3xSensor = components::sht3x::SHT3xComponentType<
    capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, nrf52840::i2c::TWI<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<SHT3xSensor>;
type HumidityDriver = components::humidity::HumidityComponentType<SHT3xSensor>;
type RngDriver = components::rng::RngComponentType<nrf52840::trng::Trng<'static>>;

type Ieee802154Driver = components::ieee802154::Ieee802154ComponentType<
    nrf52840::ieee802154_radio::Radio<'static>,
    nrf52840::aes::AesECB<'static>,
>;

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
    proximity: &'static capsules_extra::proximity::ProximitySensor<'static>,
    gpio: &'static capsules_core::gpio::GPIO<'static, nrf52::gpio::GPIOPin<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedHigh<'static, nrf52::gpio::GPIOPin<'static>>,
        2,
    >,
    button: &'static capsules_core::button::Button<'static, nrf52::gpio::GPIOPin<'static>>,
    screen: &'static capsules_extra::screen::Screen<'static>,
    rng: &'static RngDriver,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            nrf52::rtc::Rtc<'static>,
        >,
    >,
    buzzer: &'static capsules_extra::buzzer_driver::Buzzer<
        'static,
        capsules_extra::buzzer_pwm::PwmBuzzer<
            'static,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                nrf52840::rtc::Rtc<'static>,
            >,
            capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52840::pwm::Pwm>,
        >,
    >,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    temperature: &'static TemperatureDriver,
    humidity: &'static HumidityDriver,
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
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_extra::screen::DRIVER_NUM => f(Some(self.screen)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_extra::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules_extra::ieee802154::DRIVER_NUM => f(Some(self.ieee802154_radio)),
            capsules_extra::buzzer_driver::DRIVER_NUM => f(Some(self.buzzer)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_extra::humidity::DRIVER_NUM => f(Some(self.humidity)),
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
unsafe fn start() -> (
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
            // uncomment the following to use pins D0, D1, D2, D3 and D4 as gpio
            // instead of A2, A3, A4, A5 and A6
            // 0 => &nrf52840_peripherals.gpio_port[GPIO_D0],
            // 1 => &nrf52840_peripherals.gpio_port[GPIO_D1],
            // 2 => &nrf52840_peripherals.gpio_port[GPIO_D2],
            // 3 => &nrf52840_peripherals.gpio_port[GPIO_D3],
            // 4 => &nrf52840_peripherals.gpio_port[GPIO_D4],

            6 => &nrf52840_peripherals.gpio_port[GPIO_D6],
            7 => &nrf52840_peripherals.gpio_port[GPIO_D7],
            8 => &nrf52840_peripherals.gpio_port[GPIO_D8],
            9 => &nrf52840_peripherals.gpio_port[GPIO_D9],

            // uncomment the following to use pins D10 as gpio instead of A7
            // 10 => &nrf52840_peripherals.gpio_port[GPIO_D10],

            // uncomment the following to use pins D12 as gpio instead of A0
            // 12 => &nrf52840_peripherals.gpio_port[GPIO_D12],

            13 => &nrf52840_peripherals.gpio_port[GPIO_D13],
            14 => &nrf52840_peripherals.gpio_port[GPIO_D14],
            15 => &nrf52840_peripherals.gpio_port[GPIO_D15],
            16 => &nrf52840_peripherals.gpio_port[GPIO_D16]
        ),
    )
    .finalize(components::gpio_component_static!(nrf52840::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // LEDs
    //--------------------------------------------------------------------------

    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, nrf52840::gpio::GPIOPin>,
        LedHigh::new(&nrf52840_peripherals.gpio_port[LED_RED_PIN]),
        LedHigh::new(&nrf52840_peripherals.gpio_port[LED_WHITE_PIN])
    ));

    //--------------------------------------------------------------------------
    // Buttons
    //--------------------------------------------------------------------------
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            nrf52840::gpio::GPIOPin,
            (
                &nrf52840_peripherals.gpio_port[BUTTON_LEFT],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), // Left
            (
                &nrf52840_peripherals.gpio_port[BUTTON_RIGHT],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ) // Right
        ),
    )
    .finalize(components::button_component_static!(
        nrf52840::gpio::GPIOPin
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
    // PWM & BUZZER
    //--------------------------------------------------------------------------

    let mux_pwm = static_init!(
        capsules_core::virtualizers::virtual_pwm::MuxPwm<'static, nrf52840::pwm::Pwm>,
        capsules_core::virtualizers::virtual_pwm::MuxPwm::new(&base_peripherals.pwm0)
    );
    let virtual_pwm_buzzer = static_init!(
        capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52840::pwm::Pwm>,
        capsules_core::virtualizers::virtual_pwm::PwmPinUser::new(
            mux_pwm,
            nrf52840::pinmux::Pinmux::new(SPEAKER_PIN as u32)
        )
    );
    virtual_pwm_buzzer.add_to_mux();

    let virtual_alarm_buzzer = static_init!(
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_buzzer.setup();

    let pwm_buzzer = static_init!(
        capsules_extra::buzzer_pwm::PwmBuzzer<
            'static,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                nrf52840::rtc::Rtc,
            >,
            capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52840::pwm::Pwm>,
        >,
        capsules_extra::buzzer_pwm::PwmBuzzer::new(
            virtual_pwm_buzzer,
            virtual_alarm_buzzer,
            capsules_extra::buzzer_pwm::DEFAULT_MAX_BUZZ_TIME_MS,
        )
    );

    let buzzer = static_init!(
        capsules_extra::buzzer_driver::Buzzer<
            'static,
            capsules_extra::buzzer_pwm::PwmBuzzer<
                'static,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                    'static,
                    nrf52840::rtc::Rtc,
                >,
                capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52840::pwm::Pwm>,
            >,
        >,
        capsules_extra::buzzer_driver::Buzzer::new(
            pwm_buzzer,
            capsules_extra::buzzer_driver::DEFAULT_MAX_BUZZ_TIME_MS,
            board_kernel.create_grant(
                capsules_extra::buzzer_driver::DRIVER_NUM,
                &memory_allocation_capability
            )
        )
    );

    pwm_buzzer.set_client(buzzer);

    virtual_alarm_buzzer.set_alarm_client(pwm_buzzer);

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    // Setup the CDC-ACM over USB driver that we will use for UART.
    // We use the Adafruit Vendor ID and Product ID since the device is the same.

    // Create the strings we include in the USB descriptor. We use the hardcoded
    // DEVICEADDR register on the nRF52 to set the serial number.
    let serial_number_buf = static_init!([u8; 17], [0; 17]);
    let serial_number_string: &'static str =
        (*addr_of!(nrf52::ficr::FICR_INSTANCE)).address_str(serial_number_buf);
    let strings = static_init!(
        [&str; 3],
        [
            "Adafruit",               // Manufacturer
            "CLUE nRF52840 - TockOS", // Product
            serial_number_string,     // Serial number
        ]
    );

    let cdc = components::cdc::CdcAcmComponent::new(
        &nrf52840_peripherals.usbd,
        capsules_extra::usb::cdc::MAX_CTRL_PACKET_SIZE_NRF52840,
        0x239a,
        0x8071,
        strings,
        mux_alarm,
        Some(&baud_rate_reset_bootloader_enter),
    )
    .finalize(components::cdc_acm_component_static!(
        nrf52::usbd::Usbd,
        nrf52::rtc::Rtc
    ));
    CDC_REF_FOR_PANIC = Some(cdc); //for use by panic handler

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(cdc, 115200)
        .finalize(components::uart_mux_component_static!());

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
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput7)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A1
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput5)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A2
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput2)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A3
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput3)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A4
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput1)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A5
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput4)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
                // A6
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52840::adc::AdcChannelSetup::new(nrf52840::adc::AdcChannel::AnalogInput0)
                )
                .finalize(components::adc_component_static!(nrf52840::adc::Adc)),
            ));

    //--------------------------------------------------------------------------
    // SENSORS
    //--------------------------------------------------------------------------

    let sensors_i2c_bus = static_init!(
        capsules_core::virtualizers::virtual_i2c::MuxI2C<'static, nrf52840::i2c::TWI>,
        capsules_core::virtualizers::virtual_i2c::MuxI2C::new(&base_peripherals.twi1, None,)
    );
    kernel::deferred_call::DeferredCallClient::register(sensors_i2c_bus);
    base_peripherals.twi1.configure(
        nrf52840::pinmux::Pinmux::new(I2C_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(I2C_SDA_PIN as u32),
    );
    base_peripherals.twi1.set_master_client(sensors_i2c_bus);

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

    let sht3x = components::sht3x::SHT3xComponent::new(
        sensors_i2c_bus,
        capsules_extra::sht3x::BASE_ADDR,
        mux_alarm,
    )
    .finalize(components::sht3x_component_static!(
        nrf52::rtc::Rtc<'static>,
        nrf52840::i2c::TWI
    ));

    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        sht3x,
    )
    .finalize(components::temperature_component_static!(SHT3xSensor));

    let humidity = components::humidity::HumidityComponent::new(
        board_kernel,
        capsules_extra::humidity::DRIVER_NUM,
        sht3x,
    )
    .finalize(components::humidity_component_static!(SHT3xSensor));

    //--------------------------------------------------------------------------
    // TFT
    //--------------------------------------------------------------------------

    let spi_mux = components::spi::SpiMuxComponent::new(&base_peripherals.spim0)
        .finalize(components::spi_mux_component_static!(nrf52840::spi::SPIM));

    base_peripherals.spim0.configure(
        nrf52840::pinmux::Pinmux::new(ST7789H2_MOSI as u32),
        nrf52840::pinmux::Pinmux::new(ST7789H2_MISO as u32),
        nrf52840::pinmux::Pinmux::new(ST7789H2_SCK as u32),
    );

    let bus = components::bus::SpiMasterBusComponent::new(
        spi_mux,
        hil::spi::cs::IntoChipSelect::<_, hil::spi::cs::ActiveLow>::into_cs(
            &nrf52840_peripherals.gpio_port[ST7789H2_CS],
        ),
        20_000_000,
        kernel::hil::spi::ClockPhase::SampleLeading,
        kernel::hil::spi::ClockPolarity::IdleLow,
    )
    .finalize(components::spi_bus_component_static!(nrf52840::spi::SPIM));

    let tft = components::st77xx::ST77XXComponent::new(
        mux_alarm,
        bus,
        Some(&nrf52840_peripherals.gpio_port[ST7789H2_DC]),
        Some(&nrf52840_peripherals.gpio_port[ST7789H2_RESET]),
        &capsules_extra::st77xx::ST7789H2,
    )
    .finalize(components::st77xx_component_static!(
        // bus type
        capsules_extra::bus::SpiMasterBus<
            'static,
            capsules_core::virtualizers::virtual_spi::VirtualSpiMasterDevice<
                'static,
                nrf52840::spi::SPIM,
            >,
        >,
        // timer type
        nrf52840::rtc::Rtc,
        // pin type
        nrf52::gpio::GPIOPin<'static>
    ));

    let _ = tft.init();

    let screen = components::screen::ScreenComponent::new(
        board_kernel,
        capsules_extra::screen::DRIVER_NUM,
        tft,
        Some(tft),
    )
    .finalize(components::screen_component_static!(57600));

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

    let aes_mux = static_init!(
        MuxAES128CCM<'static, nrf52840::aes::AesECB>,
        MuxAES128CCM::new(&base_peripherals.ecb,)
    );
    kernel::deferred_call::DeferredCallClient::register(aes_mux);
    base_peripherals.ecb.set_client(aes_mux);

    let device_id = (*addr_of!(nrf52840::ficr::FICR_INSTANCE)).id();

    let device_id_bottom_16 = u16::from_le_bytes([device_id[0], device_id[1]]);

    let (ieee802154_radio, _mux_mac) = components::ieee802154::Ieee802154Component::new(
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

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm4::support::reset),
    )
    .finalize(components::process_console_component_static!(
        nrf52840::rtc::Rtc
    ));
    let _ = pconsole.start();

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
        proximity,
        led,
        gpio,
        adc: adc_syscall,
        screen,
        button,
        rng,
        buzzer,
        alarm,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        temperature,
        humidity,
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

    debug!("Initialization complete. Entering main loop.");

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

    let (board_kernel, board, chip) = start();
    board_kernel.kernel_loop(&board, chip, Some(&board.ipc), &main_loop_capability);
}
