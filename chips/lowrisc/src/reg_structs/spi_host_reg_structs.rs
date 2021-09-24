// Generated register struct for spi_host

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
    pub Spi_HostRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => control: ReadWrite<u32, CONTROL::Register>),
        (0x14 => status: ReadOnly<u32, STATUS::Register>),
        (0x18 => configopts: ReadWrite<u32, CONFIGOPTS::Register>),
        (0x1c => csid: ReadWrite<u32, CSID::Register>),
        (0x20 => command: ReadWrite<u32, COMMAND::Register>),
        (0x28 => error_enable: ReadWrite<u32, ERROR_ENABLE::Register>),
        (0x2c => error_status: ReadWrite<u32, ERROR_STATUS::Register>),
        (0x30 => event_enable: ReadWrite<u32, EVENT_ENABLE::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        ERROR OFFSET(0) NUMBITS(1) [],
        SPI_EVENT OFFSET(1) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        ERROR OFFSET(0) NUMBITS(1) [],
        SPI_EVENT OFFSET(1) NUMBITS(1) [],
    ],
    INTR_TEST [
        ERROR OFFSET(0) NUMBITS(1) [],
        SPI_EVENT OFFSET(1) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    CONTROL [
        RX_WATERMARK OFFSET(0) NUMBITS(8) [],
        TX_WATERMARK OFFSET(8) NUMBITS(8) [],
        SW_RST OFFSET(30) NUMBITS(1) [],
        SPIEN OFFSET(31) NUMBITS(1) [],
    ],
    STATUS [
        TXQD OFFSET(0) NUMBITS(8) [],
        RXQD OFFSET(8) NUMBITS(8) [],
        RXWM OFFSET(20) NUMBITS(1) [],
        BYTEORDER OFFSET(22) NUMBITS(1) [],
        RXSTALL OFFSET(23) NUMBITS(1) [],
        RXEMPTY OFFSET(24) NUMBITS(1) [],
        RXFULL OFFSET(25) NUMBITS(1) [],
        TXWM OFFSET(26) NUMBITS(1) [],
        TXSTALL OFFSET(27) NUMBITS(1) [],
        TXEMPTY OFFSET(28) NUMBITS(1) [],
        TXFULL OFFSET(29) NUMBITS(1) [],
        ACTIVE OFFSET(30) NUMBITS(1) [],
        READY OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGOPTS [
        CLKDIV_0 OFFSET(0) NUMBITS(16) [],
        CSNIDLE_0 OFFSET(16) NUMBITS(4) [],
        CSNTRAIL_0 OFFSET(20) NUMBITS(4) [],
        CSNLEAD_0 OFFSET(24) NUMBITS(4) [],
        FULLCYC_0 OFFSET(29) NUMBITS(1) [],
        CPHA_0 OFFSET(30) NUMBITS(1) [],
        CPOL_0 OFFSET(31) NUMBITS(1) [],
    ],
    CSID [
        CSID OFFSET(0) NUMBITS(32) [],
    ],
    COMMAND [
        LEN OFFSET(0) NUMBITS(9) [],
        CSAAT OFFSET(9) NUMBITS(1) [],
        SPEED OFFSET(10) NUMBITS(2) [],
        DIRECTION OFFSET(12) NUMBITS(2) [],
    ],
    ERROR_ENABLE [
        CMDBUSY OFFSET(0) NUMBITS(1) [],
        OVERFLOW OFFSET(1) NUMBITS(1) [],
        UNDERFLOW OFFSET(2) NUMBITS(1) [],
        CMDINVAL OFFSET(3) NUMBITS(1) [],
        CSIDINVAL OFFSET(4) NUMBITS(1) [],
    ],
    ERROR_STATUS [
        CMDBUSY OFFSET(0) NUMBITS(1) [],
        OVERFLOW OFFSET(1) NUMBITS(1) [],
        UNDERFLOW OFFSET(2) NUMBITS(1) [],
        CMDINVAL OFFSET(3) NUMBITS(1) [],
        CSIDINVAL OFFSET(4) NUMBITS(1) [],
    ],
    EVENT_ENABLE [
        RXFULL OFFSET(0) NUMBITS(1) [],
        TXEMPTY OFFSET(1) NUMBITS(1) [],
        RXWM OFFSET(2) NUMBITS(1) [],
        TXWM OFFSET(3) NUMBITS(1) [],
        READY OFFSET(4) NUMBITS(1) [],
        IDLE OFFSET(5) NUMBITS(1) [],
    ],
];

// The number of active-low chip select (cs_n) lines to create.
pub const SPI_HOST_PARAM_NUM_C_S: u32 = 1;

// The size of the Tx FIFO (in words)
pub const SPI_HOST_PARAM_TX_DEPTH: u32 = 72;

// The size of the Rx FIFO (in words)
pub const SPI_HOST_PARAM_RX_DEPTH: u32 = 64;

// Number of alerts
pub const SPI_HOST_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const SPI_HOST_PARAM_REG_WIDTH: u32 = 32;

// Memory area: SPI Transmit and Receive Data.
pub const SPI_HOST_DATA_REG_OFFSET: usize = 0x24;
pub const SPI_HOST_DATA_SIZE_WORDS: u32 = 1;
pub const SPI_HOST_DATA_SIZE_BYTES: u32 = 4;
// End generated register constants for spi_host

