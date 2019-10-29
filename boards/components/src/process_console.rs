//! Component for ProcessConsole, the command console.
//!
//! This provides one Component, ProcessConsoleComponent, which
//! implements a command console for controlling processes over USART3
//! (the DEBUG USB connector). It also attaches kernel debug output to this
//! console (for panic!, print!, debug!, etc.).

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::process_console;
use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::static_init;

pub struct ProcessConsoleComponent {
    board_kernel: &'static kernel::Kernel,
    uart_mux: &'static MuxUart<'static>,
}

impl ProcessConsoleComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        uart_mux: &'static MuxUart,
    ) -> ProcessConsoleComponent {
        ProcessConsoleComponent {
            board_kernel: board_kernel,
            uart_mux: uart_mux,
        }
    }
}

pub struct Capability;
unsafe impl capabilities::ProcessManagementCapability for Capability {}

impl Component for ProcessConsoleComponent {
    type StaticInput = ();
    type Output = &'static process_console::ProcessConsole<'static, Capability>;

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        // Create virtual device for console.
        let console_uart = static_init!(UartDevice, UartDevice::new(self.uart_mux, true));
        console_uart.setup();

        let console = static_init!(
            process_console::ProcessConsole<'static, Capability>,
            process_console::ProcessConsole::new(
                console_uart,
                &mut process_console::WRITE_BUF,
                &mut process_console::READ_BUF,
                &mut process_console::COMMAND_BUF,
                self.board_kernel,
                Capability,
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
