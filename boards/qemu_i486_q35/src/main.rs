// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Board file for qemu-system-i486 "q35" machine type

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use core::ptr;

use capsules_core::alarm;
use capsules_core::console::{self, Console};
use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use components::console::ConsoleComponent;
use components::debug_writer::DebugWriterComponent;
use kernel::capabilities;
use kernel::component::Component;
use kernel::debug;
use kernel::hil;
use kernel::ipc::IPC;
use kernel::platform::chip::Chip;
use kernel::platform::scheduler_timer::VirtualSchedulerTimer;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::process::Process;
use kernel::scheduler::cooperative::CooperativeSched;
use kernel::syscall::SyscallDriver;
use kernel::{create_capability, static_init, Kernel};

use x86::registers::bits32::paging::{PDEntry, PTEntry, PD, PT};
use x86::registers::irq;

use x86_q35::pit::{Pit, RELOAD_1KHZ};
use x86_q35::{Pc, PcComponent};

mod multiboot;
use multiboot::MultibootV1Header;

mod io;

/// Multiboot V1 header, allowing this kernel to be booted directly by QEMU
#[link_section = ".vectors"]
#[used]
static MULTIBOOT_V1_HEADER: MultibootV1Header = MultibootV1Header::new(0);

const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn Process>; NUM_PROCS] = [None; NUM_PROCS];

// Reference to the chip for panic dumps
static mut CHIP: Option<&'static Pc> = None;

// Reference to the process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

// Static allocations used for page tables
//
// These are placed into custom sections so they can be properly aligned and padded in layout.ld
#[no_mangle]
#[link_section = ".pde"]
pub static mut PAGE_DIR: PD = [PDEntry(0); 1024];
#[no_mangle]
#[link_section = ".pte"]
pub static mut PAGE_TABLE: PT = [PTEntry(0); 1024];

pub struct QemuI386Q35Platform {
    pconsole: &'static capsules_core::process_console::ProcessConsole<
        'static,
        { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN },
        VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
        components::process_console::Capability,
    >,
    console: &'static Console<'static>,
    lldb: &'static capsules_core::low_level_debug::LowLevelDebug<
        'static,
        capsules_core::virtualizers::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
    >,
    ipc: IPC<{ NUM_PROCS as u8 }>,
    scheduler: &'static CooperativeSched<'static>,
    scheduler_timer:
        &'static VirtualSchedulerTimer<VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>>,
}

impl SyscallDriverLookup for QemuI386Q35Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R,
    {
        match driver_num {
            console::DRIVER_NUM => f(Some(self.console)),
            alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl<C: Chip> KernelResources<C> for QemuI386Q35Platform {
    type SyscallDriverLookup = Self;
    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }

    type SyscallFilter = ();
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }

    type ProcessFault = ();
    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }

    type Scheduler = CooperativeSched<'static>;
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }

    type SchedulerTimer =
        VirtualSchedulerTimer<VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>>;
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        self.scheduler_timer
    }

    type WatchDog = ();
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    type ContextSwitchCallback = ();
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

#[no_mangle]
unsafe extern "cdecl" fn main() {
    // ---------- BASIC INITIALIZATION -----------

    // Basic setup of the i486 platform
    let chip = PcComponent::new(
        &mut *ptr::addr_of_mut!(PAGE_DIR),
        &mut *ptr::addr_of_mut!(PAGE_TABLE),
    )
    .finalize(x86_q35::x86_q35_component_static!());

    // Acquire required capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    // Create a board kernel instance
    let board_kernel = static_init!(Kernel, Kernel::new(&*ptr::addr_of!(PROCESSES)));

    // ---------- QEMU-SYSTEM-I386 "Q35" MACHINE PERIPHERALS ----------

    // Create a shared UART channel for the console and for kernel
    // debug over the provided 8250-compatible UART.
    let uart_mux = components::console::UartMuxComponent::new(chip.com1, 115200)
        .finalize(components::uart_mux_component_static!());

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
        MuxAlarm::new(&chip.pit),
    );
    hil::time::Alarm::set_alarm_client(&chip.pit, mux_alarm);

    // Virtual alarm for the scheduler
    let systick_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    systick_virtual_alarm.setup();

    // Virtual alarm and driver for userspace
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_user.setup();

    let alarm = static_init!(
        capsules_core::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>,
        >,
        capsules_core::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules_core::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ---------

    // PIT interrupts need to be started manually
    chip.pit.start();

    // Enable interrupts after all drivers are initialized
    irq::enable();

    // ---------- FINAL SYSTEM INITIALIZATION ----------

    // Create the process printer used in panic prints, etc.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // Initialize the kernel's process console.
    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        None,
    )
    .finalize(components::process_console_component_static!(
        Pit<'static, RELOAD_1KHZ>
    ));

    // Setup the console.
    let console = ConsoleComponent::new(board_kernel, console::DRIVER_NUM, uart_mux)
        .finalize(components::console_component_static!());

    // Create the debugger object that handles calls to `debug!()`.
    DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    let lldb = components::lldb::LowLevelDebugComponent::new(
        board_kernel,
        capsules_core::low_level_debug::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::low_level_debug_component_static!());

    let scheduler =
        components::sched::cooperative::CooperativeComponent::new(&*ptr::addr_of!(PROCESSES))
            .finalize(components::cooperative_component_static!(NUM_PROCS));

    let scheduler_timer = static_init!(
        VirtualSchedulerTimer<VirtualMuxAlarm<'static, Pit<'static, RELOAD_1KHZ>>>,
        VirtualSchedulerTimer::new(systick_virtual_alarm)
    );

    let platform = QemuI386Q35Platform {
        pconsole,
        console,
        alarm,
        lldb,
        scheduler,
        scheduler_timer,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_cap,
        ),
    };

    // Start the process console:
    let _ = platform.pconsole.start();

    debug!("QEMU i486 \"Q35\" machine, initialization complete.");
    debug!("Entering main loop.");

    // These symbols are defined in the linker script.
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

    // ---------- PROCESS LOADING, SCHEDULER LOOP ----------

    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            ptr::addr_of!(_sapps),
            ptr::addr_of!(_eapps) as usize - ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            ptr::addr_of_mut!(_sappmem),
            ptr::addr_of!(_eappmem) as usize - ptr::addr_of!(_sappmem) as usize,
        ),
        &mut *ptr::addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_cap);
}
