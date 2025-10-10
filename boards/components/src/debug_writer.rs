// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component for DebugWriter, the implementation for `debug!()`.
//!
//! This provides components for attaching the kernel debug output (for panic!,
//! print!, debug!, etc.) to the output. `DebugWriterComponent` uses a UART mux,
//! and `DebugWriterNoMuxComponent` just uses a UART interface directly.
//!
//! Usage
//! -----
//! ```rust
//! let debug_wrapper = components::debug_writer::DebugWriterComponent::new(
//!     uart_mux,
//!     create_capability!(kernel::capabilities::SetDebugWriterCapability),
//! )
//! .finalize(components::debug_writer_component_static!());
//!
//! let debug_wrapper = components::debug_writer::DebugWriterNoMuxComponent::new(
//!     &nrf52::uart::UARTE0,
//!     create_capability!(kernel::capabilities::SetDebugWriterCapability),
//! )
//! .finalize(components::debug_writer_no_mux_component_static!());
//! ```

// Author: Brad Campbell <bradjc@virginia.edu>
// Last modified: 11/07/2019

use capsules_core::virtualizers::virtual_uart::{MuxUart, UartDevice};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::capabilities::SetDebugWriterCapability;
use kernel::collections::ring_buffer::RingBuffer;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::uart;

// The sum of the output_buf and internal_buf is set to a multiple of 1024 bytes in order to avoid excessive
// padding between kernel memory and application memory (which often needs to be aligned to at
// least a 1 KiB boundary). This is not _semantically_ critical, but helps keep buffers on 1 KiB
// boundaries in some cases. Of course, these definitions are only advisory, and individual boards
// can choose to pass in their own buffers with different lengths.
pub const DEFAULT_DEBUG_BUFFER_KBYTE: usize = 2;

// Bytes [0, DEBUG_BUFFER_SPLIT) are used for output_buf while bytes
// [DEBUG_BUFFER_SPLIT, DEFAULT_DEBUG_BUFFER_KBYTE * 1024) are used for internal_buf.
const DEBUG_BUFFER_SPLIT: usize = 64;

/// The optional argument to this macro allows boards to specify the size of the in-RAM
/// buffer used for storing debug messages.
///
/// Increase this value to be able to send more debug messages in
/// quick succession.
#[macro_export]
macro_rules! debug_writer_component_static {
    ($BUF_SIZE_KB:expr) => {{
        let uart = kernel::static_buf!(capsules_core::virtualizers::virtual_uart::UartDevice);
        let ring = kernel::static_buf!(kernel::collections::ring_buffer::RingBuffer<'static, u8>);
        let buffer = kernel::static_buf!([u8; 1024 * $BUF_SIZE_KB]);
        let debug = kernel::static_buf!(kernel::debug::DebugWriter);

        (uart, ring, buffer, debug)
    };};
    () => {{
        $crate::debug_writer_component_static!($crate::debug_writer::DEFAULT_DEBUG_BUFFER_KBYTE)
    };};
}

/// The optional argument to this macro allows boards to specify the size of the in-RAM
/// buffer used for storing debug messages.
///
/// Increase this value to be able to send more debug messages in
/// quick succession.
#[macro_export]
macro_rules! debug_writer_no_mux_component_static {
    ($BUF_SIZE_KB:expr) => {{
        let ring = kernel::static_buf!(kernel::collections::ring_buffer::RingBuffer<'static, u8>);
        let buffer = kernel::static_buf!([u8; 1024 * $BUF_SIZE_KB]);
        let debug = kernel::static_buf!(kernel::debug::DebugWriter);

        (ring, buffer, debug)
    };};
    () => {{
        use $crate::debug_writer::DEFAULT_DEBUG_BUFFER_KBYTE;
        $crate::debug_writer_no_mux_component_static!(DEFAULT_DEBUG_BUFFER_KBYTE)
    };};
}

// Allow dead code because we need the `Chip` type but don't use `chip`.
#[allow(dead_code)]
pub struct DebugWriterComponent<const BUF_SIZE_BYTES: usize, C: SetDebugWriterCapability> {
    uart_mux: &'static MuxUart<'static>,
    marker: core::marker::PhantomData<[u8; BUF_SIZE_BYTES]>,
    capability: C,
}

impl<const BUF_SIZE_BYTES: usize, C: SetDebugWriterCapability>
    DebugWriterComponent<BUF_SIZE_BYTES, C>
{
    /// Create a debug writer component while binding the global variable used
    /// by debug.rs to the main thread.
    #[cfg(target_has_atomic = "ptr")]
    pub fn new<P: kernel::platform::chip::ThreadIdProvider>(
        uart_mux: &'static MuxUart,
        capability: C,
    ) -> Self {
        kernel::debug::initialize_debug_writer_wrapper::<P>();

        Self {
            uart_mux,
            marker: core::marker::PhantomData,
            capability,
        }
    }

    /// Create a debug writer component and bind the global variable used by
    /// debug.rs to the main thread, but require that the caller(i.e., main.rs)
    /// provides the actual init call.
    ///
    /// This allows moving the unsafe call to main.rs instead of being
    /// encapsulated in a component.
    ///
    /// The resulting use of this component for platforms without atomics
    /// support looks like this:
    ///
    /// ```
    ///  components::debug_writer::DebugWriterComponent::new(
    ///      ...,
    ///      || unsafe {
    ///         kernel::debug::initialize_debug_writer_wrapper_unsafe::<
    ///             <Chip as kernel::platform::chip::Chip>::ThreadIdProvider,
    ///         >();
    ///      })
    /// .finalize(components::debug_writer_component_static!());
    /// ```
    pub fn new_unsafe<F>(uart_mux: &'static MuxUart, capability: C, bind_debug_global: F) -> Self
    where
        F: FnOnce(),
    {
        bind_debug_global();

        Self {
            uart_mux,
            marker: core::marker::PhantomData,
            capability,
        }
    }
}

pub struct Capability;
unsafe impl capabilities::ProcessManagementCapability for Capability {}

impl<const BUF_SIZE_BYTES: usize, C: SetDebugWriterCapability> Component
    for DebugWriterComponent<BUF_SIZE_BYTES, C>
{
    type StaticInput = (
        &'static mut MaybeUninit<UartDevice<'static>>,
        &'static mut MaybeUninit<RingBuffer<'static, u8>>,
        &'static mut MaybeUninit<[u8; BUF_SIZE_BYTES]>,
        &'static mut MaybeUninit<kernel::debug::DebugWriter>,
    );
    type Output = ();

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let buf = s.2.write([0; BUF_SIZE_BYTES]);

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

        kernel::debug::set_debug_writer_wrapper(debugger, self.capability);
    }
}

// Allow dead code because we need the `Chip` type but don't use `chip`.
#[allow(dead_code)]
pub struct DebugWriterNoMuxComponent<
    U: uart::Uart<'static> + uart::Transmit<'static> + 'static,
    const BUF_SIZE_BYTES: usize,
    C: SetDebugWriterCapability,
> {
    uart: &'static U,
    marker: core::marker::PhantomData<[u8; BUF_SIZE_BYTES]>,
    capability: C,
}

impl<
        U: uart::Uart<'static> + uart::Transmit<'static> + 'static,
        const BUF_SIZE_BYTES: usize,
        C: SetDebugWriterCapability,
    > DebugWriterNoMuxComponent<U, BUF_SIZE_BYTES, C>
{
    pub fn new(uart: &'static U, capability: C) -> Self {
        Self {
            uart,
            marker: core::marker::PhantomData,
            capability,
        }
    }
}

impl<
        U: uart::Uart<'static> + uart::Transmit<'static> + 'static,
        const BUF_SIZE_BYTES: usize,
        C: SetDebugWriterCapability,
    > Component for DebugWriterNoMuxComponent<U, BUF_SIZE_BYTES, C>
{
    type StaticInput = (
        &'static mut MaybeUninit<RingBuffer<'static, u8>>,
        &'static mut MaybeUninit<[u8; BUF_SIZE_BYTES]>,
        &'static mut MaybeUninit<kernel::debug::DebugWriter>,
    );
    type Output = ();

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let buf = s.1.write([0; BUF_SIZE_BYTES]);
        let (output_buf, internal_buf) = buf.split_at_mut(DEBUG_BUFFER_SPLIT);

        // Create virtual device for kernel debug.
        let ring_buffer = s.0.write(RingBuffer::new(internal_buf));
        let debugger = s.2.write(kernel::debug::DebugWriter::new(
            self.uart,
            output_buf,
            ring_buffer,
        ));
        hil::uart::Transmit::set_transmit_client(self.uart, debugger);

        kernel::debug::set_debug_writer_wrapper(debugger, self.capability);

        let _ = self.uart.configure(uart::Parameters {
            baud_rate: 115200,
            width: uart::Width::Eight,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
    }
}
