#![no_std]
#![no_main]
#![feature(lang_items,drop_types_in_const,compiler_builtins_lib)]

extern crate cortexm4;
extern crate capsules;
extern crate compiler_builtins;
#[macro_use(debug, static_init)]
extern crate kernel;
use kernel::{Chip, SysTick};
extern crate nrf52;

use core::fmt::Arguments;

// The nRF52 DK LEDs (see back of board)
const LED1_PIN: usize = 17;
const LED2_PIN: usize = 18;
const LED3_PIN: usize = 19;
const LED4_PIN: usize = 20;

// load user-space processes!!!
#[inline(never)]
#[no_mangle]
unsafe fn load_process() -> &'static mut [Option<kernel::Process<'static>>] {
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    const NUM_PROCS: usize = 1;

    // how should the kernel respond when a process faults
    const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

    #[link_section = ".app_memory"]
    static mut APP_MEMORY: [u8; 8192] = [0; 8192];

    static mut PROCESSES: [Option<kernel::Process<'static>>; NUM_PROCS] = [None];

    let mut apps_in_flash_ptr = &_sapps as *const u8;
    let mut app_memory_ptr = APP_MEMORY.as_mut_ptr();
    let mut app_memory_size = APP_MEMORY.len();
    for i in 0..NUM_PROCS {
        let (process, flash_offset, memory_offset) = kernel::Process::create(apps_in_flash_ptr,
                                                                             app_memory_ptr,
                                                                             app_memory_size,
                                                                             FAULT_RESPONSE);

        if process.is_none() {
            break;
        }

        PROCESSES[i] = process;
        apps_in_flash_ptr = apps_in_flash_ptr.offset(flash_offset as isize);
        app_memory_ptr = app_memory_ptr.offset(memory_offset as isize);
        app_memory_size -= memory_offset;
    }

    &mut PROCESSES
}

pub struct Platform {
    gpio: &'static capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, nrf52::gpio::GPIOPin>,
}


impl kernel::Platform for Platform {
    #[inline(never)]
    #[no_mangle]
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {
        match driver_num {
            1 => f(Some(self.gpio)),
            8 => f(Some(self.led)),
            _ => f(None),
        }
    }
}

// this is called once crt0.s is loaded
#[no_mangle]
pub unsafe fn reset_handler() {
    nrf52::init();

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

    let gpio_pins = static_init!(
        [&'static nrf52::gpio::GPIOPin; 11],
        [&nrf52::gpio::PORT[1],  // Bottom left header on DK board
        &nrf52::gpio::PORT[2],  //   |
        &nrf52::gpio::PORT[3],  //   V
        &nrf52::gpio::PORT[4],  //
        &nrf52::gpio::PORT[5],  //
        &nrf52::gpio::PORT[6],  // -----
        &nrf52::gpio::PORT[16], //
        &nrf52::gpio::PORT[15], //
        &nrf52::gpio::PORT[14], //
        &nrf52::gpio::PORT[13], //
        &nrf52::gpio::PORT[12], //
        ],
        4 * 11);

    let gpio = static_init!(
        capsules::gpio::GPIO<'static, nrf52::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins),
        224/8);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }


    let platform = Platform {
        led: led,
        gpio: gpio,
    };

    let mut chip = nrf52::chip::NRF52::new();
    // guess this enables interrupts etc...
    chip.systick().reset();
    chip.systick().enable(true);
}


#[cfg(not(test))]
#[lang="panic_fmt"]
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(_args: Arguments,
                                           _file: &'static str,
                                           _line: usize) -> ! {
    // FIXME: "!" indicates that the function might never return
    loop {}
}
