//! Kernel implementation of signbus_app_layer
//! apps/libsignpost/signbus_app_layer.c -> kernel/tock/capsules/src/signbus_app_layer.rs
//! By: Justin Hsieh
//!
//! Usage
//! -----
//!
//! ```rust
//! // Signbus App Layer
//! let app_layer = static_init!(
//!     capsules::signbus::app_layer::SignbusAppLayer<'static>,
//!     capsules::signbus::app_layer::SignbusAppLayer::new(protocol_layer,
//!             &mut capsules::signbus::app_layer::BUFFER0,
//!             &mut capsules::signbus::app_layer::BUFFER1
//! ));
//!
//! ```


use core::cell::Cell;
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;

// Capsules
use signbus::{protocol_layer, support, test_signbus_init};

/// Buffers used to concatenate message information.
pub static mut BUFFER0: [u8; 256] = [0; 256];
pub static mut BUFFER1: [u8; 256] = [0; 256];

/// SignbusAppLayer to handle application messages.
pub struct SignbusAppLayer<'a> {
    protocol_layer: &'a protocol_layer::SignbusProtocolLayer<'a>,
    payload: TakeCell<'static, [u8]>,
    send_buf: TakeCell<'static, [u8]>,
    client: Cell<Option<&'static test_signbus_init::SignbusInitialization<'static>>>,
}

/// AppLayerClient for I2C sending/receiving callbacks. Implemented by SignbusInitialization.
pub trait AppLayerClient {
    // Called when a new packet is received over I2C.
    fn packet_received(&self, data: &'static mut [u8], length: usize, error: support::Error);

    // Called when an I2C master write command is complete.
    fn packet_sent(&self, data: &'static mut [u8], error: support::Error);

    // Called when an I2C slave read has completed.
    fn packet_read_from_slave(&self);
}

impl<'a> SignbusAppLayer<'a> {
    pub fn new(protocol_layer: &'a protocol_layer::SignbusProtocolLayer,
               payload: &'static mut [u8],
               send_buf: &'static mut [u8])
               -> SignbusAppLayer<'a> {

        SignbusAppLayer {
            protocol_layer: protocol_layer,
            payload: TakeCell::new(payload),
            send_buf: TakeCell::new(send_buf),
            client: Cell::new(None),
        }
    }

    pub fn set_client(&self,
                      client: &'static test_signbus_init::SignbusInitialization)
                      -> ReturnCode {
        self.client.set(Some(client));
        ReturnCode::SUCCESS
    }

    pub fn signbus_app_send(&self,
                            address: u8,
                            frame_type: u8,
                            api_type: u8,
                            message_type: u8,
                            message_length: usize,
                            message: &'static mut [u8])
                            -> ReturnCode {

        let len: usize = 1 + 1 + 1 + message_length;

        // Concatenate info with message
        self.payload.map(|payload| {
            payload[0] = frame_type as u8;
            payload[1] = api_type as u8;
            payload[2] = message_type;

            let d = &mut payload.as_mut()[3..len as usize];
            for (i, c) in message[0..message_length as usize].iter().enumerate() {
                d[i] = *c;
            }
        });


        let rc = self.payload.take().map_or(ReturnCode::EBUSY, |payload| {
            self.protocol_layer.signbus_protocol_send(address, payload, len)
        });

        return rc;
    }

    pub fn signbus_app_recv(&self, buffer: &'static mut [u8]) -> ReturnCode {
        self.protocol_layer.signbus_protocol_recv(buffer)
    }
}

impl<'a> protocol_layer::ProtocolLayerClient for SignbusAppLayer<'a> {
    // Called when a new packet is received over I2C.
    fn packet_received(&self, data: &'static mut [u8], length: usize, error: support::Error) {
        self.client.get().map(move |client| { client.packet_received(data, length, error); });
    }

    // Called when an I2C master write command is complete.
    fn packet_sent(&self, data: &'static mut [u8], error: support::Error) {
        self.client.get().map(move |client| {
            self.payload.replace(data);
            self.send_buf.take().map(|send_buf| { client.packet_sent(send_buf, error); });
        });
    }

    // Called when an I2C slave read has completed.
    fn packet_read_from_slave(&self) {
        // TODO: implement slave write/ master read
        unimplemented!("Implement slave write/ master read.");
    }
}
