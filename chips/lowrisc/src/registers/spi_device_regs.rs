// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for spi_device.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/spi_device/data/spi_device.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of alerts
pub const SPI_DEVICE_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const SPI_DEVICE_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub SpiDeviceRegisters {
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
        /// Configuration Register
        (0x0014 => pub(crate) cfg: ReadWrite<u32, CFG::Register>),
        /// RX/ TX FIFO levels.
        (0x0018 => pub(crate) fifo_level: ReadWrite<u32, FIFO_LEVEL::Register>),
        /// RX/ TX Async FIFO levels between main clk and spi clock
        (0x001c => pub(crate) async_fifo_level: ReadWrite<u32, ASYNC_FIFO_LEVEL::Register>),
        /// SPI Device status register
        (0x0020 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        /// Receiver FIFO (SRAM) pointers
        (0x0024 => pub(crate) rxf_ptr: ReadWrite<u32, RXF_PTR::Register>),
        /// Transmitter FIFO (SRAM) pointers
        (0x0028 => pub(crate) txf_ptr: ReadWrite<u32, TXF_PTR::Register>),
        /// Receiver FIFO (SRAM) Addresses
        (0x002c => pub(crate) rxf_addr: ReadWrite<u32, RXF_ADDR::Register>),
        /// Transmitter FIFO (SRAM) Addresses
        (0x0030 => pub(crate) txf_addr: ReadWrite<u32, TXF_ADDR::Register>),
        /// Intercept Passthrough datapath.
        (0x0034 => pub(crate) intercept_en: ReadWrite<u32, INTERCEPT_EN::Register>),
        /// Last Read Address
        (0x0038 => pub(crate) last_read_addr: ReadWrite<u32, LAST_READ_ADDR::Register>),
        /// SPI Flash Status register.
        (0x003c => pub(crate) flash_status: ReadWrite<u32, FLASH_STATUS::Register>),
        /// JEDEC Continuation Code configuration register.
        (0x0040 => pub(crate) jedec_cc: ReadWrite<u32, JEDEC_CC::Register>),
        /// JEDEC ID register.
        (0x0044 => pub(crate) jedec_id: ReadWrite<u32, JEDEC_ID::Register>),
        /// Read Buffer threshold register.
        (0x0048 => pub(crate) read_threshold: ReadWrite<u32, READ_THRESHOLD::Register>),
        /// Mailbox Base address register.
        (0x004c => pub(crate) mailbox_addr: ReadWrite<u32, MAILBOX_ADDR::Register>),
        /// Upload module status register.
        (0x0050 => pub(crate) upload_status: ReadWrite<u32, UPLOAD_STATUS::Register>),
        /// Upload module status 2 register.
        (0x0054 => pub(crate) upload_status2: ReadWrite<u32, UPLOAD_STATUS2::Register>),
        /// Command Fifo Read Port.
        (0x0058 => pub(crate) upload_cmdfifo: ReadWrite<u32, UPLOAD_CMDFIFO::Register>),
        /// Address Fifo Read Port.
        (0x005c => pub(crate) upload_addrfifo: ReadWrite<u32, UPLOAD_ADDRFIFO::Register>),
        /// Command Filter
        (0x0060 => pub(crate) cmd_filter: [ReadWrite<u32, CMD_FILTER::Register>; 8]),
        /// Address Swap Mask register.
        (0x0080 => pub(crate) addr_swap_mask: ReadWrite<u32, ADDR_SWAP_MASK::Register>),
        /// The address value for the address swap feature.
        (0x0084 => pub(crate) addr_swap_data: ReadWrite<u32, ADDR_SWAP_DATA::Register>),
        /// Write Data Swap in the passthrough mode.
        (0x0088 => pub(crate) payload_swap_mask: ReadWrite<u32, PAYLOAD_SWAP_MASK::Register>),
        /// Write Data Swap in the passthrough mode.
        (0x008c => pub(crate) payload_swap_data: ReadWrite<u32, PAYLOAD_SWAP_DATA::Register>),
        /// Command Info register.
        (0x0090 => pub(crate) cmd_info: [ReadWrite<u32, CMD_INFO::Register>; 24]),
        /// Opcode for EN4B.
        (0x00f0 => pub(crate) cmd_info_en4b: ReadWrite<u32, CMD_INFO_EN4B::Register>),
        /// Opcode for EX4B
        (0x00f4 => pub(crate) cmd_info_ex4b: ReadWrite<u32, CMD_INFO_EX4B::Register>),
        /// Opcode for Write Enable (WREN)
        (0x00f8 => pub(crate) cmd_info_wren: ReadWrite<u32, CMD_INFO_WREN::Register>),
        /// Opcode for Write Disable (WRDI)
        (0x00fc => pub(crate) cmd_info_wrdi: ReadWrite<u32, CMD_INFO_WRDI::Register>),
        (0x0100 => _reserved1),
        /// TPM HWIP Capability register.
        (0x0800 => pub(crate) tpm_cap: ReadWrite<u32, TPM_CAP::Register>),
        /// TPM Configuration register.
        (0x0804 => pub(crate) tpm_cfg: ReadWrite<u32, TPM_CFG::Register>),
        /// TPM submodule state register.
        (0x0808 => pub(crate) tpm_status: ReadWrite<u32, TPM_STATUS::Register>),
        /// TPM_ACCESS_x register.
        (0x080c => pub(crate) tpm_access: [ReadWrite<u32, TPM_ACCESS::Register>; 2]),
        /// TPM_STS_x register.
        (0x0814 => pub(crate) tpm_sts: ReadWrite<u32, TPM_STS::Register>),
        /// TPM_INTF_CAPABILITY
        (0x0818 => pub(crate) tpm_intf_capability: ReadWrite<u32, TPM_INTF_CAPABILITY::Register>),
        /// TPM_INT_ENABLE
        (0x081c => pub(crate) tpm_int_enable: ReadWrite<u32, TPM_INT_ENABLE::Register>),
        /// TPM_INT_VECTOR
        (0x0820 => pub(crate) tpm_int_vector: ReadWrite<u32, TPM_INT_VECTOR::Register>),
        /// TPM_INT_STATUS
        (0x0824 => pub(crate) tpm_int_status: ReadWrite<u32, TPM_INT_STATUS::Register>),
        /// TPM_DID/ TPM_VID register
        (0x0828 => pub(crate) tpm_did_vid: ReadWrite<u32, TPM_DID_VID::Register>),
        /// TPM_RID
        (0x082c => pub(crate) tpm_rid: ReadWrite<u32, TPM_RID::Register>),
        /// TPM Command and Address buffer
        (0x0830 => pub(crate) tpm_cmd_addr: ReadWrite<u32, TPM_CMD_ADDR::Register>),
        /// TPM Read command return data FIFO.
        (0x0834 => pub(crate) tpm_read_fifo: ReadWrite<u32, TPM_READ_FIFO::Register>),
        /// TPM Write command received data FIFO.
        (0x0838 => pub(crate) tpm_write_fifo: ReadWrite<u32, TPM_WRITE_FIFO::Register>),
        (0x083c => _reserved2),
        /// Memory area: SPI internal buffer.
        (0x1000 => pub(crate) buffer: [ReadWrite<u32>; 1024]),
        (0x2000 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        GENERIC_RX_FULL OFFSET(0) NUMBITS(1) [],
        GENERIC_RX_WATERMARK OFFSET(1) NUMBITS(1) [],
        GENERIC_TX_WATERMARK OFFSET(2) NUMBITS(1) [],
        GENERIC_RX_ERROR OFFSET(3) NUMBITS(1) [],
        GENERIC_RX_OVERFLOW OFFSET(4) NUMBITS(1) [],
        GENERIC_TX_UNDERFLOW OFFSET(5) NUMBITS(1) [],
        UPLOAD_CMDFIFO_NOT_EMPTY OFFSET(6) NUMBITS(1) [],
        UPLOAD_PAYLOAD_NOT_EMPTY OFFSET(7) NUMBITS(1) [],
        UPLOAD_PAYLOAD_OVERFLOW OFFSET(8) NUMBITS(1) [],
        READBUF_WATERMARK OFFSET(9) NUMBITS(1) [],
        READBUF_FLIP OFFSET(10) NUMBITS(1) [],
        TPM_HEADER_NOT_EMPTY OFFSET(11) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CONTROL [
        ABORT OFFSET(0) NUMBITS(1) [],
        MODE OFFSET(4) NUMBITS(2) [
            FWMODE = 0,
            FLASHMODE = 1,
            PASSTHROUGH = 2,
        ],
        RST_TXFIFO OFFSET(16) NUMBITS(1) [],
        RST_RXFIFO OFFSET(17) NUMBITS(1) [],
        SRAM_CLK_EN OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) CFG [
        CPOL OFFSET(0) NUMBITS(1) [],
        CPHA OFFSET(1) NUMBITS(1) [],
        TX_ORDER OFFSET(2) NUMBITS(1) [],
        RX_ORDER OFFSET(3) NUMBITS(1) [],
        TIMER_V OFFSET(8) NUMBITS(8) [],
        ADDR_4B_EN OFFSET(16) NUMBITS(1) [],
        MAILBOX_EN OFFSET(24) NUMBITS(1) [],
    ],
    pub(crate) FIFO_LEVEL [
        RXLVL OFFSET(0) NUMBITS(16) [],
        TXLVL OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) ASYNC_FIFO_LEVEL [
        RXLVL OFFSET(0) NUMBITS(8) [],
        TXLVL OFFSET(16) NUMBITS(8) [],
    ],
    pub(crate) STATUS [
        RXF_FULL OFFSET(0) NUMBITS(1) [],
        RXF_EMPTY OFFSET(1) NUMBITS(1) [],
        TXF_FULL OFFSET(2) NUMBITS(1) [],
        TXF_EMPTY OFFSET(3) NUMBITS(1) [],
        ABORT_DONE OFFSET(4) NUMBITS(1) [],
        CSB OFFSET(5) NUMBITS(1) [],
        TPM_CSB OFFSET(6) NUMBITS(1) [],
    ],
    pub(crate) RXF_PTR [
        RPTR OFFSET(0) NUMBITS(16) [],
        WPTR OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) TXF_PTR [
        RPTR OFFSET(0) NUMBITS(16) [],
        WPTR OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) RXF_ADDR [
        BASE OFFSET(0) NUMBITS(16) [],
        LIMIT OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) TXF_ADDR [
        BASE OFFSET(0) NUMBITS(16) [],
        LIMIT OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) INTERCEPT_EN [
        STATUS OFFSET(0) NUMBITS(1) [],
        JEDEC OFFSET(1) NUMBITS(1) [],
        SFDP OFFSET(2) NUMBITS(1) [],
        MBX OFFSET(3) NUMBITS(1) [],
    ],
    pub(crate) LAST_READ_ADDR [
        ADDR OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) FLASH_STATUS [
        BUSY OFFSET(0) NUMBITS(1) [],
        STATUS OFFSET(1) NUMBITS(23) [],
    ],
    pub(crate) JEDEC_CC [
        CC OFFSET(0) NUMBITS(8) [],
        NUM_CC OFFSET(8) NUMBITS(8) [],
    ],
    pub(crate) JEDEC_ID [
        ID OFFSET(0) NUMBITS(16) [],
        MF OFFSET(16) NUMBITS(8) [],
    ],
    pub(crate) READ_THRESHOLD [
        THRESHOLD OFFSET(0) NUMBITS(10) [],
    ],
    pub(crate) MAILBOX_ADDR [
        ADDR OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) UPLOAD_STATUS [
        CMDFIFO_DEPTH OFFSET(0) NUMBITS(5) [],
        CMDFIFO_NOTEMPTY OFFSET(7) NUMBITS(1) [],
        ADDRFIFO_DEPTH OFFSET(8) NUMBITS(5) [],
        ADDRFIFO_NOTEMPTY OFFSET(15) NUMBITS(1) [],
    ],
    pub(crate) UPLOAD_STATUS2 [
        PAYLOAD_DEPTH OFFSET(0) NUMBITS(9) [],
        PAYLOAD_START_IDX OFFSET(16) NUMBITS(8) [],
    ],
    pub(crate) UPLOAD_CMDFIFO [
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) UPLOAD_ADDRFIFO [
        DATA OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CMD_FILTER [
        FILTER_0 OFFSET(0) NUMBITS(1) [],
        FILTER_1 OFFSET(1) NUMBITS(1) [],
        FILTER_2 OFFSET(2) NUMBITS(1) [],
        FILTER_3 OFFSET(3) NUMBITS(1) [],
        FILTER_4 OFFSET(4) NUMBITS(1) [],
        FILTER_5 OFFSET(5) NUMBITS(1) [],
        FILTER_6 OFFSET(6) NUMBITS(1) [],
        FILTER_7 OFFSET(7) NUMBITS(1) [],
        FILTER_8 OFFSET(8) NUMBITS(1) [],
        FILTER_9 OFFSET(9) NUMBITS(1) [],
        FILTER_10 OFFSET(10) NUMBITS(1) [],
        FILTER_11 OFFSET(11) NUMBITS(1) [],
        FILTER_12 OFFSET(12) NUMBITS(1) [],
        FILTER_13 OFFSET(13) NUMBITS(1) [],
        FILTER_14 OFFSET(14) NUMBITS(1) [],
        FILTER_15 OFFSET(15) NUMBITS(1) [],
        FILTER_16 OFFSET(16) NUMBITS(1) [],
        FILTER_17 OFFSET(17) NUMBITS(1) [],
        FILTER_18 OFFSET(18) NUMBITS(1) [],
        FILTER_19 OFFSET(19) NUMBITS(1) [],
        FILTER_20 OFFSET(20) NUMBITS(1) [],
        FILTER_21 OFFSET(21) NUMBITS(1) [],
        FILTER_22 OFFSET(22) NUMBITS(1) [],
        FILTER_23 OFFSET(23) NUMBITS(1) [],
        FILTER_24 OFFSET(24) NUMBITS(1) [],
        FILTER_25 OFFSET(25) NUMBITS(1) [],
        FILTER_26 OFFSET(26) NUMBITS(1) [],
        FILTER_27 OFFSET(27) NUMBITS(1) [],
        FILTER_28 OFFSET(28) NUMBITS(1) [],
        FILTER_29 OFFSET(29) NUMBITS(1) [],
        FILTER_30 OFFSET(30) NUMBITS(1) [],
        FILTER_31 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) ADDR_SWAP_MASK [
        MASK OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) ADDR_SWAP_DATA [
        DATA OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) PAYLOAD_SWAP_MASK [
        MASK OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) PAYLOAD_SWAP_DATA [
        DATA OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CMD_INFO [
        OPCODE_0 OFFSET(0) NUMBITS(8) [],
        ADDR_MODE_0 OFFSET(8) NUMBITS(2) [
            ADDRDISABLED = 0,
            ADDRCFG = 1,
            ADDR3B = 2,
            ADDR4B = 3,
        ],
        ADDR_SWAP_EN_0 OFFSET(10) NUMBITS(1) [],
        MBYTE_EN_0 OFFSET(11) NUMBITS(1) [],
        DUMMY_SIZE_0 OFFSET(12) NUMBITS(3) [],
        DUMMY_EN_0 OFFSET(15) NUMBITS(1) [],
        PAYLOAD_EN_0 OFFSET(16) NUMBITS(4) [],
        PAYLOAD_DIR_0 OFFSET(20) NUMBITS(1) [
            PAYLOADIN = 0,
            PAYLOADOUT = 1,
        ],
        PAYLOAD_SWAP_EN_0 OFFSET(21) NUMBITS(1) [],
        UPLOAD_0 OFFSET(24) NUMBITS(1) [],
        BUSY_0 OFFSET(25) NUMBITS(1) [],
        VALID_0 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) CMD_INFO_EN4B [
        OPCODE OFFSET(0) NUMBITS(8) [],
        VALID OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) CMD_INFO_EX4B [
        OPCODE OFFSET(0) NUMBITS(8) [],
        VALID OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) CMD_INFO_WREN [
        OPCODE OFFSET(0) NUMBITS(8) [],
        VALID OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) CMD_INFO_WRDI [
        OPCODE OFFSET(0) NUMBITS(8) [],
        VALID OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) TPM_CAP [
        REV OFFSET(0) NUMBITS(8) [],
        LOCALITY OFFSET(8) NUMBITS(1) [],
        MAX_WR_SIZE OFFSET(16) NUMBITS(3) [],
        MAX_RD_SIZE OFFSET(20) NUMBITS(3) [],
    ],
    pub(crate) TPM_CFG [
        EN OFFSET(0) NUMBITS(1) [],
        TPM_MODE OFFSET(1) NUMBITS(1) [],
        HW_REG_DIS OFFSET(2) NUMBITS(1) [],
        TPM_REG_CHK_DIS OFFSET(3) NUMBITS(1) [],
        INVALID_LOCALITY OFFSET(4) NUMBITS(1) [],
    ],
    pub(crate) TPM_STATUS [
        CMDADDR_NOTEMPTY OFFSET(0) NUMBITS(1) [],
        WRFIFO_DEPTH OFFSET(16) NUMBITS(7) [],
    ],
    pub(crate) TPM_ACCESS [
        ACCESS_0 OFFSET(0) NUMBITS(8) [],
        ACCESS_1 OFFSET(8) NUMBITS(8) [],
        ACCESS_2 OFFSET(16) NUMBITS(8) [],
        ACCESS_3 OFFSET(24) NUMBITS(8) [],
    ],
    pub(crate) TPM_STS [
        STS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) TPM_INTF_CAPABILITY [
        INTF_CAPABILITY OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) TPM_INT_ENABLE [
        INT_ENABLE OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) TPM_INT_VECTOR [
        INT_VECTOR OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) TPM_INT_STATUS [
        INT_STATUS OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) TPM_DID_VID [
        VID OFFSET(0) NUMBITS(16) [],
        DID OFFSET(16) NUMBITS(16) [],
    ],
    pub(crate) TPM_RID [
        RID OFFSET(0) NUMBITS(8) [],
    ],
    pub(crate) TPM_CMD_ADDR [
        ADDR OFFSET(0) NUMBITS(24) [],
        CMD OFFSET(24) NUMBITS(8) [],
    ],
    pub(crate) TPM_READ_FIFO [
        VALUE OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) TPM_WRITE_FIFO [
        VALUE OFFSET(0) NUMBITS(8) [],
    ],
];

// End generated register constants for spi_device
