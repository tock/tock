//! Tock kernel for the Aconno ACD52832 board based on the Nordic nRF52832 MCU.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::entropy::Entropy32;
use kernel::hil::gpio::{Configure, InterruptWithValue, Output};
use kernel::hil::rng::Rng;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, static_init};
use nrf52832::gpio::Pin;
use nrf52832::rtc::Rtc;

use nrf52dk_base::nrf52_components::ble::BLEComponent;

const LED1_PIN: Pin = Pin::P0_26;
const LED2_PIN: Pin = Pin::P0_22;
const LED3_PIN: Pin = Pin::P0_23;
const LED4_PIN: Pin = Pin::P0_24;

const BUTTON1_PIN: Pin = Pin::P0_25;
const BUTTON2_PIN: Pin = Pin::P0_14;
const BUTTON3_PIN: Pin = Pin::P0_15;
const BUTTON4_PIN: Pin = Pin::P0_16;
const BUTTON_RST_PIN: Pin = Pin::P0_19;

/// UART Writer
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 32768] = [0; 32768];

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        nrf52832::ble_radio::Radio,
        VirtualMuxAlarm<'static, Rtc<'static>>,
    >,
    button: &'static capsules::button::Button<'static>,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static>,
    led: &'static capsules::led::LED<'static>,
    rng: &'static capsules::rng::RngDriver<'static>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    ipc: kernel::ipc::IPC,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, nrf52832::rtc::Rtc<'static>>,
    >,
    gpio_async:
        &'static capsules::gpio_async::GPIOAsync<'static, capsules::mcp230xx::MCP230xx<'static>>,
    light: &'static capsules::ambient_light::AmbientLight<'static>,
    buzzer: &'static capsules::buzzer_driver::Buzzer<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc<'static>>,
    >,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules::gpio_async::DRIVER_NUM => f(Some(self.gpio_async)),
            capsules::ambient_light::DRIVER_NUM => f(Some(self.light)),
            capsules::buzzer_driver::DRIVER_NUM => f(Some(self.buzzer)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52832::init();

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // GPIOs
    let gpio_pins = static_init!(
        [&'static dyn kernel::hil::gpio::InterruptValuePin; 7],
        [
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf52832::gpio::PORT[Pin::P0_25])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf52832::gpio::PORT[Pin::P0_26])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf52832::gpio::PORT[Pin::P0_27])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf52832::gpio::PORT[Pin::P0_28])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf52832::gpio::PORT[Pin::P0_29])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf52832::gpio::PORT[Pin::P0_30])
            )
            .finalize(),
            static_init!(
                kernel::hil::gpio::InterruptValueWrapper,
                kernel::hil::gpio::InterruptValueWrapper::new(&nrf52832::gpio::PORT[Pin::P0_31])
            )
            .finalize(),
        ]
    );

    // LEDs
    let led_pins = static_init!(
        [(
            &'static dyn hil::gpio::Pin,
            kernel::hil::gpio::ActivationMode
        ); 4],
        [
            (
                &nrf52832::gpio::PORT[LED1_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow
            ),
            (
                &nrf52832::gpio::PORT[LED2_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow
            ),
            (
                &nrf52832::gpio::PORT[LED3_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow
            ),
            (
                &nrf52832::gpio::PORT[LED4_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow
            ),
        ]
    );

    // Make non-volatile memory writable and activate the reset button
    let uicr = nrf52832::uicr::Uicr::new();
    nrf52832::nvmc::NVMC.erase_uicr();
    nrf52832::nvmc::NVMC.configure_writeable();
    while !nrf52832::nvmc::NVMC.is_ready() {}
    uicr.set_psel0_reset_pin(BUTTON_RST_PIN);
    while !nrf52832::nvmc::NVMC.is_ready() {}
    uicr.set_psel1_reset_pin(BUTTON_RST_PIN);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&nrf52832::gpio::PORT[LED2_PIN]),
        Some(&nrf52832::gpio::PORT[LED3_PIN]),
        Some(&nrf52832::gpio::PORT[LED4_PIN]),
    );

    //
    // GPIO Pins
    //
    let gpio = static_init!(
        capsules::gpio::GPIO<'static>,
        capsules::gpio::GPIO::new(
            gpio_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    //
    // LEDs
    //
    let led = static_init!(
        capsules::led::LED<'static>,
        capsules::led::LED::new(led_pins)
    );

    //
    // Buttons
    //
    let button = components::button::ButtonComponent::new(board_kernel).finalize(
        components::button_component_helper!(
            // 13
            (
                &nrf52832::gpio::PORT[BUTTON1_PIN],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            ),
            // 14
            (
                &nrf52832::gpio::PORT[BUTTON2_PIN],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            ),
            // 15
            (
                &nrf52832::gpio::PORT[BUTTON3_PIN],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            ),
            // 16
            (
                &nrf52832::gpio::PORT[BUTTON4_PIN],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            )
        ),
    );

    //
    // RTC for Timers
    //
    let rtc = &nrf52832::rtc::RTC;
    rtc.start();
    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, nrf52832::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&nrf52832::rtc::RTC)
    );
    hil::time::Alarm::set_client(rtc, mux_alarm);

    //
    // Timer/Alarm
    //

    // Virtual alarm for the userspace timers
    let alarm_driver_virtual_alarm = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );

    // Userspace timer driver
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(
            alarm_driver_virtual_alarm,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    hil::time::Alarm::set_client(alarm_driver_virtual_alarm, alarm);

    //
    // RTT and Console and `debug!()`
    //

    // RTT communication channel
    let rtt_memory = components::segger_rtt::SeggerRttMemoryComponent::new().finalize(());
    let rtt = components::segger_rtt::SeggerRttComponent::new(mux_alarm, rtt_memory)
        .finalize(components::segger_rtt_component_helper!(nrf52832::rtc::Rtc));

    //
    // Virtual UART
    //

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(rtt, 115200, dynamic_deferred_caller)
        .finalize(());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    //
    // I2C Devices
    //

    // Create shared mux for the I2C bus
    let i2c_mux = static_init!(
        capsules::virtual_i2c::MuxI2C<'static>,
        capsules::virtual_i2c::MuxI2C::new(&nrf52832::i2c::TWIM0)
    );
    nrf52832::i2c::TWIM0.configure(
        nrf52832::pinmux::Pinmux::new(21),
        nrf52832::pinmux::Pinmux::new(20),
    );
    nrf52832::i2c::TWIM0.set_client(i2c_mux);

    // Configure the MCP23017. Device address 0x20.
    let mcp_pin0 = static_init!(
        hil::gpio::InterruptValueWrapper,
        hil::gpio::InterruptValueWrapper::new(&nrf52832::gpio::PORT[Pin::P0_11])
    )
    .finalize();
    let mcp_pin1 = static_init!(
        hil::gpio::InterruptValueWrapper,
        hil::gpio::InterruptValueWrapper::new(&nrf52832::gpio::PORT[Pin::P0_12])
    )
    .finalize();
    let mcp23017_i2c = static_init!(
        capsules::virtual_i2c::I2CDevice,
        capsules::virtual_i2c::I2CDevice::new(i2c_mux, 0x40)
    );
    let mcp23017 = static_init!(
        capsules::mcp230xx::MCP230xx<'static>,
        capsules::mcp230xx::MCP230xx::new(
            mcp23017_i2c,
            Some(mcp_pin0),
            Some(mcp_pin1),
            &mut capsules::mcp230xx::BUFFER,
            8,
            2
        )
    );
    mcp23017_i2c.set_client(mcp23017);
    mcp_pin0.set_client(mcp23017);
    mcp_pin1.set_client(mcp23017);

    //
    // GPIO Extenders
    //

    // Create an array of the GPIO extenders so we can pass them to an
    // administrative layer that provides a single interface to them all.
    let async_gpio_ports = static_init!([&'static capsules::mcp230xx::MCP230xx; 1], [mcp23017]);

    // `gpio_async` is the object that manages all of the extenders.
    let gpio_async = static_init!(
        capsules::gpio_async::GPIOAsync<'static, capsules::mcp230xx::MCP230xx<'static>>,
        capsules::gpio_async::GPIOAsync::new(async_gpio_ports)
    );
    // Setup the clients correctly.
    for port in async_gpio_ports.iter() {
        port.set_client(gpio_async);
    }

    //
    // BLE
    //

    let ble_radio =
        BLEComponent::new(board_kernel, &nrf52832::ble_radio::RADIO, mux_alarm).finalize(());

    //
    // Temperature
    //

    // Setup internal temperature sensor
    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(
            &mut nrf52832::temperature::TEMP,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::sensors::TemperatureDriver::set_client(&nrf52832::temperature::TEMP, temp);

    //
    // RNG
    //

    // Convert hardware RNG to the Random interface.
    let entropy_to_random = static_init!(
        capsules::rng::Entropy32ToRandom<'static>,
        capsules::rng::Entropy32ToRandom::new(&nrf52832::trng::TRNG)
    );
    nrf52832::trng::TRNG.set_client(entropy_to_random);

    // Setup RNG for userspace
    let rng = static_init!(
        capsules::rng::RngDriver<'static>,
        capsules::rng::RngDriver::new(
            entropy_to_random,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    entropy_to_random.set_client(rng);

    //
    // Light Sensor
    //

    // Setup Analog Light Sensor
    let analog_light_sensor = static_init!(
        capsules::analog_sensor::AnalogLightSensor<'static, nrf52832::adc::Adc>,
        capsules::analog_sensor::AnalogLightSensor::new(
            &nrf52832::adc::ADC,
            &nrf52832::adc::AdcChannel::AnalogInput5,
            capsules::analog_sensor::AnalogLightSensorType::LightDependentResistor,
        )
    );
    nrf52832::adc::ADC.set_client(analog_light_sensor);

    // Create userland driver for ambient light sensor
    let light = static_init!(
        capsules::ambient_light::AmbientLight<'static>,
        capsules::ambient_light::AmbientLight::new(
            analog_light_sensor,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    hil::sensors::AmbientLight::set_client(analog_light_sensor, light);

    //
    // PWM
    //
    let mux_pwm = static_init!(
        capsules::virtual_pwm::MuxPwm<'static, nrf52832::pwm::Pwm>,
        capsules::virtual_pwm::MuxPwm::new(&nrf52832::pwm::PWM0)
    );
    let virtual_pwm_buzzer = static_init!(
        capsules::virtual_pwm::PwmPinUser<'static, nrf52832::pwm::Pwm>,
        capsules::virtual_pwm::PwmPinUser::new(mux_pwm, nrf52832::pinmux::Pinmux::new(31))
    );
    virtual_pwm_buzzer.add_to_mux();

    //
    // Buzzer
    //
    let virtual_alarm_buzzer = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let buzzer = static_init!(
        capsules::buzzer_driver::Buzzer<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>,
        >,
        capsules::buzzer_driver::Buzzer::new(
            virtual_pwm_buzzer,
            virtual_alarm_buzzer,
            capsules::buzzer_driver::DEFAULT_MAX_BUZZ_TIME_MS,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    hil::time::Alarm::set_client(virtual_alarm_buzzer, buzzer);

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52832::clock::CLOCK.low_stop();
    nrf52832::clock::CLOCK.high_stop();

    nrf52832::clock::CLOCK.low_set_source(nrf52832::clock::LowClockSource::XTAL);
    nrf52832::clock::CLOCK.low_start();
    nrf52832::clock::CLOCK.high_set_source(nrf52832::clock::HighClockSource::XTAL);
    nrf52832::clock::CLOCK.high_start();
    while !nrf52832::clock::CLOCK.low_started() {}
    while !nrf52832::clock::CLOCK.high_started() {}

    let platform = Platform {
        button: button,
        ble_radio: ble_radio,
        console: console,
        led: led,
        gpio: gpio,
        rng: rng,
        temp: temp,
        alarm: alarm,
        gpio_async: gpio_async,
        light: light,
        buzzer: buzzer,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    let chip = static_init!(nrf52832::chip::Chip, nrf52832::chip::new());

    nrf52832::gpio::PORT[Pin::P0_31].make_output();
    nrf52832::gpio::PORT[Pin::P0_31].clear();

    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &nrf52832::ficr::FICR_INSTANCE);

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;

        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _eapps: u8;
    }
    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
