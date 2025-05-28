// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Micro:bit v2.
//!
//! It is based on nRF52833 SoC (Cortex M4 core with a BLE).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;

#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use nrf52833::gpio::Pin;
use nrf52833::interrupt_service::Nrf52833DefaultPeripherals;

// Kernel LED (same as microphone LED)
const LED_KERNEL_PIN: Pin = Pin::P0_20;
const LED_MICROPHONE_PIN: Pin = Pin::P0_20;

// Buttons
const BUTTON_A: Pin = Pin::P0_14;
const BUTTON_B: Pin = Pin::P0_23;
const TOUCH_LOGO: Pin = Pin::P1_04;

// GPIOs

// P0, P1 and P2 are used as ADC, comment them in the ADC section to use them as GPIO
const _GPIO_P0: Pin = Pin::P0_02;
const _GPIO_P1: Pin = Pin::P0_03;
const _GPIO_P2: Pin = Pin::P0_04;
const GPIO_P8: Pin = Pin::P0_10;
const GPIO_P9: Pin = Pin::P0_09;
const GPIO_P16: Pin = Pin::P1_02;

const UART_TX_PIN: Pin = Pin::P0_06;
const UART_RX_PIN: Pin = Pin::P1_08;

/// LED matrix
const LED_MATRIX_COLS: [Pin; 5] = [Pin::P0_28, Pin::P0_11, Pin::P0_31, Pin::P1_05, Pin::P0_30];
const LED_MATRIX_ROWS: [Pin; 5] = [Pin::P0_21, Pin::P0_22, Pin::P0_15, Pin::P0_24, Pin::P0_19];

// Speaker

const SPEAKER_PIN: Pin = Pin::P0_00;

/// I2C pins for all of the sensors.
const I2C_SDA_PIN: Pin = Pin::P0_16;
const I2C_SCL_PIN: Pin = Pin::P0_08;

/// UART Writer for panic!()s.
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static nrf52833::chip::NRF52<Nrf52833DefaultPeripherals>> = None;
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];
// debug mode requires more stack space
// pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

type TemperatureDriver =
    components::temperature::TemperatureComponentType<nrf52::temperature::Temp<'static>>;
type RngDriver = components::rng::RngComponentType<nrf52833::trng::Trng<'static>>;
type Ieee802154RawDriver =
    components::ieee802154::Ieee802154RawComponentType<nrf52833::ieee802154_radio::Radio<'static>>;

/// Supported drivers by the platform
pub struct MicroBit {
    ble_radio: &'static capsules_extra::ble_advertising_driver::BLE<
        'static,
        nrf52::ble_radio::Radio<'static>,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            nrf52::rtc::Rtc<'static>,
        >,
    >,
    eui64: &'static capsules_extra::eui64::Eui64,
    ieee802154: &'static Ieee802154RawDriver,
    console: &'static capsules_core::console::Console<'static>,
    gpio: &'static capsules_core::gpio::GPIO<'static, nrf52::gpio::GPIOPin<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        capsules_extra::led_matrix::LedMatrixLed<
            'static,
            nrf52::gpio::GPIOPin<'static>,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                nrf52::rtc::Rtc<'static>,
            >,
        >,
        25,
    >,
    button: &'static capsules_core::button::Button<'static, nrf52::gpio::GPIOPin<'static>>,
    rng: &'static RngDriver,
    ninedof: &'static capsules_extra::ninedof::NineDof<'static>,
    lsm303agr: &'static capsules_extra::lsm303agr::Lsm303agrI2C<
        'static,
        capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, nrf52833::i2c::TWI<'static>>,
    >,
    temperature: &'static TemperatureDriver,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            nrf52::rtc::Rtc<'static>,
        >,
    >,
    buzzer_driver: &'static capsules_extra::buzzer_driver::Buzzer<
        'static,
        capsules_extra::buzzer_pwm::PwmBuzzer<
            'static,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                nrf52833::rtc::Rtc<'static>,
            >,
            capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52833::pwm::Pwm>,
        >,
    >,
    pwm: &'static capsules_extra::pwm::Pwm<'static, 1>,
    app_flash: &'static capsules_extra::app_flash_driver::AppFlash<'static>,
    sound_pressure: &'static capsules_extra::sound_pressure::SoundPressureSensor<'static>,

    scheduler: &'static RoundRobinSched<'static>,
    systick: cortexm4::systick::SysTick,
}

impl SyscallDriverLookup for MicroBit {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_extra::ninedof::DRIVER_NUM => f(Some(self.ninedof)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_extra::lsm303agr::DRIVER_NUM => f(Some(self.lsm303agr)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_extra::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules_extra::buzzer_driver::DRIVER_NUM => f(Some(self.buzzer_driver)),
            capsules_extra::pwm::DRIVER_NUM => f(Some(self.pwm)),
            capsules_extra::app_flash_driver::DRIVER_NUM => f(Some(self.app_flash)),
            capsules_extra::sound_pressure::DRIVER_NUM => f(Some(self.sound_pressure)),
            capsules_extra::eui64::DRIVER_NUM => f(Some(self.eui64)),
            capsules_extra::ieee802154::DRIVER_NUM => f(Some(self.ieee802154)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<nrf52833::chip::NRF52<'static, Nrf52833DefaultPeripherals<'static>>>
    for MicroBit
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
    MicroBit,
    &'static nrf52833::chip::NRF52<'static, Nrf52833DefaultPeripherals<'static>>,
) {
    nrf52833::init();

    let ieee802154_ack_buf = static_init!(
        [u8; nrf52833::ieee802154_radio::ACK_BUF_SIZE],
        [0; nrf52833::ieee802154_radio::ACK_BUF_SIZE]
    );
    // Initialize chip peripheral drivers
    let nrf52833_peripherals = static_init!(
        Nrf52833DefaultPeripherals,
        Nrf52833DefaultPeripherals::new(ieee802154_ack_buf)
    );

    // set up circular peripheral dependencies
    nrf52833_peripherals.init();

    let base_peripherals = &nrf52833_peripherals.nrf52;

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    //--------------------------------------------------------------------------
    // RAW 802.15.4
    //--------------------------------------------------------------------------

    let device_id = (*addr_of!(nrf52833::ficr::FICR_INSTANCE)).id();

    let eui64 = components::eui64::Eui64Component::new(u64::from_le_bytes(device_id))
        .finalize(components::eui64_component_static!());

    let ieee802154 = components::ieee802154::Ieee802154RawComponent::new(
        board_kernel,
        capsules_extra::ieee802154::DRIVER_NUM,
        &nrf52833_peripherals.ieee802154_radio,
    )
    .finalize(components::ieee802154_raw_component_static!(
        nrf52833::ieee802154_radio::Radio,
    ));
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
    // `debug_gpio!(0, toggle)` macro. We uconfigure these early so that the
    // macro is available during most of the setup code and kernel exection.
    kernel::debug::assign_gpios(
        Some(&nrf52833_peripherals.gpio_port[LED_KERNEL_PIN]),
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
            nrf52833::gpio::GPIOPin,
            // Used as ADC, comment them out in the ADC section to use them as GPIO
            // 0 => &nrf52833_peripherals.gpio_port[GPIO_P0],
            // 1 => &nrf52833_peripherals.gpio_port[_GPIO_P1],
            // 2 => &nrf52833_peripherals.gpio_port[_GPIO_P2],
            // Used as PWM, comment them out in the PWM section to use them as GPIO
            //8 => &nrf52833_peripherals.gpio_port[GPIO_P8],
            9 => &nrf52833_peripherals.gpio_port[GPIO_P9],
            16 => &nrf52833_peripherals.gpio_port[GPIO_P16],
        ),
    )
    .finalize(components::gpio_component_static!(nrf52833::gpio::GPIOPin));

    //--------------------------------------------------------------------------
    // Buttons
    //--------------------------------------------------------------------------
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            nrf52833::gpio::GPIOPin,
            (
                &nrf52833_peripherals.gpio_port[BUTTON_A],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            ), // A
            (
                &nrf52833_peripherals.gpio_port[BUTTON_B],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            ), // B
            (
                &nrf52833_peripherals.gpio_port[TOUCH_LOGO],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullNone
            ), // Touch Logo
        ),
    )
    .finalize(components::button_component_static!(
        nrf52833::gpio::GPIOPin
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

    use kernel::hil::buzzer::Buzzer;
    use kernel::hil::time::Alarm;

    let mux_pwm = components::pwm::PwmMuxComponent::new(&base_peripherals.pwm0)
        .finalize(components::pwm_mux_component_static!(nrf52833::pwm::Pwm));

    let virtual_pwm_buzzer = components::pwm::PwmPinUserComponent::new(
        mux_pwm,
        nrf52833::pinmux::Pinmux::new(SPEAKER_PIN as u32),
    )
    .finalize(components::pwm_pin_user_component_static!(
        nrf52833::pwm::Pwm
    ));

    let virtual_alarm_buzzer = static_init!(
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_buzzer.setup();

    let pwm_buzzer = static_init!(
        capsules_extra::buzzer_pwm::PwmBuzzer<
            'static,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                nrf52833::rtc::Rtc,
            >,
            capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52833::pwm::Pwm>,
        >,
        capsules_extra::buzzer_pwm::PwmBuzzer::new(
            virtual_pwm_buzzer,
            virtual_alarm_buzzer,
            capsules_extra::buzzer_pwm::DEFAULT_MAX_BUZZ_TIME_MS,
        )
    );

    let buzzer_driver = static_init!(
        capsules_extra::buzzer_driver::Buzzer<
            'static,
            capsules_extra::buzzer_pwm::PwmBuzzer<
                'static,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                    'static,
                    nrf52833::rtc::Rtc,
                >,
                capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52833::pwm::Pwm>,
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

    pwm_buzzer.set_client(buzzer_driver);

    virtual_alarm_buzzer.set_alarm_client(pwm_buzzer);

    let virtual_pwm_driver = components::pwm::PwmPinUserComponent::new(
        mux_pwm,
        nrf52833::pinmux::Pinmux::new(GPIO_P8 as u32),
    )
    .finalize(components::pwm_pin_user_component_static!(
        nrf52833::pwm::Pwm
    ));

    let pwm =
        components::pwm::PwmDriverComponent::new(board_kernel, capsules_extra::pwm::DRIVER_NUM)
            .finalize(components::pwm_driver_component_helper!(virtual_pwm_driver));

    //--------------------------------------------------------------------------
    // UART & CONSOLE & DEBUG
    //--------------------------------------------------------------------------

    base_peripherals.uarte0.initialize(
        nrf52::pinmux::Pinmux::new(UART_TX_PIN as u32),
        nrf52::pinmux::Pinmux::new(UART_RX_PIN as u32),
        None,
        None,
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(&base_peripherals.uarte0, 115200)
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
    .finalize(components::rng_component_static!(nrf52833::trng::Trng));

    //--------------------------------------------------------------------------
    // SENSORS
    //--------------------------------------------------------------------------

    base_peripherals.twi1.configure(
        nrf52833::pinmux::Pinmux::new(I2C_SCL_PIN as u32),
        nrf52833::pinmux::Pinmux::new(I2C_SDA_PIN as u32),
    );

    let sensors_i2c_bus = components::i2c::I2CMuxComponent::new(&base_peripherals.twi1, None)
        .finalize(components::i2c_mux_component_static!(
            nrf52833::i2c::TWI<'static>
        ));

    // LSM303AGR

    let lsm303agr = components::lsm303agr::Lsm303agrI2CComponent::new(
        sensors_i2c_bus,
        None,
        None,
        board_kernel,
        capsules_extra::lsm303agr::DRIVER_NUM,
    )
    .finalize(components::lsm303agr_component_static!(
        nrf52833::i2c::TWI<'static>
    ));

    if let Err(error) = lsm303agr.configure(
        capsules_extra::lsm303xx::Lsm303AccelDataRate::DataRate25Hz,
        false,
        capsules_extra::lsm303xx::Lsm303Scale::Scale2G,
        false,
        true,
        capsules_extra::lsm303xx::Lsm303MagnetoDataRate::DataRate3_0Hz,
        capsules_extra::lsm303xx::Lsm303Range::Range1_9G,
    ) {
        debug!("Failed to configure LSM303AGR sensor ({:?})", error);
    }

    let ninedof = components::ninedof::NineDofComponent::new(
        board_kernel,
        capsules_extra::ninedof::DRIVER_NUM,
    )
    .finalize(components::ninedof_component_static!(lsm303agr));

    // Temperature

    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        &base_peripherals.temp,
    )
    .finalize(components::temperature_component_static!(
        nrf52833::temperature::Temp
    ));

    //--------------------------------------------------------------------------
    // ADC
    //--------------------------------------------------------------------------
    base_peripherals.adc.calibrate();

    let adc_mux = components::adc::AdcMuxComponent::new(&base_peripherals.adc)
        .finalize(components::adc_mux_component_static!(nrf52833::adc::Adc));

    // Comment out the following to use P0, P1 and P2 as GPIO
    let adc_syscall =
        components::adc::AdcVirtualComponent::new(board_kernel, capsules_core::adc::DRIVER_NUM)
            .finalize(components::adc_syscall_component_helper!(
                // ADC Ring 0 (P0)
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52833::adc::AdcChannelSetup::new(nrf52833::adc::AdcChannel::AnalogInput0)
                )
                .finalize(components::adc_component_static!(nrf52833::adc::Adc)),
                // ADC Ring 1 (P1)
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52833::adc::AdcChannelSetup::new(nrf52833::adc::AdcChannel::AnalogInput1)
                )
                .finalize(components::adc_component_static!(nrf52833::adc::Adc)),
                // ADC Ring 2 (P2)
                components::adc::AdcComponent::new(
                    adc_mux,
                    nrf52833::adc::AdcChannelSetup::new(nrf52833::adc::AdcChannel::AnalogInput2)
                )
                .finalize(components::adc_component_static!(nrf52833::adc::Adc))
            ));

    // Microphone

    let adc_microphone = components::adc_microphone::AdcMicrophoneComponent::new(
        adc_mux,
        nrf52833::adc::AdcChannelSetup::setup(
            nrf52833::adc::AdcChannel::AnalogInput3,
            nrf52833::adc::AdcChannelGain::Gain4,
            nrf52833::adc::AdcChannelResistor::Bypass,
            nrf52833::adc::AdcChannelResistor::Pulldown,
            nrf52833::adc::AdcChannelSamplingTime::us3,
        ),
        Some(&nrf52833_peripherals.gpio_port[LED_MICROPHONE_PIN]),
    )
    .finalize(components::adc_microphone_component_static!(
        // adc
        nrf52833::adc::Adc,
        // buffer size
        50,
        // gpio
        nrf52833::gpio::GPIOPin
    ));

    nrf52833_peripherals.gpio_port[LED_MICROPHONE_PIN].set_high_drive(true);

    let sound_pressure = components::sound_pressure::SoundPressureComponent::new(
        board_kernel,
        capsules_extra::sound_pressure::DRIVER_NUM,
        adc_microphone,
    )
    .finalize(components::sound_pressure_component_static!());

    //--------------------------------------------------------------------------
    // STORAGE
    //--------------------------------------------------------------------------

    let mux_flash = components::flash::FlashMuxComponent::new(&base_peripherals.nvmc).finalize(
        components::flash_mux_component_static!(nrf52833::nvmc::Nvmc),
    );

    // App Flash

    let virtual_app_flash = components::flash::FlashUserComponent::new(mux_flash).finalize(
        components::flash_user_component_static!(nrf52833::nvmc::Nvmc),
    );

    let app_flash = components::app_flash_driver::AppFlashComponent::new(
        board_kernel,
        capsules_extra::app_flash_driver::DRIVER_NUM,
        virtual_app_flash,
    )
    .finalize(components::app_flash_component_static!(
        capsules_core::virtualizers::virtual_flash::FlashUser<'static, nrf52833::nvmc::Nvmc>,
        512
    ));

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
        nrf52833::rtc::Rtc,
        nrf52833::ble_radio::Radio
    ));

    //--------------------------------------------------------------------------
    // LED Matrix
    //--------------------------------------------------------------------------

    let led_matrix = components::led_matrix::LedMatrixComponent::new(
        mux_alarm,
        components::led_line_component_static!(
            nrf52833::gpio::GPIOPin,
            &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[0]],
            &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[1]],
            &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[2]],
            &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[3]],
            &nrf52833_peripherals.gpio_port[LED_MATRIX_COLS[4]],
        ),
        components::led_line_component_static!(
            nrf52833::gpio::GPIOPin,
            &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[0]],
            &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[1]],
            &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[2]],
            &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[3]],
            &nrf52833_peripherals.gpio_port[LED_MATRIX_ROWS[4]],
        ),
        kernel::hil::gpio::ActivationMode::ActiveLow,
        kernel::hil::gpio::ActivationMode::ActiveHigh,
        60,
    )
    .finalize(components::led_matrix_component_static!(
        nrf52833::gpio::GPIOPin,
        nrf52::rtc::Rtc<'static>,
        5,
        5
    ));

    let led = static_init!(
        capsules_core::led::LedDriver<
            'static,
            capsules_extra::led_matrix::LedMatrixLed<
                'static,
                nrf52::gpio::GPIOPin<'static>,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                    'static,
                    nrf52::rtc::Rtc<'static>,
                >,
            >,
            25,
        >,
        capsules_core::led::LedDriver::new(components::led_matrix_leds!(
            nrf52::gpio::GPIOPin<'static>,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                nrf52::rtc::Rtc<'static>,
            >,
            led_matrix,
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 0),
            (0, 1),
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 1),
            (0, 2),
            (1, 2),
            (2, 2),
            (3, 2),
            (4, 2),
            (0, 3),
            (1, 3),
            (2, 3),
            (3, 3),
            (4, 3),
            (0, 4),
            (1, 4),
            (2, 4),
            (3, 4),
            (4, 4)
        )),
    );

    //--------------------------------------------------------------------------
    // Process Console
    //--------------------------------------------------------------------------
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    let _process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm4::support::reset),
    )
    .finalize(components::process_console_component_static!(
        nrf52833::rtc::Rtc
    ));
    let _ = _process_console.start();

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    // it seems that microbit v2 has no external clock
    base_peripherals.clock.low_stop();
    base_peripherals.clock.high_stop();
    base_peripherals.clock.low_start();
    base_peripherals.clock.high_start();
    while !base_peripherals.clock.low_started() {}
    while !base_peripherals.clock.high_started() {}

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let microbit = MicroBit {
        ble_radio,
        ieee802154,
        eui64,
        console,
        gpio,
        button,
        led,
        rng,
        temperature,
        lsm303agr,
        ninedof,
        buzzer_driver,
        pwm,
        sound_pressure,
        adc: adc_syscall,
        alarm,
        app_flash,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),

        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
    };

    let chip = static_init!(
        nrf52833::chip::NRF52<Nrf52833DefaultPeripherals>,
        nrf52833::chip::NRF52::new(nrf52833_peripherals)
    );
    CHIP = Some(chip);

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

    (board_kernel, microbit, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, board, chip) = start();
    board_kernel.kernel_loop(&board, chip, Some(&board.ipc), &main_loop_capability);
}
