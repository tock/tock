use core::cell::Cell;
use kernel::hil::usb::*;
use kernel::utilities::cells::OptionalCell;
use super::configuration::*;
use super::descriptors::*;

pub struct BulkEcho<'a, C> {
    data_out: Endpoint,
    data_in: Endpoint,
    echo_buf: [Cell<u8>; 8], // Must be no larger than endpoint packet buffer
    echo_len: Cell<usize>,
    delayed_out: Cell<bool>,
    controller: &'a C,
}

impl<'a, C: UsbController<'a>> BulkEcho<'a, C> {
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
            echo_buf: Default::default(),
            echo_len: Cell::new(0),
            delayed_out: Cell::new(false),
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
            if self.delayed_out.take() {
                self.controller.endpoint_resume_out(endpoint_number as usize);
            }
        }
    }
}

impl<'a, C: UsbController<'a>> Interface for BulkEcho<'a, C> {
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
    fn packet_in(&self, transfer_type: TransferType, endpoint: usize) -> InResult {
        match transfer_type {
            TransferType::Interrupt => {
                InResult::Error
            }
            TransferType::Bulk => {
                // Write a packet into the endpoint buffer
                let packet_bytes = self.echo_len.get();
                if packet_bytes > 0 {
                    // Copy the entire echo buffer into the packet
                    let packet = self.data_out.buffer();
                    for i in 0..packet_bytes {
                        packet[i].set(self.echo_buf[i].get());
                    }
                    self.echo_len.set(0);

                    // We can receive more now
                    self.alert_empty();

                    InResult::Packet(packet_bytes)
                } else {
                    // Nothing to send
                    InResult::Delay
                }
            }
            TransferType::Control | TransferType::Isochronous => unreachable!(),
        }
    }

    /// Handle a Bulk/Interrupt OUT transaction
    fn packet_out(
        &self,
        transfer_type: TransferType,
        endpoint: usize,
        packet_bytes: u32,
    ) -> OutResult {
        match transfer_type {
            TransferType::Interrupt => {
                OutResult::Error
            }
            TransferType::Bulk => {
                // Consume a packet from the endpoint buffer
                let new_len = packet_bytes as usize;
                let current_len = self.echo_len.get();
                let total_len = current_len + new_len as usize;

                if total_len > self.echo_buf.len() {
                    // The packet won't fit in our little buffer.  We'll have
                    // to wait until it is drained
                    self.delayed_out.set(true);
                    OutResult::Delay
                } else if new_len > 0 {
                    // Copy the packet into our echo buffer
                    let packet = self.data_in.buffer();
                    for i in 0..new_len {
                        self.echo_buf[current_len + i].set(packet[i].get());
                    }
                    self.echo_len.set(total_len);

                    // We can start sending again
                    self.alert_full();
                    OutResult::Ok
                } else {
                    OutResult::Ok
                }
            }
            TransferType::Control | TransferType::Isochronous => unreachable!(),
        }
    }

    fn bus_reset(&self) {
        // Should the client initiate reconfiguration here?
        // For now, the hardware layer does it.

        // Reset the state for our pair of debugging endpoints
        self.echo_len.set(0);
        self.delayed_out.set(false);
    }

}

