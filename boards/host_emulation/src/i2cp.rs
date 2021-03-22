//! I2C peripheral driver

use core::cell::{Cell, RefCell};
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::debug;
use std::path::Path;
use std::sync::mpsc::TryRecvError;

use crate::async_data_stream::AsyncDataStream;

const SOCKET_PATH_BASE: &str = "/tmp/he_i2c";

pub struct I2CPeripheral<'a> {
    client: OptionalCell<&'static dyn kernel::hil::i2c::I2CHwSlaveClient>,
    stream: RefCell<Option<AsyncDataStream>>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<u8>,
    rx_index: Cell<u8>,
    rx_expected: Cell<u8>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<u8>,
    tx_index: Cell<u8>,
    id: &'a str,
}

impl<'a> I2CPeripheral<'a> {
    pub const fn new(id: &'a str) -> I2CPeripheral {
        I2CPeripheral {
            client: OptionalCell::empty(),
            stream: RefCell::new(None),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
            rx_expected: Cell::new(0),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            id,
        }
    }

    pub fn initialize(&mut self) {
        *self.stream.borrow_mut() = Some(AsyncDataStream::new_socket_stream(
            Path::new(&(SOCKET_PATH_BASE.to_owned() + self.id)),
            false,
        ));
    }

    pub fn handle_pending_requests(&self) {
        if self.tx_buffer.is_some() {
            self.client.map(|client| {
                self.tx_buffer.take().map(|tx_buf| {
                    client.command_complete(
                        tx_buf,
                        self.tx_len.get(),
                        kernel::hil::i2c::SlaveTransmissionType::Read,
                    );
                });
            });
        };

        let mut call_write_complete = false;

        if let Some(stream) = &mut *self.stream.borrow_mut() {
            match stream.try_recv() {
                Ok(byte) => {
                    if self.rx_expected.get() == 0 {
                        self.rx_expected.set(byte);
                        if self.rx_buffer.is_none() {
                            self.client.map(|client| {
                                // Discard (hand back) read buffer, if any, we do not need that right now.
                                // Based on implementation in dauntless.
                                self.tx_buffer.take().map(|tx_buf| {
                                    client.command_complete(
                                        tx_buf,
                                        0,
                                        kernel::hil::i2c::SlaveTransmissionType::Read,
                                    );
                                });
                                client.write_expected();
                            });
                            assert!(
                                self.rx_buffer.is_some(),
                                "client.write_expected has not called write_receive"
                            );
                        }
                    } else {
                        self.rx_buffer.map(|rx_buf| {
                            let mut idx = self.rx_index.get();
                            if idx < self.rx_len.get() {
                                rx_buf[idx as usize] = byte;
                                idx += 1;
                                self.rx_index.set(idx);
                            } else {
                                debug!("ERROR: received byte over size of rx_buffer: {:x}", byte);
                            }
                            self.rx_expected.set(self.rx_expected.get() - 1);
                        });
                        if self.rx_expected.get() == 0 {
                            call_write_complete = true;
                        }
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(err) => {
                    debug!("ERROR: receive error: {:?}", err);
                }
            }
        }

        if call_write_complete {
            self.client.map(|client| {
                client.command_complete(
                    self.rx_buffer.take().unwrap(),
                    self.rx_index.get(),
                    kernel::hil::i2c::SlaveTransmissionType::Write,
                );
            });
        }
    }
}

impl<'a> kernel::hil::i2c::I2CSlave for I2CPeripheral<'a> {
    fn set_slave_client(&self, client: &'static dyn kernel::hil::i2c::I2CHwSlaveClient) {
        self.client.set(client);
    }

    fn enable(&self) {}

    fn disable(&self) {}

    fn set_address(&self, _: u8) {
        // ignore the address, emulated slave will receive only data for itself
    }

    fn write_receive(&self, data: &'static mut [u8], mut max_len: u8) {
        if self.rx_buffer.is_some() {
            debug!("ERROR: I2CPeripheral::write_receive() already have a rx_buffer");
        }
        if max_len as usize > data.len() {
            debug!(
                "ERROR: I2CPeripheral::write_receive() max_len={} exceeds data.len()={}",
                max_len,
                data.len()
            );
            max_len = data.len() as u8;
        }
        self.rx_buffer.replace(data);
        self.rx_len.set(max_len);
        self.rx_index.set(0);
    }

    fn read_send(&self, data: &'static mut [u8], mut max_len: u8) {
        if self.tx_buffer.is_some() {
            debug!("ERROR: I2CPeripheral::read_send() already have a tx_buffer");
        }
        if max_len as usize > data.len() {
            debug!(
                "ERROR: I2CPeripheral::read_send() max_len={} exceeds data.len()={}",
                max_len,
                data.len()
            );
            max_len = data.len() as u8;
        }
        self.tx_buffer.replace(data);
        self.tx_len.set(max_len);
        self.tx_index.set(0);

        if let Some(stream) = &mut *self.stream.borrow_mut() {
            self.tx_buffer.map(|data| {
                let buf: [u8; 1] = [max_len];
                if let Err(err) = stream.write_all(&buf) {
                    debug!("ERROR: write error: {:?}", err);
                }
                if let Err(err) = stream.write_all(&data[0..max_len as usize]) {
                    debug!("ERROR: write error: {:?}", err);
                }
            });
        }
    }

    fn listen(&self) {
        panic!("unimplemented");
    }
}
