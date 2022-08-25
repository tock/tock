//! Block device driver for the XT25F64B flash chip.
//!
//! Compatible with [MX25R6435F](http://www.macronix.com/en-us/products/NOR-Flash/Serial-NOR-Flash/Pages/spec.aspx?p=MX25R6435F)
//!
//! From the datasheet:
//!
//! > XT25F64B is 64Mb bits Serial NOR Flash memory, which is configured as
//! > 8,388,608 x 8 internally. When it is in four I/O mode, the structure
//! > becomes 16,777,216 bits x 4 or 33,554,432 bits x 2. XT25F64B feature a
//! > serial peripheral interface and software protocol allowing operation on a
//! > simple 3-wire bus while it is in single I/O mode. The three bus signals
//! > are a clock input (SCLK), a serial data input (SI), and a serial data
//! > output (SO). Serial access to the device is enabled by CS# input.
//!
//! Usage with SMA_Q3 board (nRF52840)
//! -----
//!
//! ```rust
//! use kernel::hil::block_storage::BlockStorage;
//! use kernel::hil::block_storage::HasClient;
//!
//! let flash = {
//!     let mux_spi = components::spi::SpiMuxComponent::new(
//!             &base_peripherals.spim0,
//!             dynamic_deferred_caller,
//!         )
//!         .finalize(components::spi_mux_component_helper!(nrf52840::spi::SPIM));
//!     
//!     base_peripherals.spim0.configure(
//!         nrf52840::pinmux::Pinmux::new(Pin::P0_15 as u32),
//!         nrf52840::pinmux::Pinmux::new(Pin::P0_13 as u32),
//!         nrf52840::pinmux::Pinmux::new(Pin::P0_16 as u32),
//!     );
//!     
//!     components::xt25f64b::Xt25f64bComponent::new(
//!         None,
//!         None,
//!         &nrf52840_peripherals.gpio_port[Pin::P0_14] as &dyn kernel::hil::gpio::Pin,
//!         mux_alarm,
//!         mux_spi,
//!     )
//!     .finalize(components::xt25f64b_component_helper!(
//!         nrf52840::spi::SPIM,
//!         nrf52840::gpio::GPIOPin,
//!         nrf52840::rtc::Rtc,
//!     ))
//! };
//!

// This module is based on the mx25r6435f.rs driver.

use core::cell::Cell;
use kernel::debug;
use kernel::hil;
use kernel::hil::block_storage::{AddressRange, BlockIndex, ReadableStorage};
use kernel::hil::time::ConvertTicks;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::ErrorCode;

pub static mut TXBUFFER: [u8; PAGE_SIZE + 4] = [0; PAGE_SIZE + 4];
pub static mut RXBUFFER: [u8; PAGE_SIZE + 4] = [0; PAGE_SIZE + 4];

const SPI_SPEED: u32 = 8000000;
pub const SECTOR_SIZE: usize = 4096;
pub const PAGE_SIZE: usize = 256;

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
enum State {
    Idle,

    ReadSector { page_index: BlockIndex<PAGE_SIZE> },

    EraseSectorWriteEnable { sector_index: u32 },
    EraseSectorErase,
    EraseSectorCheckDone,
    EraseSectorDone,

    WriteSectorWriteEnable { region: BlockIndex<PAGE_SIZE> },
    WriteSectorWrite { region: BlockIndex<PAGE_SIZE> },
    WriteSectorCheckDone,
    WriteSectorWaitDone,

    ReadId,
}

pub struct XT25F64B<
    'a,
    S: hil::spi::SpiMasterDevice + 'a,
    P: hil::gpio::Pin + 'a,
    A: hil::time::Alarm<'a> + 'a,
> {
    spi: &'a S,
    alarm: &'a A,
    state: Cell<State>,
    write_protect_pin: Option<&'a P>,
    hold_pin: Option<&'a P>,
    txbuffer: TakeCell<'static, [u8]>,
    rxbuffer: TakeCell<'static, [u8]>,
    client: OptionalCell<&'a (dyn hil::block_storage::Client)>,
    client_sector: TakeCell<'static, [u8]>,
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
    > XT25F64B<'a, S, P, A>
{
    pub fn new(
        spi: &'a S,
        alarm: &'a A,
        txbuffer: &'static mut [u8],
        rxbuffer: &'static mut [u8],
        write_protect_pin: Option<&'a P>,
        hold_pin: Option<&'a P>,
    ) -> XT25F64B<'a, S, P, A> {
        XT25F64B {
            spi: spi,
            alarm: alarm,
            state: Cell::new(State::Idle),
            write_protect_pin: write_protect_pin,
            hold_pin: hold_pin,
            txbuffer: TakeCell::new(txbuffer),
            rxbuffer: TakeCell::new(rxbuffer),
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
            .map_or(Err(ErrorCode::RESERVE), |txbuffer| {
                self.rxbuffer
                    .take()
                    .map_or(Err(ErrorCode::RESERVE), move |rxbuffer| {
                        txbuffer[0] = Opcodes::RDID as u8;

                        self.state.set(State::ReadId);
                        if let Err((err, txbuffer, rxbuffer)) =
                            self.spi.read_write_bytes(txbuffer, Some(rxbuffer), 4)
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
            .map_or(Err(ErrorCode::RESERVE), |txbuffer| {
                txbuffer[0] = Opcodes::WREN as u8;
                if let Err((err, txbuffer, _)) = self.spi.read_write_bytes(txbuffer, None, 1) {
                    self.txbuffer.replace(txbuffer);
                    Err(err)
                } else {
                    Ok(())
                }
            })
    }

    fn erase_sector(&self, sector_index: u32) -> Result<(), ErrorCode> {
        self.configure_spi()?;
        self.state
            .set(State::EraseSectorWriteEnable { sector_index });
        self.enable_write()
    }

    fn read_page(
        &self,
        page_index: BlockIndex<PAGE_SIZE>,
        sector: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        match self.state.get() {
            State::Idle => match self.configure_spi() {
                Ok(()) => {
                    let retval = self
                        .txbuffer
                        .take()
                        .map_or(Err(ErrorCode::RESERVE), |txbuffer| {
                            self.rxbuffer
                                .take()
                                .map_or(Err(ErrorCode::RESERVE), move |rxbuffer| {
                                    let address =
                                        AddressRange::from(page_index).start_address as u32;
                                    // Setup the read instruction
                                    txbuffer[0] = Opcodes::READ as u8;
                                    txbuffer[1] = (address >> 16) as u8;
                                    txbuffer[2] = (address >> 8) as u8;
                                    txbuffer[3] = (address >> 0) as u8;
                                    // Call the SPI driver to kick things off.
                                    self.state.set(State::ReadSector { page_index });
                                    if let Err((err, txbuffer, rxbuffer)) =
                                        self.spi.read_write_bytes(
                                            txbuffer,
                                            Some(rxbuffer),
                                            (PAGE_SIZE + 4) as usize,
                                        )
                                    {
                                        self.txbuffer.replace(txbuffer);
                                        self.rxbuffer.replace(rxbuffer.unwrap());
                                        Err(err)
                                    } else {
                                        Ok(())
                                    }
                                })
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
            },
            _ => Err((ErrorCode::BUSY, sector)),
        }
    }

    fn write_pages(
        &self,
        region: &BlockIndex<PAGE_SIZE>,
        buffer: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if AddressRange::from(*region).length_bytes as usize > buffer.len() {
            Err((ErrorCode::SIZE, buffer))
        } else {
            match self.configure_spi() {
                Ok(()) => {
                    self.state
                        .set(State::WriteSectorWriteEnable { region: *region });
                    let retval = self.enable_write();

                    match retval {
                        Ok(()) => {
                            self.client_sector.replace(buffer);
                            Ok(())
                        }
                        Err(ecode) => Err((ecode, buffer)),
                    }
                }
                Err(error) => Err((error, buffer)),
            }
        }
    }
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
    > hil::spi::SpiMasterClient for XT25F64B<'a, S, P, A>
{
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
        read_write_status: Result<(), ErrorCode>,
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
            State::ReadSector { page_index: _ } => {
                self.client_sector.take().map(|sector| {
                    read_buffer.map(move |read_buffer| {
                        // Copy read in bytes to user page
                        for i in 0..PAGE_SIZE {
                            // Skip the command and address bytes (hence the +4).
                            sector[i] = read_buffer[i + 4];
                        }

                        self.state.set(State::Idle);
                        self.txbuffer.replace(write_buffer);
                        self.rxbuffer.replace(read_buffer);

                        self.client.map(move |client| {
                            client.read_complete(sector, Ok(()));
                        });
                    });
                });
            }
            State::EraseSectorWriteEnable { sector_index } => {
                self.state.set(State::EraseSectorErase);
                let address = sector_index * SECTOR_SIZE as u32;
                write_buffer[0] = Opcodes::SE as u8;
                write_buffer[1] = (address >> 16) as u8;
                write_buffer[2] = (address >> 8) as u8;
                write_buffer[3] = (address >> 0) as u8;

                // TODO verify SPI return value
                let _ = self.spi.read_write_bytes(write_buffer, None, 4);
            }
            State::EraseSectorErase => {
                self.state.set(State::EraseSectorCheckDone);
                self.txbuffer.replace(write_buffer);
                // XT25F64B-S Datasheet says erase typically takes 60ms,
                // so we wait that long.
                let delay = self.alarm.ticks_from_ms(60);
                self.alarm.set_alarm(self.alarm.now(), delay);
            }
            State::EraseSectorCheckDone => {
                read_buffer.map(move |read_buffer| {
                    let status = read_buffer[1];

                    // Check the status byte to see if the erase is done or not.
                    if status & 0x01 == 0x01 {
                        // Erase is still in progress.
                        let _ = self
                            .spi
                            .read_write_bytes(write_buffer, Some(read_buffer), 2);
                    } else {
                        // Erase has finished, so jump to the next state.
                        self.state.set(State::EraseSectorDone);
                        self.rxbuffer.replace(read_buffer);
                        self.read_write_done(write_buffer, None, len, read_write_status);
                    }
                });
            }
            State::EraseSectorDone => {
                // No need to disable write, chip does it automatically.
                self.state.set(State::Idle);
                self.txbuffer.replace(write_buffer);
                self.client.map(|client| {
                    client.discard_complete(Ok(()));
                });
            }
            State::WriteSectorWriteEnable { region } => {
                self.state.set(State::WriteSectorWrite { region });
                // Need to write enable before each PP
                write_buffer[0] = Opcodes::WREN as u8;
                // TODO verify SPI return value
                let _ = self.spi.read_write_bytes(write_buffer, None, 1);
            }
            State::WriteSectorWrite { region } => {
                self.state.set(State::WriteSectorCheckDone);
                let address = AddressRange::from(region).start_address as u32;
                write_buffer[0] = Opcodes::PP as u8;
                write_buffer[1] = (address >> 16) as u8;
                write_buffer[2] = (address >> 8) as u8;
                write_buffer[3] = (address >> 0) as u8;

                self.client_sector.map(|sector| {
                    write_buffer[4..][..PAGE_SIZE].copy_from_slice(&sector[..PAGE_SIZE]);
                });

                let _ = self.spi.read_write_bytes(write_buffer, None, PAGE_SIZE);
            }
            State::WriteSectorCheckDone => {
                self.state.set(State::WriteSectorWaitDone);
                self.txbuffer.replace(write_buffer);
                // XT25F64B-S Datasheet says program page typically takes 0.3ms,
                // so we wait that long.
                let delay = self.alarm.ticks_from_us(300);
                self.alarm.set_alarm(self.alarm.now(), delay);
            }
            State::WriteSectorWaitDone => {
                read_buffer.map(move |read_buffer| {
                    let status = read_buffer[1];

                    // Check the status byte to see if the write is done or not.
                    if status & 0x01 == 0x01 {
                        // Write is still in progress.
                        let _ = self
                            .spi
                            .read_write_bytes(write_buffer, Some(read_buffer), 2);
                    } else {
                        // Finished
                        self.state.set(State::Idle);
                        self.txbuffer.replace(write_buffer);
                        self.rxbuffer.replace(read_buffer);
                        self.client.map(|client| {
                            self.client_sector.take().map(|sector| {
                                client.write_complete(sector, Ok(()));
                            });
                        });
                    }
                });
            }
            _ => {}
        }
    }
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
    > hil::time::AlarmClient for XT25F64B<'a, S, P, A>
{
    fn alarm(&self) {
        // After the timer expires we still have to check that the erase/write
        // operation has finished.
        self.txbuffer.take().map(|write_buffer| {
            self.rxbuffer.take().map(move |read_buffer| {
                write_buffer[0] = Opcodes::RDSR as u8;
                let _ = self
                    .spi
                    .read_write_bytes(write_buffer, Some(read_buffer), 2);
            });
        });
    }
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
    > hil::block_storage::ReadableStorage<PAGE_SIZE> for XT25F64B<'a, S, P, A>
{
    fn read(
        &self,
        region: &BlockIndex<PAGE_SIZE>,
        buf: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        let errs = if buf.len() < AddressRange::from(*region).length_bytes as usize {
            Err(ErrorCode::INVAL)
        } else if AddressRange::from(*region).get_end_address() > self.get_size() {
            Err(ErrorCode::INVAL)
        } else {
            Ok(())
        };

        match errs {
            Ok(()) => self.read_page(*region, buf),
            Err(e) => Err((e, buf)),
        }
    }

    /// Returns the size of the device in bytes.
    fn get_size(&self) -> u64 {
        // TODO: it's probably a good idea to discover the size,
        // for the sake of compatible devices.
        8 * 1024 * 1024
    }
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
    > hil::block_storage::WriteableStorage<PAGE_SIZE, SECTOR_SIZE> for XT25F64B<'a, S, P, A>
{
    fn write(
        &self,
        region: &BlockIndex<PAGE_SIZE>,
        buf: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        let errs = if buf.len() < AddressRange::from(*region).length_bytes as usize {
            Err(ErrorCode::INVAL)
        } else if AddressRange::from(*region).get_end_address() > self.get_size() {
            Err(ErrorCode::INVAL)
        } else {
            Ok(())
        };

        match errs {
            Ok(()) => self.write_pages(region, buf),
            Err(e) => Err((e, buf)),
        }
    }

    fn discard(&self, region: &BlockIndex<SECTOR_SIZE>) -> Result<(), ErrorCode> {
        if AddressRange::from(*region).get_end_address() > self.get_size() {
            Err(ErrorCode::INVAL)
        } else {
            self.erase_sector(region.0)
        }
    }
}

impl<
        'a,
        S: hil::spi::SpiMasterDevice + 'a,
        P: hil::gpio::Pin + 'a,
        A: hil::time::Alarm<'a> + 'a,
        C: hil::block_storage::Client,
    > hil::block_storage::HasClient<'a, C> for XT25F64B<'a, S, P, A>
{
    fn set_client(&self, client: &'a C) {
        self.client.set(client);
    }
}
