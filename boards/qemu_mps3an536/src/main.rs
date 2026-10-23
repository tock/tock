// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Board file for the ARM MPS3-AN536 (Cortex-R52) platform.

#![no_std]
#![no_main]

//use aarch32_cpu::register;
use kernel::capabilities;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::platform::{KernelResources, SyscallDriverLookup, chip::Chip};
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{create_capability, debug, static_init};

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;

use xcku::an536_inst;
use xcku::chip::XckuChip;

mod io;
mod low_level_debug;

/// Maximum number of concurrent processes.
const NUM_PROCS: usize = 6;

/// Chip hardware type alias used by `main.rs` and `io.rs`.
type ChipHw = XckuChip;
type ProcessPrinter = capsules_system::process_printer::ProcessPrinterText;
type SchedulerInUse = components::sched::cooperative::CooperativeComponentType;

/// Resources for when a board panics used by io.rs.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinter>> =
    SingleThreadValue::new();

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// FIQ + system + other stacks, check arch/x/vectors.rs for each size
// TODO: dity code + need to check required size
kernel::stack_size! {2048 + 256 + 256}

/// Top-level platform structure holding references to all active capsules.
struct QemuMps3 {
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    scheduler: &'static SchedulerInUse,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, an536_inst::GenTimer<'static>>,
    >,
    console: &'static capsules_core::console::Console<'static>,
    low_level_debug: &'static low_level_debug::MyLowLevelDebug,
}

impl SyscallDriverLookup for QemuMps3 {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            //capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules_core::low_level_debug::DRIVER_NUM => f(Some(self.low_level_debug)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => {
                panic!("UNKNOWN DRIVER {driver_num}");
            }
        }
    }
}

impl KernelResources<ChipHw> for QemuMps3 {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = ();
    type WatchDog = ();
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }

    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }

    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }

    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &() // TODO: need dualtimer for round robin
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &() // TODO: need WDG driver
    }

    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// Entry point called after the C runtime has initialised RAM.
///
/// Must only be called once, directly by the reset handler defined in the
/// linker script / start-up assembly. The call stack and `.bss`/`.data`
/// sections must already be initialised before this function is entered.

#[no_mangle]
pub unsafe fn main() {
    sh::p!("Hello World from QEMU / MPS3 / XKCU115 (AN536) / Cortex-R52!");

    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // ---------------------------------------------------------------------------
    // Kernel object
    // ---------------------------------------------------------------------------
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    let kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    // ---------------------------------------------------------------------------
    // Chip
    // ---------------------------------------------------------------------------
    let chip = static_init!(ChipHw, XckuChip::new());

    // ---------------------------------------------------------------------------
    // IPC
    // ---------------------------------------------------------------------------
    let ipc = kernel::ipc::IPC::new(
        kernel,
        kernel::ipc::DRIVER_NUM,
        &create_capability!(capabilities::MemoryAllocationCapability),
    );

    // ---------------------------------------------------------------------------
    // Peripherals
    // ---------------------------------------------------------------------------
    // ALARM
    let mux_alarm = components::alarm::AlarmMuxComponent::new(&chip.gentimer).finalize(
        components::alarm_mux_component_static!(an536_inst::GenTimer),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(
        kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::alarm_component_static!(an536_inst::GenTimer));

    // UART
    let uart_mux = components::console::UartMuxComponent::new(&chip.uart2, 115200)
        .finalize(components::uart_mux_component_static!());
    let console = components::console::ConsoleComponent::new(
        kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::console_component_static!());

    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // ---------------------------------------------------------------------------
    // Platform
    // ---------------------------------------------------------------------------
    let platform = static_init!(
        QemuMps3,
        QemuMps3 {
            ipc,
            scheduler: components::sched::cooperative::CooperativeComponent::new(processes)
                .finalize(components::cooperative_component_static!(NUM_PROCS)),
            alarm,
            console,
            low_level_debug: static_init!(
                low_level_debug::MyLowLevelDebug,
                low_level_debug::MyLowLevelDebug::new()
            ),
        }
    );

    // ---------------------------------------------------------------------------
    // IRQ
    // ---------------------------------------------------------------------------
    chip.gic.irq_enable(cortexr52::gic::IRQ_GENTIMER_PHY, 0x1);
    chip.gic.irq_enable(an536_inst::IRQ_UART2_TX, 0x2);
    chip.gic.irq_enable(an536_inst::IRQ_UART2_RX, 0x3);
    chip.gic.irq_enable(an536_inst::IRQ_DUALTIMER1, 0x4);

    //XckuChip::init();

    ChipHw::init();

    // ---------------------------------------------------------------------------
    // App processes
    // ---------------------------------------------------------------------------

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

    kernel::process::load_processes(
        kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        ),
        &FAULT_RESPONSE,
        &create_capability!(capabilities::ProcessManagementCapability),
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    //let periphbase = aarch32_cpu::register::ImpCbar::read().periphbase();
    //sh::p!("Peripheral base address: {:010p}", periphbase);

    //unsafe {
    //    let addr = 0xE010_1008 as *mut u32;
    //    *addr = 0b10000010;
    //    let addr = 0xE010_1028 as *mut u32;
    //    *addr = 0b10001010;
    //}

    sh::p!("Init done, main loop now.");

    // TESTS:
    //debug!("REAL TEST UART!");
    //chip.gentimer.set_alarm(
    //    Time::now(&chip.gentimer),
    //    Ticks64::from(150_000_000_u64),
    //);

    fn _debugfn() {
        // unsafe {
        //let reg : u32;
        //core::arch::asm!("
        //    MRC p15, 0, {0}, c12, c1, 0 // ISR
        //    ",
        //    out(reg) reg);
        //sh::p!("ISR: 0x{reg:x?}");
        //
        //for addr in (0xE020_5000_u32..0xE020_5000 + 0x10).step_by(4) {
        //    let value = *(addr as *const u32);
        //    sh::p!("0x{addr:08x}: 0x{value:08x}");
        //}
        //}

        //sh::p!("now {:x}", );

        //let value1 = register::CntPct::read().0 as u32;
        //let value2 = unsafe { *(0xE010_1004 as *const i32) };
        //sh::p!("nox {:x}", value1 - value2 as u32);
        //sh::reg("TIMV1", 0xE010_1004);
        //sh::reg("TIMV2", 0xE010_1024);

        sh::s!(".");
    }

    let main_loop_capability = kernel::create_capability!(kernel::capabilities::MainLoopCapability);

    // Need bigger DEBUG_BUFFER_SPLIT; two calls not working
    debug!(
        concat!(
            "QEMU MPS3-AN536 (Cortex-R52) initialisation complete.\n",
            "Tock kernel version: {}"
        ),
        env!("CARGO_PKG_VERSION")
    );

    kernel.kernel_loop(
        platform,
        chip,
        Some(&platform.ipc),
        &main_loop_capability,
        //debugfn,
    );
}
