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
use kernel::common::regs::{FieldValue, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::crc::{self, CrcAlg};
use kernel::ReturnCode;
use pm::{disable_clock, enable_clock, Clock, HSBClock, PBBClock};

// Base address of CRCCU registers.  See "7.1 Product Mapping"
const BASE_ADDRESS: StaticRef<CrccuRegisters> =
    unsafe { StaticRef::new(0x400A4000 as *const CrccuRegisters) };

#[repr(C)]
struct CrccuRegisters {
    // From page 1005 of SAM4L manual
    dscr: ReadWrite<u32, DescriptorBaseAddress::Register>,
    _reserved0: u32,
    dmaen: WriteOnly<u32, DmaEnable::Register>,
    dmadis: WriteOnly<u32, DmaDisable::Register>,
    dmasr: ReadOnly<u32, DmaStatus::Register>,
    dmaier: WriteOnly<u32, DmaInterrupt::Register>,
    dmaidr: WriteOnly<u32, DmaInterrupt::Register>,
    dmaimr: ReadOnly<u32, DmaInterrupt::Register>,
    dmaisr: ReadOnly<u32, DmaInterrupt::Register>,
    _reserved1: [u32; 4],
    cr: WriteOnly<u32, Control::Register>,
    mr: ReadWrite<u32, Mode::Register>,
    sr: ReadOnly<u32, Status::Register>,
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    isr: ReadOnly<u32, Interrupt::Register>,
}

register_bitfields![u32,
    DescriptorBaseAddress [
        /// Description Base Address
        DSCR OFFSET(9) NUMBITS(23) []
    ],

    DmaEnable [
        /// DMA Enable
        DMAEN 0
    ],

    DmaDisable [
        /// DMA Disable
        DMADIS 0
    ],

    DmaStatus [
        /// DMA Channel Status
        DMASR 0
    ],

    DmaInterrupt [
        /// DMA Interrupt
        DMA 0
    ],

    Control [
        /// Reset CRC Computation
        RESET 0
    ],

    Mode [
        /// Bandwidth Divider
        DIVIDER OFFSET(4) NUMBITS(4) [],
        /// Polynomial Type
        PTYPE OFFSET(2) NUMBITS(2) [
            Ccit8023 = 0,
            Castagnoli = 1,
            Ccit16 = 2
        ],
        /// CRC Compare
        COMPARE OFFSET(1) NUMBITS(1) [],
        /// CRC Computation Enable
        ENABLE OFFSET(0) NUMBITS(1) [
            Enabled = 1,
            Disabled = 0
        ]
    ],

    Status [
        /// Cyclic Redundancy Check Value
        CRC OFFSET(0) NUMBITS(32)
    ],

    Interrupt [
        /// CRC Error Interrupt Status
        ERR 0
    ]
];

// CRCCU Descriptor (from Table 41.2 in Section 41.6):
#[repr(C)]
struct Descriptor {
    addr: u32, // Transfer Address Register (RW): Address of memory block to compute
    ctrl: TCR, // Transfer Control Register (RW): IEN, TRWIDTH, BTSIZE
    _res: [u32; 2],
    crc: u32, // Transfer Reference Register (RW): Reference CRC (for compare mode)
}

// Transfer Control Register (see Section 41.6.18)
#[derive(Copy, Clone)]
#[repr(C)]
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

fn poly_for_alg(alg: CrcAlg) -> FieldValue<u32, Mode::Register> {
    match alg {
        CrcAlg::Crc32 => Mode::PTYPE::Ccit8023,
        CrcAlg::Crc32C => Mode::PTYPE::Castagnoli,
        CrcAlg::Sam4L16 => Mode::PTYPE::Ccit16,
        CrcAlg::Sam4L32 => Mode::PTYPE::Ccit8023,
        CrcAlg::Sam4L32C => Mode::PTYPE::Castagnoli,
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
    out ^ 0xffffffff
}

/// Transfer width for DMA
#[allow(dead_code)]
enum TrWidth {
    Byte,
    HalfWord,
    Word,
}

#[derive(Copy, Clone, PartialEq)]
enum State {
    Invalid,
    Initialized,
    Enabled,
}

/// State for managing the CRCCU
pub struct Crccu<'a> {
    registers: StaticRef<CrccuRegisters>,
    client: Option<&'a crc::Client>,
    state: Cell<State>,
    alg: Cell<CrcAlg>,

    // Guaranteed room for a Descriptor with 512-byte alignment.
    // (Can we do this statically instead?)
    descriptor_space: [u8; DSCR_RESERVE],
}

const DSCR_RESERVE: usize = 512 + 5 * 4;

impl<'a> Crccu<'a> {
    const fn new(base_address: StaticRef<CrccuRegisters>) -> Self {
        Crccu {
            registers: base_address,
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
    fn enable(&self) {
        if self.state.get() != State::Enabled {
            self.init();
            // see "10.7.4 Clock Mask"
            enable_clock(Clock::HSB(HSBClock::CRCCU));
            enable_clock(Clock::PBB(PBBClock::CRCCU));
            self.state.set(State::Enabled);
        }
    }

    /// Disable the CRCCU's clocks and interrupt
    fn disable(&self) {
        if self.state.get() == State::Enabled {
            disable_clock(Clock::PBB(PBBClock::CRCCU));
            disable_clock(Clock::HSB(HSBClock::CRCCU));
            self.state.set(State::Initialized);
        }
    }

    /// Set a client to receive results from the CRCCU
    pub fn set_client(&mut self, client: &'a crc::Client) {
        self.client = Some(client);
    }

    /// Get the client currently receiving results from the CRCCU
    fn get_client(&self) -> Option<&'a crc::Client> {
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
        let regs: &CrccuRegisters = &*self.registers;

        if regs.isr.is_set(Interrupt::ERR) {
            // A CRC error has occurred
        }

        if regs.dmaisr.is_set(DmaInterrupt::DMA) {
            // A DMA transfer has completed

            if self.get_tcr().interrupt_enabled() {
                if let Some(client) = self.get_client() {
                    let result = post_process(regs.sr.read(Status::CRC), self.alg.get());
                    client.receive_result(result);
                }

                // Disable the unit
                regs.mr.write(Mode::ENABLE::Disabled);

                // Reset CTRL.IEN (for our own statekeeping)
                self.set_descriptor(0, TCR::default(), 0);

                // Disable DMA interrupt
                regs.dmaidr.write(DmaInterrupt::DMA::SET);

                // Disable DMA channel
                regs.dmadis.write(DmaDisable::DMADIS::SET);
            }
        }
    }
}

// Implement the generic CRC interface with the CRCCU
impl<'a> crc::CRC for Crccu<'a> {
    fn compute(&self, data: &[u8], alg: CrcAlg) -> ReturnCode {
        let regs: &CrccuRegisters = &*self.registers;

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
        regs.dmaier.write(DmaInterrupt::DMA::SET);

        // Enable error interrupt
        regs.ier.write(Interrupt::ERR::SET);

        // Reset intermediate CRC value
        regs.cr.write(Control::RESET::SET);

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
        regs.dscr.set(self.descriptor() as u32);

        // Record what algorithm was requested
        self.alg.set(alg);

        // Configure the unit to compute a checksum
        regs.mr.write(
            Mode::DIVIDER.val(0) + poly_for_alg(alg) + Mode::COMPARE::CLEAR + Mode::ENABLE::Enabled,
        );

        // Enable DMA channel
        regs.dmaen.write(DmaEnable::DMAEN::SET);

        return ReturnCode::SUCCESS;
    }

    fn disable(&self) {
        Crccu::disable(self);
    }
}

/// Static state to manage the CRCCU
pub static mut CRCCU: Crccu<'static> = Crccu::new(BASE_ADDRESS);
