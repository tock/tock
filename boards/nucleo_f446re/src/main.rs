//! Board file for Nucleo-F446RE development board
//!
//! - <https://www.st.com/en/evaluation-tools/nucleo-f446re.html>

#![no_std]
#![no_main]
#![feature(asm, core_intrinsics)]
#![deny(missing_docs)]

use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::hil;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

/// Support routines for debugging I/O.
pub mod io;

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
    console: &'static capsules::console::Console<'static, UartDevice<'static>>,
    ipc: kernel::ipc::IPC,
    led: &'static capsules::led::LED<'static, stm32f446re::gpio::Pin<'static>>,
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
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures DMA.
unsafe fn setup_dma() {
    use stm32f446re::dma1::{Dma1Peripheral, DMA1};
    use stm32f446re::usart;
    use stm32f446re::usart::USART2;

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
    use kernel::hil::gpio::Pin;
    use stm32f446re::gpio::{AlternateFunction, Mode, PinId, PortId, PORT};

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
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals() {
    // USART2 IRQn is 38
    cortexm4::nvic::Nvic::new(stm32f446re::nvic::USART2).enable();
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the STM32F446RE chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f446re::init();

    // We use the default HSI 16Mhz clock

    set_pin_primary_functions();

    setup_dma();

    setup_peripherals();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let chip = static_init!(
        stm32f446re::chip::Stm32f446re,
        stm32f446re::chip::Stm32f446re::new()
    );

    // UART

    // Create a shared UART channel for kernel debug.
    stm32f446re::usart::USART2.enable_clock();
    let mux_uart = static_init!(
        MuxUart<'static>,
        MuxUart::new(
            &stm32f446re::usart::USART2,
            &mut capsules::virtual_uart::RX_BUF,
            115200
        )
    );
    hil::uart::UART::set_client(&stm32f446re::usart::USART2, mux_uart);

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
    hil::uart::UART::set_client(debugger_uart, debugger);

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
        capsules::console::Console<UartDevice>,
        capsules::console::Console::new(
            console_uart,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    hil::uart::UART::set_client(console_uart, console);
    console.initialize();

    // `console.initialize()` configures the underlying USART, so we need to
    // tell `send_byte()` not to configure the USART again.
    io::WRITER.set_initialized();

    // LEDs

    // Clock to Port A is enabled in `set_pin_primary_functions()`
    let led_pins = static_init!(
        [(
            &'static stm32f446re::gpio::Pin,
            capsules::led::ActivationMode
        ); 1],
        [(
            &stm32f446re::gpio::PinId::PA05.get_pin().as_ref().unwrap(),
            capsules::led::ActivationMode::ActiveHigh
        )]
    );
    let led = static_init!(
        capsules::led::LED<'static, stm32f446re::gpio::Pin<'static>>,
        capsules::led::LED::new(led_pins)
    );

    let nucleo_f446re = NucleoF446RE {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        led: led,
    };

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
