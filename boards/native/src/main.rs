//! "Board" file for native process execution.

#![no_std]
#![no_main]
#![feature(asm, const_fn, lang_items)]
#![feature(panic_implementation)]
extern crate capsules;
#[allow(unused_imports)]
#[macro_use(debug, static_init)]
extern crate kernel;
extern crate tock_native_chip;

use kernel::Platform;

#[macro_use]
pub mod io;

// State for loading and holding applications.

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 10240] = [0; 10240];

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static mut kernel::procs::Process<'static>>; NUM_PROCS] =
    [None, None, None, None];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct NativeProcess {
    //console: &'static capsules::console::Console<'static, tock_native_chip::uart::UART>,
    //gpio: &'static capsules::gpio::GPIO<'static, tock_native_chip::gpio::GPIOPin>,
    ipc: kernel::ipc::IPC,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for NativeProcess {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            //capsules::console::DRIVER_NUM => f(Some(self.console)),
            //capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Expected entry function (linux)
#[cfg(target_os = "linux")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe { reset_handler(); }
    loop {}
}

/// Expected entry function (mac)
#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn main() -> ! {
    unsafe { reset_handler(); }
    loop {}
}

/// Reset Handler
#[no_mangle]
pub unsafe fn reset_handler() {
    tock_native_chip::init();

    //let console = static_init!(
    //    capsules::console::Console<tock_native_chip::uart::UART>,
    //    capsules::console::Console::new(
    //        &tm4c129x::uart::UART0,
    //        115200,
    //        &mut capsules::console::WRITE_BUF,
    //        &mut capsules::console::READ_BUF,
    //        kernel::Grant::create()
    //    )
    //);
    //hil::uart::UART::set_client(&tock_native_chip::uart::UART0, console);

    let native = NativeProcess {
        //console: console,
        //gpio: gpio,
        ipc: kernel::ipc::IPC::new(),
    };

    let mut chip = tock_native_chip::chip::NativeChip::new();

    //tock_native_chip.console.initialize();

    // Attach the kernel debug interface to this console
    //let kc = static_init!(capsules::console::App, capsules::console::App::default());
    //kernel::debug::assign_console_driver(Some(tock_native_chip.console), kc);

    debug!("Initialization complete. Entering main loop...\r");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }
    kernel::procs::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );
    kernel::kernel_loop(&native, &mut chip, &mut PROCESSES, Some(&native.ipc));
}

