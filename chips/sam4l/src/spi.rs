// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of DMA-based SPI master and slave communication for the
//! SAM4L.
//!
//! Driver for the SPI hardware (separate from the USARTS), described in chapter
//! 26 of the datasheet.
//!
//! - Authors: Sam Crow <samcrow@uw.edu>, Philip Levis <pal@cs.stanford.edu>

use crate::dma::DMAChannel;
use crate::dma::DMAClient;
use crate::dma::DMAPeripheral;
use crate::pm;
use core::cell::Cell;
use core::cmp;
use kernel::hil::spi;
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;
use kernel::hil::spi::SpiMasterClient;
use kernel::hil::spi::SpiSlaveClient;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::peripheral_management::{PeripheralManagement, PeripheralManager};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{self, register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

#[repr(C)]
pub struct SpiRegisters {
    cr: WriteOnly<u32, Control::Register>,
    mr: ReadWrite<u32, Mode::Register>,
    rdr: ReadOnly<u32>,
    tdr: WriteOnly<u32, TransmitData::Register>,
    sr: ReadOnly<u32, Status::Register>,
    ier: WriteOnly<u32, InterruptFlags::Register>,
    idr: WriteOnly<u32, InterruptFlags::Register>,
    imr: ReadOnly<u32, InterruptFlags::Register>,
    _reserved0: [ReadOnly<u32>; 4],
    csr: [ReadWrite<u32, ChipSelectParams::Register>; 4],
    _reserved1: [ReadOnly<u32>; 41],
    wpcr: ReadWrite<u32, WriteProtectionControl::Register>,
    wpsr: ReadOnly<u32>,
    _reserved2: [ReadOnly<u32>; 3],
    features: ReadOnly<u32>,
    version: ReadOnly<u32>,
}

register_bitfields![u32,
    Control [
        LASTXFER 24,
        FLUSHFIFO 8,
        SWRST 7,
        SPIDIS 1,
        SPIEN 0
    ],

    /// Mode of the SPI peripheral.
    Mode [
        /// Delay between chip selects
        DLYBCS   OFFSET(24)  NUMBITS(8) [],
        /// Peripheral chip select
        PCS      OFFSET(16)  NUMBITS(4) [
            /// One-hot encoding
            PCS0 = 0b1110,
            PCS1 = 0b1101,
            PCS2 = 0b1011,
            PCS3 = 0b0111
        ],
        /// Local loopback enable
        LLB      OFFSET( 7)  NUMBITS(1) [],
        /// FIFO in reception enable
        RXFIFOEN OFFSET( 6)  NUMBITS(1) [],
        /// Mode fault detection
        MODFDIS  OFFSET( 4)  NUMBITS(1) [],
        /// Chip select decode
        PCSDEC   OFFSET( 2)  NUMBITS(1) [],
        /// Peripheral select
        PS       OFFSET( 1)  NUMBITS(1) [],
        /// Master/slave mode
        MSTR     OFFSET( 0)  NUMBITS(1) []
    ],

    TransmitData [
        LASTXFER OFFSET(24)  NUMBITS(1),
        PCS      OFFSET(16)  NUMBITS(4),
        TD       OFFSET(0)   NUMBITS(16)
    ],

    Status [
        SPIENS  OFFSET(16),
        UNDES   OFFSET(10),
        TXEMPTY OFFSET(9),
        NSSR    OFFSET(8),
        OVRES   OFFSET(3),
        MODF    OFFSET(2),
        TDRE    OFFSET(1),
        RDRF    OFFSET(0)
    ],

    InterruptFlags [
        UNDES 10,
        TXEMPTY 9,
        NSSR 8,
        OVRES 3,
        MODF 2,
        TDRE 1,
        RDRF 0
    ],

    ChipSelectParams [
        DLYBCT OFFSET(24)  NUMBITS(8) [],
        DLYBS  OFFSET(16)  NUMBITS(8) [],
        SCBR   OFFSET(8)   NUMBITS(8) [],
        BITS   OFFSET(4)   NUMBITS(8) [
            Eight = 0,
            Nine = 1,
            Ten = 2,
            Eleven = 3,
            Twelve = 4,
            Thirteen = 5,
            Fourteen = 6,
            Fifteen = 7,
            Sixteen = 8,
            Four = 9,
            Five = 10,
            Six = 11,
            Seven = 12
        ],
        CSAAT OFFSET(3)  NUMBITS(1) [
            ActiveAfterTransfer = 1,
            InactiveAfterTransfer = 0
        ],
        CSNAAT OFFSET(2)  NUMBITS(1) [
            DoNotRiseBetweenTransfers = 0,
            RiseBetweenTransfers = 1
        ],
        NCPHA OFFSET(1)  NUMBITS(1) [
            CaptureLeading = 1,
            CaptureTrailing = 0
        ],
        CPOL OFFSET(0)  NUMBITS(1) [
            InactiveHigh = 1,
            InactiveLow = 0
        ]
    ],

    WriteProtectionControl [
        SPIWPKEY OFFSET(8) NUMBITS(24) [
            Key = 0x535049
        ],
        SPIWPEN OFFSET(0) NUMBITS(1) []
    ]
];

#[allow(unused_variables, dead_code)]
// Per-register masks defined in the SPI manual in chapter 26.8
mod spi_consts {
    pub mod rdr {
        pub const RD: u32 = 0xFFFF;
    }

    pub mod tdr {
        pub const TD: u32 = 0xFFFF;
        // PCSx masks from MR also apply here
        // LASTXFER from CR also applies here
    }
}

/// Values for selected peripherals
#[derive(Copy, Clone)]
pub enum Peripheral {
    Peripheral0,
    Peripheral1,
    Peripheral2,
    Peripheral3,
}

impl spi::cs::IntoChipSelect<Peripheral, spi::cs::ActiveLow> for Peripheral {
    fn into_cs(self) -> Peripheral {
        self
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum SpiRole {
    SpiMaster,
    SpiSlave,
}

/// Abstraction of the SPI Hardware
pub struct SpiHw<'a> {
    client: OptionalCell<&'a dyn SpiMasterClient>,
    dma_read: OptionalCell<&'static DMAChannel>,
    dma_write: OptionalCell<&'static DMAChannel>,
    // keep track of which how many DMA transfers are pending to correctly
    // issue completion event only after both complete.
    transfers_in_progress: Cell<u8>,
    dma_length: Cell<usize>,

    // Slave client is distinct from master client
    slave_client: OptionalCell<&'a dyn SpiSlaveClient>,
    role: Cell<SpiRole>,
    pm: &'a pm::PowerManager,
}

const SPI_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x40008000 as *const SpiRegisters) };

impl PeripheralManagement<pm::Clock> for SpiHw<'_> {
    type RegisterType = SpiRegisters;

    fn get_registers(&self) -> &SpiRegisters {
        &SPI_BASE
    }

    fn get_clock(&self) -> &pm::Clock {
        &pm::Clock::PBA(pm::PBAClock::SPI)
    }

    fn before_peripheral_access(&self, clock: &pm::Clock, _: &SpiRegisters) {
        clock.enable();
    }

    fn after_peripheral_access(&self, clock: &pm::Clock, registers: &SpiRegisters) {
        if !registers.sr.is_set(Status::SPIENS) {
            clock.disable();
        }
    }
}

type SpiRegisterManager<'a, 'm> = PeripheralManager<'m, SpiHw<'a>, pm::Clock>;

impl<'a> SpiHw<'a> {
    /// Creates a new SPI object, with peripheral 0 selected
    pub const fn new(pm: &'a pm::PowerManager) -> SpiHw<'a> {
        SpiHw {
            client: OptionalCell::empty(),
            dma_read: OptionalCell::empty(),
            dma_write: OptionalCell::empty(),
            transfers_in_progress: Cell::new(0),
            dma_length: Cell::new(0),

            slave_client: OptionalCell::empty(),
            role: Cell::new(SpiRole::SpiMaster),
            pm,
        }
    }

    fn init_as_role(&self, spi: &SpiRegisterManager<'a, '_>, role: SpiRole) {
        self.role.set(role);

        if role == SpiRole::SpiMaster {
            // Only need to set LASTXFER if we are master
            spi.registers.cr.write(Control::LASTXFER::SET);
        }

        // Sets bits per transfer to 8
        let csr = self.get_active_csr(spi);
        csr.modify(ChipSelectParams::BITS::Eight);

        // Set mode to master or slave
        let mode = match self.role.get() {
            SpiRole::SpiMaster => Mode::MSTR::SET,
            SpiRole::SpiSlave => Mode::MSTR::CLEAR,
        };

        // Disable mode fault detection (open drain outputs not supported)
        spi.registers.mr.modify(mode + Mode::MODFDIS::SET);
    }

    fn enable(&self) {
        let spi = &SpiRegisterManager::new(self);

        spi.registers.cr.write(Control::SPIEN::SET);

        if self.role.get() == SpiRole::SpiSlave {
            spi.registers.ier.write(InterruptFlags::NSSR::SET); // Enable NSSR
        }
    }

    fn disable(&self) {
        let spi = &SpiRegisterManager::new(self);

        // TODO(alevy): we actually probably want to do this asynchrounously but
        // because we're using DMA, a transfer may have completed with a byte
        // still in the TX buffer.
        while !spi.registers.sr.is_set(Status::TXEMPTY) {}

        self.dma_read.map(|read| read.disable());
        self.dma_write.map(|write| write.disable());
        spi.registers.cr.write(Control::SPIDIS::SET);

        if self.role.get() == SpiRole::SpiSlave {
            spi.registers.idr.write(InterruptFlags::NSSR::SET); // Disable NSSR
        }
    }

    /// Sets the approximate baud rate for the active peripheral,
    /// and return the actual baud rate set.
    ///
    /// Since the only supported baud rates are (system clock / n) where n
    /// is an integer from 1 to 255, the exact baud rate may not
    /// be available. In that case, the next lower baud rate will
    /// be selected.
    ///
    /// The lowest available baud rate is 188235 baud. If the
    /// requested rate is lower, 188235 baud will be selected.
    pub fn set_baud_rate(&self, rate: u32) -> u32 {
        // Main clock frequency
        let mut real_rate = rate;
        let clock = self.pm.get_system_frequency();

        if real_rate < 188235 {
            real_rate = 188235;
        }
        if real_rate > clock {
            real_rate = clock;
        }

        // Divide and truncate, resulting in a n value that might be too low
        let mut scbr = clock / real_rate;
        // If the division was not exact, increase the n to get a slower baud
        // rate, but only if we are not at the slowest rate. Since scbr is the
        // clock rate divisor, the highest divisor 0xFF corresponds to the
        // lowest rate.
        if clock % real_rate != 0 && scbr != 0xFF {
            scbr += 1;
        }
        let spi = &SpiRegisterManager::new(self);
        let csr = self.get_active_csr(spi);
        csr.modify(ChipSelectParams::SCBR.val(scbr));
        clock / scbr
    }

    fn get_baud_rate(&self) -> u32 {
        let spi = &SpiRegisterManager::new(self);
        let clock = 48000000;
        let scbr = self.get_active_csr(spi).read(ChipSelectParams::SCBR);
        clock / scbr
    }

    fn set_polarity(&self, polarity: ClockPolarity) {
        let spi = &SpiRegisterManager::new(self);
        let csr = self.get_active_csr(spi);
        match polarity {
            ClockPolarity::IdleHigh => csr.modify(ChipSelectParams::CPOL::InactiveHigh),
            ClockPolarity::IdleLow => csr.modify(ChipSelectParams::CPOL::InactiveLow),
        }
    }

    fn get_polarity(&self) -> ClockPolarity {
        let spi = &SpiRegisterManager::new(self);
        let csr = self.get_active_csr(spi);
        if csr.matches_all(ChipSelectParams::CPOL::InactiveLow) {
            ClockPolarity::IdleLow
        } else {
            ClockPolarity::IdleHigh
        }
    }

    fn set_phase(&self, phase: ClockPhase) {
        let spi = &SpiRegisterManager::new(self);
        let csr = self.get_active_csr(spi);
        match phase {
            ClockPhase::SampleLeading => csr.modify(ChipSelectParams::NCPHA::CaptureLeading),
            ClockPhase::SampleTrailing => csr.modify(ChipSelectParams::NCPHA::CaptureTrailing),
        }
    }

    fn get_phase(&self) -> ClockPhase {
        let spi = &SpiRegisterManager::new(self);
        let csr = self.get_active_csr(spi);
        if csr.matches_all(ChipSelectParams::NCPHA::CaptureTrailing) {
            ClockPhase::SampleTrailing
        } else {
            ClockPhase::SampleLeading
        }
    }

    /// Returns the currently active peripheral
    fn get_active_peripheral(&self, spi: &SpiRegisterManager<'a, '_>) -> Peripheral {
        if self.role.get() == SpiRole::SpiMaster {
            if spi.registers.mr.matches_all(Mode::PCS::PCS3) {
                Peripheral::Peripheral3
            } else if spi.registers.mr.matches_all(Mode::PCS::PCS2) {
                Peripheral::Peripheral2
            } else if spi.registers.mr.matches_all(Mode::PCS::PCS1) {
                Peripheral::Peripheral1
            } else {
                // default
                Peripheral::Peripheral0
            }
        } else {
            // The active peripheral is always 0 in slave mode
            Peripheral::Peripheral0
        }
    }

    /// Returns the value of CSR0, CSR1, CSR2, or CSR3,
    /// whichever corresponds to the active peripheral
    fn get_active_csr<'s>(
        &'s self,
        spi: &SpiRegisterManager<'a, 's>,
    ) -> &'s registers::ReadWrite<u32, ChipSelectParams::Register> {
        match self.get_active_peripheral(spi) {
            Peripheral::Peripheral0 => &spi.registers.csr[0],
            Peripheral::Peripheral1 => &spi.registers.csr[1],
            Peripheral::Peripheral2 => &spi.registers.csr[2],
            Peripheral::Peripheral3 => &spi.registers.csr[3],
        }
    }

    /// Set the DMA channels used for reading and writing.
    pub fn set_dma(&self, read: &'static DMAChannel, write: &'static DMAChannel) {
        self.dma_read.set(read);
        self.dma_write.set(write);
    }

    pub fn handle_interrupt(&self) {
        let spi = &SpiRegisterManager::new(self);

        self.slave_client.map(|client| {
            if spi.registers.sr.is_set(Status::NSSR) {
                // NSSR
                client.chip_selected()
            }
            // TODO: Do we want to support byte-level interrupts too?
            // They currently conflict with DMA.
        });
    }

    /// Asynchronous buffer read/write of SPI.
    ///
    /// Returns:
    /// - `Ok(())` if operation starts (will receive callback through
    ///   SpiMasterClient)
    /// - `INVAL` if no buffers were passed in
    // The write buffer has to be mutable because it's passed back to
    // the caller, and the caller may want to be able write into it.
    fn read_write_bytes(
        &self,
        write_buffer: Option<SubSliceMut<'static, u8>>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            Option<SubSliceMut<'static, u8>>,
        ),
    > {
        let count = match (&write_buffer, &read_buffer) {
            (Some(ref wb), Some(ref rb)) => cmp::min(wb.len(), rb.len()),
            (Some(ref wb), None) => wb.len(),
            (None, Some(ref rb)) => rb.len(),
            (None, None) => return Err((ErrorCode::INVAL, write_buffer, read_buffer)),
        };

        // Start by enabling the SPI driver.
        self.enable();

        // Configure DMA to transfer that many bytes.
        self.dma_length.set(count);

        // Reset the number of transfers in progress. This is incremented
        // depending on the presence of the read/write below
        self.transfers_in_progress.set(0);

        // Only setup the RX channel if we were passed a read_buffer inside
        // of the option. `map()` checks this for us.
        // The read DMA transfer has to be set up before the write because
        // otherwise when the CPU speed is not significantly faster than the
        // SPI's baud rate, transfer_done does not capture the interrupt
        // signaling the RX is done - may be due to missing the first read
        // byte when you start read after write.
        read_buffer.map(|rbuf| {
            self.transfers_in_progress
                .set(self.transfers_in_progress.get() + 1);
            self.dma_read.map(move |read| {
                read.enable();
                read.do_transfer(DMAPeripheral::SPI_RX, rbuf.take(), count);
            });
        });

        // The ordering of these operations matters.
        // For transfers 4 bytes or longer, this will work as expected.
        // For shorter transfers, the first byte will be missing.
        write_buffer.map(|buf| {
            self.transfers_in_progress
                .set(self.transfers_in_progress.get() + 1);
            self.dma_write.map(move |write| {
                write.enable();
                write.do_transfer(DMAPeripheral::SPI_TX, buf.take(), count);
            });
        });

        Ok(())
    }
}

impl<'a> spi::SpiMaster<'a> for SpiHw<'a> {
    type ChipSelect = Peripheral;

    fn set_client(&self, client: &'a dyn SpiMasterClient) {
        self.client.set(client);
    }

    /// By default, initialize SPI to operate at 40KHz, clock is
    /// idle on low, and sample on the leading edge.
    fn init(&self) -> Result<(), ErrorCode> {
        let spi = &SpiRegisterManager::new(self);
        self.init_as_role(spi, SpiRole::SpiMaster);
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.transfers_in_progress.get() != 0
    }

    /// Write a byte to the SPI and discard the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn write_byte(&self, out_byte: u8) -> Result<(), ErrorCode> {
        let spi = &SpiRegisterManager::new(self);

        let tdr = (out_byte as u32) & spi_consts::tdr::TD;
        // Wait for data to leave TDR and enter serializer, so TDR is free
        // for this next byte
        while !spi.registers.sr.is_set(Status::TDRE) {}
        spi.registers.tdr.set(tdr);
        Ok(())
    }

    /// Write 0 to the SPI and return the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn read_byte(&self) -> Result<u8, ErrorCode> {
        self.read_write_byte(0)
    }

    /// Write a byte to the SPI and return the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn read_write_byte(&self, val: u8) -> Result<u8, ErrorCode> {
        let spi = &SpiRegisterManager::new(self);

        self.write_byte(val)?;
        while !spi.registers.sr.is_set(Status::RDRF) {}
        Ok(spi.registers.rdr.get() as u8)
    }

    /// Asynchronous buffer read/write of SPI. `write_buffer` must be present;
    /// `read_buffer` may be `None`. If read_buffer is present, then the length
    /// of the read/write is the minimum of two buffer lengths.
    ///
    /// Returns:
    /// - `Ok(())` if operation starts (will receive callback through
    ///   SpiMasterClient)
    /// - `BUSY` if the operation does not start
    // The write buffer has to be mutable because it's passed back to
    // the caller, and the caller may want to be able write into it.
    fn read_write_bytes(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
    ) -> Result<
        (),
        (
            ErrorCode,
            SubSliceMut<'static, u8>,
            Option<SubSliceMut<'static, u8>>,
        ),
    > {
        // If busy, don't start.
        if self.is_busy() {
            return Err((ErrorCode::BUSY, write_buffer, read_buffer));
        }

        if let Err((err, write_buffer, read_buffer)) =
            self.read_write_bytes(Some(write_buffer), read_buffer)
        {
            Err((
                err,
                write_buffer.unwrap_or((&mut [] as &mut [u8]).into()),
                read_buffer,
            ))
        } else {
            Ok(())
        }
    }

    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        Ok(self.set_baud_rate(rate))
    }

    fn get_rate(&self) -> u32 {
        self.get_baud_rate()
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        self.set_polarity(polarity);
        Ok(())
    }

    fn get_polarity(&self) -> ClockPolarity {
        self.get_polarity()
    }

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode> {
        self.set_phase(phase);
        Ok(())
    }

    fn get_phase(&self) -> ClockPhase {
        self.get_phase()
    }

    fn hold_low(&self) {
        let spi = &SpiRegisterManager::new(self);
        let csr = self.get_active_csr(spi);
        csr.modify(ChipSelectParams::CSAAT::ActiveAfterTransfer);
    }

    fn release_low(&self) {
        let spi = &SpiRegisterManager::new(self);
        let csr = self.get_active_csr(spi);
        csr.modify(ChipSelectParams::CSAAT::InactiveAfterTransfer);
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        // Slave cannot set active peripheral
        if self.role.get() == SpiRole::SpiMaster {
            let spi = &SpiRegisterManager::new(self);
            let mr = match cs {
                Peripheral::Peripheral0 => Mode::PCS::PCS0,
                Peripheral::Peripheral1 => Mode::PCS::PCS1,
                Peripheral::Peripheral2 => Mode::PCS::PCS2,
                Peripheral::Peripheral3 => Mode::PCS::PCS3,
            };
            spi.registers.mr.modify(mr);
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }
}

impl<'a> spi::SpiSlave<'a> for SpiHw<'a> {
    // Set to None to disable the whole thing
    fn set_client(&self, client: Option<&'a dyn SpiSlaveClient>) {
        self.slave_client.insert(client);
    }

    fn has_client(&self) -> bool {
        self.slave_client.is_some()
    }

    fn init(&self) -> Result<(), ErrorCode> {
        let spi = &SpiRegisterManager::new(self);
        self.init_as_role(spi, SpiRole::SpiSlave);
        Ok(())
    }

    /// This sets the value in the TDR register, to be sent as soon as the
    /// chip select pin is low.
    fn set_write_byte(&self, write_byte: u8) {
        let spi = SpiRegisterManager::new(self);
        spi.registers.tdr.set(write_byte as u32);
    }

    /// Setup buffers for a SPI transaction initiated by the master device.
    ///
    /// Returns:
    /// - `Ok(())` if the operation starts. A callback will be generated.
    /// - `INVAL` if neither the read or write buffer is provided.
    fn read_write_bytes(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<&'static mut [u8]>,
            Option<&'static mut [u8]>,
        ),
    > {
        let write_buffer = write_buffer.map(|b| {
            let mut buf: SubSliceMut<u8> = b.into();
            if buf.len() > len {
                buf.slice(..len);
            }
            buf
        });
        let read_buffer = read_buffer.map(|b| {
            let mut buf: SubSliceMut<u8> = b.into();
            if buf.len() > len {
                buf.slice(..len);
            }
            buf
        });

        if let Err((e, mwb, mrb)) = self.read_write_bytes(write_buffer, read_buffer) {
            Err((e, mwb.map(|b| b.take()), mrb.map(|b| b.take())))
        } else {
            Ok(())
        }
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        self.set_polarity(polarity);
        Ok(())
    }

    fn get_polarity(&self) -> ClockPolarity {
        self.get_polarity()
    }

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode> {
        self.set_phase(phase);
        Ok(())
    }

    fn get_phase(&self) -> ClockPhase {
        self.get_phase()
    }
}

impl DMAClient for SpiHw<'_> {
    fn transfer_done(&self, _pid: DMAPeripheral) {
        // Only callback that the transfer is done if either:
        // 1) The transfer was TX only and TX finished
        // 2) The transfer was TX and RX, in that case wait for both of them to complete. Although
        //    both transactions happen simultaneously over the wire, the DMA may not finish copying
        //    data over to/from the controller at the same time, so we don't want to abort
        //    prematurely.

        self.transfers_in_progress
            .set(self.transfers_in_progress.get() - 1);

        if self.transfers_in_progress.get() == 0 {
            let txbuf = self.dma_write.map_or(None, |dma| {
                let buf = dma.abort_transfer();
                dma.disable();
                buf
            });

            let rxbuf = self.dma_read.map_or(None, |dma| {
                let buf = dma.abort_transfer();
                dma.disable();
                buf
            });

            let len = self.dma_length.get();
            self.dma_length.set(0);

            match self.role.get() {
                SpiRole::SpiMaster => {
                    self.client.map(|cb| {
                        txbuf.map(|txbuf| {
                            cb.read_write_done(txbuf.into(), rxbuf.map(|b| b.into()), Ok(len));
                        });
                    });
                }
                SpiRole::SpiSlave => {
                    self.slave_client.map(|cb| {
                        cb.read_write_done(txbuf, rxbuf, len, Ok(()));
                    });
                }
            }
            if self.transfers_in_progress.get() == 0 {
                self.disable();
            }
        }
    }
}
