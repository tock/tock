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
use hil;
use kernel;
use kernel::component::Component;
use kernel::Grant;
use sam4l;

pub struct ConsoleComponent {
    uart: &'static sam4l::usart::USART,
    baud_rate: u32,
}

impl ConsoleComponent {
    pub fn new(uart: &'static sam4l::usart::USART, rate: u32) -> ConsoleComponent {
        ConsoleComponent {
            uart: uart,
            baud_rate: rate,
        }
    }
}

impl Component for ConsoleComponent {
    type Output = &'static console::Console<'static, sam4l::usart::USART>;

    unsafe fn finalize(&mut self) -> Self::Output {
        let console = static_init!(
            console::Console<sam4l::usart::USART>,
            console::Console::new(
                self.uart,
                self.baud_rate,
                &mut console::WRITE_BUF,
                &mut console::READ_BUF,
                Grant::create()
            )
        );
        hil::uart::UART::set_client(self.uart, console);
        console.initialize();

        // Attach the kernel debug interface to this console
        let kc = static_init!(console::App, console::App::default());
        kernel::debug::assign_console_driver(Some(console), kc);

        console
    }
}
