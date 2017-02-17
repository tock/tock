//! Implementation of DMA-based SPI master and slave communication for the
//! SAM4L.
//!
//! Driver for the SPI hardware (separate from the USARTS), described in chapter
//! 26 of the datasheet.
//!
//! - Authors: Sam Crow <samcrow@uw.edu>, Philip Levis <pal@cs.stanford.edu>

use core::cell::Cell;
use core::cmp;
use core::mem;

use dma::DMAChannel;
use dma::DMAClient;
use dma::DMAPeripheral;
use kernel::ReturnCode;

use kernel::common::VolatileCell;

use kernel::hil::spi;
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;
use kernel::hil::spi::SpiMasterClient;
use kernel::hil::spi::SpiSlaveClient;
use pm;

/// The registers used to interface with the hardware
#[repr(C, packed)]
struct SpiRegisters {
    cr: VolatileCell<u32>, //       0x0
    mr: VolatileCell<u32>, //       0x4
    rdr: VolatileCell<u32>, //      0x8
    tdr: VolatileCell<u32>, //      0xC
    sr: VolatileCell<u32>, //       0x10
    ier: VolatileCell<u32>, //      0x14
    idr: VolatileCell<u32>, //      0x18
    imr: VolatileCell<u32>, //      0x1C
    _reserved0: [u32; 4], //        0x20, 0x24, 0x28, 0x2C
    csr0: VolatileCell<u32>, //     0x30
    csr1: VolatileCell<u32>, //     0x34
    csr2: VolatileCell<u32>, //     0x38
    csr3: VolatileCell<u32>, //     0x3C
    _reserved1: [u32; 41], //       0x40 - 0xE0
    wpcr: VolatileCell<u32>, //     0xE4
    wpsr: VolatileCell<u32>, //     0xE8
    _reserved2: [u32; 3], //        0xEC - 0xF4
    features: VolatileCell<u32>, // 0xF8
    version: VolatileCell<u32>, //  0xFC
}

#[allow(unused_variables,dead_code)]
// Per-register masks defined in the SPI manual in chapter 26.8
mod spi_consts {
    pub mod cr {
        pub const SPIEN: u32 = 1 << 0;
        pub const SPIDIS: u32 = 1 << 1;
        pub const SWRST: u32 = 1 << 7;
        pub const FLUSHFIFO: u32 = 1 << 8;
        pub const LASTXFER: u32 = 1 << 24;
    }

    pub mod mr {
        pub const MSTR: u32 = 1 << 0;
        pub const PS: u32 = 1 << 1;
        pub const PCSDEC: u32 = 1 << 2;
        pub const MODFDIS: u32 = 1 << 4;
        pub const RXFIFOEN: u32 = 1 << 6;
        pub const LLB: u32 = 1 << 7;
        pub const PCS_MASK: u32 = 0b1111 << 16;
        pub const PCS0: u32 = 0b1110 << 16;
        pub const PCS1: u32 = 0b1101 << 16;
        pub const PCS2: u32 = 0b1011 << 16;
        pub const PCS3: u32 = 0b0111 << 16;
        pub const DLYBCS_MASK: u32 = 0xFF << 24;
    }

    pub mod rdr {
        pub const RD: u32 = 0xFFFF;
    }

    pub mod tdr {
        pub const TD: u32 = 0xFFFF;
        // PCSx masks from MR also apply here
        // LASTXFER from CR also applies here
    }

    pub mod sr {
        // These same bits are used in IDR, IER, and IMR.
        pub const RDRF: u32 = 1 << 0;
        pub const TDRE: u32 = 1 << 1;
        pub const MODF: u32 = 1 << 2;
        pub const OVRES: u32 = 1 << 3;
        pub const NSSR: u32 = 1 << 8;
        pub const TXEMPTY: u32 = 1 << 9;
        pub const UNDES: u32 = 1 << 10;

        // This only exists in the SR
        pub const SPIENS: u32 = 1 << 16;
    }

    // These bit masks apply to CSR0; CSR1, CSR2, CSR3
    pub mod csr {
        pub const CPOL: u32 = 1 << 0;
        pub const NCPHA: u32 = 1 << 1;
        pub const CSNAAT: u32 = 1 << 2;
        pub const CSAAT: u32 = 1 << 3;
        pub const BITS_MASK: u32 = 0x1111 << 4;
        pub const BITS8: u32 = 0b0000 << 4;
        pub const BITS9: u32 = 0b0001 << 4;
        pub const BITS10: u32 = 0b0010 << 4;
        pub const BITS11: u32 = 0b0011 << 4;
        pub const BITS12: u32 = 0b0100 << 4;
        pub const BITS13: u32 = 0b0101 << 4;
        pub const BITS14: u32 = 0b0110 << 4;
        pub const BITS15: u32 = 0b0111 << 4;
        pub const BITS16: u32 = 0b1000 << 4;
        pub const BITS4: u32 = 0b1001 << 4;
        pub const BITS5: u32 = 0b1010 << 4;
        pub const BITS6: u32 = 0b1011 << 4;
        pub const BITS7: u32 = 0b1100 << 4;
        pub const SCBR_MASK: u32 = 0xFF << 8;
        pub const DLYBS_MASK: u32 = 0xFF << 16;
        pub const DLYBCT_MASK: u32 = 0xFF << 24;
    }
}

const SPI_BASE: u32 = 0x40008000;

/// Values for selected peripherals
#[derive(Copy,Clone)]
pub enum Peripheral {
    Peripheral0,
    Peripheral1,
    Peripheral2,
    Peripheral3,
}

#[derive(Copy,Clone,PartialEq)]
pub enum SpiRole {
    SpiMaster,
    SpiSlave,
}

/// The SAM4L supports four peripherals.
pub struct Spi {
    registers: *mut SpiRegisters,
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
            registers: SPI_BASE as *mut SpiRegisters,
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
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        self.role.set(role);
        self.enable_clock();

        if self.role.get() == SpiRole::SpiMaster {
            // Only need to set LASTXFER if we are master
            regs.cr.set(spi_consts::cr::LASTXFER);
        }

        // Sets bits per transfer to 8
        let mut csr = self.read_active_csr();
        csr &= !spi_consts::csr::BITS_MASK;
        csr |= spi_consts::csr::BITS8;
        self.write_active_csr(csr);

        // Set mode to master or slave
        let mut mode = regs.mr.get();
        match self.role.get() {
            SpiRole::SpiMaster => mode |= spi_consts::mr::MSTR,
            SpiRole::SpiSlave => mode &= !spi_consts::mr::MSTR,
        }

        // Disable mode fault detection (open drain outputs not supported)
        mode |= spi_consts::mr::MODFDIS;
        regs.mr.set(mode);
    }

    pub fn enable(&self) {
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        self.enable_clock();
        regs.cr.set(spi_consts::cr::SPIEN);

        if self.role.get() == SpiRole::SpiSlave {
            regs.ier.set(spi_consts::sr::NSSR); // Enable NSSR
        }
    }

    pub fn disable(&self) {
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        self.dma_read.get().map(|read| read.disable());
        self.dma_write.get().map(|write| write.disable());
        regs.cr.set(spi_consts::cr::SPIDIS);

        if self.role.get() == SpiRole::SpiSlave {
            regs.idr.set(spi_consts::sr::NSSR); // Disable NSSR
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
        csr = (csr & !spi_consts::csr::SCBR_MASK) | ((scbr & 0xFF) << 8);
        self.write_active_csr(csr);
        clock / scbr
    }

    pub fn get_baud_rate(&self) -> u32 {
        let clock = 48000000;
        let scbr = (self.read_active_csr() & spi_consts::csr::SCBR_MASK) >> 8;
        clock / scbr
    }

    fn set_clock(&self, polarity: ClockPolarity) {
        let mut csr = self.read_active_csr();
        match polarity {
            ClockPolarity::IdleHigh => csr |= spi_consts::csr::CPOL,
            ClockPolarity::IdleLow => csr &= !spi_consts::csr::CPOL,
        };
        self.write_active_csr(csr);
    }

    fn get_clock(&self) -> ClockPolarity {
        let csr = self.read_active_csr();
        let polarity = csr & spi_consts::csr::CPOL;
        match polarity {
            0 => ClockPolarity::IdleLow,
            _ => ClockPolarity::IdleHigh,
        }
    }

    fn set_phase(&self, phase: ClockPhase) {
        let mut csr = self.read_active_csr();
        match phase {
            ClockPhase::SampleLeading => csr |= spi_consts::csr::NCPHA,
            ClockPhase::SampleTrailing => csr &= !spi_consts::csr::NCPHA,
        };
        self.write_active_csr(csr);
    }

    fn get_phase(&self) -> ClockPhase {
        let csr = self.read_active_csr();
        let phase = csr & spi_consts::csr::NCPHA;
        match phase {
            0 => ClockPhase::SampleTrailing,
            _ => ClockPhase::SampleLeading,
        }
    }

    pub fn set_active_peripheral(&self, peripheral: Peripheral) {
        // Slave cannot set active peripheral
        if self.role.get() == SpiRole::SpiMaster {
            let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };
            let peripheral_number: u32 = match peripheral {
                Peripheral::Peripheral0 => spi_consts::mr::PCS0,
                Peripheral::Peripheral1 => spi_consts::mr::PCS1,
                Peripheral::Peripheral2 => spi_consts::mr::PCS2,
                Peripheral::Peripheral3 => spi_consts::mr::PCS3,
            };
            let mut mr = regs.mr.get();
            mr = (mr & !spi_consts::mr::PCS_MASK) | peripheral_number;
            regs.mr.set(mr);
        }
    }

    /// Returns the currently active peripheral
    pub fn get_active_peripheral(&self) -> Peripheral {
        if self.role.get() == SpiRole::SpiMaster {
            let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

            let mr = regs.mr.get();
            let peripheral_number = mr & (spi_consts::mr::PCS_MASK);

            match peripheral_number {
                spi_consts::mr::PCS0 => Peripheral::Peripheral0,
                spi_consts::mr::PCS1 => Peripheral::Peripheral1,
                spi_consts::mr::PCS2 => Peripheral::Peripheral2,
                spi_consts::mr::PCS3 => Peripheral::Peripheral3,
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
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => regs.csr0.get(),
            Peripheral::Peripheral1 => regs.csr1.get(),
            Peripheral::Peripheral2 => regs.csr2.get(),
            Peripheral::Peripheral3 => regs.csr3.get(),
        }
    }
    /// Sets the Chip Select Register (CSR) of the active peripheral
    /// (CSR0, CSR1, CSR2, or CSR3).
    fn write_active_csr(&self, value: u32) {
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => regs.csr0.set(value),
            Peripheral::Peripheral1 => regs.csr1.set(value),
            Peripheral::Peripheral2 => regs.csr2.set(value),
            Peripheral::Peripheral3 => regs.csr3.set(value),
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
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };
        let sr = regs.sr.get();

        self.slave_client.get().map(|client| {
            if (sr & spi_consts::sr::NSSR) != 0 {
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
    fn read_write_bytes(&self,
                        write_buffer: Option<&'static mut [u8]>,
                        read_buffer: Option<&'static mut [u8]>,
                        len: usize)
                        -> ReturnCode {
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
            self.transfers_in_progress.set(self.transfers_in_progress.get() + 1);
            self.dma_write.get().map(move |write| {
                write.enable();
                write.do_xfer(DMAPeripheral::SPI_TX, wbuf, count);
            });
        });

        // Only setup the RX channel if we were passed a read_buffer inside
        // of the option. `map()` checks this for us.
        read_buffer.map(|rbuf| {
            self.transfers_in_progress.set(self.transfers_in_progress.get() + 1);
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
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        let tdr = (out_byte as u32) & spi_consts::tdr::TD;
        // Wait for data to leave TDR and enter serializer, so TDR is free
        // for this next byte
        while (regs.sr.get() & spi_consts::sr::TDRE) == 0 {}
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
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        self.write_byte(val);
        // Wait for receive data register full
        while (regs.sr.get() & spi_consts::sr::RDRF) == 0 {}
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
    fn read_write_bytes(&self,
                        write_buffer: &'static mut [u8],
                        read_buffer: Option<&'static mut [u8]>,
                        len: usize)
                        -> ReturnCode {
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
        csr |= spi_consts::csr::CSAAT;
        self.write_active_csr(csr);
    }

    fn release_low(&self) {
        let mut csr = self.read_active_csr();
        csr &= !spi_consts::csr::CSAAT;
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
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };
        regs.tdr.set(write_byte as u32);
    }

    fn read_write_bytes(&self,
                        write_buffer: Option<&'static mut [u8]>,
                        read_buffer: Option<&'static mut [u8]>,
                        len: usize)
                        -> ReturnCode {
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

        self.transfers_in_progress.set(self.transfers_in_progress.get() - 1);

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
                    self.client
                        .get()
                        .map(|cb| {
                            txbuf.map(|txbuf| {
                                cb.read_write_done(txbuf, rxbuf, len);
                            });
                        });
                }
                SpiRole::SpiSlave => {
                    self.slave_client
                        .get()
                        .map(|cb| { cb.read_write_done(txbuf, rxbuf, len); });
                }
            }
        }
    }
}

interrupt_handler!(spi_interrupt_handler, SPI);
