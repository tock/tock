// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use core::cell::Cell;

use kernel::hil::ethernet::{EthernetAdapterDatapath, EthernetAdapterDatapathClient};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{register_bitfields, LocalRegisterCopy};
use kernel::ErrorCode;

use super::super::devices::{VirtIODeviceDriver, VirtIODeviceType};
use super::super::queues::split_queue::{SplitVirtqueue, SplitVirtqueueClient, VirtqueueBuffer};

register_bitfields![u64,
    VirtIONetFeatures [
        VirtIONetFCsum OFFSET(0) NUMBITS(1),
        VirtIONetFGuestCsum OFFSET(1) NUMBITS(1),
        VirtIONetFCtrlGuestOffloads OFFSET(2) NUMBITS(1),
        VirtIONetFMtu OFFSET(3) NUMBITS(1),
        VirtIONetFMac OFFSET(5) NUMBITS(1),
        VirtIONetFGuestTso4 OFFSET(7) NUMBITS(1),
        VirtIONetFGuestTso6 OFFSET(8) NUMBITS(1),
        VirtIONetFGuestEcn OFFSET(9) NUMBITS(1),
        VirtIONetFGuestUfo OFFSET(10) NUMBITS(1),
        VirtIONetFHostTso4 OFFSET(11) NUMBITS(1),
        VirtIONetFHostTso6 OFFSET(12) NUMBITS(1),
        VirtIONetFHostEcn OFFSET(13) NUMBITS(1),
        VirtIONetFHostUfo OFFSET(14) NUMBITS(1),
        VirtIONetFMrgRxbuf OFFSET(15) NUMBITS(1),
        VirtIONetFStatus OFFSET(16) NUMBITS(1),
        VirtIONetFCtrlVq OFFSET(17) NUMBITS(1),
        VirtIONetFCtrlRx OFFSET(18) NUMBITS(1),
        VirtIONetFCtrlVlan OFFSET(19) NUMBITS(1),
        VirtIONetFGuestAnnounce OFFSET(21) NUMBITS(1),
        VirtIONetFMq OFFSET(22) NUMBITS(1),
        VirtIONetFCtrlMacAddr OFFSET(23) NUMBITS(1),
        // these feature bits would not be passed through the driver, as
        // they are in a region reserved for future extensions?
        VirtIONetFRscExt OFFSET(61) NUMBITS(1),
        VirtIONetFStandby OFFSET(62) NUMBITS(1),
    ]
];

pub struct VirtIONet<'a> {
    rxqueue: &'a SplitVirtqueue<'static, 'static, 2>,
    txqueue: &'a SplitVirtqueue<'static, 'static, 2>,
    tx_header: OptionalCell<&'static mut [u8; 12]>,
    tx_frame_info: Cell<(u16, usize)>,
    rx_header: OptionalCell<&'static mut [u8]>,
    rx_buffer: OptionalCell<&'static mut [u8]>,
    client: OptionalCell<&'a dyn EthernetAdapterDatapathClient>,
    rx_enabled: Cell<bool>,
}

impl<'a> VirtIONet<'a> {
    pub fn new(
        txqueue: &'a SplitVirtqueue<'static, 'static, 2>,
        tx_header: &'static mut [u8; 12],
        rxqueue: &'a SplitVirtqueue<'static, 'static, 2>,
        rx_header: &'static mut [u8],
        rx_buffer: &'static mut [u8],
    ) -> VirtIONet<'a> {
        txqueue.enable_used_callbacks();
        rxqueue.enable_used_callbacks();

        VirtIONet {
            rxqueue,
            txqueue,
            tx_header: OptionalCell::new(tx_header),
            tx_frame_info: Cell::new((0, 0)),
            rx_header: OptionalCell::new(rx_header),
            rx_buffer: OptionalCell::new(rx_buffer),
            client: OptionalCell::empty(),
            rx_enabled: Cell::new(false),
        }
    }

    fn reinsert_virtqueue_receive_buffer(&self) {
        // Don't reinsert receive buffer when reception is disabled. The buffers
        // will be reinserted on the next call to `enable_receive`:
        if !self.rx_enabled.get() {
            return;
        }

        // Place the receive buffers into the device's VirtQueue
        if let Some(rx_buffer) = self.rx_buffer.take() {
            let rx_buffer_len = rx_buffer.len();

            let mut buffer_chain = [
                Some(VirtqueueBuffer {
                    buf: self.rx_header.take().unwrap(),
                    len: 12,
                    device_writeable: true,
                }),
                Some(VirtqueueBuffer {
                    buf: rx_buffer,
                    len: rx_buffer_len,
                    device_writeable: true,
                }),
            ];

            self.rxqueue
                .provide_buffer_chain(&mut buffer_chain)
                .unwrap();
        }
    }
}

impl SplitVirtqueueClient<'static> for VirtIONet<'_> {
    fn buffer_chain_ready(
        &self,
        queue_number: u32,
        buffer_chain: &mut [Option<VirtqueueBuffer<'static>>],
        bytes_used: usize,
    ) {
        if queue_number == self.rxqueue.queue_number().unwrap() {
            // Received an Ethernet frame

            let rx_header = buffer_chain[0].take().expect("No header buffer").buf;
            // TODO: do something with the header
            self.rx_header.replace(rx_header);

            let rx_buffer = buffer_chain[1].take().expect("No rx content buffer").buf;

            if self.rx_enabled.get() {
                self.client
                    .map(|client| client.received_frame(&rx_buffer[..(bytes_used - 12)], None));
            }

            self.rx_buffer.replace(rx_buffer);

            // Re-run enable RX to provide the RX buffer chain back to the
            // device (if reception is still enabled):
            self.reinsert_virtqueue_receive_buffer();
        } else if queue_number == self.txqueue.queue_number().unwrap() {
            // Sent an Ethernet frame

            let header_buf = buffer_chain[0].take().expect("No header buffer").buf;
            self.tx_header.replace(header_buf.try_into().unwrap());

            let frame_buf = buffer_chain[1].take().expect("No frame buffer").buf;

            let (frame_len, transmission_identifier) = self.tx_frame_info.get();

            self.client.map(move |client| {
                client.transmit_frame_done(
                    Ok(()),
                    frame_buf,
                    frame_len,
                    transmission_identifier,
                    None,
                )
            });
        } else {
            panic!("Callback from unknown queue");
        }
    }
}

impl VirtIODeviceDriver for VirtIONet<'_> {
    fn negotiate_features(&self, offered_features: u64) -> Option<u64> {
        let offered_features =
            LocalRegisterCopy::<u64, VirtIONetFeatures::Register>::new(offered_features);
        let mut negotiated_features = LocalRegisterCopy::<u64, VirtIONetFeatures::Register>::new(0);

        if offered_features.is_set(VirtIONetFeatures::VirtIONetFMac) {
            // VIRTIO_NET_F_MAC offered, which means that the device has a MAC
            // address. Accept this feature, which is required for this driver
            // for now.
            negotiated_features.modify(VirtIONetFeatures::VirtIONetFMac::SET);
        } else {
            return None;
        }

        // TODO: QEMU doesn't offer this, but don't we need it? Does QEMU
        // implicitly provide the feature but not offer it? Find out!
        // if offered_features & (1 << 15) != 0 {
        //     // VIRTIO_NET_F_MRG_RXBUF
        //     //
        //     // accept
        //     negotiated_features |= 1 << 15;
        // } else {
        //     panic!("Missing NET_F_MRG_RXBUF");
        // }

        // Ignore everything else
        Some(negotiated_features.get())
    }

    fn device_type(&self) -> VirtIODeviceType {
        VirtIODeviceType::NetworkCard
    }
}

impl<'a> EthernetAdapterDatapath<'a> for VirtIONet<'a> {
    fn set_client(&self, client: &'a dyn EthernetAdapterDatapathClient) {
        self.client.set(client);
    }

    fn enable_receive(&self) {
        // Enable receive callbacks:
        self.rx_enabled.set(true);

        // Attempt to reinsert any driver-owned receive buffers into the receive
        // queues. This will be a nop if reception was already enabled before
        // this call:
        self.reinsert_virtqueue_receive_buffer();
    }

    fn disable_receive(&self) {
        // Disable receive callbacks:
        self.rx_enabled.set(false);

        // We don't "steal" any receive buffers out of the virtqueue, but the
        // above flag will avoid reinserting buffers into the VirtQueue until
        // reception is enabled again:
    }

    fn transmit_frame(
        &self,
        frame_buffer: &'static mut [u8],
        len: u16,
        transmission_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Try to get a hold of the header buffer
        //
        // Otherwise, the device is currently busy transmissing a buffer
        //
        // TODO: Implement simultaneous transmissions
        let mut frame_queue_buf = Some(VirtqueueBuffer {
            buf: frame_buffer,
            len: len as usize,
            device_writeable: false,
        });

        let header_buf = self
            .tx_header
            .take()
            .ok_or(ErrorCode::BUSY)
            .map_err(|ret| (ret, frame_queue_buf.take().unwrap().buf))?;

        // Write the header
        //
        // TODO: Can this be done more elegantly using a struct of registers?
        header_buf[0] = 0; // flags -> we don't want checksumming
        header_buf[1] = 0; // gso -> no checksumming or fragmentation
        header_buf[2] = 0; // hdr_len_low
        header_buf[3] = 0; // hdr_len_high
        header_buf[4] = 0; // gso_size
        header_buf[5] = 0; // gso_size
        header_buf[6] = 0; // csum_start
        header_buf[7] = 0; // csum_start
        header_buf[8] = 0; // csum_offset
        header_buf[9] = 0; // csum_offsetb
        header_buf[10] = 0; // num_buffers
        header_buf[11] = 0; // num_buffers

        let mut buffer_chain = [
            Some(VirtqueueBuffer {
                buf: header_buf,
                len: 12,
                device_writeable: false,
            }),
            frame_queue_buf.take(),
        ];

        self.tx_frame_info.set((len, transmission_identifier));

        self.txqueue
            .provide_buffer_chain(&mut buffer_chain)
            .map_err(move |ret| (ret, buffer_chain[1].take().unwrap().buf))?;

        Ok(())
    }
}
