// Generated register constants for uart.
// This file is licensed under either of:
//   Apache License, Version 2.0 (LICENSE-APACHE <http://www.apache.org/licenses/LICENSE-2.0>)
//   MIT License (LICENSE-MIT <http://opensource.org/licenses/MIT>)

// Built for earlgrey_silver_release_v5-5654-g222658011
// https://github.com/lowRISC/opentitan/tree/222658011c27d6c1f22f02c7f589043f207ff574
// Tree status: clean
// Build date: 2022-06-02T20:40:57

// Original reference file: hw/ip/uart/data/uart.hjson
// Copyright information found in the reference file:
//   Copyright lowRISC contributors.
// Licensing information found in the reference file:
//   Licensed under the Apache License, Version 2.0, see LICENSE for details.
//   SPDX-License-Identifier: Apache-2.0

use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
// Number of alerts
pub const UART_PARAM_NUM_ALERTS: u32 = 1;
// Register width
pub const UART_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub UartRegisters {
        // Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        // Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        // Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        // Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        // UART control register
        (0x0010 => pub(crate) ctrl: ReadWrite<u32, CTRL::Register>),
        // UART live status register
        (0x0014 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        // UART read data
        (0x0018 => pub(crate) rdata: ReadWrite<u32, RDATA::Register>),
        // UART write data
        (0x001c => pub(crate) wdata: ReadWrite<u32, WDATA::Register>),
        // UART FIFO control register
        (0x0020 => pub(crate) fifo_ctrl: ReadWrite<u32, FIFO_CTRL::Register>),
        // UART FIFO status register
        (0x0024 => pub(crate) fifo_status: ReadWrite<u32, FIFO_STATUS::Register>),
        // TX pin override control. Gives direct SW control over TX pin state
        (0x0028 => pub(crate) ovrd: ReadWrite<u32, OVRD::Register>),
        // UART oversampled values
        (0x002c => pub(crate) val: ReadWrite<u32, VAL::Register>),
        // UART RX timeout control
        (0x0030 => pub(crate) timeout_ctrl: ReadWrite<u32, TIMEOUT_CTRL::Register>),
        (0x0034 => @END),
    }
}

register_bitfields![u32,
    // Common Interrupt Offsets
    pub(crate) INTR [
        TX_WATERMARK OFFSET(0) NUMBITS(1) [],
        RX_WATERMARK OFFSET(1) NUMBITS(1) [],
        TX_EMPTY OFFSET(2) NUMBITS(1) [],
        RX_OVERFLOW OFFSET(3) NUMBITS(1) [],
        RX_FRAME_ERR OFFSET(4) NUMBITS(1) [],
        RX_BREAK_ERR OFFSET(5) NUMBITS(1) [],
        RX_TIMEOUT OFFSET(6) NUMBITS(1) [],
        RX_PARITY_ERR OFFSET(7) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CTRL [
        TX OFFSET(0) NUMBITS(1) [],
        RX OFFSET(1) NUMBITS(1) [],
        NF OFFSET(2) NUMBITS(1) [],
        SLPBK OFFSET(4) NUMBITS(1) [],
        LLPBK OFFSET(5) NUMBITS(1) [],
        PARITY_EN OFFSET(6) NUMBITS(1) [],
        PARITY_ODD OFFSET(7) NUMBITS(1) [],
        RXBLVL OFFSET(8) NUMBITS(2) [
            BREAK2 = 0,
            BREAK4 = 1,
            BREAK8 = 2,
            BREAK16 = 3,
        ],
        NCO OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) STATUS [
        TXFULL OFFSET(0) NUMBITS(1) [],
        RXFULL OFFSET(1) NUMBITS(1) [],
        TXEMPTY OFFSET(2) NUMBITS(1) [],
        TXIDLE OFFSET(3) NUMBITS(1) [],
        RXIDLE OFFSET(4) NUMBITS(1) [],
        RXEMPTY OFFSET(5) NUMBITS(1) [],
    ],
    pub(crate) RDATA [
        RDATA OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) WDATA [
        WDATA OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) FIFO_CTRL [
        RXRST OFFSET(0) NUMBITS(1) [],
        TXRST OFFSET(1) NUMBITS(1) [],
        RXILVL OFFSET(2) NUMBITS(3) [
            RXLVL1 = 0,
            RXLVL4 = 1,
            RXLVL8 = 2,
            RXLVL16 = 3,
            RXLVL30 = 4,
        ],
        TXILVL OFFSET(5) NUMBITS(2) [
            TXLVL1 = 0,
            TXLVL4 = 1,
            TXLVL8 = 2,
            TXLVL16 = 3,
        ],
    ],
    pub(crate) FIFO_STATUS [
        TXLVL OFFSET(0) NUMBITS(6) [],
        RXLVL OFFSET(16) NUMBITS(6) [],
    ],
    pub(crate) OVRD [
        TXEN OFFSET(0) NUMBITS(1) [],
        TXVAL OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) VAL [
        RX OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) TIMEOUT_CTRL [
        VAL OFFSET(0) NUMBITS(24) [],
        EN OFFSET(31) NUMBITS(1) [],
    ],
];

// End generated register constants for uart
