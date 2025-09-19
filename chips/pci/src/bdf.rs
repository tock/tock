// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use core::fmt::{self, Display, Formatter};

/// Unique identifier of a PCI device
///
/// BDF stands for bus, device, function, which is the standard way to identify
/// and address individual PCI devices on a system.
///
/// Internally this is a newtype around a u32, with the BDF fields packed in such
/// a way that the can be used to easily construct PCI configuration addresses.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Bdf(u32);

impl Bdf {
    /// Constructs a new BDF value from individual components.
    ///
    /// The valid range for `bus` is 0..=255, for `device` is 0..=31, and for
    /// `function` is 0..=7. This function will silently truncate any extra leading
    /// bits from the `device` and `function` parameters before constructing the
    /// final BDF value.
    pub const fn new(bus: u8, device: u8, function: u8) -> Self {
        let bdf = ((bus as u32) << 16)
            | (((device as u32) & 0x1F) << 11)
            | (((function as u32) & 0x07) << 8);
        Bdf(bdf)
    }

    /// Returns the bus number component of this BDF.
    #[inline]
    pub const fn bus(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    /// Returns the device number component of this BDF.
    #[inline]
    pub const fn device(&self) -> u8 {
        ((self.0 >> 11) & 0x1F) as u8
    }

    /// Returns the function number component of this BDF.
    #[inline]
    pub const fn function(&self) -> u8 {
        ((self.0 >> 8) & 0x07) as u8
    }

    /// Constructs the 32-bit CONFIG_ADDRESS value for a given register offset.
    ///
    /// Sets the enable bit (bit 31), includes the BDF tag, and aligns the
    /// register offset to a 32-bit boundary as required by PCI spec.
    #[inline]
    pub(crate) const fn cfg_addr(&self, offset: u16) -> u32 {
        let reg = (offset as u32) & 0xFC;
        0x8000_0000 | self.0 | reg
    }
}

impl Display for Bdf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}.{}",
            self.bus(),
            self.device(),
            self.function()
        )
    }
}
