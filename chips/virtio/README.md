VIRTIO MMIO Devices
===================

This crate contains drivers for interacting with
[virtio](https://wiki.osdev.org/Virtio) devices via MMIO. The
[formal specification](https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html)
defines a "Virtio Over MMIO" that we implement here.

This psuedo-chip can be included on platforms that use virtio to emulate I/O
devices.
