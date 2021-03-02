//! Board file for SiFive HiFive1b RISC-V development platform.
//!
//! - <https://www.sifive.com/boards/hifive1-rev-b>
//!
//! This board file is only compatible with revision B of the HiFive1.

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use e310x::chip::E310xDefaultPeripherals;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::led::LedLow;
use kernel::hil::time::Alarm;
use kernel::Chip;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};
use rv32i::csr;

pub mod io;

#[allow(dead_code)]
mod multi_alarm_test;

pub const NUM_PROCS: usize = 4;
//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

// Reference to the chip for panic dumps.
static mut CHIP: Option<
    &'static e310x::chip::E310x<
        VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
        E310xDefaultPeripherals,
    >,
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
    led:
        &'static capsules::led::LedDriver<'static, LedLow<'static, sifive::gpio::GpioPin<'static>>>,
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

    let peripherals = static_init!(E310xDefaultPeripherals, E310xDefaultPeripherals::new());

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    peripherals.watchdog.disable();
    peripherals.rtc.disable();
    peripherals.pwm0.disable();
    peripherals.pwm1.disable();
    peripherals.pwm2.disable();

    peripherals
        .prci
        .set_clock_frequency(sifive::prci::ClockFrequency::Freq16Mhz);

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
        Some(&peripherals.gpio_port[22]), // Red
        None,
        None,
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &peripherals.uart0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // LEDs
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        LedLow<'static, sifive::gpio::GpioPin>,
        LedLow::new(&peripherals.gpio_port[22]), // Red
        LedLow::new(&peripherals.gpio_port[19]), // Green
        LedLow::new(&peripherals.gpio_port[21]), // Blue
    ))
    .finalize(components::led_component_buf!(
        LedLow<'static, sifive::gpio::GpioPin>
    ));

    peripherals
        .uart0
        .initialize_gpio_pins(&peripherals.gpio_port[17], &peripherals.gpio_port[16]);

    let hardware_timer = static_init!(
        rv32i::machine_timer::MachineTimer,
        rv32i::machine_timer::MachineTimer::new(e310x::timer::MTIME_BASE)
    );

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
        MuxAlarm::new(hardware_timer)
    );
    hil::time::Alarm::set_alarm_client(hardware_timer, mux_alarm);

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
            board_kernel.create_grant(capsules::alarm::DRIVER_NUM as u32, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    let chip = static_init!(
        e310x::chip::E310x<
            VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
            E310xDefaultPeripherals,
        >,
        e310x::chip::E310x::new(systick_virtual_alarm, peripherals, hardware_timer)
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
    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules::console::DRIVER_NUM as u32,
        uart_mux,
    )
    .finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let lldb = components::lldb::LowLevelDebugComponent::new(
        board_kernel,
        capsules::low_level_debug::DRIVER_NUM as u32,
        uart_mux,
    )
    .finalize(());

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
    board_kernel.kernel_loop(
        &hifive1,
        chip,
        None::<&kernel::ipc::IPC<NUM_PROCS>>,
        scheduler,
        &main_loop_cap,
    );
}
