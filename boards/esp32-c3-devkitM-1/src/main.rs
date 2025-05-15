// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for ESP32-C3 RISC-V development platform.
//!

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::ptr::{addr_of, addr_of_mut};

use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use esp32_c3::chip::Esp32C3DefaultPeripherals;
use kernel::capabilities;
use kernel::component::Component;
use kernel::platform::scheduler_timer::VirtualSchedulerTimer;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::priority::PrioritySched;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::{create_capability, debug, hil, static_init};
use rv32i::csr;

pub mod io;

#[cfg(test)]
mod tests;

const NUM_PROCS: usize = 4;
//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

// Reference to the chip for panic dumps.
static mut CHIP: Option<&'static esp32_c3::chip::Esp32C3<Esp32C3DefaultPeripherals>> = None;
// Static reference to process printer for panic dumps.
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Test access to the peripherals
#[cfg(test)]
static mut PERIPHERALS: Option<&'static Esp32C3DefaultPeripherals> = None;
// Test access to scheduler
#[cfg(test)]
static mut SCHEDULER: Option<&PrioritySched> = None;
// Test access to board
#[cfg(test)]
static mut BOARD: Option<&'static kernel::Kernel> = None;
// Test access to platform
#[cfg(test)]
static mut PLATFORM: Option<&'static Esp32C3Board> = None;
// Test access to main loop capability
#[cfg(test)]
static mut MAIN_CAP: Option<&dyn kernel::capabilities::MainLoopCapability> = None;
// Test access to alarm
static mut ALARM: Option<&'static MuxAlarm<'static, esp32_c3::timg::TimG<'static>>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x900] = [0; 0x900];

type RngDriver = components::rng::RngComponentType<esp32_c3::rng::Rng<'static>>;

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct Esp32C3Board {
    gpio: &'static capsules_core::gpio::GPIO<'static, esp32::gpio::GpioPin<'static>>,
    console: &'static capsules_core::console::Console<'static>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, esp32_c3::timg::TimG<'static>>,
    >,
    scheduler: &'static PrioritySched,
    scheduler_timer: &'static VirtualSchedulerTimer<esp32_c3::timg::TimG<'static>>,
    rng: &'static RngDriver,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for Esp32C3Board {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::rng::DRIVER_NUM => f(Some(self.rng)),
            _ => f(None),
        }
    }
}

impl KernelResources<esp32_c3::chip::Esp32C3<'static, Esp32C3DefaultPeripherals<'static>>>
    for Esp32C3Board
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type ContextSwitchCallback = ();
    type Scheduler = PrioritySched;
    type SchedulerTimer = VirtualSchedulerTimer<esp32_c3::timg::TimG<'static>>;
    type WatchDog = ();

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

unsafe fn setup() -> (
    &'static kernel::Kernel,
    &'static Esp32C3Board,
    &'static esp32_c3::chip::Esp32C3<'static, Esp32C3DefaultPeripherals<'static>>,
    &'static Esp32C3DefaultPeripherals<'static>,
) {
    use esp32_c3::sysreg::{CpuFrequency, PllFrequency};

    // only machine mode
    rv32i::configure_trap_handler();

    let peripherals = static_init!(Esp32C3DefaultPeripherals, Esp32C3DefaultPeripherals::new());

    peripherals.timg0.disable_wdt();
    peripherals.rtc_cntl.disable_wdt();
    peripherals.rtc_cntl.disable_super_wdt();
    peripherals.rtc_cntl.enable_fosc();
    peripherals.sysreg.disable_timg0();
    peripherals.sysreg.enable_timg0();

    peripherals
        .sysreg
        .use_pll_clock_source(PllFrequency::MHz320, CpuFrequency::MHz160);

    // initialise capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(None, None, None);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
        .finalize(components::uart_mux_component_static!());

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            esp32::gpio::GpioPin,
            0 => &peripherals.gpio[0],
            1 => &peripherals.gpio[1],
            2 => &peripherals.gpio[2],
            3 => &peripherals.gpio[3],
            4 => &peripherals.gpio[4],
            5 => &peripherals.gpio[5],
            6 => &peripherals.gpio[6],
            7 => &peripherals.gpio[7],
            8 => &peripherals.gpio[15]
        ),
    )
    .finalize(components::gpio_component_static!(esp32::gpio::GpioPin));

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, esp32_c3::timg::TimG>,
        MuxAlarm::new(&peripherals.timg0)
    );
    hil::time::Alarm::set_alarm_client(&peripherals.timg0, mux_alarm);

    ALARM = Some(mux_alarm);

    // Alarm
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, esp32_c3::timg::TimG>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    virtual_alarm_user.setup();

    let alarm = static_init!(
        capsules_core::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, esp32_c3::timg::TimG>>,
        capsules_core::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(capsules_core::alarm::DRIVER_NUM, &memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_alarm_client(virtual_alarm_user, alarm);

    let scheduler_timer = static_init!(
        VirtualSchedulerTimer<esp32_c3::timg::TimG<'static>>,
        VirtualSchedulerTimer::new(&peripherals.timg1)
    );

    let chip = static_init!(
        esp32_c3::chip::Esp32C3<
            Esp32C3DefaultPeripherals,
        >,
        esp32_c3::chip::Esp32C3::new(peripherals)
    );
    CHIP = Some(chip);

    // Need to enable all interrupts for Tock Kernel
    chip.map_pic_interrupts();
    chip.enable_pic_interrupts();

    // enable interrupts globally
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

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

    // Create process printer for panic.
    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    debug!("ESP32-C3 initialisation complete.");
    debug!("Entering main loop.");

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

    let scheduler = components::sched::priority::PriorityComponent::new(board_kernel)
        .finalize(components::priority_component_static!());

    // PROCESS CONSOLE
    let process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
        None,
    )
    .finalize(components::process_console_component_static!(
        esp32_c3::timg::TimG
    ));
    let _ = process_console.start();

    let rng = components::rng::RngComponent::new(
        board_kernel,
        capsules_core::rng::DRIVER_NUM,
        &peripherals.rng,
    )
    .finalize(components::rng_component_static!(esp32_c3::rng::Rng));

    let esp32_c3_board = static_init!(
        Esp32C3Board,
        Esp32C3Board {
            gpio,
            console,
            alarm,
            scheduler,
            scheduler_timer,
            rng,
        }
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
        &mut *addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    peripherals.init();

    (board_kernel, esp32_c3_board, chip, peripherals)
}

/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup and RAM initialization.
#[no_mangle]
pub unsafe fn main() {
    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    {
        let (board_kernel, esp32_c3_board, chip, _peripherals) = setup();

        let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

        board_kernel.kernel_loop(
            esp32_c3_board,
            chip,
            None::<&kernel::ipc::IPC<0>>,
            &main_loop_cap,
        );
    }
}

#[cfg(test)]
use kernel::platform::watchdog::WatchDog;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    unsafe {
        let (board_kernel, esp32_c3_board, _chip, peripherals) = setup();

        BOARD = Some(board_kernel);
        PLATFORM = Some(&esp32_c3_board);
        PERIPHERALS = Some(peripherals);
        SCHEDULER = Some(
            components::sched::priority::PriorityComponent::new(board_kernel)
                .finalize(components::priority_component_static!()),
        );
        MAIN_CAP = Some(&create_capability!(capabilities::MainLoopCapability));

        PLATFORM.map(|p| {
            p.watchdog().setup();
        });

        for test in tests {
            test();
        }
    }

    loop {}
}
