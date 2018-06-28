//! "Board" file for native process execution.

#![feature(asm, const_fn, lang_items)]
#![feature(core_intrinsics)] // for breakpoint()
#![feature(panic_implementation)]
extern crate capsules;
#[allow(unused_imports)]
#[macro_use(debug, static_init)]
extern crate kernel;
extern crate tock_native_chip;

// Implicit in no_std environments, but native w/ std needs it
extern crate core;

// Rust native crates
use std::fs::File;
use std::io;
use std::panic;

// crates.io crates
extern crate memmap;
use memmap::Mmap;

// Tock crates
use kernel::hil;
use kernel::Platform;

pub mod native_panic;

// State for loading and holding applications.

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
//#[link_section = ".app_memory"]
//TODO: files? maps?
static mut APP_MEMORY: [u8; 10240] = [0; 10240];

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static mut kernel::procs::Process<'static>>; NUM_PROCS] =
    [None, None, None, None];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct NativeProcess {
    console: &'static capsules::console::Console<
        'static,
        tock_native_chip::serial::NativeSerial<'static>,
    >,
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

/// Expected entry function
pub fn main() -> Result<(), io::Error> {
    // Panic setup in no_std is done via a language feature. In a std
    // environment, we install a hook to run, which should happen before any of
    // Tock proper runs.
    panic::set_hook(Box::new(|pi| native_panic::panic_hook(pi)));

    // Eventually, we'll need a real story for loading apps. I think the way to
    // do this best will be to create an image that represents what we'd
    // normally have put into flash and mmap that image so that it looks like
    // readable flash on the chip to the rest of the kernel.
    //
    // Create no-apps image with: truncate -s 1K /tmp/zeros
    let file = File::open("/tmp/zeros")?;
    let mmap = unsafe { Mmap::map(&file)? };

    // "Boot" the "machine"
    unsafe {
        reset_handler(mmap.as_ptr());
    }

    unimplemented!("Should never get past reset_handler()");
}

/// Reset Handler
#[no_mangle]
pub unsafe fn reset_handler(_sapps: *const u8) {
    tock_native_chip::init();

    let console = static_init!(
        capsules::console::Console<tock_native_chip::serial::NativeSerial>,
        capsules::console::Console::new(
            &tock_native_chip::serial::NATIVE_SERIAL_0,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            kernel::Grant::create()
        )
    );
    hil::uart::UART::set_client(&tock_native_chip::serial::NATIVE_SERIAL_0, console);

    let native = NativeProcess {
        console: console,
        //gpio: gpio,
        ipc: kernel::ipc::IPC::new(),
    };

    let mut chip = tock_native_chip::chip::NativeChip::new();

    native.console.initialize();

    // Attach the kernel debug interface to this console
    let kc = static_init!(capsules::console::App, capsules::console::App::default());
    kernel::debug::assign_console_driver(Some(native.console), kc);

    debug!("Initialization complete. Entering main loop...\r");

    /*
    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }
    */
    kernel::procs::load_processes(_sapps, &mut APP_MEMORY, &mut PROCESSES, FAULT_RESPONSE);
    kernel::kernel_loop(&native, &mut chip, &mut PROCESSES, Some(&native.ipc));
}
