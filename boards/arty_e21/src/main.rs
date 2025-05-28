// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for the SiFive E21 Bitstream running on the Arty FPGA

#![no_std]
#![no_main]

use core::ptr::{addr_of, addr_of_mut};

use arty_e21_chip::chip::ArtyExxDefaultPeripherals;
use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::scheduler::priority::PrioritySched;
use kernel::{create_capability, debug, static_init};

#[allow(dead_code)]
mod timer_test;

pub mod io;

// State for loading and holding applications.

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None, None, None, None];

// Reference to the chip for panic dumps.
static mut CHIP: Option<&'static arty_e21_chip::chip::ArtyExx<ArtyExxDefaultPeripherals>> = None;
static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> =
    None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct ArtyE21 {
    console: &'static capsules_core::console::Console<'static>,
    gpio: &'static capsules_core::gpio::GPIO<'static, arty_e21_chip::gpio::GpioPin<'static>>,
    alarm: &'static capsules_core::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, arty_e21_chip::chip::ArtyExxClint<'static>>,
    >,
    led: &'static capsules_core::led::LedDriver<
        'static,
        hil::led::LedHigh<'static, arty_e21_chip::gpio::GpioPin<'static>>,
        3,
    >,
    button: &'static capsules_core::button::Button<'static, arty_e21_chip::gpio::GpioPin<'static>>,
    // ipc: kernel::ipc::IPC<NUM_PROCS>,
    scheduler: &'static PrioritySched,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl SyscallDriverLookup for ArtyE21 {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::console::DRIVER_NUM => f(Some(self.console)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),

            capsules_core::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::button::DRIVER_NUM => f(Some(self.button)),

            // kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

impl KernelResources<arty_e21_chip::chip::ArtyExx<'static, ArtyExxDefaultPeripherals<'static>>>
    for ArtyE21
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = PrioritySched;
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

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
unsafe fn start() -> (
    &'static kernel::Kernel,
    ArtyE21,
    &'static arty_e21_chip::chip::ArtyExx<'static, ArtyExxDefaultPeripherals<'static>>,
) {
    let peripherals = static_init!(ArtyExxDefaultPeripherals, ArtyExxDefaultPeripherals::new());
    peripherals.init();

    let chip = static_init!(
        arty_e21_chip::chip::ArtyExx<ArtyExxDefaultPeripherals>,
        arty_e21_chip::chip::ArtyExx::new(&peripherals.machinetimer, peripherals)
    );
    CHIP = Some(chip);
    chip.initialize();

    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&*addr_of!(PROCESSES)));

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&peripherals.gpio_port[0]), // Blue
        Some(&peripherals.gpio_port[1]), // Green
        Some(&peripherals.gpio_port[8]),
    );

    let process_printer = components::process_printer::ProcessPrinterTextComponent::new()
        .finalize(components::process_printer_text_component_static!());
    PROCESS_PRINTER = Some(process_printer);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(&peripherals.uart0, 115200)
        .finalize(components::uart_mux_component_static!());

    let console = components::console::ConsoleComponent::new(
        board_kernel,
        capsules_core::console::DRIVER_NUM,
        uart_mux,
    )
    .finalize(components::console_component_static!());

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, arty_e21_chip::chip::ArtyExxClint>,
        MuxAlarm::new(&peripherals.machinetimer)
    );
    hil::time::Alarm::set_alarm_client(&peripherals.machinetimer, mux_alarm);

    // Alarm
    let alarm = components::alarm::AlarmDriverComponent::new(
        board_kernel,
        capsules_core::alarm::DRIVER_NUM,
        mux_alarm,
    )
    .finalize(components::alarm_component_static!(
        arty_e21_chip::chip::ArtyExxClint
    ));

    // TEST for timer
    //
    // let virtual_alarm_test = static_init!(
    //     VirtualMuxAlarm<'static, arty_e21_chip::chip::ArtyExxClint>,
    //     VirtualMuxAlarm::new(mux_alarm)
    // );
    // let timertest = static_init!(
    //     timer_test::TimerTest<'static, VirtualMuxAlarm<'static, arty_e21_chip::chip::ArtyExxClint>>,
    //     timer_test::TimerTest::new(virtual_alarm_test)
    // );
    // virtual_alarm_test.set_client(timertest);

    // LEDs
    let led = components::led::LedsComponent::new().finalize(components::led_component_static!(
        hil::led::LedHigh<'static, arty_e21_chip::gpio::GpioPin>,
        hil::led::LedHigh::new(&peripherals.gpio_port[2]), // Red
        hil::led::LedHigh::new(&peripherals.gpio_port[1]), // Green
        hil::led::LedHigh::new(&peripherals.gpio_port[0]), // Blue
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        capsules_core::button::DRIVER_NUM,
        components::button_component_helper!(
            arty_e21_chip::gpio::GpioPin,
            (
                &peripherals.gpio_port[4],
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_static!(
        arty_e21_chip::gpio::GpioPin
    ));

    // set GPIO driver controlling remaining GPIO pins
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            arty_e21_chip::gpio::GpioPin,
            0 => &peripherals.gpio_port[7],
            1 => &peripherals.gpio_port[5],
            2 => &peripherals.gpio_port[6]
        ),
    )
    .finalize(components::gpio_component_static!(
        arty_e21_chip::gpio::GpioPin
    ));

    chip.enable_all_interrupts();

    let scheduler = components::sched::priority::PriorityComponent::new(board_kernel)
        .finalize(components::priority_component_static!());

    let artye21 = ArtyE21 {
        console,
        gpio,
        alarm,
        led,
        button,
        // ipc: kernel::ipc::IPC::new(board_kernel),
        scheduler,
    };

    // Create virtual device for kernel debug.
    components::debug_writer::DebugWriterComponent::new(
        uart_mux,
        create_capability!(capabilities::SetDebugWriterCapability),
    )
    .finalize(components::debug_writer_component_static!());

    // arty_e21_chip::uart::UART0.initialize_gpio_pins(&peripherals.gpio_port[17], &peripherals.gpio_port[16]);

    debug!("Initialization complete. Entering main loop.");

    // Uncomment to run tests
    //timertest.start();
    /*components::test::multi_alarm_test::MultiAlarmTestComponent::new(mux_alarm)
    .finalize(components::multi_alarm_test_component_buf!(arty_e21_chip::chip::ArtyExxClint))
    .run();*/

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
        &mut *addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    (board_kernel, artye21, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, board, chip) = start();
    board_kernel.kernel_loop(
        &board,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
