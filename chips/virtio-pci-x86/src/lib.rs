// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

#![no_std]

//! Virtio over PCI Local Bus
//!
//! This crate implements support for enumerating and interacting with Virtio
//! devices over the legacy PCI Local Bus transport (i.e. non-PCIe), as defined
//! in section 4.1 of the Virtio specification.

mod pci;
pub use self::pci::{VirtIOPCIDevice, DEVICE_ID_BASE, VENDOR_ID};
