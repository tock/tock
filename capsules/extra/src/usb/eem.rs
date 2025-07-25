// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Ethernet Emulation Model (CDC) for USB
//!
//! This capsule allows Tock to support Ethernet over USB using the Ethernet
//! Emulation Model, a subclass of Communication Device Class (CDC).
//!
//! [EEM Specification](https://www.usb.org/document-library/cdc-subclass-specification-ethernet-emulation-model-devices-10)
//!
//! The EEM subclass is simply a CDC class descriptor with one in and one out
//! bulk endpoint.
//!
//! The capsule implements the (experimental) `EthernetDatapathAdapter` HIL
//! which allows reception and transmission of individual ethernet
//! frames. Generally, the capsule treats frames opaquely---it does not enforce
//! or validate the standard frame format or do any MAC filtering---and simply
//! adds the USB EEM header and (dummy) CRC footer. Similarly, on reception, it
//! simply strips the EEM header and CRC.

use core::cell::Cell;
use core::cmp;

use super::descriptors;
use super::descriptors::Buffer64;
use super::descriptors::CdcInterfaceDescriptor;
use super::descriptors::EndpointAddress;
use super::descriptors::EndpointDescriptor;
use super::descriptors::InterfaceDescriptor;
use super::descriptors::TransferDirection;
use super::usbc_client_ctrl::ClientCtrl;

use kernel::hil;
use kernel::hil::ethernet::EthernetAdapterDatapath;
use kernel::hil::ethernet::EthernetAdapterDatapathClient;
use kernel::hil::usb::TransferType;
use kernel::hil::usb::UsbController;
use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::cells::VolatileCell;

/// Dummy Ethernet CRC that means "don't check CRC" in the EEM standard
const DEADBEEF: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];

/// Identifying number for the endpoint when transferring data from us to the
/// host.
const ENDPOINT_IN_NUM: usize = 2;
/// Identifying number for the endpoint when transferring data from the host to
/// us.
const ENDPOINT_OUT_NUM: usize = 3;

static LANGUAGES: &'static [u16; 1] = &[
    0x0409, // English (United States)
];

const N_ENDPOINTS: usize = 2;

/// The receiver's current state
#[derive(Copy, Clone)]
enum RxState {
    /// The next transfer will begin with an EEM header.
    Idle,
    /// The next frame continues a frame from previous transfers
    Reading(
        /// Offset into the ethernet frame in bytes
        usize,
        /// Total frame length in bytes
        usize,
    ),
}

/// Implementation of the Abstract Control Model (ACM) for the Communications
/// Class Device (CDC) over USB.
pub struct CdcEem<'a, U: 'a> {
    /// Helper USB client library for handling many USB operations.
    client_ctrl: ClientCtrl<'a, 'static, U>,

    /// 64 byte buffers for each endpoint.
    buffers: [Buffer64; N_ENDPOINTS],

    /// A holder for the buffer to receive bytes into.
    rx_buffer: MapCell<[u8; 1522]>,
    /// The receiver's current state
    rx_state: Cell<RxState>,
    /// Whether received packets invoke the `EthernetAdapterDatapathClient`
    /// callbacks:
    rx_enabled: Cell<bool>,

    /// A holder for the buffer to transmit
    tx_buffer: TakeCell<'static, [u8]>,
    /// Offset into `tx_buffer` that has been transmitted
    tx_offset: Cell<u16>,
    /// Total length of `tx_buffer` to transmit
    tx_len: Cell<u16>,
    /// Client-specified identifier for the ethernet frame
    tx_identifier: Cell<usize>,

    client: OptionalCell<&'a dyn EthernetAdapterDatapathClient>,
}

pub mod subclass {
    pub const EEM: u8 = 0x0C;
}

pub mod protocol {
    pub const EEM: u8 = 0x07;
}

impl<'a, U: hil::usb::UsbController<'a>> CdcEem<'a, U> {
    pub fn new(
        controller: &'a U,
        max_ctrl_packet_size: u8,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
    ) -> Self {
        let interfaces: &mut [InterfaceDescriptor] = &mut [InterfaceDescriptor {
            interface_number: 0,
            interface_class: 0x02, // CDC communication
            interface_subclass: subclass::EEM,
            interface_protocol: protocol::EEM,
            ..InterfaceDescriptor::default()
        }];

        let cdc_descriptors: &mut [CdcInterfaceDescriptor] = &mut [CdcInterfaceDescriptor {
            subtype: descriptors::CdcInterfaceDescriptorSubType::Header,
            field1: 0x10, // CDC
            field2: 0x11, // CDC
        }];

        let endpoints: &[&[EndpointDescriptor]] = &[&[
            EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(2, TransferDirection::DeviceToHost),
                transfer_type: TransferType::Bulk,
                max_packet_size: 64,
                interval: 0,
            },
            EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(3, TransferDirection::HostToDevice),
                transfer_type: TransferType::Bulk,
                max_packet_size: 64,
                interval: 0,
            },
        ]];

        let (device_descriptor_buffer, other_descriptor_buffer) =
            descriptors::create_descriptor_buffers(
                descriptors::DeviceDescriptor {
                    vendor_id,
                    product_id,
                    manufacturer_string: 1,
                    product_string: 2,
                    serial_number_string: 3,
                    class: 0x2, // Class: CDC
                    max_packet_size_ep0: max_ctrl_packet_size,
                    ..descriptors::DeviceDescriptor::default()
                },
                descriptors::ConfigurationDescriptor {
                    ..descriptors::ConfigurationDescriptor::default()
                },
                interfaces,
                endpoints,
                None, // No HID descriptor
                Some(cdc_descriptors),
            );

        Self {
            client_ctrl: ClientCtrl::new(
                controller,
                device_descriptor_buffer,
                other_descriptor_buffer,
                None, // No HID descriptor
                None, // No report descriptor
                LANGUAGES,
                strings,
            ),
            buffers: Default::default(),
            rx_buffer: MapCell::new([0; 1522]),
            rx_state: Cell::new(RxState::Idle),
            rx_enabled: Cell::new(false),

            tx_buffer: TakeCell::empty(),
            tx_offset: Cell::new(0),
            tx_len: Cell::new(0),
            tx_identifier: Cell::new(0),

            client: OptionalCell::empty(),
        }
    }

    #[inline]
    fn controller(&self) -> &'a U {
        self.client_ctrl.controller()
    }

    #[inline]
    fn buffer(&'a self, i: usize) -> &'a [VolatileCell<u8>; 64] {
        &self.buffers[i - 2].buf
    }

    fn read_packet(&self, mut current_packet: &[VolatileCell<u8>]) {
        self.rx_buffer.map(|rx_buffer| {
            loop {
                match self.rx_state.get() {
                    RxState::Idle => {
                        // loop until we're done reading this whole USB transfer
                        if current_packet.len() < 2 {
                            // We may have processed exactly the
                            // whole transfer. If not, this should
                            // never happen with valid host driver,
                            // as EEM packets always start with a
                            // 2-byte header. So, just avoid
                            // panicking.
                            //
                            // TODO(alevy): is this for sure true?
                            // Or do we need to handle hearders
                            // split across packets?
                            break;
                        }
                        let header;
                        (header, current_packet) = current_packet.split_at(2);
                        let header0 = header[0].get();
                        let header1 = header[1].get();

                        if header1 & 0b10000000 == 0b1000000 {
                            let cmd = header1 & 0b00111000 >> 3;
                            kernel::debug!("EEM Command {}", cmd);
                        } else {
                            let _crc = header1 & 0b01000000;
                            let len = header0 as usize + (((header1 & 0b00111111) as usize) << 8);

                            let until = core::cmp::min(len, current_packet.len());
                            let eth_payload;
                            (eth_payload, current_packet) = current_packet.split_at(until);

                            for (rb, eth) in rx_buffer.iter_mut().zip(eth_payload.iter()) {
                                *rb = eth.get();
                            }

                            self.rx_state.set(RxState::Reading(eth_payload.len(), len));
                        }
                    }
                    RxState::Reading(cursor, len) => {
                        if cursor == len {
                            let len_without_mac =
                                core::cmp::min(len.saturating_sub(4), rx_buffer.len());
                            self.rx_state.set(RxState::Idle);
                            if self.rx_enabled.get() {
                                self.client.map(|client| {
                                    client.received_frame(&rx_buffer[..len_without_mac], None)
                                });
                            }
                        } else if current_packet.is_empty() {
                            break;
                        } else {
                            let until = core::cmp::min(len - cursor, current_packet.len());
                            let eth_payload;
                            (eth_payload, current_packet) = current_packet.split_at(until);
                            for (rb, eth) in
                                rx_buffer.iter_mut().skip(cursor).zip(eth_payload.iter())
                            {
                                *rb = eth.get();
                            }

                            self.rx_state
                                .set(RxState::Reading(cursor + eth_payload.len(), len));
                        }
                    }
                }
            }
        });
    }
}

impl<'a, U: hil::usb::UsbController<'a>> hil::usb::Client<'a> for CdcEem<'a, U> {
    fn enable(&'a self) {
        // Set up the default control endpoint
        self.client_ctrl.enable();

        // Setup buffers for IN and OUT data transfer.
        self.controller()
            .endpoint_set_in_buffer(ENDPOINT_IN_NUM, self.buffer(ENDPOINT_IN_NUM));
        self.controller()
            .endpoint_in_enable(TransferType::Bulk, ENDPOINT_IN_NUM);

        self.controller()
            .endpoint_set_out_buffer(ENDPOINT_OUT_NUM, self.buffer(ENDPOINT_OUT_NUM));
        self.controller()
            .endpoint_out_enable(TransferType::Bulk, ENDPOINT_OUT_NUM);
    }

    fn attach(&'a self) {
        self.client_ctrl.attach();
    }

    fn bus_reset(&'a self) {}

    /// Handle a Control Setup transaction.
    ///
    /// CDC uses special values here, and we can use these to know when a CDC
    /// client is connected or not.
    fn ctrl_setup(&'a self, endpoint: usize) -> hil::usb::CtrlSetupResult {
        self.client_ctrl.ctrl_setup(endpoint)
    }

    /// Handle a Control In transaction
    fn ctrl_in(&'a self, endpoint: usize) -> hil::usb::CtrlInResult {
        self.client_ctrl.ctrl_in(endpoint)
    }

    /// Handle a Control Out transaction
    fn ctrl_out(&'a self, endpoint: usize, packet_bytes: u32) -> hil::usb::CtrlOutResult {
        self.client_ctrl.ctrl_out(endpoint, packet_bytes)
    }

    fn ctrl_status(&'a self, endpoint: usize) {
        self.client_ctrl.ctrl_status(endpoint)
    }

    /// Handle the completion of a Control transfer
    fn ctrl_status_complete(&'a self, endpoint: usize) {
        self.client_ctrl.ctrl_status_complete(endpoint)
    }

    /// Handle a Bulk/Interrupt IN transaction.
    ///
    /// This is called when we can send data to the host. It should get called
    /// when we tell the controller we want to resume the IN endpoint (meaning
    /// we know we have data to send) and afterwards until we return
    /// `hil::usb::InResult::Delay` from this function. That means we can use
    /// this as a callback to mean that the transmission finished by waiting
    /// until this function is called when we don't have anything left to send.
    fn packet_in(&'a self, transfer_type: TransferType, endpoint: usize) -> hil::usb::InResult {
        match transfer_type {
            TransferType::Bulk => {
                self.tx_buffer
                    .take()
                    .map_or(hil::usb::InResult::Delay, |tx_buf| {
                        // Check if we have any bytes to send.
                        let offset = self.tx_offset.get();
                        let len = self.tx_len.get() + 4;
                        let remaining = len - offset;
                        if offset == 0 {
                            // Beginning of packet, send EEM header
                            let header0 = (len & 0xff) as u8;
                            let header1 = ((len >> 8) & 0b00111111) as u8;

                            // Get packet that we have shared with the underlying
                            // USB stack to copy the tx into.
                            let packet = self.buffer(endpoint);
                            packet[0].set(header0);
                            packet[1].set(header1);

                            // Calculate how much we can send
                            let to_send = cmp::min(packet.len() as u16, remaining + 2);

                            // Copy from the TX buffer to the outgoing USB packet.
                            for (p, b) in packet
                                .iter()
                                .take(to_send.into())
                                .skip(2)
                                .zip(tx_buf.iter())
                            {
                                p.set(*b);
                            }

                            for (p, b) in packet
                                .iter()
                                .skip(remaining as usize - 2)
                                .zip(DEADBEEF.iter())
                            {
                                p.set(*b);
                            }

                            // Update our state on how much more there is to send.
                            self.tx_offset.set(offset + to_send - 2);

                            // Put the TX buffer back so we can keep sending from it.
                            self.tx_buffer.replace(tx_buf);

                            // Return that we have data to send.
                            hil::usb::InResult::Packet(to_send.into())
                        } else if remaining > 0 {
                            // We do, so we go ahead and send those.

                            // Get packet that we have shared with the underlying
                            // USB stack to copy the tx into.
                            let packet = self.buffer(endpoint);

                            // Calculate how much we can send
                            let to_send = cmp::min(packet.len() as u16, remaining);

                            // Copy from the TX buffer to the outgoing USB packet.
                            for (p, b) in packet
                                .iter()
                                .zip(tx_buf.iter().chain(DEADBEEF[..].iter()).skip(offset.into()))
                            {
                                p.set(*b);
                            }

                            for (p, b) in packet
                                .iter()
                                .skip(remaining as usize - 4)
                                .zip(DEADBEEF.iter())
                            {
                                p.set(*b);
                            }

                            // Update our state on how much more there is to send.
                            self.tx_offset.set(offset + to_send);

                            // Put the TX buffer back so we can keep sending from it.
                            self.tx_buffer.replace(tx_buf);

                            // Return that we have data to send.
                            hil::usb::InResult::Packet(to_send.into())
                        } else {
                            // We don't have anything to send, so that means we are
                            // ok to signal the callback.

                            // Signal the callback and pass back the TX buffer.
                            self.client.map(move |client| {
                                client.transmit_frame_done(
                                    Ok(()),
                                    tx_buf,
                                    self.tx_len.get(),
                                    self.tx_identifier.get(),
                                    None,
                                );
                            });

                            // Return that we have nothing else to do to the USB
                            // driver.
                            hil::usb::InResult::Delay
                        }
                    })
            }
            TransferType::Control | TransferType::Isochronous | TransferType::Interrupt => {
                // Nothing to do for CDC ACM.
                hil::usb::InResult::Delay
            }
        }
    }

    /// Handle a Bulk/Interrupt OUT transaction
    fn packet_out(
        &'a self,
        transfer_type: TransferType,
        endpoint: usize,
        packet_bytes: u32,
    ) -> hil::usb::OutResult {
        match transfer_type {
            TransferType::Bulk => {
                let usb_packet = &self.buffer(endpoint)[..(packet_bytes as usize)];
                self.read_packet(usb_packet);
                // No error cases to report to the USB.
                hil::usb::OutResult::Ok
            }
            TransferType::Control | TransferType::Isochronous | TransferType::Interrupt => {
                // Nothing to do for CDC ACM.
                hil::usb::OutResult::Ok
            }
        }
    }

    fn packet_transmitted(&self, endpoint: usize) {
        // Check if more to send.
        self.tx_buffer.take().map(|tx_buf| {
            // Check if we have any bytes to send.
            let remaining = self.tx_len.get() + 4 - self.tx_offset.get();
            if remaining > 0 {
                // We do, so ask to send again.
                self.tx_buffer.replace(tx_buf);
                self.controller().endpoint_resume_in(endpoint);
            } else {
                // We don't have anything to send, so that means we are
                // ok to signal the callback.

                // Signal the callback and pass back the TX buffer.
                self.client.map(|client| {
                    client.transmit_frame_done(
                        Ok(()),
                        tx_buf,
                        self.tx_len.get(),
                        self.tx_identifier.get(),
                        None,
                    )
                });
            }
        });
    }
}

impl<'a, U: UsbController<'a>> EthernetAdapterDatapath<'a> for CdcEem<'a, U> {
    fn enable_receive(&self) {
        self.rx_enabled.set(true)
    }

    fn disable_receive(&self) {
        self.rx_enabled.set(false)
    }

    fn set_client(&self, client: &'a dyn EthernetAdapterDatapathClient) {
        self.client.set(client);
    }

    fn transmit_frame(
        &self,
        frame_buffer: &'static mut [u8],
        len: u16,
        transmission_identifier: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        if self.tx_buffer.is_some() {
            Err((kernel::ErrorCode::BUSY, frame_buffer))
        } else {
            self.tx_buffer.put(Some(frame_buffer));
            self.tx_len.set(len);
            self.tx_offset.set(0);
            self.tx_identifier.set(transmission_identifier);
            self.controller().endpoint_resume_in(ENDPOINT_IN_NUM);
            Ok(())
        }
    }
}
