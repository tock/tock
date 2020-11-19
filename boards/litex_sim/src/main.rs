//! Board file for a LiteX SoC running in a Verilated simulation

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::common::StaticRef;
use kernel::component::Component;
use kernel::hil::time::{Alarm, Timer};
use kernel::Chip;
use kernel::InterruptService;
use kernel::Platform;
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
struct LiteXSimInterruptablePeripherals {
    uart0: &'static litex_vexriscv::uart::LiteXUart<'static, socc::SoCRegisterFmt>,
    timer0: &'static litex_vexriscv::timer::LiteXTimer<
        'static,
        socc::SoCRegisterFmt,
        socc::ClockFrequency,
    >,
    ethmac0: &'static litex_vexriscv::liteeth::LiteEth<'static, socc::SoCRegisterFmt>,
}

impl InterruptService<()> for LiteXSimInterruptablePeripherals {
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

    unsafe fn service_deferred_call(&self, _task: ()) -> bool {
        false
    }
}

const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures. Need an
// empty list at least.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

// Reference to the chip and UART hardware for panic dumps
struct LiteXSimPanicReferences {
    chip: Option<
        &'static litex_vexriscv::chip::LiteXVexRiscv<
            VirtualMuxAlarm<
                'static,
                litex_vexriscv::timer::LiteXAlarm<
                    'static,
                    'static,
                    socc::SoCRegisterFmt,
                    socc::ClockFrequency,
                >,
            >,
            LiteXSimInterruptablePeripherals,
        >,
    >,
    uart: Option<&'static litex_vexriscv::uart::LiteXUart<'static, socc::SoCRegisterFmt>>,
}
static mut PANIC_REFERENCES: LiteXSimPanicReferences = LiteXSimPanicReferences {
    chip: None,
    uart: None,
};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct LiteXSim {
    console: &'static capsules::console::Console<'static>,
    lldb: &'static capsules::low_level_debug::LowLevelDebug<
        'static,
        capsules::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static capsules::alarm::AlarmDriver<
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
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for LiteXSim {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            _ => f(None),
        }
    }
}

/// Reset Handler.
///
/// This function is called from the arch crate after some very basic
/// RISC-V setup.
#[no_mangle]
pub unsafe fn reset_handler() {
    // ---------- BASIC INITIALIZATION ----------
    // Basic setup of the riscv platform.
    rv32i::init_memory();
    rv32i::configure_trap_handler(rv32i::PermissionMode::Machine);

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

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
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
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
        capsules::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(&memory_allocation_cap)
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

    // ---------- UART ----------

    // Initialize the HW UART
    let uart0 = static_init!(
        litex_vexriscv::uart::LiteXUart<socc::SoCRegisterFmt>,
        litex_vexriscv::uart::LiteXUart::new(
            StaticRef::new(
                socc::CSR_UART_BASE
                    as *const litex_vexriscv::uart::LiteXUartRegisters<socc::SoCRegisterFmt>,
            ),
            None, // LiteX simulator has no UART phy
            dynamic_deferred_caller,
        )
    );
    uart0.initialize(
        dynamic_deferred_caller
            .register(uart0)
            .expect("dynamic deferred caller out of slots"),
    );

    PANIC_REFERENCES.uart = Some(uart0);

    // Create a shared UART channel for the console and for kernel debug.
    //
    // The baudrate is ingnored, as no UART phy is present in the
    // verilated simulation.
    let uart_mux =
        components::console::UartMuxComponent::new(uart0, 115200, dynamic_deferred_caller)
            .finalize(());

    // ---------- ETHERNET ----------

    // Packet receive buffer
    let ethmac0_rxbuf0 = static_init!([u8; 1522], [0; 1522]);

    // ETHMAC peripheral
    let ethmac0 = static_init!(
        litex_vexriscv::liteeth::LiteEth<socc::SoCRegisterFmt>,
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
            ethmac0_rxbuf0,
        )
    );

    // Initialize the ETHMAC controller
    ethmac0.initialize();

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ----------

    let interrupt_service = static_init!(
        LiteXSimInterruptablePeripherals,
        LiteXSimInterruptablePeripherals {
            timer0,
            uart0,
            ethmac0,
        }
    );

    let chip = static_init!(
        litex_vexriscv::chip::LiteXVexRiscv<
            VirtualMuxAlarm<
                'static,
                litex_vexriscv::timer::LiteXAlarm<
                    'static,
                    'static,
                    socc::SoCRegisterFmt,
                    socc::ClockFrequency,
                >,
            >,
            LiteXSimInterruptablePeripherals,
        >,
        litex_vexriscv::chip::LiteXVexRiscv::new(
            "Verilated LiteX on VexRiscv",
            systick_virtual_alarm,
            interrupt_service
        )
    );
    systick_virtual_alarm.set_alarm_client(chip.scheduler_timer());

    PANIC_REFERENCES.chip = Some(chip);

    // Enable and unmask interrupts
    chip.enable_interrupts();

    // Enable RISC-V interrupts globally
    csr::CSR
        .mie
        .modify(csr::mie::mie::mext::SET + csr::mie::mie::msoft::SET + csr::mie::mie::mtimer::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let lldb = components::lldb::LowLevelDebugComponent::new(board_kernel, uart_mux).finalize(());

    debug!("Verilated LiteX+VexRiscv: initialization complete, entering main loop.");

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

    let litex_sim = LiteXSim {
        console: console,
        alarm: alarm,
        lldb: lldb,
    };

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
        FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    let scheduler = components::sched::cooperative::CooperativeComponent::new(&PROCESSES)
        .finalize(components::coop_component_helper!(NUM_PROCS));
    board_kernel.kernel_loop(&litex_sim, chip, None, scheduler, &main_loop_cap);
}
