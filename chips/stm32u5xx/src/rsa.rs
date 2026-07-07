// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.


use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};


register_structs! {
    PkaRegisters {
        // PKA control register
        (0x00 => cr: ReadWrite<u32, CR::Register>),

        // PKA status register
        (0x04 => sr: ReadOnly<u32, SR::Register>),

        // PKA clear flag register
        (0x08 => clrfr: WriteOnly<u32, CLRFR::Register>),

        // PKA RAM
        (0x0400 => ram: [ReadWrite<u32>, 5336 / 4])

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
]
