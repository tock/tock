// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Aconno ACD52832 board based on the Nordic nRF52832 MCU.

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::adc::Adc;
use kernel::hil::buzzer::Buzzer;
use kernel::hil::gpio::{Configure, InterruptWithValue, Output};
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedLow;
use kernel::hil::time::{Alarm, Counter};
use kernel::hil::uart::BAUD115200;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::round_robin::RoundRobinSched;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, static_init};
use nrf52832::gpio::Pin;
use nrf52832::interrupt_service::Nrf52832DefaultPeripherals;
use nrf52832::rtc::Rtc;

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
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

type TemperatureDriver =
    components::temperature::TemperatureComponentType<nrf52832::temperature::Temp<'static>>;
type RngDriver = components::rng::RngComponentType<nrf52832::trng::Trng<'static>>;

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules_extra::ble_advertising_driver::BLE<
        'static,
        nrf52832::ble_radio::Radio<'static>,
        VirtualMuxAlarm<'static, Rtc<'static>>,
    >,
    button: &'static capsules_core::button::Button<'static, nrf52832::gpio::GPIOPin<'static>>,
    console: &'static capsules_core::console::Console<'static>,
    gpio: &'static capsules_core::gpio::GPIO<'static, nrf52832::gpio::GPIOPin<'static>>,
    led: &'static capsules_core::led::LedDriver<
        'static,
        LedLow<'static, nrf52832::gpio::GPIOPin<'static>>,
        4,
    >,
    rng: &'static RngDriver,
    temp: &'static TemperatureDriver,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, nrf52832::rtc::Rtc<'static>>,
    >,
    gpio_async: &'static capsules_extra::gpio_async::GPIOAsync<
        'static,
        capsules_extra::mcp230xx::MCP230xx<
            'static,
            capsules_core::virtualizers::virtual_i2c::I2CDevice<
                'static,
                nrf52832::i2c::TWI<'static>,
            >,
        >,
    >,
    light: &'static capsules_extra::ambient_light::AmbientLight<'static>,
    buzzer: &'static capsules_extra::buzzer_driver::Buzzer<
        'static,
        capsules_extra::buzzer_pwm::PwmBuzzer<
            'static,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                nrf52832::rtc::Rtc<'static>,
            >,
            capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52832::pwm::Pwm>,
        >,
    >,
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
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_extra::ble_advertising_driver::DRIVER_NUM => f(Some(self.ble_radio)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules_extra::gpio_async::DRIVER_NUM => f(Some(self.gpio_async)),
            capsules_extra::ambient_light::DRIVER_NUM => f(Some(self.light)),
            capsules_extra::buzzer_driver::DRIVER_NUM => f(Some(self.buzzer)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<nrf52832::chip::NRF52<'static, Nrf52832DefaultPeripherals<'static>>>
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
    &'static nrf52832::chip::NRF52<'static, Nrf52832DefaultPeripherals<'static>>,
) {
    nrf52832::init();

    let nrf52832_peripherals = static_init!(
        Nrf52832DefaultPeripherals,
        Nrf52832DefaultPeripherals::new()
    );

    // set up circular peripheral dependencies
    nrf52832_peripherals.init();
    let base_peripherals = &nrf52832_peripherals.nrf52;

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    // Make non-volatile memory writable and activate the reset button
    let uicr = nrf52832::uicr::Uicr::new();
    base_peripherals.nvmc.erase_uicr();
    base_peripherals.nvmc.configure_writeable();
    while !base_peripherals.nvmc.is_ready() {}
    uicr.set_psel0_reset_pin(BUTTON_RST_PIN);
    while !base_peripherals.nvmc.is_ready() {}
    uicr.set_psel1_reset_pin(BUTTON_RST_PIN);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&nrf52832_peripherals.gpio_port[LED2_PIN]),
        Some(&nrf52832_peripherals.gpio_port[LED3_PIN]),
        Some(&nrf52832_peripherals.gpio_port[LED4_PIN]),
    );

    //
    // GPIO Pins
    //
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            nrf52832::gpio::GPIOPin,
            0 => &nrf52832_peripherals.gpio_port[Pin::P0_25],
            1 => &nrf52832_peripherals.gpio_port[Pin::P0_26],
            2 => &nrf52832_peripherals.gpio_port[Pin::P0_27],
            3 => &nrf52832_peripherals.gpio_port[Pin::P0_28],
            4 => &nrf52832_peripherals.gpio_port[Pin::P0_29],
            5 => &nrf52832_peripherals.gpio_port[Pin::P0_30],
            6 => &nrf52832_peripherals.gpio_port[Pin::P0_31]
        ),
    )
    .finalize(components::gpio_component_static!(nrf52832::gpio::GPIOPin));

    //
    // LEDs
    //
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        LedLow<'static, nrf52832::gpio::GPIOPin>,
        LedLow::new(&nrf52832_peripherals.gpio_port[LED1_PIN]),
        LedLow::new(&nrf52832_peripherals.gpio_port[LED2_PIN]),
        LedLow::new(&nrf52832_peripherals.gpio_port[LED3_PIN]),
        LedLow::new(&nrf52832_peripherals.gpio_port[LED4_PIN]),
    ));

    //
    // Buttons
    //
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            nrf52832::gpio::GPIOPin,
            // 13
            (
                &nrf52832_peripherals.gpio_port[BUTTON1_PIN],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            ),
            // 14
            (
                &nrf52832_peripherals.gpio_port[BUTTON2_PIN],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            ),
            // 15
            (
                &nrf52832_peripherals.gpio_port[BUTTON3_PIN],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            ),
            // 16
            (
                &nrf52832_peripherals.gpio_port[BUTTON4_PIN],
                hil::gpio::ActivationMode::ActiveLow,
                hil::gpio::FloatingState::PullUp
            )
        ),
    )
    .finalize(components::button_component_static!(
        nrf52832::gpio::GPIOPin
    ));

    //
    // RTC for Timers
    //
    let rtc = &base_peripherals.rtc;
    let _ = rtc.start();
    let mux_alarm = static_init!(
        capsules_core::virtualizers::virtual_alarm::MuxAlarm<'static, nrf52832::rtc::Rtc>,
        capsules_core::virtualizers::virtual_alarm::MuxAlarm::new(&base_peripherals.rtc)
    );
    rtc.set_alarm_client(mux_alarm);

    //
    // Timer/Alarm
    //

    // Virtual alarm for the userspace timers
    let alarm_driver_virtual_alarm = static_init!(
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    alarm_driver_virtual_alarm.setup();

    // Userspace timer driver
    let alarm = static_init!(
        capsules_core::alarm::AlarmDriver<
            'static,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                nrf52832::rtc::Rtc,
            >,
        >,
        capsules_core::alarm::AlarmDriver::new(
            alarm_driver_virtual_alarm,
            board_kernel.create_grant(
                capsules_core::alarm::DRIVER_NUM,
                &memory_allocation_capability
            )
        )
    );
    alarm_driver_virtual_alarm.set_alarm_client(alarm);

    //
    // RTT and Console and `debug!()`
    //

    // RTT communication channel
    let rtt_memory = components::segger_rtt::SeggerRttMemoryComponent::new()
        .finalize(components::segger_rtt_memory_component_static!());
    let rtt = components::segger_rtt::SeggerRttComponent::new(mux_alarm, rtt_memory)
        .finalize(components::segger_rtt_component_static!(nrf52832::rtc::Rtc));

    //
    // Virtual UART
    //

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(rtt, BAUD115200)
        .finalize(components::uart_mux_component_static!());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux)
        .finalize(components::debug_writer_component_static!());

    //
    // I2C Devices
    //

    // Create shared mux for the I2C bus
    let i2c_mux = static_init!(
        capsules_core::virtualizers::virtual_i2c::MuxI2C<'static, nrf52832::i2c::TWI<'static>>,
        capsules_core::virtualizers::virtual_i2c::MuxI2C::new(&base_peripherals.twi1, None,)
    );
    kernel::deferred_call::DeferredCallClient::register(i2c_mux);
    base_peripherals.twi1.configure(
        nrf52832::pinmux::Pinmux::new(21),
        nrf52832::pinmux::Pinmux::new(20),
    );
    base_peripherals.twi1.set_master_client(i2c_mux);

    // Configure the MCP23017. Device address 0x20.
    let mcp_pin0 = static_init!(
        hil::gpio::InterruptValueWrapper<'static, nrf52832::gpio::GPIOPin>,
        hil::gpio::InterruptValueWrapper::new(&nrf52832_peripherals.gpio_port[Pin::P0_11])
    )
    .finalize();
    let mcp_pin1 = static_init!(
        hil::gpio::InterruptValueWrapper<'static, nrf52832::gpio::GPIOPin>,
        hil::gpio::InterruptValueWrapper::new(&nrf52832_peripherals.gpio_port[Pin::P0_12])
    )
    .finalize();
    let mcp23017_i2c = static_init!(
        capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, nrf52832::i2c::TWI<'static>>,
        capsules_core::virtualizers::virtual_i2c::I2CDevice::new(i2c_mux, 0x40)
    );
    let mcp230xx_buffer = static_init!(
        [u8; capsules_extra::mcp230xx::BUFFER_LENGTH],
        [0; capsules_extra::mcp230xx::BUFFER_LENGTH]
    );
    let mcp23017 = static_init!(
        capsules_extra::mcp230xx::MCP230xx<
            'static,
            capsules_core::virtualizers::virtual_i2c::I2CDevice<
                'static,
                nrf52832::i2c::TWI<'static>,
            >,
        >,
        capsules_extra::mcp230xx::MCP230xx::new(
            mcp23017_i2c,
            Some(mcp_pin0),
            Some(mcp_pin1),
            mcp230xx_buffer,
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
    let async_gpio_ports = static_init!(
        [&'static capsules_extra::mcp230xx::MCP230xx<
            capsules_core::virtualizers::virtual_i2c::I2CDevice<
                'static,
                nrf52832::i2c::TWI<'static>,
            >,
        >; 1],
        [mcp23017]
    );

    // `gpio_async` is the object that manages all of the extenders.
    let gpio_async = static_init!(
        capsules_extra::gpio_async::GPIOAsync<
            'static,
            capsules_extra::mcp230xx::MCP230xx<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<
                    'static,
                    nrf52832::i2c::TWI<'static>,
                >,
            >,
        >,
        capsules_extra::gpio_async::GPIOAsync::new(
            async_gpio_ports,
            board_kernel.create_grant(
                capsules_extra::gpio_async::DRIVER_NUM,
                &memory_allocation_capability,
            ),
        ),
    );
    // Setup the clients correctly.
    for port in async_gpio_ports.iter() {
        port.set_client(gpio_async);
    }

    //
    // BLE
    //

    let ble_radio = components::ble::BLEComponent::new(
        board_kernel,
        capsules_extra::ble_advertising_driver::DRIVER_NUM,
        &base_peripherals.ble_radio,
        mux_alarm,
    )
    .finalize(components::ble_component_static!(
        nrf52832::rtc::Rtc,
        nrf52832::ble_radio::Radio
    ));

    //
    // Temperature
    //

    // Setup internal temperature sensor
    let temp = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        &base_peripherals.temp,
    )
    .finalize(components::temperature_component_static!(
        nrf52832::temperature::Temp
    ));

    //
    // RNG
    //

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &base_peripherals.trng,
    )
    .finalize(components::rng_component_static!(nrf52832::trng::Trng));

    //
    // Light Sensor
    //

    // Setup Analog Light Sensor
    let analog_light_channel = static_init!(
        nrf52832::adc::AdcChannelSetup,
        nrf52832::adc::AdcChannelSetup::new(nrf52832::adc::AdcChannel::AnalogInput5)
    );

    let analog_light_sensor = static_init!(
        capsules_extra::analog_sensor::AnalogLightSensor<'static, nrf52832::adc::Adc>,
        capsules_extra::analog_sensor::AnalogLightSensor::new(
            &base_peripherals.adc,
            analog_light_channel,
            capsules_extra::analog_sensor::AnalogLightSensorType::LightDependentResistor,
        )
    );
    base_peripherals.adc.set_client(analog_light_sensor);

    // Create userland driver for ambient light sensor
    let light = static_init!(
        capsules_extra::ambient_light::AmbientLight<'static>,
        capsules_extra::ambient_light::AmbientLight::new(
            analog_light_sensor,
            board_kernel.create_grant(
                capsules_extra::ambient_light::DRIVER_NUM,
                &memory_allocation_capability
            )
        )
    );
    hil::sensors::AmbientLight::set_client(analog_light_sensor, light);

    //
    // PWM
    //
    let mux_pwm = static_init!(
        capsules_core::virtualizers::virtual_pwm::MuxPwm<'static, nrf52832::pwm::Pwm>,
        capsules_core::virtualizers::virtual_pwm::MuxPwm::new(&base_peripherals.pwm0)
    );
    let virtual_pwm_buzzer = static_init!(
        capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52832::pwm::Pwm>,
        capsules_core::virtualizers::virtual_pwm::PwmPinUser::new(
            mux_pwm,
            nrf52832::pinmux::Pinmux::new(31)
        )
    );
    virtual_pwm_buzzer.add_to_mux();

    //
    // Buzzer
    //
    let virtual_alarm_buzzer = static_init!(
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_buzzer.setup();

    let pwm_buzzer = static_init!(
        capsules_extra::buzzer_pwm::PwmBuzzer<
            'static,
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                'static,
                nrf52832::rtc::Rtc,
            >,
            capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52832::pwm::Pwm>,
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
                    nrf52832::rtc::Rtc,
                >,
                capsules_core::virtualizers::virtual_pwm::PwmPinUser<'static, nrf52832::pwm::Pwm>,
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

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    base_peripherals.clock.low_stop();
    base_peripherals.clock.high_stop();

    base_peripherals
        .clock
        .low_set_source(nrf52832::clock::LowClockSource::XTAL);
    base_peripherals.clock.low_start();
    base_peripherals.clock.high_start();
    while !base_peripherals.clock.low_started() {}
    while !base_peripherals.clock.high_started() {}

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&*addr_of!(PROCESSES))
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        button,
        ble_radio,
        console,
        led,
        gpio,
        rng,
        temp,
        alarm,
        gpio_async,
        light,
        buzzer,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_capability,
        ),
        scheduler,
        systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
    };

    let chip = static_init!(
        nrf52832::chip::NRF52<Nrf52832DefaultPeripherals>,
        nrf52832::chip::NRF52::new(nrf52832_peripherals)
    );

    nrf52832_peripherals.gpio_port[Pin::P0_31].make_output();
    nrf52832_peripherals.gpio_port[Pin::P0_31].clear();

    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &*addr_of!(nrf52832::ficr::FICR_INSTANCE));

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
