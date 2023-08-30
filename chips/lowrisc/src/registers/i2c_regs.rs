// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for i2c.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/i2c/data/i2c.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Depth of FMT, RX, TX, and ACQ FIFOs
pub const I2C_PARAM_FIFO_DEPTH: u32 = 64;
/// Number of alerts
pub const I2C_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const I2C_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub I2cRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// I2C Control Register
        (0x0010 => pub(crate) ctrl: ReadWrite<u32, CTRL::Register>),
        /// I2C Live Status Register
        (0x0014 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        /// I2C Read Data
        (0x0018 => pub(crate) rdata: ReadWrite<u32, RDATA::Register>),
        /// I2C Format Data
        (0x001c => pub(crate) fdata: ReadWrite<u32, FDATA::Register>),
        /// I2C FIFO control register
        (0x0020 => pub(crate) fifo_ctrl: ReadWrite<u32, FIFO_CTRL::Register>),
        /// I2C FIFO status register
        (0x0024 => pub(crate) fifo_status: ReadWrite<u32, FIFO_STATUS::Register>),
        /// I2C Override Control Register
        (0x0028 => pub(crate) ovrd: ReadWrite<u32, OVRD::Register>),
        /// Oversampled RX values
        (0x002c => pub(crate) val: ReadWrite<u32, VAL::Register>),
        /// Detailed I2C Timings (directly corresponding to table 10 in the I2C Specification).
        (0x0030 => pub(crate) timing0: ReadWrite<u32, TIMING0::Register>),
        /// Detailed I2C Timings (directly corresponding to table 10 in the I2C Specification).
        (0x0034 => pub(crate) timing1: ReadWrite<u32, TIMING1::Register>),
        /// Detailed I2C Timings (directly corresponding to table 10 in the I2C Specification).
        (0x0038 => pub(crate) timing2: ReadWrite<u32, TIMING2::Register>),
        /// Detailed I2C Timings (directly corresponding to table 10, in the I2C Specification).
        (0x003c => pub(crate) timing3: ReadWrite<u32, TIMING3::Register>),
        /// Detailed I2C Timings (directly corresponding to table 10, in the I2C Specification).
        (0x0040 => pub(crate) timing4: ReadWrite<u32, TIMING4::Register>),
        /// I2C clock stretching timeout control
        (0x0044 => pub(crate) timeout_ctrl: ReadWrite<u32, TIMEOUT_CTRL::Register>),
        /// I2C target address and mask pairs
        (0x0048 => pub(crate) target_id: ReadWrite<u32, TARGET_ID::Register>),
        /// I2C target acquired data
        (0x004c => pub(crate) acqdata: ReadWrite<u32, ACQDATA::Register>),
        /// I2C target transmit data
        (0x0050 => pub(crate) txdata: ReadWrite<u32, TXDATA::Register>),
        /// I2C host clock generation timeout value (in units of input clock frequency)
        (0x0054 => pub(crate) host_timeout_ctrl: ReadWrite<u32, HOST_TIMEOUT_CTRL::Register>),
        (0x0058 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        FMT_THRESHOLD OFFSET(0) NUMBITS(1) [],
        RX_THRESHOLD OFFSET(1) NUMBITS(1) [],
        FMT_OVERFLOW OFFSET(2) NUMBITS(1) [],
        RX_OVERFLOW OFFSET(3) NUMBITS(1) [],
        NAK OFFSET(4) NUMBITS(1) [],
        SCL_INTERFERENCE OFFSET(5) NUMBITS(1) [],
        SDA_INTERFERENCE OFFSET(6) NUMBITS(1) [],
        STRETCH_TIMEOUT OFFSET(7) NUMBITS(1) [],
        SDA_UNSTABLE OFFSET(8) NUMBITS(1) [],
        CMD_COMPLETE OFFSET(9) NUMBITS(1) [],
        TX_STRETCH OFFSET(10) NUMBITS(1) [],
        TX_OVERFLOW OFFSET(11) NUMBITS(1) [],
        ACQ_FULL OFFSET(12) NUMBITS(1) [],
        UNEXP_STOP OFFSET(13) NUMBITS(1) [],
        HOST_TIMEOUT OFFSET(14) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CTRL [
        ENABLEHOST OFFSET(0) NUMBITS(1) [],
        ENABLETARGET OFFSET(1) NUMBITS(1) [],
        LLPBK OFFSET(2) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
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
    pub(crate) RDATA [
        RDATA OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) FDATA [
        FBYTE OFFSET(0) NUMBITS(8) [],
        START OFFSET(8) NUMBITS(1) [],
        STOP OFFSET(9) NUMBITS(1) [],
        READ OFFSET(10) NUMBITS(1) [],
        RCONT OFFSET(11) NUMBITS(1) [],
        NAKOK OFFSET(12) NUMBITS(1) [],
    ],
    pub(crate) FIFO_CTRL [
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
    pub(crate) FIFO_STATUS [
        FMTLVL OFFSET(0) NUMBITS(7) [],
        TXLVL OFFSET(8) NUMBITS(7) [],
        RXLVL OFFSET(16) NUMBITS(7) [],
        ACQLVL OFFSET(24) NUMBITS(7) [],
    ],
    pub(crate) OVRD [
        TXOVRDEN OFFSET(0) NUMBITS(1) [],
        SCLVAL OFFSET(1) NUMBITS(1) [],
        SDAVAL OFFSET(2) NUMBITS(1) [],
    ],
    pub(crate) VAL [
        SCL_RX OFFSET(0) NUMBITS(16) [],
        SDA_RX OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) TIMING0 [
        THIGH OFFSET(0) NUMBITS(16) [],
        TLOW OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) TIMING1 [
        T_R OFFSET(0) NUMBITS(16) [],
        T_F OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) TIMING2 [
        TSU_STA OFFSET(0) NUMBITS(16) [],
        THD_STA OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) TIMING3 [
        TSU_DAT OFFSET(0) NUMBITS(16) [],
        THD_DAT OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) TIMING4 [
        TSU_STO OFFSET(0) NUMBITS(16) [],
        T_BUF OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) TIMEOUT_CTRL [
        VAL OFFSET(0) NUMBITS(31) [],
        EN OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) TARGET_ID [
        ADDRESS0 OFFSET(0) NUMBITS(7) [],
        MASK0 OFFSET(7) NUMBITS(7) [],
        ADDRESS1 OFFSET(14) NUMBITS(7) [],
        MASK1 OFFSET(21) NUMBITS(7) [],
    ],
    pub(crate) ACQDATA [
        ABYTE OFFSET(0) NUMBITS(8) [],
        SIGNAL OFFSET(8) NUMBITS(2) [
            NONE = 0,
            START = 1,
            STOP = 2,
            RESTART = 3,
        ],
    ],
    pub(crate) TXDATA [
        TXDATA OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) HOST_TIMEOUT_CTRL [
        HOST_TIMEOUT_CTRL OFFSET(0) NUMBITS(32) [],
    ],
];

// End generated register constants for i2c
