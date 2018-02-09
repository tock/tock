#![no_std]
#![no_main]
#![feature(lang_items, compiler_builtins_lib, asm)]

extern crate capsules;
extern crate compiler_builtins;

extern crate cc26xx;
extern crate cc26x2;

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init)]
extern crate kernel;

use cc26xx::prcm;
use cc26xx::aon;

#[macro_use]
pub mod io;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 2;
//
static mut PROCESSES: [Option<kernel::Process<'static>>; NUM_PROCS] = [None, None];

#[link_section = ".app_memory"]
// Give half of RAM to be dedicated APP memory
static mut APP_MEMORY: [u8; 0xA000] = [0; 0xA000];

pub struct Platform {
    gpio: &'static capsules::gpio::GPIO<'static, cc26xx::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, cc26xx::gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, cc26xx::gpio::GPIOPin>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    cc26x2::init();

    // Setup AON event defaults
    aon::AON_EVENT.setup();

    // Power on peripherals (eg. GPIO)
    prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);

    // Wait for it to turn on until we continue
    while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}

    // Enable the GPIO clocks
    prcm::Clock::enable_gpio();

    // LEDs
    let led_pins = static_init!(
        [(
            &'static cc26xx::gpio::GPIOPin,
            capsules::led::ActivationMode
        ); 2],
        [
            (
                &cc26xx::gpio::PORT[6],
                capsules::led::ActivationMode::ActiveHigh
            ), // Red
            (
                &cc26xx::gpio::PORT[7],
                capsules::led::ActivationMode::ActiveHigh
            ) // Green
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, cc26xx::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONs
    let button_pins = static_init!(
        [(&'static cc26xx::gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (
                &cc26xx::gpio::PORT[13],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 2
            (
                &cc26xx::gpio::PORT[14],
                capsules::button::GpioMode::LowWhenPressed
            ) // Button 1
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, cc26xx::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Grant::create())
    );
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static cc26xx::gpio::GPIOPin; 22],
        [
            &cc26xx::gpio::PORT[1],
            &cc26xx::gpio::PORT[5],
            &cc26xx::gpio::PORT[8],
            &cc26xx::gpio::PORT[9],
            &cc26xx::gpio::PORT[10],
            &cc26xx::gpio::PORT[11],
            &cc26xx::gpio::PORT[12],
            &cc26xx::gpio::PORT[15],
            &cc26xx::gpio::PORT[16],
            &cc26xx::gpio::PORT[17],
            &cc26xx::gpio::PORT[18],
            &cc26xx::gpio::PORT[19],
            &cc26xx::gpio::PORT[20],
            &cc26xx::gpio::PORT[21],
            &cc26xx::gpio::PORT[22],
            &cc26xx::gpio::PORT[23],
            &cc26xx::gpio::PORT[24],
            &cc26xx::gpio::PORT[25],
            &cc26xx::gpio::PORT[26],
            &cc26xx::gpio::PORT[27],
            &cc26xx::gpio::PORT[30],
            &cc26xx::gpio::PORT[31]
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, cc26xx::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let launchxl = Platform {
        gpio,
        led,
        button,
    };

    let mut chip = cc26x2::chip::Cc26X2::new();

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    kernel::process::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );

    kernel::main(
        &launchxl,
        &mut chip,
        &mut PROCESSES,
        &kernel::ipc::IPC::new(),
    );
}
