// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Virtualize a UART bus.
//!
//! This allows multiple Tock capsules to use the same UART bus. This is likely
//! most useful for `printf()` like applications where multiple things want to
//! write to the same UART channel.
//!
//! Clients can choose if they want to receive. Incoming messages will be sent
//! to all clients that have enabled receiving.
//!
//! `MuxUart` provides shared access to a single UART bus for multiple users.
//! `UartDevice` provides access for a single client.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::{hil, static_init};
//! # use capsules_core::virtual_uart::{MuxUart, UartDevice};
//!
//! // Create a shared UART channel for the console and for kernel debug.
//! let uart_mux = static_init!(
//!     MuxUart<'static>,
//!     MuxUart::new(&sam4l::usart::USART0, &mut capsules_core::virtual_uart::RX_BUF)
//! );
//! hil::uart::UART::set_receive_client(&sam4l::usart::USART0, uart_mux);
//! hil::uart::UART::set_transmit_client(&sam4l::usart::USART0, uart_mux);
//!
//! // Create a UartDevice for the console.
//! let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
//! console_uart.setup(); // This is important!
//! let console = static_init!(
//!     capsules_core::console::Console<'static>,
//!     capsules_core::console::Console::new(
//!         console_uart,
//!         &mut capsules_core::console::WRITE_BUF,
//!         &mut capsules_core::console::READ_BUF,
//!         board_kernel.create_grant(&grant_cap)
//!     )
//! );
//! hil::uart::UART::set_transmit_client(console_uart, console);
//! hil::uart::UART::set_receive_client(console_uart, console);
//! ```

use core::cell::Cell;
use core::cmp;

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::uart;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

pub const RX_BUF_LEN: usize = 64;

pub struct MuxUart<'a> {
    uart: &'a dyn uart::Uart<'a>,
    speed: u32,
    devices: List<'a, UartDevice<'a>>,
    inflight: OptionalCell<&'a UartDevice<'a>>,
    buffer: TakeCell<'static, [u8]>,
    completing_read: Cell<bool>,
    deferred_call: DeferredCall,
}

impl uart::TransmitClient for MuxUart<'_> {
    fn transmitted_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        rcode: Result<(), ErrorCode>,
    ) {
        self.inflight.map(move |device| {
            self.inflight.clear();
            device.transmitted_buffer(tx_buffer, tx_len, rcode);
        });
        self.do_next_op();
    }
}

impl uart::ReceiveClient for MuxUart<'_> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        rcode: Result<(), ErrorCode>,
        error: uart::Error,
    ) {
        // Likely we will issue another receive in response to the previous one
        // finishing. `next_read_len` keeps track of the shortest outstanding
        // receive requested by any client. We start with the longest it can be,
        // i.e. the length of the buffer we pass to the UART.
        let mut next_read_len = buffer.len();
        let mut read_pending = false;

        // Set a flag that we are in this callback handler. This allows us to
        // note that we can wait until all callbacks are finished before
        // starting a new UART receive.
        self.completing_read.set(true);

        // Because clients may issue another read in their callback we need to
        // first copy out all the data, then make the callbacks.
        //
        // Multiple client reads of different sizes can be pending. This code
        // copies the underlying UART read into each of the client buffers.
        self.devices.iter().for_each(|device| {
            if device.receiver {
                device.rx_buffer.take().map(|rxbuf| {
                    let state = device.state.get();
                    // Copy the read into the buffer starting at rx_position
                    let position = device.rx_position.get();
                    let remaining = device.rx_len.get() - position;
                    let len = cmp::min(rx_len, remaining);
                    if state == UartDeviceReceiveState::Receiving
                        || state == UartDeviceReceiveState::Aborting
                    {
                        // debug!("Have {} bytes, copying in bytes {}-{}, {} remain", rx_len, position, position + len, remaining);
                        rxbuf[position..(len + position)].copy_from_slice(&buffer[..len]);
                    }
                    device.rx_position.set(position + len);
                    device.rx_buffer.replace(rxbuf);
                });
            }
        });
        // If the underlying read completes a client read, issue a callback to
        // that client. In the meanwhile, compute the length of the next
        // underlying UART read as the shortest outstanding read, including and
        // new reads setup in the callback. If any client has more to read or
        // has started a new read, issue another underlying UART receive.
        self.devices.iter().for_each(|device| {
            if device.receiver {
                device.rx_buffer.take().map(|rxbuf| {
                    let state = device.state.get();
                    let position = device.rx_position.get();
                    let remaining = device.rx_len.get() - position;
                    // If this finishes the read, signal to the caller,
                    // otherwise update state so next read will fill in
                    // more data.
                    if remaining == 0 {
                        device.state.set(UartDeviceReceiveState::Idle);
                        device.received_buffer(rxbuf, position, rcode, error);
                        // Need to check if receive was called in callback
                        if device.state.get() == UartDeviceReceiveState::Receiving {
                            read_pending = true;
                            next_read_len = cmp::min(next_read_len, device.rx_len.get());
                        }
                    } else if state == UartDeviceReceiveState::Aborting {
                        device.state.set(UartDeviceReceiveState::Idle);
                        device.received_buffer(
                            rxbuf,
                            position,
                            Err(ErrorCode::CANCEL),
                            uart::Error::Aborted,
                        );
                        // Need to check if receive was called in callback
                        if device.state.get() == UartDeviceReceiveState::Receiving {
                            read_pending = true;
                            next_read_len = cmp::min(next_read_len, device.rx_len.get());
                        }
                    } else {
                        device.rx_buffer.replace(rxbuf);
                        next_read_len = cmp::min(next_read_len, remaining);
                        read_pending = true;
                    }
                });
            }
        });

        // After we have finished all callbacks we can replace this buffer. We
        // have to wait to replace this to make sure that a client calling
        // `receive_buffer()` in its callback does not start an underlying UART
        // receive before all callbacks have finished.
        self.buffer.replace(buffer);

        // Clear the flag that we are in this handler.
        self.completing_read.set(false);

        // If either our outstanding receive was longer than the number of bytes
        // we just received, or if a new receive has been started, we start the
        // underlying UART receive again.
        if read_pending {
            if let Err((e, buf)) = self.start_receive(next_read_len) {
                self.buffer.replace(buf);

                // Report the error to all devices
                self.devices.iter().for_each(|device| {
                    if device.receiver {
                        device.rx_buffer.take().map(|rxbuf| {
                            let state = device.state.get();
                            let position = device.rx_position.get();

                            if state == UartDeviceReceiveState::Receiving {
                                device.state.set(UartDeviceReceiveState::Idle);

                                device.received_buffer(
                                    rxbuf,
                                    position,
                                    Err(e),
                                    uart::Error::Aborted,
                                );
                            }
                        });
                    }
                });
            }
        }
    }
}

impl<'a> MuxUart<'a> {
    pub fn new(uart: &'a dyn uart::Uart<'a>, buffer: &'static mut [u8], speed: u32) -> MuxUart<'a> {
        MuxUart {
            uart,
            speed,
            devices: List::new(),
            inflight: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
            completing_read: Cell::new(false),
            deferred_call: DeferredCall::new(),
        }
    }

    pub fn initialize(&self) {
        let _ = self.uart.configure(uart::Parameters {
            baud_rate: self.speed,
            width: uart::Width::Eight,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
    }

    fn do_next_op(&self) {
        if self.inflight.is_none() {
            let mnode = self.devices.iter().find(|node| node.operation.is_some());
            mnode.map(|node| {
                node.tx_buffer.take().map(|buf| {
                    node.operation.take().map(move |op| match op {
                        Operation::Transmit { len } => match self.uart.transmit_buffer(buf, len) {
                            Ok(()) => {
                                self.inflight.set(node);
                            }
                            Err((ecode, buf)) => {
                                node.tx_client.map(move |client| {
                                    node.transmitting.set(false);
                                    client.transmitted_buffer(buf, len, Err(ecode));
                                });
                            }
                        },
                        Operation::TransmitWord { word } => {
                            let rcode = self.uart.transmit_word(word);
                            if rcode != Ok(()) {
                                node.tx_client.map(|client| {
                                    node.transmitting.set(false);
                                    client.transmitted_word(rcode);
                                });
                            }
                        }
                    });
                });
            });
        }
    }

    fn transmit_hint(&self) {
        self.uart.transmit_hint();
    }

    /// Starts a new UART reception, return value denotes whether starting
    /// the reception will issue a callback before the new read. A callback
    /// needs to be issued before the new read if a read was ongoing; the
    /// callback finishes the current read so the new one can start.
    ///
    /// Three cases:
    /// 1. We are in the midst of completing a read: let the `received_buffer()`
    ///    handler restart the reads if needed (return false)
    /// 2. We are in the midst of a read: abort so we can start a new read now
    ///    (return true)
    /// 3. We are idle: start reading (return false)
    fn start_receive(&self, rx_len: usize) -> Result<bool, (ErrorCode, &'static mut [u8])> {
        self.buffer.take().map_or_else(
            || {
                // No rxbuf which means a read is ongoing
                if self.completing_read.get() {
                    // Case (1). Do nothing here, `received_buffer()` handler
                    // will call start_receive when ready.
                    Ok(false)
                } else {
                    // Case (2). Stop the previous read so we can use the
                    // `received_buffer()` handler to recalculate the minimum
                    // length for a read.
                    let _ = self.uart.receive_abort();
                    Ok(true)
                }
            },
            |rxbuf| {
                // Case (3). No ongoing receive calls, we can start one now.
                let len = cmp::min(rx_len, rxbuf.len());
                self.uart.receive_buffer(rxbuf, len)?;
                Ok(false)
            },
        )
    }

    /// Asynchronously executes the next operation, if any. Used by calls
    /// to trigger do_next_op such that it will execute after the call
    /// returns. This is important in case the operation triggers an error,
    /// requiring a callback with an error condition; if the operation
    /// is executed synchronously, the callback may be reentrant (executed
    /// during the downcall). Please see
    /// <https://github.com/tock/tock/issues/1496>
    fn do_next_op_async(&self) {
        self.deferred_call.set();
    }
}

impl DeferredCallClient for MuxUart<'_> {
    fn handle_deferred_call(&self) {
        self.do_next_op();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Operation {
    Transmit { len: usize },
    TransmitWord { word: u32 },
}

#[derive(Copy, Clone, PartialEq)]
enum UartDeviceReceiveState {
    Idle,
    Receiving,
    Aborting,
}

pub struct UartDevice<'a> {
    state: Cell<UartDeviceReceiveState>,
    mux: &'a MuxUart<'a>,
    receiver: bool, // Whether or not to pass this UartDevice incoming messages.
    tx_buffer: TakeCell<'static, [u8]>,
    transmitting: Cell<bool>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_position: Cell<usize>,
    rx_len: Cell<usize>,
    operation: OptionalCell<Operation>,
    next: ListLink<'a, UartDevice<'a>>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
}

impl<'a> UartDevice<'a> {
    pub fn new(mux: &'a MuxUart<'a>, receiver: bool) -> UartDevice<'a> {
        UartDevice {
            state: Cell::new(UartDeviceReceiveState::Idle),
            mux,
            receiver,
            tx_buffer: TakeCell::empty(),
            transmitting: Cell::new(false),
            rx_buffer: TakeCell::empty(),
            rx_position: Cell::new(0),
            rx_len: Cell::new(0),
            operation: OptionalCell::empty(),
            next: ListLink::empty(),
            rx_client: OptionalCell::empty(),
            tx_client: OptionalCell::empty(),
        }
    }

    /// Must be called right after `static_init!()`.
    pub fn setup(&'a self) {
        self.mux.devices.push_head(self);
    }
}

impl uart::TransmitClient for UartDevice<'_> {
    fn transmitted_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        rcode: Result<(), ErrorCode>,
    ) {
        self.tx_client.map(move |client| {
            self.transmitting.set(false);
            client.transmitted_buffer(tx_buffer, tx_len, rcode);
        });
    }

    fn transmitted_word(&self, rcode: Result<(), ErrorCode>) {
        self.tx_client.map(move |client| {
            self.transmitting.set(false);
            client.transmitted_word(rcode);
        });
    }
}
impl uart::ReceiveClient for UartDevice<'_> {
    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rcode: Result<(), ErrorCode>,
        error: uart::Error,
    ) {
        self.rx_client.map(move |client| {
            self.state.set(UartDeviceReceiveState::Idle);
            client.received_buffer(rx_buffer, rx_len, rcode, error);
        });
    }
}

impl<'a> ListNode<'a, UartDevice<'a>> for UartDevice<'a> {
    fn next(&'a self) -> &'a ListLink<'a, UartDevice<'a>> {
        &self.next
    }
}

impl<'a> uart::Transmit<'a> for UartDevice<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    /// Transmit data.
    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len == 0 {
            Err((ErrorCode::SIZE, tx_data))
        } else if self.transmitting.get() {
            Err((ErrorCode::BUSY, tx_data))
        } else {
            self.tx_buffer.replace(tx_data);
            self.transmitting.set(true);
            self.operation.set(Operation::Transmit { len: tx_len });
            self.mux.do_next_op_async();
            Ok(())
        }
    }

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode> {
        if self.transmitting.get() {
            Err(ErrorCode::BUSY)
        } else {
            self.transmitting.set(true);
            self.operation.set(Operation::TransmitWord { word });
            self.mux.do_next_op_async();
            Ok(())
        }
    }

    fn transmit_hint(&self) {
        self.mux.transmit_hint();
    }
}

impl<'a> uart::Receive<'a> for UartDevice<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    /// Receive data until buffer is full.
    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.rx_buffer.is_some() {
            Err((ErrorCode::BUSY, rx_buffer))
        } else if rx_len > rx_buffer.len() {
            Err((ErrorCode::SIZE, rx_buffer))
        } else {
            self.rx_buffer.replace(rx_buffer);
            self.rx_len.set(rx_len);
            self.rx_position.set(0);
            self.state.set(UartDeviceReceiveState::Idle);
            self.mux.start_receive(rx_len)?;
            self.state.set(UartDeviceReceiveState::Receiving);
            Ok(())
        }
    }

    // This virtualized device will abort its read: other devices
    // devices will continue with their reads.
    fn receive_abort(&self) -> Result<(), ErrorCode> {
        self.state.set(UartDeviceReceiveState::Aborting);
        let _ = self.mux.uart.receive_abort();
        Err(ErrorCode::BUSY)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}
