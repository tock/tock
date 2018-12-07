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
use kernel::{create_capability, debug, debug_verbose, static_init};

/// Support routines for debugging I/O.
pub mod io;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 0;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; NUM_PROCS] = [];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct NucleoF446RE {
    ipc: kernel::ipc::IPC,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for NucleoF446RE {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Helper function called during bring-up that configures DMA.
unsafe fn setup_dma() {
    use stm32f446re::dma1::DMA1;

    DMA1.enable_clock();
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
unsafe fn setup_peripherals() {}

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
    // required by `debug::panic`. If panic occurs before this point, you only
    // have `gdb` and `debug_gpio!`
    kernel::debug::set_debug_writer_wrapper(debug_wrapper);

    // Normally `console.initialize()` will call `USART2.configure()`. We do not
    // have console capsule as yet. So, we call `mux_uart.initialize()`, which
    // does the same thing.
    mux_uart.initialize();

    // Since `mux_uart.initialize()` configures the underlying USART, we need to
    // tell `send_byte()` not to configure the USART again.
    io::WRITER.set_initialized();

    // // Uncomment to test `debug!`, `debug_verbose!`, `panic!` and hardfault

    // debug!("Hello debug! macro");

    // debug_verbose!("Hello debug_verbose! macro");

    // panic!("Hello panic! macro");

    // // generate hardfault
    // asm!(
    //     "
    //     movw r0, #0xFFFF
    //     movt r0, #0xFFFF
    //     ldr r1, [r0, #0]"
    //     :::: "volatile");

    asm!("bkpt" :::: "volatile");

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let nucleo_f446re = NucleoF446RE {
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    board_kernel.kernel_loop(
        &nucleo_f446re,
        chip,
        Some(&nucleo_f446re.ipc),
        &main_loop_capability,
    );
}
