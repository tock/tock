//! Tock kernel for the Nordic Semiconductor nRF51 development
//! kit (DK), a.k.a. the PCA10028. </br>
//! This is an nRF51422 SoC (a Cortex M0 core with a BLE transceiver) with many
//! exported pins, LEDs, and buttons. </br>
//! Currently the kernel provides application timers, and GPIO. </br>
//! It will provide a console
//! once the UART is fully implemented and debugged.
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
#![feature(lang_items,drop_types_in_const,compiler_builtins_lib)]

extern crate cortexm0;
extern crate capsules;
extern crate compiler_builtins;
#[macro_use(debug, static_init)]
extern crate kernel;
extern crate nrf51;
extern crate nrf5x;

use capsules::timer::TimerDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::{Chip, SysTick};
use kernel::hil::symmetric_encryption::SymmetricEncryption;
use kernel::hil::uart::UART;
use nrf5x::pinmux::Pinmux;
use nrf5x::rtc::{RTC, Rtc};

#[macro_use]
pub mod io;

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
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 1;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 8192] = [0; 8192];

static mut PROCESSES: [Option<kernel::Process<'static>>; NUM_PROCS] = [None];


pub struct Platform {
    aes: &'static capsules::symmetric_encryption::Crypto<'static, nrf5x::aes::AesECB>,
    ble_radio: &'static nrf5x::ble_advertising_driver::BLE<'static,
                                                           nrf51::radio::Radio,
                                                           VirtualMuxAlarm<'static, Rtc>>,
    button: &'static capsules::button::Button<'static, nrf5x::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static, nrf51::uart::UART>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf5x::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, nrf5x::gpio::GPIOPin>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    timer: &'static TimerDriver<'static, VirtualMuxAlarm<'static, Rtc>>,
    rng: &'static capsules::rng::SimpleRng<'static, nrf5x::trng::Trng<'static>>,
}


impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {
        match driver_num {
            0 => f(Some(self.console)),
            1 => f(Some(self.gpio)),
            3 => f(Some(self.timer)),
            8 => f(Some(self.led)),
            9 => f(Some(self.button)),
            10 => f(Some(self.temp)),
            14 => f(Some(self.rng)),
            17 => f(Some(self.aes)),
            33 => f(Some(self.ble_radio)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    nrf51::init();

    // LEDs
    let led_pins = static_init!(
        [(&'static nrf5x::gpio::GPIOPin, capsules::led::ActivationMode); 4],
        [(&nrf5x::gpio::PORT[LED1_PIN], capsules::led::ActivationMode::ActiveLow), // 21
        (&nrf5x::gpio::PORT[LED2_PIN], capsules::led::ActivationMode::ActiveLow), // 22
        (&nrf5x::gpio::PORT[LED3_PIN], capsules::led::ActivationMode::ActiveLow), // 23
        (&nrf5x::gpio::PORT[LED4_PIN], capsules::led::ActivationMode::ActiveLow), // 24
        ], 256/8);
    let led = static_init!(
        capsules::led::LED<'static, nrf5x::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins),
        64/8);

    let button_pins = static_init!(
        [&'static nrf5x::gpio::GPIOPin; 4],
        [&nrf5x::gpio::PORT[BUTTON1_PIN], // 17
        &nrf5x::gpio::PORT[BUTTON2_PIN], // 18
        &nrf5x::gpio::PORT[BUTTON3_PIN], // 19
        &nrf5x::gpio::PORT[BUTTON4_PIN], // 20
        ],
        4 * 4);
    let button = static_init!(
        capsules::button::Button<'static, nrf5x::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Container::create()),
        96/8);
    for btn in button_pins.iter() {
        use kernel::hil::gpio::PinCtl;
        btn.set_input_mode(kernel::hil::gpio::InputMode::PullUp);
        btn.set_client(button);
    }

    let gpio_pins = static_init!(
        [&'static nrf5x::gpio::GPIOPin; 11],
        [&nrf5x::gpio::PORT[1],  // Bottom left header on DK board
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
        ],
        4 * 11);

    let gpio = static_init!(
        capsules::gpio::GPIO<'static, nrf5x::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins),
        224/8);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    nrf51::uart::UART0.configure(Pinmux::new(9), /*. tx  */
                                 Pinmux::new(11), /* rx  */
                                 Pinmux::new(10), /* cts */
                                 Pinmux::new(8) /*. rts */);
    let console = static_init!(
        capsules::console::Console<nrf51::uart::UART>,
        capsules::console::Console::new(&nrf51::uart::UART0,
                                        115200,
                                        &mut capsules::console::WRITE_BUF,
                                        kernel::Container::create()),
                                        224/8);
    UART::set_client(&nrf51::uart::UART0, console);
    console.initialize();

    // Attach the kernel debug interface to this console
    let kc = static_init!(
        capsules::console::App,
        capsules::console::App::default(),
        480/8);
    kernel::debug::assign_console_driver(Some(console), kc);

    let alarm = &nrf5x::rtc::RTC;
    alarm.start();
    let mux_alarm = static_init!(MuxAlarm<'static, Rtc>, MuxAlarm::new(&RTC), 16);
    alarm.set_client(mux_alarm);


    let virtual_alarm1 = static_init!(
        VirtualMuxAlarm<'static, Rtc>,
        VirtualMuxAlarm::new(mux_alarm),
        24);
    let timer = static_init!(
        TimerDriver<'static, VirtualMuxAlarm<'static, Rtc>>,
        TimerDriver::new(virtual_alarm1,
                         kernel::Container::create()),
                         12);
    virtual_alarm1.set_client(timer);

    let ble_radio_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, Rtc>,
        VirtualMuxAlarm::new(mux_alarm),
        192/8);

    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(&mut nrf5x::temperature::TEMP,
                                                 kernel::Container::create()), 96/8);
    kernel::hil::sensors::TemperatureDriver::set_client(&nrf5x::temperature::TEMP, temp);

    let rng = static_init!(
        capsules::rng::SimpleRng<'static, nrf5x::trng::Trng>,
        capsules::rng::SimpleRng::new(&mut nrf5x::trng::TRNG, kernel::Container::create()),
        96/8);
    nrf5x::trng::TRNG.set_client(rng);

    let aes = static_init!(
        capsules::symmetric_encryption::Crypto<'static, nrf5x::aes::AesECB>,
        capsules::symmetric_encryption::Crypto::new(&mut nrf5x::aes::AESECB,
                                                    kernel::Container::create(),
                                                    &mut capsules::symmetric_encryption::KEY,
                                                    &mut capsules::symmetric_encryption::BUF,
                                                    &mut capsules::symmetric_encryption::IV),
        288/8);
    nrf5x::aes::AESECB.ecb_init();
    SymmetricEncryption::set_client(&nrf5x::aes::AESECB, aes);

    let ble_radio = static_init!(
     nrf5x::ble_advertising_driver::BLE
        <'static, nrf51::radio::Radio, VirtualMuxAlarm<'static, Rtc>>,
     nrf5x::ble_advertising_driver::BLE::new(
         &mut nrf51::radio::RADIO,
         kernel::Container::create(),
         &mut nrf5x::ble_advertising_driver::BUF,
         ble_radio_virtual_alarm),
        256/8);
    nrf5x::ble_advertising_hil::BleAdvertisementDriver::set_client(&nrf51::radio::RADIO, ble_radio);
    ble_radio_virtual_alarm.set_client(ble_radio);

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf5x::clock::CLOCK.low_stop();
    nrf5x::clock::CLOCK.high_stop();

    nrf5x::clock::CLOCK.low_set_source(nrf5x::clock::LowClockSource::XTAL);
    nrf5x::clock::CLOCK.low_start();
    nrf5x::clock::CLOCK.high_start();
    while !nrf5x::clock::CLOCK.low_started() {}
    while !nrf5x::clock::CLOCK.high_started() {}

    let platform = Platform {
        aes: aes,
        ble_radio: ble_radio,
        button: button,
        console: console,
        gpio: gpio,
        led: led,
        rng: rng,
        timer: timer,
        temp: temp,
    };

    alarm.start();

    let mut chip = nrf51::chip::NRF51::new();
    chip.systick().reset();
    chip.systick().enable(true);

    debug!("Initialization complete. Entering main loop");
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }
    kernel::process::load_processes(&_sapps as *const u8,
                                    &mut APP_MEMORY,
                                    &mut PROCESSES,
                                    FAULT_RESPONSE);
    kernel::main(&platform,
                 &mut chip,
                 &mut PROCESSES,
                 &kernel::ipc::IPC::new());
}
