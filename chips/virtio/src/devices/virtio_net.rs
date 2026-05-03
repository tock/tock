// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use core::cell::Cell;

use kernel::hil::ethernet::{EthernetAdapterDatapath, EthernetAdapterDatapathClient};
use kernel::platform::dma_fence::DmaFence;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::{SubSliceMut, SubSliceMutImmut};
use kernel::utilities::registers::{register_bitfields, LocalRegisterCopy};
use kernel::ErrorCode;

use super::super::devices::{VirtIODeviceDriver, VirtIODeviceType};
use super::super::queues::split_queue::{
    SplitVirtqueue, SplitVirtqueueClient, VirtqueueBuffer, VirtqueueReturnBuffer,
};

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

pub struct VirtIONet<'a, F: DmaFence> {
    rxqueue: &'a SplitVirtqueue<'static, 'static, 2, F>,
    txqueue: &'a SplitVirtqueue<'static, 'static, 2, F>,
    tx_header: OptionalCell<&'static mut [u8; 12]>,
    tx_frame_info: Cell<(u16, usize)>,
    rx_header: OptionalCell<&'static mut [u8]>,
    rx_buffer: OptionalCell<&'static mut [u8]>,
    client: OptionalCell<&'a dyn EthernetAdapterDatapathClient>,
    rx_enabled: Cell<bool>,
}

impl<'a, F: DmaFence> VirtIONet<'a, F> {
    pub fn new(
        txqueue: &'a SplitVirtqueue<'static, 'static, 2, F>,
        tx_header: &'static mut [u8; 12],
        rxqueue: &'a SplitVirtqueue<'static, 'static, 2, F>,
        rx_header: &'static mut [u8],
        rx_buffer: &'static mut [u8],
    ) -> VirtIONet<'a, F> {
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
            let rx_buffer_slice = SubSliceMut::new(rx_buffer);

            let mut rx_header_slice = SubSliceMut::new(self.rx_header.take().unwrap());
            rx_header_slice.slice(0..12);

            let mut buffer_chain = [
                Some(VirtqueueBuffer::DeviceWriteable(rx_header_slice)),
                Some(VirtqueueBuffer::DeviceWriteable(rx_buffer_slice)),
            ];

            self.rxqueue
                .provide_buffer_chain(&mut buffer_chain)
                .unwrap();
        }
    }
}

impl<F: DmaFence> SplitVirtqueueClient<'static> for VirtIONet<'_, F> {
    fn buffer_chain_ready(
        &self,
        queue_number: u32,
        buffer_chain: &mut [Option<VirtqueueReturnBuffer<'static>>],
        bytes_used: usize,
    ) {
        if queue_number == self.rxqueue.queue_number().unwrap() {
            // Received an Ethernet frame

            let rx_header = buffer_chain[0].take().expect("No header buffer");
            let VirtqueueBuffer::DeviceWriteable(rx_header_slice) = rx_header.virtqueue_buffer
            else {
                panic!("VirtQueue returned DeviceReadable buffer")
            };
            // TODO: do something with the header
            self.rx_header.replace(rx_header_slice.take());

            let VirtqueueBuffer::DeviceWriteable(rx_buffer_sub_slice) = buffer_chain[1]
                .take()
                .expect("No rx content buffer")
                .virtqueue_buffer
            else {
                panic!("VirtQueue returned DeviceReadable buffer")
            };
            let rx_buffer_slice = rx_buffer_sub_slice.take();

            if self.rx_enabled.get() {
                self.client.map(|client| {
                    client.received_frame(&rx_buffer_slice[..(bytes_used - 12)], None)
                });
            }

            self.rx_buffer.replace(rx_buffer_slice);

            // Re-run enable RX to provide the RX buffer chain back to the
            // device (if reception is still enabled):
            self.reinsert_virtqueue_receive_buffer();
        } else if queue_number == self.txqueue.queue_number().unwrap() {
            // Sent an Ethernet frame

            let tx_header = buffer_chain[0].take().expect("No header buffer");
            let VirtqueueBuffer::DeviceReadable(tx_header_sub_slice_mut_immut) =
                tx_header.virtqueue_buffer
            else {
                panic!("VirtQueue returned DeviceWriteable buffer")
            };
            let SubSliceMutImmut::Mutable(tx_header_sub_slice_mut) = tx_header_sub_slice_mut_immut
            else {
                panic!("tx_header SubSliceMutImmut is not mutable!")
            };
            self.tx_header.replace(
                tx_header_sub_slice_mut
                    .take()
                    .try_into()
                    .expect("tx_header slice was truncated"),
            );

            let tx_frame = buffer_chain[1].take().expect("No frame buffer");
            let VirtqueueBuffer::DeviceReadable(tx_frame_sub_slice_mut_immut) =
                tx_frame.virtqueue_buffer
            else {
                panic!("VirtQueue returned DeviceWriteable buffer")
            };
            let SubSliceMutImmut::Mutable(tx_frame_sub_slice_mut) = tx_frame_sub_slice_mut_immut
            else {
                panic!("tx_frame SubSliceMutImmut is not mutable!")
            };

            let (frame_len, transmission_identifier) = self.tx_frame_info.get();

            self.client.map(move |client| {
                client.transmit_frame_done(
                    Ok(()),
                    tx_frame_sub_slice_mut.take(),
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

impl<F: DmaFence> VirtIODeviceDriver for VirtIONet<'_, F> {
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

impl<'a, F: DmaFence> EthernetAdapterDatapath<'a> for VirtIONet<'a, F> {
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

    // TODO: Implement simultaneous transmissions
    fn transmit_frame(
        &self,
        frame_buffer: &'static mut [u8],
        len: u16,
        transmission_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Try to get a hold of the header buffer. If this fails, the
        // device is currently busy transmissing a buffer:
        let Some(header_buf) = self.tx_header.take() else {
            return Err((ErrorCode::BUSY, frame_buffer));
        };

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

        let mut tx_header_sub_slice_mut = SubSliceMut::new(header_buf);
        tx_header_sub_slice_mut.slice(0..12);

        let mut tx_frame_sub_slice_mut = SubSliceMut::new(frame_buffer);
        tx_frame_sub_slice_mut.slice(0..(len as usize));

        let mut buffer_chain = [
            Some(VirtqueueBuffer::DeviceReadable(SubSliceMutImmut::Mutable(
                tx_header_sub_slice_mut,
            ))),
            Some(VirtqueueBuffer::DeviceReadable(SubSliceMutImmut::Mutable(
                tx_frame_sub_slice_mut,
            ))),
        ];

        self.tx_frame_info.set((len, transmission_identifier));

        self.txqueue
            .provide_buffer_chain(&mut buffer_chain)
            .map_err(move |ret| {
                let VirtqueueBuffer::DeviceReadable(tx_frame_sub_slice_mut_immut) =
                    buffer_chain[1].take().unwrap()
                else {
                    panic!("VirtQueue returned DeviceWriteable buffer")
                };
                let SubSliceMutImmut::Mutable(tx_frame_sub_slice_mut) =
                    tx_frame_sub_slice_mut_immut
                else {
                    panic!("tx_frame SubSliceMutImmut is not mutable!")
                };
                (ret, tx_frame_sub_slice_mut.take())
            })?;

        Ok(())
    }
}
