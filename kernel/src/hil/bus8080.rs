// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! The 8080 Bus Interface (used for LCD)

use crate::ErrorCode;

/// Bus width used for address width and data width
pub enum BusWidth {
    Bits8,
    Bits16LE,
    Bits16BE,
}
/// The enum represents the address of a bus-attached device.
///
/// For addresses larger than a single byte the enum variant
/// captures the endianess used by the device on the bus.
/// The address is stored in the host endianess in the u16 and
/// must be converted to the correct endianess before using the
/// address on the bus.
pub enum BusAddr8080 {
    BusAddr8(u8),
    BusAddr16BE(u16),
    BusAddr16LE(u16),
}

impl BusWidth {
    pub fn width_in_bytes(&self) -> usize {
        match self {
            BusWidth::Bits8 => 1,
            BusWidth::Bits16BE | BusWidth::Bits16LE => 2,
        }
    }
}

pub trait Bus8080<'a> {
    /// Set the address to write to
    fn set_addr(&self, addr: BusAddr8080) -> Result<(), ErrorCode>;
    /// Write data items to the previously set address
    fn write(
        &self,
        data_width: BusWidth,
        buffer: &'a mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Read data items from the previously set address
    fn read(
        &self,
        data_width: BusWidth,
        buffer: &'a mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    fn set_client(&self, client: &'a dyn Client);
}

pub trait Client {
    /// Called when set_addr, write or read are complete
    ///
    /// set_address does not return a buffer
    /// write and read return a buffer
    /// len should be set to the number of data elements written
    fn command_complete(
        &self,
        buffer: Option<&'static mut [u8]>,
        len: usize,
        status: Result<(), ErrorCode>,
    );
}
