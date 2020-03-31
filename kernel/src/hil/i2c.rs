//! Interface for I2C master and slave peripherals.

use core::fmt::{Display, Formatter, Result};

/// The type of error encoutered during I2C communication.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

    /// A start condition was received before received data has been read
    /// from the receive register.
    Overrun,

    /// No error occured and the command completed successfully.
    CommandComplete,
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        let display_str = match *self {
            Error::AddressNak => "I2C Address Not Acknowledged",
            Error::DataNak => "I2C Data Not Acknowledged",
            Error::ArbitrationLost => "I2C Bus Arbitration Lost",
            Error::Overrun => "I2C receive overrun",
            Error::CommandComplete => "I2C Command Completed",
        };
        write!(fmt, "{}", display_str)
    }
}

/// This specifies what type of transmission just finished from a Master device.
#[derive(Copy, Clone, Debug)]
pub enum SlaveTransmissionType {
    Write,
    Read,
}

/// Interface for an I2C Master hardware driver.
pub trait I2CMaster {
    fn set_master_client(&self, _master_client: &'static dyn I2CHwMasterClient) {
        panic!("not implemented");
    }
    fn enable(&self);
    fn disable(&self);
    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8);
    fn write(&self, addr: u8, data: &'static mut [u8], len: u8);
    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8);
}

/// Interface for an I2C Slave hardware driver.
pub trait I2CSlave {
    fn set_slave_client(&self, _slave_client: Option<&'static dyn I2CHwSlaveClient>) {
        panic!("not implemented");
    }
    fn enable(&self);
    fn disable(&self);
    fn set_address(&self, addr: u8);
    fn write_receive(&self, data: &'static mut [u8], max_len: u8);
    fn read_send(&self, data: &'static mut [u8], max_len: u8);
    fn listen(&self);
}

/// Convenience type for capsules that need hardware that supports both
/// Master and Slave modes.
pub trait I2CMasterSlave: I2CMaster + I2CSlave {}

/// Client interface for capsules that use I2CMaster devices.
pub trait I2CHwMasterClient {
    /// Called when an I2C command completed. The `error` denotes whether the command completed
    /// successfully or if an error occured.
    fn command_complete(&self, buffer: &'static mut [u8], error: Error);
}

/// Client interface for capsules that use I2CSlave devices.
pub trait I2CHwSlaveClient {
    /// Called when an I2C command completed.
    fn command_complete(
        &self,
        buffer: &'static mut [u8],
        length: u8,
        transmission_type: SlaveTransmissionType,
    );

    /// Called from the I2C slave hardware to say that a Master has sent us
    /// a read message, but the driver did not have a buffer containing data
    /// setup, and therefore cannot respond. The I2C slave hardware will stretch
    /// the clock while waiting for the upper layer capsule to provide data
    /// to send to the remote master. Call `I2CSlave::read_send()` to provide
    /// data.
    fn read_expected(&self);

    /// Called from the I2C slave hardware to say that a Master has sent us
    /// a write message, but there was no buffer setup to read the bytes into.
    /// The HW will stretch the clock while waiting for the user to call
    /// `I2CSlave::write_receive()` with a buffer.
    fn write_expected(&self);
}

/// Higher-level interface for I2C Master commands that wraps in the I2C
/// address. It gives an interface for communicating with a specific I2C
/// device.
pub trait I2CDevice {
    fn enable(&self);
    fn disable(&self);
    fn write_read(&self, data: &'static mut [u8], write_len: u8, read_len: u8);
    fn write(&self, data: &'static mut [u8], len: u8);
    fn read(&self, buffer: &'static mut [u8], len: u8);
}

/// Client interface for I2CDevice implementations.
pub trait I2CClient {
    /// Called when an I2C command completed. The `error` denotes whether the command completed
    /// successfully or if an error occured.
    fn command_complete(&self, buffer: &'static mut [u8], error: Error);
}
