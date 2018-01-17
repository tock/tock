#![no_std]
#![no_main]
#![feature(lang_items, compiler_builtins_lib)]

extern crate capsules;
extern crate compiler_builtins;

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init)]
extern crate kernel;

extern crate cc2650;

use core::fmt::{Arguments};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 2;
//
static mut PROCESSES: [Option<kernel::Process<'static>>; NUM_PROCS] = [None, None];

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 20480] = [0; 20480];

pub struct Platform {
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            // Todo, add drivers here
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    // IOCFGn = n*4 hex
    let iocbase = 0x40081000;
    let iocfg10 = iocbase + 0x28;

    let gpiobase = 0x40022000;
    let doe = gpiobase + 0xD0;
    //let dio8to10 = gpiobase + 0x08;

    // Set DIO10 to output
    *(iocfg10 as *mut u16) = 0x7000;
    // Set DataEnable to 1
    *(doe as *mut u32) = 0x400;
    *((gpiobase + 0x00000090) as *mut u32) = 1 << 10;
    loop { }

    let platform = Platform { };
    let mut chip = cc2650::chip::cc2650::new();

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
        &platform,
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
