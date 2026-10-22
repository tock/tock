// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the network core of the Nordic Semiconductor nRF5340
//! development kit.
//!
//! The application core kernel must release the network core from reset and
//! hand it the console pins before this kernel runs.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::hil::time::Counter;
use kernel::platform::chip::Chip;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::StaticRef;
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{capabilities, create_capability, debug, static_init};
use nrf5340_net::ble_ipc::{BleMailboxRegisters, BleServer};
use nrf5340_net::chip::Nrf5340NetDefaultPeripherals;
use nrf5340_net::gpio::Pin;
use nrf5340_net::pinmux::Pinmux;

/// UART transmit pin, connected to a J-Link virtual COM port.
pub const UART_TXD: Pin = Pin::P1_01;
/// UART receive pin, connected to a J-Link virtual COM port.
pub const UART_RXD: Pin = Pin::P1_00;

/// UART Writer
pub mod io;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

type ChipHw = nrf5340_net::chip::Nrf5340Net<'static, Nrf5340NetDefaultPeripherals<'static>>;
type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;

/// Resources for when a board panics used by io.rs.
static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

kernel::stack_size! {0x1000}

type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

//------------------------------------------------------------------------------
// SYSCALL DRIVER TYPE DEFINITIONS
//------------------------------------------------------------------------------

type AlarmHw = nrf5340_net::rtc::Rtc<'static>;
type BleHw = nrf5340_net::ble_radio::Radio<'static>;

/// The BLE mailbox, in the RAM region the application core kernel leaves out of
/// its RAM and shares with this core. It must match `../nrf5340dk`.
//
// # Safety
//
// This memory belongs to neither kernel, and only the BLE IPC drivers access
// it, through volatile reads and writes.
const BLE_MAILBOX: StaticRef<BleMailboxRegisters> =
    unsafe { StaticRef::new(0x2007E000 as *const BleMailboxRegisters) };

type AlarmDriver = components::alarm::AlarmDriverComponentType<AlarmHw>;
type ConsoleDriver = components::console::ConsoleComponentType;
kernel::define_capability_type!(ProcessConsoleCap:
    kernel::capabilities::ProcessManagementCapability,
    kernel::capabilities::ProcessStartCapability
);
type ProcessConsoleDriver =
    components::process_console::ProcessConsoleComponentType<AlarmHw, ProcessConsoleCap>;

/// Supported drivers by the platform
pub struct Platform {
    alarm: &'static AlarmDriver,
    console: &'static ConsoleDriver,
    pconsole: &'static ProcessConsoleDriver,
    scheduler: &'static SchedulerInUse,
    systick: cortexm33::systick::SysTick,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            _ => f(None),
        }
    }
}

impl KernelResources<ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = cortexm33::systick::SysTick;
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
        &self.systick
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
pub unsafe fn start() -> (&'static kernel::Kernel, Platform, &'static ChipHw) {
    ChipHw::init();

    // Initialize deferred calls very early.
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    // Bind global variables to this thread.
    let _ = PANIC_RESOURCES
        .bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );

    // SAFETY: This is the only copy and only user of the DMA peripherals.
    let peripherals = static_init!(Nrf5340NetDefaultPeripherals, unsafe {
        Nrf5340NetDefaultPeripherals::new()
    });
    peripherals
        .clock
        .start(nrf5340_net::clock::LfclkSource::Lfxo);

    // Create an array to hold process references.
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    // Setup space to store the core kernel data structure.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    let chip = static_init!(ChipHw, nrf5340_net::chip::Nrf5340Net::new(peripherals));
    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    let rtc = &peripherals.rtc;
    let _ = rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_static!(AlarmHw));
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::alarm_component_static!(AlarmHw));

    // Serve the radio to the application core over IPC.
    let ble_tx_buffer = static_init!([u8; 255], [0; 255]);
    let ble_rx_buffer = static_init!([u8; 255], [0; 255]);
    let ble_server = static_init!(
        BleServer<'static, BleHw>,
        BleServer::new(
            BLE_MAILBOX,
            &peripherals.ipc,
            &peripherals.ble_radio,
            ble_tx_buffer,
            ble_rx_buffer,
        )
    );
    ble_server.init();

    peripherals
        .uarte0
        .initialize(Pinmux::new(UART_TXD), Pinmux::new(UART_RXD), None, None);

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PANIC_RESOURCES.get().map(|resources| {
        resources.printer.put(process_printer);
    });

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.uarte0, 115200)
        .finalize(components::uart_mux_component_static!());

    let process_console_cap = unsafe { kernel::mint_defined_capability!(ProcessConsoleCap) };
    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        Some(cortexm33::support::reset),
        process_console_cap,
    )
    .finalize(components::process_console_component_static!(
        AlarmHw,
        ProcessConsoleCap,
    ));

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
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

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let platform = Platform {
        alarm,
        console,
        pconsole,
        scheduler,
        // The network core runs at a fixed 64 MHz.
        systick: cortexm33::systick::SysTick::new_with_calibration(64_000_000),
    };

    let _ = platform.pconsole.start();
    debug!("Initialization complete. Entering main loop\r");

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
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, platform, chip)
}

/// Main function called after RAM initialized.
///
/// # Safety
///
/// ## `no_mangle`
///
/// We use `main` as a symbol in the arch crate's initialization routine to jump
/// here. We guarantee no other symbol has the same name.
///
/// Note: this requires a global/uniqueness guarantee.
/// See <https://github.com/tock/tock/issues/5250>.
#[unsafe(no_mangle)]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
