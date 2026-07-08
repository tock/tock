use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::crc::{Client, Crc, CrcAlgorithm, CrcOutput};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    pub CrcRegisters {
        /// Data register
        (0x00 => pub dr: ReadWrite<u32, DR::Register>),
        /// Independent data register
        (0x04 => pub idr: ReadWrite<u32, IDR::Register>),
        /// Control register
        (0x08 => pub cr: ReadWrite<u32, CR::Register>),
        /// Padding
        (0x0C => reserved),
        /// Initial value
        (0x10 => pub init: ReadWrite<u32, INIT::Register>),
        /// Polynomial
        (0x14 => pub pol: ReadWrite<u32, POL::Register>),
        (0x18 => @END),
    }
}

/// Base address for CRC in Secure Alias mode
pub const CRC_BASE: StaticRef<CrcRegisters> =
    unsafe { StaticRef::new(0x50023000 as *const CrcRegisters) };

register_bitfields![u32,
     pub DR [
        /// Data register
        DR OFFSET(0) NUMBITS(32) []
    ],
    pub IDR [
        /// Temporary 4 byte storage
        DR OFFSET(0) NUMBITS(32) []
    ],
    pub CR [
        /// Reset bit, used for initialising and resetting
        RESET OFFSET(0) NUMBITS(1) [],
        /// Polynomial size
        PSIZE OFFSET(3) NUMBITS(2) [],
        /// Reverse input data
        REVIN OFFSET(5) NUMBITS(2) [],
        /// Reverse output data
        REVOUT OFFSET(7) NUMBITS(1) []
    ],
    pub INIT [
        /// Initial CRC value
        INIT OFFSET(0) NUMBITS(32) []
    ],
    pub POL [
        /// Polynomial coefficients to be used for CRC computation
        POL OFFSET(0) NUMBITS(32) []
    ],
];

// CRC state checkers, used in all functions
#[derive(Copy, Clone, PartialEq)]
enum State {
    Idle,
    Processing,
}

/// Checker values for verifying if the algorithm has been set
#[derive(Copy, Clone, PartialEq)]
enum AlgSet {
    Uninitialised,
    Initialised,
}

/// Checker values for the DeferredCallClient
#[derive(Copy, Clone, PartialEq)]
enum Request {
    Input,
    Compute,
    None,
}

pub struct CRC<'a> {
    registers: StaticRef<CrcRegisters>,
    client: OptionalCell<&'a dyn Client>,
    deferred_call: DeferredCall,
    state: Cell<State>,
    alg_state: Cell<AlgSet>,
    buffer: OptionalCell<SubSliceMut<'static, u8>>,
    request: Cell<Request>,
}

impl<'a> CRC<'a> {
    pub fn new(base_addr: StaticRef<CrcRegisters>) -> Self {
        Self {
            registers: base_addr,
            client: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
            state: Cell::new(State::Idle),
            alg_state: Cell::new(AlgSet::Uninitialised),
            buffer: OptionalCell::empty(),
            request: Cell::new(Request::None),
        }
    }
}

impl<'a> Crc<'a> for CRC<'a> {
    fn set_client(&self, client: &'a dyn Client) {
        self.client.set(client);
    }

    fn algorithm_supported(&self, algorithm: CrcAlgorithm) -> bool {
        matches!(algorithm, CrcAlgorithm::Crc32)
    }

    fn set_algorithm(&self, algorithm: CrcAlgorithm) -> Result<(), ErrorCode> {
        if !self.algorithm_supported(algorithm) {
            return Err(ErrorCode::NOSUPPORT);
        }

        if self.state.get() == State::Processing {
            return Err(ErrorCode::BUSY);
        }

        // The STM32U5xx features programable parameters, in order to accomodate for
        // multiple CRC algorithms, enforceable by the user
        // All the following parameters have been configured as per the
        // CRC32 Ethernet algorithm

        // initial value of the CRC, on the STM32U5xx the CRC_DR's first value is
        // set by the CR_INIT; it can be used for both writing and reading;
        self.registers.init.write(INIT::INIT.val(0xFFFFFFFF));

        // These bits control the size of the polynomial.
        // 00: 32 byt polynomial
        // 10: 8 bit polynomial
        // 01: 16 bit polynomial
        // 00: 32 bit polynomial
        // as per the CRC32 Ethernet algorithm, this one was set to 32 bits
        self.registers.cr.modify(CR::PSIZE.val(0b00));

        // This bitfield controls the reversal of the bit order of the input data.
        // 00: Bit order not affected
        // 01: Bit reversal done by byte
        // 10: Bit reversal done by half-word
        // 11: Bit reversal done by word
        // as per the CRC32 Ethernet algorithm, this one was set to byte by reversal
        self.registers.cr.modify(CR::REVIN.val(0b01));

        // Reverse output data
        // This bit controls the reversal of the bit order of the output data.
        // 0: Bit order not affected
        // 1: Bit-reversed output format
        // as per the CRC32 Ethernet algorithm, reversed-bit output was selected
        self.registers.cr.modify(CR::REVOUT.val(0b01));

        // Programmable polynomial
        // This register is used to write the coefficients of the polynomial to be
        // used for CRC calculation.
        // If the polynomial size is less than 32 bits, the least significant bits
        // have to be used to program the correct value.
        // As mentioned in the reference manual, the default polynomial value for the
        // CRC-32 Ethernet polynomial is 0x4C11DB7.
        self.registers.pol.write(POL::POL.val(0x4C11DB7));

        // Initialising the CRC engine as per the manual, by setting the RESET Bit
        self.registers.cr.modify(CR::RESET::SET);

        self.state.set(State::Idle);
        self.alg_state.set(AlgSet::Initialised);

        Ok(())
    }

    fn input(
        &self,
        data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        if self.alg_state.get() == AlgSet::Uninitialised {
            return Err((ErrorCode::RESERVE, data));
        }

        if self.state.get() == State::Processing {
            return Err((ErrorCode::BUSY, data));
        }

        self.state.set(State::Processing);

        // wrote them as mut_slice and iter_mut first, realised that I only read them
        // so would not need it
        for &byte in data.as_slice().iter() {
            // should use set? or write
            self.registers.dr.set(byte as u32);
        }

        self.buffer.set(data);
        self.request.set(Request::Input);
        self.deferred_call.set();

        Ok(())
    }

    fn compute(&self) -> Result<(), ErrorCode> {
        if self.alg_state.get() == AlgSet::Uninitialised {
            return Err(ErrorCode::RESERVE);
        }

        if self.state.get() == State::Processing {
            return Err(ErrorCode::BUSY);
        }

        self.state.set(State::Processing);
        self.request.set(Request::Compute);
        self.deferred_call.set();

        Ok(())
    }

    fn disable(&self) {
        // The STM's CRC has no bit for directly disabling it, we handle it by
        // setting the computation state to idle and the algorithm setting as
        // uninitialised.
        self.state.set(State::Idle);
        self.alg_state.set(AlgSet::Uninitialised);
    }
}

impl<'a> DeferredCallClient for CRC<'a> {
    fn handle_deferred_call(&self) {
        self.state.set(State::Idle);

        self.client.map(|client| {
            match self.request.get() {
                Request::Input => {
                    if let Some(data) = self.buffer.take() {
                        client.input_done(Ok(()), data);
                    }
                }

                Request::Compute => {
                    // the CRC32 Ethernet algorithm requires a final XOR on the data
                    // the STM's CRC does not have one such option built in,
                    // so we use a regular software XOR
                    let result = self.registers.dr.get() ^ 0xFFFFFFFF;
                    client.crc_done(Ok(CrcOutput::Crc32(result)));
                }

                Request::None => {}
            }

            self.request.set(Request::None);
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
