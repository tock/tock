// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Board library for NXP S32G3 SAIL.

#![no_std]
#![no_main]

pub mod io;
#[cfg(feature = "test-harness")]
mod tests;

use kernel::capabilities;
use kernel::component::Component;
use kernel::debug;
use kernel::debug::PanicResources;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil::time::Counter;
use kernel::platform::chip::Chip;
use kernel::platform::watchdog::WatchDog;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::utilities::single_thread_value::SingleThreadValue;
use kernel::{create_capability, static_init};
use nxp_s32g3::clocks::M7_CORE_FREQUENCY_HZ;
use nxp_s32g3::linflexd::LinFlexD;
use nxp_s32g3::siul2::AlternateFunction::{ALT1, ALT2};
use nxp_s32g3::stm::{Stm, STM_1_BASE};
use nxp_s32g3::swt::{Swt, SWT_0_BASE};
use nxp_s32g3::xrdc::xrdc_0::Peripheral::{LinFlexD0, LinFlexD1, Siul20};
use nxp_s32g3::xrdc::xrdc_0::{Config as Xrdc0Config, Pdac, Xrdc0};
use nxp_s32g3::xrdc::xrdc_1::{Mrgd as Xrdc1Mrgd, Xrdc1};
use nxp_s32g3::xrdc::{Access::FullRw, Domain, MrgdPatchOutcome, MrgdTarget, XrdcPatchError};
use nxp_s32g3::{mscm, ssramc};

/// Explicit no-op watchdog for the S32G3 SAIL bring-up board.
///
/// The S32G3 hardware watchdog (`Swt`) is present but intentionally left
/// disabled on this M7-only example board to simplify bring-up and avoid
/// spurious resets during development. This zero-sized type implements
/// [`WatchDog`] as a documented no-op so it is obvious from the platform
/// definition that no watchdog protection is active.
///
/// To enable a real watchdog, replace this with [`Swt`] (from
/// `nxp_s32g3::swt`) and ensure the kernel tickle cadence is wired.
pub struct DisabledWatchdog;

impl WatchDog for DisabledWatchdog {
    fn setup(&self) {}
}

pub const NUM_PROCS: usize = 4;

/// LF0 debug/process-console nominal baud rate.
const LF0_BAUD: u32 = 115_200;
/// LF1 userspace-console nominal baud rate.
const LF1_BAUD: u32 = 921_600;

/// XRDC_0 additive patch config. A prior boot stage owns the base policy;
/// Tock adds only M7_0 grants for its console and SIUL2 pin control.
/// CA53 remains in reset and receives no board-level XRDC grant.
const XRDC_0_CFG: Xrdc0Config = Xrdc0Config::new(
    &[],
    &[
        Pdac::new(LinFlexD0).grant(Domain::M7_0, FullRw),
        Pdac::new(LinFlexD1).grant(Domain::M7_0, FullRw),
        Pdac::new(Siul20).grant(Domain::M7_0, FullRw),
    ],
    &[],
);

/// Preparatory XRDC_1 policy for the 32 KiB standby-SRAM controller window.
/// A predecessor-owned descriptor is patched when present; a cold XMODEM boot
/// allocates this exact range only after excluding all overlap.
const XRDC_1_STDBY_SRAM: Xrdc1Mrgd = Xrdc1Mrgd::region(0x2400_0000, 0x2400_7FFF)
    .grant(Domain::Debugger, FullRw)
    .grant(Domain::M7_0, FullRw)
    .grant(Domain::M7_1, FullRw)
    .grant(Domain::M7_2, FullRw)
    .grant(Domain::M7_3, FullRw)
    .grant(Domain::EDma, FullRw)
    .grant(Domain::Hse, FullRw)
    .grant(Domain::A53, FullRw);

#[derive(Copy, Clone)]
pub struct NxpS32g3DefaultPeripherals {
    pub lf0: &'static LinFlexD<'static>,
    pub lf1: &'static LinFlexD<'static>,
    pub stm: &'static Stm<'static>,
    pub mscm: &'static mscm::Mscm,
    pub clocks: &'static nxp_s32g3::clocks::Clocks,
    pub swt0: &'static Swt,
}

impl kernel::platform::chip::InterruptService for NxpS32g3DefaultPeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            mscm::LINFLEXD_0 => {
                self.lf0.handle_interrupt();
                true
            }
            mscm::LINFLEXD_1 => {
                self.lf1.handle_interrupt();
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

pub type ChipHw = nxp_s32g3::chip::S32g3<NxpS32g3DefaultPeripherals>;
pub type ProcessPrinterInUse = capsules_system::process_printer::ProcessPrinterText;
pub type SchedulerInUse = components::sched::round_robin::RoundRobinComponentType;

pub static PANIC_RESOURCES: SingleThreadValue<PanicResources<ChipHw, ProcessPrinterInUse>> =
    SingleThreadValue::new();

kernel::stack_size! {0x8000}
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
    pub watchdog: &'static DisabledWatchdog,
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
    type WatchDog = DisabledWatchdog;
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
        self.watchdog
    }

    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}
// Prevent inlining so the stack frame is easy to identify in backtraces.
#[inline(never)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    NxpS32g3SailPlatform,
    &'static ChipHw,
) {
    ChipHw::init();
    kernel::deferred_call::initialize_deferred_call_state::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    let swt0: &'static Swt = static_init!(Swt, Swt::new(SWT_0_BASE));
    swt0.setup();

    // Clock and power
    //
    // PRTN2 (PFE/HSE) and PRTN3 (LLCE / LinFlexD1) are current boot
    // dependencies. PRTN1 (CA53) remains in reset.
    nxp_s32g3::mc_me::partition_enable(2).expect("PRTN2 enable failed");
    nxp_s32g3::mc_me::partition_enable(3).expect("PRTN3 enable failed");

    let clocks = static_init!(nxp_s32g3::clocks::Clocks, nxp_s32g3::clocks::Clocks::new());
    clocks.setup_m7_clocks().expect("M7 clock setup failed");

    // Interrupt routing
    //
    // Route all board peripherals to M7_0 so their NVIC enable() calls work.
    let mscm = static_init!(mscm::Mscm, mscm::Mscm::new());
    mscm.enable_interrupt(mscm::LINFLEXD_0, mscm::S32G3Core::M7_0)
        .expect("LINFlexD0 interrupt route failed");
    mscm.enable_interrupt(mscm::LINFLEXD_1, mscm::S32G3Core::M7_0)
        .expect("LINFlexD1 interrupt route failed");
    mscm.enable_interrupt(mscm::STM_1, mscm::S32G3Core::M7_0)
        .expect("STM1 interrupt route failed");
    cortexm7::nvic::Nvic::new(mscm::LINFLEXD_0).enable();
    cortexm7::nvic::Nvic::new(mscm::LINFLEXD_1).enable();
    cortexm7::nvic::Nvic::new(mscm::STM_1).enable();

    // Pinmux and UART bring-up
    let siul2_0 = nxp_s32g3::siul2::Siul2::new(nxp_s32g3::siul2::SIUL2_0_BASE);
    let siul2_1 = nxp_s32g3::siul2::Siul2::new(nxp_s32g3::siul2::SIUL2_1_BASE);

    // LinFlexD0 (LF0)
    siul2_0.setup_tx_pin(nxp_s32g3::siul2::Pin::PC9, ALT1);
    siul2_0.setup_rx_pin(nxp_s32g3::siul2::Pin::PC10);
    siul2_0.setup_imcr(
        nxp_s32g3::siul2::Imcr::LinflexD0Rx,
        nxp_s32g3::siul2::ImcrSource::Alt2,
    );
    // LinFlexD1 (LF1)
    siul2_0.setup_tx_pin(nxp_s32g3::siul2::Pin::PC8, ALT2);
    siul2_0.setup_rx_pin(nxp_s32g3::siul2::Pin::PC4);
    siul2_1.setup_imcr(
        nxp_s32g3::siul2::Imcr::LinflexD1Rx,
        nxp_s32g3::siul2::ImcrSource::Alt4,
    );

    let lf0 = static_init!(LinFlexD, LinFlexD::new_lf0());
    lf0.set_input_clock_hz(
        clocks
            .get_lin_baud_clk_hz()
            .expect("LIN baud clock must be configured"),
    );
    lf0.register();
    let lf1 = static_init!(LinFlexD, LinFlexD::new_lf1());
    lf1.set_input_clock_hz(
        clocks
            .get_lin_baud_clk_hz()
            .expect("LIN baud clock must be configured"),
    );
    lf1.register();

    // LF0 is the debug and process console.
    let uart_mux_lf0 = components::console::UartMuxComponent::new(lf0, LF0_BAUD)
        .finalize(components::uart_mux_component_static!());
    components::debug_writer::DebugWriterComponent::new::<
        <ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >(
        uart_mux_lf0,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!(
        2 /* buffer size kB */
    ));

    // Security
    //
    // A prior boot stage installs a base policy at boot. Tock only *adds* its
    // own grants via the XRDC driver's additive `patch()` mode so existing
    // clocks, UART, and watchdog grants stay intact.
    let xrdc0 = static_init!(Xrdc0, Xrdc0::new());
    xrdc0
        .patch(&XRDC_0_CFG)
        .expect("XRDC_0 M7 console and SIUL2 grants failed");
    // xrdc0.lock(); // Uncomment to freeze XRDC_0 policy until next reset.

    // Patch a predecessor-owned descriptor when present. A cold XMODEM boot
    // has no predecessor policy, so it can allocate only this exact range
    // after proving that no valid descriptor overlaps it.
    let xrdc1 = static_init!(Xrdc1, Xrdc1::new());
    match xrdc1.search_and_patch_mrgd(MrgdTarget::ContainsAddress(0x2400_6008), &XRDC_1_STDBY_SRAM)
    {
        Ok(MrgdPatchOutcome::PatchedExisting { .. }) => {}
        Ok(MrgdPatchOutcome::AllocatedNew { .. }) => {
            panic!("XRDC_1 containment search allocated standby-SRAM descriptor")
        }
        Err(XrdcPatchError::MissingTarget) => {
            match xrdc1.allocate_unmapped_exact_mrgd(&XRDC_1_STDBY_SRAM) {
                Ok(MrgdPatchOutcome::AllocatedNew { .. }) => {}
                Ok(MrgdPatchOutcome::PatchedExisting { .. }) => {
                    panic!("XRDC_1 cold-boot allocation patched standby-SRAM descriptor")
                }
                Err(error) => panic!(
                    "XRDC_1 cold-boot standby-SRAM allocation failed: {:?}",
                    error
                ),
            }
        }
        Err(error) => panic!(
            "XRDC_1 preparatory standby-SRAM descriptor patch failed: {:?}",
            error
        ),
    }

    // Initialize Standby SRAM ECC before anything can touch it.
    let ssramc = ssramc::Ssramc::new(ssramc::SSRAMC_BASE);
    ssramc.init().expect("SSRAMC ECC init failed");

    // Kernel structures
    let processes = components::process_array::ProcessArrayComponent::new()
        .finalize(components::process_array_component_static!(NUM_PROCS));

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(processes.as_slice()));

    let _ = PANIC_RESOURCES
        .bind_to_thread::<<ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider>(
            PanicResources::new(),
        );
    PANIC_RESOURCES.get().map(|resources| {
        resources.processes.put(processes.as_slice());
    });

    // Timer
    let stm = static_init!(Stm<'static>, Stm::new(STM_1_BASE));
    stm.start().unwrap();

    let watchdog = static_init!(DisabledWatchdog, DisabledWatchdog);

    // Peripherals and chip
    let peripherals = static_init!(
        NxpS32g3DefaultPeripherals,
        NxpS32g3DefaultPeripherals {
            lf0,
            lf1,
            stm,
            mscm,
            clocks,
            swt0,
        }
    );
    let chip = static_init!(ChipHw, ChipHw::new(peripherals));

    PANIC_RESOURCES.get().map(|resources| {
        resources.chip.put(chip);
    });

    // LF1 is the userspace console.
    let uart_mux_lf1 = components::console::UartMuxComponent::new(lf1, LF1_BAUD)
        .finalize(components::uart_mux_component_static!());
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux_lf1,
    )
    .finalize(components::console_component_static!());

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
        resources.printer.put(process_printer);
    });

    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux_lf0,
        mux_alarm,
        process_printer,
        None,
    )
    .finalize(components::process_console_component_static!(Stm));

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(processes)
        .finalize(components::round_robin_component_static!(NUM_PROCS));

    // Platform
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let platform = NxpS32g3SailPlatform {
        pconsole,
        console,
        alarm,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_cap,
        ),
        scheduler,
        systick: cortexm7::systick::SysTick::new_with_calibration(M7_CORE_FREQUENCY_HZ),
        watchdog,
    };

    // Process loading
    extern "C" {
        static _sapps: u8;
        static _eapps: u8;
        static mut _sappmem: u8;
        static _eappmem: u8;
    }
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    let fault_policy = static_init!(
        capsules_system::process_policies::PanicFaultPolicy,
        capsules_system::process_policies::PanicFaultPolicy {}
    );

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
        fault_policy,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    #[cfg(feature = "test-harness")]
    run_tests(mux_alarm, stm, *peripherals);

    (board_kernel, platform, chip)
}

/// # Safety: this function contains static initialization and must only be called once from `start()`.
///
#[cfg(feature = "test-harness")]
unsafe fn run_tests(
    mux_alarm: &'static capsules_core::virtualizers::virtual_alarm::MuxAlarm<'static, Stm<'static>>,
    stm: &'static Stm<'static>,
    peripherals: NxpS32g3DefaultPeripherals,
) {
    use kernel::hil::uart::Configure;
    peripherals.lf1.set_input_clock_hz(
        peripherals
            .clocks
            .get_lin_baud_clk_hz()
            .expect("LIN baud clock must be configured"),
    );
    peripherals
        .lf1
        .configure(kernel::hil::uart::Parameters {
            baud_rate: LF1_BAUD,
            width: kernel::hil::uart::Width::Eight,
            parity: kernel::hil::uart::Parity::None,
            stop_bits: kernel::hil::uart::StopBits::One,
            hw_flow_control: false,
        })
        .unwrap();

    tests::run(mux_alarm, stm, peripherals.lf1);
}
