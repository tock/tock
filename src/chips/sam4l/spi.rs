use helpers::*;
use core::intrinsics;
use core::cmp;

use pm;
use hil::spi_master;
use hil::spi_master::Reader;

// Driver for the SPI hardware (seperate from the USARTS, described in chapter 26 of the
// datasheet)

/// The registers used to interface with the hardware
#[repr(C, packed)]
struct SPIRegisters {
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

const BASE_ADDRESS: u32 = 0x40008000;

/// Values for selected peripherals
#[derive(Copy,Clone)]
pub enum Peripheral {
    Peripheral0,
    Peripheral1,
    Peripheral2,
    Peripheral3,
}

///
/// SPI implementation using the SPI hardware
/// Supports four peripherals. Each peripheral can have different settings.
///
/// The init, read, and write methods act on the currently selected peripheral.
/// The init method can be safely called more than once to configure different peripherals:
///
///     spi.set_active_peripheral(Peripheral::Peripheral0);
///     spi.init(/* Parameters for peripheral 0 */);
///     spi.set_active_peripheral(Peripheral::Peripheral1);
///     spi.init(/* Parameters for peripheral 1 */);
///
pub struct SPI {
    /// Registers
    regs: &'static mut SPIRegisters,
    /// Client
    client: Option<&'static mut Reader>,
}

impl SPI {
    /// Creates a new SPI object, with peripheral 0 selected
    pub fn new() -> SPI {
        // Enable clock
        unsafe { pm::enable_clock(pm::Clock::PBA(pm::PBAClock::SPI)); }
        SPI {
            regs: unsafe{ intrinsics::transmute(BASE_ADDRESS) },
            client: None,
        }
    }

    /// Sets the approximate baud rate for the active peripheral
    ///
    /// Since the only supported baud rates are 48 MHz / n where n is an integer from 1 to 255,
    /// the exact baud rate may not be available. In that case, the next lower baud rate will be
    /// selected.
    ///
    /// The lowest available baud rate is 188235 baud. If the requested rate is lower,
    /// 188235 baud will be selected.
    pub fn set_baud_rate(&mut self, mut rate: u32) {
        // Main clock frequency
        let clock = 48000000;

        if rate < 188235 {
            rate = 188235;
        }
        if rate > clock {
            rate = clock;
        }

        // Divide and truncate, resulting in a n value that might be too low
        let mut scbr = clock / rate;
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
    }

    /// Returns the currently active peripheral
    pub fn get_active_peripheral(&self) -> Peripheral {
        let mr = volatile_load(&self.regs.mr);
        let pcs = (mr >> 16) & 0xF;
        // Split into bits for matching
        let pcs_bits = ((pcs >> 3) & 1, (pcs >> 2) & 1, (pcs >> 1) & 1, pcs & 1);
        match pcs_bits {
            (_, _, _, 0) => Peripheral::Peripheral0,
            (_, _, 0, 1) => Peripheral::Peripheral1,
            (_, 0, 1, 1) => Peripheral::Peripheral2,
            (0, 1, 1, 1) => Peripheral::Peripheral3,
            _ => {
                // Invalid configuration
                // ???
                Peripheral::Peripheral0
            }
        }
    }
    /// Sets the active peripheral
    pub fn set_active_peripheral(&mut self, peripheral: Peripheral) {
        let peripheral_number: u32 = match peripheral {
            Peripheral::Peripheral0 => 0b0000,
            Peripheral::Peripheral1 => 0b0001,
            Peripheral::Peripheral2 => 0b0011,
            Peripheral::Peripheral3 => 0b0111,
        };

        let mut mr = volatile_load(&self.regs.mr);
        // Clear and set MR.PCS
        let pcs_mask: u32 = 0xFFF0FFFF;
        mr &= pcs_mask;
        mr |= peripheral_number << 16;
        volatile_store(&mut self.regs.mr, mr);
    }

    /// Returns the value of CSR0, CSR1, CSR2, or CSR3, whichever corresponds to the active
    /// peripheral
    fn read_active_csr(&self) -> u32 {
        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => volatile_load(&self.regs.csr0),
            Peripheral::Peripheral1 => volatile_load(&self.regs.csr1),
            Peripheral::Peripheral2 => volatile_load(&self.regs.csr2),
            Peripheral::Peripheral3 => volatile_load(&self.regs.csr3),
        }
    }
    /// Sets the value of CSR0, CSR1, CSR2, or CSR3, whichever corresponds to the active
    /// peripheral
    fn write_active_csr(&mut self, value: u32) {
        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => volatile_store(&mut self.regs.csr0, value),
            Peripheral::Peripheral1 => volatile_store(&mut self.regs.csr1, value),
            Peripheral::Peripheral2 => volatile_store(&mut self.regs.csr2, value),
            Peripheral::Peripheral3 => volatile_store(&mut self.regs.csr3, value),
        };
    }


    /// Ends the SPI transaction by setting the slave select high
    fn end_transaction(&mut self) {
        volatile_store(&mut self.regs.cr, 1 << 24);
    }
}

impl spi_master::SPI for SPI {
    fn init(&mut self, params: spi_master::SPIParams) {
        self.client = params.client;
        self.set_baud_rate(params.baud_rate);

        let mut csr = self.read_active_csr();
        // Clock polarity
        match params.clock_polarity {
            spi_master::ClockPolarity::IdleHigh => csr |= 1, // Set bit 0
            spi_master::ClockPolarity::IdleLow => csr &= 0xFFFFFFFE, // Clear bit 0
        };
        // Clock phase
        match params.clock_phase {
            spi_master::ClockPhase::SampleLeading => csr |= 1 << 1, // Set bit 1
            spi_master::ClockPhase::SampleTrailing => csr &= 0xFFFFFFFD, // Clear bit 1
        }
        // Keep slave select active until a last transfer bit is set
        csr |= 1 << 3;
        self.write_active_csr(csr);

        // Indicate the last transfer, so that the slave select will be disabled
        volatile_store(&mut self.regs.cr, 1 << 24);

        let mut mode = volatile_load(&self.regs.mr);
        // Enable master mode
        mode |= 1;
        // Disable mode fault detection (open drain outputs do not seem to be supported)
        mode |= 1 << 4;
        volatile_store(&mut self.regs.mr, mode);
    }

    fn write_byte(&mut self, out_byte: u8, last_transfer: bool) -> u8 {
        let tdr = out_byte as u32;
        volatile_store(&mut self.regs.tdr, tdr);
        if last_transfer {
            self.end_transaction();
        }
        // Wait for receive data register full
        while (volatile_load(&self.regs.sr) & 1) != 1 {}
        // Return read value
        volatile_load(&self.regs.rdr) as u8
    }

    fn read_byte(&mut self, last_transfer: bool) -> u8 {
        self.write_byte(0, last_transfer)
    }

    fn read(&mut self, buffer: &mut [u8], last_transfer: bool) {
        // TODO: Asynchronous
        for i in 0..buffer.len() {
            // Write 0
            let tdr: u32 = 0;
            volatile_store(&mut self.regs.tdr, tdr);
            if last_transfer && i == buffer.len() - 1 {
                self.end_transaction();
            }
            // Wait for receive data register full
            while (volatile_load(&self.regs.sr) & 1) != 1 {}

            buffer[i] = volatile_load(&self.regs.rdr) as u8;
        }
        if let Some(ref mut client) = self.client {
            client.read_done();
        }
    }

    fn write(&mut self, buffer: &[u8], last_transfer: bool) {
        // TODO: Asynchronous
        for i in 0..buffer.len() {
            let tdr: u32 = buffer[i] as u32;
            // Write the value
            volatile_store(&mut self.regs.tdr, tdr);
            if last_transfer && i == buffer.len() - 1 {
                self.end_transaction();
            }
            // Wait for transmit data register empty
            while ((volatile_load(&self.regs.sr) >> 1) & 1) != 1 {}
        }
        if let Some(ref mut client) = self.client {
            client.write_done();
        }
    }

    fn read_and_write(&mut self, read_buffer: &mut [u8], write_buffer: &[u8], last_transfer: bool) {
        // TODO: Asynchronous
        let count = cmp::min(read_buffer.len(), write_buffer.len());
        for i in 0..count {
            let tdr: u32 = write_buffer[i] as u32;
            // Write the value
            volatile_store(&mut self.regs.tdr, tdr);
            if last_transfer && i == count - 1 {
                self.end_transaction();
            }
            // Wait for receive data register full
            while (volatile_load(&self.regs.sr) & 1) != 1 {}
            // Read the received value
            let read_byte = volatile_load(&self.regs.rdr) as u8;
            read_buffer[i] = read_byte;
        }
        if let Some(ref mut client) = self.client {
            client.read_write_done();
        }
    }

    fn enable(&mut self) {
        volatile_store(&mut self.regs.cr, 0b1);
    }

    fn disable(&mut self) {
        volatile_store(&mut self.regs.cr, 0b10);
    }
}
