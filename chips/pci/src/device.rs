// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use tock_registers::{register_bitfields, LocalRegisterCopy};

use super::cfg::{self, offset};
use super::Bdf;

register_bitfields![u16,
    /// PCI Command register bitfields
    pub Command [
        /// I/O space accesses enabled
        IO_SPACE OFFSET(0) NUMBITS(1) [],

        /// Memory space accesses enabled
        MEM_SPACE OFFSET(1) NUMBITS(1) [],

        /// Device is allowed to act as a bus master
        BUS_MASTER OFFSET(2) NUMBITS(1) [],

        /// Monitor/Special cycles on PCI bus
        SPECIAL_CYCLES OFFSET(3) NUMBITS(1) [],

        /// Memory Write and Invalidate enable
        MEM_WRITE_INV OFFSET(4) NUMBITS(1) [],

        /// VGA palette snoop enable
        VGA_PALETTE OFFSET(5) NUMBITS(1) [],

        /// Parity error response enable
        PARITY_ERR_RESP OFFSET(6) NUMBITS(1) [],

        /// SERR# driver enable
        SERR_ENABLE OFFSET(8) NUMBITS(1) [],

        /// Fast back-to-back transactions enable
        FAST_BACK_TO_BACK OFFSET(9) NUMBITS(1) [],

        /// Interrupt disable
        INT_DISABLE OFFSET(10) NUMBITS(1) [],
    ],

    /// PCI Status register bitfields
    pub Status [
        /// Interrupt status (pending)
        INT_STATUS OFFSET(3) NUMBITS(1) [],

        /// Capabilities list present
        CAP_LIST OFFSET(4) NUMBITS(1) [],

        /// 66 MHz capable
        CAP_66MHZ OFFSET(5) NUMBITS(1) [],

        /// Fast back-to-back capable
        FAST_BACK_TO_BACK_CAPABLE OFFSET(7) NUMBITS(1) [],

        /// Master data parity error detected
        MASTER_DATA_PARITY_ERROR OFFSET(8) NUMBITS(1) [],

        /// DEVSEL timing encoding
        DEVSEL OFFSET(9) NUMBITS(2) [],

        /// Signaled target abort
        SIGNALED_TARGET_ABORT OFFSET(11) NUMBITS(1) [],

        /// Received target abort
        RECEIVED_TARGET_ABORT OFFSET(12) NUMBITS(1) [],

        /// Received master abort
        RECEIVED_MASTER_ABORT OFFSET(13) NUMBITS(1) [],

        /// Signaled system error (SERR#)
        SIGNALED_SYSTEM_ERROR OFFSET(14) NUMBITS(1) [],

        /// Detected parity error
        DETECTED_PARITY_ERROR OFFSET(15) NUMBITS(1) [],
    ]
];

/// Type-safe representation of PCI command register value
pub type CommandVal = LocalRegisterCopy<u16, Command::Register>;

/// Type-safe representation of PCI status register value
pub type StatusVal = LocalRegisterCopy<u16, Status::Register>;

/// Representation of a PCI device
///
/// This struct provides low-level methods for directly reading and writing
/// values from the PCI configuration space for this device, as well as
/// higer-level methods for accessing standard fields.
///
/// If you know the BDF of the device you want to access, you can directly create a [`Device`]
/// instance and use it to interact with the device:
///
/// ```ignore
/// let bdf = Bdf::new(0, 1, 0);
/// let dev = Device::new(bdf);
///
/// // Check vendor and device ID:
/// let vid = dev.vendor_id();
/// let did = dev.device_id();
/// if vid == 0x1234 && did == 0x5678 {
///     // Found the device we were looking for!
/// }
/// ```
///
/// Alternatively, you can use the [`iter`][crate::iter] function to enumerate all PCI devices in
/// the system. This method automatically filters out non-existent devices, and it returns an
/// iterator which can be chained like any other Rust iterator:
///
/// ```ignore
/// let dev = pci::iter().find(|d| d.vendor_id() == 0x1234 && d.device_id() == 0x5678);
/// if dev.is_none() {
///     // No such device found in the system.
/// }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Device {
    bdf: Bdf,
}

impl Device {
    /// Constructs a new `Device` instance from a given BDF identifier.
    pub const fn new(bdf: Bdf) -> Self {
        Self { bdf }
    }

    /// Reads an 8-bit value from this device's PCI configuration space.
    #[inline]
    pub fn read8(&self, offset: u16) -> u8 {
        cfg::read8(self.bdf, offset)
    }

    /// Writes an 8-bit value to this device's PCI configuration space.
    #[inline]
    pub fn write8(&self, offset: u16, val: u8) {
        cfg::write8(self.bdf, offset, val)
    }

    /// Reads a 16-bit value from this device's PCI configuration space.
    #[inline]
    pub fn read16(&self, offset: u16) -> u16 {
        cfg::read16(self.bdf, offset)
    }

    /// Writes a 16-bit value to this device's PCI configuration space.
    #[inline]
    pub fn write16(&self, offset: u16, val: u16) {
        cfg::write16(self.bdf, offset, val)
    }

    /// Reads a 32-bit value from this device's PCI configuration space.
    #[inline]
    pub fn read32(&self, offset: u16) -> u32 {
        cfg::read32(self.bdf, offset)
    }

    /// Writes a 32-bit value to this device's PCI configuration space.
    #[inline]
    pub fn write32(&self, offset: u16, val: u32) {
        cfg::write32(self.bdf, offset, val)
    }

    /// Reads and returns the PCI vendor ID.
    #[inline]
    pub fn vendor_id(&self) -> u16 {
        self.read16(offset::VENDOR_ID)
    }

    /// Reads and returns the PCI device ID.
    #[inline]
    pub fn device_id(&self) -> u16 {
        self.read16(offset::DEVICE_ID)
    }

    /// Reads the command register of this device.
    #[inline]
    pub fn command(&self) -> CommandVal {
        let value = self.read16(offset::COMMAND);
        LocalRegisterCopy::new(value)
    }

    /// Sets the command register of this device.
    #[inline]
    pub fn set_command(&self, value: CommandVal) {
        self.write16(offset::COMMAND, value.get());
    }

    /// Reads the status register of this device.
    #[inline]
    pub fn status(&self) -> StatusVal {
        let value = self.read16(offset::STATUS);
        LocalRegisterCopy::new(value)
    }

    /// Reset fields within status register of this device.
    ///
    /// The PCI status register implements "write 1 to reset" behavior for certain fields. This
    /// method may be used to reset such fields.
    #[inline]
    pub fn reset_status(&self, value: StatusVal) {
        self.write16(offset::STATUS, value.get());
    }

    /// Reads the header type of this device.
    #[inline]
    pub fn header_type(&self) -> u8 {
        self.read8(offset::HEADER_TYPE)
    }

    pub fn bar(&self, index: usize) -> Option<u32> {
        if index > 5 {
            return None;
        }
        let off: u16 = offset::BAR0 + (index as u16) * 4u16;
        Some(self.read32(off))
    }

    pub fn set_bar(&self, index: usize, val: u32) {
        if index > 5 {
            return;
        }
        let off: u16 = offset::BAR0 + (index as u16) * 4u16;
        self.write32(off, val)
    }

    /// Decodes the BAR at `index` and returns its memory address.
    ///
    /// - Returns `None` if `index` is out of range, the BAR is an I/O BAR,
    ///   or the BAR type is unsupported.
    /// - For 64-bit memory BARs, this will read the high dword from the next
    ///   BAR and combine them.
    pub fn bar_addr(&self, index: u8) -> Option<usize> {
        if index > 5 {
            return None;
        }

        let v = self.bar(index as usize)?;

        // I/O BARs not supported here
        if (v & 0x1) != 0 {
            return None;
        }

        let typ = (v >> 1) & 0x3;
        match typ {
            // 32-bit MMIO BAR
            0 => Some((v & 0xFFFF_FFF0) as usize),

            // 64-bit MMIO BAR (consumes the next BAR as high dword)
            2 => {
                if (index as usize) + 1 > 5 {
                    return None;
                }
                let low = (v & 0xFFFF_FFF0) as u64;
                let high = self.bar((index as usize) + 1)? as u64;
                let addr = ((high << 32) | low) as usize;
                Some(addr)
            }

            // Unsupported types
            _ => None,
        }
    }

    /// Returns the offset of the first capability pointer, if present.
    pub fn cap_ptr(&self) -> Option<u8> {
        // Check status register to see whether cap_ptr is valid (Capabilities List bit)
        if !self.status().is_set(Status::CAP_LIST) {
            return None;
        }

        let ptr = self.read8(offset::CAP_PTR);
        if ptr == 0 {
            None
        } else {
            Some(ptr)
        }
    }

    /// Iterate over this device's capabilities list.
    pub fn capabilities(&self) -> super::cap::CapIter<'_> {
        super::cap::CapIter::new(self)
    }

    /// Returns the interrupt line assigned to this device, if applicable.
    #[inline]
    pub fn int_line(&self) -> Option<u8> {
        // Config register only exists for normal devices
        if self.header_type() != 0 {
            return None;
        }

        let val = self.read8(offset::INT_LINE);

        // Per the spec, 0xFF indicates the interrupt line is disconnected
        if val == 0xFF {
            return None;
        }

        Some(val)
    }
}
