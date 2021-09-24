// Generated register struct for otp_ctrl

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
    pub Otp_CtrlRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => status: ReadOnly<u32, STATUS::Register>),
        (0x14 => err_code: ReadOnly<u32, ERR_CODE::Register>),
        (0x18 => direct_access_regwen: ReadOnly<u32, DIRECT_ACCESS_REGWEN::Register>),
        (0x1c => direct_access_cmd: WriteOnly<u32, DIRECT_ACCESS_CMD::Register>),
        (0x20 => direct_access_address: ReadWrite<u32, DIRECT_ACCESS_ADDRESS::Register>),
        (0x24 => direct_access_wdata_0: ReadWrite<u32, DIRECT_ACCESS_WDATA_0::Register>),
        (0x28 => direct_access_wdata_1: ReadWrite<u32, DIRECT_ACCESS_WDATA_1::Register>),
        (0x2c => direct_access_rdata_0: ReadOnly<u32, DIRECT_ACCESS_RDATA_0::Register>),
        (0x30 => direct_access_rdata_1: ReadOnly<u32, DIRECT_ACCESS_RDATA_1::Register>),
        (0x34 => check_trigger_regwen: ReadWrite<u32, CHECK_TRIGGER_REGWEN::Register>),
        (0x38 => check_trigger: WriteOnly<u32, CHECK_TRIGGER::Register>),
        (0x3c => check_regwen: ReadWrite<u32, CHECK_REGWEN::Register>),
        (0x40 => check_timeout: ReadWrite<u32, CHECK_TIMEOUT::Register>),
        (0x44 => integrity_check_period: ReadWrite<u32, INTEGRITY_CHECK_PERIOD::Register>),
        (0x48 => consistency_check_period: ReadWrite<u32, CONSISTENCY_CHECK_PERIOD::Register>),
        (0x4c => vendor_test_read_lock: ReadWrite<u32, VENDOR_TEST_READ_LOCK::Register>),
        (0x50 => creator_sw_cfg_read_lock: ReadWrite<u32, CREATOR_SW_CFG_READ_LOCK::Register>),
        (0x54 => owner_sw_cfg_read_lock: ReadWrite<u32, OWNER_SW_CFG_READ_LOCK::Register>),
        (0x58 => vendor_test_digest_0: ReadOnly<u32, VENDOR_TEST_DIGEST_0::Register>),
        (0x5c => vendor_test_digest_1: ReadOnly<u32, VENDOR_TEST_DIGEST_1::Register>),
        (0x60 => creator_sw_cfg_digest_0: ReadOnly<u32, CREATOR_SW_CFG_DIGEST_0::Register>),
        (0x64 => creator_sw_cfg_digest_1: ReadOnly<u32, CREATOR_SW_CFG_DIGEST_1::Register>),
        (0x68 => owner_sw_cfg_digest_0: ReadOnly<u32, OWNER_SW_CFG_DIGEST_0::Register>),
        (0x6c => owner_sw_cfg_digest_1: ReadOnly<u32, OWNER_SW_CFG_DIGEST_1::Register>),
        (0x70 => hw_cfg_digest_0: ReadOnly<u32, HW_CFG_DIGEST_0::Register>),
        (0x74 => hw_cfg_digest_1: ReadOnly<u32, HW_CFG_DIGEST_1::Register>),
        (0x78 => secret0_digest_0: ReadOnly<u32, SECRET0_DIGEST_0::Register>),
        (0x7c => secret0_digest_1: ReadOnly<u32, SECRET0_DIGEST_1::Register>),
        (0x80 => secret1_digest_0: ReadOnly<u32, SECRET1_DIGEST_0::Register>),
        (0x84 => secret1_digest_1: ReadOnly<u32, SECRET1_DIGEST_1::Register>),
        (0x88 => secret2_digest_0: ReadOnly<u32, SECRET2_DIGEST_0::Register>),
        (0x8c => secret2_digest_1: ReadOnly<u32, SECRET2_DIGEST_1::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        OTP_OPERATION_DONE OFFSET(0) NUMBITS(1) [],
        OTP_ERROR OFFSET(1) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        OTP_OPERATION_DONE OFFSET(0) NUMBITS(1) [],
        OTP_ERROR OFFSET(1) NUMBITS(1) [],
    ],
    INTR_TEST [
        OTP_OPERATION_DONE OFFSET(0) NUMBITS(1) [],
        OTP_ERROR OFFSET(1) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_MACRO_ERROR OFFSET(0) NUMBITS(1) [],
        FATAL_CHECK_ERROR OFFSET(1) NUMBITS(1) [],
        FATAL_BUS_INTEG_ERROR OFFSET(2) NUMBITS(1) [],
    ],
    STATUS [
        VENDOR_TEST_ERROR OFFSET(0) NUMBITS(1) [],
        CREATOR_SW_CFG_ERROR OFFSET(1) NUMBITS(1) [],
        OWNER_SW_CFG_ERROR OFFSET(2) NUMBITS(1) [],
        HW_CFG_ERROR OFFSET(3) NUMBITS(1) [],
        SECRET0_ERROR OFFSET(4) NUMBITS(1) [],
        SECRET1_ERROR OFFSET(5) NUMBITS(1) [],
        SECRET2_ERROR OFFSET(6) NUMBITS(1) [],
        LIFE_CYCLE_ERROR OFFSET(7) NUMBITS(1) [],
        DAI_ERROR OFFSET(8) NUMBITS(1) [],
        LCI_ERROR OFFSET(9) NUMBITS(1) [],
        TIMEOUT_ERROR OFFSET(10) NUMBITS(1) [],
        LFSR_FSM_ERROR OFFSET(11) NUMBITS(1) [],
        SCRAMBLING_FSM_ERROR OFFSET(12) NUMBITS(1) [],
        KEY_DERIV_FSM_ERROR OFFSET(13) NUMBITS(1) [],
        BUS_INTEG_ERROR OFFSET(14) NUMBITS(1) [],
        DAI_IDLE OFFSET(15) NUMBITS(1) [],
        CHECK_PENDING OFFSET(16) NUMBITS(1) [],
    ],
    ERR_CODE [
        ERR_CODE_0 OFFSET(0) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
        ERR_CODE_1 OFFSET(3) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
        ERR_CODE_2 OFFSET(6) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
        ERR_CODE_3 OFFSET(9) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
        ERR_CODE_4 OFFSET(12) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
        ERR_CODE_5 OFFSET(15) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
        ERR_CODE_6 OFFSET(18) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
        ERR_CODE_7 OFFSET(21) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
        ERR_CODE_8 OFFSET(24) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
        ERR_CODE_9 OFFSET(27) NUMBITS(3) [
            NO_ERROR = 0,
            MACRO_ERROR = 1,
            MACRO_ECC_CORR_ERROR = 2,
            MACRO_ECC_UNCORR_ERROR = 3,
            MACRO_WRITE_BLANK_ERROR = 4,
            ACCESS_ERROR = 5,
            CHECK_FAIL_ERROR = 6,
            FSM_STATE_ERROR = 7,
        ],
    ],
    DIRECT_ACCESS_REGWEN [
        DIRECT_ACCESS_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    DIRECT_ACCESS_CMD [
        RD OFFSET(0) NUMBITS(1) [],
        WR OFFSET(1) NUMBITS(1) [],
        DIGEST OFFSET(2) NUMBITS(1) [],
    ],
    DIRECT_ACCESS_ADDRESS [
        DIRECT_ACCESS_ADDRESS OFFSET(0) NUMBITS(11) [],
    ],
    DIRECT_ACCESS_WDATA_0 [
        DIRECT_ACCESS_WDATA_0 OFFSET(0) NUMBITS(32) [],
    ],
    DIRECT_ACCESS_WDATA_1 [
        DIRECT_ACCESS_WDATA_1 OFFSET(0) NUMBITS(32) [],
    ],
    DIRECT_ACCESS_RDATA_0 [
        DIRECT_ACCESS_RDATA_0 OFFSET(0) NUMBITS(32) [],
    ],
    DIRECT_ACCESS_RDATA_1 [
        DIRECT_ACCESS_RDATA_1 OFFSET(0) NUMBITS(32) [],
    ],
    CHECK_TRIGGER_REGWEN [
        CHECK_TRIGGER_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CHECK_TRIGGER [
        INTEGRITY OFFSET(0) NUMBITS(1) [],
        CONSISTENCY OFFSET(1) NUMBITS(1) [],
    ],
    CHECK_REGWEN [
        CHECK_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    CHECK_TIMEOUT [
        CHECK_TIMEOUT OFFSET(0) NUMBITS(32) [],
    ],
    INTEGRITY_CHECK_PERIOD [
        INTEGRITY_CHECK_PERIOD OFFSET(0) NUMBITS(32) [],
    ],
    CONSISTENCY_CHECK_PERIOD [
        CONSISTENCY_CHECK_PERIOD OFFSET(0) NUMBITS(32) [],
    ],
    VENDOR_TEST_READ_LOCK [
        VENDOR_TEST_READ_LOCK OFFSET(0) NUMBITS(1) [],
    ],
    CREATOR_SW_CFG_READ_LOCK [
        CREATOR_SW_CFG_READ_LOCK OFFSET(0) NUMBITS(1) [],
    ],
    OWNER_SW_CFG_READ_LOCK [
        OWNER_SW_CFG_READ_LOCK OFFSET(0) NUMBITS(1) [],
    ],
    VENDOR_TEST_DIGEST_0 [
        VENDOR_TEST_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    VENDOR_TEST_DIGEST_1 [
        VENDOR_TEST_DIGEST_1 OFFSET(0) NUMBITS(32) [],
    ],
    CREATOR_SW_CFG_DIGEST_0 [
        CREATOR_SW_CFG_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    CREATOR_SW_CFG_DIGEST_1 [
        CREATOR_SW_CFG_DIGEST_1 OFFSET(0) NUMBITS(32) [],
    ],
    OWNER_SW_CFG_DIGEST_0 [
        OWNER_SW_CFG_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    OWNER_SW_CFG_DIGEST_1 [
        OWNER_SW_CFG_DIGEST_1 OFFSET(0) NUMBITS(32) [],
    ],
    HW_CFG_DIGEST_0 [
        HW_CFG_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    HW_CFG_DIGEST_1 [
        HW_CFG_DIGEST_1 OFFSET(0) NUMBITS(32) [],
    ],
    SECRET0_DIGEST_0 [
        SECRET0_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    SECRET0_DIGEST_1 [
        SECRET0_DIGEST_1 OFFSET(0) NUMBITS(32) [],
    ],
    SECRET1_DIGEST_0 [
        SECRET1_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    SECRET1_DIGEST_1 [
        SECRET1_DIGEST_1 OFFSET(0) NUMBITS(32) [],
    ],
    SECRET2_DIGEST_0 [
        SECRET2_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    SECRET2_DIGEST_1 [
        SECRET2_DIGEST_1 OFFSET(0) NUMBITS(32) [],
    ],
];

// Number of key slots
pub const OTP_CTRL_PARAM_NUM_SRAM_KEY_REQ_SLOTS: u32 = 2;

// Width of the OTP byte address.
pub const OTP_CTRL_PARAM_OTP_BYTE_ADDR_WIDTH: u32 = 11;

// Number of error register entries.
pub const OTP_CTRL_PARAM_NUM_ERROR_ENTRIES: u32 = 10;

// Number of 32bit words in the DAI.
pub const OTP_CTRL_PARAM_NUM_DAI_WORDS: u32 = 2;

// Size of the digest fields in 32bit words.
pub const OTP_CTRL_PARAM_NUM_DIGEST_WORDS: u32 = 2;

// Size of the TL-UL window in 32bit words. Note that the effective partition
// size is smaller than that.
pub const OTP_CTRL_PARAM_NUM_SW_CFG_WINDOW_WORDS: u32 = 512;

// Size of the TL-UL window in 32bit words.
pub const OTP_CTRL_PARAM_NUM_DEBUG_WINDOW_WORDS: u32 = 16;

// Number of partitions
pub const OTP_CTRL_PARAM_NUM_PART: u32 = 8;

// Offset of the VENDOR_TEST partition
pub const OTP_CTRL_PARAM_VENDOR_TEST_OFFSET: usize = 0;

// Size of the VENDOR_TEST partition
pub const OTP_CTRL_PARAM_VENDOR_TEST_SIZE: u32 = 64;

// Offset of SCRATCH
pub const OTP_CTRL_PARAM_SCRATCH_OFFSET: usize = 0;

// Size of SCRATCH
pub const OTP_CTRL_PARAM_SCRATCH_SIZE: u32 = 56;

// Offset of VENDOR_TEST_DIGEST
pub const OTP_CTRL_PARAM_VENDOR_TEST_DIGEST_OFFSET: usize = 56;

// Size of VENDOR_TEST_DIGEST
pub const OTP_CTRL_PARAM_VENDOR_TEST_DIGEST_SIZE: u32 = 8;

// Offset of the CREATOR_SW_CFG partition
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_OFFSET: usize = 64;

// Size of the CREATOR_SW_CFG partition
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIZE: u32 = 800;

// Offset of CREATOR_SW_CFG_AST_CFG
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_AST_CFG_OFFSET: usize = 64;

// Size of CREATOR_SW_CFG_AST_CFG
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_AST_CFG_SIZE: u32 = 256;

// Offset of CREATOR_SW_CFG_ROM_EXT_SKU
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_ROM_EXT_SKU_OFFSET: usize = 320;

// Size of CREATOR_SW_CFG_ROM_EXT_SKU
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_ROM_EXT_SKU_SIZE: u32 = 4;

// Offset of CREATOR_SW_CFG_USE_SW_RSA_VERIFY
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_USE_SW_RSA_VERIFY_OFFSET: usize = 324;

// Size of CREATOR_SW_CFG_USE_SW_RSA_VERIFY
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_USE_SW_RSA_VERIFY_SIZE: u32 = 4;

// Offset of CREATOR_SW_CFG_KEY_IS_VALID
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_KEY_IS_VALID_OFFSET: usize = 328;

// Size of CREATOR_SW_CFG_KEY_IS_VALID
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_KEY_IS_VALID_SIZE: u32 = 8;

// Offset of CREATOR_SW_CFG_DIGEST
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_DIGEST_OFFSET: usize = 856;

// Size of CREATOR_SW_CFG_DIGEST
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_DIGEST_SIZE: u32 = 8;

// Offset of the OWNER_SW_CFG partition
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_OFFSET: usize = 864;

// Size of the OWNER_SW_CFG partition
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_SIZE: u32 = 800;

// Offset of ROM_ERROR_REPORTING
pub const OTP_CTRL_PARAM_ROM_ERROR_REPORTING_OFFSET: usize = 864;

// Size of ROM_ERROR_REPORTING
pub const OTP_CTRL_PARAM_ROM_ERROR_REPORTING_SIZE: u32 = 4;

// Offset of ROM_BOOTSTRAP_EN
pub const OTP_CTRL_PARAM_ROM_BOOTSTRAP_EN_OFFSET: usize = 868;

// Size of ROM_BOOTSTRAP_EN
pub const OTP_CTRL_PARAM_ROM_BOOTSTRAP_EN_SIZE: u32 = 4;

// Offset of ROM_FAULT_RESPONSE
pub const OTP_CTRL_PARAM_ROM_FAULT_RESPONSE_OFFSET: usize = 872;

// Size of ROM_FAULT_RESPONSE
pub const OTP_CTRL_PARAM_ROM_FAULT_RESPONSE_SIZE: u32 = 4;

// Offset of ROM_ALERT_CLASS_EN
pub const OTP_CTRL_PARAM_ROM_ALERT_CLASS_EN_OFFSET: usize = 876;

// Size of ROM_ALERT_CLASS_EN
pub const OTP_CTRL_PARAM_ROM_ALERT_CLASS_EN_SIZE: u32 = 4;

// Offset of ROM_ALERT_ESCALATION
pub const OTP_CTRL_PARAM_ROM_ALERT_ESCALATION_OFFSET: usize = 880;

// Size of ROM_ALERT_ESCALATION
pub const OTP_CTRL_PARAM_ROM_ALERT_ESCALATION_SIZE: u32 = 4;

// Offset of ROM_ALERT_CLASSIFICATION
pub const OTP_CTRL_PARAM_ROM_ALERT_CLASSIFICATION_OFFSET: usize = 884;

// Size of ROM_ALERT_CLASSIFICATION
pub const OTP_CTRL_PARAM_ROM_ALERT_CLASSIFICATION_SIZE: u32 = 320;

// Offset of ROM_LOCAL_ALERT_CLASSIFICATION
pub const OTP_CTRL_PARAM_ROM_LOCAL_ALERT_CLASSIFICATION_OFFSET: usize = 1204;

// Size of ROM_LOCAL_ALERT_CLASSIFICATION
pub const OTP_CTRL_PARAM_ROM_LOCAL_ALERT_CLASSIFICATION_SIZE: u32 = 64;

// Offset of ROM_ALERT_ACCUM_THRESH
pub const OTP_CTRL_PARAM_ROM_ALERT_ACCUM_THRESH_OFFSET: usize = 1268;

// Size of ROM_ALERT_ACCUM_THRESH
pub const OTP_CTRL_PARAM_ROM_ALERT_ACCUM_THRESH_SIZE: u32 = 16;

// Offset of ROM_ALERT_TIMEOUT_CYCLES
pub const OTP_CTRL_PARAM_ROM_ALERT_TIMEOUT_CYCLES_OFFSET: usize = 1284;

// Size of ROM_ALERT_TIMEOUT_CYCLES
pub const OTP_CTRL_PARAM_ROM_ALERT_TIMEOUT_CYCLES_SIZE: u32 = 16;

// Offset of ROM_ALERT_PHASE_CYCLES
pub const OTP_CTRL_PARAM_ROM_ALERT_PHASE_CYCLES_OFFSET: usize = 1300;

// Size of ROM_ALERT_PHASE_CYCLES
pub const OTP_CTRL_PARAM_ROM_ALERT_PHASE_CYCLES_SIZE: u32 = 64;

// Offset of OWNER_SW_CFG_DIGEST
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_DIGEST_OFFSET: usize = 1656;

// Size of OWNER_SW_CFG_DIGEST
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_DIGEST_SIZE: u32 = 8;

// Offset of the HW_CFG partition
pub const OTP_CTRL_PARAM_HW_CFG_OFFSET: usize = 1664;

// Size of the HW_CFG partition
pub const OTP_CTRL_PARAM_HW_CFG_SIZE: u32 = 80;

// Offset of DEVICE_ID
pub const OTP_CTRL_PARAM_DEVICE_ID_OFFSET: usize = 1664;

// Size of DEVICE_ID
pub const OTP_CTRL_PARAM_DEVICE_ID_SIZE: u32 = 32;

// Offset of MANUF_STATE
pub const OTP_CTRL_PARAM_MANUF_STATE_OFFSET: usize = 1696;

// Size of MANUF_STATE
pub const OTP_CTRL_PARAM_MANUF_STATE_SIZE: u32 = 32;

// Offset of EN_SRAM_IFETCH
pub const OTP_CTRL_PARAM_EN_SRAM_IFETCH_OFFSET: usize = 1728;

// Size of EN_SRAM_IFETCH
pub const OTP_CTRL_PARAM_EN_SRAM_IFETCH_SIZE: u32 = 1;

// Offset of EN_CSRNG_SW_APP_READ
pub const OTP_CTRL_PARAM_EN_CSRNG_SW_APP_READ_OFFSET: usize = 1729;

// Size of EN_CSRNG_SW_APP_READ
pub const OTP_CTRL_PARAM_EN_CSRNG_SW_APP_READ_SIZE: u32 = 1;

// Offset of EN_ENTROPY_SRC_FW_READ
pub const OTP_CTRL_PARAM_EN_ENTROPY_SRC_FW_READ_OFFSET: usize = 1730;

// Size of EN_ENTROPY_SRC_FW_READ
pub const OTP_CTRL_PARAM_EN_ENTROPY_SRC_FW_READ_SIZE: u32 = 1;

// Offset of EN_ENTROPY_SRC_FW_OVER
pub const OTP_CTRL_PARAM_EN_ENTROPY_SRC_FW_OVER_OFFSET: usize = 1731;

// Size of EN_ENTROPY_SRC_FW_OVER
pub const OTP_CTRL_PARAM_EN_ENTROPY_SRC_FW_OVER_SIZE: u32 = 1;

// Offset of HW_CFG_DIGEST
pub const OTP_CTRL_PARAM_HW_CFG_DIGEST_OFFSET: usize = 1736;

// Size of HW_CFG_DIGEST
pub const OTP_CTRL_PARAM_HW_CFG_DIGEST_SIZE: u32 = 8;

// Offset of the SECRET0 partition
pub const OTP_CTRL_PARAM_SECRET0_OFFSET: usize = 1744;

// Size of the SECRET0 partition
pub const OTP_CTRL_PARAM_SECRET0_SIZE: u32 = 40;

// Offset of TEST_UNLOCK_TOKEN
pub const OTP_CTRL_PARAM_TEST_UNLOCK_TOKEN_OFFSET: usize = 1744;

// Size of TEST_UNLOCK_TOKEN
pub const OTP_CTRL_PARAM_TEST_UNLOCK_TOKEN_SIZE: u32 = 16;

// Offset of TEST_EXIT_TOKEN
pub const OTP_CTRL_PARAM_TEST_EXIT_TOKEN_OFFSET: usize = 1760;

// Size of TEST_EXIT_TOKEN
pub const OTP_CTRL_PARAM_TEST_EXIT_TOKEN_SIZE: u32 = 16;

// Offset of SECRET0_DIGEST
pub const OTP_CTRL_PARAM_SECRET0_DIGEST_OFFSET: usize = 1776;

// Size of SECRET0_DIGEST
pub const OTP_CTRL_PARAM_SECRET0_DIGEST_SIZE: u32 = 8;

// Offset of the SECRET1 partition
pub const OTP_CTRL_PARAM_SECRET1_OFFSET: usize = 1784;

// Size of the SECRET1 partition
pub const OTP_CTRL_PARAM_SECRET1_SIZE: u32 = 88;

// Offset of FLASH_ADDR_KEY_SEED
pub const OTP_CTRL_PARAM_FLASH_ADDR_KEY_SEED_OFFSET: usize = 1784;

// Size of FLASH_ADDR_KEY_SEED
pub const OTP_CTRL_PARAM_FLASH_ADDR_KEY_SEED_SIZE: u32 = 32;

// Offset of FLASH_DATA_KEY_SEED
pub const OTP_CTRL_PARAM_FLASH_DATA_KEY_SEED_OFFSET: usize = 1816;

// Size of FLASH_DATA_KEY_SEED
pub const OTP_CTRL_PARAM_FLASH_DATA_KEY_SEED_SIZE: u32 = 32;

// Offset of SRAM_DATA_KEY_SEED
pub const OTP_CTRL_PARAM_SRAM_DATA_KEY_SEED_OFFSET: usize = 1848;

// Size of SRAM_DATA_KEY_SEED
pub const OTP_CTRL_PARAM_SRAM_DATA_KEY_SEED_SIZE: u32 = 16;

// Offset of SECRET1_DIGEST
pub const OTP_CTRL_PARAM_SECRET1_DIGEST_OFFSET: usize = 1864;

// Size of SECRET1_DIGEST
pub const OTP_CTRL_PARAM_SECRET1_DIGEST_SIZE: u32 = 8;

// Offset of the SECRET2 partition
pub const OTP_CTRL_PARAM_SECRET2_OFFSET: usize = 1872;

// Size of the SECRET2 partition
pub const OTP_CTRL_PARAM_SECRET2_SIZE: u32 = 88;

// Offset of RMA_TOKEN
pub const OTP_CTRL_PARAM_RMA_TOKEN_OFFSET: usize = 1872;

// Size of RMA_TOKEN
pub const OTP_CTRL_PARAM_RMA_TOKEN_SIZE: u32 = 16;

// Offset of CREATOR_ROOT_KEY_SHARE0
pub const OTP_CTRL_PARAM_CREATOR_ROOT_KEY_SHARE0_OFFSET: usize = 1888;

// Size of CREATOR_ROOT_KEY_SHARE0
pub const OTP_CTRL_PARAM_CREATOR_ROOT_KEY_SHARE0_SIZE: u32 = 32;

// Offset of CREATOR_ROOT_KEY_SHARE1
pub const OTP_CTRL_PARAM_CREATOR_ROOT_KEY_SHARE1_OFFSET: usize = 1920;

// Size of CREATOR_ROOT_KEY_SHARE1
pub const OTP_CTRL_PARAM_CREATOR_ROOT_KEY_SHARE1_SIZE: u32 = 32;

// Offset of SECRET2_DIGEST
pub const OTP_CTRL_PARAM_SECRET2_DIGEST_OFFSET: usize = 1952;

// Size of SECRET2_DIGEST
pub const OTP_CTRL_PARAM_SECRET2_DIGEST_SIZE: u32 = 8;

// Offset of the LIFE_CYCLE partition
pub const OTP_CTRL_PARAM_LIFE_CYCLE_OFFSET: usize = 1960;

// Size of the LIFE_CYCLE partition
pub const OTP_CTRL_PARAM_LIFE_CYCLE_SIZE: u32 = 88;

// Offset of LC_TRANSITION_CNT
pub const OTP_CTRL_PARAM_LC_TRANSITION_CNT_OFFSET: usize = 1960;

// Size of LC_TRANSITION_CNT
pub const OTP_CTRL_PARAM_LC_TRANSITION_CNT_SIZE: u32 = 48;

// Offset of LC_STATE
pub const OTP_CTRL_PARAM_LC_STATE_OFFSET: usize = 2008;

// Size of LC_STATE
pub const OTP_CTRL_PARAM_LC_STATE_SIZE: u32 = 40;

// Number of alerts
pub const OTP_CTRL_PARAM_NUM_ALERTS: u32 = 3;

// Register width
pub const OTP_CTRL_PARAM_REG_WIDTH: u32 = 32;

// This register holds information about error conditions that occurred in
// the agents
pub const OTP_CTRL_ERR_CODE_ERR_CODE_FIELD_WIDTH: u32 = 3;
pub const OTP_CTRL_ERR_CODE_ERR_CODE_FIELDS_PER_REG: u32 = 10;
pub const OTP_CTRL_ERR_CODE_MULTIREG_COUNT: u32 = 1;

// Write data for direct accesses.
pub const OTP_CTRL_DIRECT_ACCESS_WDATA_DIRECT_ACCESS_WDATA_FIELD_WIDTH: u32 = 32;
pub const OTP_CTRL_DIRECT_ACCESS_WDATA_DIRECT_ACCESS_WDATA_FIELDS_PER_REG: u32 = 1;
pub const OTP_CTRL_DIRECT_ACCESS_WDATA_MULTIREG_COUNT: u32 = 2;

// Read data for direct accesses.
pub const OTP_CTRL_DIRECT_ACCESS_RDATA_DIRECT_ACCESS_RDATA_FIELD_WIDTH: u32 = 32;
pub const OTP_CTRL_DIRECT_ACCESS_RDATA_DIRECT_ACCESS_RDATA_FIELDS_PER_REG: u32 = 1;
pub const OTP_CTRL_DIRECT_ACCESS_RDATA_MULTIREG_COUNT: u32 = 2;

// Integrity digest for the VENDOR_TEST partition.
pub const OTP_CTRL_VENDOR_TEST_DIGEST_VENDOR_TEST_DIGEST_FIELD_WIDTH: u32 = 32;
pub const OTP_CTRL_VENDOR_TEST_DIGEST_VENDOR_TEST_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const OTP_CTRL_VENDOR_TEST_DIGEST_MULTIREG_COUNT: u32 = 2;

// Integrity digest for the CREATOR_SW_CFG partition.
pub const OTP_CTRL_CREATOR_SW_CFG_DIGEST_CREATOR_SW_CFG_DIGEST_FIELD_WIDTH: u32 = 32;
pub const OTP_CTRL_CREATOR_SW_CFG_DIGEST_CREATOR_SW_CFG_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const OTP_CTRL_CREATOR_SW_CFG_DIGEST_MULTIREG_COUNT: u32 = 2;

// Integrity digest for the OWNER_SW_CFG partition.
pub const OTP_CTRL_OWNER_SW_CFG_DIGEST_OWNER_SW_CFG_DIGEST_FIELD_WIDTH: u32 = 32;
pub const OTP_CTRL_OWNER_SW_CFG_DIGEST_OWNER_SW_CFG_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const OTP_CTRL_OWNER_SW_CFG_DIGEST_MULTIREG_COUNT: u32 = 2;

// Integrity digest for the HW_CFG partition.
pub const OTP_CTRL_HW_CFG_DIGEST_HW_CFG_DIGEST_FIELD_WIDTH: u32 = 32;
pub const OTP_CTRL_HW_CFG_DIGEST_HW_CFG_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const OTP_CTRL_HW_CFG_DIGEST_MULTIREG_COUNT: u32 = 2;

// Integrity digest for the SECRET0 partition.
pub const OTP_CTRL_SECRET0_DIGEST_SECRET0_DIGEST_FIELD_WIDTH: u32 = 32;
pub const OTP_CTRL_SECRET0_DIGEST_SECRET0_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const OTP_CTRL_SECRET0_DIGEST_MULTIREG_COUNT: u32 = 2;

// Integrity digest for the SECRET1 partition.
pub const OTP_CTRL_SECRET1_DIGEST_SECRET1_DIGEST_FIELD_WIDTH: u32 = 32;
pub const OTP_CTRL_SECRET1_DIGEST_SECRET1_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const OTP_CTRL_SECRET1_DIGEST_MULTIREG_COUNT: u32 = 2;

// Integrity digest for the SECRET2 partition.
pub const OTP_CTRL_SECRET2_DIGEST_SECRET2_DIGEST_FIELD_WIDTH: u32 = 32;
pub const OTP_CTRL_SECRET2_DIGEST_SECRET2_DIGEST_FIELDS_PER_REG: u32 = 1;
pub const OTP_CTRL_SECRET2_DIGEST_MULTIREG_COUNT: u32 = 2;

// Memory area: Any read to this window directly maps to the corresponding
// offset in the creator and owner software
pub const OTP_CTRL_SW_CFG_WINDOW_REG_OFFSET: usize = 0x1000;
pub const OTP_CTRL_SW_CFG_WINDOW_SIZE_WORDS: u32 = 512;
pub const OTP_CTRL_SW_CFG_WINDOW_SIZE_BYTES: u32 = 2048;
// End generated register constants for otp_ctrl

