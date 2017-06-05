//! Kernel implementation of signbus_io_interface
//! apps/libsignpost/signbus_io_interface.c -> kernel/tock/capsules/src/signbus_io_interface.rs
//! By: Justin Hsieh
//!
//! Usage
//! -----
//!
//! ```rust
//! let io_layer = static_init!(
//!     capsules::signbus::io_layer::SignbusIOLayer<'static>,
//!     capsules::signbus::io_layer::SignbusIOLayer::new(port_layer,
//!     	&mut capsules::signbus::io_layer::BUFFER0,
//!     	&mut capsules::signbus::io_layer::BUFFER1
//!  ));
//!
//! ```

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;

// Capsules
use signbus;
use signbus::{support, port_layer, protocol_layer};

/// Buffers used for receiving and data storage.
pub static mut BUFFER0: [u8; 256] = [0; 256];
pub static mut BUFFER1: [u8; 256] = [0; 256];
pub static mut BUFFER2: [u8; 256] = [0; 256];


/// SignbusIOLayer handles packet sending and receiving.
pub struct SignbusIOLayer<'a> {
    port_layer: &'a port_layer::PortLayer,

    this_device_address: Cell<u8>,
    sequence_number: Cell<u16>,

    message_seq_no: Cell<u16>,
    message_src: Cell<u8>,
    length_received: Cell<usize>,

    client: Cell<Option<&'static protocol_layer::SignbusProtocolLayer<'static>>>,

    send_buf: TakeCell<'static, [u8]>,
    recv_buf: TakeCell<'static, [u8]>,
    data_buf: TakeCell<'static, [u8]>,
}

/// IOLayerClient for I2C sending/receiving callbacks. Implemented by SignbusProtocolLayer.
pub trait IOLayerClient {
    // Called when a new packet is received over I2C.
    fn packet_received(&self, data: &'static mut [u8], length: usize, error: support::Error);

    // Called when an I2C master write command is complete.
    fn packet_sent(&self, data: &'static mut [u8], error: support::Error);

    // Called when an I2C slave read has completed.
    fn packet_read_from_slave(&self);
}

impl<'a> SignbusIOLayer<'a> {
    pub fn new(port_layer: &'a port_layer::PortLayer,
               send_buf: &'static mut [u8],
               recv_buf: &'static mut [u8],
               data_buf: &'static mut [u8])
               -> SignbusIOLayer<'a> {

        SignbusIOLayer {
            port_layer: port_layer,

            this_device_address: Cell::new(0),
            sequence_number: Cell::new(0),

            message_seq_no: Cell::new(0),
            message_src: Cell::new(0),
            length_received: Cell::new(0),

            client: Cell::new(None),

            send_buf: TakeCell::new(send_buf),
            recv_buf: TakeCell::new(recv_buf),
            data_buf: TakeCell::new(data_buf),
        }
    }

    pub fn set_client(&self, client: &'static protocol_layer::SignbusProtocolLayer) -> ReturnCode {
        self.client.set(Some(client));
        ReturnCode::SUCCESS
    }

    // Initialization routine to set up the slave address for this device.
    // MUST be called before any other methods.
    pub fn signbus_io_init(&self, address: u8) -> ReturnCode {
        self.this_device_address.set(address);
        self.port_layer.init(address);

        ReturnCode::SUCCESS
    }


    // Send call, callback will handle sending multiple packets if data is
    // longer than I2C_MAX_DATA_LEN.
    pub fn signbus_io_send(&self,
                           dest: u8,
                           encrypted: bool,
                           data: &'static mut [u8],
                           len: usize)
                           -> ReturnCode {

        self.sequence_number.set(self.sequence_number.get() + 1);

        // Network Flags
        let flags: support::SignbusNetworkFlags = support::SignbusNetworkFlags {
            is_fragment: (len > support::I2C_MAX_DATA_LEN) as u8,
            is_encrypted: encrypted as u8,
            rsv_wire_bit5: 0,
            rsv_wire_bit4: 0,
            version: 0x1,
        };

        // Network Header
        let header: support::SignbusNetworkHeader = support::SignbusNetworkHeader {
            flags: flags,
            src: self.this_device_address.get(),
            sequence_number: self.sequence_number.get(),
            length: (support::HEADER_SIZE + len) as u16,
            fragment_offset: 0,
        };

        if header.flags.is_fragment == 1 {
            // Save all data in order to send in multiple packets
            self.data_buf.map(|data_buf| {
                let d = &mut data_buf.as_mut()[0..len];
                for (i, c) in data[0..len].iter().enumerate() {
                    d[i] = *c;
                }
            });

            // Copy data from slice into sized array to package into packet
            let mut data_copy: [u8; support::I2C_MAX_DATA_LEN] = [0; support::I2C_MAX_DATA_LEN];
            for (i, c) in data[0..support::I2C_MAX_DATA_LEN].iter().enumerate() {
                data_copy[i] = *c;
            }

            // Packet
            let packet: support::Packet = support::Packet {
                header: header,
                data: data_copy,
            };

            let rc = self.port_layer.i2c_master_write(dest, packet, support::I2C_MAX_LEN);
            if rc != ReturnCode::SUCCESS {
                return rc;
            }

        } else {
            // Copy data from slice into sized array to package into packet
            let mut data_copy: [u8; support::I2C_MAX_DATA_LEN] = [0; support::I2C_MAX_DATA_LEN];
            for (i, c) in data[0..len].iter().enumerate() {
                data_copy[i] = *c;
            }

            // Packet
            let packet: support::Packet = support::Packet {
                header: header,
                data: data_copy,
            };

            let rc = self.port_layer.i2c_master_write(dest, packet, len + support::HEADER_SIZE);
            if rc != ReturnCode::SUCCESS {
                return rc;
            }
        }

        self.send_buf.replace(data);

        ReturnCode::SUCCESS
    }

    // Recv call, listen for messages and callback handles stitching multiple packets together.
    pub fn signbus_io_recv(&self, buffer: &'static mut [u8]) -> ReturnCode {

        self.recv_buf.replace(buffer);

        let rc = self.port_layer.i2c_slave_listen();
        if rc != ReturnCode::SUCCESS {
            return rc;
        }

        ReturnCode::SUCCESS
    }
}


impl<'a> signbus::port_layer::PortLayerClientI2C for SignbusIOLayer<'a> {
    // Packet received, decipher packet and if needed, stitch packets together or callback upward.
    fn packet_received(&self, packet: support::Packet, length: u8, error: support::Error) {

        // Error checking
        if error != support::Error::CommandComplete {
            // Callback protocol_layer
            self.client.get().map(|client| {
                self.recv_buf
                    .take()
                    .map(|recv_buf| { client.packet_received(recv_buf, length as usize, error); });
            });
            // Reset
            self.length_received.set(0);
            return;
            // TODO: implement sending error message to source
        }

        // Record needed packet data
        let seq_no = packet.header.sequence_number;
        let src = packet.header.src;
        let more_packets = packet.header.flags.is_fragment;
        let offset = packet.header.fragment_offset as usize;
        let remainder = packet.header.length as usize - support::HEADER_SIZE -
                        packet.header.fragment_offset as usize;

        // First packet
        if self.length_received.get() == 0 {
            // Save src and seq_no
            self.message_seq_no.set(seq_no);
            self.message_src.set(src);
        }
        // Subsequent packets
        else {
            // If new src, drop current packet
            if self.message_seq_no.get() != seq_no || self.message_src.get() != src {
                // Save new src and seq_no
                self.message_seq_no.set(seq_no);
                self.message_src.set(src);
                // TODO: call some error?

                // Reset
                self.length_received.set(0);
            }
        }

        // More packets
        if more_packets == 1 {
            // Copy data and update length_received
            self.recv_buf.map(|recv_buf| {
                let d = &mut recv_buf.as_mut()[offset..offset + support::I2C_MAX_DATA_LEN];
                for (i, c) in packet.data[0..support::I2C_MAX_DATA_LEN].iter().enumerate() {
                    d[i] = *c;
                }
            });
            self.length_received.set(self.length_received.get() + support::I2C_MAX_DATA_LEN);
        }
        // Last packet
        else {
            // Copy data and update length_received
            self.recv_buf.map(|recv_buf| {
                let d = &mut recv_buf.as_mut()[offset..offset + remainder];
                for (i, c) in packet.data[0..remainder].iter().enumerate() {
                    d[i] = *c;
                }
            });
            self.length_received.set(self.length_received.get() + remainder);

            // Callback protocol_layer
            self.client.get().map(|client| {
                self.recv_buf.take().map(|recv_buf| {
                    client.packet_received(recv_buf, self.length_received.get(), error);
                });
            });

            // Reset
            self.length_received.set(0);
        }

    }

    // Packet has finished sending. If needed, send more or callback upward.
    fn packet_sent(&self, mut packet: support::Packet, error: signbus::support::Error) {

        // If error, stop sending and propogate up
        if error != support::Error::CommandComplete {
            // Callback protocol_layer
            self.client.get().map(move |client| {
                self.send_buf.take().map(|send_buf| { client.packet_sent(send_buf, error); });
            });
            return;
        }

        if packet.header.flags.is_fragment == 1 {
            // Update fragment offset
            let offset = support::I2C_MAX_DATA_LEN + packet.header.fragment_offset as usize;
            packet.header.fragment_offset = offset as u16;

            // Determines if this is last packet and update is_fragment
            let data_left_to_send = packet.header.length as usize - support::HEADER_SIZE - offset;
            let more_packets = data_left_to_send as usize > support::I2C_MAX_DATA_LEN;
            packet.header.flags.is_fragment = more_packets as u8;

            if more_packets {
                // Copy next frame of data from data_buf into packet
                self.data_buf.map(|data_buf| {
                    let d = &mut data_buf.as_mut()[offset..offset + support::I2C_MAX_DATA_LEN];
                    for (i, c) in packet.data[0..support::I2C_MAX_DATA_LEN].iter_mut().enumerate() {
                        *c = d[i];
                    }
                });

                self.port_layer.i2c_master_write(packet.header.src, packet, support::I2C_MAX_LEN);

            } else {
                // Copy next frame of data from data_buf into packet
                self.data_buf.map(|data_buf| {
                    let d = &mut data_buf.as_mut()[offset..offset + data_left_to_send];
                    for (i, c) in packet.data[0..data_left_to_send].iter_mut().enumerate() {
                        *c = d[i];
                    }
                });

                self.port_layer.i2c_master_write(packet.header.src,
                                                 packet,
                                                 data_left_to_send + support::HEADER_SIZE);
            }

        } else {
            // Callback protocol_layer
            self.client.get().map(move |client| {
                self.send_buf.take().map(|send_buf| { client.packet_sent(send_buf, error); });
            });
        }

    }

    fn packet_read_from_slave(&self) {
        // TODO: implement slave write/ master read
        unimplemented!("Implement slave write/ master read.");
    }
}
