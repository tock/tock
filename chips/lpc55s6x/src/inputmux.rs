// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;

register_structs! {
    /// Input multiplexing (INPUT MUX)
    pub InputmuxRegisters {
        /// Input mux register for SCT0 input
        (0x000 => sct0_inmux_0: ReadWrite<u32>),
        /// Input mux register for SCT0 input
        (0x004 => sct0_inmux_1: ReadWrite<u32>),
        /// Input mux register for SCT0 input
        (0x008 => sct0_inmux_2: ReadWrite<u32>),
        /// Input mux register for SCT0 input
        (0x00C => sct0_inmux_3: ReadWrite<u32>),
        /// Input mux register for SCT0 input
        (0x010 => sct0_inmux_4: ReadWrite<u32>),
        /// Input mux register for SCT0 input
        (0x014 => sct0_inmux_5: ReadWrite<u32>),
        /// Input mux register for SCT0 input
        (0x018 => sct0_inmux_6: ReadWrite<u32>),
        (0x01C => _reserved0),
        /// Capture select registers for TIMER0 inputs
        (0x020 => timer0captsel_0: ReadWrite<u32>),
        /// Capture select registers for TIMER0 inputs
        (0x024 => timer0captsel_1: ReadWrite<u32>),
        /// Capture select registers for TIMER0 inputs
        (0x028 => timer0captsel_2: ReadWrite<u32>),
        /// Capture select registers for TIMER0 inputs
        (0x02C => timer0captsel_3: ReadWrite<u32>),
        (0x030 => _reserved1),
        /// Capture select registers for TIMER1 inputs
        (0x040 => timer1captsel_0: ReadWrite<u32>),
        /// Capture select registers for TIMER1 inputs
        (0x044 => timer1captsel_1: ReadWrite<u32>),
        /// Capture select registers for TIMER1 inputs
        (0x048 => timer1captsel_2: ReadWrite<u32>),
        /// Capture select registers for TIMER1 inputs
        (0x04C => timer1captsel_3: ReadWrite<u32>),
        (0x050 => _reserved2),
        /// Capture select registers for TIMER2 inputs
        (0x060 => timer2captsel_0: ReadWrite<u32>),
        /// Capture select registers for TIMER2 inputs
        (0x064 => timer2captsel_1: ReadWrite<u32>),
        /// Capture select registers for TIMER2 inputs
        (0x068 => timer2captsel_2: ReadWrite<u32>),
        /// Capture select registers for TIMER2 inputs
        (0x06C => timer2captsel_3: ReadWrite<u32>),
        (0x070 => _reserved3),
        /// Pin interrupt select register
        (0x0C0 => pintsel_0: ReadWrite<u32, PINTSEL::Register>),
        /// Pin interrupt select register
        (0x0C4 => pintsel_1: ReadWrite<u32, PINTSEL::Register>),
        /// Pin interrupt select register
        (0x0C8 => pintsel_2: ReadWrite<u32, PINTSEL::Register>),
        /// Pin interrupt select register
        (0x0CC => pintsel_3: ReadWrite<u32, PINTSEL::Register>),
        /// Pin interrupt select register
        (0x0D0 => pintsel_4: ReadWrite<u32, PINTSEL::Register>),
        /// Pin interrupt select register
        (0x0D4 => pintsel_5: ReadWrite<u32, PINTSEL::Register>),
        /// Pin interrupt select register
        (0x0D8 => pintsel_6: ReadWrite<u32, PINTSEL::Register>),
        /// Pin interrupt select register
        (0x0DC => pintsel_7: ReadWrite<u32, PINTSEL::Register>),
        /// Trigger select register for DMA0 channel
        (0x0E0 => dma0_itrig_inmux_0: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x0E4 => dma0_itrig_inmux_1: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x0E8 => dma0_itrig_inmux_2: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x0EC => dma0_itrig_inmux_3: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x0F0 => dma0_itrig_inmux_4: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x0F4 => dma0_itrig_inmux_5: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x0F8 => dma0_itrig_inmux_6: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x0FC => dma0_itrig_inmux_7: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x100 => dma0_itrig_inmux_8: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x104 => dma0_itrig_inmux_9: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x108 => dma0_itrig_inmux_10: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x10C => dma0_itrig_inmux_11: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x110 => dma0_itrig_inmux_12: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x114 => dma0_itrig_inmux_13: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x118 => dma0_itrig_inmux_14: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x11C => dma0_itrig_inmux_15: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x120 => dma0_itrig_inmux_16: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x124 => dma0_itrig_inmux_17: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x128 => dma0_itrig_inmux_18: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x12C => dma0_itrig_inmux_19: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x130 => dma0_itrig_inmux_20: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x134 => dma0_itrig_inmux_21: ReadWrite<u32>),
        /// Trigger select register for DMA0 channel
        (0x138 => dma0_itrig_inmux_22: ReadWrite<u32>),
        (0x13C => _reserved4),
        /// DMA0 output trigger selection to become DMA0 trigger
        (0x160 => dma0_otrig_inmux_0: ReadWrite<u32>),
        /// DMA0 output trigger selection to become DMA0 trigger
        (0x164 => dma0_otrig_inmux_1: ReadWrite<u32>),
        /// DMA0 output trigger selection to become DMA0 trigger
        (0x168 => dma0_otrig_inmux_2: ReadWrite<u32>),
        /// DMA0 output trigger selection to become DMA0 trigger
        (0x16C => dma0_otrig_inmux_3: ReadWrite<u32>),
        (0x170 => _reserved5),
        /// Selection for frequency measurement reference clock
        (0x180 => freqmeas_ref: ReadWrite<u32>),
        /// Selection for frequency measurement target clock
        (0x184 => freqmeas_target: ReadWrite<u32>),
        (0x188 => _reserved6),
        /// Capture select registers for TIMER3 inputs
        (0x1A0 => timer3captsel_0: ReadWrite<u32>),
        /// Capture select registers for TIMER3 inputs
        (0x1A4 => timer3captsel_1: ReadWrite<u32>),
        /// Capture select registers for TIMER3 inputs
        (0x1A8 => timer3captsel_2: ReadWrite<u32>),
        /// Capture select registers for TIMER3 inputs
        (0x1AC => timer3captsel_3: ReadWrite<u32>),
        (0x1B0 => _reserved7),
        /// Capture select registers for TIMER4 inputs
        (0x1C0 => timer4captsel_0: ReadWrite<u32>),
        /// Capture select registers for TIMER4 inputs
        (0x1C4 => timer4captsel_1: ReadWrite<u32>),
        /// Capture select registers for TIMER4 inputs
        (0x1C8 => timer4captsel_2: ReadWrite<u32>),
        /// Capture select registers for TIMER4 inputs
        (0x1CC => timer4captsel_3: ReadWrite<u32>),
        (0x1D0 => _reserved8),
        /// Pin interrupt secure select register
        (0x1E0 => pintsecsel_0: ReadWrite<u32>),
        /// Pin interrupt secure select register
        (0x1E4 => pintsecsel_1: ReadWrite<u32>),
        (0x1E8 => _reserved9),
        /// Trigger select register for DMA1 channel
        (0x200 => dma1_itrig_inmux_0: ReadWrite<u32>),
        /// Trigger select register for DMA1 channel
        (0x204 => dma1_itrig_inmux_1: ReadWrite<u32>),
        /// Trigger select register for DMA1 channel
        (0x208 => dma1_itrig_inmux_2: ReadWrite<u32>),
        /// Trigger select register for DMA1 channel
        (0x20C => dma1_itrig_inmux_3: ReadWrite<u32>),
        /// Trigger select register for DMA1 channel
        (0x210 => dma1_itrig_inmux_4: ReadWrite<u32>),
        /// Trigger select register for DMA1 channel
        (0x214 => dma1_itrig_inmux_5: ReadWrite<u32>),
        /// Trigger select register for DMA1 channel
        (0x218 => dma1_itrig_inmux_6: ReadWrite<u32>),
        /// Trigger select register for DMA1 channel
        (0x21C => dma1_itrig_inmux_7: ReadWrite<u32>),
        /// Trigger select register for DMA1 channel
        (0x220 => dma1_itrig_inmux_8: ReadWrite<u32>),
        /// Trigger select register for DMA1 channel
        (0x224 => dma1_itrig_inmux_9: ReadWrite<u32>),
        (0x228 => _reserved10),
        /// DMA1 output trigger selection to become DMA1 trigger
        (0x240 => dma1_otrig_inmux_0: ReadWrite<u32>),
        /// DMA1 output trigger selection to become DMA1 trigger
        (0x244 => dma1_otrig_inmux_1: ReadWrite<u32>),
        /// DMA1 output trigger selection to become DMA1 trigger
        (0x248 => dma1_otrig_inmux_2: ReadWrite<u32>),
        /// DMA1 output trigger selection to become DMA1 trigger
        (0x24C => dma1_otrig_inmux_3: ReadWrite<u32>),
        (0x250 => _reserved11),
        /// Enable DMA0 requests
        (0x740 => dma0_req_ena: ReadWrite<u32>),
        (0x744 => _reserved12),
        /// Set one or several bits in DMA0_REQ_ENA register
        (0x748 => dma0_req_ena_set: WriteOnly<u32>),
        (0x74C => _reserved13),
        /// Clear one or several bits in DMA0_REQ_ENA register
        (0x750 => dma0_req_ena_clr: WriteOnly<u32>),
        (0x754 => _reserved14),
        /// Enable DMA1 requests
        (0x760 => dma1_req_ena: ReadWrite<u32>),
        (0x764 => _reserved15),
        /// Set one or several bits in DMA1_REQ_ENA register
        (0x768 => dma1_req_ena_set: WriteOnly<u32>),
        (0x76C => _reserved16),
        /// Clear one or several bits in DMA1_REQ_ENA register
        (0x770 => dma1_req_ena_clr: WriteOnly<u32>),
        (0x774 => _reserved17),
        /// Enable DMA0 triggers
        (0x780 => dma0_itrig_ena: ReadWrite<u32>),
        (0x784 => _reserved18),
        /// Set one or several bits in DMA0_ITRIG_ENA register
        (0x788 => dma0_itrig_ena_set: WriteOnly<u32>),
        (0x78C => _reserved19),
        /// Clear one or several bits in DMA0_ITRIG_ENA register
        (0x790 => dma0_itrig_ena_clr: WriteOnly<u32>),
        (0x794 => _reserved20),
        /// Enable DMA1 triggers
        (0x7A0 => dma1_itrig_ena: ReadWrite<u32>),
        (0x7A4 => _reserved21),
        /// Set one or several bits in DMA1_ITRIG_ENA register
        (0x7A8 => dma1_itrig_ena_set: WriteOnly<u32>),
        (0x7AC => _reserved22),
        /// Clear one or several bits in DMA1_ITRIG_ENA register
        (0x7B0 => dma1_itrig_ena_clr: WriteOnly<u32>),
        (0x7B4 => @END),
    }
}
register_bitfields![u32,
PINTSEL [
    /// Pin number select for pin interrupt or pattern match engine input. For PIOx_y: I
    INTPIN OFFSET(0) NUMBITS(7) []
],
];
const INPUTMUX_BASE: StaticRef<InputmuxRegisters> =
    unsafe { StaticRef::new(0x50006000 as *const InputmuxRegisters) };

pub struct Inputmux {
    registers: StaticRef<InputmuxRegisters>,
}

// Safety: Inputmux only contains a StaticRef, which is safe to share between threads.

impl Inputmux {
    pub const fn new() -> Self {
        Inputmux {
            registers: INPUTMUX_BASE,
        }
    }

    pub fn registers(&self) -> &InputmuxRegisters {
        &self.registers
    }

    pub fn set_pintsel(&self, channel: usize, pin: u8) {
        assert!(channel < 8);
        let pintsel = match channel {
            0 => &self.registers.pintsel_0,
            1 => &self.registers.pintsel_1,
            2 => &self.registers.pintsel_2,
            3 => &self.registers.pintsel_3,
            4 => &self.registers.pintsel_4,
            5 => &self.registers.pintsel_5,
            6 => &self.registers.pintsel_6,
            7 => &self.registers.pintsel_7,
            _ => unreachable!(),
        };
        pintsel.modify(PINTSEL::INTPIN.val(pin as u32));
    }
}
