//! Tock kernel for the Nordic Semiconductor nRF51 development
//! kit (DK), a.k.a. the PCA10028. This is an nRF51422 SoC (a
//! Cortex M0 core with a BLE transciver) with many exported
//! pins, LEDs, and buttons. Currently the kernel provides
//! application timers, and GPIO. It will provide a console
//! once the UART is fully implemented and debugged. The
//! application GPIO pins are:
//!
//!   0 -> LED1 (pin 21)
//!   1 -> LED2 (pin 22)
//!   2 -> LED3 (pin 23)
//!   3 -> LED4 (pin 24)
//!   5 -> BUTTON1 (pin 17)
//!   6 -> BUTTON2 (pin 18)
//!   7 -> BUTTON3 (pin 19)
//!   8 -> BUTTON4 (pin 20)
//!   9 -> P0.01   (bottom left header)
//!  10 -> P0.02   (bottom left header)
//!  11 -> P0.03   (bottom left header)
//!  12 -> P0.04   (bottom left header)
//!  12 -> P0.05   (bottom left header)
//!  13 -> P0.06   (bottom left header)
//!  14 -> P0.19   (mid right header)
//!  15 -> P0.18   (mid right header)
//!  16 -> P0.17   (mid right header)
//!  17 -> P0.16   (mid right header)
//!  18 -> P0.15   (mid right header)
//!  19 -> P0.14   (mid right header)
//!  20 -> P0.13   (mid right header)
//!  21 -> P0.12   (mid right header)
//!
//!  Author: Philip Levis <pal@cs.stanford.edu>
//!  Author: Anderson Lizardo <anderson.lizardo@gmail.com>
//!  Date: August 18, 2016

#![no_std]
#![no_main]
#![feature(lang_items)]

#[macro_use]
pub mod io;

extern crate cortexm0;
extern crate capsules;
#[macro_use(static_init)]
extern crate kernel;
extern crate nrf51;

use capsules::timer::TimerDriver;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::{Chip, SysTick};
use kernel::hil::uart::UART;
use nrf51::rtc::{RTC, Rtc};

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

unsafe fn load_process() -> &'static mut [Option<kernel::process::Process<'static>>] {
    use core::ptr::{read_volatile, write_volatile};
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }


    #[link_section = ".app_memory"]
    static mut MEMORY: [u8; 8192] = [0; 8192];
    static mut PROCS: [Option<kernel::process::Process<'static>>; 1] = [None];

    let addr = &_sapps as *const u8;

    // The first member of the LoadInfo header contains the total size of
    // each process image. A sentinel value of 0 (invalid because it is
    // smaller than the header itself) is used to mark the end of the list
    // of processes.
    let total_size = read_volatile(addr as *const usize);
    if total_size != 0 {
        write_volatile(&mut PROCS[0],
                       Some(kernel::process::Process::create(addr, total_size, &mut MEMORY)));
    }
    &mut PROCS
}

pub struct Platform {
    gpio: &'static capsules::gpio::GPIO<'static, nrf51::gpio::GPIOPin>,
    timer: &'static TimerDriver<'static, VirtualMuxAlarm<'static, Rtc>>,
    console: &'static capsules::console::Console<'static, nrf51::uart::UART>,
    led: &'static capsules::led::LED<'static, nrf51::gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, nrf51::gpio::GPIOPin>,
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
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    nrf51::init();

    // LEDs
    let led_pins = static_init!(
        [&'static nrf51::gpio::GPIOPin; 4],
        [&nrf51::gpio::PORT[LED1_PIN], // 21
         &nrf51::gpio::PORT[LED2_PIN], // 22
         &nrf51::gpio::PORT[LED3_PIN], // 23
         &nrf51::gpio::PORT[LED4_PIN], // 24
        ],
        4 * 4);
    let led = static_init!(
        capsules::led::LED<'static, nrf51::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins, capsules::led::ActivationMode::ActiveLow),
        96/8);

    let button_pins = static_init!(
        [&'static nrf51::gpio::GPIOPin; 4],
        [&nrf51::gpio::PORT[BUTTON1_PIN], // 17
         &nrf51::gpio::PORT[BUTTON2_PIN], // 18
         &nrf51::gpio::PORT[BUTTON3_PIN], // 19
         &nrf51::gpio::PORT[BUTTON4_PIN], // 20
        ],
        4 * 4);
    let button = static_init!(
        capsules::button::Button<'static, nrf51::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Container::create()),
        96/8);
    for btn in button_pins.iter() {
        use kernel::hil::gpio::PinCtl;
        btn.set_input_mode(kernel::hil::gpio::InputMode::PullUp);
        btn.set_client(button);
    }

    let gpio_pins = static_init!(
        [&'static nrf51::gpio::GPIOPin; 11],
        [&nrf51::gpio::PORT[1],  // Bottom left header on DK board
         &nrf51::gpio::PORT[2],  //   |
         &nrf51::gpio::PORT[3],  //   V
         &nrf51::gpio::PORT[4],  //
         &nrf51::gpio::PORT[5],  //
         &nrf51::gpio::PORT[6],  // -----
         &nrf51::gpio::PORT[16], //
         &nrf51::gpio::PORT[15], //
         &nrf51::gpio::PORT[14], //
         &nrf51::gpio::PORT[13], //
         &nrf51::gpio::PORT[12], //
        ],
        4 * 11);

    let gpio = static_init!(
        capsules::gpio::GPIO<'static, nrf51::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins),
        20);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let console = static_init!(
        capsules::console::Console<nrf51::uart::UART>,
        capsules::console::Console::new(&nrf51::uart::UART0,
                                        115200,
                                        &mut capsules::console::WRITE_BUF,
                                        kernel::Container::create()),
        224/8);
    UART::set_client(&nrf51::uart::UART0, console);
    console.initialize();

    let alarm = &nrf51::rtc::RTC;
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
        gpio: gpio,
        timer: timer,
        console: console,
        led: led,
        button: button,
    };

    alarm.start();

    let mut chip = nrf51::chip::NRF51::new();
    chip.systick().reset();
    chip.systick().enable(true);

    kernel::main(&platform,
                 &mut chip,
                 load_process(),
                 &kernel::ipc::IPC::new());

}


use core::fmt::Arguments;
#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(_args: &Arguments,
                                           _file: &'static str,
                                           _line: usize)
                                           -> ! {
    use kernel::hil::gpio::Pin;

    let led0 = &nrf51::gpio::PORT[LED1_PIN];
    let led1 = &nrf51::gpio::PORT[LED2_PIN];

    led0.make_output();
    led1.make_output();
    loop {
        for _ in 0..100000 {
            led0.set();
            led1.set();
        }
        for _ in 0..100000 {
            led0.clear();
            led1.clear();
        }
    }
}
