use core::fmt::{Display,Formatter,Result};

/// The type of error encoutered during an I2C command transmission.
#[derive(Copy,Clone)]
pub enum Error {
    /// The slave did not acknowledge the chip address. Most likely the address
    /// is incorrect or the slave is not properly connected.
    AddressNak,

    /// The data was not acknowledged by the slave
    DataNak,

    /// Arbitration lost, meaning the state of the data line does not correspond
    /// to the data driven onto it. This can happen, for example, when a
    /// higher-priority transmission is in progress by a different master.
    ArbitrationLost
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        let display_str = match *self {
            Error::AddressNak => "I2C Address Not Acknowledged",
            Error::DataNak => "I2C Data Not Acknowledged",
            Error::ArbitrationLost => "ArbitrationLost"
        };
        write!(fmt, "{}", display_str)
    }
}

pub trait I2C {
    fn enable(&self);
    fn disable(&self);
    fn write_sync(&self, addr: u16, data: &[u8]);
    fn read_sync(&self, addr: u16, buffer: &mut [u8]);
}

pub trait I2CClient {
    /// Called when an I2C command completed successfully
    fn command_complete(&self, buffer: &'static mut [u8]);

    /// Called when an I2C command did not complete because of an error
    fn command_error(&self, buffer: &'static mut [u8], error: Error);
}
