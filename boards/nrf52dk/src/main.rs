//! Tock kernel for the Nordic Semiconductor nRF52 development kit (DK), a.k.a. the PCA10040. </br>
//! It is based on nRF52838 SoC (Cortex M4 core with a BLE transceiver) with many exported
//! I/O and peripherals.
//!
//! nRF52838 has only one port and uses pins 0-31!
//!
//! Furthermore, there exist another a preview development kit for nRF52840 but it is not supported
//! yet because unfortunately the pin configuration differ from nRF52-DK whereas nRF52840 uses two
//! ports where port 0 has 32 pins and port 1 has 16 pins.
//!
//! Pin Configuration
//! -------------------
//!
//! ### `GPIOs`
//! * P0.27 -> (top left header)
//! * P0.26 -> (top left header)
//! * P0.02 -> (top left header)
//! * P0.25 -> (top left header)
//! * P0.24 -> (top left header)
//! * P0.23 -> (top left header)
//! * P0.22 -> (top left header)
//! * P0.12 -> (top mid header)
//! * P0.11 -> (top mid header)
//! * P0.01 -> (top right header)
//! * P0.00 -> (top right header)
//! * P0.03 -> (bottom right header)
//! * P0.04 -> (bottom right header)
//! * P0.28 -> (bottom right header)
//! * P0.29 -> (bottom right header)
//! * P0.30 -> (bottom right header)
//! * P0.31 -> (bottom right header)
//!
//! ### `LEDs`
//! * P0.17 -> LED1
//! * P0.18 -> LED2
//! * P0.19 -> LED3
//! * P0.20 -> LED4
//!
//! ### `Buttons`
//! * P0.13 -> Button1
//! * P0.14 -> Button2
//! * P0.15 -> Button3
//! * P0.16 -> Button4
//! * P0.21 -> Reset Button
//!
//! ### `UART`
//! * P0.05 -> RTS
//! * P0.06 -> TXD
//! * P0.07 -> CTS
//! * P0.08 -> RXD
//!
//! ### `NFC`
//! * P0.09 -> NFC1
//! * P0.10 -> NFC2
//!
//! Author
//! -------------------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * July 16, 2017

#![no_std]
#![no_main]
#![feature(lang_items,drop_types_in_const,compiler_builtins_lib)]

extern crate cortexm4;
extern crate capsules;
extern crate compiler_builtins;
#[macro_use(debug, static_init)]
extern crate kernel;
extern crate nrf52;
extern crate nrf5x;

use capsules::virtual_alarm::VirtualMuxAlarm;
use nrf5x::rtc::Rtc;

// The nRF52 DK LEDs (see back of board)
const LED1_PIN: usize = 17;
const LED2_PIN: usize = 18;
const LED3_PIN: usize = 19;
const LED4_PIN: usize = 20;

// The nRF52 DK buttons (see back of board)
const BUTTON1_PIN: usize = 13;
const BUTTON2_PIN: usize = 14;
const BUTTON3_PIN: usize = 15;
const BUTTON4_PIN: usize = 16;
const BUTTON_RST_PIN: usize = 21;

#[macro_use]
pub mod io;


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
    ble_radio: &'static nrf5x::ble_advertising_driver::BLE
        <'static, nrf52::radio::Radio, VirtualMuxAlarm<'static, Rtc>>,
    button: &'static capsules::button::Button<'static, nrf5x::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static, nrf52::uart::UARTE>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf5x::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, nrf5x::gpio::GPIOPin>,
    rng: &'static capsules::rng::SimpleRng<'static, nrf5x::trng::Trng<'static>>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    timer: &'static capsules::timer::TimerDriver
        <'static, capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>>,
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

// this is called once crt0.s is loaded
#[no_mangle]
pub unsafe fn reset_handler() {
    nrf52::init();

    // make non-volatile memory writable and activate the reset button (pin 21)
    let nvmc = nrf52::nvmc::NVMC::new();
    let uicr = nrf52::uicr::UICR::new();
    nvmc.configure_writeable();
    while !nvmc.is_ready() {}
    uicr.set_psel0_reset_pin(BUTTON_RST_PIN);
    while !nvmc.is_ready() {}
    uicr.set_psel1_reset_pin(BUTTON_RST_PIN);

    // GPIOs
    // FIXME: Test if it works and remove un-commented code!
    let gpio_pins = static_init!(
        [&'static nrf5x::gpio::GPIOPin; 15],
        [&nrf5x::gpio::PORT[3],  // Bottom left header on DK board
        &nrf5x::gpio::PORT[4],   //
        &nrf5x::gpio::PORT[28],  //
        &nrf5x::gpio::PORT[29],  //
        &nrf5x::gpio::PORT[30],  //
        &nrf5x::gpio::PORT[31],  // -----
        &nrf5x::gpio::PORT[10],  // Top right header on DK board
        &nrf5x::gpio::PORT[9],   //
        &nrf5x::gpio::PORT[8],   //
        &nrf5x::gpio::PORT[7],   //
        &nrf5x::gpio::PORT[6],   //
        &nrf5x::gpio::PORT[5],   //
        &nrf5x::gpio::PORT[21],  //
        &nrf5x::gpio::PORT[1],   //
        &nrf5x::gpio::PORT[0],   // -----
        /*&nrf52::gpio::PORT[18],  // Top mid header on DK board
        &nrf52::gpio::PORT[17],  //
        &nrf52::gpio::PORT[16],  //
        &nrf52::gpio::PORT[15],  //
        &nrf52::gpio::PORT[14],  //
        &nrf52::gpio::PORT[13],  //
        &nrf52::gpio::PORT[12],  //
        &nrf52::gpio::PORT[11],  // ----
        &nrf52::gpio::PORT[27],  // Top left header on DK board
        &nrf52::gpio::PORT[26],  //
        &nrf52::gpio::PORT[2],  //
        &nrf52::gpio::PORT[25],  //
        &nrf52::gpio::PORT[24],  //
        &nrf52::gpio::PORT[23],  //
        &nrf52::gpio::PORT[22],  //
        &nrf52::gpio::PORT[20],  //
        &nrf52::gpio::PORT[19],  // ----*/
        ],
        4 * 11);

    let gpio = static_init!(
        capsules::gpio::GPIO<'static, nrf5x::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins),
        224/8);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    // LEDs
    let led_pins = static_init!(
        [(&'static nrf5x::gpio::GPIOPin, capsules::led::ActivationMode); 4],
        [(&nrf5x::gpio::PORT[LED1_PIN], capsules::led::ActivationMode::ActiveLow),
        (&nrf5x::gpio::PORT[LED2_PIN], capsules::led::ActivationMode::ActiveLow),
        (&nrf5x::gpio::PORT[LED3_PIN], capsules::led::ActivationMode::ActiveLow),
        (&nrf5x::gpio::PORT[LED4_PIN], capsules::led::ActivationMode::ActiveLow),
        ], 256/8);

    let led = static_init!(
        capsules::led::LED<'static, nrf5x::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins),
        64/8);

    let button_pins = static_init!(
        [&'static nrf5x::gpio::GPIOPin; 4],
        [&nrf5x::gpio::PORT[BUTTON1_PIN], // 13
        &nrf5x::gpio::PORT[BUTTON2_PIN],  // 14
        &nrf5x::gpio::PORT[BUTTON3_PIN],  // 15
        &nrf5x::gpio::PORT[BUTTON4_PIN],  // 16
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

    let alarm = &nrf5x::rtc::RTC;
    alarm.start();
    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, nrf5x::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&nrf5x::rtc::RTC), 16);
    alarm.set_client(mux_alarm);


    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm),
        24);
    let timer = static_init!(
        capsules::timer::TimerDriver<'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>>,
        capsules::timer::TimerDriver::new(virtual_alarm1,
                         kernel::Container::create()),
                         12);
    virtual_alarm1.set_client(timer);
    let ble_radio_virtual_alarm = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm),
        192/8);

    nrf52::uart::UART0.configure(nrf5x::pinmux::Pinmux::new(6), // tx
                                 nrf5x::pinmux::Pinmux::new(8), // rx
                                 nrf5x::pinmux::Pinmux::new(7), // cts
                                 nrf5x::pinmux::Pinmux::new(5)); // rts
    let console = static_init!(
        capsules::console::Console<nrf52::uart::UARTE>,
        capsules::console::Console::new(&nrf52::uart::UART0,
                                        115200,
                                        &mut capsules::console::WRITE_BUF,
                                        kernel::Container::create()),
                                        224/8);
    kernel::hil::uart::UART::set_client(&nrf52::uart::UART0, console);
    console.initialize();

    // Attach the kernel debug interface to this console
    let kc = static_init!(
        capsules::console::App,
        capsules::console::App::default(),
        480/8);
    kernel::debug::assign_console_driver(Some(console), kc);


    let ble_radio = static_init!(
     nrf5x::ble_advertising_driver::BLE
        <'static, nrf52::radio::Radio, VirtualMuxAlarm<'static, Rtc>>,
     nrf5x::ble_advertising_driver::BLE::new(
         &mut nrf52::radio::RADIO,
         kernel::Container::create(),
         &mut nrf5x::ble_advertising_driver::BUF,
         ble_radio_virtual_alarm),
        256/8);
    nrf5x::ble_advertising_hil::BleAdvertisementDriver::set_client(&nrf52::radio::RADIO, ble_radio);
    ble_radio_virtual_alarm.set_client(ble_radio);


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
    kernel::hil::symmetric_encryption::SymmetricEncryption::set_client(&nrf5x::aes::AESECB, aes);

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
        button: button,
        ble_radio: ble_radio,
        console: console,
        led: led,
        gpio: gpio,
        rng: rng,
        temp: temp,
        timer: timer,
    };

    let mut chip = nrf52::chip::NRF52::new();

    debug!("Initialization complete. Entering main loop\r");
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
