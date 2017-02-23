//! CRCCU implementation for the SAM4L
//
//  see datasheet section "41. Cyclic Redundancy Check Calculation Unit (CRCCU)"

// Infelicities:
//
// - As much as 512 bytes of RAM is wasted to allow runtime alignment of the
//   CRCCU Descriptor.  Reliable knowledge of kernel alignment might allow this
//   to be done statically.
//
// - It doesn't work:
//      - Although the DMA transfer appears to complete, the CRCCU doesn't
//        seem to issue an interrupt.  (If "compare" mode is enabled and the
//        reference value doesn't match the result, the CRCCU *does* issue
//        the error interrupt.)
//      - The CRC values are not as expected.  (See note below.)

// Notes:
//
// http://www.at91.com/discussions/viewtopic.php/f,29/t,24859.html
//      Atmel is using the low bit instead of the high bit so reversing
//      the values before calculation did the trick. Here is a calculator
//      that matches (click CCITT and check the 'reverse data bytes' to
//      get the correct value).  http://www.zorc.breitbandkatze.de/crc.html
//
//      The SAM4L calculates 0x1541 for "ABCDEFG".

use kernel::returncode::ReturnCode;
use kernel::hil::crc;
use nvic;
use pm::{Clock, HSBClock, PBBClock, enable_clock};

// A memory-mapped register
struct Reg(*mut u32);

impl Reg {
    fn read(self) -> u32 {
        unsafe { ::core::ptr::read_volatile(self.0) }
    }

    fn write(self, n: u32) {
        unsafe { ::core::ptr::write_volatile(self.0, n); }
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
    { 0x00, "Descriptor Base Register", DSCR, "RW" },        // Address of descriptor (512-byte aligned)
    { 0x08, "DMA Enable Register", DMAEN, "W" },             // Write a one to enable DMA channel
    { 0x0C, "DMA Disable Register", DMADIS, "W" },           // Write a one to disable DMA channel
    { 0x10, "DMA Status Register", DMASR, "R" },             // DMA channel enabled?
    { 0x14, "DMA Interrupt Enable Register", DMAIER, "W" },  // Write a one to enable DMA interrupt
    { 0x18, "DMA Interrupt Disable Register", DMAIDR, "W" }, // Write a one to disable DMA interrupt
    { 0x1C, "DMA Interrupt Mask Register", DMAIMR, "R" },    // DMA interrupt enabled?
    { 0x20, "DMA Interrupt Status Register", DMAISR, "R" },  // DMA transfer completed? (cleared when read)
    { 0x34, "Control Register", CR, "W" },                   // Write a one to reset SR
    { 0x38, "Mode Register", MR, "RW" },                     // Bandwidth divider, Polynomial type, Compare?, Enable?
    { 0x3C, "Status Register", SR, "R" },                    // CRC result (unreadable if MR.COMPARE=1)
    { 0x40, "Interrupt Enable Register", IER, "W" },         // Write one to set IMR.ERR bit (zero no effect)
    { 0x44, "Interrupt Disable Register", IDR, "W" },        // Write zero to clear IMR.ERR bit (one no effect)
    { 0x48, "Interrupt Mask Register", IMR, "R" },           // If IMR.ERR bit is set, error-interrupt (for compare) is enabled
    { 0x4C, "Interrupt Status Register", ISR, "R" },         // CRC error (for compare)? (cleared when read)
    { 0xFC, "Version Register", VERSION, "R" }               // 12 low-order bits: version of this module.  = 0x00000202
];

// CRCCU Descriptor (from Table 41.2 in Section 41.6):
#[repr(C, packed)]
struct Descriptor {
    addr: u32,       // Transfer Address Register (RW): Address of memory block to compute
    ctrl: TCR,       // Transfer Control Register (RW): IEN, TRWIDTH, BTSIZE
    _res: [u32; 2],
    crc: u32         // Transfer Reference Register (RW): Reference CRC (for compare mode)
}

// Transfer Control Register (see Section 41.6.18)
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct TCR(u32);

impl TCR {
    const fn new(enable_interrupt: bool, trwidth: TrWidth, btsize: u16) -> Self {
        TCR((!enable_interrupt as u32) << 27
            | (trwidth as u32) << 24
            | (btsize as u32))
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

pub enum TrWidth { Byte, HalfWord, Word }

// Mode Register (see Section 41.6.10)
struct Mode(u32);

impl Mode {
	fn new(divider: u8, ptype: Polynomial, compare: bool, enable: bool) -> Self {
        Mode((((divider & 0x0f) as u32) << 4)
             | (ptype as u32) << 2
             | (compare as u32) << 1
             | (enable as u32))
    }
    fn disabled() -> Self {
        Mode::new(0, Polynomial::CCIT8023, false, false)
    }
}

pub enum Polynomial {
	CCIT8023,   // Polynomial 0x04C11DB7
	CASTAGNOLI, // Polynomial 0x1EDC6F41
	CCIT16,		// Polynomial 0x1021
}

// State for managing the CRCCU
pub struct Crccu<'a> {
    client: Option<&'a crc::Client>,

    // Guaranteed room for a Descriptor with 512-byte alignment.
    // (Can we do this statically instead?)
    descriptor_space: [u8; DSCR_RESERVE],
}

const DSCR_RESERVE: usize = 512 + 5*4;

impl<'a> Crccu<'a> {
    const fn new() -> Self {
        Crccu { client: None,
                descriptor_space: [0; DSCR_RESERVE] }
    }

    pub fn set_client(&mut self, client: &'a crc::Client) {
        self.client = Some(client);
    }

    pub fn get_client(&self) -> Option<&'a crc::Client> {
        self.client
    }

    fn set_descriptor(&mut self, addr: u32, ctrl: TCR, crc: u32) {
        let d = unsafe { &mut *self.descriptor() };
        d.addr = addr;
        d.ctrl = ctrl;
        d.crc = crc;
    }

    // Dynamically calculate the 512-byte-aligned location for Descriptor
    fn descriptor(&self) -> *mut Descriptor {
        let s = &self.descriptor_space as *const [u8; DSCR_RESERVE] as u32;
        let t = s % 512;
        let u = 512 - t;
        let d = s + u;
        return d as *mut Descriptor;
    }

    fn get_tcr(&self) -> TCR {
        let d = unsafe { &*self.descriptor() };
        d.ctrl
    }

    pub fn enable_unit(&self) {
        unsafe {
            // see "10.7.4 Clock Mask"
            enable_clock(Clock::HSB(HSBClock::CRCCU));
            enable_clock(Clock::PBB(PBBClock::CRCCU));

            nvic::disable(nvic::NvicIdx::CRCCU);
            nvic::clear_pending(nvic::NvicIdx::CRCCU);
            nvic::enable(nvic::NvicIdx::CRCCU);
        }
    }

    pub fn handle_interrupt(&mut self) {

        if ISR.read() & 1 == 1 {
            // A CRC error has occurred
            if let Some(client) = self.get_client() {
                client.receive_err();
            }
        }

        if DMAISR.read() & 1 == 1 {
            // A DMA transfer has completed

            if self.get_tcr().interrupt_enabled() {
                if let Some(client) = self.get_client() {
                    let result = SR.read();
                    client.receive_result(result);
                }

                // Disable the unit
                MR.write(Mode::disabled().0);

                // Clear CTRL.IEN (for our own statekeeping)
                self.set_descriptor(0, TCR::default(), 0);
                
                // Disable DMA interrupt and DMA channel
                DMAIDR.write(1);
                DMADIS.write(1);
            }

            /*
            // When is it appropriate to unclock the unit?
            unsafe {
                nvic::disable(nvic::NvicIdx::CRCCU);
                disable_clock(Clock::PBB(PBBClock::CRCCU));
                disable_clock(Clock::HSB(HSBClock::CRCCU));
            }
            */
        }
    }
}

// Implement the generic CRC interface with the CRCCU
impl<'a> crc::CRC for Crccu<'a> {
    fn init(&mut self) -> ReturnCode {
        let daddr = self.descriptor() as u32;
        if daddr & 0x1ff != 0 {
            // Alignment failure
            return ReturnCode::FAIL;
        }

        self.set_descriptor(0, TCR::default(), 0);
        self.enable_unit();
        return ReturnCode::SUCCESS;
    }

    fn get_version(&self) -> u32 {
        VERSION.read()
    }

    fn compute(&mut self, data: &[u8]) -> ReturnCode {
        if self.get_tcr().interrupt_enabled() {
            // A computation is already in progress
            return ReturnCode::EBUSY;
        }

        if data.len() > (2^16 - 1) {
            // Buffer too long
            // TODO: Chain CRCCU computations to handle large buffers
            return ReturnCode::ESIZE;
        }

        self.enable_unit();

        // Enable DMA interrupt
        DMAIER.write(1);

        // Enable error interrupt
        IER.write(1);

        // Reset intermediate CRC value
        CR.write(1);

        // Configure the data transfer
        let addr = data.as_ptr() as u32;
        let ctrl = TCR::new(true, TrWidth::Word, data.len() as u16);
        let crc = 0;
        self.set_descriptor(addr, ctrl, crc);
        DSCR.write(self.descriptor() as u32);

        // Configure the unit to compute a checksum
        let divider = 0;
        let compare = false;
        let enable = true;
        let mode = Mode::new(divider, Polynomial::CCIT16, compare, enable);
        MR.write(mode.0);

        // Enable DMA channel
        DMAEN.write(1);

        /*
        // DEBUG: Don't wait for the interrupt that isn't coming for some reason.
        // Instead, just busy-wait until DMA has completed
        loop {
            if DMASR.read() & 1 == 0 {
                // DMA channel disabled
                if let Some(client) = self.get_client() {
                    let result = SR.read();
                    client.receive_result(result);
                }
                break;
            }
        }
        */

        return ReturnCode::SUCCESS;
    }
}

// If this static is mutable, only unsafe code may use it.
// If it is not (and instead uses internal mutability), it must implement Sync.
pub static mut CRCCU: Crccu<'static> = Crccu::new();

interrupt_handler!(interrupt_handler, CRCCU);
