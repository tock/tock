//! Component for LowLevelDebug
//!
//! This provides one Component, LowLevelDebugComponent, which provides the LowLevelDebug
//! driver---a driver that can prints messages to the serial port relying on only `command`s from
//! userspace. It is particularly useful for board or runtime bringup when more complex operations
//! (allow and subscribe) may still not be working.
//!
//! Usage
//! -----
//! ```rust
//! let lldb = LowLevelDebugComponent::new(board_kernel, uart_mux).finalize(());
//! ```

// Author: Amit Levy <amit@amitlevy.com>
// Last modified: 12/04/2019

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::low_level_debug;
use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init;

pub struct LowLevelDebugComponent {
    board_kernel: &'static kernel::Kernel,
    uart_mux: &'static MuxUart<'static>,
}

impl LowLevelDebugComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        uart_mux: &'static MuxUart,
    ) -> LowLevelDebugComponent {
        LowLevelDebugComponent {
            board_kernel: board_kernel,
            uart_mux: uart_mux,
        }
    }
}

impl Component for LowLevelDebugComponent {
    type StaticInput = ();
    type Output = &'static low_level_debug::LowLevelDebug<'static, UartDevice<'static>>;

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        // Create virtual device for console.
        let lldb_uart = static_init!(UartDevice, UartDevice::new(self.uart_mux, true));
        lldb_uart.setup();

        static mut MYBUF: [u8; low_level_debug::BUF_LEN] = [0; low_level_debug::BUF_LEN];

        let lldb = static_init!(
            low_level_debug::LowLevelDebug<'static, UartDevice<'static>>,
            low_level_debug::LowLevelDebug::new(
                &mut MYBUF,
                lldb_uart,
                self.board_kernel.create_grant(&grant_cap)
            )
        );
        hil::uart::Transmit::set_transmit_client(lldb_uart, lldb);

        lldb
    }
}
