// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! VirtIO Virtqueues.
//!
//! This module and its submodules provide abstractions for and
//! implementations of VirtIO Virtqueues. For more information, see
//! the documentation of the [`Virtqueue`] trait.

pub mod split_queue;

/// A set of addresses representing a Virtqueue in memory.
pub struct VirtqueueAddresses {
    pub descriptor_area: u64,
    pub driver_area: u64,
    pub device_area: u64,
}

/// A VirtIO Virtqueue.
///
/// Virtqueues are VirtIO's mechanism to exchange data between a host and a
/// guest. Each queue instance provides a bidirectional communication channel to
/// send data from the guest (VirtIO driver) to the host (VirtIO device) and
/// back. Typically, a given Virtqueue is only used for communication in a
/// single direction. A VirtIO device can support multiple Virtqueues.
///
/// Fundamentally, every Virtqueue refers to three distinct regions in memory:
///
/// - the **descriptor area**: this memory region contains an array of so-called
///   Virtqueue descriptors. Each descriptor is a data structure containing
///   metadata concerning a buffer shared by the guest (VirtIO driver) into the
///   Virtqueue, such as its guest-physical address in memory and
///   length. Multiple shared buffers in distinct memory locations can be
///   chained into a single buffer.
///
/// - the **available ring**: this available ring is a data structure maintained
///   by the guest (VirtIO driver). It contains a ring-buffer which is used for
///   the guest to share descriptors (as maintained in the _descriptor area_)
///   with the host (VirtIO device).
///
/// - the **used ring**: buffers shared with the host (VirtIO device) will
///   eventually be returned to the guest (VirtIO driver) by the device placing
///   them into the used ring. The host will further issue an interrupt, which
///   shall be routed to the Virtqueue by means of the
///   [`Virtqueue::used_interrupt`] method.
pub trait Virtqueue {
    /// Negotiate the number of used descriptors in the Virtqueue.
    ///
    /// The method is presented with the maximum number of queue elements the
    /// intended device can support. The method must return a value smaller or
    /// equal than `device_max_elements`. The device's addresses as returned by
    /// [`Virtqueue::physical_addresses`] after Virtqueue initialization must
    /// have a memory layout adhering to the [Virtual I/O Device (VIRTIO)
    /// Specification, Version
    /// 1.1](https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html),
    /// for at least the returned number of queue elements.
    ///
    /// This method may be called any number of times before initialization of
    /// the [`Virtqueue`]. Only its latest returned value is valid and must be
    /// passed to [`Virtqueue::initialize`].
    fn negotiate_queue_size(&self, device_max_elements: usize) -> usize;

    /// Initialize the Virtqueue.
    ///
    /// This method must bring the Virtqueue into a state where it can be
    /// exposed to a VirtIO transport based on the return value of
    /// [`Virtqueue::physical_addresses`]. A Virtqueue must refuse operations
    /// which require it to be initialized until this method has been called.
    ///
    /// The passed `queue_number` must identify the queue to the device. After
    /// returning from [`Virtqueue::initialize`], calls into the Virtqueue
    /// (except for [`Virtqueue::physical_addresses`]) may attempt to
    /// communicate with the VirtIO device referencing this passed
    /// `queue_number`. In practice, this means that the VirtIO device should be
    /// made aware of this [`Virtqueue`] promptly after calling this method, at
    /// least before invoking [`Virtqueue::used_interrupt`].
    ///
    /// The provided `queue_elements` is the latest negotiated queue size, as
    /// returned by [`Virtqueue::negotiate_queue_size`].
    fn initialize(&self, queue_number: u32, queue_elements: usize);

    /// The physical addresses of the Virtqueue descriptors, available and used ring.
    ///
    /// The returned addresses and their memory contents must adhere to the
    /// [Virtual I/O Device (VIRTIO) Specification, Version
    /// 1.1](https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html)
    ///
    /// This method must not be called before the Virtqueue has been
    /// initialized.
    fn physical_addresses(&self) -> VirtqueueAddresses;

    /// Interrupt indicating that a VirtIO device may have placed a buffer into
    /// this Virtqueue's used ring.
    ///
    /// A [`Virtqueue`] must be tolerant of spurious calls to this method.
    fn used_interrupt(&self);
}
