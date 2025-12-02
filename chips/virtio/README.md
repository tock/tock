VIRTIO Devices
==============

This crate contains drivers for interacting with
[virtio](https://wiki.osdev.org/Virtio) devices. The
[formal specification](https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html)
defines a "Virtio Over MMIO" that we implement here. Other Virtio transports, which may require
platform-specific dependencies, are broken out into separate `virtio-xxx` crates.

This psuedo-chip can be included on platforms that use virtio to emulate I/O
devices.
