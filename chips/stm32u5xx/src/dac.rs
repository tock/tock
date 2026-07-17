// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::hil::dac::DacChannel;
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::registers::{ReadWrite, register_bitfields, register_structs};

register_structs! {
    pub DacRegisters {
        /// control register
        (0x000 => cr: ReadWrite<u32, CR::Register>),
        /// software trigger register
        (0x004 => swtrgr: ReadWrite<u32>),
        /// channel1 12-bit right-aligned data holding register
        (0x008 => dhr12r1: ReadWrite<u32>),
        /// channel1 12-bit left-aligned data holding register
        (0x00C => dhr12l1: ReadWrite<u32>),
        /// channel1 8-bit right-aligned data holding register
        (0x010 => dhr8r1: ReadWrite<u32>),
        /// channel2 12-bit right-aligned data holding register
        (0x014 => dhr12r2: ReadWrite<u32>),
        /// channel2 12-bit left-aligned data holding register
        (0x018 => dhr12l2: ReadWrite<u32>),
        /// channel2 8-bit right-aligned data holding register
        (0x01C => dhr8r2: ReadWrite<u32>),
        /// dual dac 12-bit right-aligned data holding register
        (0x020 => dhr12rd: ReadWrite<u32>),
        /// dual dac 12-bit left-aligned data holding register
        (0x024 => dhr12ld: ReadWrite<u32>),
        /// dual dac 8-bit right-aligned data holding register
        (0x028 => dhr8rd: ReadWrite<u32>),
        /// channel1 data output register
        (0x02C => dor1: ReadWrite<u32>),
        /// channel2 data output register
        (0x030 => dor2: ReadWrite<u32>),
        /// status register
        (0x034 => sr: ReadWrite<u32>),
        /// calibration control register
        (0x038 => ccr: ReadWrite<u32>),
        /// mode control register
        (0x03C => mcr: ReadWrite<u32, MCR::Register>),
        /// channel1 sample and hold sample time register
        (0x040 => shsr1: ReadWrite<u32>),
        /// channel2 sample and hold sample time register
        (0x044 => shsr2: ReadWrite<u32>),
        /// sample and hold time register
        (0x048 => shhr: ReadWrite<u32>),
        /// sample and hold refresh time register
        (0x04C => shrr: ReadWrite<u32>),
        /// 0x050 reserved
        (0x050 => _reserved0),
        /// autonomous mode control register
        (0x054 => autocr: ReadWrite<u32>),
        (0x058 => @END),
    }
}

register_bitfields! [u32,
    pub CR [
        /// channel1 enable
        EN1 OFFSET(0) NUMBITS(1) [],
        /// channel1 trigger enable
        TEN1 OFFSET(1) NUMBITS(1) [],
        /// channel1 trigger selection
        TSEL1 OFFSET(2) NUMBITS(4) [
            SWTRIG1 = 0,
            dac_ch1_trg1 = 1,
            dac_ch1_trg2 = 2,
            dac_ch1_trg3 = 3,
            dac_ch1_trg4 = 4,
            dac_ch1_trg5 = 5,
            dac_ch1_trg6 = 6,
            dac_ch1_trg7 = 7,
            dac_ch1_trg8 = 8,
            dac_ch1_trg9 = 9,
            dac_ch1_trg10 = 10,
            dac_ch1_trg11 = 11,
            dac_ch1_trg12 = 12,
            dac_ch1_trg13 = 13,
            dac_ch1_trg14 = 14,
            dac_ch1_trg15 = 15,
        ],
        /// channel1 noise/triangle wave generation
        WAVE1 OFFSET(6) NUMBITS(2) [
            disabled = 0,
            noise_wave = 1,
            triangle_wave = 2
        ],
        /// mask in noise wave generation mode or ampl in triangle gen mode
        MAMP1 OFFSET(8) NUMBITS(4) [
            /// unmask bit0 of LFSR / triangle amplitude 1
            AMP1 = 0,
            /// unmask bits[1:0] of LFSR / triangle amplitude 3
            AMP3 = 1,
            /// unmask bits[2:0] of LFSR / triangle amplitude 7
            AMP7 = 2,
            /// unmask bits[3:0] of LFSR / triangle amplitude 15
            AMP15 = 3,
            /// unmask bits[4:0] of LFSR / triangle amplitude 31
            AMP31 = 4,
            /// unmask bits[5:0] of LFSR / triangle amplitude 63
            AMP63 = 5,
            /// unmask bits[6:0] of LFSR / triangle amplitude 127
            AMP127 = 6,
            /// unmask bits[7:0] of LFSR / triangle amplitude 255
            AMP255 = 7,
            /// unmask bits[8:0] of LFSR / triangle amplitude 511
            AMP511 = 8,
            /// unmask bits[9:0] of LFSR / triangle amplitude 1023
            AMP1023 = 9,
            /// unmask bits[10:0] of LFSR / triangle amplitude 2047
            AMP2047 = 10,
            /// unmask bits[11:0] of LFSR / triangle amplitude 4095 (>= 1011)
            AMP4095 = 11,
        ],
        /// channel 1 dma enable
        DMAEN1 OFFSET(12) NUMBITS(1) [],
        /// channel1 DMA underrun interrupt enable
        DMAUDRIE1 OFFSET(13) NUMBITS(1) [],
        /// channel1 calibration enable
        CEN1 OFFSET(14) NUMBITS(1) [],
        /// channel2 enable
        EN2 OFFSET(16) NUMBITS(1) [],
        /// channel2 trigger enable
        TEN2 OFFSET(17) NUMBITS(1) [],
        /// channel2 trigger selection
        TSEL2 OFFSET(18) NUMBITS(4) [
            SWTRIG2 = 0,
            dac_ch2_trg1=1,
            dac_ch2_trg2=2,
            dac_ch2_trg3=3,
            dac_ch2_trg4=4,
            dac_ch2_trg5=5,
            dac_ch2_trg6=6,
            dac_ch2_trg7=7,
            dac_ch2_trg8=8,
            dac_ch2_trg9=9,
            dac_ch2_trg10=10,
            dac_ch2_trg11=11,
            dac_ch2_trg12=12,
            dac_ch2_trg13=13,
            dac_ch2_trg14=14,
            dac_ch2_trg15=15,
        ],
        /// channel2 noise/triangle wave generation
        WAVE2 OFFSET(22) NUMBITS(2) [
            disabled=0,
            noise_wave=1,
            triangle_wave=2
        ],
        /// mask in noise wave generation mode or ampl in triangle gen mode
        MAMP2 OFFSET(24) NUMBITS(4) [
            /// unmask bit0 of LFSR / triangle amplitude 1
            AMP1 = 0,
            /// unmask bits[1:0] of LFSR / triangle amplitude 3
            AMP3 = 1,
            /// unmask bits[2:0] of LFSR / triangle amplitude 7
            AMP7 = 2,
            /// unmask bits[3:0] of LFSR / triangle amplitude 15
            AMP15 = 3,
            /// unmask bits[4:0] of LFSR / triangle amplitude 31
            AMP31 = 4,
            /// unmask bits[5:0] of LFSR / triangle amplitude 63
            AMP63 = 5,
            /// unmask bits[6:0] of LFSR / triangle amplitude 127
            AMP127 = 6,
            /// unmask bits[7:0] of LFSR / triangle amplitude 255
            AMP255 = 7,
            /// unmask bits[8:0] of LFSR / triangle amplitude 511
            AMP511 = 8,
            /// unmask bits[9:0] of LFSR / triangle amplitude 1023
            AMP1023 = 9,
            /// unmask bits[10:0] of LFSR / triangle amplitude 2047
            AMP2047 = 10,
            /// unmask bits[11:0] of LFSR / triangle amplitude 4095 (>= 1011)
            AMP4095 = 11,
        ],
        /// dma channel2 enable
        DMAEN2 OFFSET(28) NUMBITS(1) [],
        /// ch2 dma underrun interrupt enable
        DMAUDRIE2 OFFSET(29) NUMBITS(1) [],
        /// ch2 calibration enable
        CEN2 OFFSET(30) NUMBITS(1) []
    ],
   /// mode control register
    pub MCR [
        /// dac ch1 mode
        MODE1 OFFSET(0) NUMBITS(3) [
            /// CH1 in normal mode
            EXT_PIN_BUF_EN = 0,
            EXT_PIN_PERI_BUF_EN = 1,
            EXT_PIN_BUF_NEN = 2,
            PERI_BUF_NEN = 3,
            /// CH1 in sample and hold
            SH_EXT_PIN_BUF_EN = 4,
            SH_EXT_PIN_PERI_BUF_EN = 5,
            SH_EXT_PIN_PERI_BUF_NEN = 6,
            SH_PERI_BUF_NEN = 7
        ],
        /// ch1 dma double mode
        DMADOUBLE1 OFFSET(8) NUMBITS(1) [],
        /// signed formar for ch1
        SINFORMAT1 OFFSET(9) NUMBITS(1) [],
        HFSET OFFSET(14) NUMBITS(2) [
            disabled = 0,
            enabled_ahb_80 = 1,
            enabled_ahb_160 = 2
        ]
    ]
];

pub const DAC_BASE: StaticRef<DacRegisters> =
    unsafe { StaticRef::new(0x4602_1800 as *const DacRegisters) };

/// The DAC takes in an arbitrary 12 bit number and outputs a voltage proportional to it.
pub struct Dac {
    registers: StaticRef<DacRegisters>,
    enable_clock: fn(),
    initialized: Cell<bool>,
}

impl Dac {
    /// Creates a new instance of the driver.
    ///
    /// - `base`: The StaticRef pointing to the MMIO base address of the peripheral.
    /// - `enable_clock`: A callback function to provide the peripheral clock via RCC.
    /// - `initialized``: Bool cell that tracks whether the DAC has been initialized yet.
    pub fn new(base: StaticRef<DacRegisters>, enable_clock: fn()) -> Self {
        Self {
            registers: base,
            enable_clock,
            initialized: Cell::new(false),
        }
    }

    fn enable_clock(&self) {
        (self.enable_clock)();
    }

    /// Initialization function that only gets called on first set_value write.
    /// Because the HIL only exposes the set_value function, we have to store the initialization state as a bool to make sure we only initialize once.
    /// Configures the MODE1 register and then enables CH1 of the DAC. MODE1 is set for clarity as it set to 0 regardless and that's what we need.
    fn initialize(&self) {
        self.enable_clock();
        self.registers.mcr.modify(MCR::MODE1::EXT_PIN_BUF_EN);
        self.registers.cr.modify(CR::EN1::SET);
        self.initialized.set(true);
    }
}

impl DacChannel for Dac {
    /// Bound checks the value and if ok, writes it to the 12-bit right aligned DAC register after padding.
    fn set_value(&self, value: usize) -> Result<(), ErrorCode> {
        if !self.initialized.get() {
            self.initialize();
        }
        if value > 0xFFF {
            Err(ErrorCode::FAIL)
        } else {
            self.registers.dhr12r1.set((value as u32) & 0xFFF);
            Ok(())
        }
    }
}
