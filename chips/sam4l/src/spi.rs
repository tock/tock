//! Implementation of DMA-based SPI master and slave communication for the
//! SAM4L.
//!
//! Driver for the SPI hardware (separate from the USARTS), described in chapter
//! 26 of the datasheet.
//!
//! - Authors: Sam Crow <samcrow@uw.edu>, Philip Levis <pal@cs.stanford.edu>

use core::cell::Cell;
use core::cmp;
use dma::DMAChannel;
use dma::DMAClient;
use dma::DMAPeripheral;
use kernel::ReturnCode;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::hil::spi;
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;
use kernel::hil::spi::SpiMasterClient;
use kernel::hil::spi::SpiSlaveClient;
use pm;

#[repr(C)]
pub struct Registers {
    pub cr: WriteOnly<u32, Control::Register>,
    pub mr: ReadWrite<u32, Mode::Register>,
    pub rdr: ReadOnly<u32>,
    pub tdr: WriteOnly<u32, TransmitData::Register>,
    pub sr: ReadOnly<u32, Status::Register>,
    pub ier: WriteOnly<u32, InterruptFlags::Register>,
    pub idr: WriteOnly<u32, InterruptFlags::Register>,
    pub imr: ReadOnly<u32, InterruptFlags::Register>,
    _reserved0: [ReadOnly<u32>; 4],
    pub csr: [ReadWrite<u32, ChipSelectParams::Register>; 4],
    _reserved1: [ReadOnly<u32>; 41],
    pub wpcr: ReadWrite<u32, WriteProtectionControl::Register>,
    pub wpsr: ReadOnly<u32>,
    _reserved2: [ReadOnly<u32>; 3],
    pub features: ReadOnly<u32>,
    pub version: ReadOnly<u32>,
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

const SPI_BASE: u32 = 0x40008000;

/// Values for selected peripherals
#[derive(Copy, Clone)]
pub enum Peripheral {
    Peripheral0,
    Peripheral1,
    Peripheral2,
    Peripheral3,
}

#[derive(Copy, Clone, PartialEq)]
pub enum SpiRole {
    SpiMaster,
    SpiSlave,
}

/// The SAM4L supports four peripherals.
pub struct Spi {
    registers: *mut Registers,
    client: Cell<Option<&'static SpiMasterClient>>,
    dma_read: Cell<Option<&'static DMAChannel>>,
    dma_write: Cell<Option<&'static DMAChannel>>,
    // keep track of which how many DMA transfers are pending to correctly
    // issue completion event only after both complete.
    transfers_in_progress: Cell<u8>,
    dma_length: Cell<usize>,

    // Slave client is distinct from master client
    slave_client: Cell<Option<&'static SpiSlaveClient>>,
    role: Cell<SpiRole>,
}

pub static mut SPI: Spi = Spi::new();

impl Spi {
    /// Creates a new SPI object, with peripheral 0 selected
    pub const fn new() -> Spi {
        Spi {
            registers: SPI_BASE as *mut Registers,
            client: Cell::new(None),
            dma_read: Cell::new(None),
            dma_write: Cell::new(None),
            transfers_in_progress: Cell::new(0),
            dma_length: Cell::new(0),

            slave_client: Cell::new(None),
            role: Cell::new(SpiRole::SpiMaster),
        }
    }

    fn init_as_role(&self, role: SpiRole) {
        let regs: &Registers = unsafe { &*self.registers };

        self.role.set(role);
        self.enable_clock();

        if self.role.get() == SpiRole::SpiMaster {
            // Only need to set LASTXFER if we are master
            regs.cr.write(Control::LASTXFER::SET);
        }

        // Sets bits per transfer to 8
        let mut csr = self.read_active_csr();
        csr &= !(0x1111 << 4);
        csr |= 0b0000 << 4;
        self.write_active_csr(csr);

        // Set mode to master or slave
        let mode = match self.role.get() {
            SpiRole::SpiMaster => Mode::MSTR::SET,
            SpiRole::SpiSlave => Mode::MSTR::CLEAR,
        };

        // Disable mode fault detection (open drain outputs not supported)
        regs.mr.modify(mode + Mode::MODFDIS::SET);
    }

    pub fn enable(&self) {
        let regs: &Registers = unsafe { &*self.registers };

        self.enable_clock();
        regs.cr.write(Control::SPIEN::SET);

        if self.role.get() == SpiRole::SpiSlave {
            regs.ier.write(InterruptFlags::NSSR::SET); // Enable NSSR
        }
    }

    pub fn disable(&self) {
        let regs: &Registers = unsafe { &*self.registers };

        self.dma_read.get().map(|read| read.disable());
        self.dma_write.get().map(|write| write.disable());
        regs.cr.write(Control::SPIDIS::SET);

        if self.role.get() == SpiRole::SpiSlave {
            regs.idr.write(InterruptFlags::NSSR::SET);; // Disable NSSR
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
        let clock = pm::get_system_frequency();

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
        let mut csr = self.read_active_csr();
        csr = (csr & !(0xFF << 8)) | ((scbr & 0xFF) << 8);
        self.write_active_csr(csr);
        clock / scbr
    }

    pub fn get_baud_rate(&self) -> u32 {
        let clock = 48000000;
        let scbr = (self.read_active_csr() & (0xFF << 8)) >> 8;
        clock / scbr
    }

    fn set_clock(&self, polarity: ClockPolarity) {
        let mut csr = self.read_active_csr();
        match polarity {
            ClockPolarity::IdleHigh => csr |= 0x01,
            ClockPolarity::IdleLow => csr &= !0x01,
        };
        self.write_active_csr(csr);
    }

    fn get_clock(&self) -> ClockPolarity {
        let csr = self.read_active_csr();
        let polarity = csr & 0x01;
        match polarity {
            0 => ClockPolarity::IdleLow,
            _ => ClockPolarity::IdleHigh,
        }
    }

    fn set_phase(&self, phase: ClockPhase) {
        let mut csr = self.read_active_csr();
        match phase {
            ClockPhase::SampleLeading => csr |= 0x02,
            ClockPhase::SampleTrailing => csr &= !0x02,
        };
        self.write_active_csr(csr);
    }

    fn get_phase(&self) -> ClockPhase {
        let csr = self.read_active_csr();
        let phase = csr & 0x02;
        match phase {
            0 => ClockPhase::SampleTrailing,
            _ => ClockPhase::SampleLeading,
        }
    }

    pub fn set_active_peripheral(&self, peripheral: Peripheral) {
        // Slave cannot set active peripheral
        if self.role.get() == SpiRole::SpiMaster {
            let regs: &Registers = unsafe { &*self.registers };
            let mr = match peripheral {
                Peripheral::Peripheral0 => Mode::PCS::PCS0,
                Peripheral::Peripheral1 => Mode::PCS::PCS1,
                Peripheral::Peripheral2 => Mode::PCS::PCS2,
                Peripheral::Peripheral3 => Mode::PCS::PCS3,
            };
            regs.mr.modify(mr);
        }
    }

    /// Returns the currently active peripheral
    pub fn get_active_peripheral(&self) -> Peripheral {
        if self.role.get() == SpiRole::SpiMaster {
            let regs: &Registers = unsafe { &*self.registers };

            let peripheral_number = regs.mr.read(Mode::PCS);
            match peripheral_number {
                0b1110 => Peripheral::Peripheral0,
                0b1101 => Peripheral::Peripheral1,
                0b1011 => Peripheral::Peripheral2,
                0b0111 => Peripheral::Peripheral3,
                _ => {
                    // Invalid configuration
                    Peripheral::Peripheral0
                }
            }
        } else {
            // The active peripheral is always 0 in slave mode
            Peripheral::Peripheral0
        }
    }

    /// Returns the value of CSR0, CSR1, CSR2, or CSR3,
    /// whichever corresponds to the active peripheral
    fn read_active_csr(&self) -> u32 {
        let regs: &Registers = unsafe { &*self.registers };

        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => regs.csr[0].get(),
            Peripheral::Peripheral1 => regs.csr[1].get(),
            Peripheral::Peripheral2 => regs.csr[2].get(),
            Peripheral::Peripheral3 => regs.csr[3].get(),
        }
    }
    /// Sets the Chip Select Register (CSR) of the active peripheral
    /// (CSR0, CSR1, CSR2, or CSR3).
    fn write_active_csr(&self, value: u32) {
        let regs: &Registers = unsafe { &*self.registers };

        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => regs.csr[0].set(value),
            Peripheral::Peripheral1 => regs.csr[1].set(value),
            Peripheral::Peripheral2 => regs.csr[2].set(value),
            Peripheral::Peripheral3 => regs.csr[3].set(value),
        };
    }

    /// Set the DMA channels used for reading and writing.
    pub fn set_dma(&mut self, read: &'static DMAChannel, write: &'static DMAChannel) {
        self.dma_read.set(Some(read));
        self.dma_write.set(Some(write));
    }

    fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(pm::Clock::PBA(pm::PBAClock::SPI));
        }
    }

    pub fn handle_interrupt(&self) {
        let regs: &Registers = unsafe { &*self.registers };

        self.slave_client.get().map(|client| {
            if regs.sr.is_set(Status::NSSR) {
                // NSSR
                client.chip_selected()
            }
            // TODO: Do we want to support byte-level interrupts too?
            // They currently conflict with DMA.
        });
    }

    /// Asynchronous buffer read/write of SPI.
    /// returns `SUCCESS` if operation starts (will receive callback through SpiMasterClient),
    /// returns `EBUSY` if the operation does not start.
    // The write buffer has to be mutable because it's passed back to
    // the caller, and the caller may want to be able write into it.
    fn read_write_bytes(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        self.enable();

        if write_buffer.is_none() && read_buffer.is_none() {
            return ReturnCode::SUCCESS;
        }

        let mut opt_len = None;
        write_buffer.as_ref().map(|buf| opt_len = Some(buf.len()));
        read_buffer.as_ref().map(|buf| {
            let min_len = opt_len.map_or(buf.len(), |old_len| cmp::min(old_len, buf.len()));
            opt_len = Some(min_len);
        });

        let count = cmp::min(opt_len.unwrap_or(0), len);
        self.dma_length.set(count);

        // Reset the number of transfers in progress. This is incremented
        // depending on the presence of the read/write below
        self.transfers_in_progress.set(0);

        // The ordering of these operations matters.
        // For transfers 4 bytes or longer, this will work as expected.
        // For shorter transfers, the first byte will be missing.
        write_buffer.map(|wbuf| {
            self.transfers_in_progress
                .set(self.transfers_in_progress.get() + 1);
            self.dma_write.get().map(move |write| {
                write.enable();
                write.do_xfer(DMAPeripheral::SPI_TX, wbuf, count);
            });
        });

        // Only setup the RX channel if we were passed a read_buffer inside
        // of the option. `map()` checks this for us.
        read_buffer.map(|rbuf| {
            self.transfers_in_progress
                .set(self.transfers_in_progress.get() + 1);
            self.dma_read.get().map(move |read| {
                read.enable();
                read.do_xfer(DMAPeripheral::SPI_RX, rbuf, count);
            });
        });
        ReturnCode::SUCCESS
    }
}

impl spi::SpiMaster for Spi {
    type ChipSelect = u8;

    fn set_client(&self, client: &'static SpiMasterClient) {
        self.client.set(Some(client));
    }

    /// By default, initialize SPI to operate at 40KHz, clock is
    /// idle on low, and sample on the leading edge.
    fn init(&self) {
        self.init_as_role(SpiRole::SpiMaster);
    }

    fn is_busy(&self) -> bool {
        self.transfers_in_progress.get() != 0
    }

    /// Write a byte to the SPI and discard the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn write_byte(&self, out_byte: u8) {
        let regs: &Registers = unsafe { &*self.registers };

        let tdr = (out_byte as u32) & spi_consts::tdr::TD;
        // Wait for data to leave TDR and enter serializer, so TDR is free
        // for this next byte
        while !regs.sr.is_set(Status::TDRE) {}
        regs.tdr.set(tdr);
    }

    /// Write 0 to the SPI and return the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn read_byte(&self) -> u8 {
        self.read_write_byte(0)
    }

    /// Write a byte to the SPI and return the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn read_write_byte(&self, val: u8) -> u8 {
        let regs: &Registers = unsafe { &*self.registers };

        self.write_byte(val);
        // Wait for receive data register full
        while !regs.sr.is_set(Status::RDRF) {}
        // Return read value
        regs.rdr.get() as u8
    }

    /// Asynchronous buffer read/write of SPI.
    /// write_buffer must  be Some; read_buffer may be None;
    /// if read_buffer is Some, then length of read/write is the
    /// minimum of two buffer lengths; returns `SUCCESS` if operation
    /// starts (will receive callback through SpiMasterClient), returns
    /// `EBUSY` if the operation does not start.
    // The write buffer has to be mutable because it's passed back to
    // the caller, and the caller may want to be able write into it.
    fn read_write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        // TODO: Remove? Included in read_write_bytes call
        self.enable();

        // If busy, don't start.
        if self.is_busy() {
            return ReturnCode::EBUSY;
        }

        self.read_write_bytes(Some(write_buffer), read_buffer, len)
    }

    fn set_rate(&self, rate: u32) -> u32 {
        self.set_baud_rate(rate)
    }

    fn get_rate(&self) -> u32 {
        self.get_baud_rate()
    }

    fn set_clock(&self, polarity: ClockPolarity) {
        self.set_clock(polarity);
    }

    fn get_clock(&self) -> ClockPolarity {
        self.get_clock()
    }

    fn set_phase(&self, phase: ClockPhase) {
        self.set_phase(phase);
    }

    fn get_phase(&self) -> ClockPhase {
        self.get_phase()
    }

    fn hold_low(&self) {
        let mut csr = self.read_active_csr();
        csr |= 1 << 3;
        self.write_active_csr(csr);
    }

    fn release_low(&self) {
        let mut csr = self.read_active_csr();
        csr &= !(1 << 3);
        self.write_active_csr(csr);
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) {
        let peripheral_number = match cs {
            0 => Peripheral::Peripheral0,
            1 => Peripheral::Peripheral1,
            2 => Peripheral::Peripheral2,
            3 => Peripheral::Peripheral3,
            _ => Peripheral::Peripheral0,
        };
        self.set_active_peripheral(peripheral_number);
    }
}

impl spi::SpiSlave for Spi {
    // Set to None to disable the whole thing
    fn set_client(&self, client: Option<&'static SpiSlaveClient>) {
        self.slave_client.set(client);
    }

    fn has_client(&self) -> bool {
        self.slave_client.get().is_some()
    }

    fn init(&self) {
        self.init_as_role(SpiRole::SpiSlave);
    }

    /// This sets the value in the TDR register, to be sent as soon as the
    /// chip select pin is low.
    fn set_write_byte(&self, write_byte: u8) {
        let regs: &Registers = unsafe { &*self.registers };
        regs.tdr.set(write_byte as u32);
    }

    fn read_write_bytes(
        &self,
        write_buffer: Option<&'static mut [u8]>,
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        self.read_write_bytes(write_buffer, read_buffer, len)
    }

    fn set_clock(&self, polarity: ClockPolarity) {
        self.set_clock(polarity);
    }

    fn get_clock(&self) -> ClockPolarity {
        self.get_clock()
    }

    fn set_phase(&self, phase: ClockPhase) {
        self.set_phase(phase);
    }

    fn get_phase(&self) -> ClockPhase {
        self.get_phase()
    }
}

impl DMAClient for Spi {
    fn xfer_done(&self, _pid: DMAPeripheral) {
        // Only callback that the transfer is done if either:
        // 1) The transfer was TX only and TX finished
        // 2) The transfer was TX and RX, in that case wait for both of them to complete. Although
        //    both transactions happen simultaneously over the wire, the DMA may not finish copying
        //    data over to/from the controller at the same time, so we don't want to abort
        //    prematurely.

        self.transfers_in_progress
            .set(self.transfers_in_progress.get() - 1);

        if self.transfers_in_progress.get() == 0 {
            let txbuf = self.dma_write.get().map_or(None, |dma| {
                let buf = dma.abort_xfer();
                dma.disable();
                buf
            });

            let rxbuf = self.dma_read.get().map_or(None, |dma| {
                let buf = dma.abort_xfer();
                dma.disable();
                buf
            });

            let len = self.dma_length.get();
            self.dma_length.set(0);

            match self.role.get() {
                SpiRole::SpiMaster => {
                    self.client.get().map(|cb| {
                        txbuf.map(|txbuf| {
                            cb.read_write_done(txbuf, rxbuf, len);
                        });
                    });
                }
                SpiRole::SpiSlave => {
                    self.slave_client.get().map(|cb| {
                        cb.read_write_done(txbuf, rxbuf, len);
                    });
                }
            }
        }
    }
}
