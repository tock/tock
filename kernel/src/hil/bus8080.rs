//! The 8080 Bus Interface (used for LCD)

use crate::ReturnCode;

/// Bus width used for address width and data width
pub enum BusWidth {
    Bits8,
    Bits16LE,
    Bits16BE,
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
    fn set_addr(&self, addr_width: BusWidth, addr: usize) -> ReturnCode;

    /// Write data items to the previously set address
    fn write(&self, data_width: BusWidth, buffer: &'a mut [u8], len: usize) -> ReturnCode;

    /// Read data items from the previously set address
    fn read(&self, data_width: BusWidth, buffer: &'a mut [u8], len: usize) -> ReturnCode;

    fn set_client(&self, client: &'a dyn Client);
}

pub trait Client {
    /// Called when set_addr, write or read are complete
    ///
    /// set_address does not return a buffer
    /// write and read return a buffer
    /// len should be set to the number of data elements written
    fn command_complete(&self, buffer: Option<&'static mut [u8]>, len: usize);
}
