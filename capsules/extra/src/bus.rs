// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Abstraction Interface for several busses.
//! Useful for devices that support multiple protocols
//!
//! Usage
//! -----
//!
//! I2C example
//! ```rust,ignore
//! let bus = components::bus::I2CMasterBusComponent::new(i2c_mux, address)
//!     .finalize(components::spi_bus_component_helper!());
//! ```
//!
//! SPI example
//! ```rust,ignore
//! let bus =
//!     components::bus::SpiMasterBusComponent::new().finalize(components::spi_bus_component_helper!(
//!         // spi type
//!         nrf52840::spi::SPIM,
//!         // chip select
//!         &nrf52840::gpio::PORT[GPIO_D4],
//!          // spi mux
//!         spi_mux
//!     ));
//! ```

use core::cell::Cell;
use kernel::debug;
use kernel::hil::bus8080::{self, Bus8080, BusAddr8080};
use kernel::hil::i2c::{Error, I2CClient, I2CDevice};
use kernel::hil::spi::{ClockPhase, ClockPolarity, SpiMasterClient, SpiMasterDevice};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

// Buses, such as I2C or SPI, are generally serial and transmit data byte by byte,
// without taking endianness into account. The receiving device—in this case,
// the screen—interprets the data and determines the endianness.
// In Tock, the lower-level screen driver sets the address endianness.
// For most buses, with the exception of the parallel 8080 bus, endianness
// is largely transparent.
// We store addresses using primitive data types like `u8`, `u16`, `u32`, and `u64`.

/// The `DataWidth` enum and associated `BusAddr` structs define the width
/// of the data transmitted over a bus.
///
/// The `BusAddr::bytes`` function transforms the address into the
/// specified endianness and returns an iterator.
pub enum DataWidth {
    Bits8,
    Bits16LE,
    Bits16BE,
    Bits32LE,
    Bits32BE,
    Bits64LE,
    Bits64BE,
}

/// Each `BusAddr` struct represents a specific data width and endianness.

/// 8 bit Bus Address
pub struct BusAddr8(u8);

/// 16 bit Big Endian Bus Address
pub struct BusAddr16BE(u16);

/// 16 bit Little Endian Bus Address
pub struct BusAddr16LE(u16);

/// 32 bit Big Endian Bus Address
pub struct BusAddr32BE(u32);

/// 32 bit Little Endian Bus Address
pub struct BusAddr32LE(u32);

/// 64 bit Big Endian Bus Address
pub struct BusAddr64BE(u64);

/// 64 bit Little Endian Bus Address
pub struct BusAddr64LE(u64);

impl From<BusAddr8> for BusAddr8080 {
    fn from(value: BusAddr8) -> Self {
        BusAddr8080::BusAddr8(value.0)
    }
}
impl From<BusAddr16BE> for BusAddr8080 {
    fn from(value: BusAddr16BE) -> Self {
        BusAddr8080::BusAddr16BE(value.0)
    }
}
impl From<BusAddr16LE> for BusAddr8080 {
    fn from(value: BusAddr16LE) -> Self {
        BusAddr8080::BusAddr16LE(value.0)
    }
}

impl From<u8> for BusAddr8 {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<u16> for BusAddr16BE {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<u16> for BusAddr16LE {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<u32> for BusAddr32BE {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<u32> for BusAddr32LE {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<u64> for BusAddr64BE {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<u64> for BusAddr64LE {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// The `BusAddr` trait is implemented for each BusAddr struct.
/// It provides information about the data width and a way
/// to access the underlying byte representation.
pub trait BusAddr {
    const DATA_WIDTH: DataWidth;
    fn len(&self) -> usize {
        Self::DATA_WIDTH.width_in_bytes()
    }
    fn bytes(&self) -> impl Iterator<Item = u8>;
}
impl BusAddr for BusAddr8 {
    const DATA_WIDTH: DataWidth = DataWidth::Bits8;
    fn bytes(&self) -> impl Iterator<Item = u8> {
        self.0.to_be_bytes().into_iter()
    }
}
impl BusAddr for BusAddr16BE {
    const DATA_WIDTH: DataWidth = DataWidth::Bits16BE;
    fn bytes(&self) -> impl Iterator<Item = u8> {
        self.0.to_be_bytes().into_iter()
    }
}
impl BusAddr for BusAddr16LE {
    const DATA_WIDTH: DataWidth = DataWidth::Bits16LE;
    fn bytes(&self) -> impl Iterator<Item = u8> {
        self.0.to_le_bytes().into_iter()
    }
}
impl BusAddr for BusAddr32BE {
    const DATA_WIDTH: DataWidth = DataWidth::Bits32BE;
    fn bytes(&self) -> impl Iterator<Item = u8> {
        self.0.to_be_bytes().into_iter()
    }
}
impl BusAddr for BusAddr32LE {
    const DATA_WIDTH: DataWidth = DataWidth::Bits32LE;
    fn bytes(&self) -> impl Iterator<Item = u8> {
        self.0.to_le_bytes().into_iter()
    }
}
impl BusAddr for BusAddr64BE {
    const DATA_WIDTH: DataWidth = DataWidth::Bits64BE;
    fn bytes(&self) -> impl Iterator<Item = u8> {
        self.0.to_be_bytes().into_iter()
    }
}
impl BusAddr for BusAddr64LE {
    const DATA_WIDTH: DataWidth = DataWidth::Bits64LE;
    fn bytes(&self) -> impl Iterator<Item = u8> {
        self.0.to_le_bytes().into_iter()
    }
}

impl DataWidth {
    pub fn width_in_bytes(&self) -> usize {
        match self {
            DataWidth::Bits8 => 1,
            DataWidth::Bits16BE | DataWidth::Bits16LE => 2,
            DataWidth::Bits32BE | DataWidth::Bits32LE => 4,
            DataWidth::Bits64BE | DataWidth::Bits64LE => 8,
        }
    }
}

pub trait Bus<'a, A: BusAddr> {
    /// Set the address to write to
    ///
    /// If the underlying bus does not support addresses (eg UART)
    /// this function returns ENOSUPPORT
    fn set_addr(&self, addr: A) -> Result<(), ErrorCode>;
    /// Write data items to the previously set address
    ///
    /// data_width specifies the encoding of the data items placed in the buffer
    /// len specifies the number of data items (the number of bytes is len * data_width.width_in_bytes)
    fn write(
        &self,
        data_width: DataWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Read data items from the previously set address
    ///
    /// data_width specifies the encoding of the data items placed in the buffer
    /// len specifies the number of data items (the number of bytes is len * data_width.width_in_bytes)
    fn read(
        &self,
        data_width: DataWidth,
        buffer: &'static mut [u8],
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

#[derive(Copy, Clone)]
enum BusStatus {
    Idle,
    SetAddress,
    Write,
    Read,
}

/*********** SPI ************/

pub struct SpiMasterBus<'a, S: SpiMasterDevice<'a>> {
    spi: &'a S,
    read_write_buffer: OptionalCell<SubSliceMut<'static, u8>>,
    bus_width: Cell<usize>,
    client: OptionalCell<&'a dyn Client>,
    addr_buffer: OptionalCell<SubSliceMut<'static, u8>>,
    status: Cell<BusStatus>,
}

impl<'a, S: SpiMasterDevice<'a>> SpiMasterBus<'a, S> {
    pub fn new(spi: &'a S, addr_buffer: &'static mut [u8]) -> SpiMasterBus<'a, S> {
        SpiMasterBus {
            spi,
            read_write_buffer: OptionalCell::empty(),
            bus_width: Cell::new(1),
            client: OptionalCell::empty(),
            addr_buffer: OptionalCell::new(addr_buffer.into()),
            status: Cell::new(BusStatus::Idle),
        }
    }

    pub fn set_read_write_buffer(&self, buffer: &'static mut [u8]) {
        self.read_write_buffer.replace(buffer.into());
    }

    pub fn configure(
        &self,
        cpol: ClockPolarity,
        cpal: ClockPhase,
        rate: u32,
    ) -> Result<(), ErrorCode> {
        self.spi.configure(cpol, cpal, rate)
    }
}

impl<'a, A: BusAddr, S: SpiMasterDevice<'a>> Bus<'a, A> for SpiMasterBus<'a, S> {
    fn set_addr(&self, addr: A) -> Result<(), ErrorCode> {
        self.addr_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |mut buffer| {
                let bytes = addr.bytes();
                if buffer.len() >= addr.len() {
                    buffer.reset();
                    buffer.slice(0..addr.len());
                    self.status.set(BusStatus::SetAddress);
                    buffer
                        .as_mut_slice()
                        .iter_mut()
                        .zip(bytes)
                        .for_each(|(d, s)| *d = s);
                    if let Err((error, buffer, _)) = self.spi.read_write_bytes(buffer, None) {
                        self.status.set(BusStatus::Idle);
                        self.addr_buffer.replace(buffer);
                        Err(error)
                    } else {
                        Ok(())
                    }
                } else {
                    self.addr_buffer.replace(buffer);
                    Err(ErrorCode::SIZE)
                }
            })
    }

    fn write(
        &self,
        data_width: DataWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is sent as is
        let bytes = data_width.width_in_bytes();
        self.bus_width.set(bytes);
        if buffer.len() >= len * bytes {
            let mut buffer_slice: SubSliceMut<'static, u8> = buffer.into();
            buffer_slice.slice(0..(len * bytes));
            self.status.set(BusStatus::Write);
            if let Err((error, buffer, _)) = self.spi.read_write_bytes(buffer_slice, None) {
                self.status.set(BusStatus::Idle);
                Err((error, buffer.take()))
            } else {
                Ok(())
            }
        } else {
            Err((ErrorCode::NOMEM, buffer))
        }
    }

    fn read(
        &self,
        data_width: DataWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is read as is
        let bytes = data_width.width_in_bytes();
        self.bus_width.set(bytes);
        self.read_write_buffer.take().map_or_else(
            || panic!("bus::read: spi did not return the read write buffer"),
            move |write_buffer| {
                if write_buffer.len() >= len * bytes
                    && write_buffer.len() > 0
                    && buffer.len() > len * bytes
                {
                    let mut buffer_slice: SubSliceMut<'static, u8> = buffer.into();
                    buffer_slice.slice(0..(len * bytes));
                    self.status.set(BusStatus::Read);
                    if let Err((error, write_buffer, buffer)) =
                        self.spi.read_write_bytes(write_buffer, Some(buffer_slice))
                    {
                        self.status.set(BusStatus::Idle);
                        self.read_write_buffer.replace(write_buffer);
                        Err((error, buffer.map_or(&mut [] as &mut [u8], |b| b.take())))
                    } else {
                        Ok(())
                    }
                } else {
                    Err((ErrorCode::NOMEM, buffer))
                }
            },
        )
    }

    fn set_client(&self, client: &'a dyn Client) {
        self.client.replace(client);
    }
}

impl<'a, S: SpiMasterDevice<'a>> SpiMasterClient for SpiMasterBus<'a, S> {
    fn read_write_done(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
        status: Result<usize, ErrorCode>,
    ) {
        match self.status.get() {
            BusStatus::SetAddress => {
                self.addr_buffer.replace(write_buffer);
                self.client.map(move |client| {
                    client.command_complete(None, status.unwrap_or(0), status.map(|_| ()))
                });
            }
            BusStatus::Write | BusStatus::Read => {
                let mut buffer = write_buffer;
                if let Some(buf) = read_buffer {
                    self.read_write_buffer.replace(buffer);
                    buffer = buf;
                }
                self.client.map(move |client| {
                    client.command_complete(
                        Some(buffer.take()),
                        status.unwrap_or(0) / self.bus_width.get(),
                        status.map(|_| ()),
                    )
                });
            }
            _ => {
                panic!("spi sent an extra read_write_done");
            }
        }
    }
}

/*********** I2C ************/

pub struct I2CMasterBus<'a, I: I2CDevice> {
    i2c: &'a I,
    len: Cell<usize>,
    client: OptionalCell<&'a dyn Client>,
    addr_buffer: OptionalCell<&'static mut [u8]>,
    status: Cell<BusStatus>,
}

impl<'a, I: I2CDevice> I2CMasterBus<'a, I> {
    pub fn new(i2c: &'a I, addr_buffer: &'static mut [u8]) -> I2CMasterBus<'a, I> {
        I2CMasterBus {
            i2c,
            len: Cell::new(0),
            client: OptionalCell::empty(),
            addr_buffer: OptionalCell::new(addr_buffer),
            status: Cell::new(BusStatus::Idle),
        }
    }
}

impl<'a, A: BusAddr, I: I2CDevice> Bus<'a, A> for I2CMasterBus<'a, I> {
    fn set_addr(&self, addr: A) -> Result<(), ErrorCode> {
        self.addr_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |buffer| {
                self.status.set(BusStatus::SetAddress);
                let bytes = addr.bytes();
                if buffer.len() >= addr.len() {
                    let () = buffer.iter_mut().zip(bytes).for_each(|(d, s)| *d = s);
                    match self.i2c.write(buffer, addr.len()) {
                        Ok(()) => Ok(()),
                        Err((error, buffer)) => {
                            self.addr_buffer.replace(buffer);
                            Err(error.into())
                        }
                    }
                } else {
                    self.addr_buffer.replace(buffer);
                    Err(ErrorCode::SIZE)
                }
            })
    }

    fn write(
        &self,
        data_width: DataWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is sent as is
        let bytes = data_width.width_in_bytes();
        self.len.set(len * bytes);
        if len * bytes < 255 && buffer.len() >= len * bytes {
            debug!("write len {}", len);
            self.len.set(len);
            self.status.set(BusStatus::Write);
            match self.i2c.write(buffer, len * bytes) {
                Ok(()) => Ok(()),
                Err((error, buffer)) => Err((error.into(), buffer)),
            }
        } else {
            Err((ErrorCode::NOMEM, buffer))
        }
    }

    fn read(
        &self,
        data_width: DataWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is read as is
        let bytes = data_width.width_in_bytes();
        self.len.set(len * bytes);
        if len & bytes < 255 && buffer.len() >= len * bytes {
            self.len.set(len);
            self.status.set(BusStatus::Read);
            match self.i2c.read(buffer, len * bytes) {
                Ok(()) => Ok(()),
                Err((error, buffer)) => Err((error.into(), buffer)),
            }
        } else {
            Err((ErrorCode::NOMEM, buffer))
        }
    }

    fn set_client(&self, client: &'a dyn Client) {
        self.client.replace(client);
    }
}

impl<I: I2CDevice> I2CClient for I2CMasterBus<'_, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), Error>) {
        let len = match status {
            Ok(()) => self.len.get(),
            _ => 0,
        };
        let report_status = match status {
            Ok(()) => Ok(()),
            Err(error) => Err(error.into()),
        };
        match self.status.get() {
            BusStatus::SetAddress => {
                self.addr_buffer.replace(buffer);
                self.client
                    .map(move |client| client.command_complete(None, 0, report_status));
            }
            BusStatus::Write | BusStatus::Read => {
                self.client
                    .map(move |client| client.command_complete(Some(buffer), len, report_status));
            }
            _ => {
                panic!("i2c sent an extra read_write_done");
            }
        }
    }
}

/*************** Bus 8080  ***************/
pub struct Bus8080Bus<'a, B: Bus8080<'static>> {
    bus: &'a B,
    client: OptionalCell<&'a dyn Client>,
    status: Cell<BusStatus>,
}

impl<'a, B: Bus8080<'static>> Bus8080Bus<'a, B> {
    pub fn new(bus: &'a B) -> Bus8080Bus<'a, B> {
        Bus8080Bus {
            bus,
            client: OptionalCell::empty(),
            status: Cell::new(BusStatus::Idle),
        }
    }

    fn to_bus8080_width(bus_width: DataWidth) -> Option<bus8080::BusWidth> {
        match bus_width {
            DataWidth::Bits8 => Some(bus8080::BusWidth::Bits8),
            DataWidth::Bits16LE => Some(bus8080::BusWidth::Bits16LE),
            DataWidth::Bits16BE => Some(bus8080::BusWidth::Bits16BE),
            _ => None,
        }
    }
}

impl<'a, A: BusAddr + Into<BusAddr8080>, B: Bus8080<'static>> Bus<'a, A> for Bus8080Bus<'a, B> {
    fn set_addr(&self, addr: A) -> Result<(), ErrorCode> {
        let _ = self.bus.set_addr(addr.into());
        Ok(())
    }

    fn write(
        &self,
        data_width: DataWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if let Some(bus_width) = Self::to_bus8080_width(data_width) {
            self.bus.write(bus_width, buffer, len)
        } else {
            Err((ErrorCode::INVAL, buffer))
        }
    }

    fn read(
        &self,
        data_width: DataWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if let Some(bus_width) = Self::to_bus8080_width(data_width) {
            self.bus.read(bus_width, buffer, len)
        } else {
            Err((ErrorCode::INVAL, buffer))
        }
    }

    fn set_client(&self, client: &'a dyn Client) {
        self.client.replace(client);
    }
}

impl<B: Bus8080<'static>> bus8080::Client for Bus8080Bus<'_, B> {
    fn command_complete(
        &self,
        buffer: Option<&'static mut [u8]>,
        len: usize,
        status: Result<(), ErrorCode>,
    ) {
        self.status.set(BusStatus::Idle);
        self.client.map(|client| {
            client.command_complete(buffer, len, status);
        });
    }
}
