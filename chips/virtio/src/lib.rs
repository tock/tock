//! VirtIO support for Tock.

#![no_std]
#![crate_name = "virtio"]
#![crate_type = "rlib"]

pub mod devices;
pub mod queues;
pub mod transports;
