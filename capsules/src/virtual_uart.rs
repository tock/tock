//! Virtualizes the UART interface to multiple clients.
//!
//! Intended as an example for a generic virtualization approach.
//!
//! - Author: Philip Levis
//! - Date: Jan 12 2017

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;
use kernel::common::virtualizer::{QueuedCall, CallQueue, Dequeued};
use kernel::hil::uart;

pub struct UartMux<'a> {
    uart: &'a uart::UART<'a>,
    busy: Cell<bool>,
    queue: CallQueue<'a>,
}

pub struct VirtualUartDevice<'a> {
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    queued_call: QueuedCall<'a>,
    mux: &'a UartMux<'a>,
    client: Cell<Option<&'a uart::Client>>,
}

impl<'a> UartMux<'a> {
    pub fn new(uart: &'a uart::UART<'a>) -> UartMux<'a> {
        UartMux {
            uart: uart,
            busy: Cell::new(false),
            queue: CallQueue::new(),
        }
    }

    pub fn busy(&self) -> bool {
        self.busy.get()
    }

    pub fn next(&self) -> ReturnCode {
        if !self.busy() {
            self.busy.set(true);
            self.queue.dequeue_and_trigger();
        }
        ReturnCode::SUCCESS
    }

    pub fn clear_busy(&self) {
        self.busy.set(false);
    }

    pub fn set_client(&self, client: &'a uart::Client) {
        self.uart.set_client(client);
    }
}

impl<'a> VirtualUartDevice<'a> {
    pub fn new(mux: &'a UartMux<'a>) -> VirtualUartDevice<'a> {
        VirtualUartDevice {
            tx_buffer: TakeCell::empty(),
            queued_call: QueuedCall::new(&mux.queue),
            mux: mux,
            client: Cell::new(None),
            tx_len: Cell::new(0),
        }
    }

    pub fn init(&'a self, client: &'static uart::Client) {
        self.client.set(Some(client));
        self.queued_call.set_callback(self);
    }
}

impl<'a> uart::Client for VirtualUartDevice<'a> {
    fn transmit_complete(&self, tx_buffer: &'static mut [u8], error: uart::Error) {
        self.client.get().map(move |c| c.transmit_complete(tx_buffer, error));
        self.mux.clear_busy();
        self.mux.next();
    }
    fn receive_complete(&self,
                        _rx_buffer: &'static mut [u8],
                        _rx_len: usize,
                        _error: uart::Error) {
        // do nothing, this should not be called
    }
}

impl<'a> Dequeued<'a> for VirtualUartDevice<'a> {
    fn id(&'a self) -> u32 {
        0
    }
    fn dequeued(&'a self) {
        self.mux.set_client(self);
        self.tx_buffer.take().map(|buf| { self.mux.uart.transmit(buf, self.tx_len.get()); });
    }
}

impl<'a> uart::UART<'a> for VirtualUartDevice<'a> {
    /// Set the client for this UART peripheral. The client will be
    /// called when events finish.
    fn set_client(&self, client: &'a uart::Client) {
        self.client.set(Some(client));
    }

    /// Initialize UART
    /// Panics if UARTParams are invalid for the current chip.
    fn init(&self, _params: uart::UARTParams) {
        // Do nothing, shouldn't have a control path here
    }

    /// Transmit data.
    fn transmit(&'a self, tx_data: &'static mut [u8], tx_len: usize) {
        if self.queued_call.insert() {
            self.tx_len.set(tx_len);
            self.tx_buffer.replace(tx_data);
            self.mux.next();
        }
    }

    /// Receive data until buffer is full.
    fn receive(&self, _rx_buffer: &'static mut [u8], _rx_len: usize) {
        // Should not be part of this trait.
    }
}
