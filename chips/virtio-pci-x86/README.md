Virtio PCI Transport
====================

This crate provides an implementation of "Virtio Over PCI Bus", as described in section 4.1 of the
[virtio specification](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.pdf).

Note that this crate can currently only be used on x86 platforms. This is because the underlying PCI
support library carries a hard dependency on I/O port routines from the `x86` crate to access the
PCI configuration space.
