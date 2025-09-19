// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use crate::bdf::Bdf;
use crate::device::Device;

/// Iterator which enumerates all valid BDF combinations and yields those
/// for which a device is present.
struct Iter {
    bus: u8,
    dev: u8,
    func: u8,
}

impl Iter {
    /// Creates a new iterator starting from bus 0, device 0, function 0.
    fn new() -> Self {
        Self {
            bus: 0,
            dev: 0,
            func: 0,
        }
    }
}

impl Iterator for Iter {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        // Loop over all valid bus indices 0..=255
        loop {
            // Loop over all valid device indices 0..=31
            while self.dev <= 31 {
                // Loop over all valid function indices 0..=7
                while self.func <= 7 {
                    let bdf = Bdf::new(self.bus, self.dev, self.func);

                    // Increment function index before potentially returning
                    // a device, so that the next call starts with the next
                    // function.
                    self.func += 1;

                    // Query vendor ID and check if it is valid before yielding
                    // this device instance
                    let device = Device::new(bdf);
                    if device.vendor_id() != 0xFFFF {
                        return Some(device);
                    }
                }

                // Reset function index and increment device index
                self.func = 0;
                self.dev += 1;
            }

            // Break early before incrementing bus index if we have reached the
            // end (else we would overflow u8 and enumerate everything again).
            if self.bus == 255 {
                break;
            }

            // Reset device index and increment bus index
            self.dev = 0;
            self.bus += 1;
        }

        None
    }
}

/// Returns an iterator over all present PCI devices.
///
/// A PCI device is considered "present" if the vendor ID read from its
/// configuration space is not 0xFFFF.
pub fn iter() -> impl Iterator<Item = Device> {
    Iter::new()
}
