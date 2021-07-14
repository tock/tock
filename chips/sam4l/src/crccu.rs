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

use crate::deferred_call_tasks::Task;
use crate::pm::{disable_clock, enable_clock, Clock, HSBClock, PBBClock};
use core::cell::Cell;
use kernel::deferred_call::DeferredCall;
use kernel::hil::crc::{Client, Crc, CrcAlgorithm, CrcOutput};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, FieldValue, InMemoryRegister, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

// Base address of CRCCU registers.  See "7.1 Product Mapping"
pub const BASE_ADDRESS: StaticRef<CrccuRegisters> =
    unsafe { StaticRef::new(0x400A4000 as *const CrccuRegisters) };

static DEFERRED_CALL: DeferredCall<Task> = unsafe { DeferredCall::new(Task::CRCCU) };

#[repr(C)]
pub struct CrccuRegisters {
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

#[repr(C)]
#[repr(align(512))]
struct Descriptor {
    /// Transfer Address Register (RW): Address of memory block to compute
    addr: InMemoryRegister<u32>,
    /// Transfer Control Register (RW): IEN, TRWIDTH, BTSIZE
    ctrl: InMemoryRegister<u32>,
    _res: [u32; 2],
    /// Transfer Reference Register (RW): Reference CRC (for compare mode)
    crc: InMemoryRegister<u32>,
}

impl Descriptor {
    pub fn new() -> Descriptor {
        Descriptor {
            addr: InMemoryRegister::new(0),
            ctrl: InMemoryRegister::new(TCR::default().0),
            _res: [0; 2],
            crc: InMemoryRegister::new(0),
        }
    }
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

fn poly_for_alg(alg: CrcAlgorithm) -> FieldValue<u32, Mode::Register> {
    match alg {
        CrcAlgorithm::Crc32 => Mode::PTYPE::Ccit8023,
        CrcAlgorithm::Crc32C => Mode::PTYPE::Castagnoli,
        CrcAlgorithm::Crc16CCITT => Mode::PTYPE::Ccit16,
        // CrcAlg::Sam4L32 => Mode::PTYPE::Ccit8023,
        // CrcAlg::Sam4L32C => Mode::PTYPE::Castagnoli,
    }
}

fn post_process(result: u32, alg: CrcAlgorithm) -> CrcOutput {
    match alg {
        CrcAlgorithm::Crc32 => CrcOutput::Crc32(reverse_and_invert(result)),
        CrcAlgorithm::Crc32C => CrcOutput::Crc32C(reverse_and_invert(result)),
        CrcAlgorithm::Crc16CCITT => CrcOutput::Crc16CCITT(result as u16),
        // CrcAlg::Sam4L32 => result,
        // CrcAlg::Sam4L32C => result,
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
    client: OptionalCell<&'a dyn Client>,
    state: Cell<State>,
    algorithm: OptionalCell<CrcAlgorithm>,

    // This store the full leasable-buffer boundaries for
    // reconstruction when a call to [`Crc::input`] finishes
    current_full_buffer: Cell<(*mut u8, usize)>,

    // Marker whether a "computation" (pending deferred call) is in
    // progress
    compute_requested: Cell<bool>,

    // CRC DMA descriptor
    //
    // Must be aligned to a 512-byte boundary, which is guaranteed by
    // the struct definition.
    descriptor: Descriptor,
}

impl Crccu<'_> {
    pub fn new(base_addr: StaticRef<CrccuRegisters>) -> Self {
        Crccu {
            registers: base_addr,
            client: OptionalCell::empty(),
            state: Cell::new(State::Invalid),
            algorithm: OptionalCell::empty(),
            current_full_buffer: Cell::new((0 as *mut u8, 0)),
            compute_requested: Cell::new(false),
            descriptor: Descriptor::new(),
        }
    }

    fn init(&self) {
        if self.state.get() == State::Invalid {
            self.descriptor.addr.set(0);
            self.descriptor.ctrl.set(TCR::default().0);
            self.descriptor.crc.set(0);
            self.state.set(State::Initialized);
        }
    }

    /// Enable the CRCCU's clocks and interrupt
    fn enable(&self) {
        if self.state.get() != State::Enabled {
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

    /// Handle an interrupt from the CRCCU
    pub fn handle_interrupt(&self) {
        if self.registers.isr.is_set(Interrupt::ERR) {
            // A CRC error has occurred
        }

        if self.registers.dmaisr.is_set(DmaInterrupt::DMA) {
            // A DMA transfer has completed

            if TCR(self.descriptor.ctrl.get()).interrupt_enabled() {
                // We have the current temporary result ready, but
                // wait for the client to finish the CRC computation
                // by calling [`Crc::compute`]

                // self.client.map(|client| {
                //     let result = post_process(self.registers.sr.read(Status::CRC), self.alg.get());
                //     client.receive_result(result);
                // });

                // Disable the unit
                self.registers.mr.write(Mode::ENABLE::Disabled);

                // Recover the window into the LeasableBuffer
                let window_addr = self.descriptor.addr.get();
                let window_len = TCR(self.descriptor.ctrl.get()).get_btsize() as usize;

                // Reset CTRL.IEN (for our own statekeeping)
                self.descriptor.addr.set(0);
                self.descriptor.ctrl.set(TCR::default().0);
                self.descriptor.crc.set(0);

                // Disable DMA interrupt
                self.registers.dmaidr.write(DmaInterrupt::DMA::SET);

                // Disable DMA channel
                self.registers.dmadis.write(DmaDisable::DMADIS::SET);

                // Reconstruct the leasable buffer from stored
                // information and slice into the proper window
                let (full_buffer_addr, full_buffer_len) = self.current_full_buffer.get();
                let mut data = LeasableBuffer::<'static, u8>::new(unsafe {
                    core::slice::from_raw_parts_mut(full_buffer_addr, full_buffer_len)
                });

                // Must be strictly positive or zero
                let start_offset = (window_addr as usize) - (full_buffer_addr as usize);
                data.slice(start_offset..(start_offset + window_len));

                // Pass the properly sliced and reconstructed buffer
                // back to the client
                self.client.map(move |client| {
                    client.input_done(Ok(()), data);
                });
            }
        }
    }

    pub fn handle_deferred_call(&self) {
        // A deferred call is currently only issued on a call to
        // compute, in which case we need to provide the CRC to the
        // client
        let result = post_process(
            self.registers.sr.read(Status::CRC),
            self.algorithm.expect("crccu deferred call: no algorithm"),
        );

        // Reset the internal CRC state such that the next call to
        // input will start a new CRC
        self.registers.cr.write(Control::RESET::SET);
        self.descriptor.ctrl.set(TCR::default().0);
        self.compute_requested.set(false);

        self.client.map(|client| {
            client.crc_done(Ok(result));
        });
    }
}

// Implement the generic CRC interface with the CRCCU
impl<'a> Crc<'a> for Crccu<'a> {
    /// Set a client to receive results from the CRCCU
    fn set_client(&self, client: &'a dyn Client) {
        self.client.set(client);
    }

    fn algorithm_supported(&self, algorithm: CrcAlgorithm) -> bool {
        // Deliberately has an exhaustive list here to avoid
        // advertising support for added variants to CrcAlgorithm
        match algorithm {
            CrcAlgorithm::Crc32 => true,
            CrcAlgorithm::Crc32C => true,
            CrcAlgorithm::Crc16CCITT => true,
        }
    }

    fn set_algorithm(&self, algorithm: CrcAlgorithm) -> Result<(), ErrorCode> {
        // If there currently is a DMA operation in progress, refuse
        // to set the algorithm.
        if TCR(self.descriptor.ctrl.get()).interrupt_enabled() || self.compute_requested.get() {
            // A computation is already in progress
            return Err(ErrorCode::BUSY);
        }

        self.init();
        // Clear the descriptor contents
        self.descriptor.addr.set(0);
        self.descriptor.ctrl.set(TCR::default().0);
        self.descriptor.crc.set(0);
        self.algorithm.set(algorithm);

        // Reset intermediate CRC value
        self.registers.cr.write(Control::RESET::SET);

        Ok(())
    }

    fn input(
        &self,
        mut data: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, LeasableBuffer<'static, u8>)> {
        let algorithm = if let Some(algorithm) = self.algorithm.extract() {
            algorithm
        } else {
            return Err((ErrorCode::RESERVE, data));
        };

        if TCR(self.descriptor.ctrl.get()).interrupt_enabled() || self.compute_requested.get() {
            // A computation is already in progress
            return Err((ErrorCode::BUSY, data));
        }

        // Need to initialize after checking business, because init will
        // clear out interrupt state.
        self.init();

        // Initialize the descriptor, since it is used to track business
        let len = data.len() as u16;
        let ctrl = TCR::new(true, TrWidth::Byte, len);

        // Make sure we don't try to process more data than the CRC
        // DMA operation supports.
        if data.len() > u16::MAX as usize {
            // Restore the full slice, calculate the current
            // window's start offset.
            let window_ptr = data.as_ptr();
            data.reset();
            let start_ptr = data.as_ptr();
            // Must be strictly positive or zero
            let start_offset = unsafe { window_ptr.offset_from(start_ptr) } as usize;

            // Reslice the buffer such that it start at the same
            // position as the old window, but fits the size
            // constraints
            data.slice(start_offset..=(start_offset + u16::MAX as usize));
        }

        self.enable();

        // Enable DMA interrupt
        self.registers.dmaier.write(DmaInterrupt::DMA::SET);

        // Enable error interrupt
        self.registers.ier.write(Interrupt::ERR::SET);

        // Configure the data transfer descriptor
        //
        // The data length is guaranteed to be <= u16::MAX by the
        // above LeasableBuffer resizing mechanism
        self.descriptor.addr.set(data.as_ptr() as u32);
        self.descriptor.ctrl.set(ctrl.0);
        self.descriptor.crc.set(0); // this is the CRC compare field, not used

        // Prior to starting the DMA operation, drop the
        // LeasableBuffer slice. Otherwise we violate Rust's mutable
        // aliasing rules.
        let full_slice = data.take();
        let full_slice_ptr_len = (full_slice.as_mut_ptr(), full_slice.len());
        self.current_full_buffer.set(full_slice_ptr_len);

        // Ensure the &'static mut slice reference goes out of scope
        //
        // We can't use mem::drop on a reference here, clippy will
        // complain, even though it would be effective at making this
        // 'static mut buffer inaccessible. For now, just make sure to
        // not reference it below.
        //
        // TODO: this needs a proper type and is a broader issue. See
        // tock/tock#2637 for more information.
        //
        // core::mem::drop(full_slice);

        // Set the descriptor memory address accordingly
        self.registers
            .dscr
            .set(&self.descriptor as *const Descriptor as u32);

        // Configure the unit to compute a checksum
        self.registers.mr.write(
            Mode::DIVIDER.val(0)
                + poly_for_alg(algorithm)
                + Mode::COMPARE::CLEAR
                + Mode::ENABLE::Enabled,
        );

        // Enable DMA channel
        self.registers.dmaen.write(DmaEnable::DMAEN::SET);

        Ok(())
    }

    fn compute(&self) -> Result<(), ErrorCode> {
        // In this hardware implementation, we compute the CRC in
        // parallel to the DMA operations. Thus this can simply
        // request the CR and reset the state in a deferred call.

        if TCR(self.descriptor.ctrl.get()).interrupt_enabled() || self.compute_requested.get() {
            // A computation is already in progress
            return Err(ErrorCode::BUSY);
        }

        // Mark the device as busy
        self.compute_requested.set(true);

        // Request a deferred call such that we can provide the result
        // back to the client
        DEFERRED_CALL.set();

        Ok(())
    }

    fn disable(&self) {
        Crccu::disable(self);
    }
}
