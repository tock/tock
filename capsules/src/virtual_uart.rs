//! Virtualize a UART bus.
//!
//! This allows multiple Tock capsules to use the same UART bus. This is likely
//! most useful for `printf()` like applications where multiple things want to
//! write to the same UART channel.
//!
//! Clients can choose if they want to receive. Incoming messages will be sent
//! to all clients that have enabled receiving.
//!
//! `UartMux` provides shared access to a single UART bus for multiple users.
//! `UartDevice` provides access for a single client.
//!
//! Usage
//! -----
//!
//! ```
//! // Create a shared UART channel for the console and for kernel debug.
//! let uart_mux = static_init!(
//!     UartMux<'static>,
//!     UartMux::new(&sam4l::usart::USART0, &mut capsules::virtual_uart::RX_BUF)
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
use kernel::ReturnCode;

pub static mut RX_BUF: [u8; 64] = [0; 64];

pub struct UartMux<'a> {
    uart: &'a hil::uart::UART,
    devices: List<'a, UartDevice<'a>>,
    inflight: OptionalCell<&'a UartDevice<'a>>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl<'a> hil::uart::Client for UartMux<'a> {
    fn transmit_complete(&self, tx_buffer: &'static mut [u8], error: hil::uart::Error) {
        self.inflight.map(move |device| {
            self.inflight.clear();
            device.transmit_complete(tx_buffer, error);
        });
        self.do_next_op();
    }

    fn receive_complete(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        error: hil::uart::Error,
    ) {
        // Pass the data to each receiver that wants to receive messages.
        self.devices.iter().for_each(|device| {
            if device.receiver {
                device.rx_buffer.take().map(|rxbuf| {
                    let len = cmp::min(rx_len, device.rx_len.get());
                    for i in 0..len {
                        rxbuf[i] = rx_buffer[i];
                    }
                    device.receive_complete(rxbuf, len, error);
                });
            }
        });
        self.rx_buffer.replace(rx_buffer);
    }
}

impl<'a> UartMux<'a> {
    pub fn new(uart: &'a hil::uart::UART, rx_buffer: &'static mut [u8]) -> UartMux<'a> {
        UartMux {
            uart: uart,
            devices: List::new(),
            inflight: OptionalCell::empty(),
            rx_buffer: TakeCell::new(rx_buffer),
        }
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

    /// If we are not currently listening, start listening.
    fn start_receive(&self, rx_len: usize) {
        self.rx_buffer.take().map(|rxbuf| {
            let len = cmp::min(rx_len, rxbuf.len());
            self.uart.receive(rxbuf, len);
        });
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Operation {
    Transmit { len: usize },
}

pub struct UartDevice<'a> {
    mux: &'a UartMux<'a>,
    receiver: bool, // Whether or not to pass this UartDevice incoming messages.
    tx_buffer: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    operation: OptionalCell<Operation>,
    next: ListLink<'a, UartDevice<'a>>,
    client: OptionalCell<&'a hil::uart::Client>,
}

impl<'a> UartDevice<'a> {
    pub const fn new(mux: &'a UartMux<'a>, receiver: bool) -> UartDevice<'a> {
        UartDevice {
            mux: mux,
            receiver: receiver,
            tx_buffer: TakeCell::empty(),
            rx_buffer: TakeCell::empty(),
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
        self.mux.start_receive(rx_len);
    }

    fn abort_receive(&self) {
        unimplemented!();
    }
}
