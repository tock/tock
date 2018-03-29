//! Kernel implementation of signbus_protocol_layer
//! apps/libsignpost/signbus_protocol_layer.c -> kernel/tock/capsules/src/signbus_protocol_layer.rs
//! By: Justin Hsieh
//!
//! Usage
//! -----
//!
//! ```rust
//! let protocol_layer = static_init!(
//!     capsules::signbus::protocol_layer::SignbusProtocolLayer<'static>,
//!     capsules::signbus::protocol_layer::SignbusProtocolLayer::new(io_layer,
//! ));
//!
//! ```

use core::cell::Cell;
use kernel::ReturnCode;

// Capsules
use signbus::{support, io_layer, app_layer};

/// Buffers not present because encryption and decryption not available.

/// SignbusProtocolLayer to handle encryption and decryption of messages.
pub struct SignbusProtocolLayer<'a> {
    io_layer: &'a io_layer::SignbusIOLayer<'a>,

    client: Cell<Option<&'static app_layer::SignbusAppLayer<'static>>>,
}

/// ProtocolLayerClient for I2C sending/receiving callbacks. Implemented by SignbusAppLayer.
pub trait ProtocolLayerClient {
    // Called when a new packet is received over I2C.
    fn packet_received(&self, data: &'static mut [u8], length: usize, error: support::Error);

    // Called when an I2C master write command is complete.
    fn packet_sent(&self, data: &'static mut [u8], error: support::Error);

    // Called when an I2C slave read has completed.
    fn packet_read_from_slave(&self);
}

impl<'a> SignbusProtocolLayer<'a> {
    pub fn new(io_layer: &'a io_layer::SignbusIOLayer) -> SignbusProtocolLayer<'a> {

        SignbusProtocolLayer {
            io_layer: io_layer,
            client: Cell::new(None),
        }
    }

    pub fn set_client(&self, client: &'static app_layer::SignbusAppLayer) -> ReturnCode {
        self.client.set(Some(client));
        ReturnCode::SUCCESS
    }

    pub fn signbus_protocol_send(&self,
                                 dest: u8,
                                 data: &'static mut [u8],
                                 len: usize)
                                 -> ReturnCode {
        // TODO: encryption not availabe in Rust
        let encrypted: bool = false;

        self.io_layer.signbus_io_send(dest, encrypted, data, len)
    }

    pub fn signbus_protocol_recv(&self, buffer: &'static mut [u8]) -> ReturnCode {
        self.io_layer.signbus_io_recv(buffer)
    }
}

impl<'a> io_layer::IOLayerClient for SignbusProtocolLayer<'a> {
    // Called when a new packet is received over I2C.
    fn packet_received(&self, data: &'static mut [u8], length: usize, error: support::Error) {
        // TODO: decryption not available in Rust
        self.client.get().map(move |client| { client.packet_received(data, length, error); });
    }

    // Called when an I2C master write command is complete.
    fn packet_sent(&self, data: &'static mut [u8], error: support::Error) {
        self.client.get().map(move |client| { client.packet_sent(data, error); });
    }

    // Called when an I2C slave read has completed.
    fn packet_read_from_slave(&self) {
        // TODO: implement slave write/ master read
        unimplemented!("Implement slave write/ master read.");
    }
}
