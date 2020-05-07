//! Board file for the SiFive E21 Bitstream running on the Arty FPGA

#![no_std]
#![no_main]
#![feature(const_fn, in_band_lifetimes)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

mod timer_test;

pub mod io;

// State for loading and holding applications.

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 49152] = [0; 49152];

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

// Reference to the chip for panic dumps.
static mut CHIP: Option<&'static arty_e21_chip::chip::ArtyExx> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct ArtyE21 {
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, arty_e21_chip::gpio::GpioPin>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer<'static>>,
    >,
    led: &'static capsules::led::LED<'static, arty_e21_chip::gpio::GpioPin>,
    button: &'static capsules::button::Button<'static, arty_e21_chip::gpio::GpioPin>,
    // ipc: kernel::ipc::IPC,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for ArtyE21 {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),

            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),

            // kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
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

    let chip = static_init!(
        arty_e21_chip::chip::ArtyExx,
        arty_e21_chip::chip::ArtyExx::new()
    );
    CHIP = Some(chip);
    chip.initialize();

    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
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
        Some(&arty_e21_chip::gpio::PORT[0]), // Blue
        Some(&arty_e21_chip::gpio::PORT[1]), // Green
        Some(&arty_e21_chip::gpio::PORT[8]),
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = components::console::UartMuxComponent::new(
        &arty_e21_chip::uart::UART0,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
        MuxAlarm::new(&arty_e21_chip::timer::MACHINETIMER)
    );
    hil::time::Alarm::set_client(&arty_e21_chip::timer::MACHINETIMER, mux_alarm);

    // Alarm
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm).finalize(
        components::alarm_component_helper!(rv32i::machine_timer::MachineTimer),
    );

    // TEST for timer
    //
    // let virtual_alarm_test = static_init!(
    //     VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
    //     VirtualMuxAlarm::new(mux_alarm)
    // );
    // let timertest = static_init!(
    //     timer_test::TimerTest<'static, VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>>,
    //     timer_test::TimerTest::new(virtual_alarm_test)
    // );
    // virtual_alarm_test.set_client(timertest);

    // LEDs
    let led = components::led::LedsComponent::new(components::led_component_helper!(
        arty_e21_chip::gpio::GpioPin,
        (
            // Red
            &arty_e21_chip::gpio::PORT[2],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            // Green
            &arty_e21_chip::gpio::PORT[1],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            // Blue
            &arty_e21_chip::gpio::PORT[0],
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ))
    .finalize(components::led_component_buf!(arty_e21_chip::gpio::GpioPin));

    // BUTTONs
    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            arty_e21_chip::gpio::GpioPin,
            (
                &arty_e21_chip::gpio::PORT[4],
                kernel::hil::gpio::ActivationMode::ActiveHigh,
                kernel::hil::gpio::FloatingState::PullNone
            )
        ),
    )
    .finalize(components::button_component_buf!(
        arty_e21_chip::gpio::GpioPin
    ));

    // set GPIO driver controlling remaining GPIO pins
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            arty_e21_chip::gpio::GpioPin,
            0 => &arty_e21_chip::gpio::PORT[7],
            1 => &arty_e21_chip::gpio::PORT[5],
            2 => &arty_e21_chip::gpio::PORT[6]
        ),
    )
    .finalize(components::gpio_component_buf!(
        arty_e21_chip::gpio::GpioPin
    ));

    chip.enable_all_interrupts();

    let artye21 = ArtyE21 {
        console: console,
        gpio: gpio,
        alarm: alarm,
        led: led,
        button: button,
        // ipc: kernel::ipc::IPC::new(board_kernel),
    };

    // Create virtual device for kernel debug.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    // arty_e21_chip::uart::UART0.initialize_gpio_pins(&arty_e21_chip::gpio::PORT[17], &arty_e21_chip::gpio::PORT[16]);

    debug!("Initialization complete. Entering main loop.");

    // timertest.start();

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;

        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _eapps: u8;
    }

    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&artye21, chip, None, &main_loop_cap);
}
