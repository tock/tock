//! Tock kernel for the Nordic Semiconductor nRF52 development kit (DK), a.k.a. the PCA10040.
//! It is based on nRF52838 SoC (Cortex M4 core with a BLE transceiver) with many exported
//! I/O and peripherals.
//!
//! nRF52838 has only one port and uses pins 0-31!
//!
//! Furthermore, there exist another a preview development kit for nRF52840 but it is not supported
//! yet because unfortunately the pin configuration differ from nRF52-DK whereas nRF52840 uses two
//! ports where port 0 has 32 pins and port 1 has 16 pins.
//!
//!
//!   GPIO:
//!     P0.27 -> (top left header)
//!     P0.26 -> (top left header)
//!     P0.02 -> (top left header)
//!     P0.25 -> (top left header)
//!     P0.24 -> (top left header)
//!     P0.23 -> (top left header)
//!     P0.22 -> (top left header)
//!     P0.20 -> (top left header)
//!     P0.19 -> (top left header)
//!     P0.18 -> (top mid header)
//!     P0.17 -> (top mid header)
//!     P0.16 -> (top mid header)
//!     P0.15 -> (top mid header)
//!     P0.14 -> (top mid header)
//!     P0.13 -> (top mid header)
//!     P0.12 -> (top mid header)
//!     P0.11 -> (top mid header)
//!     P0.10 -> (top right header)
//!     P0.09 -> (top right header)
//!     P0.08 -> (top right header)
//!     P0.07 -> (top right header)
//!     P0.06 -> (top right header)
//!     P0.05 -> (top right header)
//!     P0.21 -> (top right header)
//!     P0.01 -> (top right header)
//!     P0.00 -> (top right header)
//!     P0.03 -> (bottom right header)
//!     P0.04 -> (bottom right header)
//!     P0.28 -> (bottom right header)
//!     P0.29 -> (bottom right header)
//!     P0.30 -> (bottom right header)
//!     P0.31 -> (bottom right header)
//!
//!   LEDs:
//!     P0.17 -> LED1
//!     P0.18 -> LED2
//!     P0.19 -> LED3
//!     P0.20 -> LED4
//!
//!   Buttons:
//!     P0.13 -> Button1
//!     P0.14 -> Button2
//!     P0.15 -> Button3
//!     P0.16 -> Button4
//!     P0.21 -> Reset Button
//!
//!   UART:
//!     P0.05 -> RTS
//!     P0.06 -> TXD
//!     P0.07 -> CTS
//!     P0.08 -> RXD
//!
//!   NFC:
//!     P0.09 -> NFC1
//!     P0.10 -> NFC2
//!
//!  Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//!  Date: July 16, 2017

#![no_std]
#![no_main]
#![feature(lang_items,drop_types_in_const,compiler_builtins_lib)]

extern crate cortexm4;
extern crate capsules;
extern crate compiler_builtins;
#[macro_use(debug, static_init)]
extern crate kernel;
extern crate nrf52;

use kernel::{Chip, SysTick};

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
    console: &'static capsules::console::Console<'static, nrf52::uart::UART>,
    button: &'static capsules::button::Button<'static, nrf52::gpio::GPIOPin>,
    gpio: &'static capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, nrf52::gpio::GPIOPin>,
    timer: &'static capsules::timer::TimerDriver
        <'static, capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc>>,
}


impl kernel::Platform for Platform {
    #[inline(never)]
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {
        match driver_num {
            0 => f(Some(self.console)),
            1 => f(Some(self.gpio)),
            3 => f(Some(self.timer)),
            8 => f(Some(self.led)),
            9 => f(Some(self.button)),
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
        [&'static nrf52::gpio::GPIOPin; 15],
        [&nrf52::gpio::PORT[3],  // Bottom left header on DK board
        &nrf52::gpio::PORT[4],   //
        &nrf52::gpio::PORT[28],  //
        &nrf52::gpio::PORT[29],  //
        &nrf52::gpio::PORT[30],  //
        &nrf52::gpio::PORT[31],  // -----
        &nrf52::gpio::PORT[10],  // Top right header on DK board
        &nrf52::gpio::PORT[9],   //
        &nrf52::gpio::PORT[8],   //
        &nrf52::gpio::PORT[7],   //
        &nrf52::gpio::PORT[6],   //
        &nrf52::gpio::PORT[5],   //
        &nrf52::gpio::PORT[21],  //
        &nrf52::gpio::PORT[1],   //
        &nrf52::gpio::PORT[0],   // -----
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
        capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins),
        224/8);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    // LEDs
    let led_pins = static_init!(
        [(&'static nrf52::gpio::GPIOPin, capsules::led::ActivationMode); 4],
        [(&nrf52::gpio::PORT[LED1_PIN], capsules::led::ActivationMode::ActiveLow),
        (&nrf52::gpio::PORT[LED2_PIN], capsules::led::ActivationMode::ActiveLow),
        (&nrf52::gpio::PORT[LED3_PIN], capsules::led::ActivationMode::ActiveLow),
        (&nrf52::gpio::PORT[LED4_PIN], capsules::led::ActivationMode::ActiveLow),
        ], 256/8);

    let led = static_init!(
        capsules::led::LED<'static, nrf52::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins),
        64/8);

    let button_pins = static_init!(
        [&'static nrf52::gpio::GPIOPin; 4],
        [&nrf52::gpio::PORT[BUTTON1_PIN], // 13
        &nrf52::gpio::PORT[BUTTON2_PIN],  // 14
        &nrf52::gpio::PORT[BUTTON3_PIN],  // 15
        &nrf52::gpio::PORT[BUTTON4_PIN],  // 16
        ],
        4 * 4);
    let button = static_init!(
        capsules::button::Button<'static, nrf52::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Container::create()),
        96/8);
    for btn in button_pins.iter() {
        use kernel::hil::gpio::PinCtl;
        btn.set_input_mode(kernel::hil::gpio::InputMode::PullUp);
        btn.set_client(button);
    }

    let alarm = &nrf52::rtc::RTC;
    alarm.start();
    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, nrf52::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&nrf52::rtc::RTC), 16);
    alarm.set_client(mux_alarm);


    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm),
        24);
    let timer = static_init!(
        capsules::timer::TimerDriver<'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc>>,
        capsules::timer::TimerDriver::new(virtual_alarm1,
                         kernel::Container::create()),
                         12);
    virtual_alarm1.set_client(timer);

    nrf52::uart::UART0.configure(nrf52::pinmux::Pinmux::new(6), // tx
                                 nrf52::pinmux::Pinmux::new(8), // rx
                                 nrf52::pinmux::Pinmux::new(7), // cts
                                 nrf52::pinmux::Pinmux::new(5)); // rts
    let console = static_init!(
        capsules::console::Console<nrf52::uart::UART>,
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

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52::clock::CLOCK.low_stop();
    nrf52::clock::CLOCK.high_stop();

    nrf52::clock::CLOCK.low_set_source(nrf52::clock::LowClockSource::XTAL);
    nrf52::clock::CLOCK.low_start();
    nrf52::clock::CLOCK.high_start();
    while !nrf52::clock::CLOCK.low_started() {}
    while !nrf52::clock::CLOCK.high_started() {}


    let platform = Platform {
        button: button,
        console: console,
        led: led,
        gpio: gpio,
        timer: timer,
    };

    let mut chip = nrf52::chip::NRF52::new();
    chip.systick().reset();
    chip.systick().enable(true);


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
