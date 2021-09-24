// Generated register struct for usbuart

// Copyright information found in source file:
// Copyright lowRISC contributors.

// Licensing information found in source file:
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};

register_structs! {
    pub UsbuartRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x14 => status: ReadOnly<u32, STATUS::Register>),
        (0x18 => rdata: ReadOnly<u32, RDATA::Register>),
        (0x1c => wdata: WriteOnly<u32, WDATA::Register>),
        (0x20 => fifo_ctrl: ReadWrite<u32, FIFO_CTRL::Register>),
        (0x24 => fifo_status: ReadOnly<u32, FIFO_STATUS::Register>),
        (0x28 => ovrd: ReadWrite<u32, OVRD::Register>),
        (0x2c => val: ReadOnly<u32, VAL::Register>),
        (0x30 => timeout_ctrl: ReadWrite<u32, TIMEOUT_CTRL::Register>),
        (0x34 => usbstat: ReadOnly<u32, USBSTAT::Register>),
        (0x38 => usbparam: ReadOnly<u32, USBPARAM::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        TX_WATERMARK OFFSET(0) NUMBITS(1) [],
        RX_WATERMARK OFFSET(1) NUMBITS(1) [],
        TX_OVERFLOW OFFSET(2) NUMBITS(1) [],
        RX_OVERFLOW OFFSET(3) NUMBITS(1) [],
        RX_FRAME_ERR OFFSET(4) NUMBITS(1) [],
        RX_BREAK_ERR OFFSET(5) NUMBITS(1) [],
        RX_TIMEOUT OFFSET(6) NUMBITS(1) [],
        RX_PARITY_ERR OFFSET(7) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        TX_WATERMARK OFFSET(0) NUMBITS(1) [],
        RX_WATERMARK OFFSET(1) NUMBITS(1) [],
        TX_OVERFLOW OFFSET(2) NUMBITS(1) [],
        RX_OVERFLOW OFFSET(3) NUMBITS(1) [],
        RX_FRAME_ERR OFFSET(4) NUMBITS(1) [],
        RX_BREAK_ERR OFFSET(5) NUMBITS(1) [],
        RX_TIMEOUT OFFSET(6) NUMBITS(1) [],
        RX_PARITY_ERR OFFSET(7) NUMBITS(1) [],
    ],
    INTR_TEST [
        TX_WATERMARK OFFSET(0) NUMBITS(1) [],
        RX_WATERMARK OFFSET(1) NUMBITS(1) [],
        TX_OVERFLOW OFFSET(2) NUMBITS(1) [],
        RX_OVERFLOW OFFSET(3) NUMBITS(1) [],
        RX_FRAME_ERR OFFSET(4) NUMBITS(1) [],
        RX_BREAK_ERR OFFSET(5) NUMBITS(1) [],
        RX_TIMEOUT OFFSET(6) NUMBITS(1) [],
        RX_PARITY_ERR OFFSET(7) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    CTRL [
        TX OFFSET(0) NUMBITS(1) [],
        RX OFFSET(1) NUMBITS(1) [],
        NF OFFSET(2) NUMBITS(1) [],
        SLPBK OFFSET(4) NUMBITS(1) [],
        LLPBK OFFSET(5) NUMBITS(1) [],
        PARITY_EN OFFSET(6) NUMBITS(1) [],
        PARITY_ODD OFFSET(7) NUMBITS(1) [],
        RXBLVL OFFSET(8) NUMBITS(2) [],
        NCO OFFSET(16) NUMBITS(16) [],
    ],
    STATUS [
        TXFULL OFFSET(0) NUMBITS(1) [],
        RXFULL OFFSET(1) NUMBITS(1) [],
        TXEMPTY OFFSET(2) NUMBITS(1) [],
        TXIDLE OFFSET(3) NUMBITS(1) [],
        RXIDLE OFFSET(4) NUMBITS(1) [],
        RXEMPTY OFFSET(5) NUMBITS(1) [],
    ],
    RDATA [
        RDATA OFFSET(0) NUMBITS(8) [],
    ],
    WDATA [
        WDATA OFFSET(0) NUMBITS(8) [],
    ],
    FIFO_CTRL [
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
    FIFO_STATUS [
        TXLVL OFFSET(0) NUMBITS(6) [],
        RXLVL OFFSET(16) NUMBITS(6) [],
    ],
    OVRD [
        TXEN OFFSET(0) NUMBITS(1) [],
        TXVAL OFFSET(1) NUMBITS(1) [],
    ],
    VAL [
        RX OFFSET(0) NUMBITS(16) [],
    ],
    TIMEOUT_CTRL [
        VAL OFFSET(0) NUMBITS(24) [],
        EN OFFSET(31) NUMBITS(1) [],
    ],
    USBSTAT [
        FRAME OFFSET(0) NUMBITS(11) [],
        HOST_TIMEOUT OFFSET(14) NUMBITS(1) [],
        HOST_LOST OFFSET(15) NUMBITS(1) [],
        DEVICE_ADDRESS OFFSET(16) NUMBITS(7) [],
    ],
    USBPARAM [
        BAUD_REQ OFFSET(0) NUMBITS(16) [],
        PARITY_REQ OFFSET(16) NUMBITS(2) [
            NONE = 0,
            ODD = 1,
            EVEN = 2,
        ],
    ],
];

// Number of alerts
pub const USBUART_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const USBUART_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for usbuart

