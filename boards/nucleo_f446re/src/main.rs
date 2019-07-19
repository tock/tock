//! Board file for Nucleo-F446RE development board
//!
//! - <https://www.st.com/en/evaluation-tools/nucleo-f446re.html>

#![no_std]
#![no_main]
#![feature(asm, core_intrinsics)]
#![deny(missing_docs)]

use capsules::button;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::hil;
use kernel::hil::gpio;
use kernel::hil::gpio::{Configure, InterruptPin, InterruptWithValue};
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

/// Support routines for debugging I/O.
pub mod io;

// Unit Tests for drivers.
#[allow(dead_code)]
mod virtual_uart_rx_test;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 65536] = [0; 65536];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct NucleoF446RE {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
    led: &'static capsules::led::LED<'static>,
    button: &'static capsules::button::Button<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f4xx::tim2::Tim2<'static>>,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for NucleoF446RE {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures DMA.
unsafe fn setup_dma() {
    use stm32f4xx::dma1::{Dma1Peripheral, DMA1};
    use stm32f4xx::usart;
    use stm32f4xx::usart::USART2;

    DMA1.enable_clock();

    let usart2_tx_stream = Dma1Peripheral::USART2_TX.get_stream();
    let usart2_rx_stream = Dma1Peripheral::USART2_RX.get_stream();

    USART2.set_dma(
        usart::TxDMA(usart2_tx_stream),
        usart::RxDMA(usart2_rx_stream),
    );

    usart2_tx_stream.set_client(&USART2);
    usart2_rx_stream.set_client(&USART2);

    usart2_tx_stream.setup(Dma1Peripheral::USART2_TX);
    usart2_rx_stream.setup(Dma1Peripheral::USART2_RX);

    cortexm4::nvic::Nvic::new(Dma1Peripheral::USART2_TX.get_stream_irqn()).enable();
    cortexm4::nvic::Nvic::new(Dma1Peripheral::USART2_RX.get_stream_irqn()).enable();
}

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    use stm32f4xx::exti::{LineId, EXTI};
    use stm32f4xx::gpio::{AlternateFunction, Mode, PinId, PortId, PORT};
    use stm32f4xx::syscfg::SYSCFG;

    SYSCFG.enable_clock();

    PORT[PortId::A as usize].enable_clock();

    // User LD2 is connected to PA05. Configure PA05 as `debug_gpio!(0, ...)`
    PinId::PA05.get_pin().as_ref().map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    // pa2 and pa3 (USART2) is connected to ST-LINK virtual COM port
    PinId::PA02.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART2_TX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
    PinId::PA03.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART2_RX
        pin.set_alternate_function(AlternateFunction::AF7);
    });

    PORT[PortId::C as usize].enable_clock();

    // button is connected on pc13
    PinId::PC13.get_pin().as_ref().map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        EXTI.associate_line_gpiopin(LineId::Exti13, pin);
    });
    // EXTI13 interrupts is delivered at IRQn 40 (EXTI15_10)
    cortexm4::nvic::Nvic::new(stm32f4xx::nvic::EXTI15_10).enable();
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals() {
    use stm32f4xx::tim2::TIM2;

    // USART2 IRQn is 38
    cortexm4::nvic::Nvic::new(stm32f4xx::nvic::USART2).enable();

    // TIM2 IRQn is 28
    TIM2.enable_clock();
    TIM2.start();
    cortexm4::nvic::Nvic::new(stm32f4xx::nvic::TIM2).enable();
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the STM32F446RE chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f4xx::init();

    // We use the default HSI 16Mhz clock

    set_pin_primary_functions();

    setup_dma();

    setup_peripherals();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let chip = static_init!(
        stm32f4xx::chip::Stm32f4xx,
        stm32f4xx::chip::Stm32f4xx::new()
    );

    // UART

    // Create a shared UART channel for kernel debug.
    stm32f4xx::usart::USART2.enable_clock();
    let mux_uart = static_init!(
        MuxUart<'static>,
        MuxUart::new(
            &stm32f4xx::usart::USART2,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    mux_uart.initialize();
    // `mux_uart.initialize()` configures the underlying USART, so we need to
    // tell `send_byte()` not to configure the USART again.
    io::WRITER.set_initialized();

    hil::uart::Transmit::set_transmit_client(&stm32f4xx::usart::USART2, mux_uart);
    hil::uart::Receive::set_receive_client(&stm32f4xx::usart::USART2, mux_uart);

    // Create a virtual device for kernel debug.
    let debugger_uart = static_init!(UartDevice, UartDevice::new(mux_uart, false));
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

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    // Create a UartDevice for console
    let console_uart = static_init!(UartDevice, UartDevice::new(mux_uart, true));
    console_uart.setup();
    let console = static_init!(
        capsules::console::Console,
        capsules::console::Console::new(
            console_uart,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    hil::uart::Transmit::set_transmit_client(console_uart, console);
    hil::uart::Receive::set_receive_client(console_uart, console);

    // // Setup the process inspection console
    // let process_console_uart = static_init!(UartDevice, UartDevice::new(mux_uart, true));
    // process_console_uart.setup();
    // pub struct ProcessConsoleCapability;
    // unsafe impl capabilities::ProcessManagementCapability for ProcessConsoleCapability {}
    // let process_console = static_init!(
    //     capsules::process_console::ProcessConsole<'static, ProcessConsoleCapability>,
    //     capsules::process_console::ProcessConsole::new(
    //         process_console_uart,
    //         &mut capsules::process_console::WRITE_BUF,
    //         &mut capsules::process_console::READ_BUF,
    //         &mut capsules::process_console::COMMAND_BUF,
    //         board_kernel,
    //         ProcessConsoleCapability,
    //     )
    // );
    // hil::uart::Transmit::set_transmit_client(process_console_uart, process_console);
    // hil::uart::Receive::set_receive_client(process_console_uart, process_console);
    // process_console.start();

    // LEDs

    // Clock to Port A is enabled in `set_pin_primary_functions()`
    let led_pins = static_init!(
        [(&'static kernel::hil::gpio::Pin, capsules::led::ActivationMode); 1],
        [(
            stm32f4xx::gpio::PinId::PA05.get_pin().as_ref().unwrap(),
            capsules::led::ActivationMode::ActiveHigh
        )]
    );
    let led = static_init!(
        capsules::led::LED<'static>,
        capsules::led::LED::new(&led_pins[..])
    );

    // BUTTONs
    let button_pins = static_init!(
        [&'static InterruptPin; 1],
        [stm32f4xx::gpio::PinId::PC13.get_pin().as_ref().unwrap()]
    );
    let values = static_init!(
        [kernel::hil::gpio::InterruptValueWrapper; 1],
        [gpio::InterruptValueWrapper::new()]
    );

    // Button expects a configured InterruptValuePin so configure it here.
    for i in 0..1 {
        let pin = button_pins[i];
        let value = &values[i];
        pin.set_client(value);
        value.set_source(pin);
    }
    let config_values = static_init!(
        [(
           &'static kernel::hil::gpio::InterruptValuePin,
           button::GpioMode
        ); 1],
       [(&values[0], button::GpioMode::LowWhenPressed)]
    );
    let button = static_init!(
        capsules::button::Button<'static>,
        capsules::button::Button::new(
            &config_values[..],
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    for i in 0..1 {
        values[i].set_client(button);
    }

    // ALARM
    let mux_alarm = static_init!(
        MuxAlarm<'static, stm32f4xx::tim2::Tim2>,
        MuxAlarm::new(&stm32f4xx::tim2::TIM2)
    );
    stm32f4xx::tim2::TIM2.set_client(mux_alarm);

    let virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, stm32f4xx::tim2::Tim2>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, stm32f4xx::tim2::Tim2>>,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    virtual_alarm.set_client(alarm);

    let nucleo_f446re = NucleoF446RE {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        led: led,
        button: button,
        alarm: alarm,
    };

    // // Optional kernel tests
    // //
    // // See comment in `boards/imix/src/main.rs`
    // virtual_uart_rx_test::run_virtual_uart_receive(mux_uart);

    debug!("Initialization complete. Entering main loop");

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
        &process_management_capability,
    );

    board_kernel.kernel_loop(
        &nucleo_f446re,
        chip,
        Some(&nucleo_f446re.ipc),
        &main_loop_capability,
    );
}
