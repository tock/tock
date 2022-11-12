//! Component for DebugWriter, the implementation for `debug!()`.
//!
//! This provides components for attaching the kernel debug output (for panic!,
//! print!, debug!, etc.) to the output. `DebugWriterComponent` uses a UART mux,
//! and `DebugWriterNoMuxComponent` just uses a UART interface directly.
//!
//! Usage
//! -----
//! ```rust
//! DebugWriterComponent::new(uart_mux).finalize(components::debug_writer_component_static!());
//!
//! components::debug_writer::DebugWriterNoMuxComponent::new(
//!     &nrf52::uart::UARTE0,
//! )
//! .finalize(());
//! ```

// Author: Brad Campbell <bradjc@virginia.edu>
// Last modified: 11/07/2019

use capsules::virtual_uart::{MuxUart, UartDevice};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::collections::ring_buffer::RingBuffer;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::uart;

// The sum of the output_buf and internal_buf is set to a multiple of 1024 bytes in order to avoid excessive
// padding between kernel memory and application memory (which often needs to be aligned to at
// least a 1 KiB boundary). This is not _semantically_ critical, but helps keep buffers on 1 KiB
// boundaries in some cases. Of course, these definitions are only advisory, and individual boards
// can choose to pass in their own buffers with different lengths.
pub const DEBUG_BUFFER_KBYTE: usize = 8;

// Bytes [0, DEBUG_BUFFER_SPLIT) are used for output_buf while bytes
// [DEBUG_BUFFER_SPLIT, DEBUG_BUFFER_KBYTE * 1024) are used for internal_buf.
const DEBUG_BUFFER_SPLIT: usize = 64;

#[macro_export]
macro_rules! debug_writer_component_static {
    () => {{
        use $crate::debug_writer::DEBUG_BUFFER_KBYTE;

        let uart = kernel::static_buf!(capsules::virtual_uart::UartDevice);
        let ring = kernel::static_buf!(kernel::collections::ring_buffer::RingBuffer<'static, u8>);
        let buffer = kernel::static_buf!([u8; 1024 * DEBUG_BUFFER_KBYTE]);
        let debug = kernel::static_buf!(kernel::debug::DebugWriter);
        let debug_wrapper = kernel::static_buf!(kernel::debug::DebugWriterWrapper);

        (uart, ring, buffer, debug, debug_wrapper)
    };};
}

#[macro_export]
macro_rules! debug_writer_no_mux_component_static {
    () => {{
        use $crate::debug_writer::DEBUG_BUFFER_KBYTE;

        let ring = kernel::static_buf!(kernel::collections::ring_buffer::RingBuffer<'static, u8>);
        let buffer = kernel::static_buf!([u8; 1024 * DEBUG_BUFFER_KBYTE]);
        let debug = kernel::static_buf!(kernel::debug::DebugWriter);
        let debug_wrapper = kernel::static_buf!(kernel::debug::DebugWriterWrapper);

        (ring, buffer, debug, debug_wrapper)
    };};
}

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
    type StaticInput = (
        &'static mut MaybeUninit<UartDevice<'static>>,
        &'static mut MaybeUninit<RingBuffer<'static, u8>>,
        &'static mut MaybeUninit<[u8; 1024 * DEBUG_BUFFER_KBYTE]>,
        &'static mut MaybeUninit<kernel::debug::DebugWriter>,
        &'static mut MaybeUninit<kernel::debug::DebugWriterWrapper>,
    );
    type Output = ();

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let buf = s.2.write([0; 1024 * DEBUG_BUFFER_KBYTE]);

        let (output_buf, internal_buf) = buf.split_at_mut(DEBUG_BUFFER_SPLIT);

        // Create virtual device for kernel debug.
        let debugger_uart = s.0.write(UartDevice::new(self.uart_mux, false));
        debugger_uart.setup();
        let ring_buffer = s.1.write(RingBuffer::new(internal_buf));
        let debugger = s.3.write(kernel::debug::DebugWriter::new(
            debugger_uart,
            output_buf,
            ring_buffer,
        ));
        hil::uart::Transmit::set_transmit_client(debugger_uart, debugger);

        let debug_wrapper = s.4.write(kernel::debug::DebugWriterWrapper::new(debugger));
        unsafe {
            kernel::debug::set_debug_writer_wrapper(debug_wrapper);
        }
    }
}

pub struct DebugWriterNoMuxComponent<U: uart::Uart<'static> + uart::Transmit<'static> + 'static> {
    uart: &'static U,
}

impl<U: uart::Uart<'static> + uart::Transmit<'static> + 'static> DebugWriterNoMuxComponent<U> {
    pub fn new(uart: &'static U) -> Self {
        Self { uart }
    }
}

impl<U: uart::Uart<'static> + uart::Transmit<'static> + 'static> Component
    for DebugWriterNoMuxComponent<U>
{
    type StaticInput = (
        &'static mut MaybeUninit<RingBuffer<'static, u8>>,
        &'static mut MaybeUninit<[u8; 1024 * DEBUG_BUFFER_KBYTE]>,
        &'static mut MaybeUninit<kernel::debug::DebugWriter>,
        &'static mut MaybeUninit<kernel::debug::DebugWriterWrapper>,
    );
    type Output = ();

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let buf = s.1.write([0; 1024 * DEBUG_BUFFER_KBYTE]);
        let (output_buf, internal_buf) = buf.split_at_mut(DEBUG_BUFFER_SPLIT);

        // Create virtual device for kernel debug.
        let ring_buffer = s.0.write(RingBuffer::new(internal_buf));
        let debugger = s.2.write(kernel::debug::DebugWriter::new(
            self.uart,
            output_buf,
            ring_buffer,
        ));
        hil::uart::Transmit::set_transmit_client(self.uart, debugger);

        let debug_wrapper = s.3.write(kernel::debug::DebugWriterWrapper::new(debugger));
        unsafe {
            kernel::debug::set_debug_writer_wrapper(debug_wrapper);
        }

        let _ = self.uart.configure(uart::Parameters {
            baud_rate: 115200,
            width: uart::Width::Eight,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
    }
}
