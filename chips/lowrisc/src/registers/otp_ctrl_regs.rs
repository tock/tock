// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for otp_ctrl.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/otp_ctrl/data/otp_ctrl.hjson
use kernel::utilities::registers::ReadOnly;
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of key slots
pub const OTP_CTRL_PARAM_NUM_SRAM_KEY_REQ_SLOTS: u32 = 3;
/// Width of the OTP byte address.
pub const OTP_CTRL_PARAM_OTP_BYTE_ADDR_WIDTH: u32 = 11;
/// Number of error register entries.
pub const OTP_CTRL_PARAM_NUM_ERROR_ENTRIES: u32 = 10;
/// Number of 32bit words in the DAI.
pub const OTP_CTRL_PARAM_NUM_DAI_WORDS: u32 = 2;
/// Size of the digest fields in 32bit words.
pub const OTP_CTRL_PARAM_NUM_DIGEST_WORDS: u32 = 2;
/// Size of the TL-UL window in 32bit words. Note that the effective partition size is smaller
/// than that.
pub const OTP_CTRL_PARAM_NUM_SW_CFG_WINDOW_WORDS: u32 = 512;
/// Number of partitions
pub const OTP_CTRL_PARAM_NUM_PART: u32 = 8;
/// Offset of the VENDOR_TEST partition
pub const OTP_CTRL_PARAM_VENDOR_TEST_OFFSET: usize = 0;
/// Size of the VENDOR_TEST partition
pub const OTP_CTRL_PARAM_VENDOR_TEST_SIZE: u32 = 64;
/// Offset of SCRATCH
pub const OTP_CTRL_PARAM_SCRATCH_OFFSET: usize = 0;
/// Size of SCRATCH
pub const OTP_CTRL_PARAM_SCRATCH_SIZE: u32 = 56;
/// Offset of VENDOR_TEST_DIGEST
pub const OTP_CTRL_PARAM_VENDOR_TEST_DIGEST_OFFSET: usize = 56;
/// Size of VENDOR_TEST_DIGEST
pub const OTP_CTRL_PARAM_VENDOR_TEST_DIGEST_SIZE: u32 = 8;
/// Offset of the CREATOR_SW_CFG partition
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_OFFSET: usize = 64;
/// Size of the CREATOR_SW_CFG partition
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIZE: u32 = 800;
/// Offset of CREATOR_SW_CFG_AST_CFG
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_AST_CFG_OFFSET: usize = 64;
/// Size of CREATOR_SW_CFG_AST_CFG
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_AST_CFG_SIZE: u32 = 156;
/// Offset of CREATOR_SW_CFG_AST_INIT_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_AST_INIT_EN_OFFSET: usize = 220;
/// Size of CREATOR_SW_CFG_AST_INIT_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_AST_INIT_EN_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_ROM_EXT_SKU
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_ROM_EXT_SKU_OFFSET: usize = 224;
/// Size of CREATOR_SW_CFG_ROM_EXT_SKU
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_ROM_EXT_SKU_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_SIGVERIFY_RSA_MOD_EXP_IBEX_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIGVERIFY_RSA_MOD_EXP_IBEX_EN_OFFSET: usize = 228;
/// Size of CREATOR_SW_CFG_SIGVERIFY_RSA_MOD_EXP_IBEX_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIGVERIFY_RSA_MOD_EXP_IBEX_EN_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_SIGVERIFY_RSA_KEY_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIGVERIFY_RSA_KEY_EN_OFFSET: usize = 232;
/// Size of CREATOR_SW_CFG_SIGVERIFY_RSA_KEY_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIGVERIFY_RSA_KEY_EN_SIZE: u32 = 8;
/// Offset of CREATOR_SW_CFG_SIGVERIFY_SPX_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIGVERIFY_SPX_EN_OFFSET: usize = 240;
/// Size of CREATOR_SW_CFG_SIGVERIFY_SPX_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIGVERIFY_SPX_EN_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_SIGVERIFY_SPX_KEY_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIGVERIFY_SPX_KEY_EN_OFFSET: usize = 244;
/// Size of CREATOR_SW_CFG_SIGVERIFY_SPX_KEY_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SIGVERIFY_SPX_KEY_EN_SIZE: u32 = 8;
/// Offset of CREATOR_SW_CFG_FLASH_DATA_DEFAULT_CFG
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_FLASH_DATA_DEFAULT_CFG_OFFSET: usize = 252;
/// Size of CREATOR_SW_CFG_FLASH_DATA_DEFAULT_CFG
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_FLASH_DATA_DEFAULT_CFG_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_FLASH_INFO_BOOT_DATA_CFG
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_FLASH_INFO_BOOT_DATA_CFG_OFFSET: usize = 256;
/// Size of CREATOR_SW_CFG_FLASH_INFO_BOOT_DATA_CFG
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_FLASH_INFO_BOOT_DATA_CFG_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_FLASH_HW_INFO_CFG_OVERRIDE
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_FLASH_HW_INFO_CFG_OVERRIDE_OFFSET: usize = 260;
/// Size of CREATOR_SW_CFG_FLASH_HW_INFO_CFG_OVERRIDE
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_FLASH_HW_INFO_CFG_OVERRIDE_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_EN_OFFSET: usize = 264;
/// Size of CREATOR_SW_CFG_RNG_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_EN_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_JITTER_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_JITTER_EN_OFFSET: usize = 268;
/// Size of CREATOR_SW_CFG_JITTER_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_JITTER_EN_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RET_RAM_RESET_MASK
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RET_RAM_RESET_MASK_OFFSET: usize = 272;
/// Size of CREATOR_SW_CFG_RET_RAM_RESET_MASK
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RET_RAM_RESET_MASK_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_MANUF_STATE
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_MANUF_STATE_OFFSET: usize = 276;
/// Size of CREATOR_SW_CFG_MANUF_STATE
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_MANUF_STATE_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_ROM_EXEC_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_ROM_EXEC_EN_OFFSET: usize = 280;
/// Size of CREATOR_SW_CFG_ROM_EXEC_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_ROM_EXEC_EN_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_CPUCTRL
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_CPUCTRL_OFFSET: usize = 284;
/// Size of CREATOR_SW_CFG_CPUCTRL
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_CPUCTRL_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_MIN_SEC_VER_ROM_EXT
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_MIN_SEC_VER_ROM_EXT_OFFSET: usize = 288;
/// Size of CREATOR_SW_CFG_MIN_SEC_VER_ROM_EXT
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_MIN_SEC_VER_ROM_EXT_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_MIN_SEC_VER_BL0
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_MIN_SEC_VER_BL0_OFFSET: usize = 292;
/// Size of CREATOR_SW_CFG_MIN_SEC_VER_BL0
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_MIN_SEC_VER_BL0_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_DEFAULT_BOOT_DATA_IN_PROD_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_DEFAULT_BOOT_DATA_IN_PROD_EN_OFFSET: usize = 296;
/// Size of CREATOR_SW_CFG_DEFAULT_BOOT_DATA_IN_PROD_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_DEFAULT_BOOT_DATA_IN_PROD_EN_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RMA_SPIN_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RMA_SPIN_EN_OFFSET: usize = 300;
/// Size of CREATOR_SW_CFG_RMA_SPIN_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RMA_SPIN_EN_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RMA_SPIN_CYCLES
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RMA_SPIN_CYCLES_OFFSET: usize = 304;
/// Size of CREATOR_SW_CFG_RMA_SPIN_CYCLES
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RMA_SPIN_CYCLES_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_REPCNT_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_REPCNT_THRESHOLDS_OFFSET: usize = 308;
/// Size of CREATOR_SW_CFG_RNG_REPCNT_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_REPCNT_THRESHOLDS_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_REPCNTS_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_REPCNTS_THRESHOLDS_OFFSET: usize = 312;
/// Size of CREATOR_SW_CFG_RNG_REPCNTS_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_REPCNTS_THRESHOLDS_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_ADAPTP_HI_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_ADAPTP_HI_THRESHOLDS_OFFSET: usize = 316;
/// Size of CREATOR_SW_CFG_RNG_ADAPTP_HI_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_ADAPTP_HI_THRESHOLDS_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_ADAPTP_LO_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_ADAPTP_LO_THRESHOLDS_OFFSET: usize = 320;
/// Size of CREATOR_SW_CFG_RNG_ADAPTP_LO_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_ADAPTP_LO_THRESHOLDS_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_BUCKET_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_BUCKET_THRESHOLDS_OFFSET: usize = 324;
/// Size of CREATOR_SW_CFG_RNG_BUCKET_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_BUCKET_THRESHOLDS_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_MARKOV_HI_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_MARKOV_HI_THRESHOLDS_OFFSET: usize = 328;
/// Size of CREATOR_SW_CFG_RNG_MARKOV_HI_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_MARKOV_HI_THRESHOLDS_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_MARKOV_LO_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_MARKOV_LO_THRESHOLDS_OFFSET: usize = 332;
/// Size of CREATOR_SW_CFG_RNG_MARKOV_LO_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_MARKOV_LO_THRESHOLDS_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_EXTHT_HI_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_EXTHT_HI_THRESHOLDS_OFFSET: usize = 336;
/// Size of CREATOR_SW_CFG_RNG_EXTHT_HI_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_EXTHT_HI_THRESHOLDS_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_EXTHT_LO_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_EXTHT_LO_THRESHOLDS_OFFSET: usize = 340;
/// Size of CREATOR_SW_CFG_RNG_EXTHT_LO_THRESHOLDS
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_EXTHT_LO_THRESHOLDS_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_ALERT_THRESHOLD
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_ALERT_THRESHOLD_OFFSET: usize = 344;
/// Size of CREATOR_SW_CFG_RNG_ALERT_THRESHOLD
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_ALERT_THRESHOLD_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_RNG_HEALTH_CONFIG_DIGEST
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_HEALTH_CONFIG_DIGEST_OFFSET: usize = 348;
/// Size of CREATOR_SW_CFG_RNG_HEALTH_CONFIG_DIGEST
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_RNG_HEALTH_CONFIG_DIGEST_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_SRAM_KEY_RENEW_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SRAM_KEY_RENEW_EN_OFFSET: usize = 352;
/// Size of CREATOR_SW_CFG_SRAM_KEY_RENEW_EN
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_SRAM_KEY_RENEW_EN_SIZE: u32 = 4;
/// Offset of CREATOR_SW_CFG_DIGEST
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_DIGEST_OFFSET: usize = 856;
/// Size of CREATOR_SW_CFG_DIGEST
pub const OTP_CTRL_PARAM_CREATOR_SW_CFG_DIGEST_SIZE: u32 = 8;
/// Offset of the OWNER_SW_CFG partition
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_OFFSET: usize = 864;
/// Size of the OWNER_SW_CFG partition
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_SIZE: u32 = 800;
/// Offset of OWNER_SW_CFG_ROM_ERROR_REPORTING
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ERROR_REPORTING_OFFSET: usize = 864;
/// Size of OWNER_SW_CFG_ROM_ERROR_REPORTING
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ERROR_REPORTING_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_BOOTSTRAP_DIS
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_BOOTSTRAP_DIS_OFFSET: usize = 868;
/// Size of OWNER_SW_CFG_ROM_BOOTSTRAP_DIS
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_BOOTSTRAP_DIS_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_ALERT_CLASS_EN
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_CLASS_EN_OFFSET: usize = 872;
/// Size of OWNER_SW_CFG_ROM_ALERT_CLASS_EN
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_CLASS_EN_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_ALERT_ESCALATION
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_ESCALATION_OFFSET: usize = 876;
/// Size of OWNER_SW_CFG_ROM_ALERT_ESCALATION
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_ESCALATION_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_ALERT_CLASSIFICATION
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_CLASSIFICATION_OFFSET: usize = 880;
/// Size of OWNER_SW_CFG_ROM_ALERT_CLASSIFICATION
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_CLASSIFICATION_SIZE: u32 = 320;
/// Offset of OWNER_SW_CFG_ROM_LOCAL_ALERT_CLASSIFICATION
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_LOCAL_ALERT_CLASSIFICATION_OFFSET: usize = 1200;
/// Size of OWNER_SW_CFG_ROM_LOCAL_ALERT_CLASSIFICATION
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_LOCAL_ALERT_CLASSIFICATION_SIZE: u32 = 64;
/// Offset of OWNER_SW_CFG_ROM_ALERT_ACCUM_THRESH
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_ACCUM_THRESH_OFFSET: usize = 1264;
/// Size of OWNER_SW_CFG_ROM_ALERT_ACCUM_THRESH
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_ACCUM_THRESH_SIZE: u32 = 16;
/// Offset of OWNER_SW_CFG_ROM_ALERT_TIMEOUT_CYCLES
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_TIMEOUT_CYCLES_OFFSET: usize = 1280;
/// Size of OWNER_SW_CFG_ROM_ALERT_TIMEOUT_CYCLES
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_TIMEOUT_CYCLES_SIZE: u32 = 16;
/// Offset of OWNER_SW_CFG_ROM_ALERT_PHASE_CYCLES
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_PHASE_CYCLES_OFFSET: usize = 1296;
/// Size of OWNER_SW_CFG_ROM_ALERT_PHASE_CYCLES
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_PHASE_CYCLES_SIZE: u32 = 64;
/// Offset of OWNER_SW_CFG_ROM_ALERT_DIGEST_PROD
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_DIGEST_PROD_OFFSET: usize = 1360;
/// Size of OWNER_SW_CFG_ROM_ALERT_DIGEST_PROD
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_DIGEST_PROD_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_ALERT_DIGEST_PROD_END
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_DIGEST_PROD_END_OFFSET: usize = 1364;
/// Size of OWNER_SW_CFG_ROM_ALERT_DIGEST_PROD_END
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_DIGEST_PROD_END_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_ALERT_DIGEST_DEV
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_DIGEST_DEV_OFFSET: usize = 1368;
/// Size of OWNER_SW_CFG_ROM_ALERT_DIGEST_DEV
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_DIGEST_DEV_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_ALERT_DIGEST_RMA
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_DIGEST_RMA_OFFSET: usize = 1372;
/// Size of OWNER_SW_CFG_ROM_ALERT_DIGEST_RMA
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_ALERT_DIGEST_RMA_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_WATCHDOG_BITE_THRESHOLD_CYCLES
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_WATCHDOG_BITE_THRESHOLD_CYCLES_OFFSET: usize = 1376;
/// Size of OWNER_SW_CFG_ROM_WATCHDOG_BITE_THRESHOLD_CYCLES
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_WATCHDOG_BITE_THRESHOLD_CYCLES_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_KEYMGR_ROM_EXT_MEAS_EN
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_KEYMGR_ROM_EXT_MEAS_EN_OFFSET: usize = 1380;
/// Size of OWNER_SW_CFG_ROM_KEYMGR_ROM_EXT_MEAS_EN
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_KEYMGR_ROM_EXT_MEAS_EN_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_MANUF_STATE
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_MANUF_STATE_OFFSET: usize = 1384;
/// Size of OWNER_SW_CFG_MANUF_STATE
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_MANUF_STATE_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_ROM_RSTMGR_INFO_EN
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_RSTMGR_INFO_EN_OFFSET: usize = 1388;
/// Size of OWNER_SW_CFG_ROM_RSTMGR_INFO_EN
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_ROM_RSTMGR_INFO_EN_SIZE: u32 = 4;
/// Offset of OWNER_SW_CFG_DIGEST
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_DIGEST_OFFSET: usize = 1656;
/// Size of OWNER_SW_CFG_DIGEST
pub const OTP_CTRL_PARAM_OWNER_SW_CFG_DIGEST_SIZE: u32 = 8;
/// Offset of the HW_CFG partition
pub const OTP_CTRL_PARAM_HW_CFG_OFFSET: usize = 1664;
/// Size of the HW_CFG partition
pub const OTP_CTRL_PARAM_HW_CFG_SIZE: u32 = 80;
/// Offset of DEVICE_ID
pub const OTP_CTRL_PARAM_DEVICE_ID_OFFSET: usize = 1664;
/// Size of DEVICE_ID
pub const OTP_CTRL_PARAM_DEVICE_ID_SIZE: u32 = 32;
/// Offset of MANUF_STATE
pub const OTP_CTRL_PARAM_MANUF_STATE_OFFSET: usize = 1696;
/// Size of MANUF_STATE
pub const OTP_CTRL_PARAM_MANUF_STATE_SIZE: u32 = 32;
/// Offset of EN_SRAM_IFETCH
pub const OTP_CTRL_PARAM_EN_SRAM_IFETCH_OFFSET: usize = 1728;
/// Size of EN_SRAM_IFETCH
pub const OTP_CTRL_PARAM_EN_SRAM_IFETCH_SIZE: u32 = 1;
/// Offset of EN_CSRNG_SW_APP_READ
pub const OTP_CTRL_PARAM_EN_CSRNG_SW_APP_READ_OFFSET: usize = 1729;
/// Size of EN_CSRNG_SW_APP_READ
pub const OTP_CTRL_PARAM_EN_CSRNG_SW_APP_READ_SIZE: u32 = 1;
/// Offset of EN_ENTROPY_SRC_FW_READ
pub const OTP_CTRL_PARAM_EN_ENTROPY_SRC_FW_READ_OFFSET: usize = 1730;
/// Size of EN_ENTROPY_SRC_FW_READ
pub const OTP_CTRL_PARAM_EN_ENTROPY_SRC_FW_READ_SIZE: u32 = 1;
/// Offset of EN_ENTROPY_SRC_FW_OVER
pub const OTP_CTRL_PARAM_EN_ENTROPY_SRC_FW_OVER_OFFSET: usize = 1731;
/// Size of EN_ENTROPY_SRC_FW_OVER
pub const OTP_CTRL_PARAM_EN_ENTROPY_SRC_FW_OVER_SIZE: u32 = 1;
/// Offset of HW_CFG_DIGEST
pub const OTP_CTRL_PARAM_HW_CFG_DIGEST_OFFSET: usize = 1736;
/// Size of HW_CFG_DIGEST
pub const OTP_CTRL_PARAM_HW_CFG_DIGEST_SIZE: u32 = 8;
/// Offset of the SECRET0 partition
pub const OTP_CTRL_PARAM_SECRET0_OFFSET: usize = 1744;
/// Size of the SECRET0 partition
pub const OTP_CTRL_PARAM_SECRET0_SIZE: u32 = 40;
/// Offset of TEST_UNLOCK_TOKEN
pub const OTP_CTRL_PARAM_TEST_UNLOCK_TOKEN_OFFSET: usize = 1744;
/// Size of TEST_UNLOCK_TOKEN
pub const OTP_CTRL_PARAM_TEST_UNLOCK_TOKEN_SIZE: u32 = 16;
/// Offset of TEST_EXIT_TOKEN
pub const OTP_CTRL_PARAM_TEST_EXIT_TOKEN_OFFSET: usize = 1760;
/// Size of TEST_EXIT_TOKEN
pub const OTP_CTRL_PARAM_TEST_EXIT_TOKEN_SIZE: u32 = 16;
/// Offset of SECRET0_DIGEST
pub const OTP_CTRL_PARAM_SECRET0_DIGEST_OFFSET: usize = 1776;
/// Size of SECRET0_DIGEST
pub const OTP_CTRL_PARAM_SECRET0_DIGEST_SIZE: u32 = 8;
/// Offset of the SECRET1 partition
pub const OTP_CTRL_PARAM_SECRET1_OFFSET: usize = 1784;
/// Size of the SECRET1 partition
pub const OTP_CTRL_PARAM_SECRET1_SIZE: u32 = 88;
/// Offset of FLASH_ADDR_KEY_SEED
pub const OTP_CTRL_PARAM_FLASH_ADDR_KEY_SEED_OFFSET: usize = 1784;
/// Size of FLASH_ADDR_KEY_SEED
pub const OTP_CTRL_PARAM_FLASH_ADDR_KEY_SEED_SIZE: u32 = 32;
/// Offset of FLASH_DATA_KEY_SEED
pub const OTP_CTRL_PARAM_FLASH_DATA_KEY_SEED_OFFSET: usize = 1816;
/// Size of FLASH_DATA_KEY_SEED
pub const OTP_CTRL_PARAM_FLASH_DATA_KEY_SEED_SIZE: u32 = 32;
/// Offset of SRAM_DATA_KEY_SEED
pub const OTP_CTRL_PARAM_SRAM_DATA_KEY_SEED_OFFSET: usize = 1848;
/// Size of SRAM_DATA_KEY_SEED
pub const OTP_CTRL_PARAM_SRAM_DATA_KEY_SEED_SIZE: u32 = 16;
/// Offset of SECRET1_DIGEST
pub const OTP_CTRL_PARAM_SECRET1_DIGEST_OFFSET: usize = 1864;
/// Size of SECRET1_DIGEST
pub const OTP_CTRL_PARAM_SECRET1_DIGEST_SIZE: u32 = 8;
/// Offset of the SECRET2 partition
pub const OTP_CTRL_PARAM_SECRET2_OFFSET: usize = 1872;
/// Size of the SECRET2 partition
pub const OTP_CTRL_PARAM_SECRET2_SIZE: u32 = 88;
/// Offset of RMA_TOKEN
pub const OTP_CTRL_PARAM_RMA_TOKEN_OFFSET: usize = 1872;
/// Size of RMA_TOKEN
pub const OTP_CTRL_PARAM_RMA_TOKEN_SIZE: u32 = 16;
/// Offset of CREATOR_ROOT_KEY_SHARE0
pub const OTP_CTRL_PARAM_CREATOR_ROOT_KEY_SHARE0_OFFSET: usize = 1888;
/// Size of CREATOR_ROOT_KEY_SHARE0
pub const OTP_CTRL_PARAM_CREATOR_ROOT_KEY_SHARE0_SIZE: u32 = 32;
/// Offset of CREATOR_ROOT_KEY_SHARE1
pub const OTP_CTRL_PARAM_CREATOR_ROOT_KEY_SHARE1_OFFSET: usize = 1920;
/// Size of CREATOR_ROOT_KEY_SHARE1
pub const OTP_CTRL_PARAM_CREATOR_ROOT_KEY_SHARE1_SIZE: u32 = 32;
/// Offset of SECRET2_DIGEST
pub const OTP_CTRL_PARAM_SECRET2_DIGEST_OFFSET: usize = 1952;
/// Size of SECRET2_DIGEST
pub const OTP_CTRL_PARAM_SECRET2_DIGEST_SIZE: u32 = 8;
/// Offset of the LIFE_CYCLE partition
pub const OTP_CTRL_PARAM_LIFE_CYCLE_OFFSET: usize = 1960;
/// Size of the LIFE_CYCLE partition
pub const OTP_CTRL_PARAM_LIFE_CYCLE_SIZE: u32 = 88;
/// Offset of LC_TRANSITION_CNT
pub const OTP_CTRL_PARAM_LC_TRANSITION_CNT_OFFSET: usize = 1960;
/// Size of LC_TRANSITION_CNT
pub const OTP_CTRL_PARAM_LC_TRANSITION_CNT_SIZE: u32 = 48;
/// Offset of LC_STATE
pub const OTP_CTRL_PARAM_LC_STATE_OFFSET: usize = 2008;
/// Size of LC_STATE
pub const OTP_CTRL_PARAM_LC_STATE_SIZE: u32 = 40;
/// Number of alerts
pub const OTP_CTRL_PARAM_NUM_ALERTS: u32 = 5;
/// Register width
pub const OTP_CTRL_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub OtpCtrlRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// OTP status register.
        (0x0010 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        /// This register holds information about error conditions that occurred in the agents
        (0x0014 => pub(crate) err_code: [ReadWrite<u32, ERR_CODE::Register>; 1]),
        /// Register write enable for all direct access interface registers.
        (0x0018 => pub(crate) direct_access_regwen: ReadWrite<u32, DIRECT_ACCESS_REGWEN::Register>),
        /// Command register for direct accesses.
        (0x001c => pub(crate) direct_access_cmd: ReadWrite<u32, DIRECT_ACCESS_CMD::Register>),
        /// Address register for direct accesses.
        (0x0020 => pub(crate) direct_access_address: ReadWrite<u32, DIRECT_ACCESS_ADDRESS::Register>),
        /// Write data for direct accesses.
        (0x0024 => pub(crate) direct_access_wdata: [ReadWrite<u32, DIRECT_ACCESS_WDATA::Register>; 2]),
        /// Read data for direct accesses.
        (0x002c => pub(crate) direct_access_rdata: [ReadWrite<u32, DIRECT_ACCESS_RDATA::Register>; 2]),
        /// Register write enable for !!CHECK_TRIGGER.
        (0x0034 => pub(crate) check_trigger_regwen: ReadWrite<u32, CHECK_TRIGGER_REGWEN::Register>),
        /// Command register for direct accesses.
        (0x0038 => pub(crate) check_trigger: ReadWrite<u32, CHECK_TRIGGER::Register>),
        /// Register write enable for !!INTEGRITY_CHECK_PERIOD and !!CONSISTENCY_CHECK_PERIOD.
        (0x003c => pub(crate) check_regwen: ReadWrite<u32, CHECK_REGWEN::Register>),
        /// Timeout value for the integrity and consistency checks.
        (0x0040 => pub(crate) check_timeout: ReadWrite<u32, CHECK_TIMEOUT::Register>),
        /// This value specifies the maximum period that can be generated pseudo-randomly.
        (0x0044 => pub(crate) integrity_check_period: ReadWrite<u32, INTEGRITY_CHECK_PERIOD::Register>),
        /// This value specifies the maximum period that can be generated pseudo-randomly.
        (0x0048 => pub(crate) consistency_check_period: ReadWrite<u32, CONSISTENCY_CHECK_PERIOD::Register>),
        /// Runtime read lock for the VENDOR_TEST partition.
        (0x004c => pub(crate) vendor_test_read_lock: ReadWrite<u32, VENDOR_TEST_READ_LOCK::Register>),
        /// Runtime read lock for the CREATOR_SW_CFG partition.
        (0x0050 => pub(crate) creator_sw_cfg_read_lock: ReadWrite<u32, CREATOR_SW_CFG_READ_LOCK::Register>),
        /// Runtime read lock for the OWNER_SW_CFG partition.
        (0x0054 => pub(crate) owner_sw_cfg_read_lock: ReadWrite<u32, OWNER_SW_CFG_READ_LOCK::Register>),
        /// Integrity digest for the VENDOR_TEST partition.
        (0x0058 => pub(crate) vendor_test_digest: [ReadWrite<u32, VENDOR_TEST_DIGEST::Register>; 2]),
        /// Integrity digest for the CREATOR_SW_CFG partition.
        (0x0060 => pub(crate) creator_sw_cfg_digest: [ReadWrite<u32, CREATOR_SW_CFG_DIGEST::Register>; 2]),
        /// Integrity digest for the OWNER_SW_CFG partition.
        (0x0068 => pub(crate) owner_sw_cfg_digest: [ReadWrite<u32, OWNER_SW_CFG_DIGEST::Register>; 2]),
        /// Integrity digest for the HW_CFG partition.
        (0x0070 => pub(crate) hw_cfg_digest: [ReadWrite<u32, HW_CFG_DIGEST::Register>; 2]),
        /// Integrity digest for the SECRET0 partition.
        (0x0078 => pub(crate) secret0_digest: [ReadWrite<u32, SECRET0_DIGEST::Register>; 2]),
        /// Integrity digest for the SECRET1 partition.
        (0x0080 => pub(crate) secret1_digest: [ReadWrite<u32, SECRET1_DIGEST::Register>; 2]),
        /// Integrity digest for the SECRET2 partition.
        (0x0088 => pub(crate) secret2_digest: [ReadWrite<u32, SECRET2_DIGEST::Register>; 2]),
        (0x0090 => _reserved1),
        /// Memory area: Any read to this window directly maps to the corresponding offset in the creator
        /// and owner software
        (0x1000 => pub(crate) sw_cfg_window: [ReadOnly<u32>; 512]),
        (0x1800 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        OTP_OPERATION_DONE OFFSET(0) NUMBITS(1) [],
        OTP_ERROR OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_MACRO_ERROR OFFSET(0) NUMBITS(1) [],
        FATAL_CHECK_ERROR OFFSET(1) NUMBITS(1) [],
        FATAL_BUS_INTEG_ERROR OFFSET(2) NUMBITS(1) [],
        FATAL_PRIM_OTP_ALERT OFFSET(3) NUMBITS(1) [],
        RECOV_PRIM_OTP_ALERT OFFSET(4) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
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
    pub(crate) ERR_CODE [
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
    pub(crate) DIRECT_ACCESS_REGWEN [
        DIRECT_ACCESS_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) DIRECT_ACCESS_CMD [
        RD OFFSET(0) NUMBITS(1) [],
        WR OFFSET(1) NUMBITS(1) [],
        DIGEST OFFSET(2) NUMBITS(1) [],
    ],
    pub(crate) DIRECT_ACCESS_ADDRESS [
        DIRECT_ACCESS_ADDRESS OFFSET(0) NUMBITS(11) [],
    ],
    pub(crate) DIRECT_ACCESS_WDATA [
        DIRECT_ACCESS_WDATA_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) DIRECT_ACCESS_RDATA [
        DIRECT_ACCESS_RDATA_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CHECK_TRIGGER_REGWEN [
        CHECK_TRIGGER_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CHECK_TRIGGER [
        INTEGRITY OFFSET(0) NUMBITS(1) [],
        CONSISTENCY OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) CHECK_REGWEN [
        CHECK_REGWEN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CHECK_TIMEOUT [
        CHECK_TIMEOUT OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) INTEGRITY_CHECK_PERIOD [
        INTEGRITY_CHECK_PERIOD OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CONSISTENCY_CHECK_PERIOD [
        CONSISTENCY_CHECK_PERIOD OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) VENDOR_TEST_READ_LOCK [
        VENDOR_TEST_READ_LOCK OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CREATOR_SW_CFG_READ_LOCK [
        CREATOR_SW_CFG_READ_LOCK OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) OWNER_SW_CFG_READ_LOCK [
        OWNER_SW_CFG_READ_LOCK OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) VENDOR_TEST_DIGEST [
        VENDOR_TEST_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) CREATOR_SW_CFG_DIGEST [
        CREATOR_SW_CFG_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) OWNER_SW_CFG_DIGEST [
        OWNER_SW_CFG_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) HW_CFG_DIGEST [
        HW_CFG_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) SECRET0_DIGEST [
        SECRET0_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) SECRET1_DIGEST [
        SECRET1_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) SECRET2_DIGEST [
        SECRET2_DIGEST_0 OFFSET(0) NUMBITS(32) [],
    ],
];

// End generated register constants for otp_ctrl
