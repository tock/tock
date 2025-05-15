// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for a LiteX-built VexRiscv-based SoC synthesized for a
//! Digilent Arty-A7 FPGA board

#![no_std]
#![no_main]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::time::{Alarm, Timer};
use kernel::platform::chip::InterruptService;
use kernel::platform::scheduler_timer::VirtualSchedulerTimer;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::mlfq::MLFQSched;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::StaticRef;
use kernel::{create_capability, debug, static_init};
use rv32i::csr;

mod io;
mod litex_generated_constants;

// This module contains the LiteX SoC configuration options, register
// positions, interrupt mappings and other implementation details of
// the generated bitstream.
//
// Its values are used throughout the file, hence import it under a
// short name.
use litex_generated_constants as socc;

/// Structure for dynamic interrupt mapping, depending on the SoC
/// configuration
///
/// This struct is deliberately kept in the board crate. Because of
/// the configurable nature of LiteX, it does not make sense to define
/// a default interrupt mapping, as the interrupt numbers are
/// generated sequentially for all softcores.
struct LiteXArtyInterruptablePeripherals {
    uart0: &'static litex_vexriscv::uart::LiteXUart<'static, socc::SoCRegisterFmt>,
    timer0: &'static litex_vexriscv::timer::LiteXTimer<
        'static,
        socc::SoCRegisterFmt,
        socc::ClockFrequency,
    >,
    ethmac0: &'static litex_vexriscv::liteeth::LiteEth<
        'static,
        { socc::ETHMAC_TX_SLOTS },
        socc::SoCRegisterFmt,
    >,
}

impl LiteXArtyInterruptablePeripherals {
    // Resolve any recursive dependencies and set up deferred calls:
    pub fn init(&'static self) {
        kernel::deferred_call::DeferredCallClient::register(self.uart0);
    }
}

impl InterruptService for LiteXArtyInterruptablePeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt as usize {
            socc::UART_INTERRUPT => {
                self.uart0.service_interrupt();
                true
            }
            socc::TIMER0_INTERRUPT => {
                self.timer0.service_interrupt();
                true
            }
            socc::ETHMAC_INTERRUPT => {
                self.ethmac0.service_interrupt();
                true
            }
            _ => false,
        }
    }
}

const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures. Need an
// empty list at least.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

// Reference to the chip, led controller, UART hardware, and process printer for
// panic dumps.
struct LiteXArtyPanicReferences {
    chip: Option<&'static litex_vexriscv::chip::LiteXVexRiscv<LiteXArtyInterruptablePeripherals>>,
    uart: Option<&'static litex_vexriscv::uart::LiteXUart<'static, socc::SoCRegisterFmt>>,
    led_controller:
        Option<&'static litex_vexriscv::led_controller::LiteXLedController<socc::SoCRegisterFmt>>,
    process_printer: Option<&'static capsules_system::process_printer::ProcessPrinterText>,
}
static mut PANIC_REFERENCES: LiteXArtyPanicReferences = LiteXArtyPanicReferences {
    chip: None,
    uart: None,
    led_controller: None,
    process_printer: None,
};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct LiteXArty {
    led_driver: &'static capsules_core::led::LedDriver<
        'static,
        litex_vexriscv::led_controller::LiteXLed<'static, socc::SoCRegisterFmt>,
        4,
    >,
    console: &'static capsules_core::console::Console<'static>,
    pconsole: &'static capsules_core::process_console::ProcessConsole<
        'static,
        { capsules_core::process_console::DEFAULT_COMMAND_HISTORY_LEN },
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
        components::process_console::Capability,
    >,
    lldb: &'static capsules_core::low_level_debug::LowLevelDebug<
        'static,
        capsules_core::virtualizers::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
    >,
    ipc: kernel::ipc::IPC<{ NUM_PROCS as u8 }>,
    scheduler: &'static MLFQSched<
        'static,
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
    >,
    scheduler_timer: &'static VirtualSchedulerTimer<
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls
impl SyscallDriverLookup for LiteXArty {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::led::DRIVER_NUM => f(Some(self.led_driver)),
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<litex_vexriscv::chip::LiteXVexRiscv<LiteXArtyInterruptablePeripherals>>
    for LiteXArty
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = MLFQSched<
        'static,
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
    >;
    type SchedulerTimer = VirtualSchedulerTimer<
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
    >;
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
        self.scheduler_timer
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
unsafe fn start() -> (
    &'static kernel::Kernel,
    LiteXArty,
    &'static litex_vexriscv::chip::LiteXVexRiscv<LiteXArtyInterruptablePeripherals>,
) {
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
        /// The start of the kernel text (Included only for kernel PMP)
        static _stext: u8;
        /// The end of the kernel text (Included only for kernel PMP)
        static _etext: u8;
        /// The start of the kernel / app / storage flash (Included only for kernel PMP)
        static _sflash: u8;
        /// The end of the kernel / app / storage flash (Included only for kernel PMP)
        static _eflash: u8;
        /// The start of the kernel / app RAM (Included only for kernel PMP)
        static _ssram: u8;
        /// The end of the kernel / app RAM (Included only for kernel PMP)
        static _esram: u8;
    }

    // ---------- BASIC INITIALIZATION ----------

    // Basic setup of the riscv platform.
    rv32i::configure_trap_handler();

    // Set up memory protection immediately after setting the trap handler, to
    // ensure that much of the board initialization routine runs with PMP kernel
    // memory protection.
    let pmp = rv32i::pmp::kernel_protection::KernelProtectionPMP::new(
        rv32i::pmp::kernel_protection::FlashRegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_end(
                core::ptr::addr_of!(_sflash),
                core::ptr::addr_of!(_eflash),
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection::RAMRegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_end(
                core::ptr::addr_of!(_ssram),
                core::ptr::addr_of!(_esram),
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection::MMIORegion(
            rv32i::pmp::NAPOTRegionSpec::from_start_size(
                0xf0000000 as *const u8, // start
                0x10000000,              // size
            )
            .unwrap(),
        ),
        rv32i::pmp::kernel_protection::KernelTextRegion(
            rv32i::pmp::TORRegionSpec::from_start_end(
                core::ptr::addr_of!(_stext),
                core::ptr::addr_of!(_etext),
            )
            .unwrap(),
        ),
    )
    .unwrap();

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    // ---------- LED CONTROLLER HARDWARE ----------

    // Initialize the LEDs, stopping any patterns from the bootloader
    // / bios still running in HW and turn them all off
    let led0 = static_init!(
        litex_vexriscv::led_controller::LiteXLedController<socc::SoCRegisterFmt>,
        litex_vexriscv::led_controller::LiteXLedController::new(
            StaticRef::new(
                socc::CSR_LEDS_BASE
                    as *const litex_vexriscv::led_controller::LiteXLedRegisters<
                        socc::SoCRegisterFmt,
                    >
            ),
            4, // 4 LEDs on this board
        )
    );
    led0.initialize();

    PANIC_REFERENCES.led_controller = Some(led0);

    // --------- TIMER & UPTIME CORE; ALARM INITIALIZATION ----------

    // Initialize the hardware timer
    let timer0 = static_init!(
        litex_vexriscv::timer::LiteXTimer<'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
        litex_vexriscv::timer::LiteXTimer::new(StaticRef::new(
            socc::CSR_TIMER0_BASE
                as *const litex_vexriscv::timer::LiteXTimerRegisters<socc::SoCRegisterFmt>
        ),)
    );

    // The SoC is expected to feature the 64-bit uptime extension to the timer hardware
    let timer0_uptime = static_init!(
        litex_vexriscv::timer::LiteXTimerUptime<
            'static,
            socc::SoCRegisterFmt,
            socc::ClockFrequency,
        >,
        litex_vexriscv::timer::LiteXTimerUptime::new(timer0)
    );

    // Create the LiteXAlarm based on the hardware LiteXTimer core and
    // the uptime peripheral
    let litex_alarm = static_init!(
        litex_vexriscv::timer::LiteXAlarm<
            'static,
            'static,
            socc::SoCRegisterFmt,
            socc::ClockFrequency,
        >,
        litex_vexriscv::timer::LiteXAlarm::new(timer0_uptime, timer0)
    );
    timer0.set_timer_client(litex_alarm);
    litex_alarm.initialize();

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
        MuxAlarm::new(litex_alarm)
    );
    litex_alarm.set_alarm_client(mux_alarm);

    // Userspace alarm driver
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
        VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_user.setup();

    let alarm = static_init!(
        capsules_core::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<
                'static,
                litex_vexriscv::timer::LiteXAlarm<
                    'static,
                    'static,
                    socc::SoCRegisterFmt,
                    socc::ClockFrequency,
                >,
            >,
        >,
        capsules_core::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules_core::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    virtual_alarm_user.set_alarm_client(alarm);

    // Systick virtual alarm for scheduling
    let systick_virtual_alarm = static_init!(
        VirtualMuxAlarm<
            'static,
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
        >,
        VirtualMuxAlarm::new(mux_alarm)
    );
    systick_virtual_alarm.setup();

    let scheduler_timer = static_init!(
        VirtualSchedulerTimer<
            VirtualMuxAlarm<
                'static,
                litex_vexriscv::timer::LiteXAlarm<
                    'static,
                    'static,
                    socc::SoCRegisterFmt,
                    socc::ClockFrequency,
                >,
            >,
        >,
        VirtualSchedulerTimer::new(systick_virtual_alarm)
    );

    // ---------- UART ----------

    // Initialize the HW UART
    let uart0 = static_init!(
        litex_vexriscv::uart::LiteXUart<socc::SoCRegisterFmt>,
        litex_vexriscv::uart::LiteXUart::new(
            StaticRef::new(
                socc::CSR_UART_BASE
                    as *const litex_vexriscv::uart::LiteXUartRegisters<socc::SoCRegisterFmt>,
            ),
            // No UART PHY CSR present, thus baudrate fixed in
            // hardware. Change with --uart-baudrate during SoC
            // generation. Fixed to 1MBd.
            None,
        )
    );
    uart0.initialize();

    PANIC_REFERENCES.uart = Some(uart0);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(uart0, socc::UART_BAUDRATE)
        .finalize(components::uart_mux_component_static!());

    // ---------- ETHERNET ----------

    // ETHMAC peripheral
    let ethmac0 = static_init!(
        litex_vexriscv::liteeth::LiteEth<{socc::ETHMAC_TX_SLOTS}, socc::SoCRegisterFmt>,
        litex_vexriscv::liteeth::LiteEth::new(
            StaticRef::new(
                socc::CSR_ETHMAC_BASE
                    as *const litex_vexriscv::liteeth::LiteEthMacRegisters<socc::SoCRegisterFmt>,
            ),
            socc::MEM_ETHMAC_BASE,
            socc::MEM_ETHMAC_SIZE,
            socc::ETHMAC_SLOT_SIZE,
            socc::ETHMAC_RX_SLOTS,
            socc::ETHMAC_TX_SLOTS,
        )
    );

    // Initialize the ETHMAC controller
    ethmac0.initialize();

    // ---------- LED DRIVER ----------

    // LEDs
    let led_driver =
        components::led::LedsComponent::new().finalize(components::led_component_static!(
            litex_vexriscv::led_controller::LiteXLed<'static, socc::SoCRegisterFmt>,
            led0.get_led(0).unwrap(),
            led0.get_led(1).unwrap(),
            led0.get_led(2).unwrap(),
            led0.get_led(3).unwrap(),
        ));

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ----------

    let interrupt_service = static_init!(
        LiteXArtyInterruptablePeripherals,
        LiteXArtyInterruptablePeripherals {
            uart0,
            timer0,
            ethmac0,
        }
    );
    interrupt_service.init();

    let chip = static_init!(
        litex_vexriscv::chip::LiteXVexRiscv<
            LiteXArtyInterruptablePeripherals,
        >,
        litex_vexriscv::chip::LiteXVexRiscv::new(
            "LiteX on Arty A7",
            interrupt_service,
            pmp,
        )
    );

    PANIC_REFERENCES.chip = Some(chip);

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());

    PANIC_REFERENCES.process_printer = Some(process_printer);

    // Enable RISC-V interrupts globally
    csr::CSR
        .mie
        .modify(csr::mie::mie::mext::SET + csr::mie::mie::msoft::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // Unmask all interrupt sources in the interrupt controller
    chip.unmask_interrupts();

    // Setup the process console.
    let pconsole = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        None,
    )
    .finalize(components::process_console_component_static!(
        litex_vexriscv::timer::LiteXAlarm<
            'static,
            'static,
            socc::SoCRegisterFmt,
            socc::ClockFrequency,
        >
    ));

    // Setup the console.
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());

    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(
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

    let scheduler = components::sched::mlfq::MLFQComponent::new(mux_alarm, &*addr_of!(PROCESSES))
        .finalize(components::mlfq_component_static!(
            litex_vexriscv::timer::LiteXAlarm<
                'static,
                'static,
                socc::SoCRegisterFmt,
                socc::ClockFrequency,
            >,
            NUM_PROCS
        ));

    let litex_arty = LiteXArty {
        console,
        pconsole,
        alarm,
        lldb,
        led_driver,
        scheduler,
        scheduler_timer,
        ipc: kernel::ipc::IPC::new(
            board_kernel,
            kernel::ipc::DRIVER_NUM,
            &memory_allocation_cap,
        ),
    };

    debug!("LiteX+VexRiscv on ArtyA7: initialization complete, entering main loop.");
    let _ = litex_arty.pconsole.start();

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
        &mut *addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, litex_arty, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, board, chip) = start();
    board_kernel.kernel_loop(&board, chip, Some(&board.ipc), &main_loop_capability);
}
