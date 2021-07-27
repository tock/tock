//! Abstraction Interface for several busses.
//! Useful for devices that support multiple protocols
//!
//! Usage
//! -----
//!
//! I2C example
//! ```rust
//! let bus = components::bus::I2CMasterBusComponent::new(i2c_mux, address)
//!     .finalize(components::spi_bus_component_helper!());
//! ```
//!
//! SPI example
//! ```rust
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
use kernel::hil::bus8080::{self, Bus8080};
use kernel::hil::i2c::{Error, I2CClient, I2CDevice};
use kernel::hil::spi::{ClockPhase, ClockPolarity, SpiMasterClient, SpiMasterDevice};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

/// Bus width used for address width and data width
pub enum BusWidth {
    Bits8,
    Bits16LE,
    Bits16BE,
    Bits32LE,
    Bits32BE,
    Bits64LE,
    Bits64BE,
}

impl BusWidth {
    pub fn width_in_bytes(&self) -> usize {
        match self {
            BusWidth::Bits8 => 1,
            BusWidth::Bits16BE | BusWidth::Bits16LE => 2,
            BusWidth::Bits32BE | BusWidth::Bits32LE => 3,
            BusWidth::Bits64BE | BusWidth::Bits64LE => 4,
        }
    }
}

pub trait Bus<'a> {
    /// Set the address to write to
    ///
    /// If the underlaying bus does not support addresses (eg UART)
    /// this function returns ENOSUPPORT
    fn set_addr(&self, addr_width: BusWidth, addr: usize) -> Result<(), ErrorCode>;

    /// Write data items to the previously set address
    ///
    /// data_width specifies the encoding of the data items placed in the buffer
    /// len specifies the number of data items (the number of bytes is len * data_width.width_in_bytes)
    fn write(
        &self,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Read data items from the previously set address
    ///
    /// data_width specifies the encoding of the data items placed in the buffer
    /// len specifies the number of data items (the number of bytes is len * data_width.width_in_bytes)
    fn read(
        &self,
        data_width: BusWidth,
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

pub struct SpiMasterBus<'a, S: SpiMasterDevice> {
    spi: &'a S,
    read_write_buffer: OptionalCell<&'static mut [u8]>,
    bus_width: Cell<usize>,
    client: OptionalCell<&'a dyn Client>,
    addr_buffer: OptionalCell<&'static mut [u8]>,
    status: Cell<BusStatus>,
}

impl<'a, S: SpiMasterDevice> SpiMasterBus<'a, S> {
    pub fn new(spi: &'a S, addr_buffer: &'static mut [u8]) -> SpiMasterBus<'a, S> {
        SpiMasterBus {
            spi,
            read_write_buffer: OptionalCell::empty(),
            bus_width: Cell::new(1),
            client: OptionalCell::empty(),
            addr_buffer: OptionalCell::new(addr_buffer),
            status: Cell::new(BusStatus::Idle),
        }
    }

    pub fn set_read_write_buffer(&self, buffer: &'static mut [u8]) {
        self.read_write_buffer.replace(buffer);
    }

    pub fn configure(&self, cpol: ClockPolarity, cpal: ClockPhase, rate: u32) {
        self.spi.configure(cpol, cpal, rate);
    }
}

impl<'a, S: SpiMasterDevice> Bus<'a> for SpiMasterBus<'a, S> {
    fn set_addr(&self, addr_width: BusWidth, addr: usize) -> Result<(), ErrorCode> {
        match addr_width {
            BusWidth::Bits8 => self
                .addr_buffer
                .take()
                .map_or(Err(ErrorCode::NOMEM), |buffer| {
                    self.status.set(BusStatus::SetAddress);
                    buffer[0] = addr as u8;
                    let _ = self.spi.read_write_bytes(buffer, None, 1);
                    Ok(())
                }),

            _ => Err(ErrorCode::NOSUPPORT),
        }
    }

    fn write(
        &self,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is sent as is
        let bytes = data_width.width_in_bytes();
        self.bus_width.set(bytes);
        if buffer.len() >= len * bytes {
            self.status.set(BusStatus::Write);
            let _ = self.spi.read_write_bytes(buffer, None, len * bytes);
            Ok(())
        } else {
            Err((ErrorCode::NOMEM, buffer))
        }
    }

    fn read(
        &self,
        data_width: BusWidth,
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
                    self.status.set(BusStatus::Read);
                    let _ = self
                        .spi
                        .read_write_bytes(write_buffer, Some(buffer), len * bytes);
                    Ok(())
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

impl<'a, S: SpiMasterDevice> SpiMasterClient for SpiMasterBus<'a, S> {
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) {
        // debug!("write done {}", len);
        match self.status.get() {
            BusStatus::SetAddress => {
                self.addr_buffer.replace(write_buffer);
                self.client
                    .map(move |client| client.command_complete(None, 0, Ok(())));
            }
            BusStatus::Write | BusStatus::Read => {
                let mut buffer = write_buffer;
                if let Some(buf) = read_buffer {
                    self.read_write_buffer.replace(buffer);
                    buffer = buf;
                }
                self.client.map(move |client| {
                    client.command_complete(Some(buffer), len / self.bus_width.get(), Ok(()))
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

impl<'a, I: I2CDevice> Bus<'a> for I2CMasterBus<'a, I> {
    fn set_addr(&self, addr_width: BusWidth, addr: usize) -> Result<(), ErrorCode> {
        match addr_width {
            BusWidth::Bits8 => self
                .addr_buffer
                .take()
                .map_or(Err(ErrorCode::NOMEM), |buffer| {
                    buffer[0] = addr as u8;
                    self.status.set(BusStatus::SetAddress);
                    match self.i2c.write(buffer, 1) {
                        Ok(()) => Ok(()),
                        Err((error, buffer)) => {
                            self.addr_buffer.replace(buffer);
                            Err(error.into())
                        }
                    }
                }),

            _ => Err(ErrorCode::NOSUPPORT),
        }
    }

    fn write(
        &self,
        data_width: BusWidth,
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
            match self.i2c.write(buffer, (len * bytes) as u8) {
                Ok(()) => Ok(()),
                Err((error, buffer)) => Err((error.into(), buffer)),
            }
        } else {
            Err((ErrorCode::NOMEM, buffer))
        }
    }

    fn read(
        &self,
        data_width: BusWidth,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // endianess does not matter as the buffer is read as is
        let bytes = data_width.width_in_bytes();
        self.len.set(len * bytes);
        if len & bytes < 255 && buffer.len() >= len * bytes {
            self.len.set(len);
            self.status.set(BusStatus::Read);
            match self.i2c.read(buffer, (len * bytes) as u8) {
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

impl<'a, I: I2CDevice> I2CClient for I2CMasterBus<'a, I> {
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
            bus: bus,
            client: OptionalCell::empty(),
            status: Cell::new(BusStatus::Idle),
        }
    }

    fn to_bus8080_width(bus_width: BusWidth) -> Option<bus8080::BusWidth> {
        match bus_width {
            BusWidth::Bits8 => Some(bus8080::BusWidth::Bits8),
            BusWidth::Bits16LE => Some(bus8080::BusWidth::Bits16LE),
            BusWidth::Bits16BE => Some(bus8080::BusWidth::Bits16BE),
            _ => None,
        }
    }
}

impl<'a, B: Bus8080<'static>> Bus<'a> for Bus8080Bus<'a, B> {
    fn set_addr(&self, addr_width: BusWidth, addr: usize) -> Result<(), ErrorCode> {
        if let Some(bus_width) = Self::to_bus8080_width(addr_width) {
            self.bus.set_addr(bus_width, addr)
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    fn write(
        &self,
        data_width: BusWidth,
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
        data_width: BusWidth,
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

impl<'a, B: Bus8080<'static>> bus8080::Client for Bus8080Bus<'a, B> {
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
