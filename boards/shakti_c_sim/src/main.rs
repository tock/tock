// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Board file for the SHAKTI C-Class (RV64) Verilator simulation.
//!
//! Boots a single real TBF process under `kernel_loop` and exercises the full
//! RV64 userspace round-trip: the process performs a time-based syscall via the
//! Alarm capsule (Subscribe -> Command(set-alarm) -> Yield), is resumed by the
//! machine-timer interrupt through `_start_trap`, and reads back the elapsed
//! time, proving switch_to_process -> mret-to-user -> ecall -> kernel -> UART
//! end-to-end on rv64. The test app is hand-written assembly (raw `ecall`s; no
//! libtock-rs rv64 target yet). A board-local SimControl driver times the alarm
//! and prints the delta; boot progress is logged with `debug!()` over the SoC
//! UART.

#![no_std]
#![no_main]

use core::cell::Cell;

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::debug::PanicResources;
use kernel::hil::time::{Alarm, Time};
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{ProcessId, debug, static_init};

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use shakti_c::chip::{ShaktiC, ShaktiCClint};

mod io;

/// Process capacity. loads exactly one app.
pub const NUM_PROCS: usize = 1;

/// Board-local "SimControl" syscall driver number (board-private range).
const SIMCTL_DRIVER_NUM: usize = 0x9000;

type ShaktiCChip = ShaktiC<'static>;
type SchedulerInUse = capsules_system::scheduler::round_robin::RoundRobinSched<'static>;
type AlarmDriverInUse =
    capsules_core::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, ShaktiCClint<'static>>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources the panic handler in `io.rs` needs to print CPU + process state.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ShaktiCChip, ProcessPrinterInUse>> =
    SingleThreadValue::new();

/// How the kernel responds when a process faults: panic, which the standard
/// panic handler (`io.rs`) turns into a UART dump + clean sim exit.
static FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Reserve the kernel stack (referenced by the linker script).
kernel::stack_size! {0x2000}

const SIM_FINISH: *mut u32 = 0x0002_000C as *mut u32;

/// End the Verilator simulation cleanly (so the testbench flushes `app_log`).
///
/// `debug!()` output is buffered twice over, and both buffers must be drained
/// before the machine is halted or the last lines are lost:
///
/// 1. The debug writer hands a single output buffer to the UART mux at a time
///    and only refills it from the deferred call that completes the previous
///    transmission, so back-to-back `debug!()` calls leave bytes in the debug
///    ring buffer. Servicing the pending deferred calls drains it; each chip
///    transmit is synchronous, so this terminates once the ring is empty.
/// 2. The async `transmit_buffer` path only waits for `tx_full` to clear before
///    each write, so the last bytes are still in the UART's hardware FIFO when
///    it returns. `transmit_sync` additionally waits on `transmission_done`;
///    calling it with an empty slice waits without emitting anything.
unsafe fn sim_finish() -> ! {
    while kernel::deferred_call::DeferredCall::has_tasks() {
        let _ = kernel::deferred_call::DeferredCall::service_next_pending();
    }
    shakti_c::uart::Uart::new(shakti_c::uart::UART0_BASE).transmit_sync(&[]);

    core::ptr::write_volatile(SIM_FINISH, 1);
    loop {
        core::hint::spin_loop();
    }
}

// --- SimControl syscall driver (timestamps + sim exit) ------------------------

/// Board-local driver to time the alarm and end the simulation.
///
/// Command 2 captures `t_before` (when the app arms the alarm, just before it
/// yields); command 1 captures `t_after` (when the fired upcall has resumed
/// the app) and prints the elapsed delta, then ends the sim.
struct SimControl {
    timer: &'static ShaktiCClint<'static>,
    t_before: Cell<u64>,
}

impl SyscallDriver for SimControl {
    fn command(&self, command_num: usize, _r2: usize, _r3: usize, _id: ProcessId) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            2 => {
                // t_before: the app has armed the alarm and is about to yield.
                let t = self.timer.now().into_u64();
                self.t_before.set(t);
                debug!("t_before  mtime={:#018x}", t);
                CommandReturn::success()
            }
            1 => {
                // t_after: the fired upcall has resumed the app after the delay.
                let t = self.timer.now().into_u64();
                let dt = t.wrapping_sub(self.t_before.get());
                debug!("process resumed after alarm-fired upcall");
                debug!("t_after   mtime={:#018x}", t);
                debug!("elapsed ticks (10 MHz) = {:#018x}", dt);
                debug!("*** STAGE 5 PASS ***");
                unsafe { sim_finish() };
            }
            _ => CommandReturn::failure(kernel::ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _process_id: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}

// --- Board / KernelResources --------------------------------------------------

struct ShaktiCSim {
    scheduler: &'static SchedulerInUse,
    alarm: &'static AlarmDriverInUse,
    simctl: &'static SimControl,
}

impl SyscallDriverLookup for ShaktiCSim {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            SIMCTL_DRIVER_NUM => f(Some(self.simctl)),
            _ => f(None),
        }
    }
}

impl KernelResources<ShaktiCChip> for ShaktiCSim {
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
        &()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// Entry point, reached from the RV64 startup assembly once RAM is initialized.
///
/// # Safety
/// Accesses memory-mapped registers and CSRs, and performs one-time static init.
#[no_mangle]
pub unsafe fn main() {
    use rv64i::csr;

    // Point mtvec at _start_trap and mark mscratch = 0 (kernel mode).
    rv64i::configure_trap_handler();

    // Deferred calls may be created during bring-up (the virtual-alarm layer can
    // use one for already-expired alarms); init the state first.
    kernel::deferred_call::initialize_deferred_call_state::<rv64i::thread_id::RiscvThreadIdProvider>(
    );

    let _ = PANIC_RESOURCES
        .bind_to_thread::<<ShaktiCChip as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );

    let uart0 = static_init!(
        shakti_c::uart::Uart,
        shakti_c::uart::Uart::new(shakti_c::uart::UART0_BASE)
    );
    kernel::deferred_call::DeferredCallClient::register(uart0);

    let uart_mux = components::console::UartMuxComponent::new(uart0, 115200)
        .finalize(components::uart_mux_component_static!());

    components::debug_writer::DebugWriterComponent::new::<
        <ShaktiCChip as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES
        .get()
        .map(|r| r.printer.put(process_printer));

    debug!("=== Tock OS on SHAKTI C-Class (RV64IMAC) ===");
    debug!("[shakti_c_sim] real TBF process, time-based syscall via Alarm");

    // Core kernel + process array.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES
        .get()
        .map(|r| r.processes.put(processes.as_slice()));
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    // The 64-bit CLINT timer, shared by the chip (for service_pending_interrupts)
    // and the alarm mux.
    let timer = static_init!(
        ShaktiCClint,
        ShaktiCClint::new(&shakti_c::clint::CLINT_BASE)
    );
    let chip = static_init!(ShaktiCChip, ShaktiC::new(timer));
    PANIC_RESOURCES.get().map(|r| r.chip.put(chip));

    // Alarm stack: MuxAlarm over the CLINT, a user VirtualMuxAlarm, and the
    // AlarmDriver (driver 0).
    let mux_alarm = static_init!(MuxAlarm<ShaktiCClint>, MuxAlarm::new(timer));
    Alarm::set_alarm_client(timer, mux_alarm);

    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<ShaktiCClint>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_user.setup();

    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let alarm = static_init!(
        AlarmDriverInUse,
        capsules_core::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules_core::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    Alarm::set_alarm_client(virtual_alarm_user, alarm);

    let simctl = static_init!(
        SimControl,
        SimControl {
            timer,
            t_before: Cell::new(0),
        }
    );

    // Linker symbols bracketing the app flash region (TBF spliced at _sapps =
    // 0x80100000) and the RAM available for app memory.
    extern "C" {
        static _sapps: u8;
        static _eapps: u8;
        static mut _sappmem: u8;
        static _eappmem: u8;
    }

    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);

    // Load the TBF app image through the standard process loader.
    debug!(
        "calling load_processes from _sapps={:#018x}",
        core::ptr::addr_of!(_sapps) as u64
    );

    let load_result = kernel::process::load_processes(
        board_kernel,
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
        &process_mgmt_cap,
    );

    match load_result {
        Ok(()) => debug!("load_processes Ok"),
        Err(e) => {
            debug!("FAIL: load_processes returned Err (TBF rejected): {:?}", e);
            sim_finish();
        }
    }

    let mut found = 0usize;
    board_kernel.process_each_capability(&process_mgmt_cap, |_p| found += 1);
    debug!("processes loaded = {}", found);
    if found == 0 {
        debug!("FAIL: no process in any slot");
        sim_finish();
    }

    // Enable machine-timer interrupts and enter the kernel loop.
    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));
    let board = ShaktiCSim {
        scheduler,
        alarm,
        simctl,
    };

    // Enable the machine-timer interrupt source + global machine interrupts so
    // the CLINT alarm can wake the kernel from `wfi` (as during timer bring-up).
    csr::CSR.mie.modify(csr::mie::mie::mtimer::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    debug!("alarm wired (driver 0); mtimer IRQ enabled; entering kernel_loop");
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(&board, chip, None::<&kernel::ipc::IPC<0>>, &main_loop_cap);
}
