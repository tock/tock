//! Component for Console on the imix board.
//!
//! This provides one Component, ConsoleComponent, which implements
//! a buffered read/write console over a serial port. This is typically
//! USART3 (the DEBUG USB connector). It attaches kernel debug output
//! to this console (for panic!, print!, debug!, etc.).
//!
//! Usage
//! -----
//! ```rust
//! let spi_syscalls = SpiSyscallComponent::new(mux_spi).finalize();
//! let rf233_spi = SpiComponent::new(mux_spi).finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::console;
use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init;

pub struct ConsoleComponent {
    board_kernel: &'static kernel::Kernel,
    uart_mux: &'static MuxUart<'static>,
}

impl ConsoleComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        uart_mux: &'static MuxUart,
    ) -> ConsoleComponent {
        ConsoleComponent {
            board_kernel: board_kernel,
            uart_mux: uart_mux,
        }
    }
}

impl Component for ConsoleComponent {
    type Output = &'static console::Console<'static>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        // Create virtual device for console.
        let console_uart = static_init!(UartDevice, UartDevice::new(self.uart_mux, true));
        console_uart.setup();

        let console = static_init!(
            console::Console<'static>,
            console::Console::new(
                console_uart,
                &mut console::WRITE_BUF,
                &mut console::READ_BUF,
                self.board_kernel.create_grant(&grant_cap)
            )
        );
        hil::uart::Transmit::set_transmit_client(console_uart, console);
        hil::uart::Receive::set_receive_client(console_uart, console);

        // Create virtual device for kernel debug.
        let debugger_uart = static_init!(UartDevice, UartDevice::new(self.uart_mux, false));
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

        console
    }
}
