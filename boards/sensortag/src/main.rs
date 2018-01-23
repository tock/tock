#![no_std]
#![no_main]
#![feature(lang_items, compiler_builtins_lib, asm)]

extern crate capsules;
extern crate compiler_builtins;

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init)]
extern crate kernel;
extern crate cc2650;

use core::fmt::{Arguments};

// Only used for testing gpio driver
use kernel::common::VolatileCell;
use cc2650::peripheral_registers::{PRCM, PRCM_BASE};
use kernel::hil::gpio::Pin;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 2;
//
static mut PROCESSES: [Option<kernel::Process<'static>>; NUM_PROCS] = [None, None];

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 10240] = [0; 10240];

unsafe fn delay() {
    for _i in 0..0x2FFFFF {
        asm!("nop;");
    }
}

#[repr(C)]
pub struct DEBUG {
    pub val: VolatileCell<u32>,
}

pub struct Platform {
    gpio: &'static capsules::gpio::GPIO<'static, cc2650::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, cc2650::gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, cc2650::gpio::GPIOPin>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    let prcm = &*(PRCM_BASE as *const PRCM);

    // PERIPH power domain on
    prcm.pd_ctl0.set(4);

    // Wait until peripheral power is on
    while (prcm.pd_stat0_periph.get() & 1) != 1 { }

    // Enable GPIO clocks
    prcm.gpio_clk_gate_run.set(1);
    prcm.clk_load_ctl.set(1);

    // LEDs
    let led_pins = static_init!(
        [(&'static cc2650::gpio::GPIOPin, capsules::led::ActivationMode); 2],
        [
            (
                &cc2650::gpio::PORT[10],
                capsules::led::ActivationMode::ActiveLow
            ), // Red
            (
                &cc2650::gpio::PORT[15],
                capsules::led::ActivationMode::ActiveLow
            ), // Green
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, cc2650::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONs
    let button_pins = static_init!(
        [(&'static cc2650::gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (
                &cc2650::gpio::PORT[0],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 2
            (
                &cc2650::gpio::PORT[4],
                capsules::button::GpioMode::LowWhenPressed
            ) // Button 1
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, cc2650::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Grant::create())
    );
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static cc2650::gpio::GPIOPin; 28],
        [
            &cc2650::gpio::PORT[1],
            &cc2650::gpio::PORT[2],
            &cc2650::gpio::PORT[3],
            &cc2650::gpio::PORT[5],
            &cc2650::gpio::PORT[6],
            &cc2650::gpio::PORT[7],
            &cc2650::gpio::PORT[8],
            &cc2650::gpio::PORT[9],
            &cc2650::gpio::PORT[11],
            &cc2650::gpio::PORT[12],
            &cc2650::gpio::PORT[13],
            &cc2650::gpio::PORT[14],
            &cc2650::gpio::PORT[16],
            &cc2650::gpio::PORT[17],
            &cc2650::gpio::PORT[18],
            &cc2650::gpio::PORT[19],
            &cc2650::gpio::PORT[20],
            &cc2650::gpio::PORT[21],
            &cc2650::gpio::PORT[22],
            &cc2650::gpio::PORT[23],
            &cc2650::gpio::PORT[24],
            &cc2650::gpio::PORT[25],
            &cc2650::gpio::PORT[26],
            &cc2650::gpio::PORT[27],
            &cc2650::gpio::PORT[28],
            &cc2650::gpio::PORT[29],
            &cc2650::gpio::PORT[30],
            &cc2650::gpio::PORT[31]
    ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, cc2650::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let pin: cc2650::gpio::GPIOPin = cc2650::gpio::GPIOPin::new(15);
    pin.make_output();

    loop {
        pin.toggle();
        delay();
    }

    let sensortag = Platform {
        gpio,
        led,
        button
    };

    let mut chip = cc2650::chip::Cc2650::new();

    debug!("Initialization complete. Entering main loop\r");
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
        &sensortag,
        &mut chip,
        &mut PROCESSES,
        &kernel::ipc::IPC::new(),
    );
}

#[cfg(not(test))]
#[no_mangle]
#[lang = "panic_fmt"]
pub unsafe extern "C" fn panic_fmt(_args: Arguments, _file: &'static str, _line: u32) -> ! {
    loop { }
}
