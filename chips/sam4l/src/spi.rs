use core::cell::Cell;
use core::cmp;
use core::mem;

use dma::DMAChannel;
use dma::DMAClient;
use dma::DMAPeripheral;

use kernel::common::volatile_cell::VolatileCell;

use kernel::hil::spi;
use kernel::hil::spi::ClockPhase;
use kernel::hil::spi::ClockPolarity;
use kernel::hil::spi::SpiMasterClient;
use pm;

/// Implementation of DMA-based SPI master communication for
/// the Atmel SAM4L CortexM4 microcontroller.
/// Authors: Sam Crow <samcrow@uw.edu>
///          Philip Levis <pal@cs.stanford.edu>
///
// Driver for the SPI hardware (separate from the USARTS),
// described in chapter 26 of the datasheet
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

const SPI_BASE: u32 = 0x40008000;

/// Values for selected peripherals
#[derive(Copy,Clone)]
pub enum Peripheral {
    Peripheral0,
    Peripheral1,
    Peripheral2,
    Peripheral3,
}

///
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
        }
    }

    pub fn enable(&self) {
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        unsafe {
            pm::enable_clock(pm::Clock::PBA(pm::PBAClock::SPI));
        }
        regs.cr.set(0b1);
    }

    pub fn disable(&self) {
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        self.dma_read.get().map(|read| read.disable());
        self.dma_write.get().map(|write| write.disable());
        regs.cr.set(0b10);
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
        let clock = unsafe { pm::get_system_frequency() };

        if real_rate < 188235 {
            real_rate = 188235;
        }
        if real_rate > clock {
            real_rate = clock;
        }

        // Divide and truncate, resulting in a n value that might be too low
        let mut scbr = clock / real_rate;
        // If the division was not exact, increase the n to get a slower baud rate
        if clock % rate != 0 {
            scbr += 1;
        }
        let mut csr = self.read_active_csr();
        let csr_mask: u32 = 0xFFFF00FF;
        // Clear, then write CSR bits
        csr &= csr_mask;
        csr |= (scbr & 0xFF) << 8;
        self.write_active_csr(csr);
        clock / scbr
    }

    pub fn get_baud_rate(&self) -> u32 {
        let clock = 48000000;
        let scbr = (self.read_active_csr() >> 8) & 0xFF;
        clock / scbr
    }

    pub fn set_active_peripheral(&self, peripheral: Peripheral) {
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        let peripheral_number: u32 = match peripheral {
            Peripheral::Peripheral0 => 0b1110,
            Peripheral::Peripheral1 => 0b1101,
            Peripheral::Peripheral2 => 0b1011,
            Peripheral::Peripheral3 => 0b0111,
        };
        let mut mr = regs.mr.get();
        let pcs_mask: u32 = 0xFFF0FFFF;
        mr &= pcs_mask;
        mr |= peripheral_number << 16;
        regs.mr.set(mr);
    }

    /// Returns the currently active peripheral
    pub fn get_active_peripheral(&self) -> Peripheral {
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        let mr = regs.mr.get();
        let pcs = (mr >> 16) & 0xF;
        // Split into bits for matching
        match pcs {
            0b1110 => Peripheral::Peripheral0,
            0b1101 => Peripheral::Peripheral1,
            0b1011 => Peripheral::Peripheral2,
            0b0111 => Peripheral::Peripheral3,
            _ => {
                // Invalid configuration
                // ???
                Peripheral::Peripheral0
            }
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
}

impl spi::SpiMaster for Spi {
    type ChipSelect = u8;

    fn set_client(&self, client: &'static SpiMasterClient) {
        self.client.set(Some(client));
    }

    /// By default, initialize SPI to operate at 40KHz, clock is
    /// idle on low, and sample on the leading edge.
    fn init(&self) {
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        self.enable_clock();
        regs.cr.set(1 << 24);

        let mut mode = regs.mr.get();
        mode |= 1; // Enable master mode
        mode |= 1 << 4; // Disable mode fault detection (open drain outputs not supported)
        regs.mr.set(mode);
    }

    fn is_busy(&self) -> bool {
        self.transfers_in_progress.get() != 0
    }

    /// Write a byte to the SPI and discard the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn write_byte(&self, out_byte: u8) {
        let regs: &mut SpiRegisters = unsafe { mem::transmute(self.registers) };

        if self.is_busy() {
            //           return;
        }

        let tdr = out_byte as u32;
        // Wait for data to leave TDR and enter serializer, so TDR is free
        // for this next byte
        while (regs.sr.get() & (1 << 1)) == 0 {}
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

        if self.is_busy() {
            //          return 0;
        }
        self.write_byte(val);
        // Wait for receive data register full
        while (regs.sr.get() & 1) != 1 {}
        // Return read value
        regs.rdr.get() as u8
    }

    /// Asynchronous buffer read/write of SPI.
    /// write_buffer must  be Some; read_buffer may be None;
    /// if read_buffer is Some, then length of read/write is the
    /// minimum of two buffer lengths; returns true if operation
    /// starts (will receive callback through SpiMasterClient), returns
    /// false if the operation does not start.
    // The write buffer has to be mutable because it's passed back to
    // the caller, and the caller may want to be able write into it.
    fn read_write_bytes(&self,
                        write_buffer: &'static mut [u8],
                        read_buffer: Option<&'static mut [u8]>,
                        len: usize)
                        -> bool {
        self.enable();
        // If busy, don't start.
        if self.is_busy() {
            return false;
        }

        // We will have at least a write transfer in progress
        self.transfers_in_progress.set(1);

        let read_len = match read_buffer {
            Some(ref buf) => buf.len(),
            None => 0,
        };
        let write_len = write_buffer.len();
        let buflen = if !read_buffer.is_some() {
            write_len
        } else {
            cmp::min(read_len, write_len)
        };
        let count = cmp::min(buflen, len);
        self.dma_length.set(count);

        // The ordering of these operations matters.
        // For transfers 4 bytes or longer, this will work as expected.
        // For shorter transfers, the first byte will be missing.
        self.dma_write.get().map(move |write| {
            write.enable();
            write.do_xfer(DMAPeripheral::SPI_TX, write_buffer, count);
        });

        // Only setup the RX channel if we were passed a read_buffer inside
        // of the option. `map()` checks this for us.
        read_buffer.map(|rbuf| {
            self.transfers_in_progress.set(2);
            self.dma_read.get().map(move |read| {
                read.enable();
                read.do_xfer(DMAPeripheral::SPI_RX, rbuf, count);
            });
        });
        true
    }

    fn set_rate(&self, rate: u32) -> u32 {
        self.set_baud_rate(rate)
    }

    fn get_rate(&self) -> u32 {
        self.get_baud_rate()
    }

    fn set_clock(&self, polarity: ClockPolarity) {
        let mut csr = self.read_active_csr();
        match polarity {
            ClockPolarity::IdleHigh => csr |= 1,
            ClockPolarity::IdleLow => csr &= 0xFFFFFFFE,
        };
        self.write_active_csr(csr);
    }

    fn get_clock(&self) -> ClockPolarity {
        let csr = self.read_active_csr();
        let polarity = csr & 0x1;
        match polarity {
            0 => ClockPolarity::IdleLow,
            _ => ClockPolarity::IdleHigh,
        }
    }

    fn set_phase(&self, phase: ClockPhase) {
        let mut csr = self.read_active_csr();
        match phase {
            ClockPhase::SampleLeading => csr |= 1 << 1,
            ClockPhase::SampleTrailing => csr &= 0xFFFFFFFD,
        };
        self.write_active_csr(csr);
    }

    fn get_phase(&self) -> ClockPhase {
        let csr = self.read_active_csr();
        let phase = (csr >> 1) & 0x1;
        match phase {
            0 => ClockPhase::SampleTrailing,
            _ => ClockPhase::SampleLeading,
        }
    }

    fn hold_low(&self) {
        let mut csr = self.read_active_csr();
        csr |= 1 << 3;
        self.write_active_csr(csr);
    }

    fn release_low(&self) {
        let mut csr = self.read_active_csr();
        csr &= 0xFFFFFFF7;
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
            self.client
                .get()
                .map(|cb| { txbuf.map(|txbuf| { cb.read_write_done(txbuf, rxbuf, len); }); });
        }
    }
}
