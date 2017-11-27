//! Implementation of the SAM4L CRCCU.
//!
//! See datasheet section "41. Cyclic Redundancy Check Calculation Unit (CRCCU)".
//!
//! The SAM4L can compute CRCs using three different polynomials:
//!
//!   * `0x04C11DB7` (as used in "CRC-32"; Atmel calls this "CCIT8023")
//!   * `0x1EDC6F41` (as used in "CRC-32C"; Atmel calls this "CASTAGNOLI")
//!   * `0x1021`     (as used in "CRC-16-CCITT"; Atmel calls this "CCIT16")
//!
//! (The integers above give each polynomial from most-significant to least-significant
//! bit, except that the most significant bit is omitted because it is always 1.)
//!
//! In all cases, the unit consumes each input byte from LSB to MSB.
//!
//! Note that the chip's behavior differs from some "standard" CRC algorithms,
//! which may do some of these things:
//!
//!   * Consume input from MSB to LSB (CRC-16-CCITT?)
//!   * Bit-reverse and then bit-invert the output (CRC-32)
//!
//! # Notes
//!
//! This [calculator](http://www.zorc.breitbandkatze.de/crc.html) may be used to
//! generate CRC values.  To match the output of the SAM4L, the parameters must
//! be set as follows:
//!
//!   * Final XOR value: 0  (equivalent to no final XOR)
//!   * reverse data bytes: yes
//!   * reverse CRC result before Final XOR: no
//!
//! For one example, the SAM4L calculates 0x1541 for "ABCDEFG" when using
//! polynomial 0x1021.

// Infelicities:
//
// - As much as 512 bytes of RAM is wasted to allow runtime alignment of the
//   CRCCU Descriptor.  Reliable knowledge of kernel alignment might allow this
//   to be done statically.
//
// - CRC performance would be improved by using transfer-widths larger than Byte,
//   but it is not clear in what cases that is possible.

// TODO:
//
// - Chain computations to permit arbitrary-size computations, or at least
//   publish the max buffer size the unit can handle.
//
// - Support continuous-mode CRC

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::hil::crc::{self, CrcAlg};
use pm::{Clock, HSBClock, PBBClock, enable_clock, disable_clock};

// A memory-mapped register
struct Reg(*mut u32);

impl Reg {
    fn read(self) -> u32 {
        unsafe { ::core::ptr::read_volatile(self.0) }
    }

    fn write(self, n: u32) {
        unsafe {
            ::core::ptr::write_volatile(self.0, n);
        }
    }
}

// Base address of CRCCU registers.  See "7.1 Product Mapping"
const CRCCU_BASE: u32 = 0x400A4000;

// The following macro expands a list of expressions like this:
//
//    { 0x00, "Descriptor Base Register", DSCR, "RW" },
//
// into a series of items like this:
//
//    #[allow(dead_code)]
//    const DSCR: Reg = Reg((CRCCU_BASE + 0x00) as *mut u32);

macro_rules! registers {
    [ $( { $offset:expr, $description:expr, $name:ident, $access:expr } ),* ] => {
        $( #[allow(dead_code)]
           const $name: Reg = Reg((CRCCU_BASE + $offset) as *mut u32); )*
    };
}

// CRCCU Registers (from Table 41.1 in Section 41.6):
registers![
    // Address of descriptor (512-byte aligned)
    { 0x00, "Descriptor Base Register", DSCR, "RW" },
    // Write a one to enable DMA channel
    { 0x08, "DMA Enable Register", DMAEN, "W" },
    // Write a one to disable DMA channel
    { 0x0C, "DMA Disable Register", DMADIS, "W" },
    // DMA channel enabled?
    { 0x10, "DMA Status Register", DMASR, "R" },
    // Write a one to enable DMA interrupt
    { 0x14, "DMA Interrupt Enable Register", DMAIER, "W" },
    // Write a one to disable DMA interrupt
    { 0x18, "DMA Interrupt Disable Register", DMAIDR, "W" },
    // DMA interrupt enabled?
    { 0x1C, "DMA Interrupt Mask Register", DMAIMR, "R" },
    // DMA transfer completed? (cleared when read)
    { 0x20, "DMA Interrupt Status Register", DMAISR, "R" },
    // Write a one to reset SR
    { 0x34, "Control Register", CR, "W" },
    // Bandwidth divider, Polynomial type, Compare?, Enable?
    { 0x38, "Mode Register", MR, "RW" },
    // CRC result (unreadable if MR.COMPARE=1)
    { 0x3C, "Status Register", SR, "R" },
    // Write one to set IMR.ERR bit (zero no effect)
    { 0x40, "Interrupt Enable Register", IER, "W" },
    // Write zero to clear IMR.ERR bit (one no effect)
    { 0x44, "Interrupt Disable Register", IDR, "W" },
    // If IMR.ERR bit is set, error-interrupt (for compare) is enabled
    { 0x48, "Interrupt Mask Register", IMR, "R" },
    // CRC error (for compare)? (cleared when read)
    { 0x4C, "Interrupt Status Register", ISR, "R" },
    // 12 low-order bits: version of this module.  = 0x00000202
    { 0xFC, "Version Register", VERSION, "R" }
];

// CRCCU Descriptor (from Table 41.2 in Section 41.6):
#[repr(C, packed)]
struct Descriptor {
    addr: u32, // Transfer Address Register (RW): Address of memory block to compute
    ctrl: TCR, // Transfer Control Register (RW): IEN, TRWIDTH, BTSIZE
    _res: [u32; 2],
    crc: u32, // Transfer Reference Register (RW): Reference CRC (for compare mode)
}

// Transfer Control Register (see Section 41.6.18)
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct TCR(u32);

impl TCR {
    const fn new(enable_interrupt: bool, trwidth: TrWidth, btsize: u16) -> Self {
        TCR((!enable_interrupt as u32) << 27 | (trwidth as u32) << 24 | (btsize as u32))
    }

    const fn default() -> Self {
        Self::new(false, TrWidth::Byte, 0)
    }

    fn interrupt_enabled(self) -> bool {
        (self.0 & (1 << 27)) == 0
    }

    #[allow(dead_code)]
    fn get_btsize(self) -> u16 {
        (self.0 & 0xffff) as u16
    }
}

#[derive(Copy, Clone)]
enum Polynomial {
    CCIT8023, // Polynomial 0x04C11DB7
    CASTAGNOLI, // Polynomial 0x1EDC6F41
    CCIT16, // Polynomial 0x1021
}

fn poly_for_alg(alg: CrcAlg) -> Polynomial {
    match alg {
        CrcAlg::Crc32 => Polynomial::CCIT8023,
        CrcAlg::Crc32C => Polynomial::CASTAGNOLI,
        CrcAlg::Sam4L16 => Polynomial::CCIT16,
        CrcAlg::Sam4L32 => Polynomial::CCIT8023,
        CrcAlg::Sam4L32C => Polynomial::CASTAGNOLI,
    }
}

fn post_process(result: u32, alg: CrcAlg) -> u32 {
    match alg {
        CrcAlg::Crc32 => reverse_and_invert(result),
        CrcAlg::Crc32C => reverse_and_invert(result),
        CrcAlg::Sam4L16 => result,
        CrcAlg::Sam4L32 => result,
        CrcAlg::Sam4L32C => result,
    }
}

fn reverse_and_invert(n: u32) -> u32 {
    let mut out: u32 = 0;

    // Bit-reverse
    for j in 0..32 {
        let i = 31 - j;
        out |= ((n & (1 << i)) >> i) << j;
    }

    // Bit-invert
    out ^= 0xffffffff;

    out
}

/// Transfer width for DMA
pub enum TrWidth {
    Byte,
    HalfWord,
    Word,
}

// Mode Register (see Section 41.6.10)
struct Mode(u32);

impl Mode {
    fn new(divider: u8, ptype: Polynomial, compare: bool, enable: bool) -> Self {
        Mode((((divider & 0x0f) as u32) << 4) | (ptype as u32) << 2 | (compare as u32) << 1 |
             (enable as u32))
    }
    fn disabled() -> Self {
        Mode::new(0, Polynomial::CCIT8023, false, false)
    }
}

#[derive(Copy, Clone, PartialEq)]
enum State {
    Invalid,
    Initialized,
    Enabled,
}

/// State for managing the CRCCU
pub struct Crccu<'a> {
    client: Option<&'a crc::Client>,
    state: Cell<State>,
    alg: Cell<CrcAlg>,

    // Guaranteed room for a Descriptor with 512-byte alignment.
    // (Can we do this statically instead?)
    descriptor_space: [u8; DSCR_RESERVE],
}

const DSCR_RESERVE: usize = 512 + 5 * 4;

impl<'a> Crccu<'a> {
    const fn new() -> Self {
        Crccu {
            client: None,
            state: Cell::new(State::Invalid),
            alg: Cell::new(CrcAlg::Crc32C),
            descriptor_space: [0; DSCR_RESERVE],
        }
    }

    fn init(&self) {
        if self.state.get() == State::Invalid {
            self.set_descriptor(0, TCR::default(), 0);
            self.state.set(State::Initialized);
        }
    }

    /// Enable the CRCCU's clocks and interrupt
    pub fn enable(&self) {
        if self.state.get() != State::Enabled {
            self.init();
            unsafe {
                // see "10.7.4 Clock Mask"
                enable_clock(Clock::HSB(HSBClock::CRCCU));
                enable_clock(Clock::PBB(PBBClock::CRCCU));
            }
            self.state.set(State::Enabled);
        }
    }

    /// Disable the CRCCU's clocks and interrupt
    pub fn disable(&self) {
        if self.state.get() == State::Enabled {
            unsafe {
                disable_clock(Clock::PBB(PBBClock::CRCCU));
                disable_clock(Clock::HSB(HSBClock::CRCCU));
            }
            self.state.set(State::Initialized);
        }
    }

    /// Set a client to receive results from the CRCCU
    pub fn set_client(&mut self, client: &'a crc::Client) {
        self.client = Some(client);
    }

    /// Get the client currently receiving results from the CRCCU
    pub fn get_client(&self) -> Option<&'a crc::Client> {
        self.client
    }

    fn set_descriptor(&self, addr: u32, ctrl: TCR, crc: u32) {
        let d = unsafe { &mut *self.descriptor() };
        d.addr = addr;
        d.ctrl = ctrl;
        d.crc = crc;
    }

    fn get_tcr(&self) -> TCR {
        let d = unsafe { &*self.descriptor() };
        d.ctrl
    }

    // Dynamically calculate the 512-byte-aligned location for Descriptor
    fn descriptor(&self) -> *mut Descriptor {
        let s = &self.descriptor_space as *const [u8; DSCR_RESERVE] as u32;
        let t = s % 512;
        let u = 512 - t;
        let d = s + u;
        return d as *mut Descriptor;
    }

    /// Handle an interrupt from the CRCCU
    pub fn handle_interrupt(&mut self) {
        if ISR.read() & 1 == 1 {
            // A CRC error has occurred
        }

        if DMAISR.read() & 1 == 1 {
            // A DMA transfer has completed

            if self.get_tcr().interrupt_enabled() {
                if let Some(client) = self.get_client() {
                    let result = post_process(SR.read(), self.alg.get());
                    client.receive_result(result);
                }

                // Disable the unit
                MR.write(Mode::disabled().0);

                // Reset CTRL.IEN (for our own statekeeping)
                self.set_descriptor(0, TCR::default(), 0);

                // Disable DMA interrupt
                DMAIDR.write(1);

                // Disable DMA channel
                DMADIS.write(1);
            }
        }
    }
}

// Implement the generic CRC interface with the CRCCU
impl<'a> crc::CRC for Crccu<'a> {
    fn get_version(&self) -> u32 {
        VERSION.read()
    }

    fn compute(&self, data: &[u8], alg: CrcAlg) -> ReturnCode {
        self.init();

        if self.get_tcr().interrupt_enabled() {
            // A computation is already in progress
            return ReturnCode::EBUSY;
        }

        if data.len() > 2usize.pow(16) - 1 {
            // Buffer too long
            // TODO: Chain CRCCU computations to handle large buffers
            return ReturnCode::ESIZE;
        }

        self.enable();

        // Enable DMA interrupt
        DMAIER.write(1);

        // Enable error interrupt
        IER.write(1);

        // Reset intermediate CRC value
        CR.write(1);

        // Configure the data transfer
        let addr = data.as_ptr() as u32;
        let len = data.len() as u16;
        /*
        // It's not clear under what circumstances a transfer width other than Byte will work
        let tr_width = if addr % 4 == 0 && len % 4 == 0 { TrWidth::Word }
                       else { if addr % 2 == 0 && len % 2 == 0 { TrWidth::HalfWord }
                              else { TrWidth::Byte } };
        */
        let tr_width = TrWidth::Byte;
        let ctrl = TCR::new(true, tr_width, len);
        let crc = 0;
        self.set_descriptor(addr, ctrl, crc);
        DSCR.write(self.descriptor() as u32);

        // Record what algorithm was requested
        self.alg.set(alg);

        // Configure the unit to compute a checksum
        let divider = 0;
        let compare = false;
        let enable = true;
        let mode = Mode::new(divider, poly_for_alg(alg), compare, enable);
        MR.write(mode.0);

        // Enable DMA channel
        DMAEN.write(1);

        return ReturnCode::SUCCESS;
    }

    fn disable(&self) {
        Crccu::disable(self);
    }
}

/// Static state to manage the CRCCU
pub static mut CRCCU: Crccu<'static> = Crccu::new();
