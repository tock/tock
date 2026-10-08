// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for `UartRxBuffer`, an RX buffering layer for a chip UART.
//!
//! This sets up a [`UartRxBuffer`](capsules_extra::uart_rx_buffer::UartRxBuffer)
//! on top of the given chip UART, and wraps it in a
//! [`UartRxBufferFull`](capsules_extra::uart_rx_buffer::UartRxBufferFull) so
//! the result is a drop-in `hil::uart::Uart`, with `Transmit`/`Configure`
//! passed straight through to the chip UART and `Receive` buffered.
//!
//! Usage
//! -----
//! ```rust,ignore
//! let uart_rx_buffer = components::uart_rx_buffer::UartRxBufferComponent::new(&chip.uart0)
//!     .finalize(components::uart_rx_buffer_component_static!(nrf52::uart::Uarte, 1024));
//! ```

use capsules_extra::uart_rx_buffer::{UartRxBuffer, UartRxBufferFull};
use core::mem::MaybeUninit;
use kernel::collections::ring_buffer::RingBuffer;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil::uart;

/// The optional second argument to this macro allows boards to specify the
/// size, in bytes, of the ring buffer used to hold received-but-undelivered
/// data.
///
/// It defaults to
/// [`DEFAULT_RING_BUFFER_SIZE_BYTE`](capsules_extra::uart_rx_buffer::DEFAULT_RING_BUFFER_SIZE_BYTE).
#[macro_export]
macro_rules! uart_rx_buffer_component_static {
    ($U:ty, $RING_BUF_SIZE_BYTES:expr $(,)?) => {{
        let rx_byte = kernel::static_buf!([u8; 1]);
        let ring_buf = kernel::static_buf!([u8; $RING_BUF_SIZE_BYTES]);
        let ring_buffer =
            kernel::static_buf!(kernel::collections::ring_buffer::RingBuffer<'static, u8>);
        let uart_rx_buffer =
            kernel::static_buf!(capsules_extra::uart_rx_buffer::UartRxBuffer<'static, $U>);
        let uart_rx_buffer_full =
            kernel::static_buf!(capsules_extra::uart_rx_buffer::UartRxBufferFull<'static, $U, $U>);

        (
            rx_byte,
            ring_buf,
            ring_buffer,
            uart_rx_buffer,
            uart_rx_buffer_full,
        )
    }};
    ($U:ty $(,)?) => {{
        $crate::uart_rx_buffer_component_static!(
            $U,
            capsules_extra::uart_rx_buffer::DEFAULT_RING_BUFFER_SIZE_BYTE
        )
    }};
}

pub type UartRxBufferComponentType<U> = UartRxBufferFull<'static, U, U>;

pub struct UartRxBufferComponent<U: uart::Uart<'static> + 'static, const RING_BUF_SIZE_BYTES: usize>
{
    uart: &'static U,
}

impl<U: uart::Uart<'static> + 'static, const RING_BUF_SIZE_BYTES: usize>
    UartRxBufferComponent<U, RING_BUF_SIZE_BYTES>
{
    pub fn new(uart: &'static U) -> Self {
        Self { uart }
    }
}

impl<U: uart::Uart<'static> + 'static, const RING_BUF_SIZE_BYTES: usize> Component
    for UartRxBufferComponent<U, RING_BUF_SIZE_BYTES>
{
    type StaticInput = (
        &'static mut MaybeUninit<[u8; 1]>,
        &'static mut MaybeUninit<[u8; RING_BUF_SIZE_BYTES]>,
        &'static mut MaybeUninit<RingBuffer<'static, u8>>,
        &'static mut MaybeUninit<UartRxBuffer<'static, U>>,
        &'static mut MaybeUninit<UartRxBufferFull<'static, U, U>>,
    );
    type Output = &'static UartRxBufferFull<'static, U, U>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let rx_byte = s.0.write([0; 1]);
        let ring_buf = s.1.write([0; RING_BUF_SIZE_BYTES]);
        let ring_buf = kernel::utilities::slice_uninit::mut_slice_as_maybeuninit(ring_buf);
        let ring_buffer = s.2.write(RingBuffer::new(ring_buf));

        let uart_rx_buffer =
            s.3.write(UartRxBuffer::new(self.uart, rx_byte, ring_buffer));
        uart_rx_buffer.register();
        uart::Receive::set_receive_client(self.uart, uart_rx_buffer);
        uart_rx_buffer.start_receiving();

        s.4.write(UartRxBufferFull::new(self.uart, uart_rx_buffer))
    }
}
