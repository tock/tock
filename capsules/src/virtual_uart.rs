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
//! ```
//! // Create a shared UART channel for the console and for kernel debug.
//! let uart_mux = static_init!(
//!     MuxUart<'static>,
//!     MuxUart::new(&sam4l::usart::USART0, &mut capsules::virtual_uart::RX_BUF)
//! );
//! hil::uart::UART::set_client(&sam4l::usart::USART0, uart_mux);

//! // Create a UartDevice for the console.
//! let console_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
//! console_uart.setup(); // This is important!
//! let console = static_init!(
//!     capsules::console::Console<UartDevice>,
//!     capsules::console::Console::new(
//!         console_uart,
//!         115200,
//!         &mut capsules::console::WRITE_BUF,
//!         &mut capsules::console::READ_BUF,
//!         kernel::Grant::create()
//!     )
//! );
//! hil::uart::UART::set_client(console_uart, console);
//! ```

use core::cell::Cell;
use core::cmp;

use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::{List, ListLink, ListNode};
use kernel::hil;
use kernel::hil::uart;
use kernel::ReturnCode;

const RX_BUF_LEN: usize = 64;
pub static mut RX_BUF: [u8; RX_BUF_LEN] = [0; RX_BUF_LEN];

pub struct MuxUart<'a> {
    uart: &'a hil::uart::UART,
    speed: u32,
    devices: List<'a, UartDevice<'a>>,
    inflight: OptionalCell<&'a UartDevice<'a>>,
    buffer: TakeCell<'static, [u8]>,
    completing_read: Cell<bool>,
}

impl<'a> hil::uart::Client for MuxUart<'a> {
    fn transmit_complete(&self, tx_buffer: &'static mut [u8], error: hil::uart::Error) {
        self.inflight.map(move |device| {
            self.inflight.clear();
            device.transmit_complete(tx_buffer, error);
        });
        self.do_next_op();
    }

    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, error: hil::uart::Error) {
        let mut next_read_len = RX_BUF_LEN;
        let mut read_pending = false;
        self.completing_read.set(true);
        // Because clients may issue another read in their callback we need to first
        // copy out all the data, then make the callbacks.
        //
        // Multiple client reads of different sizes can be pending. This code copies
        // the underlying UART read into each of the client buffers; if the
        // underlying read completes a client read, issue a callback to that
        // client. In the meanwhile, compute the length of the next underlying
        // UART read: if any client has more to read, issue another underlying
        // read.
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
                        //                        debug!("Have {} bytes, copying in bytes {}-{}, {} remain", rx_len, position, position + len, remaining);
                        for i in 0..len {
                            rxbuf[position + i] = buffer[i];
                        }
                    }
                    device.rx_position.set(position + len);
                    device.rx_buffer.replace(rxbuf);
                });
            }
        });
        self.buffer.replace(buffer);
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
                        device.receive_complete(rxbuf, position, error);
                        // Need to check if receive was called in callback
                        if device.state.get() == UartDeviceReceiveState::Receiving {
                            read_pending = true;
                        }
                    } else if state == UartDeviceReceiveState::Aborting {
                        device.state.set(UartDeviceReceiveState::Idle);
                        device.receive_complete(rxbuf, position, hil::uart::Error::Aborted);
                        // Need to check if receive was called in callback
                        if device.state.get() == UartDeviceReceiveState::Receiving {
                            read_pending = true;
                        }
                    } else {
                        device.rx_buffer.replace(rxbuf);
                        next_read_len = cmp::min(next_read_len, remaining);
                        read_pending = true;
                    }
                });
            }
        });
        self.completing_read.set(false);
        if read_pending {
            self.start_receive(next_read_len);
        }
    }
}

impl<'a> MuxUart<'a> {
    pub fn new(uart: &'a hil::uart::UART, buffer: &'static mut [u8], speed: u32) -> MuxUart<'a> {
        MuxUart {
            uart: uart,
            speed: speed,
            devices: List::new(),
            inflight: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
            completing_read: Cell::new(false),
        }
    }

    pub fn initialize(&self) {
        self.uart.configure(uart::UARTParameters {
            baud_rate: self.speed,
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
                    node.operation.map(move |op| match op {
                        Operation::Transmit { len } => self.uart.transmit(buf, *len),
                    });
                });
                node.operation.clear();
                self.inflight.set(node);
            });
        }
    }

    /// Starts a new UART reception, return value denotes whether starting
    /// the reception will issue a callback before the new read. A callback
    /// needs to be issued before the new read if a read was ongoing; the
    /// callback finishes the current read so the new one can start.
    /// Three cases:
    ///    1) We are in the midst of completing a read: let it restart the
    ///       reads if needed (return false)
    ///    2) We are in the midst of a read: abort so we can start a new
    ///       read now (return true)
    ///    3) We are idle: start reading (return false)
    fn start_receive(&self, rx_len: usize) -> bool {
        self.buffer.take().map_or_else(
            || {
                // No rxbuf which means a read is ongoing
                if self.completing_read.get() {
                    // Do nothing, read completion will call start_receive when ready
                    false
                } else {
                    self.uart.abort_receive();
                    true
                }
            },
            |rxbuf| {
                let len = cmp::min(rx_len, rxbuf.len());
                self.uart.receive(rxbuf, len);
                false
            },
        )
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Operation {
    Transmit { len: usize },
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
    rx_buffer: TakeCell<'static, [u8]>,
    rx_position: Cell<usize>,
    rx_len: Cell<usize>,
    operation: OptionalCell<Operation>,
    next: ListLink<'a, UartDevice<'a>>,
    client: OptionalCell<&'a hil::uart::Client>,
}

impl<'a> UartDevice<'a> {
    pub const fn new(mux: &'a MuxUart<'a>, receiver: bool) -> UartDevice<'a> {
        UartDevice {
            state: Cell::new(UartDeviceReceiveState::Idle),
            mux: mux,
            receiver: receiver,
            tx_buffer: TakeCell::empty(),
            rx_buffer: TakeCell::empty(),
            rx_position: Cell::new(0),
            rx_len: Cell::new(0),
            operation: OptionalCell::empty(),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }

    /// Must be called right after `static_init!()`.
    pub fn setup(&'a self) {
        self.mux.devices.push_head(self);
    }
}

impl<'a> hil::uart::Client for UartDevice<'a> {
    fn transmit_complete(&self, tx_buffer: &'static mut [u8], error: hil::uart::Error) {
        self.client.map(move |client| {
            client.transmit_complete(tx_buffer, error);
        });
    }

    fn receive_complete(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        error: hil::uart::Error,
    ) {
        self.client.map(move |client| {
            self.state.set(UartDeviceReceiveState::Idle);
            client.receive_complete(rx_buffer, rx_len, error);
        });
    }
}

impl<'a> ListNode<'a, UartDevice<'a>> for UartDevice<'a> {
    fn next(&'a self) -> &'a ListLink<'a, UartDevice<'a>> {
        &self.next
    }
}

impl<'a> hil::uart::UART for UartDevice<'a> {
    fn set_client(&self, client: &'a hil::uart::Client) {
        self.client.set(client);
    }

    // Ideally this wouldn't be here, and if we ever create a "UartDevice" trait
    // in the HIL then this could be removed.
    fn configure(&self, params: hil::uart::UARTParameters) -> ReturnCode {
        self.mux.uart.configure(params)
    }

    /// Transmit data.
    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        self.tx_buffer.replace(tx_data);
        self.operation.set(Operation::Transmit { len: tx_len });
        self.mux.do_next_op();
    }

    /// Receive data until buffer is full.
    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_position.set(0);
        self.state.set(UartDeviceReceiveState::Idle);
        self.mux.start_receive(rx_len);
        self.state.set(UartDeviceReceiveState::Receiving);
    }

    // This virtualized device will abort its read: other devices
    // devices will continue with their reads.
    fn abort_receive(&self) {
        self.state.set(UartDeviceReceiveState::Aborting);
        self.mux.uart.abort_receive();
    }
}
