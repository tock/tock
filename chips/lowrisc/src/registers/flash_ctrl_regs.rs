// Generated register constants for FLASH_CTRL.
// This file is licensed under either of:
//   Apache License, Version 2.0 (LICENSE-APACHE <http://www.apache.org/licenses/LICENSE-2.0>)
//   MIT License (LICENSE-MIT <http://opensource.org/licenses/MIT>)

// Built for earlgrey_silver_release_v5-5654-g222658011
// https://github.com/lowRISC/opentitan/tree/222658011c27d6c1f22f02c7f589043f207ff574
// Tree status: clean
// Build date: 2022-06-02T20:40:57

// Original reference file: hw/ip/flash_ctrl/data/flash_ctrl.hjson
// Copyright information found in the reference file:
//   Copyright lowRISC contributors.
// Licensing information found in the reference file:
//   Licensed under the Apache License, Version 2.0, see LICENSE for details.
//   SPDX-License-Identifier: Apache-2.0

use kernel::utilities::registers::ReadOnly;
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::WriteOnly;
use kernel::utilities::registers::{register_bitfields, register_structs};
// Number of flash banks
pub const FLASH_CTRL_PARAM_REG_NUM_BANKS: u32 = 2;
// Number of pages per bank
pub const FLASH_CTRL_PARAM_REG_PAGES_PER_BANK: u32 = 256;
// Number of pages per bank
pub const FLASH_CTRL_PARAM_REG_BUS_PGM_RES_BYTES: u32 = 512;
// Number of bits needed to represent the pages within a bank
pub const FLASH_CTRL_PARAM_REG_PAGE_WIDTH: u32 = 8;
// Number of bits needed to represent the number of banks
pub const FLASH_CTRL_PARAM_REG_BANK_WIDTH: u32 = 1;
// Number of configurable flash regions
pub const FLASH_CTRL_PARAM_NUM_REGIONS: u32 = 8;
// Number of info partition types
pub const FLASH_CTRL_PARAM_NUM_INFO_TYPES: u32 = 3;
// Number of configurable flash info pages for info type 0
pub const FLASH_CTRL_PARAM_NUM_INFOS0: u32 = 10;
// Number of configurable flash info pages for info type 1
pub const FLASH_CTRL_PARAM_NUM_INFOS1: u32 = 1;
// Number of configurable flash info pages for info type 2
pub const FLASH_CTRL_PARAM_NUM_INFOS2: u32 = 2;
// Number of words per page
pub const FLASH_CTRL_PARAM_WORDS_PER_PAGE: u32 = 256;
// Number of bytes per word
pub const FLASH_CTRL_PARAM_BYTES_PER_WORD: u32 = 8;
// Number of bytes per page
pub const FLASH_CTRL_PARAM_BYTES_PER_PAGE: u32 = 2048;
// Number of bytes per bank
pub const FLASH_CTRL_PARAM_BYTES_PER_BANK: u32 = 524288;
// Maximum depth for read / program fifos
pub const FLASH_CTRL_PARAM_MAX_FIFO_DEPTH: u32 = 16;
// Maximum depth for read / program fifos
pub const FLASH_CTRL_PARAM_MAX_FIFO_WIDTH: u32 = 5;
// Number of alerts
pub const FLASH_CTRL_PARAM_NUM_ALERTS: u32 = 3;
// Register width
pub const FLASH_CTRL_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub FlashCtrlRegisters {
        // Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        // Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        // Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        // Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        // Disable flash functionality
        (0x0010 => pub(crate) dis: ReadWrite<u32, DIS::Register>),
        // Controls whether flash can be used for code execution fetches
        (0x0014 => pub(crate) exec: ReadWrite<u32, EXEC::Register>),
        // Controller init register
        (0x0018 => pub(crate) init: ReadWrite<u32, INIT::Register>),
        // Controls the configurability of the !!CONTROL register.
        (0x001c => pub(crate) ctrl_regwen: ReadWrite<u32, CTRL_REGWEN::Register>),
        // Control register
        (0x0020 => pub(crate) control: ReadWrite<u32, CONTROL::Register>),
        // Address for flash operation
        (0x0024 => pub(crate) addr: ReadWrite<u32, ADDR::Register>),
        // Enable different program types
        (0x0028 => pub(crate) prog_type_en: ReadWrite<u32, PROG_TYPE_EN::Register>),
        // Suspend erase
        (0x002c => pub(crate) erase_suspend: ReadWrite<u32, ERASE_SUSPEND::Register>),
        // Memory region registers configuration enable.
        (0x0030 => pub(crate) region_cfg_regwen: [ReadWrite<u32, REGION_CFG_REGWEN::Register>; 8]),
        // Memory property configuration for data partition
        (0x0050 => pub(crate) mp_region_cfg: [ReadWrite<u32, MP_REGION_CFG::Register>; 8]),
        // Memory base and size configuration for data partition
        (0x0070 => pub(crate) mp_region: [ReadWrite<u32, MP_REGION::Register>; 8]),
        // Default region properties
        (0x0090 => pub(crate) default_region: ReadWrite<u32, DEFAULT_REGION::Register>),
        // Memory region registers configuration enable.
        (0x0094 => pub(crate) bank0_info0_regwen: [ReadWrite<u32, BANK0_INFO0_REGWEN::Register>; 10]),
        //   Memory property configuration for info partition in bank0,
        (0x00bc => pub(crate) bank0_info0_page_cfg: [ReadWrite<u32, BANK0_INFO0_PAGE_CFG::Register>; 10]),
        // Memory region registers configuration enable.
        (0x00e4 => pub(crate) bank0_info1_regw: [ReadWrite<u32, BANK0_INFO1_REGW::Register>; 1]),
        //   Memory property configuration for info partition in bank0,
        (0x00e8 => pub(crate) bank0_info1_page_c: [ReadWrite<u32, BANK0_INFO1_PAGE_C::Register>; 1]),
        // Memory region registers configuration enable.
        (0x00ec => pub(crate) bank0_info2_regwen: [ReadWrite<u32, BANK0_INFO2_REGWEN::Register>; 2]),
        //   Memory property configuration for info partition in bank0,
        (0x00f4 => pub(crate) bank0_info2_page_cfg: [ReadWrite<u32, BANK0_INFO2_PAGE_CFG::Register>; 2]),
        // Memory region registers configuration enable.
        (0x00fc => pub(crate) bank1_info0_regwen: [ReadWrite<u32, BANK1_INFO0_REGWEN::Register>; 10]),
        //   Memory property configuration for info partition in bank1,
        (0x0124 => pub(crate) bank1_info0_page_cfg: [ReadWrite<u32, BANK1_INFO0_PAGE_CFG::Register>; 10]),
        // Memory region registers configuration enable.
        (0x014c => pub(crate) bank1_info1_regw: [ReadWrite<u32, BANK1_INFO1_REGW::Register>; 1]),
        //   Memory property configuration for info partition in bank1,
        (0x0150 => pub(crate) bank1_info1_page_c: [ReadWrite<u32, BANK1_INFO1_PAGE_C::Register>; 1]),
        // Memory region registers configuration enable.
        (0x0154 => pub(crate) bank1_info2_regwen: [ReadWrite<u32, BANK1_INFO2_REGWEN::Register>; 2]),
        //   Memory property configuration for info partition in bank1,
        (0x015c => pub(crate) bank1_info2_page_cfg: [ReadWrite<u32, BANK1_INFO2_PAGE_CFG::Register>; 2]),
        // Bank configuration registers configuration enable.
        (0x0164 => pub(crate) bank_cfg_regwen: ReadWrite<u32, BANK_CFG_REGWEN::Register>),
        // Memory properties bank configuration
        (0x0168 => pub(crate) mp_bank_cfg_shadow: [ReadWrite<u32, MP_BANK_CFG_SHADOW::Register>; 1]),
        // Flash Operation Status
        (0x016c => pub(crate) op_status: ReadWrite<u32, OP_STATUS::Register>),
        // Flash Controller Status
        (0x0170 => pub(crate) status: ReadWrite<u32, STATUS::Register>),
        // Flash error code register.
        (0x0174 => pub(crate) err_code: ReadWrite<u32, ERR_CODE::Register>),
        // This register tabulates standard fault status of the flash.
        (0x0178 => pub(crate) std_fault_status: ReadWrite<u32, STD_FAULT_STATUS::Register>),
        // This register tabulates customized fault status of the flash.
        (0x017c => pub(crate) fault_status: ReadWrite<u32, FAULT_STATUS::Register>),
        // Synchronous error address
        (0x0180 => pub(crate) err_addr: ReadWrite<u32, ERR_ADDR::Register>),
        // Total number of single bit ECC error count
        (0x0184 => pub(crate) ecc_single_err_c: [ReadWrite<u32, ECC_SINGLE_ERR_C::Register>; 1]),
        // Latest address of ECC single err
        (0x0188 => pub(crate) ecc_single_err_addr: [ReadWrite<u32, ECC_SINGLE_ERR_ADDR::Register>; 2]),
        // Phy alert configuration
        (0x0190 => pub(crate) phy_alert_cfg: ReadWrite<u32, PHY_ALERT_CFG::Register>),
        // Flash Phy Status
        (0x0194 => pub(crate) phy_status: ReadWrite<u32, PHY_STATUS::Register>),
        // Flash Controller Scratch
        (0x0198 => pub(crate) scratch: ReadWrite<u32, SCRATCH::Register>),
        // Programmable depth where FIFOs should generate interrupts
        (0x019c => pub(crate) fifo_lvl: ReadWrite<u32, FIFO_LVL::Register>),
        // Reset for flash controller FIFOs
        (0x01a0 => pub(crate) fifo_rst: ReadWrite<u32, FIFO_RST::Register>),
        // Current program and read fifo depth
        (0x01a4 => pub(crate) curr_fifo_lvl: ReadWrite<u32, CURR_FIFO_LVL::Register>),
        // Memory area: Flash program FIFO.
        (0x01a8 => pub(crate) prog_fifo: [WriteOnly<u32>; 1]),
        // Memory area: Flash read FIFO.
        (0x01ac => pub(crate) rd_fifo: [ReadOnly<u32>; 1]),
        (0x01b0 => @END),
    }
}

register_bitfields![u32,
    // Common Interrupt Offsets
    pub(crate) INTR [
        PROG_EMPTY OFFSET(0) NUMBITS(1) [],
        PROG_LVL OFFSET(1) NUMBITS(1) [],
        RD_FULL OFFSET(2) NUMBITS(1) [],
        RD_LVL OFFSET(3) NUMBITS(1) [],
        OP_DONE OFFSET(4) NUMBITS(1) [],
        CORR_ERR OFFSET(5) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        RECOV_ERR OFFSET(0) NUMBITS(1) [],
        FATAL_STD_ERR OFFSET(1) NUMBITS(1) [],
        FATAL_ERR OFFSET(2) NUMBITS(1) [],
    ],
    pub(crate) DIS [
        VAL OFFSET(0) NUMBITS(4) [],
    ],
    pub(crate) EXEC [],
    pub(crate) INIT [
        VAL OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CTRL_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CONTROL [
        START OFFSET(0) NUMBITS(1) [],
        OP OFFSET(4) NUMBITS(2) [
            READ = 0,
            PROG = 1,
            ERASE = 2,
        ],
        PROG_SEL OFFSET(6) NUMBITS(1) [
            NORMAL PROGRAM = 0,
            PROGRAM REPAIR = 1,
        ],
        ERASE_SEL OFFSET(7) NUMBITS(1) [
            PAGE ERASE = 0,
            BANK ERASE = 1,
        ],
        PARTITION_SEL OFFSET(8) NUMBITS(1) [],
        INFO_SEL OFFSET(9) NUMBITS(2) [],
        NUM OFFSET(16) NUMBITS(12) [],
    ],
    pub(crate) ADDR [
        START OFFSET(0) NUMBITS(20) [],
    ],
    pub(crate) PROG_TYPE_EN [
        NORMAL OFFSET(0) NUMBITS(1) [],
        REPAIR OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) ERASE_SUSPEND [
        REQ OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) REGION_CFG_REGWEN [
        REGION_0 OFFSET(0) NUMBITS(1) [
            REGION LOCKED = 0,
            REGION ENABLED = 1,
        ],
    ],
    pub(crate) MP_REGION_CFG [
        EN_0 OFFSET(0) NUMBITS(4) [],
        RD_EN_0 OFFSET(4) NUMBITS(4) [],
        PROG_EN_0 OFFSET(8) NUMBITS(4) [],
        ERASE_EN_0 OFFSET(12) NUMBITS(4) [],
        SCRAMBLE_EN_0 OFFSET(16) NUMBITS(4) [],
        ECC_EN_0 OFFSET(20) NUMBITS(4) [],
        HE_EN_0 OFFSET(24) NUMBITS(4) [],
    ],
    pub(crate) MP_REGION [
        BASE_0 OFFSET(0) NUMBITS(9) [],
        SIZE_0 OFFSET(9) NUMBITS(10) [],
    ],
    pub(crate) DEFAULT_REGION [
        RD_EN OFFSET(0) NUMBITS(4) [],
        PROG_EN OFFSET(4) NUMBITS(4) [],
        ERASE_EN OFFSET(8) NUMBITS(4) [],
        SCRAMBLE_EN OFFSET(12) NUMBITS(4) [],
        ECC_EN OFFSET(16) NUMBITS(4) [],
        HE_EN OFFSET(20) NUMBITS(4) [],
    ],
    pub(crate) BANK0_INFO0_REGWEN [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE LOCKED = 0,
            PAGE ENABLED = 1,
        ],
    ],
    pub(crate) BANK0_INFO0_PAGE_CFG [
        EN_0 OFFSET(0) NUMBITS(4) [],
        RD_EN_0 OFFSET(4) NUMBITS(4) [],
        PROG_EN_0 OFFSET(8) NUMBITS(4) [],
        ERASE_EN_0 OFFSET(12) NUMBITS(4) [],
        SCRAMBLE_EN_0 OFFSET(16) NUMBITS(4) [],
        ECC_EN_0 OFFSET(20) NUMBITS(4) [],
        HE_EN_0 OFFSET(24) NUMBITS(4) [],
    ],
    pub(crate) BANK0_INFO1_REGW [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE LOCKED = 0,
            PAGE ENABLED = 1,
        ],
    ],
    pub(crate) BANK0_INFO1_PAGE_C [
        EN_0 OFFSET(0) NUMBITS(4) [],
        RD_EN_0 OFFSET(4) NUMBITS(4) [],
        PROG_EN_0 OFFSET(8) NUMBITS(4) [],
        ERASE_EN_0 OFFSET(12) NUMBITS(4) [],
        SCRAMBLE_EN_0 OFFSET(16) NUMBITS(4) [],
        ECC_EN_0 OFFSET(20) NUMBITS(4) [],
        HE_EN_0 OFFSET(24) NUMBITS(4) [],
    ],
    pub(crate) BANK0_INFO2_REGWEN [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE LOCKED = 0,
            PAGE ENABLED = 1,
        ],
    ],
    pub(crate) BANK0_INFO2_PAGE_CFG [
        EN_0 OFFSET(0) NUMBITS(4) [],
        RD_EN_0 OFFSET(4) NUMBITS(4) [],
        PROG_EN_0 OFFSET(8) NUMBITS(4) [],
        ERASE_EN_0 OFFSET(12) NUMBITS(4) [],
        SCRAMBLE_EN_0 OFFSET(16) NUMBITS(4) [],
        ECC_EN_0 OFFSET(20) NUMBITS(4) [],
        HE_EN_0 OFFSET(24) NUMBITS(4) [],
    ],
    pub(crate) BANK1_INFO0_REGWEN [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE LOCKED = 0,
            PAGE ENABLED = 1,
        ],
    ],
    pub(crate) BANK1_INFO0_PAGE_CFG [
        EN_0 OFFSET(0) NUMBITS(4) [],
        RD_EN_0 OFFSET(4) NUMBITS(4) [],
        PROG_EN_0 OFFSET(8) NUMBITS(4) [],
        ERASE_EN_0 OFFSET(12) NUMBITS(4) [],
        SCRAMBLE_EN_0 OFFSET(16) NUMBITS(4) [],
        ECC_EN_0 OFFSET(20) NUMBITS(4) [],
        HE_EN_0 OFFSET(24) NUMBITS(4) [],
    ],
    pub(crate) BANK1_INFO1_REGW [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE LOCKED = 0,
            PAGE ENABLED = 1,
        ],
    ],
    pub(crate) BANK1_INFO1_PAGE_C [
        EN_0 OFFSET(0) NUMBITS(4) [],
        RD_EN_0 OFFSET(4) NUMBITS(4) [],
        PROG_EN_0 OFFSET(8) NUMBITS(4) [],
        ERASE_EN_0 OFFSET(12) NUMBITS(4) [],
        SCRAMBLE_EN_0 OFFSET(16) NUMBITS(4) [],
        ECC_EN_0 OFFSET(20) NUMBITS(4) [],
        HE_EN_0 OFFSET(24) NUMBITS(4) [],
    ],
    pub(crate) BANK1_INFO2_REGWEN [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE LOCKED = 0,
            PAGE ENABLED = 1,
        ],
    ],
    pub(crate) BANK1_INFO2_PAGE_CFG [
        EN_0 OFFSET(0) NUMBITS(4) [],
        RD_EN_0 OFFSET(4) NUMBITS(4) [],
        PROG_EN_0 OFFSET(8) NUMBITS(4) [],
        ERASE_EN_0 OFFSET(12) NUMBITS(4) [],
        SCRAMBLE_EN_0 OFFSET(16) NUMBITS(4) [],
        ECC_EN_0 OFFSET(20) NUMBITS(4) [],
        HE_EN_0 OFFSET(24) NUMBITS(4) [],
    ],
    pub(crate) BANK_CFG_REGWEN [
        BANK OFFSET(0) NUMBITS(1) [
            BANK LOCKED = 0,
            BANK ENABLED = 1,
        ],
    ],
    pub(crate) MP_BANK_CFG_SHADOW [
        ERASE_EN_0 OFFSET(0) NUMBITS(1) [],
        ERASE_EN_1 OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) OP_STATUS [
        DONE OFFSET(0) NUMBITS(1) [],
        ERR OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) STATUS [
        RD_FULL OFFSET(0) NUMBITS(1) [],
        RD_EMPTY OFFSET(1) NUMBITS(1) [],
        PROG_FULL OFFSET(2) NUMBITS(1) [],
        PROG_EMPTY OFFSET(3) NUMBITS(1) [],
        INIT_WIP OFFSET(4) NUMBITS(1) [],
    ],
    pub(crate) ERR_CODE [
        OP_ERR OFFSET(0) NUMBITS(1) [],
        MP_ERR OFFSET(1) NUMBITS(1) [],
        RD_ERR OFFSET(2) NUMBITS(1) [],
        PROG_ERR OFFSET(3) NUMBITS(1) [],
        PROG_WIN_ERR OFFSET(4) NUMBITS(1) [],
        PROG_TYPE_ERR OFFSET(5) NUMBITS(1) [],
        FLASH_MACRO_ERR OFFSET(6) NUMBITS(1) [],
        UPDATE_ERR OFFSET(7) NUMBITS(1) [],
    ],
    pub(crate) STD_FAULT_STATUS [
        REG_INTG_ERR OFFSET(0) NUMBITS(1) [],
        PROG_INTG_ERR OFFSET(1) NUMBITS(1) [],
        LCMGR_ERR OFFSET(2) NUMBITS(1) [],
        LCMGR_INTG_ERR OFFSET(3) NUMBITS(1) [],
        ARB_FSM_ERR OFFSET(4) NUMBITS(1) [],
        STORAGE_ERR OFFSET(5) NUMBITS(1) [],
        PHY_FSM_ERR OFFSET(6) NUMBITS(1) [],
        CTRL_CNT_ERR OFFSET(7) NUMBITS(1) [],
        FIFO_ERR OFFSET(8) NUMBITS(1) [],
    ],
    pub(crate) FAULT_STATUS [
        OP_ERR OFFSET(0) NUMBITS(1) [],
        MP_ERR OFFSET(1) NUMBITS(1) [],
        RD_ERR OFFSET(2) NUMBITS(1) [],
        PROG_ERR OFFSET(3) NUMBITS(1) [],
        PROG_WIN_ERR OFFSET(4) NUMBITS(1) [],
        PROG_TYPE_ERR OFFSET(5) NUMBITS(1) [],
        FLASH_MACRO_ERR OFFSET(6) NUMBITS(1) [],
        SEED_ERR OFFSET(7) NUMBITS(1) [],
        PHY_RELBL_ERR OFFSET(8) NUMBITS(1) [],
        PHY_STORAGE_ERR OFFSET(9) NUMBITS(1) [],
        SPURIOUS_ACK OFFSET(10) NUMBITS(1) [],
        ARB_ERR OFFSET(11) NUMBITS(1) [],
        HOST_GNT_ERR OFFSET(12) NUMBITS(1) [],
    ],
    pub(crate) ERR_ADDR [
        ERR_ADDR OFFSET(0) NUMBITS(20) [],
    ],
    pub(crate) ECC_SINGLE_ERR_C [
        ECC_SINGLE_ERR_CNT_0 OFFSET(0) NUMBITS(8) [],
        ECC_SINGLE_ERR_CNT_1 OFFSET(8) NUMBITS(8) [],
    ],
    pub(crate) ECC_SINGLE_ERR_ADDR [
        ECC_SINGLE_ERR_ADDR_0 OFFSET(0) NUMBITS(20) [],
    ],
    pub(crate) PHY_ALERT_CFG [
        ALERT_ACK OFFSET(0) NUMBITS(1) [],
        ALERT_TRIG OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) PHY_STATUS [
        INIT_WIP OFFSET(0) NUMBITS(1) [],
        PROG_NORMAL_AVAIL OFFSET(1) NUMBITS(1) [],
        PROG_REPAIR_AVAIL OFFSET(2) NUMBITS(1) [],
    ],
    pub(crate) SCRATCH [],
    pub(crate) FIFO_LVL [
        PROG OFFSET(0) NUMBITS(5) [],
        RD OFFSET(8) NUMBITS(5) [],
    ],
    pub(crate) FIFO_RST [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) CURR_FIFO_LVL [
        PROG OFFSET(0) NUMBITS(5) [],
        RD OFFSET(8) NUMBITS(5) [],
    ],
];

// End generated register constants for FLASH_CTRL
