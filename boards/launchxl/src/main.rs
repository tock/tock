#![no_std]
#![no_main]
#![feature(lang_items, asm, panic_implementation)]

extern crate capsules;
extern crate cc26x2;
extern crate cortexm4;
#[macro_use]
extern crate enum_primitive;
extern crate fixedvec;

#[allow(unused_imports)]
#[macro_use(create_capability, debug, debug_gpio, static_init)]
extern crate kernel;

use capsules::helium;
use capsules::helium::{device::Device, virtual_rfcore::RFCore};
use capsules::virtual_uart::{UartDevice, UartMux};
use cc26x2::adc;
use cc26x2::aon;
use cc26x2::osc;
use cc26x2::prcm;
use cc26x2::pwm;
use cc26x2::radio;
use kernel::capabilities;
use kernel::hil;
use kernel::hil::entropy::Entropy32;
use kernel::hil::gpio::InterruptMode;
use kernel::hil::gpio::Pin;
use kernel::hil::gpio::PinCtl;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::rng::Rng;
use kernel::Chip;

#[macro_use]
pub mod io;

#[allow(dead_code)]
mod ccfg_test;
#[allow(dead_code)]
mod i2c_tests;
#[allow(dead_code)]
mod uart_echo;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 2;
static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; NUM_PROCS] = [None, None];

#[link_section = ".app_memory"]
// Give half of RAM to be dedicated APP memory
static mut APP_MEMORY: [u8; 0xA000] = [0; 0xA000];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

pub struct Platform {
    gpio: &'static capsules::gpio::GPIO<'static, cc26x2::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static, UartDevice<'static>>,
    button: &'static capsules::button::Button<'static, cc26x2::gpio::GPIOPin>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
    >,
    rng: &'static capsules::rng::RngDriver<'static>,
    i2c_master: &'static capsules::i2c_master::I2CMasterDriver<cc26x2::i2c::I2CMaster<'static>>,
    adc: &'static capsules::adc::Adc<'static, cc26x2::adc::Adc>,
    helium: &'static capsules::helium::driver::Helium<'static>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::i2c_master::DRIVER_NUM => f(Some(self.i2c_master)),
            capsules::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules::helium::driver::DRIVER_NUM => f(Some(self.helium)),
            _ => f(None),
        }
    }
}

static mut HELIUM_BUF: [u8; 128] = [0x00; 128];

mod cc1312r;
mod cc1352p;

pub struct Pinmap {
    uart0_rx: usize,
    uart0_tx: usize,
    i2c0_scl: usize,
    i2c0_sda: usize,
    red_led: usize,
    green_led: usize,
    button1: usize,
    button2: usize,
    gpio0: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    pwm0: usize,
    pwm1: usize,
}

unsafe fn configure_pins(pin: &Pinmap) {
    cc26x2::gpio::PORT[pin.uart0_rx].enable_uart0_rx();
    cc26x2::gpio::PORT[pin.uart0_tx].enable_uart0_tx();

    cc26x2::gpio::PORT[pin.i2c0_scl].enable_i2c_scl();
    cc26x2::gpio::PORT[pin.i2c0_sda].enable_i2c_sda();

    cc26x2::gpio::PORT[pin.red_led].enable_gpio();
    cc26x2::gpio::PORT[pin.green_led].enable_gpio();

    cc26x2::gpio::PORT[pin.button1].enable_gpio();
    cc26x2::gpio::PORT[pin.button2].enable_gpio();

    cc26x2::gpio::PORT[pin.gpio0].enable_gpio();

    cc26x2::gpio::PORT[pin.a7].enable_analog_input();
    cc26x2::gpio::PORT[pin.a6].enable_analog_input();
    cc26x2::gpio::PORT[pin.a5].enable_analog_input();
    cc26x2::gpio::PORT[pin.a4].enable_analog_input();
    cc26x2::gpio::PORT[pin.a3].enable_analog_input();
    cc26x2::gpio::PORT[pin.a2].enable_analog_input();
    cc26x2::gpio::PORT[pin.a1].enable_analog_input();
    cc26x2::gpio::PORT[pin.a0].enable_analog_input();

    cc26x2::gpio::PORT[pin.pwm0].enable_pwm(pwm::Timer::GPT0A);
    cc26x2::gpio::PORT[pin.pwm1].enable_pwm(pwm::Timer::GPT0B);
}

#[no_mangle]
pub unsafe fn reset_handler() {
    cc26x2::init();

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Setup AON event defaults
    aon::AON.setup();

    // Power on peripherals (eg. GPIO)
    prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);

    // Wait for it to turn on until we continue
    while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}

    // Power on Serial domain
    prcm::Power::enable_domain(prcm::PowerDomain::Serial);

    while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}

    osc::OSC.request_switch_to_hf_xosc();
    osc::OSC.switch_to_hf_xosc();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Enable the GPIO clocks
    prcm::Clock::enable_gpio();

    let pinmap: &Pinmap;
    let chip_id = (cc26x2::rom::HAPI.get_chip_id)();

    if chip_id == cc1352p::CHIP_ID {
        pinmap = &cc1352p::PINMAP;
    } else {
        pinmap = &cc1312r::PINMAP;
    }

    configure_pins(pinmap);

    // LEDs
    let led_pins = static_init!(
        [(
            &'static cc26x2::gpio::GPIOPin,
            capsules::led::ActivationMode
        ); 2],
        [
            (
                &cc26x2::gpio::PORT[pinmap.red_led],
                capsules::led::ActivationMode::ActiveHigh
            ), // Red
            (
                &cc26x2::gpio::PORT[pinmap.green_led],
                capsules::led::ActivationMode::ActiveHigh
            ), // Green
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONS
    let button_pins = static_init!(
        [(&'static cc26x2::gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (
                &cc26x2::gpio::PORT[pinmap.button1],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 1
            (
                &cc26x2::gpio::PORT[pinmap.button2],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 2
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, cc26x2::gpio::GPIOPin>,
        capsules::button::Button::new(
            button_pins,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );

    let mut count = 0;
    for &(btn, _) in button_pins.iter() {
        btn.set_input_mode(hil::gpio::InputMode::PullUp);
        btn.enable_interrupt(count, InterruptMode::FallingEdge);
        btn.set_client(button);
        count += 1;
    }

    // UART

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        UartMux<'static>,
        UartMux::new(
            &cc26x2::uart::UART0,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    hil::uart::UART::set_client(&cc26x2::uart::UART0, uart_mux);

    // Create a UartDevice for the console.
    let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    console_uart.setup();

    cc26x2::uart::UART0.initialize();

    let console = static_init!(
        capsules::console::Console<UartDevice>,
        capsules::console::Console::new(
            console_uart,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::uart::UART::set_client(console_uart, console);
    console.initialize();

    // Create virtual device for kernel debug.
    let debugger_uart = static_init!(UartDevice, UartDevice::new(uart_mux, false));
    debugger_uart.setup();
    let debugger = static_init!(
        kernel::debug::DebugWriter,
        kernel::debug::DebugWriter::new(
            debugger_uart,
            &mut kernel::debug::OUTPUT_BUF,
            &mut kernel::debug::INTERNAL_BUF,
        )
    );
    hil::uart::UART::set_client(debugger_uart, debugger);

    let debug_wrapper = static_init!(
        kernel::debug::DebugWriterWrapper,
        kernel::debug::DebugWriterWrapper::new(debugger)
    );
    kernel::debug::set_debug_writer_wrapper(debug_wrapper);

    cc26x2::i2c::I2C0.initialize();

    let i2c_master = static_init!(
        capsules::i2c_master::I2CMasterDriver<cc26x2::i2c::I2CMaster<'static>>,
        capsules::i2c_master::I2CMasterDriver::new(
            &cc26x2::i2c::I2C0,
            &mut capsules::i2c_master::BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );

    cc26x2::i2c::I2C0.set_client(i2c_master);
    cc26x2::i2c::I2C0.enable();

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static cc26x2::gpio::GPIOPin; 1],
        [
            // This is the order they appear on the launchxl headers.
            // Pins 5, 8, 11, 29, 30
            &cc26x2::gpio::PORT[pinmap.gpio0],
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, cc26x2::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let rtc = &cc26x2::rtc::RTC;
    rtc.start();

    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&cc26x2::rtc::RTC)
    );
    rtc.set_client(mux_alarm);

    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm1,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    virtual_alarm1.set_client(alarm);

    let entropy_to_random = static_init!(
        capsules::rng::Entropy32ToRandom<'static>,
        capsules::rng::Entropy32ToRandom::new(&cc26x2::trng::TRNG)
    );
    let rng = static_init!(
        capsules::rng::RngDriver<'static>,
        capsules::rng::RngDriver::new(
            entropy_to_random,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    cc26x2::trng::TRNG.set_client(entropy_to_random);
    entropy_to_random.set_client(rng);

    // Set underlying radio client to the radio mode wrapper
    radio::RFC.set_client(&radio::MULTIMODE_RADIO);
    let radio = static_init!(
        helium::virtual_rfcore::VirtualRadio<'static, cc26x2::radio::multimode::Radio>,
        helium::virtual_rfcore::VirtualRadio::new(&cc26x2::radio::MULTIMODE_RADIO)
    );
    // Set mode client in hil
    kernel::hil::rfcore::RadioDriver::set_transmit_client(&radio::MULTIMODE_RADIO, radio);
    kernel::hil::rfcore::RadioDriver::set_receive_client(
        &radio::MULTIMODE_RADIO,
        radio,
        &mut HELIUM_BUF,
    );
    kernel::hil::rfcore::RadioDriver::set_power_client(&radio::MULTIMODE_RADIO, radio);

    // Virtual device that will respond to callbacks from the underlying radio and library
    // operations
    let virtual_device = static_init!(
        helium::framer::Framer<
            'static,
            helium::virtual_rfcore::VirtualRadio<'static, cc26x2::radio::multimode::Radio>,
        >,
        helium::framer::Framer::new(radio)
    );
    // Set client for underlying radio as virtual device
    radio.set_transmit_client(virtual_device);
    radio.set_receive_client(virtual_device);

    // Driver for user to interface with
    let radio_driver = static_init!(
        helium::driver::Helium<'static>,
        helium::driver::Helium::new(
            board_kernel.create_grant(&memory_allocation_capability),
            &mut HELIUM_BUF,
            virtual_device
        )
    );

    virtual_device.set_transmit_client(radio_driver);
    virtual_device.set_receive_client(radio_driver);

    let _rfc = &cc26x2::radio::MULTIMODE_RADIO;

    // set nominal voltage
    cc26x2::adc::ADC.nominal_voltage = Some(3300);
    cc26x2::adc::ADC.configure(adc::Source::Fixed4P5V, adc::SampleCycle::_10p9_ms);

    // Setup ADC
    let adc_channels = static_init!(
        [&cc26x2::adc::Input; 8],
        [
            &cc26x2::adc::Input::Auxio0, // pin 30
            &cc26x2::adc::Input::Auxio1, // pin 29
            &cc26x2::adc::Input::Auxio2, // pin 28
            &cc26x2::adc::Input::Auxio3, // pin 27
            &cc26x2::adc::Input::Auxio4, // pin 26
            &cc26x2::adc::Input::Auxio5, // pin 25
            &cc26x2::adc::Input::Auxio6, // pin 24
            &cc26x2::adc::Input::Auxio7, // pin 23
        ]
    );

    let adc = static_init!(
        capsules::adc::Adc<'static, cc26x2::adc::Adc>,
        capsules::adc::Adc::new(
            &mut cc26x2::adc::ADC,
            adc_channels,
            &mut capsules::adc::ADC_BUFFER1,
            &mut capsules::adc::ADC_BUFFER2,
            &mut capsules::adc::ADC_BUFFER3
        )
    );

    for channel in adc_channels.iter() {
        cc26x2::adc::ADC.set_client(adc, channel);
    }

    let pwm_channels = [
        pwm::Channel::new(pwm::Timer::GPT0A),
        pwm::Channel::new(pwm::Timer::GPT0B),
        pwm::Channel::new(pwm::Timer::GPT1A),
        pwm::Channel::new(pwm::Timer::GPT1B),
        pwm::Channel::new(pwm::Timer::GPT2A),
        pwm::Channel::new(pwm::Timer::GPT2B),
        pwm::Channel::new(pwm::Timer::GPT3A),
        pwm::Channel::new(pwm::Timer::GPT3B),
    ];

    // for testing for now, just enable all channels with PWM
    for pwm_channel in pwm_channels.iter() {
        pwm_channel.enable(0xFF, 0xFF >> 1)
    }

    let launchxl = Platform {
        console,
        gpio,
        led,
        button,
        alarm,
        rng,
        i2c_master,
        adc,
        helium: radio_driver,
    };

    let chip = static_init!(cc26x2::chip::Cc26X2, cc26x2::chip::Cc26X2::new());

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    let ipc = &kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);

    kernel::procs::load_processes(
        board_kernel,
        &cortexm4::syscall::SysCall::new(),
        chip.mpu(),
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );

    board_kernel.kernel_loop(&launchxl, chip, Some(&ipc), &main_loop_capability);
}
