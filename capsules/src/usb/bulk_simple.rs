use core::cell::Cell;
use core::cmp;
use kernel::hil::uart::{self, Configure, Transmit, TransmitClient, Receive, ReceiveClient, Parameters};
use kernel::hil::usb::*;
use kernel::ErrorCode;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use super::configuration::*;
use super::descriptors::*;

pub struct BulkSimple<'a, C> {
    data_out: Endpoint,
    data_in: Endpoint,
    controller: &'a C,

    /// A holder reference for the TX buffer we are transmitting from.
    tx_buffer: TakeCell<'static, [u8]>,
    /// The number of bytes the client has asked us to send. We track this so we
    /// can pass it back to the client when the transmission has finished.
    tx_len: Cell<usize>,
    /// Where in the `tx_buffer` we need to start sending from when we continue.
    tx_offset: Cell<usize>,
    /// The TX client to use when transmissions finish.
    tx_client: OptionalCell<&'a dyn TransmitClient>,

    /// A holder for the buffer to receive bytes into. We use this as a flag as
    /// well, if we have a buffer then we are actively doing a receive.
    rx_buffer: TakeCell<'static, [u8]>,
    /// How many bytes the client wants us to receive.
    rx_len: Cell<usize>,
    /// How many bytes we have received so far.
    rx_offset: Cell<usize>,
    /// The RX client to use when RX data is received.
    rx_client: OptionalCell<&'a dyn ReceiveClient>,
    reset: Cell<bool>,
}

impl<'a, C: UsbController<'a>> BulkSimple<'a, C> {
    pub fn new(controller: &'a C) -> Self {
        Self {
            controller,
            data_out: Endpoint {
                direction: TransferDirection::DeviceToHost,
                transfer_type: TransferType::Bulk,
                interval: 0,
                buffer: Some(super::descriptors::Buffer64::empty()),
                endpoint_number: OptionalCell::empty(),
            },
            data_in: Endpoint {
                direction: TransferDirection::HostToDevice,
                transfer_type: TransferType::Bulk,
                interval: 0,
                buffer: Some(super::descriptors::Buffer64::empty()),
                endpoint_number: OptionalCell::empty(),
            },
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_offset: Cell::new(0),
            tx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_offset: Cell::new(0),
            rx_client: OptionalCell::empty(),

            reset: Cell::new(false),
        }
    }

    fn alert_full(&self) {
        // Alert the controller that we now have data to send on the Bulk IN endpoint 1
        if let Some(endpoint_number) = self.data_out.endpoint_number.extract() {
            self.controller.endpoint_resume_in(endpoint_number as usize);
        }
    }

    fn alert_empty(&self) {
        if let Some(endpoint_number) = self.data_in.endpoint_number.extract() {
            self.controller.endpoint_resume_out(endpoint_number as usize);
        }
    }
}

impl<'a, C: UsbController<'a>> Interface for BulkSimple<'a, C> {
    fn details(&self) -> InterfaceDetails {
        InterfaceDetails {
            interface_class: 0xff,    // vendor specific
            interface_subclass: 0xab, // none
            interface_protocol: 0x00, // none
            alternate_setting: 0,
            string_index: 0,
            num_endpoints: 2,
        }
    }

    fn class_descriptor(&self, _i: usize) -> Option<&dyn Descriptor> {
        None
    }

    fn endpoint(&self, i: usize) -> Option<&Endpoint> {
        match i {
            0 => Some(&self.data_out),
            1 => Some(&self.data_in),
            _ => None,
        }
    }

    /// Handle a Bulk/Interrupt IN transaction
    fn packet_in(&self, transfer_type: TransferType, _endpoint: usize) -> InResult {
        match transfer_type {
            TransferType::Bulk => {
                self.tx_buffer
                    .take()
                    .map_or(InResult::Delay, |tx_buf| {
                        // Check if we have any bytes to send.
                        let offset = self.tx_offset.get();
                        let remaining = self.tx_len.get() - offset;
                        if remaining > 0 {
                            // We do, so we go ahead and send those.

                            // Get packet that we have shared with the underlying
                            // USB stack to copy the tx into.
                            let packet = self.data_out.buffer();

                            // Calculate how much more we can send.
                            let to_send = cmp::min(packet.len(), remaining);

                            // Copy from the TX buffer to the outgoing USB packet.
                            for i in 0..to_send {
                                packet[i].set(tx_buf[offset + i]);
                            }

                            // Update our state on how much more there is to send.
                            self.tx_offset.set(offset + to_send);

                            // Put the TX buffer back so we can keep sending from it.
                            self.tx_buffer.replace(tx_buf);

                            // Return that we have data to send.
                            InResult::Packet(to_send)
                        } else {
                            // We don't have anything to send, so that means we are
                            // ok to signal the callback.

                            // Signal the callback and pass back the TX buffer.
                            self.tx_client.map(move |tx_client| {
                                tx_client.transmitted_buffer(tx_buf, self.tx_len.get(), Ok(()))
                            });

                            // Return that we have nothing else to do to the USB
                            // driver.
                            InResult::Delay
                        }
                    })
            }
            TransferType::Control | TransferType::Isochronous | TransferType::Interrupt => {
                // Nothing to do for CDC ACM.
                InResult::Delay
            }
        }
    }

    fn packet_transmitted(&self, _endpoint: usize) {
        // Check if more to send.
        self.tx_buffer.take().map(|tx_buf| {
            // Check if we have any bytes to send.
            let remaining = self.tx_len.get() - self.tx_offset.get();
            if remaining > 0 {
                // We do, so ask to send again.
                self.tx_buffer.replace(tx_buf);
                self.alert_full();
            } else {
                // We don't have anything to send, so that means we are
                // ok to signal the callback.

                // Signal the callback and pass back the TX buffer.
                self.tx_client.map(move |tx_client| {
                    tx_client.transmitted_buffer(tx_buf, self.tx_len.get(), Ok(()))
                });
            }
        });
    }

    /// Handle a Bulk/Interrupt OUT transaction
    fn packet_out(
        &self,
        transfer_type: TransferType,
        _endpoint: usize,
        packet_bytes: u32,
    ) -> OutResult {
        match transfer_type {
            TransferType::Bulk => {
                // Start by checking to see if we even care about this RX or
                // not.
                self.rx_buffer.take().map_or(OutResult::Delay, |rx_buf| {
                    let rx_offset = self.rx_offset.get();

                    // How many more bytes can we store in our RX buffer?
                    let available_bytes = rx_buf.len() - rx_offset;
                    let copy_length = cmp::min(packet_bytes as usize, available_bytes);

                    // Do the copy into the RX buffer.
                    let packet = self.data_in.buffer();
                    for i in 0..copy_length {
                        rx_buf[rx_offset + i] = packet[i].get();
                    }

                    // Keep track of how many bytes we have received so far.
                    let total_received_bytes = rx_offset + copy_length;

                    // Update how many bytes we have gotten.
                    self.rx_offset.set(total_received_bytes);

                    // Check if we have received at least as many bytes as the
                    // client asked for.
                    if total_received_bytes >= self.rx_len.get() {
                        self.rx_client.map(move |client| {
                            client.received_buffer(
                                rx_buf,
                                total_received_bytes,
                                Ok(()),
                                uart::Error::None,
                            );
                        });
                    } else {
                        // Make sure to put the RX buffer back.
                        self.rx_buffer.replace(rx_buf);
                    }
                    OutResult::Ok
                })

                // No error cases to report to the USB.
            }
            TransferType::Control | TransferType::Isochronous | TransferType::Interrupt => {
                // Nothing to do for CDC ACM.
                OutResult::Ok
            }
        }
    }

    fn bus_reset(&self) {
        self.reset.set(true);
        if self.tx_buffer.is_some() {
            self.alert_full();
        }
    }

}

impl<'a, C: UsbController<'a>> Transmit<'a> for BulkSimple<'a, C> {
    fn set_transmit_client(&self, client: &'a (dyn TransmitClient + 'a)) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(&self, tx_buffer: &'static mut [u8], tx_len: usize) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        if self.tx_buffer.is_some() {
            // We are already handling a transmission, we cannot queue another
            // request.
            Err((ErrorCode::BUSY, tx_buffer))
        } else if tx_len > tx_buffer.len() {
            // Can't send more bytes than will fit in the buffer.
            Err((ErrorCode::SIZE, tx_buffer))
        } else {
            // Ok, we can handle this transmission. Initialize all of our state
            // for our TX state machine.
            self.tx_len.set(tx_len);
            self.tx_offset.set(0);
            self.tx_buffer.replace(tx_buffer);

            // Then signal to the lower layer that we are ready to do a TX
            // by putting data in the IN endpoint.
            if self.reset.get() {
                self.alert_full();
            }
            Ok(())
        }
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a, C: UsbController<'a>> Receive<'a> for BulkSimple<'a, C> {
    fn set_receive_client(&self, client: &'a (dyn ReceiveClient + 'a)) {
        self.rx_client.set(client);
    }
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
            self.rx_offset.set(0);
            self.rx_len.set(rx_len);

            Ok(())
        }
    }
    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
    fn receive_abort(&self) -> Result<(), kernel::ErrorCode> { Ok(()) }
}

impl<'a, C: UsbController<'a>> Configure for BulkSimple<'a, C> {
    fn configure(&self, _parameters: Parameters) -> Result<(), ErrorCode> {
        // Since this is not a real UART, we don't need to consider these
        // parameters.
        Ok(())
    }
}
