//! Board file for SiFive HiFive1b RISC-V development platform.
//!
//! - <https://www.sifive.com/boards/hifive1-rev-b>
//!
//! This board file is only compatible with revision B of the HiFive1.

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::time::Alarm;
use kernel::Chip;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};
use rv32i::csr;

pub mod io;

pub const NUM_PROCS: usize = 4;
//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

// Reference to the chip for panic dumps.
static mut CHIP: Option<
    &'static e310x::chip::E310x<VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>>,
> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x900] = [0; 0x900];

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct HiFive1 {
    led: &'static capsules::led::LED<'static, sifive::gpio::GpioPin<'static>>,
    console: &'static capsules::console::Console<'static>,
    lldb: &'static capsules::low_level_debug::LowLevelDebug<
        'static,
        capsules::virtual_uart::UartDevice<'static>,
    >,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer<'static>>,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for HiFive1 {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::low_level_debug::DRIVER_NUM => f(Some(self.lldb)),
            _ => f(None),
        }
    }
}

/// Reset Handler.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Basic setup of the platform.
    rv32i::init_memory();
    // only machine mode
    rv32i::configure_trap_handler(rv32i::PermissionMode::Machine);

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    e310x::watchdog::WATCHDOG.disable();
    e310x::rtc::RTC.disable();
    e310x::pwm::PWM0.disable();
    e310x::pwm::PWM1.disable();
    e310x::pwm::PWM2.disable();

    e310x::prci::PRCI.set_clock_frequency(sifive::prci::ClockFrequency::Freq16Mhz);

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&e310x::gpio::PORT[22]), // Red
        None,
        None,
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &e310x::uart::UART0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // LEDs
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        sifive::gpio::GpioPin,
        (
            &e310x::gpio::PORT[22], // Red
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &e310x::gpio::PORT[19], // Green
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &e310x::gpio::PORT[21], // Blue
            kernel::hil::gpio::ActivationMode::ActiveLow
        )
    ))
    .finalize(components::led_component_buf!(sifive::gpio::GpioPin));

    e310x::uart::UART0.initialize_gpio_pins(&e310x::gpio::PORT[17], &e310x::gpio::PORT[16]);

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
        MuxAlarm::new(&e310x::timer::MACHINETIMER)
    );
    hil::time::Alarm::set_alarm_client(&e310x::timer::MACHINETIMER, mux_alarm);

    // Alarm
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let systick_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
        >,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(&memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    let chip = static_init!(
        e310x::chip::E310x<VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>>,
        e310x::chip::E310x::new(systick_virtual_alarm)
    );
    systick_virtual_alarm.set_alarm_client(chip.scheduler_timer());
    CHIP = Some(chip);

    // Need to enable all interrupts for Tock Kernel
    chip.enable_plic_interrupts();

    // enable interrupts globally
    csr::CSR
        .mie
        .modify(csr::mie::mie::mext::SET + csr::mie::mie::msoft::SET + csr::mie::mie::mtimer::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let lldb = components::lldb::LowLevelDebugComponent::new(board_kernel, uart_mux).finalize(());

    // Need two debug!() calls to actually test with QEMU. QEMU seems to have
    // a much larger UART TX buffer (or it transmits faster).
    debug!("HiFive1 initialization complete.");
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

    let hifive1 = HiFive1 {
        console: console,
        alarm: alarm,
        lldb: lldb,
        led,
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
    board_kernel.kernel_loop(&hifive1, chip, None, scheduler, &main_loop_cap);
}
