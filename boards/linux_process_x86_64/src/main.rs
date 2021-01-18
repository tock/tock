//! Board file for Linux process for AMD64

// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
// #![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]
// #![deny(missing_docs)]

// use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::Platform;
use kernel::{capabilities, create_capability, debug, static_init};
use std::panic;

// /// Support routines for debugging I/O.
pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
#[no_mangle]
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

static mut CHIP: Option<&'static linux_x86_64::chip::Linux> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

#[link_section = ".apps"]
static mut APP_FLASH: [u8; 256 * 1024] = [0; 256 * 1024];
static mut APP_RAM: [u8; 256 * 1024] = [0; 256 * 1024];

// #[no_mangle]
// #[link_section = ".stack_buffer"]
// pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

struct LinuxProcess {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC<NUM_PROCS>,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for LinuxProcess {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

unsafe fn reset_handler() {
    linux_x86_64::init();

    let flash = posix_x86_64::initialize_flash(&APP_FLASH);

    panic::set_hook(Box::new(|panic_info| {
        io::panic_fmt(panic_info);
    }));

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let chip = static_init!(linux_x86_64::chip::Linux, linux_x86_64::chip::Linux::new());
    CHIP = Some(chip);

    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    let uart_mux = components::console::UartMuxComponent::new(
        &linux_x86_64::console::CONSOLE,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let linux_process = LinuxProcess {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    // /// These symbols are defined in the linker script.
    // extern "C" {
    //     /// Beginning of the ROM region containing app images.
    //     static _sapps: u8;
    //     /// End of the ROM region containing app images.
    //     static _eapps: u8;
    //     /// Beginning of the RAM region for app memory.
    //     static mut _sappmem: u8;
    //     /// End of the RAM region for app memory.
    //     static _eappmem: u8;
    // }

    debug!("Initialization complete. Entering main loop");

    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(flash, APP_FLASH.len()),
        core::slice::from_raw_parts_mut(&APP_RAM as *const u8 as *mut u8, APP_RAM.len()),
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));
    board_kernel.kernel_loop(
        &linux_process,
        chip,
        Some(&linux_process.ipc),
        scheduler,
        &main_loop_capability,
    );
}

fn main() {
    unsafe {
        reset_handler();
    }
    // loop {}
}
