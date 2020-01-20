//! Component for DebugWriter, the implementation for `debug!()`.
//!
//! This provides one `Component`, `DebugWriter`, which attaches kernel debug
//! output (for panic!, print!, debug!, etc.) to the provided UART mux.
//!
//! Usage
//! -----
//! ```rust
//! DebugWriterComponent::new(uart_mux).finalize(());
//! ```

// Author: Brad Campbell <bradjc@virginia.edu>
// Last modified: 11/07/2019

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::common::ring_buffer::RingBuffer;
use kernel::component::Component;
use kernel::hil;
use kernel::static_init;

pub struct DebugWriterComponent {
    uart_mux: &'static MuxUart<'static>,
}

impl DebugWriterComponent {
    pub fn new(uart_mux: &'static MuxUart) -> DebugWriterComponent {
        DebugWriterComponent { uart_mux: uart_mux }
    }
}

pub struct Capability;
unsafe impl capabilities::ProcessManagementCapability for Capability {}

impl Component for DebugWriterComponent {
    type StaticInput = ();
    type Output = ();

    unsafe fn finalize(&mut self, _s: Self::StaticInput) -> Self::Output {
        // The sum of the output_buf and internal_buf is set to 1024 bytes in order to avoid excessive
        // padding between kernel memory and application memory (which often needs to be aligned to at
        // least a 1kB boundary). This is not _semantically_ critical, but helps keep buffers on 1kB
        // boundaries in some cases. Of course, these definitions are only advisory, and individual boards
        // can choose to pass in their own buffers with different lengths.
        let buf = static_init!([u8; 1024], [0; 1024]);
        let (output_buf, internal_buf) = buf.split_at_mut(64);

        // Create virtual device for kernel debug.
        let debugger_uart = static_init!(UartDevice, UartDevice::new(self.uart_mux, false));
        debugger_uart.setup();
        let ring_buffer = static_init!(RingBuffer<'static, u8>, RingBuffer::new(internal_buf));
        let debugger = static_init!(
            kernel::debug::DebugWriter,
            kernel::debug::DebugWriter::new(debugger_uart, output_buf, ring_buffer)
        );
        hil::uart::Transmit::set_transmit_client(debugger_uart, debugger);

        let debug_wrapper = static_init!(
            kernel::debug::DebugWriterWrapper,
            kernel::debug::DebugWriterWrapper::new(debugger)
        );
        kernel::debug::set_debug_writer_wrapper(debug_wrapper);
    }
}
