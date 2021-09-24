// Generated register struct for i2c

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
    pub I2CRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x14 => status: ReadOnly<u32, STATUS::Register>),
        (0x18 => rdata: ReadOnly<u32, RDATA::Register>),
        (0x1c => fdata: WriteOnly<u32, FDATA::Register>),
        (0x20 => fifo_ctrl: ReadWrite<u32, FIFO_CTRL::Register>),
        (0x24 => fifo_status: ReadOnly<u32, FIFO_STATUS::Register>),
        (0x28 => ovrd: ReadWrite<u32, OVRD::Register>),
        (0x2c => val: ReadOnly<u32, VAL::Register>),
        (0x30 => timing0: ReadWrite<u32, TIMING0::Register>),
        (0x34 => timing1: ReadWrite<u32, TIMING1::Register>),
        (0x38 => timing2: ReadWrite<u32, TIMING2::Register>),
        (0x3c => timing3: ReadWrite<u32, TIMING3::Register>),
        (0x40 => timing4: ReadWrite<u32, TIMING4::Register>),
        (0x44 => timeout_ctrl: ReadWrite<u32, TIMEOUT_CTRL::Register>),
        (0x48 => target_id: ReadWrite<u32, TARGET_ID::Register>),
        (0x4c => acqdata: ReadOnly<u32, ACQDATA::Register>),
        (0x50 => txdata: WriteOnly<u32, TXDATA::Register>),
        (0x54 => stretch_ctrl: ReadWrite<u32, STRETCH_CTRL::Register>),
        (0x58 => host_timeout_ctrl: ReadWrite<u32, HOST_TIMEOUT_CTRL::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        FMT_WATERMARK OFFSET(0) NUMBITS(1) [],
        RX_WATERMARK OFFSET(1) NUMBITS(1) [],
        FMT_OVERFLOW OFFSET(2) NUMBITS(1) [],
        RX_OVERFLOW OFFSET(3) NUMBITS(1) [],
        NAK OFFSET(4) NUMBITS(1) [],
        SCL_INTERFERENCE OFFSET(5) NUMBITS(1) [],
        SDA_INTERFERENCE OFFSET(6) NUMBITS(1) [],
        STRETCH_TIMEOUT OFFSET(7) NUMBITS(1) [],
        SDA_UNSTABLE OFFSET(8) NUMBITS(1) [],
        TRANS_COMPLETE OFFSET(9) NUMBITS(1) [],
        TX_EMPTY OFFSET(10) NUMBITS(1) [],
        TX_NONEMPTY OFFSET(11) NUMBITS(1) [],
        TX_OVERFLOW OFFSET(12) NUMBITS(1) [],
        ACQ_OVERFLOW OFFSET(13) NUMBITS(1) [],
        ACK_STOP OFFSET(14) NUMBITS(1) [],
        HOST_TIMEOUT OFFSET(15) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        FMT_WATERMARK OFFSET(0) NUMBITS(1) [],
        RX_WATERMARK OFFSET(1) NUMBITS(1) [],
        FMT_OVERFLOW OFFSET(2) NUMBITS(1) [],
        RX_OVERFLOW OFFSET(3) NUMBITS(1) [],
        NAK OFFSET(4) NUMBITS(1) [],
        SCL_INTERFERENCE OFFSET(5) NUMBITS(1) [],
        SDA_INTERFERENCE OFFSET(6) NUMBITS(1) [],
        STRETCH_TIMEOUT OFFSET(7) NUMBITS(1) [],
        SDA_UNSTABLE OFFSET(8) NUMBITS(1) [],
        TRANS_COMPLETE OFFSET(9) NUMBITS(1) [],
        TX_EMPTY OFFSET(10) NUMBITS(1) [],
        TX_NONEMPTY OFFSET(11) NUMBITS(1) [],
        TX_OVERFLOW OFFSET(12) NUMBITS(1) [],
        ACQ_OVERFLOW OFFSET(13) NUMBITS(1) [],
        ACK_STOP OFFSET(14) NUMBITS(1) [],
        HOST_TIMEOUT OFFSET(15) NUMBITS(1) [],
    ],
    INTR_TEST [
        FMT_WATERMARK OFFSET(0) NUMBITS(1) [],
        RX_WATERMARK OFFSET(1) NUMBITS(1) [],
        FMT_OVERFLOW OFFSET(2) NUMBITS(1) [],
        RX_OVERFLOW OFFSET(3) NUMBITS(1) [],
        NAK OFFSET(4) NUMBITS(1) [],
        SCL_INTERFERENCE OFFSET(5) NUMBITS(1) [],
        SDA_INTERFERENCE OFFSET(6) NUMBITS(1) [],
        STRETCH_TIMEOUT OFFSET(7) NUMBITS(1) [],
        SDA_UNSTABLE OFFSET(8) NUMBITS(1) [],
        TRANS_COMPLETE OFFSET(9) NUMBITS(1) [],
        TX_EMPTY OFFSET(10) NUMBITS(1) [],
        TX_NONEMPTY OFFSET(11) NUMBITS(1) [],
        TX_OVERFLOW OFFSET(12) NUMBITS(1) [],
        ACQ_OVERFLOW OFFSET(13) NUMBITS(1) [],
        ACK_STOP OFFSET(14) NUMBITS(1) [],
        HOST_TIMEOUT OFFSET(15) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    CTRL [
        ENABLEHOST OFFSET(0) NUMBITS(1) [],
        ENABLETARGET OFFSET(1) NUMBITS(1) [],
        LLPBK OFFSET(2) NUMBITS(1) [],
    ],
    STATUS [
        FMTFULL OFFSET(0) NUMBITS(1) [],
        RXFULL OFFSET(1) NUMBITS(1) [],
        FMTEMPTY OFFSET(2) NUMBITS(1) [],
        HOSTIDLE OFFSET(3) NUMBITS(1) [],
        TARGETIDLE OFFSET(4) NUMBITS(1) [],
        RXEMPTY OFFSET(5) NUMBITS(1) [],
        TXFULL OFFSET(6) NUMBITS(1) [],
        ACQFULL OFFSET(7) NUMBITS(1) [],
        TXEMPTY OFFSET(8) NUMBITS(1) [],
        ACQEMPTY OFFSET(9) NUMBITS(1) [],
    ],
    RDATA [
        RDATA OFFSET(0) NUMBITS(8) [],
    ],
    FDATA [
        FBYTE OFFSET(0) NUMBITS(8) [],
        START OFFSET(8) NUMBITS(1) [],
        STOP OFFSET(9) NUMBITS(1) [],
        READ OFFSET(10) NUMBITS(1) [],
        RCONT OFFSET(11) NUMBITS(1) [],
        NAKOK OFFSET(12) NUMBITS(1) [],
    ],
    FIFO_CTRL [
        RXRST OFFSET(0) NUMBITS(1) [],
        FMTRST OFFSET(1) NUMBITS(1) [],
        RXILVL OFFSET(2) NUMBITS(3) [
            RXLVL1 = 0,
            RXLVL4 = 1,
            RXLVL8 = 2,
            RXLVL16 = 3,
            RXLVL30 = 4,
        ],
        FMTILVL OFFSET(5) NUMBITS(2) [
            FMTLVL1 = 0,
            FMTLVL4 = 1,
            FMTLVL8 = 2,
            FMTLVL16 = 3,
        ],
        ACQRST OFFSET(7) NUMBITS(1) [],
        TXRST OFFSET(8) NUMBITS(1) [],
    ],
    FIFO_STATUS [
        FMTLVL OFFSET(0) NUMBITS(7) [],
        TXLVL OFFSET(8) NUMBITS(7) [],
        RXLVL OFFSET(16) NUMBITS(7) [],
        ACQLVL OFFSET(24) NUMBITS(7) [],
    ],
    OVRD [
        TXOVRDEN OFFSET(0) NUMBITS(1) [],
        SCLVAL OFFSET(1) NUMBITS(1) [],
        SDAVAL OFFSET(2) NUMBITS(1) [],
    ],
    VAL [
        SCL_RX OFFSET(0) NUMBITS(16) [],
        SDA_RX OFFSET(16) NUMBITS(16) [],
    ],
    TIMING0 [
        THIGH OFFSET(0) NUMBITS(16) [],
        TLOW OFFSET(16) NUMBITS(16) [],
    ],
    TIMING1 [
        T_R OFFSET(0) NUMBITS(16) [],
        T_F OFFSET(16) NUMBITS(16) [],
    ],
    TIMING2 [
        TSU_STA OFFSET(0) NUMBITS(16) [],
        THD_STA OFFSET(16) NUMBITS(16) [],
    ],
    TIMING3 [
        TSU_DAT OFFSET(0) NUMBITS(16) [],
        THD_DAT OFFSET(16) NUMBITS(16) [],
    ],
    TIMING4 [
        TSU_STO OFFSET(0) NUMBITS(16) [],
        T_BUF OFFSET(16) NUMBITS(16) [],
    ],
    TIMEOUT_CTRL [
        VAL OFFSET(0) NUMBITS(31) [],
        EN OFFSET(31) NUMBITS(1) [],
    ],
    TARGET_ID [
        ADDRESS0 OFFSET(0) NUMBITS(7) [],
        MASK0 OFFSET(7) NUMBITS(7) [],
        ADDRESS1 OFFSET(14) NUMBITS(7) [],
        MASK1 OFFSET(21) NUMBITS(7) [],
    ],
    ACQDATA [
        ABYTE OFFSET(0) NUMBITS(8) [],
        SIGNAL OFFSET(8) NUMBITS(2) [],
    ],
    TXDATA [
        TXDATA OFFSET(0) NUMBITS(8) [],
    ],
    STRETCH_CTRL [
        EN_ADDR_TX OFFSET(0) NUMBITS(1) [],
        EN_ADDR_ACQ OFFSET(1) NUMBITS(1) [],
        STOP_TX OFFSET(2) NUMBITS(1) [],
        STOP_ACQ OFFSET(3) NUMBITS(1) [],
    ],
    HOST_TIMEOUT_CTRL [
        HOST_TIMEOUT_CTRL OFFSET(0) NUMBITS(32) [],
    ],
];

// Depth of FMT, RX, TX, and ACQ FIFOs
pub const I2C_PARAM_FIFO_DEPTH: u32 = 64;

// Number of alerts
pub const I2C_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const I2C_PARAM_REG_WIDTH: u32 = 32;

// End generated register constants for i2c

