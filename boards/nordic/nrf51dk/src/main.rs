//! Tock kernel for the Nordic Semiconductor nRF51 development
//! kit (DK), a.k.a. the PCA10028. </br>
//! This is an nRF51422 SoC (a Cortex M0 core with a BLE transceiver) with many
//! exported pins, LEDs, and buttons. </br>
//!
//! Currently the kernel provides:
//!
//! * Timers
//! * GPIO
//! * UART
//! * AES Encryption
//! * Bluetooth Low Energy Advertisements
//! * Temperature Sensor
//! * True Random Number Generator
//!
//! ### Pin configuration
//! * 0 -> LED1 (pin 21)
//! * 1 -> LED2 (pin 22)
//! * 2 -> LED3 (pin 23)
//! * 3 -> LED4 (pin 24)
//! * 5 -> BUTTON1 (pin 17)
//! * 6 -> BUTTON2 (pin 18)
//! * 7 -> BUTTON3 (pin 19)
//! * 8 -> BUTTON4 (pin 20)
//! * 9 -> P0.01   (bottom left header)
//! * 10 -> P0.02   (bottom left header)
//! * 11 -> P0.03   (bottom left header)
//! * 12 -> P0.04   (bottom left header)
//! * 12 -> P0.05   (bottom left header)
//! * 13 -> P0.06   (bottom left header)
//! * 14 -> P0.19   (mid right header)
//! * 15 -> P0.18   (mid right header)
//! * 16 -> P0.17   (mid right header)
//! * 17 -> P0.16   (mid right header)
//! * 18 -> P0.15   (mid right header)
//! * 19 -> P0.14   (mid right header)
//! * 20 -> P0.13   (mid right header)
//! * 21 -> P0.12   (mid right header)
//!
//! ### Authors
//! * Philip Levis <pal@cs.stanford.edu>
//! * Anderson Lizardo <anderson.lizardo@gmail.com>
//! * Date: August 18, 2016

#![no_std]
#![no_main]
#![feature(panic_implementation)]
#![deny(missing_docs)]

extern crate capsules;
#[allow(unused_imports)]
#[macro_use(debug, debug_verbose, debug_gpio, static_init)]
extern crate kernel;
extern crate cortexm0;
extern crate nrf51;
extern crate nrf5x;

use capsules::alarm::AlarmDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_uart::{UartDevice, UartMux};
use kernel::hil;
use kernel::hil::uart::UART;
use kernel::{Chip, SysTick};
use nrf5x::pinmux::Pinmux;
use nrf5x::rtc::{Rtc, RTC};

/// UART Writer
#[macro_use]
pub mod io;
#[allow(dead_code)]
mod aes_test;

// The nRF51 DK LEDs (see back of board)
const LED1_PIN: usize = 21;
const LED2_PIN: usize = 22;
const LED3_PIN: usize = 23;
const LED4_PIN: usize = 24;

// The nRF51 DK buttons (see back of board)
const BUTTON1_PIN: usize = 17;
const BUTTON2_PIN: usize = 18;
const BUTTON3_PIN: usize = 19;
const BUTTON4_PIN: usize = 20;

// State for loading and holding applications.

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 1;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 8192] = [0; 8192];

static mut PROCESSES: [Option<&'static kernel::procs::Process<'static>>; NUM_PROCS] = [None];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Platform {
    ble_radio: &'static capsules::ble_advertising_driver::BLE<
        'static,
        nrf51::radio::Radio,
        VirtualMuxAlarm<'static, Rtc>,
    >,
    button: &'static capsules::button::Button<'static, nrf5x::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static, UartDevice<'static>>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf5x::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, nrf5x::gpio::GPIOPin>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    alarm: &'static AlarmDriver<'static, VirtualMuxAlarm<'static, Rtc>>,
    rng: &'static capsules::rng::SimpleRng<'static, nrf5x::trng::Trng<'static>>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
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
            _ => f(None),
        }
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf51::init();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // LEDs
    let led_pins = static_init!(
        [(&'static nrf5x::gpio::GPIOPin, capsules::led::ActivationMode); 4],
        [
            (
                &nrf5x::gpio::PORT[LED1_PIN],
                capsules::led::ActivationMode::ActiveLow
            ), // 21
            (
                &nrf5x::gpio::PORT[LED2_PIN],
                capsules::led::ActivationMode::ActiveLow
            ), // 22
            (
                &nrf5x::gpio::PORT[LED3_PIN],
                capsules::led::ActivationMode::ActiveLow
            ), // 23
            (
                &nrf5x::gpio::PORT[LED4_PIN],
                capsules::led::ActivationMode::ActiveLow
            ), // 24
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, nrf5x::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    let button_pins = static_init!(
        [(&'static nrf5x::gpio::GPIOPin, capsules::button::GpioMode); 4],
        [
            (
                &nrf5x::gpio::PORT[BUTTON1_PIN],
                capsules::button::GpioMode::LowWhenPressed
            ), // 17
            (
                &nrf5x::gpio::PORT[BUTTON2_PIN],
                capsules::button::GpioMode::LowWhenPressed
            ), // 18
            (
                &nrf5x::gpio::PORT[BUTTON3_PIN],
                capsules::button::GpioMode::LowWhenPressed
            ), // 19
            (
                &nrf5x::gpio::PORT[BUTTON4_PIN],
                capsules::button::GpioMode::LowWhenPressed
            ), // 20
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, nrf5x::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, board_kernel.create_grant())
    );
    for &(btn, _) in button_pins.iter() {
        use kernel::hil::gpio::PinCtl;
        btn.set_input_mode(kernel::hil::gpio::InputMode::PullUp);
        btn.set_client(button);
    }

    let gpio_pins = static_init!(
        [&'static nrf5x::gpio::GPIOPin; 11],
        [
            &nrf5x::gpio::PORT[1],  // Bottom left header on DK board
            &nrf5x::gpio::PORT[2],  //   |
            &nrf5x::gpio::PORT[3],  //   V
            &nrf5x::gpio::PORT[4],  //
            &nrf5x::gpio::PORT[5],  //
            &nrf5x::gpio::PORT[6],  // -----
            &nrf5x::gpio::PORT[16], //
            &nrf5x::gpio::PORT[15], //
            &nrf5x::gpio::PORT[14], //
            &nrf5x::gpio::PORT[13], //
            &nrf5x::gpio::PORT[12], //
        ]
    );

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&nrf5x::gpio::PORT[LED1_PIN]),
        Some(&nrf5x::gpio::PORT[LED2_PIN]),
        Some(&nrf5x::gpio::PORT[LED3_PIN]),
    );

    let gpio = static_init!(
        capsules::gpio::GPIO<'static, nrf5x::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    nrf51::uart::UART0.initialize(
        Pinmux::new(9),  /*. tx  */
        Pinmux::new(11), /* rx  */
        Pinmux::new(10), /* cts */
        Pinmux::new(8),  /*. rts */
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        UartMux<'static>,
        UartMux::new(
            &nrf51::uart::UART0,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    hil::uart::UART::set_client(&nrf51::uart::UART0, uart_mux);

    // Create a UartDevice for the console.
    let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    console_uart.setup();

    let console = static_init!(
        capsules::console::Console<UartDevice>,
        capsules::console::Console::new(
            console_uart,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant()
        )
    );
    UART::set_client(console_uart, console);
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

    let rtc = &nrf5x::rtc::RTC;
    rtc.start();
    let mux_alarm = static_init!(MuxAlarm<'static, Rtc>, MuxAlarm::new(&RTC));
    rtc.set_client(mux_alarm);

    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, Rtc>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        AlarmDriver<'static, VirtualMuxAlarm<'static, Rtc>>,
        AlarmDriver::new(virtual_alarm1, board_kernel.create_grant())
    );
    virtual_alarm1.set_client(alarm);

    let ble_radio_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, Rtc>,
        VirtualMuxAlarm::new(mux_alarm)
    );

    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(
            &mut nrf5x::temperature::TEMP,
            board_kernel.create_grant()
        )
    );
    kernel::hil::sensors::TemperatureDriver::set_client(&nrf5x::temperature::TEMP, temp);

    let rng = static_init!(
        capsules::rng::SimpleRng<'static, nrf5x::trng::Trng>,
        capsules::rng::SimpleRng::new(&mut nrf5x::trng::TRNG, board_kernel.create_grant())
    );
    nrf5x::trng::TRNG.set_client(rng);

    let ble_radio = static_init!(
        capsules::ble_advertising_driver::BLE<
            'static,
            nrf51::radio::Radio,
            VirtualMuxAlarm<'static, Rtc>,
        >,
        capsules::ble_advertising_driver::BLE::new(
            &mut nrf51::radio::RADIO,
            board_kernel.create_grant(),
            &mut capsules::ble_advertising_driver::BUF,
            ble_radio_virtual_alarm
        )
    );
    kernel::hil::ble_advertising::BleAdvertisementDriver::set_receive_client(
        &nrf51::radio::RADIO,
        ble_radio,
    );
    kernel::hil::ble_advertising::BleAdvertisementDriver::set_transmit_client(
        &nrf51::radio::RADIO,
        ble_radio,
    );
    ble_radio_virtual_alarm.set_client(ble_radio);

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf51::clock::CLOCK.low_stop();
    nrf51::clock::CLOCK.high_stop();

    nrf51::clock::CLOCK.low_set_source(nrf51::clock::LowClockSource::XTAL);
    nrf51::clock::CLOCK.low_start();
    nrf51::clock::CLOCK.high_start();
    while !nrf51::clock::CLOCK.low_started() {}
    while !nrf51::clock::CLOCK.high_started() {}

    let platform = Platform {
        // aes: aes,
        ble_radio: ble_radio,
        button: button,
        console: console,
        gpio: gpio,
        led: led,
        rng: rng,
        alarm: alarm,
        temp: temp,
    };

    rtc.start();

    let mut chip = nrf51::chip::NRF51::new();
    chip.systick().reset();
    chip.systick().enable(true);

    debug!("Initialization complete. Entering main loop");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }
    kernel::procs::load_processes(
        board_kernel,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );

    board_kernel.kernel_loop(
        &platform,
        &mut chip,
        Some(&kernel::ipc::IPC::new(board_kernel)),
    );
}
