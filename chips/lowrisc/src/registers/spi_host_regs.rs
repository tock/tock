// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for spi_host.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/spi_host/data/spi_host.hjson
use kernel::utilities::registers::ReadOnly;
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::WriteOnly;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// The number of active-low chip select (cs_n) lines to create.
pub const SPI_HOST_PARAM_NUM_C_S: u32 = 1;
/// The size of the Tx FIFO (in words)
pub const SPI_HOST_PARAM_TX_DEPTH: u32 = 72;
/// The size of the Rx FIFO (in words)
pub const SPI_HOST_PARAM_RX_DEPTH: u32 = 64;
/// The size of the Cmd FIFO (one segment descriptor per entry)
pub const SPI_HOST_PARAM_CMD_DEPTH: u32 = 4;
/// Number of alerts
pub const SPI_HOST_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const SPI_HOST_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub SpiHostRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Control register
        (0x0010 => pub(crate) control: ReadWrite<u32, CONTROL::Register>),
        /// Status register
        (0x0014 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        /// Configuration options register.
        (0x0018 => pub(crate) configopts: [ReadWrite<u32, CONFIGOPTS::Register>; 1]),
        /// Chip-Select ID
        (0x001c => pub(crate) csid: ReadWrite<u32, CSID::Register>),
        /// Command Register
        (0x0020 => pub(crate) command: ReadWrite<u32, COMMAND::Register>),
        /// Memory area: SPI Receive Data.
        (0x0024 => pub(crate) rxdata: [ReadOnly<u32>; 1]),
        /// Memory area: SPI Transmit Data.
        (0x0028 => pub(crate) txdata: [WriteOnly<u32>; 1]),
        /// Controls which classes of errors raise an interrupt.
        (0x002c => pub(crate) error_enable: ReadWrite<u32, ERROR_ENABLE::Register>),
        /// Indicates that any errors that have occurred.
        (0x0030 => pub(crate) error_status: ReadWrite<u32, ERROR_STATUS::Register>),
        /// Controls which classes of SPI events raise an interrupt.
        (0x0034 => pub(crate) event_enable: ReadWrite<u32, EVENT_ENABLE::Register>),
        (0x0038 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        ERROR OFFSET(0) NUMBITS(1) [],
        SPI_EVENT OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CONTROL [
        RX_WATERMARK OFFSET(0) NUMBITS(8) [],
        TX_WATERMARK OFFSET(8) NUMBITS(8) [],
        OUTPUT_EN OFFSET(29) NUMBITS(1) [],
        SW_RST OFFSET(30) NUMBITS(1) [],
        SPIEN OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
        TXQD OFFSET(0) NUMBITS(8) [],
        RXQD OFFSET(8) NUMBITS(8) [],
        CMDQD OFFSET(16) NUMBITS(4) [],
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
    pub(crate) CONFIGOPTS [
        CLKDIV_0 OFFSET(0) NUMBITS(16) [],
        CSNIDLE_0 OFFSET(16) NUMBITS(4) [],
        CSNTRAIL_0 OFFSET(20) NUMBITS(4) [],
        CSNLEAD_0 OFFSET(24) NUMBITS(4) [],
        FULLCYC_0 OFFSET(29) NUMBITS(1) [],
        CPHA_0 OFFSET(30) NUMBITS(1) [],
        CPOL_0 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) CSID [
        CSID OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) COMMAND [
        LEN OFFSET(0) NUMBITS(9) [],
        CSAAT OFFSET(9) NUMBITS(1) [],
        SPEED OFFSET(10) NUMBITS(2) [],
        DIRECTION OFFSET(12) NUMBITS(2) [],
    ],
    pub(crate) ERROR_ENABLE [
        CMDBUSY OFFSET(0) NUMBITS(1) [],
        OVERFLOW OFFSET(1) NUMBITS(1) [],
        UNDERFLOW OFFSET(2) NUMBITS(1) [],
        CMDINVAL OFFSET(3) NUMBITS(1) [],
        CSIDINVAL OFFSET(4) NUMBITS(1) [],
    ],
    pub(crate) ERROR_STATUS [
        CMDBUSY OFFSET(0) NUMBITS(1) [],
        OVERFLOW OFFSET(1) NUMBITS(1) [],
        UNDERFLOW OFFSET(2) NUMBITS(1) [],
        CMDINVAL OFFSET(3) NUMBITS(1) [],
        CSIDINVAL OFFSET(4) NUMBITS(1) [],
        ACCESSINVAL OFFSET(5) NUMBITS(1) [],
    ],
    pub(crate) EVENT_ENABLE [
        RXFULL OFFSET(0) NUMBITS(1) [],
        TXEMPTY OFFSET(1) NUMBITS(1) [],
        RXWM OFFSET(2) NUMBITS(1) [],
        TXWM OFFSET(3) NUMBITS(1) [],
        READY OFFSET(4) NUMBITS(1) [],
        IDLE OFFSET(5) NUMBITS(1) [],
    ],
];

// End generated register constants for spi_host
