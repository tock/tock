// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for uart.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/uart/data/uart.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of alerts
pub const UART_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const UART_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub UartRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// UART control register
        (0x0010 => pub(crate) ctrl: ReadWrite<u32, CTRL::Register>),
        /// UART live status register
        (0x0014 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        /// UART read data
        (0x0018 => pub(crate) rdata: ReadWrite<u32, RDATA::Register>),
        /// UART write data
        (0x001c => pub(crate) wdata: ReadWrite<u32, WDATA::Register>),
        /// UART FIFO control register
        (0x0020 => pub(crate) fifo_ctrl: ReadWrite<u32, FIFO_CTRL::Register>),
        /// UART FIFO status register
        (0x0024 => pub(crate) fifo_status: ReadWrite<u32, FIFO_STATUS::Register>),
        /// TX pin override control. Gives direct SW control over TX pin state
        (0x0028 => pub(crate) ovrd: ReadWrite<u32, OVRD::Register>),
        /// UART oversampled values
        (0x002c => pub(crate) val: ReadWrite<u32, VAL::Register>),
        /// UART RX timeout control
        (0x0030 => pub(crate) timeout_ctrl: ReadWrite<u32, TIMEOUT_CTRL::Register>),
        (0x0034 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
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
            RXLVL2 = 1,
            RXLVL4 = 2,
            RXLVL8 = 3,
            RXLVL16 = 4,
            RXLVL32 = 5,
            RXLVL64 = 6,
            RXLVL126 = 7,
        ],
        TXILVL OFFSET(5) NUMBITS(3) [
            TXLVL1 = 0,
            TXLVL2 = 1,
            TXLVL4 = 2,
            TXLVL8 = 3,
            TXLVL16 = 4,
            TXLVL32 = 5,
            TXLVL64 = 6,
        ],
    ],
    pub(crate) FIFO_STATUS [
        TXLVL OFFSET(0) NUMBITS(8) [],
        RXLVL OFFSET(16) NUMBITS(8) [],
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
