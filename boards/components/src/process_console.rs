//! Component for ProcessConsole, the command console.
//!
//! This provides one Component, ProcessConsoleComponent, which implements a
//! command console for controlling processes over a UART bus. On imix this is
//! typically USART3 (the DEBUG USB connector).
//!
//! Usage
//! -----
//! ```rust
//! let pconsole = ProcessConsoleComponent::new(board_kernel, uart_mux).finalize(());
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

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

        console
    }
}
