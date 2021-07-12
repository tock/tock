//! Board file for ESP32-C3 RISC-V development platform.
//!

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use esp32_c3::chip::Esp32C3DefaultPeripherals;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::common::registers::interfaces::ReadWriteable;
use kernel::component::Component;
use kernel::{create_capability, debug, hil, static_init, Platform};
use rv32i::csr;

pub mod io;

#[cfg(test)]
mod tests;

const NUM_PROCS: usize = 4;
const NUM_UPCALLS_IPC: usize = NUM_PROCS + 1;
//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::procs::Process>; NUM_PROCS] = [None; NUM_PROCS];

// Reference to the chip for panic dumps.
static mut CHIP: Option<&'static esp32_c3::chip::Esp32C3<Esp32C3DefaultPeripherals>> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::PanicFaultPolicy = kernel::procs::PanicFaultPolicy {};

// Test access to the peripherals
#[cfg(test)]
static mut PERIPHERALS: Option<&'static Esp32C3DefaultPeripherals> = None;
// Test access to scheduler
#[cfg(test)]
static mut SCHEDULER: Option<&kernel::PrioritySched> = None;
// Test access to board
#[cfg(test)]
static mut BOARD: Option<&'static kernel::Kernel> = None;
// Test access to platform
#[cfg(test)]
static mut PLATFORM: Option<&'static Esp32C3Board> = None;
// Test access to main loop capability
#[cfg(test)]
static mut MAIN_CAP: Option<&dyn kernel::capabilities::MainLoopCapability> = None;
// Test access to alarm
static mut ALARM: Option<&'static MuxAlarm<'static, esp32::timg::TimG<'static>>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x900] = [0; 0x900];

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct Esp32C3Board {
    console: &'static capsules::console::Console<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, esp32::timg::TimG<'static>>,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for Esp32C3Board {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            _ => f(None),
        }
    }
}

unsafe fn setup() -> (
    &'static kernel::Kernel,
    &'static Esp32C3Board,
    &'static esp32_c3::chip::Esp32C3<'static, Esp32C3DefaultPeripherals<'static>>,
    &'static Esp32C3DefaultPeripherals<'static>,
) {
    // only machine mode
    rv32i::configure_trap_handler(rv32i::PermissionMode::Machine);

    let peripherals = static_init!(Esp32C3DefaultPeripherals, Esp32C3DefaultPeripherals::new());

    peripherals.timg0.disable_wdt();
    peripherals.rtc_cntl.disable_wdt();
    peripherals.rtc_cntl.disable_super_wdt();

    // initialise capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 1], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(None, None, None);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &peripherals.uart0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, esp32::timg::TimG>,
        MuxAlarm::new(&peripherals.timg0)
    );
    hil::time::Alarm::set_alarm_client(&peripherals.timg0, mux_alarm);

    ALARM = Some(mux_alarm);

    // Alarm
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, esp32::timg::TimG>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, esp32::timg::TimG>>,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    let chip = static_init!(
        esp32_c3::chip::Esp32C3<
            Esp32C3DefaultPeripherals,
        >,
        esp32_c3::chip::Esp32C3::new(peripherals)
    );
    CHIP = Some(chip);

    // Need to enable all interrupts for Tock Kernel
    chip.enable_pic_interrupts();

    // enable interrupts globally
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    debug!("ESP32-C3 initialisation complete.");
    debug!("Entering main loop.");

    /// These symbols are defined in the linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    let esp32_c3_board = static_init!(Esp32C3Board, Esp32C3Board { console, alarm });

    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        core::slice::from_raw_parts_mut(
            &mut _sappmem as *mut u8,
            &_eappmem as *const u8 as usize - &_sappmem as *const u8 as usize,
        ),
        &mut PROCESSES,
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, esp32_c3_board, chip, peripherals)
}

/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup and RAM initialization.
#[no_mangle]
pub unsafe fn main() {
    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    {
        let (board_kernel, esp32_c3_board, chip, _peripherals) = setup();

        let scheduler =
            components::sched::priority::PriorityComponent::new(board_kernel).finalize(());
        let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

        board_kernel.kernel_loop(
            esp32_c3_board,
            chip,
            None::<&kernel::ipc::IPC<NUM_PROCS, NUM_UPCALLS_IPC>>,
            scheduler,
            &main_loop_cap,
        );
    }
}

#[cfg(test)]
use kernel::watchdog::WatchDog;
#[cfg(test)]
use kernel::Chip;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    unsafe {
        let (board_kernel, esp32_c3_board, chip, peripherals) = setup();

        BOARD = Some(board_kernel);
        PLATFORM = Some(&esp32_c3_board);
        PERIPHERALS = Some(peripherals);
        SCHEDULER =
            Some(components::sched::priority::PriorityComponent::new(board_kernel).finalize(()));
        MAIN_CAP = Some(&create_capability!(capabilities::MainLoopCapability));

        chip.watchdog().setup();

        for test in tests {
            test();
        }
    }

    loop {}
}
