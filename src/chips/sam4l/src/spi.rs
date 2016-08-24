
use core::cell::Cell;
use core::cmp;
use dma::DMAChannel;
use dma::DMAClient;
use dma::DMAPeripheral;
use helpers::*;

use hil::spi_master;
use hil::spi_master::ClockPhase;
use hil::spi_master::ClockPolarity;
use hil::spi_master::SpiCallback;
use pm;

/// Implementation of DMA-based SPI master communication for
/// the Atmel SAM4L CortexM4 microcontroller.
/// Authors: Sam Crow <samcrow@uw.edu>
///          Philip Levis <pal@cs.stanfored.edu>
///
// Driver for the SPI hardware (seperate from the USARTS),
// described in chapter 26 of the datasheet
/// The registers used to interface with the hardware
#[repr(C, packed)]
struct SpiRegisters {
    cr: u32, // 0x0
    mr: u32, // 0x4
    rdr: u32, // 0x8
    tdr: u32, // 0xC
    sr: u32, // 0x10
    ier: u32, // 0x14
    idr: u32, // 0x18
    imr: u32, // 0x1C
    reserved0: [u32; 4], // 0x20, 0x24, 0x28, 0x2C
    csr0: u32, // 0x30
    csr1: u32, // 0x34
    csr2: u32, // 0x38
    csr3: u32, // 0x3C
    reserved1: [u32; 41], // 0x40 - 0xE0
    wpcr: u32, // 0xE4
    wpsr: u32, // 0xE8
    reserved2: [u32; 3], // 0xEC - 0xF4
    features: u32, // 0xF8
    version: u32, // 0xFC
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
    regs: *mut SpiRegisters,
    callback: Option<&'static SpiCallback>,
    dma_read: Option<&'static mut DMAChannel>,
    dma_write: Option<&'static mut DMAChannel>,
    // keep track of which // interrupts are pending in order
    // to correctly issue completion event only after both complete.
    reading: Cell<bool>,
    writing: Cell<bool>,
    read_buffer: Option<&'static mut [u8]>,
    write_buffer: Option<&'static mut [u8]>,
    dma_length: Cell<usize>,
}

pub static mut SPI: Spi = Spi::new();

impl Spi {
    /// Creates a new SPI object, with peripheral 0 selected
    pub const fn new() -> Spi {
        Spi {
            regs: SPI_BASE as *mut SpiRegisters,
            callback: None,
            dma_read: None,
            dma_write: None,
            reading: Cell::new(false),
            writing: Cell::new(false),
            read_buffer: None,
            write_buffer: None,
            dma_length: Cell::new(0),
        }
    }

    pub fn enable(&self) {
        unsafe {
            pm::enable_clock(pm::Clock::PBA(pm::PBAClock::SPI));
        }
        // self.dma_read.as_ref().map(|read| read.enable());
        // self.dma_write.as_ref().map(|write| write.enable());
        unsafe {
            volatile_store(&mut (*self.regs).cr, 0b1);
        }
    }

    pub fn disable(&self) {
        self.dma_read.as_ref().map(|read| read.disable());
        self.dma_write.as_ref().map(|write| write.disable());
        unsafe {
            volatile_store(&mut (*self.regs).cr, 0b10);
        }
    }

    /// Sets the approximate baud rate for the active peripheral,
    /// and return the actual baud rate set.
    ///
    /// Since the only supported baud rates are 48 MHz / n where n
    /// is an integer from 1 to 255, the exact baud rate may not
    /// be available. In that case, the next lower baud rate will
    /// be selected.
    ///
    /// The lowest available baud rate is 188235 baud. If the
    /// requested rate is lower, 188235 baud will be selected.
    pub fn set_baud_rate(&self, rate: u32) -> u32 {
        // Main clock frequency
        let mut real_rate = rate;
        let clock = 48000000;

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
        let peripheral_number: u32 = match peripheral {
            Peripheral::Peripheral0 => 0b1110,
            Peripheral::Peripheral1 => 0b1101,
            Peripheral::Peripheral2 => 0b1011,
            Peripheral::Peripheral3 => 0b0111,
        };
        let mut mr = unsafe { volatile_load(&(*self.regs).mr) };
        let pcs_mask: u32 = 0xFFF0FFFF;
        mr &= pcs_mask;
        mr |= peripheral_number << 16;
        unsafe {
            volatile_store(&mut (*self.regs).mr, mr);
        }
    }

    /// Returns the currently active peripheral
    pub fn get_active_peripheral(&self) -> Peripheral {
        let mr = unsafe { volatile_load(&(*self.regs).mr) };
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
        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => unsafe { volatile_load(&(*self.regs).csr0) },
            Peripheral::Peripheral1 => unsafe { volatile_load(&(*self.regs).csr1) },
            Peripheral::Peripheral2 => unsafe { volatile_load(&(*self.regs).csr2) },
            Peripheral::Peripheral3 => unsafe { volatile_load(&(*self.regs).csr3) },
        }
    }
    /// Sets the Chip Select Register (CSR) of the active peripheral
    /// (CSR0, CSR1, CSR2, or CSR3).
    fn write_active_csr(&self, value: u32) {
        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => unsafe { volatile_store(&mut (*self.regs).csr0, value) },
            Peripheral::Peripheral1 => unsafe { volatile_store(&mut (*self.regs).csr1, value) },
            Peripheral::Peripheral2 => unsafe { volatile_store(&mut (*self.regs).csr2, value) },
            Peripheral::Peripheral3 => unsafe { volatile_store(&mut (*self.regs).csr3, value) },
        };
    }

    /// Set the DMA channels used for reading and writing.
    pub fn set_dma(&mut self, read: &'static mut DMAChannel, write: &'static mut DMAChannel) {
        self.dma_read = Some(read);
        self.dma_write = Some(write);
    }

    fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(pm::Clock::PBA(pm::PBAClock::SPI));
        }
    }
}

impl spi_master::SpiMaster for Spi {
    /// By default, initialize SPI to operate at 40KHz, clock is
    /// idle on low, and sample on the leading edge.
    fn init(&mut self, callback: &'static SpiCallback) {
        self.enable_clock();

        self.callback = Some(callback);
        unsafe { volatile_store(&mut (*self.regs).cr, 1 << 24) };

        let mut mode = unsafe { volatile_load(&(*self.regs).mr) };
        mode |= 1; // Enable master mode
        mode |= 1 << 4; // Disable mode fault detection (open drain outputs not supported)
        unsafe { volatile_store(&mut (*self.regs).mr, mode) };
    }

    fn is_busy(&self) -> bool {
        self.reading.get() || self.writing.get()
    }


    /// Write a byte to the SPI and discard the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn write_byte(&self, out_byte: u8) {
        if self.reading.get() || self.writing.get() {
            //           return;
        }
        let tdr = out_byte as u32;
        // Wait for data to leave TDR and enter serializer, so TDR is free
        // for this next byte
        while (unsafe { volatile_load(&(*self.regs).sr) } & 1 << 1) == 0 {}
        unsafe { volatile_store(&mut (*self.regs).tdr, tdr) };
    }

    /// Write 0 to the SPI and return the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn read_byte(&self) -> u8 {
        self.read_write_byte(0)
    }

    /// Write a byte to the SPI and return the read; if an
    /// asynchronous operation is outstanding, do nothing.
    fn read_write_byte(&self, val: u8) -> u8 {
        if self.reading.get() || self.writing.get() {
            //          return 0;
        }
        self.write_byte(val);
        // Wait for receive data register full
        while (unsafe { volatile_load(&(*self.regs).sr) } & 1) != 1 {}
        // Return read value
        unsafe { volatile_load(&(*self.regs).rdr) as u8 }
    }

    /// Asynchonous buffer read/write of SPI.
    /// write_buffer must  be Some; read_buffer may be None;
    /// if read_buffer is Some, then length of read/write is the
    /// minimum of two buffer lengths; returns true if operation
    /// starts (will receive callback through SpiClient), returns
    /// false if the operation does not start.
    // The write buffer has to be mutable because it's passed back to
    // the caller, and the caller may want to be able write into it.
    fn read_write_bytes(&self,
                        write_buffer: Option<&'static mut [u8]>,
                        read_buffer: Option<&'static mut [u8]>,
                        len: usize)
                        -> bool {
        self.enable();
        let writing = write_buffer.is_some();
        let reading = read_buffer.is_some();
        // If there is no write buffer, or busy, then don't start.
        // Need to check self.reading as well as self.writing in case
        // write interrupt comes back first.
        if !writing || self.reading.get() || self.writing.get() {
            panic!("Busy {} {} {}",
                   writing,
                   self.reading.get(),
                   self.writing.get());
        }

        // Need to mark if reading or writing so we correctly
        // regenerate Options on callback
        self.writing.set(writing);
        self.reading.set(reading);

        let read_len = match read_buffer {
            Some(ref buf) => buf.len(),
            None => 0,
        };
        let write_len = match write_buffer {
            Some(ref buf) => buf.len(),
            None => 0,
        };
        let buflen = if !reading {
            write_len
        } else {
            cmp::min(read_len, write_len)
        };
        let count = cmp::min(buflen, len);
        self.dma_length.set(count);
        // The ordering of these operations matters; if you enable then
        // perform the operation, you can read a byte early on the SPI data register
        self.dma_write.as_ref().map(|write| {
            write.enable();
            write.do_xfer(DMAPeripheral::SPI_TX, write_buffer.unwrap(), count)
        });
        if reading {
            self.dma_read.as_ref().map(|read| {
                read.enable();
                read.do_xfer(DMAPeripheral::SPI_RX, read_buffer.unwrap(), count)
            });
        }

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

    fn set_chip_select(&self, cs: u8) -> bool {
        let peripheral_number = match cs {
            0 => Peripheral::Peripheral0,
            1 => Peripheral::Peripheral1,
            2 => Peripheral::Peripheral2,
            3 => Peripheral::Peripheral3,
            _ => return false,
        };
        self.set_active_peripheral(peripheral_number);
        true
    }

    fn get_chip_select(&self) -> u8 {
        let mr = unsafe { volatile_load(&(*self.regs).mr) };
        let cs = (mr >> 16) & 0xF;
        match cs {
            0b0000 => 0,
            0b0001 => 1,
            0b0011 => 2,
            0b0111 => 3,
            _ => 0,
        }
    }

    fn clear_chip_select(&self) {
        unsafe { volatile_store(&mut (*self.regs).cr, 1 << 24) };
    }
}

impl DMAClient for Spi {
    fn xfer_done(&mut self, pid: usize) {
        // I don't know if there are ordering guarantees on the read and
        // write interrupts, guessing not, so issue the callback when both
        // reading and writing are complete. In practice it seems like
        // the read interrupt happens second. -pal
        //
        // The disable calls are commented out because executing them
        // causes subsequent operations to fail. The call to read_write_bytes
        // calls enable(), so I don't know why. -pal
        if pid == 4 {
            // SPI RX
            self.reading.set(false);
            let buf = match self.dma_read.as_mut() {
                Some(dma) => {
                    let buf = dma.abort_xfer();
                    dma.disable();
                    buf
                }
                None => None,
            };
            self.read_buffer = buf;
            if !self.reading.get() && !self.writing.get() {
                let rb = self.read_buffer.take();
                let wb = self.write_buffer.take();
                let len = self.dma_length.get();
                self.dma_length.set(0);
                self.callback.as_ref().map(|cb| cb.read_write_done(wb, rb, len));
            }
        } else if pid == 22 {
            // SPI TX
            self.writing.set(false);
            let buf = match self.dma_write.as_mut() {
                Some(dma) => {
                    let buf = dma.abort_xfer();
                    dma.disable();
                    buf
                }
                None => None,
            };
            self.write_buffer = buf;
            if !self.reading.get() && !self.writing.get() {
                let rb = self.read_buffer.take();
                let wb = self.write_buffer.take();
                let len = self.dma_length.get();
                self.dma_length.set(0);
                self.callback.as_ref().map(|cb| cb.read_write_done(wb, rb, len));
            }
        }
    }
}
