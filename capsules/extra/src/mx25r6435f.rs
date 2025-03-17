// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! SyscallDriver for the MX25R6435F flash chip.
//!
//! <http://www.macronix.com/en-us/products/NOR-Flash/Serial-NOR-Flash/Pages/spec.aspx?p=MX25R6435F>
//!
//! From the datasheet:
//!
//! > MX25R6435F is 64Mb bits Serial NOR Flash memory, which is configured as
//! > 8,388,608 x 8 internally. When it is in four I/O mode, the structure
//! > becomes 16,777,216 bits x 4 or 33,554,432 bits x 2. MX25R6435F feature a
//! > serial peripheral interface and software protocol allowing operation on a
//! > simple 3-wire bus while it is in single I/O mode. The three bus signals
//! > are a clock input (SCLK), a serial data input (SI), and a serial data
//! > output (SO). Serial access to the device is enabled by CS# input.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::static_init;
//! # use capsules::virtual_alarm::VirtualMuxAlarm;
//!
//! // Create a SPI device for this chip.
//! let mx25r6435f_spi = static_init!(
//!     capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
//!     capsules::virtual_spi::VirtualSpiMasterDevice::new(mux_spi, &nrf5x::gpio::PORT[17])
//! );
//! // Create an alarm for this chip.
//! let mx25r6435f_virtual_alarm = static_init!(
//!     VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
//!     VirtualMuxAlarm::new(mux_alarm)
//! );
//! mx25r6435f_virtual_alarm.setup();
//!
//! // Setup the actual MX25R6435F driver.
//! let mx25r6435f = static_init!(
//!     capsules::mx25r6435f::MX25R6435F<
//!         'static,
//!         capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52::spi::SPIM>,
//!         nrf5x::gpio::GPIOPin,
//!     >,
//!     capsules::mx25r6435f::MX25R6435F::new(
//!         mx25r6435f_spi,
//!         &mut capsules::mx25r6435f::TXBUFFER,
//!         &mut capsules::mx25r6435f::RXBUFFER,
//!         Some(&nrf5x::gpio::PORT[22]),
//!         Some(&nrf5x::gpio::PORT[23])
//!     )
//! );
//! mx25r6435f_spi.set_client(mx25r6435f);
//! mx25r6435f_virtual_alarm.set_client(mx25r6435f);
//! ```

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::debug;
use kernel::hil;
use kernel::hil::time::ConvertTicks;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

pub const TX_BUF_LEN: usize = PAGE_SIZE as usize + 4;
pub const RX_BUF_LEN: usize = PAGE_SIZE as usize + 4;

const SPI_SPEED: u32 = 8000000;
pub const SECTOR_SIZE: u32 = 4096;
pub const PAGE_SIZE: u32 = 256;

/// This is a wrapper around a u8 array that is sized to a single page for the
/// MX25R6435F. The page size is 4k because that is the smallest size that can
/// be erased (even though 256 bytes can be written).
///
/// An example looks like:
///
/// ```rust,ignore
/// # use capsules::mx25r6435f::Mx25r6435fSector;
///
/// static mut PAGEBUFFER: Mx25r6435fSector = Mx25r6435fSector::new();
/// ```
pub struct Mx25r6435fSector(pub [u8; SECTOR_SIZE as usize]);

impl Mx25r6435fSector {
    pub const fn new() -> Self {
        Self([0; SECTOR_SIZE as usize])
    }
}

impl Default for Mx25r6435fSector {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for Mx25r6435fSector {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for Mx25r6435fSector {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for Mx25r6435fSector {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

#[allow(dead_code)]
enum Opcodes {
    WREN = 0x06, // Write Enable
    WRDI = 0x04, // Write Disable
    SE = 0x20,   // Sector Erase
    READ = 0x03, // Normal Read
    PP = 0x02,   // Page Program (write)
    RDID = 0x9f, // Read Identification
    RDSR = 0x05, // Read Status Register
}

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    Erase,
    Write { sector_index: u32 },
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,

    ReadSector {
        sector_index: u32,
        page_index: u32,
    },

    EraseSectorWriteEnable {
        sector_index: u32,
        operation: Operation,
    },
    EraseSectorErase {
        operation: Operation,
    },
    EraseSectorCheckDone {
        operation: Operation,
    },
    EraseSectorDone,

    WriteSectorWriteEnable {
        sector_index: u32,
        page_index: u32,
    },
    WriteSectorWrite {
        sector_index: u32,
        page_index: u32,
    },
    WriteSectorCheckDone {
        sector_index: u32,
        page_index: u32,
    },
    WriteSectorWaitDone {
        sector_index: u32,
        page_index: u32,
    },

    ReadId,
}

pub struct MX25R6435F<
    'a,
    S: hil::spi::SpiMasterDevice<'a> + 'a,
    P: hil::gpio::Pin + 'a,
    A: hil::time::Alarm<'a> + 'a,
> {
    spi: &'a S,
    alarm: &'a A,
    state: Cell<State>,
    write_protect_pin: Option<&'a P>,
    hold_pin: Option<&'a P>,
    txbuffer: MapCell<SubSliceMut<'static, u8>>,
    rxbuffer: MapCell<SubSliceMut<'static, u8>>,
    client: OptionalCell<&'a dyn hil::flash::Client<MX25R6435F<'a, S, P, A>>>,
    client_sector: TakeCell<'static, Mx25r6435fSector>,
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice<'a> + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
    > MX25R6435F<'a, S, P, A>
{
    pub fn new(
        spi: &'a S,
        alarm: &'a A,
        txbuffer: &'static mut [u8],
        rxbuffer: &'static mut [u8],
        write_protect_pin: Option<&'a P>,
        hold_pin: Option<&'a P>,
    ) -> MX25R6435F<'a, S, P, A> {
        MX25R6435F {
            spi,
            alarm,
            state: Cell::new(State::Idle),
            write_protect_pin,
            hold_pin,
            txbuffer: MapCell::new(txbuffer.into()),
            rxbuffer: MapCell::new(rxbuffer.into()),
            client: OptionalCell::empty(),
            client_sector: TakeCell::empty(),
        }
    }

    /// Setup SPI for this chip
    fn configure_spi(&self) -> Result<(), ErrorCode> {
        self.hold_pin.map(|pin| {
            pin.set();
        });
        self.spi.configure(
            hil::spi::ClockPolarity::IdleLow,
            hil::spi::ClockPhase::SampleLeading,
            SPI_SPEED,
        )
    }

    /// Requests the readout of a 24-bit identification number.
    /// This command will cause a debug print when succeeded.
    pub fn read_identification(&self) -> Result<(), ErrorCode> {
        self.configure_spi()?;

        self.txbuffer
            .take()
            .map_or(Err(ErrorCode::RESERVE), |mut txbuffer| {
                self.rxbuffer
                    .take()
                    .map_or(Err(ErrorCode::RESERVE), move |rxbuffer| {
                        txbuffer.reset();
                        txbuffer[0] = Opcodes::RDID as u8;
                        txbuffer.slice(0..4);

                        self.state.set(State::ReadId);
                        if let Err((err, txbuffer, rxbuffer)) =
                            self.spi.read_write_bytes(txbuffer, Some(rxbuffer))
                        {
                            self.txbuffer.replace(txbuffer);
                            self.rxbuffer.replace(rxbuffer.unwrap());
                            Err(err)
                        } else {
                            Ok(())
                        }
                    })
            })
    }

    fn enable_write(&self) -> Result<(), ErrorCode> {
        self.write_protect_pin.map(|pin| {
            pin.set();
        });
        self.txbuffer
            .take()
            .map_or(Err(ErrorCode::RESERVE), |mut txbuffer| {
                txbuffer.reset();
                txbuffer[0] = Opcodes::WREN as u8;
                txbuffer.slice(0..1);
                if let Err((err, txbuffer, _)) = self.spi.read_write_bytes(txbuffer, None) {
                    self.txbuffer.replace(txbuffer);
                    Err(err)
                } else {
                    Ok(())
                }
            })
    }

    fn erase_sector(&self, sector_index: u32) -> Result<(), ErrorCode> {
        self.configure_spi()?;
        self.state.set(State::EraseSectorWriteEnable {
            sector_index,
            operation: Operation::Erase,
        });
        self.enable_write()
    }

    fn read_sector(
        &self,
        sector_index: u32,
        sector: &'static mut Mx25r6435fSector,
    ) -> Result<(), (ErrorCode, &'static mut Mx25r6435fSector)> {
        match self.configure_spi() {
            Ok(()) => {
                let retval =
                    self.txbuffer
                        .take()
                        .map_or(Err(ErrorCode::RESERVE), |mut txbuffer| {
                            self.rxbuffer.take().map_or(
                                Err(ErrorCode::RESERVE),
                                move |mut rxbuffer| {
                                    // Setup the read instruction
                                    txbuffer.reset();
                                    txbuffer[0] = Opcodes::READ as u8;
                                    txbuffer[1] = ((sector_index * SECTOR_SIZE) >> 16) as u8;
                                    txbuffer[2] = ((sector_index * SECTOR_SIZE) >> 8) as u8;
                                    txbuffer[3] = ((sector_index * SECTOR_SIZE) >> 0) as u8;
                                    txbuffer.slice(0..(PAGE_SIZE as usize + 4));

                                    rxbuffer.reset();
                                    rxbuffer.slice(0..(PAGE_SIZE as usize + 4));

                                    // Call the SPI driver to kick things off.
                                    self.state.set(State::ReadSector {
                                        sector_index,
                                        page_index: 0,
                                    });
                                    if let Err((err, txbuffer, rxbuffer)) =
                                        self.spi.read_write_bytes(txbuffer, Some(rxbuffer))
                                    {
                                        self.txbuffer.replace(txbuffer);
                                        self.rxbuffer.replace(rxbuffer.unwrap());
                                        Err(err)
                                    } else {
                                        Ok(())
                                    }
                                },
                            )
                        });

                match retval {
                    Ok(()) => {
                        self.client_sector.replace(sector);
                        Ok(())
                    }
                    Err(ecode) => Err((ecode, sector)),
                }
            }
            Err(error) => Err((error, sector)),
        }
    }

    fn write_sector(
        &self,
        sector_index: u32,
        sector: &'static mut Mx25r6435fSector,
    ) -> Result<(), (ErrorCode, &'static mut Mx25r6435fSector)> {
        match self.configure_spi() {
            Ok(()) => {
                self.state.set(State::EraseSectorWriteEnable {
                    sector_index,
                    operation: Operation::Write { sector_index },
                });
                let retval = self.enable_write();

                match retval {
                    Ok(()) => {
                        self.client_sector.replace(sector);
                        Ok(())
                    }
                    Err(ecode) => Err((ecode, sector)),
                }
            }
            Err(error) => Err((error, sector)),
        }
    }
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice<'a> + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
    > hil::spi::SpiMasterClient for MX25R6435F<'a, S, P, A>
{
    fn read_write_done(
        &self,
        mut write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
        read_write_status: Result<usize, ErrorCode>,
    ) {
        match self.state.get() {
            State::ReadId => {
                self.txbuffer.replace(write_buffer);
                read_buffer.map(|read_buffer| {
                    debug!(
                        "id 0x{:02x}{:02x}{:02x}",
                        read_buffer[1], read_buffer[2], read_buffer[3]
                    );
                    self.rxbuffer.replace(read_buffer);
                });
            }
            State::ReadSector {
                sector_index,
                page_index,
            } => {
                self.client_sector.take().map(|sector| {
                    read_buffer.map(move |read_buffer| {
                        // Copy read in bytes to user page
                        for i in 0..(PAGE_SIZE as usize) {
                            // Skip the command and address bytes (hence the +4).
                            sector[i + (page_index * PAGE_SIZE) as usize] = read_buffer[i + 4];
                        }

                        if (page_index + 1) * PAGE_SIZE == SECTOR_SIZE {
                            // Done reading
                            self.state.set(State::Idle);
                            self.txbuffer.replace(write_buffer);
                            self.rxbuffer.replace(read_buffer);

                            self.client.map(move |client| {
                                client.read_complete(
                                    sector,
                                    read_write_status
                                        .and(Ok(()))
                                        .or(Err(hil::flash::Error::FlashError)),
                                );
                            });
                        } else {
                            let address =
                                (sector_index * SECTOR_SIZE) + ((page_index + 1) * PAGE_SIZE);
                            write_buffer.reset();
                            write_buffer[0] = Opcodes::READ as u8;
                            write_buffer[1] = (address >> 16) as u8;
                            write_buffer[2] = (address >> 8) as u8;
                            write_buffer[3] = (address >> 0) as u8;
                            write_buffer.slice(0..(PAGE_SIZE as usize + 4));

                            self.state.set(State::ReadSector {
                                sector_index,
                                page_index: page_index + 1,
                            });
                            self.client_sector.replace(sector);

                            // TODO verify SPI return value
                            let _ = self.spi.read_write_bytes(write_buffer, Some(read_buffer));
                        }
                    });
                });
            }
            State::EraseSectorWriteEnable {
                sector_index,
                operation,
            } => {
                self.state.set(State::EraseSectorErase { operation });

                write_buffer.reset();
                write_buffer[0] = Opcodes::SE as u8;
                write_buffer[1] = ((sector_index * SECTOR_SIZE) >> 16) as u8;
                write_buffer[2] = ((sector_index * SECTOR_SIZE) >> 8) as u8;
                write_buffer[3] = ((sector_index * SECTOR_SIZE) >> 0) as u8;
                write_buffer.slice(0..4);

                // TODO verify SPI return value
                let _ = self.spi.read_write_bytes(write_buffer, None);
            }
            State::EraseSectorErase { operation } => {
                self.state.set(State::EraseSectorCheckDone { operation });
                self.txbuffer.replace(write_buffer);
                // Datasheet says erase takes 58 ms on average. So we wait that
                // long.
                let delay = self.alarm.ticks_from_ms(58);
                self.alarm.set_alarm(self.alarm.now(), delay);
            }
            State::EraseSectorCheckDone { operation } => {
                read_buffer.map(move |read_buffer| {
                    let status = read_buffer[1];

                    // Check the status byte to see if the erase is done or not.
                    if status & 0x01 == 0x01 {
                        // Erase is still in progress.
                        let _ = self.spi.read_write_bytes(write_buffer, Some(read_buffer));
                    } else {
                        // Erase has finished, so jump to the next state.
                        let next_state = match operation {
                            Operation::Erase => State::EraseSectorDone,
                            Operation::Write { sector_index } => State::WriteSectorWriteEnable {
                                sector_index,
                                page_index: 0,
                            },
                        };
                        self.state.set(next_state);
                        self.rxbuffer.replace(read_buffer);
                        self.read_write_done(write_buffer, None, read_write_status);
                    }
                });
            }
            State::EraseSectorDone => {
                // No need to disable write, chip does it automatically.
                self.state.set(State::Idle);
                self.txbuffer.replace(write_buffer);
                self.client.map(|client| {
                    client.erase_complete(
                        read_write_status
                            .and(Ok(()))
                            .or(Err(hil::flash::Error::FlashError)),
                    );
                });
            }
            State::WriteSectorWriteEnable {
                sector_index,
                page_index,
            } => {
                // Check if we are done. This happens when we have written a
                // sector's worth of data, one page at a time.
                if page_index * PAGE_SIZE == SECTOR_SIZE {
                    // No need to disable writes since it happens automatically.
                    self.state.set(State::Idle);
                    self.txbuffer.replace(write_buffer);
                    self.client.map(|client| {
                        self.client_sector.take().map(|sector| {
                            client.write_complete(
                                sector,
                                read_write_status
                                    .and(Ok(()))
                                    .or(Err(hil::flash::Error::FlashError)),
                            );
                        });
                    });
                } else {
                    self.state.set(State::WriteSectorWrite {
                        sector_index,
                        page_index,
                    });
                    // Need to write enable before each PP
                    write_buffer[0] = Opcodes::WREN as u8;
                    write_buffer.slice(0..1);
                    // TODO verify SPI return value
                    let _ = self.spi.read_write_bytes(write_buffer, None);
                }
            }
            State::WriteSectorWrite {
                sector_index,
                page_index,
            } => {
                // Continue writing page by page.
                self.state.set(State::WriteSectorCheckDone {
                    sector_index,
                    page_index: page_index + 1,
                });
                let address = (sector_index * SECTOR_SIZE) + (page_index * PAGE_SIZE);
                write_buffer.reset();
                write_buffer.slice(0..(PAGE_SIZE as usize + 4));

                write_buffer[0] = Opcodes::PP as u8;
                write_buffer[1] = (address >> 16) as u8;
                write_buffer[2] = (address >> 8) as u8;
                write_buffer[3] = (address >> 0) as u8;

                self.client_sector.map(|sector| {
                    for i in 0..(PAGE_SIZE as usize) {
                        write_buffer[i + 4] = sector[i + (page_index * PAGE_SIZE) as usize];
                    }
                });

                let _ = self.spi.read_write_bytes(write_buffer, None);
            }
            State::WriteSectorCheckDone {
                sector_index,
                page_index,
            } => {
                self.state.set(State::WriteSectorWaitDone {
                    sector_index,
                    page_index,
                });
                self.txbuffer.replace(write_buffer);
                // Datasheet says write page takes 3.2 ms on average. So we wait
                // that long.
                let delay = self.alarm.ticks_from_us(3200);
                self.alarm.set_alarm(self.alarm.now(), delay);
            }
            State::WriteSectorWaitDone {
                sector_index,
                page_index,
            } => {
                read_buffer.map(move |mut read_buffer| {
                    let status = read_buffer[1];

                    read_buffer.slice(0..2);

                    // Check the status byte to see if the write is done or not.
                    if status & 0x01 == 0x01 {
                        // Write is still in progress.
                        let _ = self.spi.read_write_bytes(write_buffer, Some(read_buffer));
                    } else {
                        // Write has finished, so go back to writing.
                        self.state.set(State::WriteSectorWriteEnable {
                            sector_index,
                            page_index,
                        });
                        self.rxbuffer.replace(read_buffer);
                        self.read_write_done(write_buffer, None, read_write_status);
                    }
                });
            }
            _ => {}
        }
    }
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice<'a> + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
    > hil::time::AlarmClient for MX25R6435F<'a, S, P, A>
{
    fn alarm(&self) {
        // After the timer expires we still have to check that the erase/write
        // operation has finished.
        self.txbuffer.take().map(|mut write_buffer| {
            self.rxbuffer.take().map(move |read_buffer| {
                write_buffer.reset();
                write_buffer.slice(0..2);
                write_buffer[0] = Opcodes::RDSR as u8;
                let _ = self.spi.read_write_bytes(write_buffer, Some(read_buffer));
            });
        });
    }
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice<'a> + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
        C: hil::flash::Client<Self>,
    > hil::flash::HasClient<'a, C> for MX25R6435F<'a, S, P, A>
{
    fn set_client(&self, client: &'a C) {
        self.client.set(client);
    }
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice<'a> + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
    > hil::flash::Flash for MX25R6435F<'a, S, P, A>
{
    type Page = Mx25r6435fSector;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        self.read_sector(page_number as u32, buf)
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        self.write_sector(page_number as u32, buf)
    }

    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> {
        self.erase_sector(page_number as u32)
    }
}
