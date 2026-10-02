// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use kernel::{
    hil::crypto::modular_arithmetic::{OpAddition, OpInverse, OpModulo, OpMultiplication},
    utilities::{
        StaticRef,
        registers::{ReadOnly, ReadWrite, WriteOnly, register_bitfields, register_structs},
    },
};

register_structs! {
    pub PkaRegisters {
        /// PKA control register
        (0x00 => pub(crate) cr: ReadWrite<u32, CR::Register>),

        /// PKA status register
        (0x04 => pub(crate) sr: ReadOnly<u32, SR::Register>),

        /// PKA clear flag register
        (0x08 => pub(crate) clrfr: WriteOnly<u32, CLRFR::Register>),

        (0x0C => _reserved0),

        /// PKA RAM
        /// 0x14D8-0x400 is 0x10D8 bytes, which is 4312 bytes in decimal. We divide by the size of u32 (4 bytes)
        /// Two 32-bit slices would correspond to one 64-bit "word" as defined in datasheet
        (0x400 => pub(crate) ram: [ReadWrite<u32>; (0x14D8 - 0x400) / size_of::<u32>()]),

        (0x14D8 => @END),
    }
}

register_bitfields! [u32,
    pub(crate) CR [
        /// Operation error interrupt enable
        OPERRIE OFFSET(21) NUMBITS(1) [],

        /// Address error interrupt enable
        ADDERRIE OFFSET(20) NUMBITS(1) [],

        /// RAM error interrupt enable
        RAMERRIE OFFSET(19) NUMBITS(1) [],

        /// End of operation interrupt enable
        PROCENDIE OFFSET(17) NUMBITS(1) [],

        /// PKA operation code
        MODE OFFSET(8) NUMBITS(6) [
            /// Montgomery parameter computation then modular exponentiation
            MontgomeryModularExp = 0b000000,

            /// Montgomery parameter computation only
            MontgomeryOnly = 0b000001,

            /// Modular exponentiation only (Montgomery parameter must be loaded first)
            ModularExpOnly = 0b000010,

            /// Modular exponentiation (protected, used when manipulating secrets)
            ModularExp = 0b000011,

            /// Montgomery parameter computation then ECC scalar multiplication (protected)
            MontgomeryECC = 0b100000,

            /// ECDSA sign (protected)
            ECDSASign = 0b100100,

            /// ECDSA verification
            ECDSAVerfication = 0b100110,

            /// Point on elliptic curve Fp check
            FpCheck = 0b101000,

            /// RSA CRT exponentiation
            RSACRTExp = 0b000111,

            /// Modular inversion
            ModularInversion = 0b001000,

            /// Arithmetic addition
            ArithmeticAddition = 0b001001,

            /// Arithmetic substraction
            ArithmeticSubstraction = 0b001010,

            /// Arithmetic multiplication
            ArithmeticMultiplication = 0b001011,

            /// Arithmetic comparison
            ArithmeticComparison = 0b001100,

            /// Modular reduction
            ModularReduction = 0b001101,

            /// Modular addition
            ModularAddition = 0b001110,

            /// Modular substraction
            ModularSubstraction = 0b001111,

            /// Montgomery multiplication
            MontgomeryMultiplication = 0b010000,

            /// ECC complete addition
            ECCCompleteAddition = 0b100011,

            /// ECC double base ladder
            ECCDoubleBaseLadder = 0b100111,

            /// ECC projective to affine
            ECCProjectiveToAffine = 0b101111,
        ],

        /// Start the operation
        START OFFSET(1) NUMBITS(1) [],

        /// PKA enable
        EN OFFSET(0) NUMBITS(1) [],
    ],

    pub(crate) SR [
        /// Operation error flag
        OPERRF OFFSET(21) NUMBITS(1) [],

        /// Address error flag
        ADDRERRF OFFSET(20) NUMBITS(1) [],

        /// PKA RAM error flag
        RAMERRF OFFSET(19) NUMBITS(1) [],

        /// PKA end of operation flag
        PROCENDF OFFSET(17) NUMBITS(1) [],

        /// Busy flag
        BUSY OFFSET(16) NUMBITS(1) [],

        /// PKA initialization OK
        INITOK OFFSET(0) NUMBITS(1) [],
    ],

    pub(crate) CLRFR [
        /// Clear operation error flag
        OPERRFC OFFSET(21) NUMBITS(1) [],

        /// Clear address error flag
        ADDERRFC OFFSET(20) NUMBITS(1) [],

        /// Clear PKA RAM error flag
        RAMERRFC OFFSET(19) NUMBITS(1) [],

        /// Clear PKA end of operation flag
        PROCENDFC OFFSET(17) NUMBITS(1) [],
    ]
];

/// Base address for PKA registers
pub(crate) const PKA_BASE: StaticRef<PkaRegisters> =
    unsafe { StaticRef::new(0x520C2000 as *const PkaRegisters) };

// ---------------------------------------------------------------------------
// PKA RAM layout
//
// The PKA RAM is reused by every mode, so the same offset can have a different
// meaning depending on the operation. Constants are grouped per mode; where two
// names share an offset, they are deliberate aliases.
//
// `*_ADDR` is the byte address relative to the PKA base, `*_IDX` is the index
// into `PkaRegisters::ram` (u32 words).
// ---------------------------------------------------------------------------

/// Start of the RAM region
const RAM_START: usize = 0x400;

/// Converts a PKA RAM address into an index in the `ram` array.
const fn calc_idx(addr: usize) -> usize {
    (addr - RAM_START) / size_of::<u32>()
}

// Constants have been taken from the STM32U5 Reference manual, section 53.4 PKA
// operating modes, pages  2067-2082

// ---- Montgomery modular exponentiation and integer/modular arithmetic ------

/// Exponent length in bits (also the modulus length for modular reduction)
const EXP_LEN_BITS_ADDR: usize = 0x400;
/// Operand length in bits
const OPERAND_LEN_BITS_ADDR: usize = 0x408;
/// First operand of integer/modular arithmetic
const ARITH_OP1_ADDR: usize = 0xA50;
/// Second operand of integer/modular arithmetic
const ARITH_OP2_ADDR: usize = 0xC68;
/// Base of the modular exponentiation (same slot as `ARITH_OP2_ADDR`)
const MODEXP_BASE_ADDR: usize = 0xC68;
/// Modulus for inversion and reduction (same slot as `ARITH_OP2_ADDR`)
const INV_RED_MODULUS_ADDR: usize = 0xC68;
/// Exponent of the modular exponentiation
const MODEXP_EXPONENT_ADDR: usize = 0xE78;
/// Modulus for modular exponentiation and modular addition/multiplication
const MODULUS_ADDR: usize = 0x1088;
/// Result of the modular exponentiation
const MODEXP_RESULT_ADDR: usize = 0x838;
/// Result of integer/modular arithmetic operations
const ARITH_RESULT_ADDR: usize = 0xE78;
/// Output of the Montgomery parameter computation (R^2 mod n)
const MONT_R2_OUT_ADDR: usize = 0x620;

pub(crate) const EXP_LEN_BITS_IDX: usize = calc_idx(EXP_LEN_BITS_ADDR);
pub(crate) const OPERAND_LEN_BITS_IDX: usize = calc_idx(OPERAND_LEN_BITS_ADDR);
pub(crate) const ARITH_OP1_IDX: usize = calc_idx(ARITH_OP1_ADDR);
pub(crate) const ARITH_OP2_IDX: usize = calc_idx(ARITH_OP2_ADDR);
pub(crate) const MODEXP_BASE_IDX: usize = calc_idx(MODEXP_BASE_ADDR);
pub(crate) const INV_RED_MODULUS_IDX: usize = calc_idx(INV_RED_MODULUS_ADDR);
pub(crate) const MODEXP_EXPONENT_IDX: usize = calc_idx(MODEXP_EXPONENT_ADDR);
pub(crate) const MODULUS_IDX: usize = calc_idx(MODULUS_ADDR);
pub(crate) const MODEXP_RESULT_IDX: usize = calc_idx(MODEXP_RESULT_ADDR);
pub(crate) const ARITH_RESULT_IDX: usize = calc_idx(ARITH_RESULT_ADDR);
pub(crate) const MONT_R2_OUT_IDX: usize = calc_idx(MONT_R2_OUT_ADDR);

// ---- ECC Fp: curve parameters and scalar multiplication --------------------

/// Length of the curve group order n, in bits
const ECC_N_LEN_BITS_ADDR: usize = 0x400;
/// Length of the curve field prime p, in bits
const ECC_P_LEN_BITS_ADDR: usize = 0x408;
/// Sign of the curve coefficient a
const ECC_A_SIGN_ADDR: usize = 0x410;
/// Absolute value of the curve coefficient a
const ECC_A_ABS_ADDR: usize = 0x418;
/// Curve coefficient b
const ECC_B_ADDR: usize = 0x520;
/// Curve field prime p
const ECC_P_ADDR: usize = 0x1088;
/// Curve group order n
const ECC_N_ADDR: usize = 0xF88;
/// R^2 mod p (Montgomery parameter)
const ECC_P_R2_ADDR: usize = 0x4C8;
/// Scalar multiplier k
const ECC_MUL_K_ADDR: usize = 0x12A0;
/// Input point X coordinate
const ECC_MUL_IN_X_ADDR: usize = 0x578;
/// Input point Y coordinate
const ECC_MUL_IN_Y_ADDR: usize = 0x470;
/// Output point X coordinate (also used by projective-to-affine)
const ECC_OUT_X_ADDR: usize = 0x578;
/// Output point Y coordinate (also used by projective-to-affine)
const ECC_OUT_Y_ADDR: usize = 0x5D0;

pub(crate) const ECC_N_LEN_BITS_IDX: usize = calc_idx(ECC_N_LEN_BITS_ADDR);
pub(crate) const ECC_P_LEN_BITS_IDX: usize = calc_idx(ECC_P_LEN_BITS_ADDR);
pub(crate) const ECC_A_SIGN_IDX: usize = calc_idx(ECC_A_SIGN_ADDR);
pub(crate) const ECC_A_ABS_IDX: usize = calc_idx(ECC_A_ABS_ADDR);
pub(crate) const ECC_B_IDX: usize = calc_idx(ECC_B_ADDR);
pub(crate) const ECC_P_IDX: usize = calc_idx(ECC_P_ADDR);
pub(crate) const ECC_N_IDX: usize = calc_idx(ECC_N_ADDR);
pub(crate) const ECC_P_R2_IDX: usize = calc_idx(ECC_P_R2_ADDR);
pub(crate) const ECC_MUL_K_IDX: usize = calc_idx(ECC_MUL_K_ADDR);
pub(crate) const ECC_MUL_IN_X_IDX: usize = calc_idx(ECC_MUL_IN_X_ADDR);
pub(crate) const ECC_MUL_IN_Y_IDX: usize = calc_idx(ECC_MUL_IN_Y_ADDR);
pub(crate) const ECC_OUT_X_IDX: usize = calc_idx(ECC_OUT_X_ADDR);
pub(crate) const ECC_OUT_Y_IDX: usize = calc_idx(ECC_OUT_Y_ADDR);

/// Result code written by the PKA when an ECC operation / point check succeeded
pub(crate) const ECC_RESULT_OK: u32 = 0xD60D;

// ---- ECC complete addition -------------------------------------------------

/// Curve field prime p (addition mode)
const ECC_ADD_P_ADDR: usize = 0x470;
/// First point P1 (projective)
const ECC_ADD_PT1_X_ADDR: usize = 0x628;
const ECC_ADD_PT1_Y_ADDR: usize = 0x680;
const ECC_ADD_PT1_Z_ADDR: usize = 0x6D8;
/// Second point P2 (projective)
const ECC_ADD_PT2_X_ADDR: usize = 0x730;
const ECC_ADD_PT2_Y_ADDR: usize = 0x788;
const ECC_ADD_PT2_Z_ADDR: usize = 0x7E0;
/// Result point
const ECC_ADD_OUT_X_ADDR: usize = 0xD60;
const ECC_ADD_OUT_Y_ADDR: usize = 0xDB8;

pub(crate) const ECC_ADD_P_IDX: usize = calc_idx(ECC_ADD_P_ADDR);
pub(crate) const ECC_ADD_PT1_X_IDX: usize = calc_idx(ECC_ADD_PT1_X_ADDR);
pub(crate) const ECC_ADD_PT1_Y_IDX: usize = calc_idx(ECC_ADD_PT1_Y_ADDR);
pub(crate) const ECC_ADD_PT1_Z_IDX: usize = calc_idx(ECC_ADD_PT1_Z_ADDR);
pub(crate) const ECC_ADD_PT2_X_IDX: usize = calc_idx(ECC_ADD_PT2_X_ADDR);
pub(crate) const ECC_ADD_PT2_Y_IDX: usize = calc_idx(ECC_ADD_PT2_Y_ADDR);
pub(crate) const ECC_ADD_PT2_Z_IDX: usize = calc_idx(ECC_ADD_PT2_Z_ADDR);
pub(crate) const ECC_ADD_OUT_X_IDX: usize = calc_idx(ECC_ADD_OUT_X_ADDR);
pub(crate) const ECC_ADD_OUT_Y_IDX: usize = calc_idx(ECC_ADD_OUT_Y_ADDR);

// ---- Point-on-curve check (FpCheck) ----------------------------------------
// FpCheck has no slots of its own here: the driver reuses the ones below.

/// Point X coordinate to check
pub(crate) const FPCHECK_X_IDX: usize = ECC_MUL_IN_X_IDX;
/// Point Y coordinate to check
pub(crate) const FPCHECK_Y_IDX: usize = ECC_OUT_Y_IDX;
/// Where the driver currently reads the check result. Verify this offset
/// against the reference manual.
pub(crate) const FPCHECK_RESULT_IDX: usize = ECC_ADD_PT1_Y_IDX;

/// R^2 mod p for NIST P-256, big-endian
pub(crate) const P256_R2_MOD_P: [u8; 32] = [
    0xff, 0xff, 0xff, 0xfc, 0xff, 0xff, 0xff, 0xfc, 0xff, 0xff, 0xff, 0xfb, 0xff, 0xff, 0xff, 0xf9,
    0xff, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x02,
];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SupportedOp {
    Addition,
    Multiplication,
    Inverse,
    Modulus,
}

impl OpAddition for SupportedOp {
    fn addition() -> Self {
        SupportedOp::Addition
    }
}

impl OpMultiplication for SupportedOp {
    fn multiplication() -> Self {
        SupportedOp::Multiplication
    }
}

impl OpInverse for SupportedOp {
    fn inverse() -> Self {
        SupportedOp::Inverse
    }
}

impl OpModulo for SupportedOp {
    fn modulo() -> Self {
        SupportedOp::Modulus
    }
}
