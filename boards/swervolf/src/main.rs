//! Board file for SweRVolf RISC-V development platform.
//!
//! - <https://github.com/chipsalliance/Cores-SweRVolf>
//!

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::common::registers::interfaces::ReadWriteable;
use kernel::component::Component;
use kernel::hil;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};
use rv32i::csr;
use swervolf_eh1::chip::SweRVolfDefaultPeripherals;

pub mod io;

pub const NUM_PROCS: usize = 4;
const NUM_UPCALLS_IPC: usize = NUM_PROCS + 1;
//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::procs::Process>; NUM_PROCS] = [None; NUM_PROCS];

// Reference to the chip for panic dumps.
static mut CHIP: Option<&'static swervolf_eh1::chip::SweRVolf<SweRVolfDefaultPeripherals>> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::PanicFaultPolicy = kernel::procs::PanicFaultPolicy {};

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x900] = [0; 0x900];

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct SweRVolf {
    console: &'static capsules::console::Console<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, swervolf_eh1::syscon::SysCon<'static>>,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for SweRVolf {
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

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    // only machine mode
    rv32i::configure_trap_handler(rv32i::PermissionMode::Machine);

    let peripherals = static_init!(
        SweRVolfDefaultPeripherals,
        SweRVolfDefaultPeripherals::new()
    );

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

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
        &peripherals.uart,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    let mtimer = static_init!(
        swervolf_eh1::syscon::SysCon,
        swervolf_eh1::syscon::SysCon::new()
    );

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, swervolf_eh1::syscon::SysCon>,
        MuxAlarm::new(mtimer)
    );
    hil::time::Alarm::set_alarm_client(mtimer, mux_alarm);

    // Alarm
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, swervolf_eh1::syscon::SysCon>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<'static, swervolf_eh1::syscon::SysCon>,
        >,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    let chip = static_init!(
        swervolf_eh1::chip::SweRVolf<
            SweRVolfDefaultPeripherals,
        >,
        swervolf_eh1::chip::SweRVolf::new(peripherals, mtimer)
    );
    CHIP = Some(chip);

    // Need to enable all interrupts for Tock Kernel
    chip.enable_pic_interrupts();

    // enable interrupts globally, including timer0 (bit 29) and timer1 (bit 28)
    csr::CSR.mie.modify(
        csr::mie::mie::mext::SET
            + csr::mie::mie::msoft::SET
            + csr::mie::mie::mtimer::SET
            + csr::mie::mie::BIT28::SET
            + csr::mie::mie::BIT29::SET,
    );
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

    debug!("SweRVolf initialisation complete.");
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

    let swervolf = SweRVolf { console, alarm };

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

    let scheduler = components::sched::cooperative::CooperativeComponent::new(&PROCESSES)
        .finalize(components::coop_component_helper!(NUM_PROCS));
    board_kernel.kernel_loop(
        &swervolf,
        chip,
        None::<&kernel::ipc::IPC<NUM_PROCS, NUM_UPCALLS_IPC>>,
        None::<&kernel::ros::ROSDriver<swervolf_eh1::syscon::SysCon>>,
        scheduler,
        &main_loop_cap,
    );
}
