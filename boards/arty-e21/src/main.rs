//! Board file for the SiFive E21 Bitstream running on the Arty FPGA

#![no_std]
#![no_main]
#![feature(const_fn, in_band_lifetimes)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
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
static mut APP_MEMORY: [u8; 8192] = [0; 8192];

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct ArtyE21 {
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static, sifive::gpio::GpioPin>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
    >,
    led: &'static capsules::led::LED<'static, sifive::gpio::GpioPin>,
    button: &'static capsules::button::Button<'static, sifive::gpio::GpioPin>,
    // ipc: kernel::ipc::IPC,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for ArtyE21 {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
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

    let chip = static_init!(arty_e21::chip::ArtyExx, arty_e21::chip::ArtyExx::new());
    chip.initialize();

    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&arty_e21::gpio::PORT[0]), // Red
        Some(&arty_e21::gpio::PORT[1]),
        Some(&arty_e21::gpio::PORT[8]),
    );

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        MuxUart<'static>,
        MuxUart::new(
            &arty_e21::uart::UART0,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    uart_mux.initialize();

    hil::uart::Transmit::set_transmit_client(&arty_e21::uart::UART0, uart_mux);
    hil::uart::Receive::set_receive_client(&arty_e21::uart::UART0, uart_mux);

    // Create a UartDevice for the console.
    let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    console_uart.setup();
    let console = static_init!(
        capsules::console::Console<'static>,
        capsules::console::Console::new(
            console_uart,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant(&memory_allocation_cap)
        )
    );
    hil::uart::Transmit::set_transmit_client(console_uart, console);
    hil::uart::Receive::set_receive_client(console_uart, console);

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, rv32i::machine_timer::MachineTimer>,
        MuxAlarm::new(&rv32i::machine_timer::MACHINETIMER)
    );
    rv32i::machine_timer::MACHINETIMER.set_client(mux_alarm);

    // Alarm
    let virtual_alarm_user = static_init!(
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
    virtual_alarm_user.set_client(alarm);

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
    let led_pins = static_init!(
        [(
            &'static sifive::gpio::GpioPin,
            capsules::led::ActivationMode
        ); 3],
        [
            (
                // Red
                &arty_e21::gpio::PORT[0],
                capsules::led::ActivationMode::ActiveHigh
            ),
            (
                // Green
                &arty_e21::gpio::PORT[1],
                capsules::led::ActivationMode::ActiveHigh
            ),
            (
                // Blue
                &arty_e21::gpio::PORT[2],
                capsules::led::ActivationMode::ActiveHigh
            ),
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, sifive::gpio::GpioPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONs
    let button_pins = static_init!(
        [(&'static sifive::gpio::GpioPin, capsules::button::GpioMode); 1],
        [(
            &arty_e21::gpio::PORT[4],
            capsules::button::GpioMode::HighWhenPressed
        )]
    );
    let button = static_init!(
        capsules::button::Button<'static, sifive::gpio::GpioPin>,
        capsules::button::Button::new(
            button_pins,
            board_kernel.create_grant(&memory_allocation_cap)
        )
    );
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    // set GPIO driver controlling remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static sifive::gpio::GpioPin; 3],
        [
            &arty_e21::gpio::PORT[7],
            &arty_e21::gpio::PORT[5],
            &arty_e21::gpio::PORT[6],
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, sifive::gpio::GpioPin>,
        capsules::gpio::GPIO::new(gpio_pins, board_kernel.create_grant(&memory_allocation_cap))
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

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
    let debugger_uart = static_init!(UartDevice, UartDevice::new(uart_mux, false));
    debugger_uart.setup();
    let debugger = static_init!(
        kernel::debug::DebugWriter,
        kernel::debug::DebugWriter::new(
            debugger_uart,
            &mut kernel::debug::OUTPUT_BUF,
            &mut kernel::debug::INTERNAL_BUF,
        )
    );
    hil::uart::Transmit::set_transmit_client(debugger_uart, debugger);

    let debug_wrapper = static_init!(
        kernel::debug::DebugWriterWrapper,
        kernel::debug::DebugWriterWrapper::new(debugger)
    );
    kernel::debug::set_debug_writer_wrapper(debug_wrapper);

    // arty_e21::uart::UART0.initialize_gpio_pins(&arty_e21::gpio::PORT[17], &arty_e21::gpio::PORT[16]);

    debug!("Initialization complete. Entering main loop.");

    // timertest.start();

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }

    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_mgmt_cap,
    );

    board_kernel.kernel_loop(&artye21, chip, None, &main_loop_cap);
}
