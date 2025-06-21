// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use core::cell::Cell;
use core::num::NonZeroUsize;
use kernel::errorcode::ErrorCode;
use kernel::hil::uart::{self, Configure, Receive, ReceiveClient, Transmit, TransmitClient};
use kernel::utilities::StaticRef;
use kernel::utilities::{
    cells::{OptionalCell, TakeCell},
    registers::{
        interfaces::{ReadWriteable, Readable, Writeable},
        register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
    },
};

register_structs! {
    Scb5Registers {
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        (0x004 => status: ReadOnly<u32>),
        (0x008 => cmd_resp_ctrl: ReadWrite<u32, CMD_RESP_CTRL::Register>),
        (0x00C => cmd_resp_status: ReadOnly<u32, CMD_RESP_STATUS::Register>),
        (0x010 => _reserved0),
        (0x020 => spi_ctrl: ReadWrite<u32, SPI_CTRL::Register>),
        (0x024 => spi_status: ReadOnly<u32, SPI_STATUS::Register>),
        (0x028 => _reserved1),
        (0x040 => uart_ctrl: ReadWrite<u32, UART_CTRL::Register>),
        (0x044 => uart_tx_ctrl: ReadWrite<u32, UART_TX_CTRL::Register>),
        (0x048 => uart_rx_ctrl: ReadWrite<u32, UART_RX_CTRL::Register>),
        (0x04C => uart_rx_status: ReadOnly<u32>),
        (0x050 => uart_flow_ctrl: ReadWrite<u32, UART_FLOW_CTRL::Register>),
        (0x054 => _reserved2),
        (0x060 => i2c_ctrl: ReadWrite<u32, I2C_CTRL::Register>),
        (0x064 => i2c_status: ReadOnly<u32, I2C_STATUS::Register>),
        (0x068 => i2c_m_cmd: ReadWrite<u32, I2C_M_CMD::Register>),
        (0x06C => i2c_s_cmd: ReadWrite<u32, I2C_S_CMD::Register>),
        (0x070 => i2c_cfg: ReadWrite<u32, I2C_CFG::Register>),
        (0x074 => _reserved3),
        (0x200 => tx_ctrl: ReadWrite<u32, TX_CTRL::Register>),
        (0x204 => tx_fifo_ctrl: ReadWrite<u32, TX_FIFO_CTRL::Register>),
        (0x208 => tx_fifo_status: ReadOnly<u32, TX_FIFO_STATUS::Register>),
        (0x20C => _reserved4),
        (0x240 => tx_fifo_wr: WriteOnly<u32, TX_FIFO_WR::Register>),
        (0x244 => _reserved5),
        (0x300 => rx_ctrl: ReadWrite<u32, RX_CTRL::Register>),
        (0x304 => rx_fifo_ctrl: ReadWrite<u32, RX_FIFO_CTRL::Register>),
        (0x308 => rx_fifo_status: ReadOnly<u32, RX_FIFO_STATUS::Register>),
        (0x30C => _reserved6),
        (0x310 => rx_match: ReadWrite<u32, RX_MATCH::Register>),
        (0x314 => _reserved7),
        (0x340 => rx_fifo_rd: ReadOnly<u32, RX_FIFO_RD::Register>),
        (0x344 => rx_fifo_rd_silent: ReadOnly<u32>),
        (0x348 => _reserved8),
        (0xE00 => intr_cause: ReadOnly<u32, INTR_CAUSE::Register>),
        (0xE04 => _reserved9),
        (0xE80 => intr_i2c_ec: ReadWrite<u32, INTR_I2C_EC::Register>),
        (0xE84 => _reserved10),
        (0xE88 => intr_i2c_ec_mask: ReadWrite<u32, INTR_I2C_EC_MASK::Register>),
        (0xE8C => intr_i2c_ec_masked: ReadOnly<u32, INTR_I2C_EC_MASKED::Register>),
        (0xE90 => _reserved11),
        (0xEC0 => intr_spi_ec: ReadWrite<u32, INTR_SPI_EC::Register>),
        (0xEC4 => _reserved12),
        (0xEC8 => intr_spi_ec_mask: ReadWrite<u32, INTR_SPI_EC_MASK::Register>),
        (0xECC => intr_spi_ec_masked: ReadOnly<u32, INTR_SPI_EC_MASKED::Register>),
        (0xED0 => _reserved13),
        (0xF00 => intr_m: ReadWrite<u32, INTR_M::Register>),
        (0xF04 => intr_m_set: ReadWrite<u32, INTR_M_SET::Register>),
        (0xF08 => intr_m_mask: ReadWrite<u32, INTR_M_MASK::Register>),
        (0xF0C => intr_m_masked: ReadOnly<u32, INTR_M_MASKED::Register>),
        (0xF10 => _reserved14),
        (0xF40 => intr_s: ReadWrite<u32, INTR_S::Register>),
        (0xF44 => intr_s_set: ReadWrite<u32, INTR_S_SET::Register>),
        (0xF48 => intr_s_mask: ReadWrite<u32, INTR_S_MASK::Register>),
        (0xF4C => intr_s_masked: ReadOnly<u32, INTR_S_MASKED::Register>),
        (0xF50 => _reserved15),
        (0xF80 => intr_tx: ReadWrite<u32, INTR_TX::Register>),
        (0xF84 => intr_tx_set: ReadWrite<u32, INTR_TX_SET::Register>),
        (0xF88 => intr_tx_mask: ReadWrite<u32, INTR_TX_MASK::Register>),
        (0xF8C => intr_tx_masked: ReadOnly<u32, INTR_TX_MASKED::Register>),
        (0xF90 => _reserved16),
        (0xFC0 => intr_rx: ReadWrite<u32, INTR_RX::Register>),
        (0xFC4 => intr_rx_set: ReadWrite<u32, INTR_RX_SET::Register>),
        (0xFC8 => intr_rx_mask: ReadWrite<u32, INTR_RX_MASK::Register>),
        (0xFCC => intr_rx_masked: ReadOnly<u32, INTR_RX_MASKED::Register>),
        (0xFD0 => @END),
    }
}
register_bitfields![u32,
CTRL [
    OVS OFFSET(0) NUMBITS(4) [],
    EC_AM_MODE OFFSET(8) NUMBITS(1) [],
    EC_OP_MODE OFFSET(9) NUMBITS(1) [],
    EZ_MODE OFFSET(10) NUMBITS(1) [],
    BYTE_MODE OFFSET(11) NUMBITS(1) [],
    CMD_RESP_MODE OFFSET(12) NUMBITS(1) [],
    ADDR_ACCEPT OFFSET(16) NUMBITS(1) [],
    BLOCK OFFSET(17) NUMBITS(1) [],
    MODE OFFSET(24) NUMBITS(2) [
        InterIntegratedCircuitsI2CMode = 0,
        SerialPeripheralInterfaceSPIMode = 1,
        UniversalAsynchronousReceiverTransmitterUARTMode = 2
    ],
    ENABLED OFFSET(31) NUMBITS(1) []
],
STATUS [
    EC_BUSY OFFSET(0) NUMBITS(1) []
],
CMD_RESP_CTRL [
    BASE_RD_ADDR OFFSET(0) NUMBITS(9) [],
    BASE_WR_ADDR OFFSET(16) NUMBITS(9) []
],
CMD_RESP_STATUS [
    CURR_RD_ADDR OFFSET(0) NUMBITS(9) [],
    CURR_WR_ADDR OFFSET(16) NUMBITS(9) [],
    CMD_RESP_EC_BUS_BUSY OFFSET(30) NUMBITS(1) [],
    CMD_RESP_EC_BUSY OFFSET(31) NUMBITS(1) []
],
SPI_CTRL [
    SSEL_CONTINUOUS OFFSET(0) NUMBITS(1) [],
    SELECT_PRECEDE OFFSET(1) NUMBITS(1) [],
    CPHA OFFSET(2) NUMBITS(1) [],
    CPOL OFFSET(3) NUMBITS(1) [],
    LATE_MISO_SAMPLE OFFSET(4) NUMBITS(1) [],
    SCLK_CONTINUOUS OFFSET(5) NUMBITS(1) [],
    SSEL_POLARITY0 OFFSET(8) NUMBITS(1) [],
    SSEL_POLARITY1 OFFSET(9) NUMBITS(1) [],
    SSEL_POLARITY2 OFFSET(10) NUMBITS(1) [],
    SSEL_POLARITY3 OFFSET(11) NUMBITS(1) [],
    LOOPBACK OFFSET(16) NUMBITS(1) [],
    MODE OFFSET(24) NUMBITS(2) [
        SPI_MOTOROLA = 0,
        SPI_TI = 1,
        SPI_NS = 2
    ],
    SSEL OFFSET(26) NUMBITS(2) [],
    MASTER_MODE OFFSET(31) NUMBITS(1) []
],
SPI_STATUS [
    BUS_BUSY OFFSET(0) NUMBITS(1) [],
    SPI_EC_BUSY OFFSET(1) NUMBITS(1) [],
    CURR_EZ_ADDR OFFSET(8) NUMBITS(8) [],
    BASE_EZ_ADDR OFFSET(16) NUMBITS(8) []
],
UART_CTRL [
    LOOPBACK OFFSET(16) NUMBITS(1) [],
    MODE OFFSET(24) NUMBITS(2) [
        StandardUARTSubmode = 0,
        UART_SMARTCARD = 1,
        InfraredDataAssociationIrDASubmodeReturnToZeroModulationScheme = 2
    ]
],
UART_TX_CTRL [
    STOP_BITS OFFSET(0) NUMBITS(3) [],
    PARITY OFFSET(4) NUMBITS(1) [],
    PARITY_ENABLED OFFSET(5) NUMBITS(1) [],
    RETRY_ON_NACK OFFSET(8) NUMBITS(1) []
],
UART_RX_CTRL [
    STOP_BITS OFFSET(0) NUMBITS(3) [],
    PARITY OFFSET(4) NUMBITS(1) [],
    PARITY_ENABLED OFFSET(5) NUMBITS(1) [],
    POLARITY OFFSET(6) NUMBITS(1) [],
    DROP_ON_PARITY_ERROR OFFSET(8) NUMBITS(1) [],
    DROP_ON_FRAME_ERROR OFFSET(9) NUMBITS(1) [],
    MP_MODE OFFSET(10) NUMBITS(1) [],
    LIN_MODE OFFSET(12) NUMBITS(1) [],
    SKIP_START OFFSET(13) NUMBITS(1) [],
    BREAK_WIDTH OFFSET(16) NUMBITS(4) []
],
UART_RX_STATUS [
    BR_COUNTER OFFSET(0) NUMBITS(12) []
],
UART_FLOW_CTRL [
    TRIGGER_LEVEL OFFSET(0) NUMBITS(8) [],
    RTS_POLARITY OFFSET(16) NUMBITS(1) [],
    CTS_POLARITY OFFSET(24) NUMBITS(1) [],
    CTS_ENABLED OFFSET(25) NUMBITS(1) []
],
I2C_CTRL [
    HIGH_PHASE_OVS OFFSET(0) NUMBITS(4) [],
    LOW_PHASE_OVS OFFSET(4) NUMBITS(4) [],
    M_READY_DATA_ACK OFFSET(8) NUMBITS(1) [],
    M_NOT_READY_DATA_NACK OFFSET(9) NUMBITS(1) [],
    S_GENERAL_IGNORE OFFSET(11) NUMBITS(1) [],
    S_READY_ADDR_ACK OFFSET(12) NUMBITS(1) [],
    S_READY_DATA_ACK OFFSET(13) NUMBITS(1) [],
    S_NOT_READY_ADDR_NACK OFFSET(14) NUMBITS(1) [],
    S_NOT_READY_DATA_NACK OFFSET(15) NUMBITS(1) [],
    LOOPBACK OFFSET(16) NUMBITS(1) [],
    SLAVE_MODE OFFSET(30) NUMBITS(1) [],
    MASTER_MODE OFFSET(31) NUMBITS(1) []
],
I2C_STATUS [
    BUS_BUSY OFFSET(0) NUMBITS(1) [],
    I2C_EC_BUSY OFFSET(1) NUMBITS(1) [],
    S_READ OFFSET(4) NUMBITS(1) [],
    M_READ OFFSET(5) NUMBITS(1) [],
    CURR_EZ_ADDR OFFSET(8) NUMBITS(8) [],
    BASE_EZ_ADDR OFFSET(16) NUMBITS(8) []
],
I2C_M_CMD [
    M_START OFFSET(0) NUMBITS(1) [],
    M_START_ON_IDLE OFFSET(1) NUMBITS(1) [],
    M_ACK OFFSET(2) NUMBITS(1) [],
    M_NACK OFFSET(3) NUMBITS(1) [],
    M_STOP OFFSET(4) NUMBITS(1) []
],
I2C_S_CMD [
    S_ACK OFFSET(0) NUMBITS(1) [],
    S_NACK OFFSET(1) NUMBITS(1) []
],
I2C_CFG [
    SDA_IN_FILT_TRIM OFFSET(0) NUMBITS(2) [],
    SDA_IN_FILT_SEL OFFSET(4) NUMBITS(1) [],
    SCL_IN_FILT_TRIM OFFSET(8) NUMBITS(2) [],
    SCL_IN_FILT_SEL OFFSET(12) NUMBITS(1) [],
    SDA_OUT_FILT0_TRIM OFFSET(16) NUMBITS(2) [],
    SDA_OUT_FILT1_TRIM OFFSET(18) NUMBITS(2) [],
    SDA_OUT_FILT2_TRIM OFFSET(20) NUMBITS(2) [],
    SDA_OUT_FILT_SEL OFFSET(28) NUMBITS(2) []
],
TX_CTRL [
    DATA_WIDTH OFFSET(0) NUMBITS(4) [],
    MSB_FIRST OFFSET(8) NUMBITS(1) [],
    OPEN_DRAIN OFFSET(16) NUMBITS(1) []
],
TX_FIFO_CTRL [
    TRIGGER_LEVEL OFFSET(0) NUMBITS(8) [],
    CLEAR OFFSET(16) NUMBITS(1) [],
    FREEZE OFFSET(17) NUMBITS(1) []
],
TX_FIFO_STATUS [
    USED OFFSET(0) NUMBITS(9) [],
    SR_VALID OFFSET(15) NUMBITS(1) [],
    RD_PTR OFFSET(16) NUMBITS(8) [],
    WR_PTR OFFSET(24) NUMBITS(8) []
],
TX_FIFO_WR [
    DATA OFFSET(0) NUMBITS(16) []
],
RX_CTRL [
    DATA_WIDTH OFFSET(0) NUMBITS(4) [],
    MSB_FIRST OFFSET(8) NUMBITS(1) [],
    MEDIAN OFFSET(9) NUMBITS(1) []
],
RX_FIFO_CTRL [
    TRIGGER_LEVEL OFFSET(0) NUMBITS(8) [],
    CLEAR OFFSET(16) NUMBITS(1) [],
    FREEZE OFFSET(17) NUMBITS(1) []
],
RX_FIFO_STATUS [
    USED OFFSET(0) NUMBITS(9) [],
    SR_VALID OFFSET(15) NUMBITS(1) [],
    RD_PTR OFFSET(16) NUMBITS(8) [],
    WR_PTR OFFSET(24) NUMBITS(8) []
],
RX_MATCH [
    ADDR OFFSET(0) NUMBITS(8) [],
    MASK OFFSET(16) NUMBITS(8) []
],
RX_FIFO_RD [
    DATA OFFSET(0) NUMBITS(16) []
],
RX_FIFO_RD_SILENT [
    DATA OFFSET(0) NUMBITS(16) []
],
INTR_CAUSE [
    M OFFSET(0) NUMBITS(1) [],
    S OFFSET(1) NUMBITS(1) [],
    TX OFFSET(2) NUMBITS(1) [],
    RX OFFSET(3) NUMBITS(1) [],
    I2C_EC OFFSET(4) NUMBITS(1) [],
    SPI_EC OFFSET(5) NUMBITS(1) []
],
INTR_I2C_EC [
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
INTR_I2C_EC_MASK [
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
INTR_I2C_EC_MASKED [
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
INTR_SPI_EC [
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
INTR_SPI_EC_MASK [
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
INTR_SPI_EC_MASKED [
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
INTR_M [
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    SPI_DONE OFFSET(9) NUMBITS(1) []
],
INTR_M_SET [
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    SPI_DONE OFFSET(9) NUMBITS(1) []
],
INTR_M_MASK [
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    SPI_DONE OFFSET(9) NUMBITS(1) []
],
INTR_M_MASKED [
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    SPI_DONE OFFSET(9) NUMBITS(1) []
],
INTR_S [
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    I2C_WRITE_STOP OFFSET(3) NUMBITS(1) [],
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    I2C_START OFFSET(5) NUMBITS(1) [],
    I2C_ADDR_MATCH OFFSET(6) NUMBITS(1) [],
    I2C_GENERAL OFFSET(7) NUMBITS(1) [],
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    SPI_EZ_WRITE_STOP OFFSET(9) NUMBITS(1) [],
    SPI_EZ_STOP OFFSET(10) NUMBITS(1) [],
    SPI_BUS_ERROR OFFSET(11) NUMBITS(1) []
],
INTR_S_SET [
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    I2C_WRITE_STOP OFFSET(3) NUMBITS(1) [],
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    I2C_START OFFSET(5) NUMBITS(1) [],
    I2C_ADDR_MATCH OFFSET(6) NUMBITS(1) [],
    I2C_GENERAL OFFSET(7) NUMBITS(1) [],
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    SPI_EZ_WRITE_STOP OFFSET(9) NUMBITS(1) [],
    SPI_EZ_STOP OFFSET(10) NUMBITS(1) [],
    SPI_BUS_ERROR OFFSET(11) NUMBITS(1) []
],
INTR_S_MASK [
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    I2C_WRITE_STOP OFFSET(3) NUMBITS(1) [],
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    I2C_START OFFSET(5) NUMBITS(1) [],
    I2C_ADDR_MATCH OFFSET(6) NUMBITS(1) [],
    I2C_GENERAL OFFSET(7) NUMBITS(1) [],
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    SPI_EZ_WRITE_STOP OFFSET(9) NUMBITS(1) [],
    SPI_EZ_STOP OFFSET(10) NUMBITS(1) [],
    SPI_BUS_ERROR OFFSET(11) NUMBITS(1) []
],
INTR_S_MASKED [
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    I2C_WRITE_STOP OFFSET(3) NUMBITS(1) [],
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    I2C_START OFFSET(5) NUMBITS(1) [],
    I2C_ADDR_MATCH OFFSET(6) NUMBITS(1) [],
    I2C_GENERAL OFFSET(7) NUMBITS(1) [],
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    SPI_EZ_WRITE_STOP OFFSET(9) NUMBITS(1) [],
    SPI_EZ_STOP OFFSET(10) NUMBITS(1) [],
    SPI_BUS_ERROR OFFSET(11) NUMBITS(1) []
],
INTR_TX [
    TRIGGER OFFSET(0) NUMBITS(1) [],
    NOT_FULL OFFSET(1) NUMBITS(1) [],
    EMPTY OFFSET(4) NUMBITS(1) [],
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    BLOCKED OFFSET(7) NUMBITS(1) [],
    UART_NACK OFFSET(8) NUMBITS(1) [],
    UART_DONE OFFSET(9) NUMBITS(1) [],
    UART_ARB_LOST OFFSET(10) NUMBITS(1) []
],
INTR_TX_SET [
    TRIGGER OFFSET(0) NUMBITS(1) [],
    NOT_FULL OFFSET(1) NUMBITS(1) [],
    EMPTY OFFSET(4) NUMBITS(1) [],
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    BLOCKED OFFSET(7) NUMBITS(1) [],
    UART_NACK OFFSET(8) NUMBITS(1) [],
    UART_DONE OFFSET(9) NUMBITS(1) [],
    UART_ARB_LOST OFFSET(10) NUMBITS(1) []
],
INTR_TX_MASK [
    TRIGGER OFFSET(0) NUMBITS(1) [],
    NOT_FULL OFFSET(1) NUMBITS(1) [],
    EMPTY OFFSET(4) NUMBITS(1) [],
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    BLOCKED OFFSET(7) NUMBITS(1) [],
    UART_NACK OFFSET(8) NUMBITS(1) [],
    UART_DONE OFFSET(9) NUMBITS(1) [],
    UART_ARB_LOST OFFSET(10) NUMBITS(1) []
],
INTR_TX_MASKED [
    TRIGGER OFFSET(0) NUMBITS(1) [],
    NOT_FULL OFFSET(1) NUMBITS(1) [],
    EMPTY OFFSET(4) NUMBITS(1) [],
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    BLOCKED OFFSET(7) NUMBITS(1) [],
    UART_NACK OFFSET(8) NUMBITS(1) [],
    UART_DONE OFFSET(9) NUMBITS(1) [],
    UART_ARB_LOST OFFSET(10) NUMBITS(1) []
],
INTR_RX [
    TRIGGER OFFSET(0) NUMBITS(1) [],
    NOT_EMPTY OFFSET(2) NUMBITS(1) [],
    FULL OFFSET(3) NUMBITS(1) [],
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    BLOCKED OFFSET(7) NUMBITS(1) [],
    FRAME_ERROR OFFSET(8) NUMBITS(1) [],
    PARITY_ERROR OFFSET(9) NUMBITS(1) [],
    BAUD_DETECT OFFSET(10) NUMBITS(1) [],
    BREAK_DETECT OFFSET(11) NUMBITS(1) []
],
INTR_RX_SET [
    TRIGGER OFFSET(0) NUMBITS(1) [],
    NOT_EMPTY OFFSET(2) NUMBITS(1) [],
    FULL OFFSET(3) NUMBITS(1) [],
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    BLOCKED OFFSET(7) NUMBITS(1) [],
    FRAME_ERROR OFFSET(8) NUMBITS(1) [],
    PARITY_ERROR OFFSET(9) NUMBITS(1) [],
    BAUD_DETECT OFFSET(10) NUMBITS(1) [],
    BREAK_DETECT OFFSET(11) NUMBITS(1) []
],
INTR_RX_MASK [
    TRIGGER OFFSET(0) NUMBITS(1) [],
    NOT_EMPTY OFFSET(2) NUMBITS(1) [],
    FULL OFFSET(3) NUMBITS(1) [],
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    BLOCKED OFFSET(7) NUMBITS(1) [],
    FRAME_ERROR OFFSET(8) NUMBITS(1) [],
    PARITY_ERROR OFFSET(9) NUMBITS(1) [],
    BAUD_DETECT OFFSET(10) NUMBITS(1) [],
    BREAK_DETECT OFFSET(11) NUMBITS(1) []
],
INTR_RX_MASKED [
    TRIGGER OFFSET(0) NUMBITS(1) [],
    NOT_EMPTY OFFSET(2) NUMBITS(1) [],
    FULL OFFSET(3) NUMBITS(1) [],
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    BLOCKED OFFSET(7) NUMBITS(1) [],
    FRAME_ERROR OFFSET(8) NUMBITS(1) [],
    PARITY_ERROR OFFSET(9) NUMBITS(1) [],
    BAUD_DETECT OFFSET(10) NUMBITS(1) [],
    BREAK_DETECT OFFSET(11) NUMBITS(1) []
]
];
const SCB5_BASE: StaticRef<Scb5Registers> =
    unsafe { StaticRef::new(0x40650000 as *const Scb5Registers) };

pub struct Scb<'a> {
    registers: StaticRef<Scb5Registers>,

    tx_client: OptionalCell<&'a dyn TransmitClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_length: OptionalCell<NonZeroUsize>,
    tx_position: Cell<usize>,

    rx_client: OptionalCell<&'a dyn ReceiveClient>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_length: OptionalCell<NonZeroUsize>,
    rx_position: Cell<usize>,
}

impl Scb<'_> {
    pub const fn new() -> Self {
        Self {
            registers: SCB5_BASE,

            tx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_length: OptionalCell::empty(),
            tx_position: Cell::new(0),

            rx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            rx_length: OptionalCell::empty(),
            rx_position: Cell::new(0),
        }
    }

    pub fn enable_tx_interrupts(&self) {
        self.registers
            .intr_tx_mask
            .modify(INTR_TX_MASK::UART_DONE::SET);
    }

    pub fn disable_tx_interrupts(&self) {
        self.registers
            .intr_tx_mask
            .modify(INTR_TX_MASK::UART_DONE::CLEAR);
    }

    pub fn enable_rx_interrupts(&self) {
        self.registers
            .intr_rx_mask
            .modify(INTR_RX_MASK::NOT_EMPTY::SET);
    }

    pub fn disable_rx_interrupts(&self) {
        self.registers
            .intr_rx_mask
            .modify(INTR_RX_MASK::NOT_EMPTY::CLEAR);
    }

    pub(crate) fn handle_interrupt(&self) {
        if self.registers.intr_tx.is_set(INTR_TX::UART_DONE) {
            self.disable_tx_interrupts();
            self.registers.intr_tx.modify(INTR_TX::UART_DONE::SET);
            // SAFETY: When a transmit is started, length is set to a non-zero value.
            if self.tx_length.get().is_none() {
                return;
            }
            let tx_length = self.tx_length.get().unwrap().get();
            if tx_length == self.tx_position.get() + 1 {
                self.tx_length.clear();
                // SAFETY: When a transmit is started, a buffer is passed.
                self.tx_client.map(|client| {
                    client.transmitted_buffer(self.tx_buffer.take().unwrap(), tx_length, Ok(()))
                });
            } else {
                let current_position = self.tx_position.get();
                // SAFETY: Because of the if condition, current_position + 1 < buffer.len().
                self.tx_buffer.map(|buffer| {
                    self.registers.tx_fifo_wr.write(
                        TX_FIFO_WR::DATA.val(*buffer.get(current_position + 1).unwrap() as u32),
                    )
                });
                self.tx_position.set(current_position + 1);
                self.enable_tx_interrupts();
            }
        }
        if self.registers.intr_rx.is_set(INTR_RX::NOT_EMPTY) {
            let byte = self.registers.rx_fifo_rd.read(RX_FIFO_RD::DATA) as u8;
            // The caller must ensure that the FIFO buffer is empty before clearing the interrupt.
            self.registers.intr_rx.modify(INTR_RX::NOT_EMPTY::SET);
            // If no rx_buffer is set, then no reception is pending. Simply discard the received
            // byte.
            if let Some(rx_buffer) = self.rx_buffer.take() {
                let mut current_position = self.rx_position.get();
                rx_buffer[current_position] = byte;
                current_position += 1;
                // SAFETY: When a read is started, rx_length is set to a non-zero value.
                let rx_length = self.rx_length.get().unwrap().get();
                if current_position == rx_length {
                    self.rx_length.clear();
                    self.rx_client.map(|client| {
                        client.received_buffer(
                            rx_buffer,
                            rx_length,
                            Ok(()),
                            kernel::hil::uart::Error::None,
                        )
                    });
                } else {
                    self.rx_position.set(current_position);
                    self.rx_buffer.replace(rx_buffer);
                }
            }
        }
    }

    pub fn set_standard_uart_mode(&self) {
        self.registers
            .ctrl
            .modify(CTRL::MODE::UniversalAsynchronousReceiverTransmitterUARTMode);
        self.registers
            .ctrl
            .modify(CTRL::OVS.val(14) + CTRL::EC_AM_MODE.val(0) + CTRL::EC_OP_MODE.val(0));
        self.registers
            .uart_ctrl
            .modify(UART_CTRL::MODE::StandardUARTSubmode);
        self.registers
            .uart_rx_ctrl
            .modify(UART_RX_CTRL::MP_MODE::CLEAR + UART_RX_CTRL::LIN_MODE::CLEAR);

        self.set_uart_sync();
    }

    pub fn enable_scb(&self) {
        self.registers.ctrl.modify(CTRL::ENABLED::SET);
    }

    pub fn disable_scb(&self) {
        self.registers.ctrl.modify(CTRL::ENABLED::CLEAR);
    }

    fn set_uart_sync(&self) {
        self.registers.ctrl.modify(CTRL::BYTE_MODE::SET);
        self.registers
            .tx_ctrl
            .modify(TX_CTRL::DATA_WIDTH.val(7) + TX_CTRL::MSB_FIRST::CLEAR);

        self.registers
            .rx_ctrl
            .modify(RX_CTRL::DATA_WIDTH.val(7) + RX_CTRL::MSB_FIRST::CLEAR);

        self.registers.tx_fifo_wr.write(TX_FIFO_WR::DATA.val(0));

        self.registers
            .tx_fifo_ctrl
            .modify(TX_FIFO_CTRL::TRIGGER_LEVEL.val(1));
        self.registers.tx_fifo_ctrl.modify(TX_FIFO_CTRL::CLEAR::SET);
        while !self.uart_is_transmitter_done() {}
        self.registers
            .tx_fifo_ctrl
            .modify(TX_FIFO_CTRL::CLEAR::CLEAR);

        self.registers
            .rx_fifo_ctrl
            .modify(RX_FIFO_CTRL::TRIGGER_LEVEL.val(1));
        self.registers.rx_fifo_ctrl.modify(RX_FIFO_CTRL::CLEAR::SET);
        self.registers
            .rx_fifo_ctrl
            .modify(RX_FIFO_CTRL::CLEAR::CLEAR);

        self.registers
            .uart_tx_ctrl
            .modify(UART_TX_CTRL::PARITY::CLEAR);
        self.registers
            .uart_tx_ctrl
            .modify(UART_TX_CTRL::STOP_BITS.val(1));

        self.registers
            .uart_rx_ctrl
            .modify(UART_RX_CTRL::PARITY::CLEAR);
        self.registers
            .uart_rx_ctrl
            .modify(UART_RX_CTRL::STOP_BITS.val(1));

        self.registers
            .uart_flow_ctrl
            .modify(UART_FLOW_CTRL::CTS_ENABLED::CLEAR);
    }

    fn uart_is_transmitter_done(&self) -> bool {
        self.registers.tx_fifo_status.read(TX_FIFO_STATUS::SR_VALID) == 0
    }

    pub fn transmit_uart_sync(&self, buffer: &[u8]) {
        for byte in buffer {
            self.registers
                .tx_fifo_wr
                .write(TX_FIFO_WR::DATA.val(*byte as u32));

            while !self.uart_is_transmitter_done() {}
        }
    }

    pub fn transmit_uart_async(
        &self,
        buffer: &'static mut [u8],
        buffer_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.tx_length.is_some() {
            Err((ErrorCode::BUSY, buffer))
        } else if buffer.len() < buffer_len || buffer_len == 0 {
            Err((ErrorCode::SIZE, buffer))
        } else {
            match NonZeroUsize::new(buffer_len) {
                Some(tx_length) => {
                    self.registers
                        .tx_fifo_wr
                        .write(TX_FIFO_WR::DATA.val(*buffer.get(0).unwrap() as u32));
                    self.tx_buffer.put(Some(buffer));
                    self.tx_length.set(tx_length);
                    self.tx_position.set(0);
                    self.enable_tx_interrupts();
                    Ok(())
                }
                None => Err((ErrorCode::SIZE, buffer)),
            }
        }
    }

    pub fn receive_uart_async(
        &self,
        buffer: &'static mut [u8],
        buffer_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.rx_length.is_some() {
            Err((ErrorCode::BUSY, buffer))
        } else if buffer.len() < buffer_len || buffer_len == 0 {
            Err((ErrorCode::SIZE, buffer))
        } else {
            match NonZeroUsize::new(buffer_len) {
                Some(rx_length) => {
                    self.enable_rx_interrupts();
                    self.rx_buffer.put(Some(buffer));
                    self.rx_length.set(rx_length);
                    self.rx_position.set(0);
                    Ok(())
                }
                None => Err((ErrorCode::SIZE, buffer)),
            }
        }
    }
}

impl<'a> Transmit<'a> for Scb<'a> {
    fn set_transmit_client(&self, client: &'a dyn TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        self.transmit_uart_async(tx_buffer, tx_len)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl<'a> Receive<'a> for Scb<'a> {
    fn set_receive_client(&self, client: &'a dyn ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.receive_uart_async(rx_buffer, rx_len)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl Configure for Scb<'_> {
    fn configure(&self, params: kernel::hil::uart::Parameters) -> Result<(), ErrorCode> {
        if params.baud_rate != 115200 || params.hw_flow_control {
            Err(ErrorCode::NOSUPPORT)
        } else {
            // Modification of the SCB parameters require it to be disabled.
            if self.registers.ctrl.is_set(CTRL::ENABLED) {
                return Err(ErrorCode::BUSY);
            }
            match params.stop_bits {
                uart::StopBits::One => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::STOP_BITS.val(1));
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::STOP_BITS.val(1));
                }
                uart::StopBits::Two => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::STOP_BITS.val(3));
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::STOP_BITS.val(3));
                }
            }
            match params.parity {
                uart::Parity::None => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::PARITY_ENABLED::CLEAR);
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::PARITY_ENABLED::CLEAR);
                }
                uart::Parity::Odd => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::PARITY_ENABLED::SET + UART_TX_CTRL::PARITY::SET);
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::PARITY_ENABLED::SET + UART_RX_CTRL::PARITY::SET);
                }
                uart::Parity::Even => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::PARITY_ENABLED::SET + UART_TX_CTRL::PARITY::CLEAR);
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::PARITY_ENABLED::SET + UART_RX_CTRL::PARITY::CLEAR);
                }
            }
            match params.width {
                uart::Width::Six => {
                    self.registers.tx_ctrl.modify(TX_CTRL::DATA_WIDTH.val(5));
                }
                uart::Width::Seven => {
                    self.registers.tx_ctrl.modify(TX_CTRL::DATA_WIDTH.val(6));
                }
                uart::Width::Eight => {
                    self.registers.tx_ctrl.modify(TX_CTRL::DATA_WIDTH.val(7));
                }
            }
            Ok(())
        }
    }
}
