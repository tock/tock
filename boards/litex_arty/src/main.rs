//! Board file for a LiteX-built VexRiscv-based SoC synthesized for a
//! Digilent Arty-A7 FPGA board

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
use kernel::hil::time::{Alarm, Frequency, Timer};
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
struct LiteXArtyInterruptablePeripherals {
    uart0: &'static litex::uart::LiteXUart<'static, socc::SoCRegisterFmt>,
    timer0: &'static litex::timer::LiteXTimer<'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
    ethmac0: &'static litex::liteeth::LiteEth<'static, socc::SoCRegisterFmt>,
}

impl InterruptService<()> for LiteXArtyInterruptablePeripherals {
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

// Reference to the chip, led controller and UART hardware for panic
// dumps
struct LiteXArtyPanicReferences {
    chip: Option<
        &'static litex_vexriscv::chip::LiteXVexRiscv<
            VirtualMuxAlarm<
                'static,
                litex::timer::LiteXAlarm<
                    'static,
                    'static,
                    socc::SoCRegisterFmt,
                    socc::ClockFrequency,
                >,
            >,
            LiteXArtyInterruptablePeripherals,
        >,
    >,
    uart: Option<&'static litex::uart::LiteXUart<'static, socc::SoCRegisterFmt>>,
    led_controller:
        Option<&'static litex::led_controller::LiteXLedController<socc::SoCRegisterFmt>>,
}
static mut PANIC_REFERENCES: LiteXArtyPanicReferences = LiteXArtyPanicReferences {
    chip: None,
    uart: None,
    led_controller: None,
};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct LiteXArty {
    led_driver: &'static capsules::led::LedDriver<
        'static,
        litex::led_controller::LiteXLed<'static, socc::SoCRegisterFmt>,
    >,
    console: &'static capsules::console::Console<'static>,
    lldb: &'static capsules::low_level_debug::LowLevelDebug<
        'static,
        capsules::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<
            'static,
            litex::timer::LiteXAlarm<'static, 'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
        >,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls
impl Platform for LiteXArty {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::led::DRIVER_NUM => f(Some(self.led_driver)),
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            _ => f(None),
        }
    }
}

/// Reset Handler
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup.
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

    // ---------- LED CONTROLLER HARDWARE ----------

    // Initialize the LEDs, stopping any patterns from the bootloader
    // / bios still running in HW and turn them all off
    let led0 = static_init!(
        litex::led_controller::LiteXLedController<socc::SoCRegisterFmt>,
        litex::led_controller::LiteXLedController::new(
            StaticRef::new(
                socc::CSR_LEDS_BASE
                    as *const litex::led_controller::LiteXLedRegisters<socc::SoCRegisterFmt>
            ),
            4,     // 4 LEDs on this board
            false, // The LEDs are active-high
        )
    );
    led0.initialize();

    PANIC_REFERENCES.led_controller = Some(led0);

    // --------- TIMER & UPTIME CORE; ALARM INITIALIZATION ----------

    // Initialize the hardware timer
    let timer0 = static_init!(
        litex::timer::LiteXTimer<'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
        litex::timer::LiteXTimer::new(StaticRef::new(
            socc::CSR_TIMER0_BASE as *const litex::timer::LiteXTimerRegisters<socc::SoCRegisterFmt>
        ),)
    );

    // The SoC is expected to feature the 64-bit uptime extension to the timer hardware
    let timer0_uptime = static_init!(
        litex::timer::LiteXTimerUptime<'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
        litex::timer::LiteXTimerUptime::new(timer0)
    );

    // Create the LiteXAlarm based on the hardware LiteXTimer core and
    // the uptime peripheral
    let litex_alarm = static_init!(
        litex::timer::LiteXAlarm<'static, 'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
        litex::timer::LiteXAlarm::new(timer0_uptime, timer0)
    );
    timer0.set_timer_client(litex_alarm);
    litex_alarm.initialize();

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<
            'static,
            litex::timer::LiteXAlarm<'static, 'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
        >,
        MuxAlarm::new(litex_alarm)
    );
    litex_alarm.set_alarm_client(mux_alarm);

    // Userspace alarm driver
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<
            'static,
            litex::timer::LiteXAlarm<'static, 'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
        >,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<
                'static,
                litex::timer::LiteXAlarm<
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
            litex::timer::LiteXAlarm<'static, 'static, socc::SoCRegisterFmt, socc::ClockFrequency>,
        >,
        VirtualMuxAlarm::new(mux_alarm)
    );

    // ---------- UART ----------

    // Initialize the HW UART
    let uart0 = static_init!(
        litex::uart::LiteXUart<socc::SoCRegisterFmt>,
        litex::uart::LiteXUart::new(
            StaticRef::new(
                socc::CSR_UART_BASE as *const litex::uart::LiteXUartRegisters<socc::SoCRegisterFmt>,
            ),
            Some((
                StaticRef::new(
                    socc::CSR_UART_PHY_BASE
                        as *const litex::uart::LiteXUartPhyRegisters<socc::SoCRegisterFmt>,
                ),
                socc::ClockFrequency::frequency()
            )),
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
    let uart_mux =
        components::console::UartMuxComponent::new(uart0, 1_000_000, dynamic_deferred_caller)
            .finalize(());

    // ---------- ETHERNET ----------

    // Packet receive buffer
    let ethmac0_rxbuf0 = static_init!([u8; 1522], [0; 1522]);

    // ETHMAC peripheral
    let ethmac0 = static_init!(
        litex::liteeth::LiteEth<socc::SoCRegisterFmt>,
        litex::liteeth::LiteEth::new(
            StaticRef::new(
                socc::CSR_ETHMAC_BASE
                    as *const litex::liteeth::LiteEthMacRegisters<socc::SoCRegisterFmt>,
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

    // ---------- LED DRIVER ----------

    // LEDs
    let led_driver = components::led::LedsComponent::new(components::led_component_helper!(
        litex::led_controller::LiteXLed<'static, socc::SoCRegisterFmt>,
        led0.get_led(0).unwrap(),
        led0.get_led(1).unwrap(),
        led0.get_led(2).unwrap(),
        led0.get_led(3).unwrap(),
    ))
    .finalize(components::led_component_buf!(
        litex::led_controller::LiteXLed<'static, socc::SoCRegisterFmt>
    ));

    // ---------- INITIALIZE CHIP, ENABLE INTERRUPTS ----------

    let interrupt_service = static_init!(
        LiteXArtyInterruptablePeripherals,
        LiteXArtyInterruptablePeripherals {
            timer0,
            uart0,
            ethmac0,
        }
    );

    let chip = static_init!(
        litex_vexriscv::chip::LiteXVexRiscv<
            VirtualMuxAlarm<
                'static,
                litex::timer::LiteXAlarm<
                    'static,
                    'static,
                    socc::SoCRegisterFmt,
                    socc::ClockFrequency,
                >,
            >,
            LiteXArtyInterruptablePeripherals,
        >,
        litex_vexriscv::chip::LiteXVexRiscv::new(
            "LiteX on Arty A7",
            systick_virtual_alarm,
            interrupt_service
        )
    );
    systick_virtual_alarm.set_alarm_client(chip.scheduler_timer());

    PANIC_REFERENCES.chip = Some(chip);

    // Enable and unmask all PLIC interrupts
    chip.enable_plic_interrupts();

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

    debug!("LiteX+VexRiscv on ArtyA7: initialization complete, entering main loop.");

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

    let litex_arty = LiteXArty {
        console: console,
        alarm: alarm,
        lldb: lldb,
        led_driver,
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
    board_kernel.kernel_loop(&litex_arty, chip, None, scheduler, &main_loop_cap);
}
