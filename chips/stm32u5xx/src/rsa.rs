// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use kernel::hil::public_key_crypto::rsa_math::{Client, RsaCryptoBase};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

const RAM_START: usize = 0x400;

register_structs! {
    PkaRegisters {
        // PKA control register
        (0x00 => cr: ReadWrite<u32, CR::Register>),

        // PKA status register
        (0x04 => sr: ReadOnly<u32, SR::Register>),

        // PKA clear flag register
        (0x08 => clrfr: WriteOnly<u32, CLRFR::Register>),

        (0x0C => _reserved0),

        // PKA RAM
        (0x400 => ram: [ReadWrite<u32>; (0x14D8 - 0x400) / 4]),

        (0x14D8 => @END),
    }
}

register_bitfields! [u32,
    CR [
        // Operation error interrupt enable
        OPERRIE OFFSET(21) NUMBITS(1) [],

        // Address error interrupt enable
        ADDERRIE OFFSET(20) NUMBITS(1) [],

        // RAM error interrupt enable
        RAMERRIE OFFSET(19) NUMBITS(1) [],

        // End of operation interrupt enable
        PROCENDIE OFFSET(17) NUMBITS(1) [],

        // PKA operation code
        MODE OFFSET(13) NUMBITS(6) [
            // Montogomery parameter computation then modular exponentioantion
            MontgomeryModularExp = 0b000000,

            // Montgomery parameter computation only
            MontgomeryOnly = 0b000001,

            // Modular exponentiation only (Montgomery parameter must be loaded first)
            ModularExpOnly = 0b000010,

            // Modular exponentiation (protected, used when manipulating secrets)
            ModularExp = 0b000011,

            // Montgomery parameter computation then ECC scalar multiplication (protected)
            MontgomeryECC = 0b100000,

            // ECDSA sign (protected)
            ECDSASign = 0b100100,

            // ECDSA verification
            ECDSAVerfication = 0b100110,

            // Point on elliptic curve Fp check
            FpCheck = 0b101000,

            // RSA CRT exponentiation
            RSACRTExp = 0b000111,

            // Modular inversion
            ModularInversion = 0b001000,

            // Arithmetic addition
            ArithmeticAddition = 0b001001,

            // Arithmetic substraction
            ArithmeticSubstraction = 0b001010,

            // Arithmetic multiplication
            ArithmeticMultiplication = 0b001011,

            // Arithmetic comparison
            ArithmeticComparison = 0b001100,

            // Modular reduction
            ModularReduction = 0b001101,

            // Modular addition
            ModularAddition = 0b001110,

            // Modular substraction
            ModularSubstraction = 0b001111,

            // Montgomery multiplication
            MontgomeryMultiplication = 0b010000,

            // ECC complete addition
            ECCCompleteAddition = 0b100011,

            // ECC double base ladder
            ECCDoubleBaseLadder = 0b100111,

            // ECC projective to affine
            ECCProjectiveToAffine = 0b101111,
        ],

        // Start the operation
        START OFFSET(1) NUMBITS(1) [],

        // PKA enable
        EN OFFSET(0) NUMBITS(1) [],
    ],

    SR [
        // Operation error flag
        OPERRF OFFSET(21) NUMBITS(1) [],

        // Address error flag
        ADDRERRF OFFSET(20) NUMBITS(1) [],

        // PKA RAM Error flag
        RAMERRF OFFSET(19) NUMBITS(1) [],

        // PKA end of operation flag
        PROCENDF OFFSET(17) NUMBITS(1) [],

        // Busy flag
        BUSY OFFSET(16) NUMBITS(1) [],

        // PKA initialization OK
        INITOK OFFSET(0) NUMBITS(1) [],
    ],

    CLRFR [
        // Clear oferation error flag
        OPERRFC OFFSET(21) NUMBITS(1) [],

        // Clear address error flag
        ADDERRFC OFFSET(20) NUMBITS(1) [],

        // Clear PKA RAM error flag
        RAMERRFC OFFSET(19) NUMBITS(1) [],

        // Clear PKA end of op flag
        PROCENDFC OFFSET(17) NUMBITS(1) [],
    ]
];

const PKA_BASE: StaticRef<PkaRegisters> =
    unsafe { StaticRef::new(0x520C2000 as *const PkaRegisters) };

// RAM mapping
// TODO consider moving somewhere else, because these mapping in mode specific
const EXP_LEN_IDX: usize = (0x400 - RAM_START) / 4;
const OP_LEN_IDX: usize = (0x408 - RAM_START) / 4;
const OP_A_IDX: usize = (0xC68 - RAM_START) / 4;
const EXP_IDX: usize = (0xE78 - RAM_START) / 4;
const MOD_VALUE_IDX: usize = (0x1088 - RAM_START) / 4;
const RESULT_IDX: usize = (0x838 - RAM_START) / 4;

pub struct Pka<'a> {
    registers: StaticRef<PkaRegisters>,

    client: OptionalCell<&'a dyn Client<'a>>,

    modulus: OptionalCell<&'static [u8]>,
    exponent: OptionalCell<&'static [u8]>,

    message: TakeCell<'static, [u8]>,
    result: TakeCell<'static, [u8]>,
}

impl<'a> Pka<'a> {
    pub const fn new() -> Pka<'a> {
        Pka {
            registers: PKA_BASE,

            client: OptionalCell::empty(),

            modulus: OptionalCell::empty(),
            exponent: OptionalCell::empty(),

            message: TakeCell::empty(),
            result: TakeCell::empty(),
        }
    }

    // Helper function to write the data to RAM
    fn write_slice(&self, idx: usize, data: &[u8]) {
        let chunks = data.rchunks(4);
        for (i, chunk) in chunks.enumerate() {
            let mut slice = [0u8; 4];
            let offset = 4 - chunk.len(); // in case chunk is less then 4 bytes

            slice[offset..].copy_from_slice(chunk);

            let word = u32::from_be_bytes(slice);
            self.registers.ram[idx + i].set(word);
        }
    }

    // Helper function to read data from RAM
    fn read_slice(&self, idx: usize, buffer: &mut [u8]) {
        let chunks = buffer.rchunks_mut(4);
        for (i, chunk) in chunks.enumerate() {
            let word = self.registers.ram[idx + i].get();
            let bytes = word.to_be_bytes();
            let offset = 4 - chunk.len();
            chunk.copy_from_slice(&bytes[offset..])
        }
    }

    pub fn handle_interrupt(&self) {
        if self.registers.sr.is_set(SR::PROCENDF) {
            // Prevent interrupt from firing again
            self.registers.clrfr.write(CLRFR::PROCENDFC::SET);

            // Unpack the cells
            let modulus = self.modulus.take().unwrap();
            let exponent = self.exponent.take().unwrap();
            let message = self.message.take().unwrap();
            let mut result = self.result.take().unwrap();

            // Read the result
            self.read_slice(RESULT_IDX, &mut result);

            // TODO remove before PR
            kernel::debug!("RSA RESULT: {:02x?}", &result[0..4]);

            self.client.map(|client| {
                client.mod_exponent_done(Ok(true), message, modulus, exponent, result)
            });
        }
    }
}

impl<'a> RsaCryptoBase<'a> for Pka<'a> {
    fn set_client(&'a self, client: &'a dyn Client<'a>) {
        self.client.set(client);
    }

    fn clear_data(&self) {
        // Zero-out all current data
        for i in 0..self.registers.ram.len() {
            self.registers.ram[i].set(0);
        }
    }

    fn mod_exponent(
        &self,
        message: &'static mut [u8],
        modulus: &'static [u8],
        exponent: &'static [u8],
        result: &'static mut [u8],
    ) -> Result<
        (),
        (
            ErrorCode,
            &'static mut [u8],
            &'static [u8],
            &'static [u8],
            &'static mut [u8],
        ),
    > {
        // Check if PKA is not busy
        if self.registers.sr.is_set(SR::BUSY) {
            return Err((ErrorCode::BUSY, message, modulus, exponent, result));
        }

        // Check if parameters are correct
        if result.len() < modulus.len() || exponent.is_empty() || message.is_empty() {
            return Err((ErrorCode::SIZE, message, modulus, exponent, result));
        }

        // Enable the peripheral
        self.registers.cr.modify(CR::EN::SET);

        // Wait for initialization
        while !self.registers.sr.is_set(SR::INITOK) {}

        // Bytes to bits
        let exp_bits = (exponent.len() * 8) as u32;
        let op_bits = (modulus.len() * 8) as u32;

        // Write necessary data to RAM
        self.registers.ram[EXP_LEN_IDX].set(exp_bits);
        self.registers.ram[OP_LEN_IDX].set(op_bits);

        self.write_slice(EXP_IDX, exponent);
        self.write_slice(MOD_VALUE_IDX, modulus);
        self.write_slice(OP_A_IDX, message);

        // Put the values into cells
        self.message.replace(message);
        self.modulus.set(modulus);
        self.exponent.set(exponent);
        self.result.replace(result);

        // Configure the periferal
        self.registers.cr.modify(
            CR::MODE::MontgomeryModularExp + CR::PROCENDIE::SET + CR::START::SET + CR::EN::SET,
        );

        // TODO remove
        kernel::debug!("\nCR::OPERRIE {:#b}", self.registers.cr.read(CR::OPERRIE));
        kernel::debug!("CR::ADDERIE {:#b}", self.registers.cr.read(CR::ADDERRIE));
        kernel::debug!("CR::RAMERRIE {:#b}", self.registers.cr.read(CR::RAMERRIE));
        kernel::debug!("CR::PROCENDIE {:#b}", self.registers.cr.read(CR::PROCENDIE));
        kernel::debug!("CR::MODE {:#b}", self.registers.cr.read(CR::MODE));
        kernel::debug!("CR::START {:#b}", self.registers.cr.read(CR::START));
        kernel::debug!("CR::EN {:#b}", self.registers.cr.read(CR::EN));

        kernel::debug!("\nSR::OPERRF {:#b}", self.registers.sr.read(SR::OPERRF));
        kernel::debug!("SR::ADDERRF {:#b}", self.registers.sr.read(SR::ADDRERRF));
        kernel::debug!("SR::RAMERRF {:#b}", self.registers.sr.read(SR::RAMERRF));
        kernel::debug!("SR::PROCENDF {:#b}", self.registers.sr.read(SR::PROCENDF));
        kernel::debug!("SR::BUSY {:#b}", self.registers.sr.read(SR::BUSY));
        kernel::debug!("SR::INITOK {:#b}\n", self.registers.sr.read(SR::INITOK));

        Ok(())
    }
}
