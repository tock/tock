// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Board library for NXP S32G3 SAIL.

#![no_std]
#![no_main]

pub mod io;

use kernel::capabilities;
use kernel::component::Component;
use kernel::debug::PanicResources;
use kernel::hil::time::Counter;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{create_capability, static_init};
use nxp_s32g3::linflexd::LinFlexD;
use nxp_s32g3::mscm;
use nxp_s32g3::stm::{Stm, STM_1_BASE};
pub const NUM_PROCS: usize = 4;

pub struct NxpS32g3SailPeripherals {
    pub uart: &'static LinFlexD<'static>,
    pub stm: &'static Stm<'static>,
    pub mscm: &'static mscm::Mscm,
}

impl kernel::platform::chip::InterruptService for NxpS32g3SailPeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            mscm::LINFLEXD_0 => {
                self.uart.handle_interrupt();
                true
            }
            mscm::STM_1 => {
                self.stm.handle_interrupt();
                true
            }
            _ => false,
        }
    }
}

pub type ChipHw = nxp_s32g3::chip::S32g3<NxpS32g3SailPeripherals>;
pub type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;
pub type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

pub static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

kernel::stack_size! {0x2000}

pub struct NxpS32g3SailPlatform {
    pub pconsole: &'static capsules_core::process_console::ProcessConsole<
        'static,
        { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN },
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, Stm<'static>>,
        components::process_console::Capability,
    >,
    pub console: &'static capsules_core::console::Console<'static>,
    pub alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, Stm<'static>>,
    >,
    pub ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    pub scheduler: &'static SchedulerInUse,
    pub systick: cortexm7::systick::SysTick,
}

impl SyscallDriverLookup for NxpS32g3SailPlatform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<ChipHw> for NxpS32g3SailPlatform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = SchedulerInUse;
    type SchedulerTimer = cortexm7::systick::SysTick;
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

#[inline(never)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    NxpS32g3SailPlatform,
    &'static ChipHw,
) {
    nxp_s32g3::init();

    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    let _ = PANIC_RESOURCES
        .bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );

    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    // Initialize pinmux using the SIUL2 driver for LinFlexD0 (UART Console) and LinFlexD1
    let siul2_0 = nxp_s32g3::siul2::Siul2::new(nxp_s32g3::siul2::SIUL2_0_BASE);
    let siul2_1 = nxp_s32g3::siul2::Siul2::new(nxp_s32g3::siul2::SIUL2_1_BASE);
    // Pinmux for LinFlexD0 (LF0)
    // TX: MSCR[41] = ALT1 (1) | OBE | IBE | SRE
    // RX: MSCR[42] = IBE
    // IMCR[0] = 2 (ALT2)
    siul2_0.setup_tx_pin(nxp_s32g3::siul2::Pin::PC9, 1);
    siul2_0.setup_rx_pin(nxp_s32g3::siul2::Pin::PC10);
    siul2_0.setup_imcr(
        nxp_s32g3::siul2::Imcr::LinflexD0Rx,
        nxp_s32g3::siul2::ImcrSource::Alt2,
    );
    // Pinmux for LinFlexD1 (LF1)
    // TX: MSCR[40] = ALT2 (2) | OBE | IBE | SRE
    // RX: MSCR[36] = IBE
    // IMCR[224] = 4 (ALT4)
    siul2_0.setup_tx_pin(nxp_s32g3::siul2::Pin::PC8, 2);
    siul2_0.setup_rx_pin(nxp_s32g3::siul2::Pin::PC4);
    siul2_1.setup_imcr(
        nxp_s32g3::siul2::Imcr::LinflexD1Rx,
        nxp_s32g3::siul2::ImcrSource::Alt4,
    );
    // MC_ME partitions: Partition 2 (PFE/HSE) and Partition 3 (LLCE - which contains LinFlexD1)
    // TODO: remove magic numbers
    nxp_s32g3::mc_me::partition_enable(2);
    nxp_s32g3::mc_me::partition_enable(3);
    let uart = static_init!(LinFlexD, LinFlexD::new_lf0());
    let stm = static_init!(Stm<'static>, Stm::new(STM_1_BASE));
    let mscm = static_init!(mscm::Mscm, mscm::Mscm::new());

    // MSCM Shared Peripheral Routing: steering interrupts to M7_0
    for &irq in &[mscm::LINFLEXD_0, mscm::STM_1] {
        mscm.enable_interrupt(irq, mscm::S32G3Core::M7_0);
        cortexm7::nvic::Nvic::new(irq).enable();
    }

    let peripherals = static_init!(
        NxpS32g3SailPeripherals,
        NxpS32g3SailPeripherals { uart, stm, mscm }
    );
    let chip = static_init!(ChipHw, ChipHw::new(peripherals));

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(uart, 115200)
        .finalize(components::uart_mux_component_static!());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
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

    stm.start().unwrap();

    let mux_alarm = components::alarm::AlarmMuxComponent::new(stm)
        .finalize(components::alarm_mux_component_static!(Stm));

    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(Stm));

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());

    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
        resources.chip.put(chip);
        resources.printer.put(process_printer);
    });

    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        None,
    )
    .finalize(components::process_console_component_static!(Stm));

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let ipc = kernel::ipc::IPC::new(
        board_kernel,
        kernel::ipc::DRIVER_NUM,
        &memory_allocation_cap,
    );

    let platform = NxpS32g3SailPlatform {
        pconsole,
        console,
        alarm,
        ipc,
        scheduler,
        systick: cortexm7::systick::SysTick::new_with_calibration(100_000_000),
    };
    (board_kernel, platform, chip)
}
