// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;

register_structs! {
    pub PintRegisters {
        /// Pin Interrupt Mode register
        (0x00 => isel: ReadWrite<u32, ISEL::Register>),
        /// Pin interrupt level or rising edge interrupt enable register
        (0x04 => ienr: ReadWrite<u32, IENR::Register>),
        /// Pin interrupt level or rising edge interrupt set register
        (0x08 => sienr: WriteOnly<u32, SIENR::Register>),
        /// Pin interrupt level (rising edge interrupt) clear register
        (0x0C => cienr: WriteOnly<u32, CIENR::Register>),
        /// Pin interrupt active level or falling edge interrupt enable register
        (0x10 => ienf: ReadWrite<u32, IENF::Register>),
        /// Pin interrupt active level or falling edge interrupt set register
        (0x14 => sienf: WriteOnly<u32, SIENF::Register>),
        /// Pin interrupt active level or falling edge interrupt clear register
        (0x18 => cienf: WriteOnly<u32, CIENF::Register>),
        /// Pin interrupt rising edge register
        (0x1C => rise: ReadWrite<u32, RISE::Register>),
        /// Pin interrupt falling edge register
        (0x20 => fall: ReadWrite<u32, FALL::Register>),
        /// Pin interrupt status register
        (0x24 => ist: ReadWrite<u32, IST::Register>),
        (0x28 => @END),
    }
}
register_bitfields![u32,
ISEL [
    /// Selects the interrupt mode for each pin interrupt. Bit n configures the pin inte
    PMODE OFFSET(0) NUMBITS(8) []
],
IENR [
    /// Enables the rising edge or level interrupt for each pin interrupt. Bit n configu
    ENRL OFFSET(0) NUMBITS(8) []
],
SIENR [
    /// Ones written to this address set bits in the IENR, thus enabling interrupts. Bit
    SETENRL OFFSET(0) NUMBITS(8) []
],
CIENR [
    /// Ones written to this address clear bits in the IENR, thus disabling the interrup
    CENRL OFFSET(0) NUMBITS(8) []
],
IENF [
    /// Enables the falling edge or configures the active level interrupt for each pin i
    ENAF OFFSET(0) NUMBITS(8) []
],
SIENF [
    /// Ones written to this address set bits in the IENF, thus enabling interrupts. Bit
    SETENAF OFFSET(0) NUMBITS(8) []
],
CIENF [
    /// Ones written to this address clears bits in the IENF, thus disabling interrupts.
    CENAF OFFSET(0) NUMBITS(8) []
],
RISE [
    /// Rising edge detect. Bit n detects the rising edge of the pin selected in PINTSEL
    RDET OFFSET(0) NUMBITS(8) []
],
FALL [
    /// Falling edge detect. Bit n detects the falling edge of the pin selected in PINTS
    FDET OFFSET(0) NUMBITS(8) []
],
IST [
    /// Pin interrupt status. Bit n returns the status, clears the edge interrupt, or in
    PSTAT OFFSET(0) NUMBITS(8) []
],
PMCTRL [
    /// Specifies whether the 8 pin interrupts are controlled by the pin interrupt funct
    SEL_PMATCH OFFSET(0) NUMBITS(1) [
        /// Pin interrupt. Interrupts are driven in response to the standard pin interrupt f
        PinInterruptInterruptsAreDrivenInResponseToTheStandardPinInterruptFunction = 0,
        /// Pattern match. Interrupts are driven in response to pattern matches.
        PatternMatchInterruptsAreDrivenInResponseToPatternMatches = 1
    ],
    /// Enables the RXEV output to the CPU and/or to a GPIO output when the specified bo
    ENA_RXEV OFFSET(1) NUMBITS(1) [
        /// Disabled. RXEV output to the CPU is disabled.
        DisabledRXEVOutputToTheCPUIsDisabled = 0,
        /// Enabled. RXEV output to the CPU is enabled.
        EnabledRXEVOutputToTheCPUIsEnabled = 1
    ],
    /// This field displays the current state of pattern matches. A 1 in any bit of this
    PMAT OFFSET(24) NUMBITS(8) []
],
PMSRC [
    /// Selects the input source for bit slice 0
    SRC0 OFFSET(8) NUMBITS(3) [
        /// Input 0. Selects the pin selected in the PINTSEL0 register as the source to bit
        Input0SelectsThePinSelectedInThePINTSEL0RegisterAsTheSourceToBitSlice0 = 0,
        /// Input 1. Selects the pin selected in the PINTSEL1 register as the source to bit
        Input1SelectsThePinSelectedInThePINTSEL1RegisterAsTheSourceToBitSlice0 = 1,
        /// Input 2. Selects the pin selected in the PINTSEL2 register as the source to bit
        Input2SelectsThePinSelectedInThePINTSEL2RegisterAsTheSourceToBitSlice0 = 2,
        /// Input 3. Selects the pin selected in the PINTSEL3 register as the source to bit
        Input3SelectsThePinSelectedInThePINTSEL3RegisterAsTheSourceToBitSlice0 = 3,
        /// Input 4. Selects the pin selected in the PINTSEL4 register as the source to bit
        Input4SelectsThePinSelectedInThePINTSEL4RegisterAsTheSourceToBitSlice0 = 4,
        /// Input 5. Selects the pin selected in the PINTSEL5 register as the source to bit
        Input5SelectsThePinSelectedInThePINTSEL5RegisterAsTheSourceToBitSlice0 = 5,
        /// Input 6. Selects the pin selected in the PINTSEL6 register as the source to bit
        Input6SelectsThePinSelectedInThePINTSEL6RegisterAsTheSourceToBitSlice0 = 6,
        /// Input 7. Selects the pin selected in the PINTSEL7 register as the source to bit
        Input7SelectsThePinSelectedInThePINTSEL7RegisterAsTheSourceToBitSlice0 = 7
    ],
    /// Selects the input source for bit slice 1
    SRC1 OFFSET(11) NUMBITS(3) [
        /// Input 0. Selects the pin selected in the PINTSEL0 register as the source to bit
        Input0SelectsThePinSelectedInThePINTSEL0RegisterAsTheSourceToBitSlice1 = 0,
        /// Input 1. Selects the pin selected in the PINTSEL1 register as the source to bit
        Input1SelectsThePinSelectedInThePINTSEL1RegisterAsTheSourceToBitSlice1 = 1,
        /// Input 2. Selects the pin selected in the PINTSEL2 register as the source to bit
        Input2SelectsThePinSelectedInThePINTSEL2RegisterAsTheSourceToBitSlice1 = 2,
        /// Input 3. Selects the pin selected in the PINTSEL3 register as the source to bit
        Input3SelectsThePinSelectedInThePINTSEL3RegisterAsTheSourceToBitSlice1 = 3,
        /// Input 4. Selects the pin selected in the PINTSEL4 register as the source to bit
        Input4SelectsThePinSelectedInThePINTSEL4RegisterAsTheSourceToBitSlice1 = 4,
        /// Input 5. Selects the pin selected in the PINTSEL5 register as the source to bit
        Input5SelectsThePinSelectedInThePINTSEL5RegisterAsTheSourceToBitSlice1 = 5,
        /// Input 6. Selects the pin selected in the PINTSEL6 register as the source to bit
        Input6SelectsThePinSelectedInThePINTSEL6RegisterAsTheSourceToBitSlice1 = 6,
        /// Input 7. Selects the pin selected in the PINTSEL7 register as the source to bit
        Input7SelectsThePinSelectedInThePINTSEL7RegisterAsTheSourceToBitSlice1 = 7
    ],
    /// Selects the input source for bit slice 2
    SRC2 OFFSET(14) NUMBITS(3) [
        /// Input 0. Selects the pin selected in the PINTSEL0 register as the source to bit
        Input0SelectsThePinSelectedInThePINTSEL0RegisterAsTheSourceToBitSlice2 = 0,
        /// Input 1. Selects the pin selected in the PINTSEL1 register as the source to bit
        Input1SelectsThePinSelectedInThePINTSEL1RegisterAsTheSourceToBitSlice2 = 1,
        /// Input 2. Selects the pin selected in the PINTSEL2 register as the source to bit
        Input2SelectsThePinSelectedInThePINTSEL2RegisterAsTheSourceToBitSlice2 = 2,
        /// Input 3. Selects the pin selected in the PINTSEL3 register as the source to bit
        Input3SelectsThePinSelectedInThePINTSEL3RegisterAsTheSourceToBitSlice2 = 3,
        /// Input 4. Selects the pin selected in the PINTSEL4 register as the source to bit
        Input4SelectsThePinSelectedInThePINTSEL4RegisterAsTheSourceToBitSlice2 = 4,
        /// Input 5. Selects the pin selected in the PINTSEL5 register as the source to bit
        Input5SelectsThePinSelectedInThePINTSEL5RegisterAsTheSourceToBitSlice2 = 5,
        /// Input 6. Selects the pin selected in the PINTSEL6 register as the source to bit
        Input6SelectsThePinSelectedInThePINTSEL6RegisterAsTheSourceToBitSlice2 = 6,
        /// Input 7. Selects the pin selected in the PINTSEL7 register as the source to bit
        Input7SelectsThePinSelectedInThePINTSEL7RegisterAsTheSourceToBitSlice2 = 7
    ],
    /// Selects the input source for bit slice 3
    SRC3 OFFSET(17) NUMBITS(3) [
        /// Input 0. Selects the pin selected in the PINTSEL0 register as the source to bit
        Input0SelectsThePinSelectedInThePINTSEL0RegisterAsTheSourceToBitSlice3 = 0,
        /// Input 1. Selects the pin selected in the PINTSEL1 register as the source to bit
        Input1SelectsThePinSelectedInThePINTSEL1RegisterAsTheSourceToBitSlice3 = 1,
        /// Input 2. Selects the pin selected in the PINTSEL2 register as the source to bit
        Input2SelectsThePinSelectedInThePINTSEL2RegisterAsTheSourceToBitSlice3 = 2,
        /// Input 3. Selects the pin selected in the PINTSEL3 register as the source to bit
        Input3SelectsThePinSelectedInThePINTSEL3RegisterAsTheSourceToBitSlice3 = 3,
        /// Input 4. Selects the pin selected in the PINTSEL4 register as the source to bit
        Input4SelectsThePinSelectedInThePINTSEL4RegisterAsTheSourceToBitSlice3 = 4,
        /// Input 5. Selects the pin selected in the PINTSEL5 register as the source to bit
        Input5SelectsThePinSelectedInThePINTSEL5RegisterAsTheSourceToBitSlice3 = 5,
        /// Input 6. Selects the pin selected in the PINTSEL6 register as the source to bit
        Input6SelectsThePinSelectedInThePINTSEL6RegisterAsTheSourceToBitSlice3 = 6,
        /// Input 7. Selects the pin selected in the PINTSEL7 register as the source to bit
        Input7SelectsThePinSelectedInThePINTSEL7RegisterAsTheSourceToBitSlice3 = 7
    ],
    /// Selects the input source for bit slice 4
    SRC4 OFFSET(20) NUMBITS(3) [
        /// Input 0. Selects the pin selected in the PINTSEL0 register as the source to bit
        Input0SelectsThePinSelectedInThePINTSEL0RegisterAsTheSourceToBitSlice4 = 0,
        /// Input 1. Selects the pin selected in the PINTSEL1 register as the source to bit
        Input1SelectsThePinSelectedInThePINTSEL1RegisterAsTheSourceToBitSlice4 = 1,
        /// Input 2. Selects the pin selected in the PINTSEL2 register as the source to bit
        Input2SelectsThePinSelectedInThePINTSEL2RegisterAsTheSourceToBitSlice4 = 2,
        /// Input 3. Selects the pin selected in the PINTSEL3 register as the source to bit
        Input3SelectsThePinSelectedInThePINTSEL3RegisterAsTheSourceToBitSlice4 = 3,
        /// Input 4. Selects the pin selected in the PINTSEL4 register as the source to bit
        Input4SelectsThePinSelectedInThePINTSEL4RegisterAsTheSourceToBitSlice4 = 4,
        /// Input 5. Selects the pin selected in the PINTSEL5 register as the source to bit
        Input5SelectsThePinSelectedInThePINTSEL5RegisterAsTheSourceToBitSlice4 = 5,
        /// Input 6. Selects the pin selected in the PINTSEL6 register as the source to bit
        Input6SelectsThePinSelectedInThePINTSEL6RegisterAsTheSourceToBitSlice4 = 6,
        /// Input 7. Selects the pin selected in the PINTSEL7 register as the source to bit
        Input7SelectsThePinSelectedInThePINTSEL7RegisterAsTheSourceToBitSlice4 = 7
    ],
    /// Selects the input source for bit slice 5
    SRC5 OFFSET(23) NUMBITS(3) [
        /// Input 0. Selects the pin selected in the PINTSEL0 register as the source to bit
        Input0SelectsThePinSelectedInThePINTSEL0RegisterAsTheSourceToBitSlice5 = 0,
        /// Input 1. Selects the pin selected in the PINTSEL1 register as the source to bit
        Input1SelectsThePinSelectedInThePINTSEL1RegisterAsTheSourceToBitSlice5 = 1,
        /// Input 2. Selects the pin selected in the PINTSEL2 register as the source to bit
        Input2SelectsThePinSelectedInThePINTSEL2RegisterAsTheSourceToBitSlice5 = 2,
        /// Input 3. Selects the pin selected in the PINTSEL3 register as the source to bit
        Input3SelectsThePinSelectedInThePINTSEL3RegisterAsTheSourceToBitSlice5 = 3,
        /// Input 4. Selects the pin selected in the PINTSEL4 register as the source to bit
        Input4SelectsThePinSelectedInThePINTSEL4RegisterAsTheSourceToBitSlice5 = 4,
        /// Input 5. Selects the pin selected in the PINTSEL5 register as the source to bit
        Input5SelectsThePinSelectedInThePINTSEL5RegisterAsTheSourceToBitSlice5 = 5,
        /// Input 6. Selects the pin selected in the PINTSEL6 register as the source to bit
        Input6SelectsThePinSelectedInThePINTSEL6RegisterAsTheSourceToBitSlice5 = 6,
        /// Input 7. Selects the pin selected in the PINTSEL7 register as the source to bit
        Input7SelectsThePinSelectedInThePINTSEL7RegisterAsTheSourceToBitSlice5 = 7
    ],
    /// Selects the input source for bit slice 6
    SRC6 OFFSET(26) NUMBITS(3) [
        /// Input 0. Selects the pin selected in the PINTSEL0 register as the source to bit
        Input0SelectsThePinSelectedInThePINTSEL0RegisterAsTheSourceToBitSlice6 = 0,
        /// Input 1. Selects the pin selected in the PINTSEL1 register as the source to bit
        Input1SelectsThePinSelectedInThePINTSEL1RegisterAsTheSourceToBitSlice6 = 1,
        /// Input 2. Selects the pin selected in the PINTSEL2 register as the source to bit
        Input2SelectsThePinSelectedInThePINTSEL2RegisterAsTheSourceToBitSlice6 = 2,
        /// Input 3. Selects the pin selected in the PINTSEL3 register as the source to bit
        Input3SelectsThePinSelectedInThePINTSEL3RegisterAsTheSourceToBitSlice6 = 3,
        /// Input 4. Selects the pin selected in the PINTSEL4 register as the source to bit
        Input4SelectsThePinSelectedInThePINTSEL4RegisterAsTheSourceToBitSlice6 = 4,
        /// Input 5. Selects the pin selected in the PINTSEL5 register as the source to bit
        Input5SelectsThePinSelectedInThePINTSEL5RegisterAsTheSourceToBitSlice6 = 5,
        /// Input 6. Selects the pin selected in the PINTSEL6 register as the source to bit
        Input6SelectsThePinSelectedInThePINTSEL6RegisterAsTheSourceToBitSlice6 = 6,
        /// Input 7. Selects the pin selected in the PINTSEL7 register as the source to bit
        Input7SelectsThePinSelectedInThePINTSEL7RegisterAsTheSourceToBitSlice6 = 7
    ],
    /// Selects the input source for bit slice 7
    SRC7 OFFSET(29) NUMBITS(3) [
        /// Input 0. Selects the pin selected in the PINTSEL0 register as the source to bit
        Input0SelectsThePinSelectedInThePINTSEL0RegisterAsTheSourceToBitSlice7 = 0,
        /// Input 1. Selects the pin selected in the PINTSEL1 register as the source to bit
        Input1SelectsThePinSelectedInThePINTSEL1RegisterAsTheSourceToBitSlice7 = 1,
        /// Input 2. Selects the pin selected in the PINTSEL2 register as the source to bit
        Input2SelectsThePinSelectedInThePINTSEL2RegisterAsTheSourceToBitSlice7 = 2,
        /// Input 3. Selects the pin selected in the PINTSEL3 register as the source to bit
        Input3SelectsThePinSelectedInThePINTSEL3RegisterAsTheSourceToBitSlice7 = 3,
        /// Input 4. Selects the pin selected in the PINTSEL4 register as the source to bit
        Input4SelectsThePinSelectedInThePINTSEL4RegisterAsTheSourceToBitSlice7 = 4,
        /// Input 5. Selects the pin selected in the PINTSEL5 register as the source to bit
        Input5SelectsThePinSelectedInThePINTSEL5RegisterAsTheSourceToBitSlice7 = 5,
        /// Input 6. Selects the pin selected in the PINTSEL6 register as the source to bit
        Input6SelectsThePinSelectedInThePINTSEL6RegisterAsTheSourceToBitSlice7 = 6,
        /// Input 7. Selects the pin selected in the PINTSEL7 register as the source to bit
        Input7SelectsThePinSelectedInThePINTSEL7RegisterAsTheSourceToBitSlice7 = 7
    ]
],
PMCFG [
    /// Determines whether slice 0 is an endpoint.
    PROD_ENDPTS0 OFFSET(0) NUMBITS(1) [
        /// No effect. Slice 0 is not an endpoint.
        NoEffectSlice0IsNotAnEndpoint = 0,
        /// endpoint. Slice 0 is the endpoint of a product term (minterm). Pin interrupt 0 i
        ENDPOINT = 1
    ],
    /// Determines whether slice 1 is an endpoint.
    PROD_ENDPTS1 OFFSET(1) NUMBITS(1) [
        /// No effect. Slice 1 is not an endpoint.
        NoEffectSlice1IsNotAnEndpoint = 0,
        /// endpoint. Slice 1 is the endpoint of a product term (minterm). Pin interrupt 1 i
        ENDPOINT = 1
    ],
    /// Determines whether slice 2 is an endpoint.
    PROD_ENDPTS2 OFFSET(2) NUMBITS(1) [
        /// No effect. Slice 2 is not an endpoint.
        NoEffectSlice2IsNotAnEndpoint = 0,
        /// endpoint. Slice 2 is the endpoint of a product term (minterm). Pin interrupt 2 i
        ENDPOINT = 1
    ],
    /// Determines whether slice 3 is an endpoint.
    PROD_ENDPTS3 OFFSET(3) NUMBITS(1) [
        /// No effect. Slice 3 is not an endpoint.
        NoEffectSlice3IsNotAnEndpoint = 0,
        /// endpoint. Slice 3 is the endpoint of a product term (minterm). Pin interrupt 3 i
        ENDPOINT = 1
    ],
    /// Determines whether slice 4 is an endpoint.
    PROD_ENDPTS4 OFFSET(4) NUMBITS(1) [
        /// No effect. Slice 4 is not an endpoint.
        NoEffectSlice4IsNotAnEndpoint = 0,
        /// endpoint. Slice 4 is the endpoint of a product term (minterm). Pin interrupt 4 i
        ENDPOINT = 1
    ],
    /// Determines whether slice 5 is an endpoint.
    PROD_ENDPTS5 OFFSET(5) NUMBITS(1) [
        /// No effect. Slice 5 is not an endpoint.
        NoEffectSlice5IsNotAnEndpoint = 0,
        /// endpoint. Slice 5 is the endpoint of a product term (minterm). Pin interrupt 5 i
        ENDPOINT = 1
    ],
    /// Determines whether slice 6 is an endpoint.
    PROD_ENDPTS6 OFFSET(6) NUMBITS(1) [
        /// No effect. Slice 6 is not an endpoint.
        NoEffectSlice6IsNotAnEndpoint = 0,
        /// endpoint. Slice 6 is the endpoint of a product term (minterm). Pin interrupt 6 i
        ENDPOINT = 1
    ],
    /// Specifies the match contribution condition for bit slice 0.
    CFG0 OFFSET(8) NUMBITS(3) [
        /// Constant HIGH. This bit slice always contributes to a product term match.
        ConstantHIGHThisBitSliceAlwaysContributesToAProductTermMatch = 0,
        /// Sticky rising edge. Match occurs if a rising edge on the specified input has occ
        STICKY_RISING_EDGE = 1,
        /// Sticky falling edge. Match occurs if a falling edge on the specified input has o
        STICKY_FALLING_EDGE = 2,
        /// Sticky rising or falling edge. Match occurs if either a rising or falling edge o
        STICKY_RISING_FALLING_EDGE = 3,
        /// High level. Match (for this bit slice) occurs when there is a high level on the
        HIGH_LEVEL = 4,
        /// Low level. Match occurs when there is a low level on the specified input.
        LowLevelMatchOccursWhenThereIsALowLevelOnTheSpecifiedInput = 5,
        /// Constant 0. This bit slice never contributes to a match (should be used to disab
        CONSTANT_ZERO = 6,
        /// Event. Non-sticky rising or falling edge. Match occurs on an event - i.e. when e
        EVENT = 7
    ],
    /// Specifies the match contribution condition for bit slice 1.
    CFG1 OFFSET(11) NUMBITS(3) [
        /// Constant HIGH. This bit slice always contributes to a product term match.
        ConstantHIGHThisBitSliceAlwaysContributesToAProductTermMatch = 0,
        /// Sticky rising edge. Match occurs if a rising edge on the specified input has occ
        STICKY_RISING_EDGE = 1,
        /// Sticky falling edge. Match occurs if a falling edge on the specified input has o
        STICKY_FALLING_EDGE = 2,
        /// Sticky rising or falling edge. Match occurs if either a rising or falling edge o
        STICKY_RISING_FALLING_EDGE = 3,
        /// High level. Match (for this bit slice) occurs when there is a high level on the
        HIGH_LEVEL = 4,
        /// Low level. Match occurs when there is a low level on the specified input.
        LowLevelMatchOccursWhenThereIsALowLevelOnTheSpecifiedInput = 5,
        /// Constant 0. This bit slice never contributes to a match (should be used to disab
        CONSTANT_ZERO = 6,
        /// Event. Non-sticky rising or falling edge. Match occurs on an event - i.e. when e
        EVENT = 7
    ],
    /// Specifies the match contribution condition for bit slice 2.
    CFG2 OFFSET(14) NUMBITS(3) [
        /// Constant HIGH. This bit slice always contributes to a product term match.
        ConstantHIGHThisBitSliceAlwaysContributesToAProductTermMatch = 0,
        /// Sticky rising edge. Match occurs if a rising edge on the specified input has occ
        STICKY_RISING_EDGE = 1,
        /// Sticky falling edge. Match occurs if a falling edge on the specified input has o
        STICKY_FALLING_EDGE = 2,
        /// Sticky rising or falling edge. Match occurs if either a rising or falling edge o
        STICKY_RISING_FALLING_EDGE = 3,
        /// High level. Match (for this bit slice) occurs when there is a high level on the
        HIGH_LEVEL = 4,
        /// Low level. Match occurs when there is a low level on the specified input.
        LowLevelMatchOccursWhenThereIsALowLevelOnTheSpecifiedInput = 5,
        /// Constant 0. This bit slice never contributes to a match (should be used to disab
        CONSTANT_ZERO = 6,
        /// Event. Non-sticky rising or falling edge. Match occurs on an event - i.e. when e
        EVENT = 7
    ],
    /// Specifies the match contribution condition for bit slice 3.
    CFG3 OFFSET(17) NUMBITS(3) [
        /// Constant HIGH. This bit slice always contributes to a product term match.
        ConstantHIGHThisBitSliceAlwaysContributesToAProductTermMatch = 0,
        /// Sticky rising edge. Match occurs if a rising edge on the specified input has occ
        STICKY_RISING_EDGE = 1,
        /// Sticky falling edge. Match occurs if a falling edge on the specified input has o
        STICKY_FALLING_EDGE = 2,
        /// Sticky rising or falling edge. Match occurs if either a rising or falling edge o
        STICKY_RISING_FALLING_EDGE = 3,
        /// High level. Match (for this bit slice) occurs when there is a high level on the
        HIGH_LEVEL = 4,
        /// Low level. Match occurs when there is a low level on the specified input.
        LowLevelMatchOccursWhenThereIsALowLevelOnTheSpecifiedInput = 5,
        /// Constant 0. This bit slice never contributes to a match (should be used to disab
        CONSTANT_ZERO = 6,
        /// Event. Non-sticky rising or falling edge. Match occurs on an event - i.e. when e
        EVENT = 7
    ],
    /// Specifies the match contribution condition for bit slice 4.
    CFG4 OFFSET(20) NUMBITS(3) [
        /// Constant HIGH. This bit slice always contributes to a product term match.
        ConstantHIGHThisBitSliceAlwaysContributesToAProductTermMatch = 0,
        /// Sticky rising edge. Match occurs if a rising edge on the specified input has occ
        STICKY_RISING_EDGE = 1,
        /// Sticky falling edge. Match occurs if a falling edge on the specified input has o
        STICKY_FALLING_EDGE = 2,
        /// Sticky rising or falling edge. Match occurs if either a rising or falling edge o
        STICKY_RISING_FALLING_EDGE = 3,
        /// High level. Match (for this bit slice) occurs when there is a high level on the
        HIGH_LEVEL = 4,
        /// Low level. Match occurs when there is a low level on the specified input.
        LowLevelMatchOccursWhenThereIsALowLevelOnTheSpecifiedInput = 5,
        /// Constant 0. This bit slice never contributes to a match (should be used to disab
        CONSTANT_ZERO = 6,
        /// Event. Non-sticky rising or falling edge. Match occurs on an event - i.e. when e
        EVENT = 7
    ],
    /// Specifies the match contribution condition for bit slice 5.
    CFG5 OFFSET(23) NUMBITS(3) [
        /// Constant HIGH. This bit slice always contributes to a product term match.
        ConstantHIGHThisBitSliceAlwaysContributesToAProductTermMatch = 0,
        /// Sticky rising edge. Match occurs if a rising edge on the specified input has occ
        STICKY_RISING_EDGE = 1,
        /// Sticky falling edge. Match occurs if a falling edge on the specified input has o
        STICKY_FALLING_EDGE = 2,
        /// Sticky rising or falling edge. Match occurs if either a rising or falling edge o
        STICKY_RISING_FALLING_EDGE = 3,
        /// High level. Match (for this bit slice) occurs when there is a high level on the
        HIGH_LEVEL = 4,
        /// Low level. Match occurs when there is a low level on the specified input.
        LowLevelMatchOccursWhenThereIsALowLevelOnTheSpecifiedInput = 5,
        /// Constant 0. This bit slice never contributes to a match (should be used to disab
        CONSTANT_ZERO = 6,
        /// Event. Non-sticky rising or falling edge. Match occurs on an event - i.e. when e
        EVENT = 7
    ],
    /// Specifies the match contribution condition for bit slice 6.
    CFG6 OFFSET(26) NUMBITS(3) [
        /// Constant HIGH. This bit slice always contributes to a product term match.
        ConstantHIGHThisBitSliceAlwaysContributesToAProductTermMatch = 0,
        /// Sticky rising edge. Match occurs if a rising edge on the specified input has occ
        STICKY_RISING_EDGE = 1,
        /// Sticky falling edge. Match occurs if a falling edge on the specified input has o
        STICKY_FALLING_EDGE = 2,
        /// Sticky rising or falling edge. Match occurs if either a rising or falling edge o
        STICKY_RISING_FALLING_EDGE = 3,
        /// High level. Match (for this bit slice) occurs when there is a high level on the
        HIGH_LEVEL = 4,
        /// Low level. Match occurs when there is a low level on the specified input.
        LowLevelMatchOccursWhenThereIsALowLevelOnTheSpecifiedInput = 5,
        /// Constant 0. This bit slice never contributes to a match (should be used to disab
        CONSTANT_ZERO = 6,
        /// Event. Non-sticky rising or falling edge. Match occurs on an event - i.e. when e
        EVENT = 7
    ],
    /// Specifies the match contribution condition for bit slice 7.
    CFG7 OFFSET(29) NUMBITS(3) [
        /// Constant HIGH. This bit slice always contributes to a product term match.
        ConstantHIGHThisBitSliceAlwaysContributesToAProductTermMatch = 0,
        /// Sticky rising edge. Match occurs if a rising edge on the specified input has occ
        STICKY_RISING_EDGE = 1,
        /// Sticky falling edge. Match occurs if a falling edge on the specified input has o
        STICKY_FALLING_EDGE = 2,
        /// Sticky rising or falling edge. Match occurs if either a rising or falling edge o
        STICKY_RISING_FALLING_EDGE = 3,
        /// High level. Match (for this bit slice) occurs when there is a high level on the
        HIGH_LEVEL = 4,
        /// Low level. Match occurs when there is a low level on the specified input.
        LowLevelMatchOccursWhenThereIsALowLevelOnTheSpecifiedInput = 5,
        /// Constant 0. This bit slice never contributes to a match (should be used to disab
        CONSTANT_ZERO = 6,
        /// Event. Non-sticky rising or falling edge. Match occurs on an event - i.e. when e
        EVENT = 7
    ]
]
];
pub(crate) const PINT_BASE: StaticRef<PintRegisters> =
    unsafe { StaticRef::new(0x50004000 as *const PintRegisters) };

#[derive(Clone, Copy)]
pub enum Edge {
    Rising,
    Falling,
    Both,
}

pub struct Pint<'a> {
    registers: StaticRef<PintRegisters>,
    clients: [OptionalCell<&'a dyn kernel::hil::gpio::Client>; 8],
}

// pub static PINT: Pint = Pint::new();

impl<'a> Pint<'a> {
    pub const fn new() -> Self {
        Self {
            registers: PINT_BASE,
            clients: [
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
                OptionalCell::empty(),
            ],
        }
    }

    // pub fn find_and_take_channel(&self) -> Option<u8> {
    //     for i in 0..self.clients.len() {
    //         if self.clients[i].is_none() {
    //             self.clients[i].put(unsafe {core::mem::transmute(&())});
    //             return Some(i as u8);
    //         }
    //     }

    //     None
    // }

    // pub fn select_pin(&self, pin_num: usize, channel: u8) {
    //     if channel < 8 {
    //     inputmux::INPUTMUX.pintsel[channel as usize].set(pin_num as u32);
    //     }
    // }

    pub fn set_client(&self, channel: u8, client: &'a dyn kernel::hil::gpio::Client) {
        if channel < 8 {
            self.clients[channel as usize].replace(client);
        }
    }

    pub fn configure_interrupt(&self, channel: usize, edge: Edge) {
        if channel < 8 {
            let mask = 1 << channel;

            self.registers.isel.modify(ISEL::PMODE.val(!mask));
            // self.registers.rise.modify(RISE::RDET.val(mask));
            // self.registers.fall.modify(FALL::FDET.val(mask));

            match edge {
                Edge::Rising => {
                    self.registers.sienr.write(SIENR::SETENRL.val(mask));
                    self.registers.cienf.write(CIENF::CENAF.val(mask));
                }
                Edge::Falling => {
                    self.registers.sienf.write(SIENF::SETENAF.val(mask));
                    self.registers.cienr.write(CIENR::CENRL.val(mask));
                }
                Edge::Both => {
                    self.registers.sienr.write(SIENR::SETENRL.val(mask));
                    self.registers.sienf.write(SIENF::SETENAF.val(mask));
                }
            }
        }

        // self.registers.isel.modify(ISEL::PMODE.val(1));
        // self.registers.ienr.modify(IENR::ENRL.val(1));
        // self.registers.ienf.modify(IENF::ENAF.val(1));
    }

    // pub fn disable_and_free_channel(&mut self, channel: u8) {
    //     if channel < 8 {
    //     let mask = 1 << channel;

    //     self.registers.cienr.write(CIENR::CENRL.val(mask));
    //     self.registers.cienf.write(CIENF::CENAF.val(mask));
    //     // self.clients[channel as usize].take();
    //     }

    // }

    pub fn handle_interrupt(&self) {
        let status = self.registers.ist.get();

        self.registers.rise.write(RISE::RDET.val(status));
        self.registers.fall.write(FALL::FDET.val(status));

        // self.registers.ist.get();

        // let blue_led = GpioPin::new(LPCPin::P1_6);

        for i in 0..8 {
            if (status & (1 << i)) != 0 {
                self.registers.ist.write(IST::PSTAT.val(1 << i));
                // hprintln!("IST loop {}", self.registers.ist.get());

                self.clients[i].map(|client| client.fired());
            }
        }

        // blue_led.toggle();
        // Self::delay_ms(1000);
        // blue_led.toggle();

        // self.configure_interrupt(0, Edge::Rising);
    }

    pub fn disable_interrupt(&self, channel: usize) {
        if channel < 8 {
            let mask = 1 << channel;

            self.registers.cienr.write(CIENR::CENRL.val(mask));
            self.registers.cienf.write(CIENF::CENAF.val(mask));
        }
    }

    pub fn read_interrupt(&self) -> u32 {
        self.registers.rise.get()
    }
}
