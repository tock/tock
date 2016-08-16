use core::fmt::{Display,Formatter,Result};

/// The type of error encoutered during an I2C command transmission.
#[derive(Copy,Clone)]
pub enum Error {
    /// The slave did not acknowledge the chip address. Most likely the address
    /// is incorrect or the slave is not properly connected.
    AddressNak,

    /// The data was not acknowledged by the slave.
    DataNak,

    /// Arbitration lost, meaning the state of the data line does not correspond
    /// to the data driven onto it. This can happen, for example, when a
    /// higher-priority transmission is in progress by a different master.
    ArbitrationLost,

    /// No error occured and the command completed successfully.
    CommandComplete
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        let display_str = match *self {
            Error::AddressNak => "I2C Address Not Acknowledged",
            Error::DataNak => "I2C Data Not Acknowledged",
            Error::ArbitrationLost => "I2C Bus Arbitration Lost",
            Error::CommandComplete => "I2C Command Completed"
        };
        write!(fmt, "{}", display_str)
    }
}

pub trait I2CController {
    fn enable(&self);
    fn disable(&self);
    fn write_read(&self, addr: u8, data: &'static mut [u8],
                  write_len: u8, read_len: u8);
    fn write(&self, addr: u8, data: &'static mut [u8], len: u8);
    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8);
}

pub trait I2CDevice {
    fn enable(&self);
    fn disable(&self);
    fn write_read(&self, data: &'static mut [u8],
                  write_len: u8, read_len: u8);
    fn write(&self, data: &'static mut [u8], len: u8);
    fn read(&self, buffer: &'static mut [u8], len: u8);
}

pub trait I2CClient {
    /// Called when an I2C command completed. The `error` denotes whether the command completed
    /// successfully or if an error occured.
    fn command_complete(&self, buffer: &'static mut [u8], error: Error);
}
