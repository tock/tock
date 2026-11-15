// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Specific device definition for the MPS3-AN536 instance pf the Cortex R52.
//! All data come from the document "Example Cortex™-R52x2 Subsystem for MPS3
//! Application Note AN536 / Issue B / DAI 0536"
//! Implented as as separate crate to avoid cyclic package dependency

#![no_std]
use kernel::hil::time::Frequency;

//
// CLOCKS
//

// Main clock
pub const FREQ_ACLK: u32 = 50_000_000;
// RTC and DualTimer clock
pub const FREQ_TCLK: u32 = 24_000_000;

/// Frequency struct for main clock
pub struct FreqAclk;
impl Frequency for FreqAclk {
    fn frequency() -> u32 {
        FREQ_ACLK
    }
}

/// Frequency struct for timer clock
pub struct FreqTclk;
impl Frequency for FreqTclk {
    fn frequency() -> u32 {
        FREQ_TCLK
    }
}

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

/// Representation of a GIC interrupt (IRQ) with its ID and trigger type
/// to allow easy definition.
const TRIG_LEVEL: u16 = 0;
const TRIG_EDGE: u16 = 1;

macro_rules! spi {
    ($id:expr, $trig:expr, $prio:expr) => {
        (($id + 32) << 8)| (($prio >> 3) & 0xF8) | $trig 
    };
}

macro_rules! ppi {
    ($id:expr, $trig:expr, $prio:expr) => {
        (($id + 16)<< 8) | (($prio >> 3) & 0xF8) | $trig 
    };
}


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

//pub const IRQ_WDG = spi!(0, todo);
pub const IRQ_DUALTIMER1: u16 = spi!(1, TRIG_EDGE, 0x20);
pub const IRQ_DUALTIMER2: u16 = spi!(2, TRIG_EDGE, 0x20);
pub const IRQ_DUALTIMER_CMB: u16 = spi!(3, TRIG_EDGE, 0x20);

//pub const IRQ_RTC:              GicIrq = GicIrq { id: IntId::spi(4), trig: todo }                       ;
pub const IRQ_UART2_RX: u16 = spi!(5, TRIG_LEVEL, 0x40);
pub const IRQ_UART2_TX: u16 = spi!(6, TRIG_LEVEL, 0x40);
pub const IRQ_UART3_RX: u16 = spi!(7, TRIG_LEVEL, 0x40);
pub const IRQ_UART3_TX: u16 = spi!(8, TRIG_LEVEL, 0x40);
pub const IRQ_UART4_RX: u16 = spi!(9, TRIG_LEVEL, 0x40);
pub const IRQ_UART4_TX: u16 = spi!(10, TRIG_LEVEL, 0x40);
pub const IRQ_UART5_RX: u16 = spi!(11, TRIG_LEVEL, 0x40);
pub const IRQ_UART5_TX: u16 = spi!(12, TRIG_LEVEL, 0x40);
pub const IRQ_UART2_CMB: u16 = spi!(13, TRIG_LEVEL, 0x40);
pub const IRQ_UART3_CMB: u16 = spi!(14, TRIG_LEVEL, 0x40);
pub const IRQ_UART4_CMB: u16 = spi!(15, TRIG_LEVEL, 0x40);
pub const IRQ_UART5_CMB: u16 = spi!(16, TRIG_LEVEL, 0x40);
pub const IRQ_UART_OVF: u16 = spi!(17, TRIG_LEVEL, 0x40);
//pub const IRQ_ETHERNET  = spi!(18, todo);
//pub const IRQ_USB = spi!(19, todo);
//pub const IRQ_FPGA_AUDIO_I2S = spi!(20, todo);
//pub const IRQ_TOUCH_SCREEN = spi!(21, todo);
//pub const IRQ_SPI_ADC = spi!(22, todo);
//pub const IRQ_SPI_SHIELD0 = spi!(23, todo);
//pub const IRQ_SPI_SHIELD1 = spi!(24, todo);
//pub const IRQ_HDCLCD = spi!(25, todo);
//pub const IRQ_GPIO0_COMB = spi!(26, todo);
pub const IRQ_UART01_RX: u16 = ppi!(0, TRIG_LEVEL, 0x30);
pub const IRQ_UART01_TX: u16 = ppi!(1, TRIG_LEVEL, 0x30);
pub const IRQ_UART01_CMB: u16 = ppi!(2, TRIG_LEVEL, 0x30);
pub const IRQ_UART01_OVF: u16 = ppi!(3, TRIG_LEVEL, 0x30);

// Default PPI for Cortex-R52, need to substract 16 from the
// Cortex R52 Tech RM Table 10-3.

//   22 Debug Communications Channel interrupt
//   23 Performance Monitor Counter Overflow interrupt
//   24 Cross Trigger Interface interrupt
//   25 Virtual CPU Interface Maintenance interrupt
//   26 Hypervisor timer interrupt
//   27 Virtual timer interrupt
//   30 Non-secure physical timer interrupt

pub const IRQ_GENTIMER_VIRT: u16 = ppi!(11, TRIG_EDGE, 0x20);
pub const IRQ_GENTIMER_PHY: u16 = ppi!(14, TRIG_EDGE, 0x10);

