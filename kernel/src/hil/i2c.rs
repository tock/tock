// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for I2C master and slave peripherals.

use crate::ErrorCode;

use core::fmt;
use core::fmt::{Display, Formatter};

/// The type of error encountered during I2C communication.
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

    /// The requested operation wasn't supported.
    NotSupported,

    /// The underlying device has another request in progress
    Busy,
}

impl From<Error> for ErrorCode {
    fn from(val: Error) -> Self {
        match val {
            Error::AddressNak | Error::DataNak => ErrorCode::NOACK,
            Error::ArbitrationLost => ErrorCode::RESERVE,
            Error::Overrun => ErrorCode::SIZE,
            Error::NotSupported => ErrorCode::NOSUPPORT,
            Error::Busy => ErrorCode::BUSY,
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        let display_str = match *self {
            Error::AddressNak => "I2C Address Not Acknowledged",
            Error::DataNak => "I2C Data Not Acknowledged",
            Error::ArbitrationLost => "I2C Bus Arbitration Lost",
            Error::Overrun => "I2C receive overrun",
            Error::NotSupported => "I2C/SMBus command not supported",
            Error::Busy => "I2C/SMBus is busy",
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
pub trait I2CMaster<'a> {
    fn set_master_client(&self, master_client: &'a dyn I2CHwMasterClient);
    fn enable(&self);
    fn disable(&self);
    fn write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;
    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;
    fn read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;
}

/// Interface for an SMBus Master hardware driver.
/// The device implementing this will also seperately implement
/// I2CMaster.
pub trait SMBusMaster<'a>: I2CMaster<'a> {
    /// Write data then read data via the I2C Master device in an SMBus
    /// compatible way.
    ///
    /// This function will use the I2C master to write data to a device and
    /// then read data from the device in a SMBus compatible way. This will be
    /// a best effort attempt to match the SMBus specification based on what
    /// the hardware can support.
    /// This function is expected to make any hardware changes required to
    /// support SMBus and then revert those changes to support future I2C.
    ///
    /// addr: The address of the device to write to
    /// data: The buffer to write the data from and read back to
    /// write_len: The length of the write operation
    /// read_len: The length of the read operation
    fn smbus_write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;

    /// Write data via the I2C Master device in an SMBus compatible way.
    ///
    /// This function will use the I2C master to write data to a device in a
    /// SMBus compatible way. This will be a best effort attempt to match the
    /// SMBus specification based on what the hardware can support.
    /// This function is expected to make any hardware changes required to
    /// support SMBus and then revert those changes to support future I2C.
    ///
    /// addr: The address of the device to write to
    /// data: The buffer to write the data from
    /// len: The length of the operation
    fn smbus_write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;

    /// Read data via the I2C Master device in an SMBus compatible way.
    ///
    /// This function will use the I2C master to read data from a device in a
    /// SMBus compatible way. This will be a best effort attempt to match the
    /// SMBus specification based on what the hardware can support.
    /// This function is expected to make any hardware changes required to
    /// support SMBus and then revert those changes to support future I2C.
    ///
    /// addr: The address of the device to read from
    /// buffer: The buffer to store the data to
    /// len: The length of the operation
    fn smbus_read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;
}

/// Interface for an I2C Slave hardware driver.
pub trait I2CSlave<'a> {
    fn set_slave_client(&self, slave_client: &'a dyn I2CHwSlaveClient);
    fn enable(&self);
    fn disable(&self);
    fn set_address(&self, addr: u8) -> Result<(), Error>;
    fn write_receive(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;
    fn read_send(
        &self,
        data: &'static mut [u8],
        max_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;
    fn listen(&self);
}

/// Convenience type for capsules that need hardware that supports both
/// Master and Slave modes.
pub trait I2CMasterSlave<'a>: I2CMaster<'a> + I2CSlave<'a> {}
// Provide blanket implementations for trait group
// impl<T: I2CMaster + I2CSlave> I2CMasterSlave for T {}

/// Client interface for capsules that use I2CMaster devices.
pub trait I2CHwMasterClient {
    /// Called when an I2C command completed. The `error` denotes whether the command completed
    /// successfully or if an error occurred.
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), Error>);
}

/// Client interface for capsules that use I2CSlave devices.
pub trait I2CHwSlaveClient {
    /// Called when an I2C command completed.
    fn command_complete(
        &self,
        buffer: &'static mut [u8],
        length: usize,
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
    fn write_read(
        &self,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;
    fn write(&self, data: &'static mut [u8], len: usize) -> Result<(), (Error, &'static mut [u8])>;
    fn read(&self, buffer: &'static mut [u8], len: usize)
        -> Result<(), (Error, &'static mut [u8])>;
}

/// Extend the I2CDevice to add support for targetting multiple `I2CDevice`s.
pub trait I2CMultiDevice: I2CDevice {
    fn set_address(&self, addr: u8);
}

pub trait SMBusDevice: I2CDevice {
    /// Write data then read data to a slave device in an SMBus
    /// compatible way.
    ///
    /// This function will use the I2C master to write data to a device and
    /// then read data from the device in a SMBus compatible way. This will be
    /// a best effort attempt to match the SMBus specification based on what
    /// the hardware can support.
    /// This function is expected to make any hardware changes required to
    /// support SMBus and then revert those changes to support future I2C.
    ///
    /// data: The buffer to write the data from and read back to
    /// write_len: The length of the write operation
    /// read_len: The length of the read operation
    fn smbus_write_read(
        &self,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;

    /// Write data to a slave device in an SMBus compatible way.
    ///
    /// This function will use the I2C master to write data to a device in a
    /// SMBus compatible way. This will be a best effort attempt to match the
    /// SMBus specification based on what the hardware can support.
    /// This function is expected to make any hardware changes required to
    /// support SMBus and then revert those changes to support future I2C.
    ///
    /// data: The buffer to write the data from
    /// len: The length of the operation
    fn smbus_write(
        &self,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;

    /// Read data from a slave device in an SMBus compatible way.
    ///
    /// This function will use the I2C master to read data from a device in a
    /// SMBus compatible way. This will be a best effort attempt to match the
    /// SMBus specification based on what the hardware can support.
    /// This function is expected to make any hardware changes required to
    /// support SMBus and then revert those changes to support future I2C.
    ///
    /// buffer: The buffer to store the data to
    /// len: The length of the operation
    fn smbus_read(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (Error, &'static mut [u8])>;
}

/// Client interface for I2CDevice implementations.
pub trait I2CClient {
    /// Called when an I2C command completed. The `error` denotes whether the command completed
    /// successfully or if an error occured.
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), Error>);
}

pub struct NoSMBus;

impl<'a> I2CMaster<'a> for NoSMBus {
    fn set_master_client(&self, _master_client: &'a dyn I2CHwMasterClient) {}
    fn enable(&self) {}
    fn disable(&self) {}
    fn write_read(
        &self,
        _addr: u8,
        data: &'static mut [u8],
        _write_len: usize,
        _read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        Err((Error::NotSupported, data))
    }
    fn write(
        &self,
        _addr: u8,
        data: &'static mut [u8],
        _len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        Err((Error::NotSupported, data))
    }
    fn read(
        &self,
        _addr: u8,
        buffer: &'static mut [u8],
        _len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        Err((Error::NotSupported, buffer))
    }
}

impl<'a> SMBusMaster<'a> for NoSMBus {
    fn smbus_write_read(
        &self,
        _addr: u8,
        data: &'static mut [u8],
        _write_len: usize,
        _read_len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        Err((Error::NotSupported, data))
    }

    fn smbus_write(
        &self,
        _addr: u8,
        data: &'static mut [u8],
        _len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        Err((Error::NotSupported, data))
    }

    fn smbus_read(
        &self,
        _addr: u8,
        buffer: &'static mut [u8],
        _len: usize,
    ) -> Result<(), (Error, &'static mut [u8])> {
        Err((Error::NotSupported, buffer))
    }
}
