// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! VirtIO Split Virtqueue implementation.
//!
//! This module contains an implementation of a Split Virtqueue, as defined in
//! 2.6 Split Virtqueues of the [Virtual I/O Device (VIRTIO) Specification,
//! Version
//! 1.1](https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html).
//! This implementation can be used in conjunction with the VirtIO transports
//! defined in [`transports`](`super::super::transports`) and
//! [`devices`](`super::super::devices`) to interact with VirtIO-compatible
//! devices.

use core::cell::Cell;
use core::cmp;

use kernel::platform::dma_fence::DmaFence;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::dma_slice::{DmaSubSliceMut, DmaSubSliceMutImmut};
use kernel::utilities::leasable_buffer::{SubSliceMut, SubSliceMutImmut};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, InMemoryRegister};
use kernel::ErrorCode;

use super::super::queues::{Virtqueue, VirtqueueAddresses};
use super::super::transports::VirtIOTransport;

pub const DESCRIPTOR_ALIGNMENT: usize = 16;
pub const AVAILABLE_RING_ALIGNMENT: usize = 2;
pub const USED_RING_ALIGNMENT: usize = 4;

register_bitfields![u16,
    DescriptorFlags [
        Next OFFSET(0) NUMBITS(1) [],
        WriteOnly OFFSET(1) NUMBITS(1) [],
        Indirect OFFSET(2) NUMBITS(1) []
    ],
    AvailableRingFlags [
        NoInterrupt OFFSET(0) NUMBITS(1) []
    ],
    UsedRingFlags [
        NoNotify OFFSET(0) NUMBITS(1) []
    ],
];

/// A single Virtqueue descriptor.
///
/// Implements the memory layout of a single Virtqueue descriptor of a
/// split-virtqueue, to be placed into the queue's descriptor table, as defined
/// in section 2.6.5 of the spec.
#[repr(C)]
pub struct VirtqueueDescriptor {
    /// Guest physical address of the buffer to share
    addr: InMemoryRegister<u64>,
    /// Length of the shared buffer
    len: InMemoryRegister<u32>,
    /// Descriptor flags
    flags: InMemoryRegister<u16, DescriptorFlags::Register>,
    /// Pointer to the next entry in the descriptor queue (if two
    /// buffers are chained)
    next: InMemoryRegister<u16>,
}

impl Default for VirtqueueDescriptor {
    fn default() -> VirtqueueDescriptor {
        VirtqueueDescriptor {
            addr: InMemoryRegister::new(0),
            len: InMemoryRegister::new(0),
            flags: InMemoryRegister::new(0),
            next: InMemoryRegister::new(0),
        }
    }
}

/// The Virtqueue descriptor table.
///
/// This table is provided to the VirtIO device (host) as a means to communicate
/// information about shared buffers, maintained in the individual
/// [`VirtqueueDescriptor`] elements. Elements in this table are referenced by
/// the [`VirtqueueAvailableRing`] and [`VirtqueueUsedRing`] for exposing them
/// to the VirtIO device in order, and receiving exposed ("used") buffers back
/// from the device.
///
/// Multiple entries of the descriptor table can be chained in order to treat
/// disjoint regions of memory as a single buffer through the
/// `VirtqueueDescriptor::next` field, where the value of this field indexes
/// into this table.
#[repr(C, align(16))]
pub struct VirtqueueDescriptors<const MAX_QUEUE_SIZE: usize>([VirtqueueDescriptor; MAX_QUEUE_SIZE]);

impl<const MAX_QUEUE_SIZE: usize> Default for VirtqueueDescriptors<MAX_QUEUE_SIZE> {
    fn default() -> Self {
        VirtqueueDescriptors(core::array::from_fn(|_| VirtqueueDescriptor::default()))
    }
}

// This is required to be able to implement Default and hence to
// initialize an entire array of default values with size specified by
// a constant.
#[repr(transparent)]
pub struct VirtqueueAvailableElement(InMemoryRegister<u16>);

/// The Virtqueue available ring.
///
/// This struct is exposed to the VirtIO device as a means to share buffers
/// (pointed to by descriptors of the [`VirtqueueDescriptors`] descriptors
/// table) with the VirtIO device (host). It avoids the need for explicit
/// locking by using two distinct rings, each undirectionally exchanging
/// information about used buffers. When a new buffer is placed into the
/// available ring, the VirtIO driver (guest) must increment `idx` to the index
/// where it would place the next available descriptor pointer in the ring
/// field. After such an update, the queue must inform the device about this
/// change through a call to [`VirtIOTransport::queue_notify`]. Given that
/// volatile writes cannot be reordered with respect to each other, changes to
/// the available ring are guaranteed to be visible to the VirtIO device (host).
#[repr(C, align(2))]
pub struct VirtqueueAvailableRing<const MAX_QUEUE_SIZE: usize> {
    /// Virtqueue available ring flags.
    flags: InMemoryRegister<u16, AvailableRingFlags::Register>,
    /// Incrementing index, pointing to where the driver would put the next
    /// descriptor entry in the ring (modulo the queue size).
    ///
    /// The driver must not decrement this field. There is no way to "unexpose"
    /// buffers.
    idx: InMemoryRegister<u16>,
    /// Ring containing the shared buffers (indices into the
    /// [`VirtqueueDescriptors`] descriptor table).
    ring: [VirtqueueAvailableElement; MAX_QUEUE_SIZE],
    /// "Used event" queue notification suppression mechanism.
    ///
    /// This field is only honored by the VirtIO device if the EventIdx feature
    /// was negotiated.
    ///
    /// The driver can set this field to a target `idx` value of the
    /// [`VirtqueueUsedRing`] to indicate to the device that notifications are
    /// unnecessary until the device writes a buffer with the corresponding
    /// index into the used ring.
    used_event: InMemoryRegister<u16>,
}

impl Default for VirtqueueAvailableElement {
    fn default() -> VirtqueueAvailableElement {
        VirtqueueAvailableElement(InMemoryRegister::new(0))
    }
}

impl<const MAX_QUEUE_SIZE: usize> Default for VirtqueueAvailableRing<MAX_QUEUE_SIZE> {
    fn default() -> Self {
        VirtqueueAvailableRing {
            flags: InMemoryRegister::new(0),
            idx: InMemoryRegister::new(0),
            ring: core::array::from_fn(|_| VirtqueueAvailableElement::default()),
            used_event: InMemoryRegister::new(0),
        }
    }
}

/// The Virtqueue used ring.
///
/// This struct is exposed to the VirtIO device for the device to indicate which
/// shared buffers (through the [`VirtqueueAvailableRing`] have been processed.
/// It works similar to the available ring, but must never be written by the
/// VirtIO driver (guest) after it has been shared with the device, and as long
/// as the device is initialized.
#[repr(C, align(4))]
pub struct VirtqueueUsedRing<const MAX_QUEUE_SIZE: usize> {
    /// Virtqueue used ring flags.
    flags: InMemoryRegister<u16, UsedRingFlags::Register>,
    /// Incrementing index, pointing to where the device would put the next
    /// descriptor entry in the ring (modulo the queue size).
    ///
    /// The device must not decrement this field. There is no way to "take back"
    /// buffers.
    idx: InMemoryRegister<u16>,
    /// Ring containing the used buffers (indices into the
    /// [`VirtqueueDescriptors`] descriptor table).
    ring: [VirtqueueUsedElement; MAX_QUEUE_SIZE],
    /// "Available event" queue notification suppression mechanism.
    ///
    /// This field must only be honored by the VirtIO driver if the EventIdx
    /// feature was negotiated.
    ///
    /// The device can set this field to a target `idx` value of the
    /// [`VirtqueueAvailableRing`] to indicate to the driver that notifications
    /// are unnecessary until the driver writes a buffer with the corresponding
    /// index into the available ring.
    avail_event: InMemoryRegister<u16>,
}

impl<const MAX_QUEUE_SIZE: usize> Default for VirtqueueUsedRing<MAX_QUEUE_SIZE> {
    fn default() -> Self {
        VirtqueueUsedRing {
            flags: InMemoryRegister::new(0),
            idx: InMemoryRegister::new(0),
            ring: core::array::from_fn(|_| VirtqueueUsedElement::default()),
            avail_event: InMemoryRegister::new(0),
        }
    }
}

/// A single element of the [`VirtqueueUsedRing`].
#[repr(C)]
pub struct VirtqueueUsedElement {
    /// Index into the [`VirtqueueDescriptors`] descriptor table indicating the
    /// head element of the returned descriptor chain.
    id: InMemoryRegister<u32>,
    /// Total length of the descriptor chain which was used by the device.
    ///
    /// Commonly this is used as a mechanism to communicate how much data the
    /// device has written to a shared buffer.
    len: InMemoryRegister<u32>,
}

impl Default for VirtqueueUsedElement {
    fn default() -> VirtqueueUsedElement {
        VirtqueueUsedElement {
            id: InMemoryRegister::new(0),
            len: InMemoryRegister::new(0),
        }
    }
}

/// A helper struct to manage the state of the Virtqueue available ring.
///
/// This struct reduces the complexity of the [`SplitVirtqueue`] implementation
/// by encapsulating operations which depend on and modify the state of the
/// driver-controlled available ring of the Virtqueue. It is essentially a
/// glorified ring-buffer state machine, following the semantics as defined by
/// VirtIO for the Virtqueue's available ring.
struct AvailableRingHelper {
    max_elements: Cell<usize>,
    start: Cell<u16>,
    end: Cell<u16>,
    empty: Cell<bool>,
}

impl AvailableRingHelper {
    pub fn new(max_elements: usize) -> AvailableRingHelper {
        AvailableRingHelper {
            max_elements: Cell::new(max_elements),
            start: Cell::new(0),
            end: Cell::new(0),
            empty: Cell::new(true),
        }
    }

    fn ring_wrapping_add(&self, a: u16, b: u16) -> u16 {
        if self.max_elements.get() - (a as usize) - 1 < (b as usize) {
            b - (self.max_elements.get() - a as usize) as u16
        } else {
            a + b
        }
    }

    /// Reset the state of the available ring.
    ///
    /// This must be called before signaling to the device that the driver is
    /// initialized. It takes the maximum queue elements as the `max_elements`
    /// parameter, as negotiated with the device.
    pub fn reset(&self, max_elements: usize) {
        self.max_elements.set(max_elements);
        self.start.set(0);
        self.end.set(0);
        self.empty.set(true);
    }

    /// Whether the available ring of the Virtqueue is empty.
    pub fn is_empty(&self) -> bool {
        self.empty.get()
    }

    /// Whether the available ring of the Virtqueue is full.
    pub fn is_full(&self) -> bool {
        !self.empty.get() && self.start.get() == self.end.get()
    }

    /// Try to insert an element into the Virtqueue available ring.
    ///
    /// If there is space in the Virtqueue's available ring, this increments the
    /// internal state and returns the index of the element to be
    /// written. Otherwise, it returns `None`.
    pub fn insert(&self) -> Option<u16> {
        if !self.is_full() {
            let pos = self.end.get();
            self.end.set(self.ring_wrapping_add(pos, 1));
            self.empty.set(false);
            Some(pos)
        } else {
            None
        }
    }

    /// Try to remove an element from the Virtqueue available ring.
    ///
    /// If there is an element in the Virtqueue's available ring, this removes
    /// it from its internal state and returns the index of that element.
    pub fn pop(&self) -> Option<u16> {
        if !self.is_empty() {
            let pos = self.start.get();
            self.start.set(self.ring_wrapping_add(pos, 1));
            if self.start.get() == self.end.get() {
                self.empty.set(true);
            }
            Some(pos)
        } else {
            None
        }
    }
}

/// A slice of memory to be shared with a VirtIO device, either as
/// device-readable or device-writeable.
///
/// We can use either mutable or immutable Rust slices to expose device-readable
/// buffers. The VirtIO Specification Version 1.3 states that, for Split
/// Virtqueues:
///
///    A device MUST NOT write to a device-readable buffer, and a device SHOULD
///    NOT read a device-writable buffer (it MAY do so for debugging or
///    diagnostic purposes).
#[derive(Debug)]
pub enum VirtqueueBuffer<'b> {
    DeviceReadable(SubSliceMutImmut<'b, u8>),
    DeviceWriteable(SubSliceMut<'b, u8>),
}

/// A [`VirtqueueBuffer`] as returned by the device.
///
/// In addition to the same [`VirtqueueBuffer`] that was original passed to the
/// device, it contains a `device_len` field that indicates how many bytes the
/// device has read from or written to the buffer.
#[derive(Debug)]
pub struct VirtqueueReturnBuffer<'b> {
    pub virtqueue_buffer: VirtqueueBuffer<'b>,
    pub device_len: usize,
}

/// Internal, DMA-safe version of the [`VirtqueueBuffer`].
///
/// This has identical semantics to the [`VirtqueueBuffer`] enum, but does not
/// hold onto Rust slices during a DMA operation and uses DMA fences to ensure
/// that Rust writes are correctly exposed to DMA operations, and DMA writes are
/// visible to Rust reads.
#[derive(Debug)]
enum VirtqueueDmaBuffer<'b> {
    DeviceReadable(DmaSubSliceMutImmut<'b, u8>),
    DeviceWriteable(DmaSubSliceMut<'b, u8>),
}

impl<'b> VirtqueueDmaBuffer<'b> {
    unsafe fn from_virtqueue_buffer(
        virtqueue_buffer: VirtqueueBuffer<'b>,
        fence: impl DmaFence,
    ) -> Self {
        match virtqueue_buffer {
            VirtqueueBuffer::DeviceReadable(sub_slice_mut_immut) => {
                VirtqueueDmaBuffer::DeviceReadable(DmaSubSliceMutImmut::from_sub_slice_mut_immut(
                    sub_slice_mut_immut,
                    fence,
                ))
            }
            VirtqueueBuffer::DeviceWriteable(sub_slice_mut) => VirtqueueDmaBuffer::DeviceWriteable(
                DmaSubSliceMut::from_sub_slice_mut(sub_slice_mut, fence),
            ),
        }
    }

    unsafe fn into_virtqueue_buffer(self, fence: impl DmaFence) -> VirtqueueBuffer<'b> {
        match self {
            VirtqueueDmaBuffer::DeviceReadable(dma_sub_slice_mut_immut) => {
                VirtqueueBuffer::DeviceReadable(
                    dma_sub_slice_mut_immut.restore_sub_slice_mut_immut(),
                )
            }
            VirtqueueDmaBuffer::DeviceWriteable(dma_sub_slice_mut) => {
                VirtqueueBuffer::DeviceWriteable(unsafe {
                    dma_sub_slice_mut.restore_sub_slice_mut(fence)
                })
            }
        }
    }

    fn as_ptr(&self) -> *const u8 {
        match self {
            VirtqueueDmaBuffer::DeviceReadable(dma_sub_slice_mut_immut) => {
                dma_sub_slice_mut_immut.as_ptr()
            }
            VirtqueueDmaBuffer::DeviceWriteable(dma_sub_slice_mut) => {
                dma_sub_slice_mut.as_mut_ptr() as *const u8
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            VirtqueueDmaBuffer::DeviceReadable(dma_sub_slice_mut_immut) => {
                dma_sub_slice_mut_immut.len()
            }
            VirtqueueDmaBuffer::DeviceWriteable(dma_sub_slice_mut) => dma_sub_slice_mut.len(),
        }
    }

    fn device_writeable(&self) -> bool {
        match self {
            VirtqueueDmaBuffer::DeviceReadable(_) => false,
            VirtqueueDmaBuffer::DeviceWriteable(_) => true,
        }
    }
}

/// A VirtIO split Virtqueue.
///
/// For documentation on Virtqueues in general, please see the [`Virtqueue`
/// trait documentation](Virtqueue).
///
/// A split Virtqueue is split into separate memory areas, namely:
///
/// - a **descriptor table** (VirtIO driver / guest writeable,
///   [`VirtqueueDescriptors`])
///
/// - an **available ring** (VirtIO driver / guest writeable,
///   [`VirtqueueAvailableRing`])
///
/// - a **used ring** (VirtIO device / host writeable, [`VirtqueueUsedRing`])
///
/// Each of these areas must be located physically-contiguous in guest-memory
/// and have different alignment constraints.
///
/// This is in constrast to _packed Virtqueues_, which use memory regions that
/// are read and written by both the VirtIO device (host) and VirtIO driver
/// (guest).
pub struct SplitVirtqueue<'a, 'b, const MAX_QUEUE_SIZE: usize, F: DmaFence> {
    fence: F,

    descriptors: &'a mut VirtqueueDescriptors<MAX_QUEUE_SIZE>,
    available_ring: &'a mut VirtqueueAvailableRing<MAX_QUEUE_SIZE>,
    used_ring: &'a mut VirtqueueUsedRing<MAX_QUEUE_SIZE>,

    available_ring_state: AvailableRingHelper,
    last_used_idx: Cell<u16>,

    transport: OptionalCell<&'a dyn VirtIOTransport>,

    initialized: Cell<bool>,
    queue_number: Cell<u32>,
    max_elements: Cell<usize>,

    descriptor_buffers: [OptionalCell<VirtqueueDmaBuffer<'b>>; MAX_QUEUE_SIZE],

    client: OptionalCell<&'a dyn SplitVirtqueueClient<'b>>,
    used_callbacks_enabled: Cell<bool>,
}

impl<'a, 'b, const MAX_QUEUE_SIZE: usize, F: DmaFence> SplitVirtqueue<'a, 'b, MAX_QUEUE_SIZE, F> {
    pub fn new(
        descriptors: &'a mut VirtqueueDescriptors<MAX_QUEUE_SIZE>,
        available_ring: &'a mut VirtqueueAvailableRing<MAX_QUEUE_SIZE>,
        used_ring: &'a mut VirtqueueUsedRing<MAX_QUEUE_SIZE>,
        fence: F,
    ) -> Self {
        assert!((core::ptr::from_ref(descriptors) as usize).is_multiple_of(DESCRIPTOR_ALIGNMENT));
        assert!(
            (core::ptr::from_ref(available_ring) as usize).is_multiple_of(AVAILABLE_RING_ALIGNMENT)
        );
        assert!((core::ptr::from_ref(used_ring) as usize).is_multiple_of(USED_RING_ALIGNMENT));

        SplitVirtqueue {
            fence,

            descriptors,
            available_ring,
            used_ring,

            available_ring_state: AvailableRingHelper::new(MAX_QUEUE_SIZE),
            last_used_idx: Cell::new(0),

            transport: OptionalCell::empty(),

            initialized: Cell::new(false),
            queue_number: Cell::new(0),
            max_elements: Cell::new(MAX_QUEUE_SIZE),

            descriptor_buffers: core::array::from_fn(|_| OptionalCell::empty()),

            client: OptionalCell::empty(),
            used_callbacks_enabled: Cell::new(false),
        }
    }

    /// Set the [`SplitVirtqueueClient`].
    pub fn set_client(&self, client: &'a dyn SplitVirtqueueClient<'b>) {
        self.client.set(client);
    }

    /// Set the underlying [`VirtIOTransport`]. This must be done prior to
    /// initialization.
    pub fn set_transport(&self, transport: &'a dyn VirtIOTransport) {
        assert!(!self.initialized.get());
        self.transport.set(transport);
    }

    /// Get the queue number associated with this Virtqueue.
    ///
    /// Prior to initialization the SplitVirtqueue does not have an associated
    /// queue number and will return `None`.
    pub fn queue_number(&self) -> Option<u32> {
        if self.initialized.get() {
            Some(self.queue_number.get())
        } else {
            None
        }
    }

    /// Get the number of free descriptor slots in the descriptor table.
    ///
    /// This takes into account the negotiated maximum queue length.
    pub fn free_descriptor_count(&self) -> usize {
        assert!(self.initialized.get());
        self.descriptor_buffers
            .iter()
            .take(self.max_elements.get())
            .fold(0, |count, descbuf_entry| {
                if descbuf_entry.is_none() {
                    count + 1
                } else {
                    count
                }
            })
    }

    /// Get the number of (unprocessed) descriptor chains in the Virtqueue's
    /// used ring.
    pub fn used_descriptor_chains_count(&self) -> usize {
        let pending_chains = self
            .used_ring
            .idx
            .get()
            .wrapping_sub(self.last_used_idx.get());

        // If we ever have more than max_elements pending descriptors,
        // the used ring increased too fast and has overwritten data
        assert!(pending_chains as usize <= self.max_elements.get());

        pending_chains as usize
    }

    /// Remove an element from the Virtqueue's used ring.
    ///
    /// If `self.last_used_idx.get() == self.used_ring.idx.get()` (e.g. we don't
    /// have an unprocessed used buffer chain) this will return
    /// `None`. Otherwise it will return the remove ring element's index, as
    /// well as the number of processed bytes as reported by the VirtIO device.
    ///
    /// This will update `self.last_used_idx`.
    ///
    /// The caller is responsible for keeping the available ring in sync,
    /// freeing one entry if a used buffer was removed through this method.
    fn remove_used_chain(&self) -> Option<(usize, usize)> {
        assert!(self.initialized.get());

        let pending_chains = self.used_descriptor_chains_count();

        if pending_chains > 0 {
            let last_used_idx = self.last_used_idx.get();

            // Remove the element one below the index (as 0 indicates
            // _no_ buffer has been written), hence the index points
            // to the next element to be written
            let ring_pos = (last_used_idx as usize) % self.max_elements.get();
            let chain_top_idx = self.used_ring.ring[ring_pos].id.get();
            let written_len = self.used_ring.ring[ring_pos].len.get();

            // Increment our local idx counter
            self.last_used_idx.set(last_used_idx.wrapping_add(1));

            Some((chain_top_idx as usize, written_len as usize))
        } else {
            None
        }
    }

    /// Add an element to the available queue.
    ///
    /// Returns either the inserted ring index or `None` if the Virtqueue's
    /// available ring is fully occupied.
    ///
    /// This will update the available ring's `idx` field.
    ///
    /// The caller is responsible for notifying the device about any inserted
    /// available buffers.
    fn add_available_descriptor(&self, descriptor_chain_head: usize) -> Option<usize> {
        assert!(self.initialized.get());

        if let Some(element_pos) = self.available_ring_state.insert() {
            // Write the element
            self.available_ring.ring[element_pos as usize]
                .0
                .set(descriptor_chain_head as u16);

            // TODO: Perform a suitable memory barrier using a method exposed by
            // the transport. For now, we don't negotiate
            // VIRTIO_F_ORDER_PLATFORM, which means that any device which
            // requires proper memory barriers (read: not implemented in
            // software, like QEMU) should refuse operation. We use volatile
            // memory accesses, so read/write reordering by the compiler is not
            // an issue.

            // Update the idx
            self.available_ring
                .idx
                .set(self.available_ring.idx.get().wrapping_add(1));

            Some(element_pos as usize)
        } else {
            None
        }
    }

    fn add_descriptor_chain(
        &self,
        buffer_chain: &mut [Option<VirtqueueBuffer<'b>>],
    ) -> Result<usize, ErrorCode> {
        assert!(self.initialized.get());

        // Get size of actual chain, until the first None
        let queue_length = buffer_chain
            .iter()
            .take_while(|elem| elem.is_some())
            .count();

        // Make sure we have sufficient space available
        //
        // This takes into account the negotiated max size and will
        // only list free iterators within that range
        if self.free_descriptor_count() < queue_length {
            return Err(ErrorCode::NOMEM);
        }

        // Walk over the descriptor table & buffer chain in parallel,
        // inserting where empty
        //
        // We don't need to do any bounds checking here, if we run
        // over the boundary it's safe to panic as something is
        // seriously wrong with `free_descriptor_count`
        let mut i = 0;
        let mut previous_descriptor: Option<usize> = None;
        let mut head = None;
        let queuebuf_iter = buffer_chain.iter_mut().peekable();
        for queuebuf in queuebuf_iter.take_while(|queuebuf| queuebuf.is_some()) {
            // Take the queuebuf out of the caller array
            let taken_queuebuf = queuebuf.take().expect("queuebuf is None");

            while self.descriptor_buffers[i].is_some() {
                i += 1;

                // We should never run over the end, as we should have
                // sufficient free descriptors
                assert!(i < self.descriptor_buffers.len());
            }

            // Alright, we found a slot to insert the descriptor
            //
            // Check if it's the first one and store it's index as head
            if head.is_none() {
                head = Some(i);
            }

            // Convert the VirtqueueBuffer into a DMA-safe variant:
            //
            // # Safety
            //
            // This function requires that we don't drop or mem::forget the
            // returned result (which captures the buffer's lifetime), and
            // eventually restore the original `VirtqueueBuffer` after the DMA
            // operation is complete:
            let virtqueue_dma_buffer =
                unsafe { VirtqueueDmaBuffer::from_virtqueue_buffer(taken_queuebuf, self.fence) };

            // Write out the descriptor
            let desc = &self.descriptors.0[i];
            desc.len.set(virtqueue_dma_buffer.len() as u32);
            assert!(desc.len.get() > 0);
            desc.addr.set(virtqueue_dma_buffer.as_ptr() as u64);
            desc.flags
                .write(if virtqueue_dma_buffer.device_writeable() {
                    DescriptorFlags::WriteOnly::SET
                } else {
                    DescriptorFlags::WriteOnly::CLEAR
                });

            // Now that we know our descriptor position, check whether
            // we must chain ourself to a previous descriptor
            if let Some(prev_index) = previous_descriptor {
                self.descriptors.0[prev_index]
                    .flags
                    .modify(DescriptorFlags::Next::SET);
                self.descriptors.0[prev_index].next.set(i as u16);
            }

            // Finally, store the full slice for reference. We don't store a
            // proper Rust slice reference, as this would violate aliasing
            // requirements: while the buffer is in the chain, it may be written
            // by the VirtIO device.
            //
            // This can be changed to something slightly more elegant, once the
            // NonNull functions around slices have been stabilized:
            // https://doc.rust-lang.org/stable/std/ptr/struct.NonNull.html#method.slice_from_raw_parts
            self.descriptor_buffers[i].replace(virtqueue_dma_buffer);

            // Set ourself as the previous descriptor, as we know the position
            // of `next` only in the next loop iteration.
            previous_descriptor = Some(i);

            // Increment the counter to not check the current
            // descriptor entry again
            i += 1;
        }

        Ok(head.expect("No head added to the descriptor table"))
    }

    fn remove_descriptor_chain(
        &self,
        top_descriptor_index: usize,
    ) -> [Option<VirtqueueReturnBuffer<'b>>; MAX_QUEUE_SIZE] {
        assert!(self.initialized.get());

        let mut res: [Option<VirtqueueReturnBuffer<'b>>; MAX_QUEUE_SIZE] = [const { None }; _];

        let mut i = 0;
        let mut next_index: Option<usize> = Some(top_descriptor_index);

        while let Some(current_index) = next_index {
            // Get a reference over the current descriptor
            let current_desc = &self.descriptors.0[current_index];

            // Check whether we have a chained descriptor and store that in next_index
            if current_desc.flags.is_set(DescriptorFlags::Next) {
                next_index = Some(current_desc.next.get() as usize);
            } else {
                next_index = None;
            }

            // Recover the slice originally associated with this
            // descriptor & delete it from the buffers array
            //
            // The caller may have provided us a larger Rust slice,
            // but indicated to only provide a subslice to VirtIO,
            // hence we'll use the stored original slice and also
            // return the subslice length
            let dma_virtqueue_buffer = self.descriptor_buffers[current_index]
                .take()
                .expect("Virtqueue descriptors and slices out of sync");
            assert!(dma_virtqueue_buffer.as_ptr() as u64 == current_desc.addr.get());

            // Return the original VirtqueueBuffer (which we obtain from the DMA
            // buffer, now that the operation is over), and hand it back with
            // the device-indicated length:
            let virtqueue_buffer =
                unsafe { dma_virtqueue_buffer.into_virtqueue_buffer(self.fence) };
            res[i] = Some(VirtqueueReturnBuffer {
                virtqueue_buffer,
                device_len: current_desc.len.get() as usize,
            });

            // Zero the descriptor
            current_desc.addr.set(0);
            current_desc.len.set(0);
            current_desc.flags.set(0);
            current_desc.next.set(0);

            // Increment the loop iterator
            i += 1;
        }

        res
    }

    /// Provide a single chain of buffers to the device.
    ///
    /// This method will iterate over the passed slice until it encounters the
    /// first `None`. It will first validate that the number of buffers can be
    /// inserted into its descriptor table, and if not return
    /// `Err(ErrorCode::NOMEM)`. If sufficient space is available, it takes the
    /// passed buffers out of the provided `Option`s until encountering the
    /// first `None` and shares this buffer chain with the device.
    ///
    /// When the device has finished processing the passed buffer chain, it is
    /// returned to the client either through the
    /// [`SplitVirtqueueClient::buffer_chain_ready`] callback, or can be
    /// retrieved through the [`SplitVirtqueue::pop_used_buffer_chain`] method.
    pub fn provide_buffer_chain(
        &self,
        buffer_chain: &mut [Option<VirtqueueBuffer<'b>>],
    ) -> Result<(), ErrorCode> {
        assert!(self.initialized.get());

        // Try to add the chain into the descriptor array
        let descriptor_chain_head = self.add_descriptor_chain(buffer_chain)?;

        // Now make it available to the device. If there was sufficient space
        // available to add the chain's descriptors (of which there may be
        // multiple), there should also be sufficient space in the available
        // ring (where a multi-descriptor chain will occupy only one elements).
        self.add_available_descriptor(descriptor_chain_head)
            .expect("Insufficient space in available ring");

        // Notify the queue. This must not fail, given that the SplitVirtqueue
        // requires a transport to be set prior to initialization.
        self.transport
            .map(|t| t.queue_notify(self.queue_number.get()))
            .unwrap();

        Ok(())
    }

    /// Attempt to take a buffer chain out of the Virtqueue used ring.
    ///
    /// Returns `None` if the used ring is empty.
    pub fn pop_used_buffer_chain(
        &self,
    ) -> Option<([Option<VirtqueueReturnBuffer<'b>>; MAX_QUEUE_SIZE], usize)> {
        assert!(self.initialized.get());

        self.remove_used_chain()
            .map(|(descriptor_idx, bytes_used)| {
                // Get the descriptor chain
                let chain = self.remove_descriptor_chain(descriptor_idx);

                // Remove the first entry of the available ring, since we
                // got a single buffer back and can therefore make another
                // buffer available to the device without risking an
                // overflow of the used ring
                self.available_ring_state.pop();

                (chain, bytes_used)
            })
    }

    /// Disable callback delivery for the
    /// [`SplitVirtqueueClient::buffer_chain_ready`] method on the registered
    /// client.
    pub fn enable_used_callbacks(&self) {
        self.used_callbacks_enabled.set(true);
    }

    /// Enable callback delivery for the
    /// [`SplitVirtqueueClient::buffer_chain_ready`] method on the registered
    /// client.
    ///
    /// Callback delivery is enabled by default. If this is not desired, call
    /// this method prior to registering a client.
    pub fn disable_used_callbacks(&self) {
        self.used_callbacks_enabled.set(false);
    }
}

impl<const MAX_QUEUE_SIZE: usize, F: DmaFence> Virtqueue
    for SplitVirtqueue<'_, '_, MAX_QUEUE_SIZE, F>
{
    fn used_interrupt(&self) {
        assert!(self.initialized.get());
        // A buffer MAY have been put into the used in by the device
        //
        // Try to extract all pending used buffers and return them to
        // the clients via callbacks

        while self.used_callbacks_enabled.get() {
            if let Some((mut chain, bytes_used)) = self.pop_used_buffer_chain() {
                self.client.map(move |client| {
                    client.buffer_chain_ready(self.queue_number.get(), chain.as_mut(), bytes_used)
                });
            } else {
                break;
            }
        }
    }

    fn physical_addresses(&self) -> VirtqueueAddresses {
        VirtqueueAddresses {
            descriptor_area: core::ptr::from_ref(self.descriptors) as u64,
            driver_area: core::ptr::from_ref(self.available_ring) as u64,
            device_area: core::ptr::from_ref(self.used_ring) as u64,
        }
    }

    fn negotiate_queue_size(&self, max_elements: usize) -> usize {
        assert!(!self.initialized.get());
        let negotiated = cmp::min(MAX_QUEUE_SIZE, max_elements);
        self.max_elements.set(negotiated);
        self.available_ring_state.reset(negotiated);
        negotiated
    }

    fn initialize(&self, queue_number: u32, _queue_elements: usize) {
        assert!(!self.initialized.get());

        // The transport must be set prior to initialization:
        assert!(self.transport.is_some());

        // TODO: Zero the queue
        //
        // For now we assume all passed queue buffers are already
        // zeroed

        self.queue_number.set(queue_number);
        self.initialized.set(true);
    }
}

pub trait SplitVirtqueueClient<'b> {
    fn buffer_chain_ready(
        &self,
        queue_number: u32,
        buffer_chain: &mut [Option<VirtqueueReturnBuffer<'b>>],
        bytes_used: usize,
    );
}
