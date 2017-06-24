#![no_std]
#![no_main]
#![feature(lang_items,drop_types_in_const,compiler_builtins_lib)]

extern crate cortexm4;
extern crate capsules;
extern crate compiler_builtins;
#[macro_use(debug, static_init)]
extern crate kernel;

// use capsules::timer::TimerDriver;
// use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
// use kernel::{Chip, SysTick};
// use kernel::hil::uart::UART;
use core::fmt::Arguments;


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
}


impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {
        match driver_num {
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {}


#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(_args: Arguments,
                                           _file: &'static str,
                                           _line: usize)
                                           {}
