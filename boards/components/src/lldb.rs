// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for LowLevelDebug
//!
//! This provides one Component, LowLevelDebugComponent, which provides the
//! LowLevelDebug driver---a driver that can prints messages to the serial port
//! relying on only `command`s from userspace. It is particularly useful for
//! board or runtime bringup when more complex operations (allow and subscribe)
//! may still not be working.
//!
//! Usage
//! -----
//! ```rust
//! let lldb = LowLevelDebugComponent::new(board_kernel, uart_mux)
//!     .finalize(components::low_level_debug_component_static!());
//! ```

// Author: Amit Levy <amit@amitlevy.com>
// Last modified: 12/04/2019

use capsules_core::low_level_debug::LowLevelDebug;
use capsules_core::virtualizers::virtual_uart::{MuxUart, UartDevice};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;

#[macro_export]
macro_rules! low_level_debug_component_static {
    () => {{
        let uart =
            kernel::static_buf!(capsules_core::virtualizers::virtual_uart::UartDevice<'static>);
        let buffer = kernel::static_buf!([u8; capsules_core::low_level_debug::BUF_LEN]);
        let lldb = kernel::static_buf!(
            capsules_core::low_level_debug::LowLevelDebug<
                'static,
                capsules_core::virtualizers::virtual_uart::UartDevice<'static>,
            >
        );

        (uart, buffer, lldb)
    };};
}

pub struct LowLevelDebugComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    uart_mux: &'static MuxUart<'static>,
}

impl LowLevelDebugComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        uart_mux: &'static MuxUart,
    ) -> LowLevelDebugComponent {
        LowLevelDebugComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            uart_mux: uart_mux,
        }
    }
}

impl Component for LowLevelDebugComponent {
    type StaticInput = (
        &'static mut MaybeUninit<UartDevice<'static>>,
        &'static mut MaybeUninit<[u8; capsules_core::low_level_debug::BUF_LEN]>,
        &'static mut MaybeUninit<LowLevelDebug<'static, UartDevice<'static>>>,
    );
    type Output = &'static LowLevelDebug<'static, UartDevice<'static>>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let lldb_uart = s.0.write(UartDevice::new(self.uart_mux, true));
        lldb_uart.setup();

        let buffer = s.1.write([0; capsules_core::low_level_debug::BUF_LEN]);

        let lldb = s.2.write(LowLevelDebug::new(
            buffer,
            lldb_uart,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));
        hil::uart::Transmit::set_transmit_client(lldb_uart, lldb);

        lldb
    }
}
