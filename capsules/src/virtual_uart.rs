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
//! )
//! hil::uart::UART::set_receive_client(&sam4l::usart::USART0, uart_mux);
//! hil::uart::UART::set_transmit_client(&sam4l::usart::USART0, uart_mux);
//!
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
//! hil::uart::UART::set_transmit_client(console_uart, console);
//! hil::uart::UART::set_receive_client(console_uart, console);
//! ```

use core::cell::Cell;
use core::cmp;

use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::common::{List, ListLink, ListNode};
use kernel::hil::uart;
use kernel::ReturnCode;

const RX_BUF_LEN: usize = 64;
pub static mut RX_BUF: [u8; RX_BUF_LEN] = [0; RX_BUF_LEN];

pub struct MuxUart<'a> {
    uart: &'a dyn uart::Uart<'a>,
    speed: u32,
    devices: List<'a, UartDevice<'a>>,
    inflight: OptionalCell<&'a UartDevice<'a>>,
    buffer: TakeCell<'static, [u8]>,
    completing_read: Cell<bool>,
    deferred_caller: &'a DynamicDeferredCall,
    handle: OptionalCell<DeferredCallHandle>,
}

impl<'a> uart::TransmitClient for MuxUart<'a> {
    fn transmitted_buffer(&self, tx_buffer: &'static mut [u8], tx_len: usize, rcode: ReturnCode) {
        self.inflight.map(move |device| {
            self.inflight.clear();
            device.transmitted_buffer(tx_buffer, tx_len, rcode);
        });
        self.do_next_op_async();
    }
}

impl<'a> uart::ReceiveClient for MuxUart<'a> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        rcode: ReturnCode,
        error: uart::Error,
    ) {
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
                        device.received_buffer(rxbuf, position, rcode, error);
                        // Need to check if receive was called in callback
                        if device.state.get() == UartDeviceReceiveState::Receiving {
                            read_pending = true;
                        }
                    } else if state == UartDeviceReceiveState::Aborting {
                        device.state.set(UartDeviceReceiveState::Idle);
                        device.received_buffer(
                            rxbuf,
                            position,
                            ReturnCode::ECANCEL,
                            uart::Error::Aborted,
                        );
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
    pub fn new(
        uart: &'a dyn uart::Uart<'a>,
        buffer: &'static mut [u8],
        speed: u32,
        deferred_caller: &'a DynamicDeferredCall,
    ) -> MuxUart<'a> {
        MuxUart {
            uart: uart,
            speed: speed,
            devices: List::new(),
            inflight: OptionalCell::empty(),
            buffer: TakeCell::new(buffer),
            completing_read: Cell::new(false),
            deferred_caller: deferred_caller,
            handle: OptionalCell::empty(),
        }
    }

    pub fn initialize(&self) {
        self.uart.configure(uart::Parameters {
            baud_rate: self.speed,
            width: uart::Width::Eight,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
    }

    pub fn initialize_callback_handle(&self, handle: DeferredCallHandle) {
        self.handle.replace(handle);
    }

    fn do_next_op(&self) {
        if self.inflight.is_none() {
            let mnode = self.devices.iter().find(|node| node.operation.is_some());
            mnode.map(|node| {
                node.tx_buffer.take().map(|buf| {
                    self.inflight.set(node);
                    node.operation.take().map(move |op| match op {
                        Operation::Transmit { len } => {
                            let (rcode, rbuf) = self.uart.transmit_buffer(buf, len);
                            if rcode != ReturnCode::SUCCESS {
                                node.tx_client.map(|client| {
                                    self.inflight.clear();
                                    node.transmitting.set(false);
                                    client.transmitted_buffer(rbuf.unwrap(), 0, rcode);
                                });
                            }
                        }
                        Operation::TransmitWord { word } => {
                            let rcode = self.uart.transmit_word(word);
                            if rcode != ReturnCode::SUCCESS {
                                node.tx_client.map(|client| {
                                    self.inflight.clear();
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
                    self.uart.receive_abort();
                    true
                }
            },
            |rxbuf| {
                let len = cmp::min(rx_len, rxbuf.len());
                self.uart.receive_buffer(rxbuf, len);
                false
            },
        )
    }

    /// Asynchronously executes the next operation, if any. Used by calls
    /// to trigger do_next_op such that it will execute after the call
    /// returns. This is important in case the operation triggers an error,
    /// requiring a callback with an error condition; if the operation
    /// is executed synchronously, the callback may be reentrant (executed
    /// during the downcall). Please see
    ///
    /// https://github.com/tock/tock/issues/1496
    fn do_next_op_async(&self) {
        self.handle.map(|handle| self.deferred_caller.set(*handle));
    }
}

impl<'a> DynamicDeferredCallClient for MuxUart<'a> {
    fn call(&self, _handle: DeferredCallHandle) {
        self.do_next_op();
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

impl uart::UartData<'a> for UartDevice<'a> {}

impl<'a> UartDevice<'a> {
    pub const fn new(mux: &'a MuxUart<'a>, receiver: bool) -> UartDevice<'a> {
        UartDevice {
            state: Cell::new(UartDeviceReceiveState::Idle),
            mux: mux,
            receiver: receiver,
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

impl<'a> uart::TransmitClient for UartDevice<'a> {
    fn transmitted_buffer(&self, tx_buffer: &'static mut [u8], tx_len: usize, rcode: ReturnCode) {
        self.tx_client.map(move |client| {
            self.transmitting.set(false);
            client.transmitted_buffer(tx_buffer, tx_len, rcode);
        });
    }

    fn transmitted_word(&self, rcode: ReturnCode) {
        self.tx_client.map(move |client| {
            self.transmitting.set(false);
            client.transmitted_word(rcode);
        });
    }
}
impl<'a> uart::ReceiveClient for UartDevice<'a> {
    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rcode: ReturnCode,
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

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    /// Transmit data.
    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.transmitting.get() {
            (ReturnCode::EBUSY, Some(tx_data))
        } else {
            self.tx_buffer.replace(tx_data);
            self.transmitting.set(true);
            self.operation.set(Operation::Transmit { len: tx_len });
            self.mux.do_next_op_async();
            (ReturnCode::SUCCESS, None)
        }
    }

    fn transmit_word(&self, word: u32) -> ReturnCode {
        if self.transmitting.get() {
            ReturnCode::EBUSY
        } else {
            self.transmitting.set(true);
            self.operation.set(Operation::TransmitWord { word: word });
            self.mux.do_next_op_async();
            ReturnCode::SUCCESS
        }
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
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.rx_buffer.is_some() {
            (ReturnCode::EBUSY, Some(rx_buffer))
        } else {
            self.rx_buffer.replace(rx_buffer);
            self.rx_len.set(rx_len);
            self.rx_position.set(0);
            self.state.set(UartDeviceReceiveState::Idle);
            self.mux.start_receive(rx_len);
            self.state.set(UartDeviceReceiveState::Receiving);
            (ReturnCode::SUCCESS, None)
        }
    }

    // This virtualized device will abort its read: other devices
    // devices will continue with their reads.
    fn receive_abort(&self) -> ReturnCode {
        self.state.set(UartDeviceReceiveState::Aborting);
        self.mux.uart.receive_abort();
        ReturnCode::EBUSY
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}
