// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Specific device definition for the MPS3-AN536 instance pf the Cortex R52.
//! All data come from the document "Example Cortex™-R52x2 Subsystem for MPS3
//! Application Note AN536 / Issue B / DAI 0536"

use arm_gic::IntId;
use arm_gic::Trigger::{Edge, Level};
use kernel::hil::time::Frequency;

use cortexr52::gic::GicIrq;

//
// CLOCKS
//

// Main clock
pub const FREQ_ACLK: u32 = 50_000_000;
// RTC and Timer clock
pub const FREQ_TCLK: u32 = 24_000_000;

/// Frequency struct for main clock
pub struct FreqAclk;
impl Frequency for FreqAclk {
    fn frequency() -> u32 {
        FREQ_ACLK
    }
}
pub type GenTimer<'a> = cortexr52::gentimer::GenTimer<'a, FreqAclk>;

/// Frequency struct for timer clock
pub struct FreqTclk;
impl Frequency for FreqTclk {
    fn frequency() -> u32 {
        FREQ_TCLK
    }
}
pub type DualTimer<'a> = crate::dualtimer::DualTimer<'a, FreqTclk>;

//
// PERIPHERALS ADDRESSES
//

/// GICv3 distributor (GICD) registers
pub const BASE_GICD: u32 = 0xf000_0000;

/// First GICv3 redistributor (GICR) frame
pub const BASE_GICR: u32 = 0xf010_0000;

// UARTs registers
// Instances 0 and 1 are per core only and use the same Low Latency Peripheral Port address
pub const BASE_UART01: u32 = 0xE7C0_0000;
// Instances 2 is the only one tested.
// Qemu command line should include: "-serial none -serial none -serial mon:stdio"
pub const BASE_UART2: u32 = 0xE020_5000;
pub const BASE_UART3: u32 = 0xE020_6000;
pub const BASE_UART4: u32 = 0xE020_7000;
pub const BASE_UART5: u32 = 0xE020_8000;

pub const BASE_DUALTIMER1: u32 = 0xE010_1000;
pub const BASE_DUALTIMER2: u32 = 0xE010_1020;

//
// INTERRUPTS MAP
//

// Note: Internal PPI is Cortex 52 specific and defined in gic.rs; Also,
// "External" PPI is different from "Extended". Extended PPI are not used with An536

// Idx  SPI                             Exter PPI (per core)
// -----------------------------------------------------------------
//  0   WDG                             UART 0/1 Receive Interrupt
//  1   DualTimer 1                     UART 0/1 Transmit Interrupt
//  2   DualTimer 2                     UART 0/1 Combined Interrupt
//  3   DualTimer Combined              UART 0/1 Overflow
//  4   RTC
//  5   UART 2 Receive Interrupt
//  6   UART 2 Transmit Interrupt
//  7   UART 3 Receive Interrupt
//  8   UART 3 Transmit Interrupt
//  9   UART 4 Receive Interrupt
// 10   UART 4 Transmit Interrupt
// 11   UART 5 Receive Interrupt
// 12   UART 5 Transmit Interrupt
// 13   UART 2 Combined Interrupt
// 14   UART 3 Combined Interrupt
// 15   UART 4 Combined Interrupt
// 16   UART 5 Combined Interrupt
// 17   UART Overflow (2, 3, 4 & 5)
// 18   Ethernet
// 19   USB
// 20   FPGA Audio I2S
// 21   Touch Screen
// 22   SPI ADC
// 23   SPI (Shield 0)
// 24   SPI (Shield 1)
// 25   HDCLCD Interrupt
// 26   GPIO 0 Combined Interrupt

//pub const IRQ_WDG:            GicIrq = GicIrq { id: IntId::spi(0), trig: todo };
pub const IRQ_DUALTIMER1: GicIrq = GicIrq {
    id: IntId::spi(1),
    trig: Edge,
};
pub const IRQ_DUALTIMER2: GicIrq = GicIrq {
    id: IntId::spi(2),
    trig: Edge,
};
pub const IRQ_DUALTIMER_CMB: GicIrq = GicIrq {
    id: IntId::spi(3),
    trig: Edge,
};
//pub const IRQ_RTC:              GicIrq = GicIrq { id: IntId::spi(4), trig: todo }                       ;
pub const IRQ_UART2_RX: GicIrq = GicIrq {
    id: IntId::spi(5),
    trig: Level,
};
pub const IRQ_UART2_TX: GicIrq = GicIrq {
    id: IntId::spi(6),
    trig: Level,
};
pub const IRQ_UART3_RX: GicIrq = GicIrq {
    id: IntId::spi(7),
    trig: Level,
};
pub const IRQ_UART3_TX: GicIrq = GicIrq {
    id: IntId::spi(8),
    trig: Level,
};
pub const IRQ_UART4_RX: GicIrq = GicIrq {
    id: IntId::spi(9),
    trig: Level,
};
pub const IRQ_UART4_TX: GicIrq = GicIrq {
    id: IntId::spi(10),
    trig: Level,
};
pub const IRQ_UART5_RX: GicIrq = GicIrq {
    id: IntId::spi(11),
    trig: Level,
};
pub const IRQ_UART5_TX: GicIrq = GicIrq {
    id: IntId::spi(12),
    trig: Level,
};
pub const IRQ_UART2_CMB: GicIrq = GicIrq {
    id: IntId::spi(13),
    trig: Level,
};
pub const IRQ_UART3_CMB: GicIrq = GicIrq {
    id: IntId::spi(14),
    trig: Level,
};
pub const IRQ_UART4_CMB: GicIrq = GicIrq {
    id: IntId::spi(15),
    trig: Level,
};
pub const IRQ_UART5_CMB: GicIrq = GicIrq {
    id: IntId::spi(16),
    trig: Level,
};
pub const IRQ_UART_OVF: GicIrq = GicIrq {
    id: IntId::spi(17),
    trig: Level,
};
//pub const IRQ_ETHERNET:         GicIrq = GicIrq { id: IntId::spi(18), trig: todo };
//pub const IRQ_USB:              GicIrq = GicIrq { id: IntId::spi(19), trig: todo };
//pub const IRQ_FPGA_AUDIO_I2S:   GicIrq = GicIrq { id: IntId::spi(20), trig: todo };
//pub const IRQ_TOUCH_SCREEN:     GicIrq = GicIrq { id: IntId::spi(21), trig: todo };
//pub const IRQ_SPI_ADC:          GicIrq = GicIrq { id: IntId::spi(22), trig: todo };
//pub const IRQ_SPI_SHIELD0:      GicIrq = GicIrq { id: IntId::spi(23), trig: todo };
//pub const IRQ_SPI_SHIELD1:      GicIrq = GicIrq { id: IntId::spi(24), trig: todo };
//pub const IRQ_HDCLCD:           GicIrq = GicIrq { id: IntId::spi(25), trig: todo };
//pub const IRQ_GPIO0_COMB:       GicIrq = GicIrq { id: IntId::spi(26), trig: todo };

pub const IRQ_UART01_RX: GicIrq = GicIrq {
    id: IntId::ppi(0),
    trig: Level,
};
pub const IRQ_UART01_TX: GicIrq = GicIrq {
    id: IntId::ppi(1),
    trig: Level,
};
pub const IRQ_UART01_CMB: GicIrq = GicIrq {
    id: IntId::ppi(2),
    trig: Level,
};
pub const IRQ_UART01_OVF: GicIrq = GicIrq {
    id: IntId::ppi(3),
    trig: Level,
};
