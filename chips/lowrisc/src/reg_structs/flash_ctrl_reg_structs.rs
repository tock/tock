// Generated register struct for FLASH_CTRL

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
    pub Flash_CtrlRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => flash_disable: ReadWrite<u32, FLASH_DISABLE::Register>),
        (0x14 => init: ReadWrite<u32, INIT::Register>),
        (0x18 => ctrl_regwen: ReadOnly<u32, CTRL_REGWEN::Register>),
        (0x1c => control: ReadWrite<u32, CONTROL::Register>),
        (0x20 => addr: ReadWrite<u32, ADDR::Register>),
        (0x24 => prog_type_en: ReadWrite<u32, PROG_TYPE_EN::Register>),
        (0x28 => erase_suspend: ReadWrite<u32, ERASE_SUSPEND::Register>),
        (0x2c => region_cfg_regwen_0: ReadWrite<u32, REGION_CFG_REGWEN_0::Register>),
        (0x30 => region_cfg_regwen_1: ReadWrite<u32, REGION_CFG_REGWEN_1::Register>),
        (0x34 => region_cfg_regwen_2: ReadWrite<u32, REGION_CFG_REGWEN_2::Register>),
        (0x38 => region_cfg_regwen_3: ReadWrite<u32, REGION_CFG_REGWEN_3::Register>),
        (0x3c => region_cfg_regwen_4: ReadWrite<u32, REGION_CFG_REGWEN_4::Register>),
        (0x40 => region_cfg_regwen_5: ReadWrite<u32, REGION_CFG_REGWEN_5::Register>),
        (0x44 => region_cfg_regwen_6: ReadWrite<u32, REGION_CFG_REGWEN_6::Register>),
        (0x48 => region_cfg_regwen_7: ReadWrite<u32, REGION_CFG_REGWEN_7::Register>),
        (0x4c => mp_region_cfg_0: ReadWrite<u32, MP_REGION_CFG_0::Register>),
        (0x50 => mp_region_cfg_1: ReadWrite<u32, MP_REGION_CFG_1::Register>),
        (0x54 => mp_region_cfg_2: ReadWrite<u32, MP_REGION_CFG_2::Register>),
        (0x58 => mp_region_cfg_3: ReadWrite<u32, MP_REGION_CFG_3::Register>),
        (0x5c => mp_region_cfg_4: ReadWrite<u32, MP_REGION_CFG_4::Register>),
        (0x60 => mp_region_cfg_5: ReadWrite<u32, MP_REGION_CFG_5::Register>),
        (0x64 => mp_region_cfg_6: ReadWrite<u32, MP_REGION_CFG_6::Register>),
        (0x68 => mp_region_cfg_7: ReadWrite<u32, MP_REGION_CFG_7::Register>),
        (0x6c => default_region: ReadWrite<u32, DEFAULT_REGION::Register>),
        (0x70 => bank0_info0_regwen_0: ReadWrite<u32, BANK0_INFO0_REGWEN_0::Register>),
        (0x74 => bank0_info0_regwen_1: ReadWrite<u32, BANK0_INFO0_REGWEN_1::Register>),
        (0x78 => bank0_info0_regwen_2: ReadWrite<u32, BANK0_INFO0_REGWEN_2::Register>),
        (0x7c => bank0_info0_regwen_3: ReadWrite<u32, BANK0_INFO0_REGWEN_3::Register>),
        (0x80 => bank0_info0_regwen_4: ReadWrite<u32, BANK0_INFO0_REGWEN_4::Register>),
        (0x84 => bank0_info0_regwen_5: ReadWrite<u32, BANK0_INFO0_REGWEN_5::Register>),
        (0x88 => bank0_info0_regwen_6: ReadWrite<u32, BANK0_INFO0_REGWEN_6::Register>),
        (0x8c => bank0_info0_regwen_7: ReadWrite<u32, BANK0_INFO0_REGWEN_7::Register>),
        (0x90 => bank0_info0_regwen_8: ReadWrite<u32, BANK0_INFO0_REGWEN_8::Register>),
        (0x94 => bank0_info0_regwen_9: ReadWrite<u32, BANK0_INFO0_REGWEN_9::Register>),
        (0x98 => bank0_info0_page_cfg_0: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_0::Register>),
        (0x9c => bank0_info0_page_cfg_1: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_1::Register>),
        (0xa0 => bank0_info0_page_cfg_2: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_2::Register>),
        (0xa4 => bank0_info0_page_cfg_3: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_3::Register>),
        (0xa8 => bank0_info0_page_cfg_4: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_4::Register>),
        (0xac => bank0_info0_page_cfg_5: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_5::Register>),
        (0xb0 => bank0_info0_page_cfg_6: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_6::Register>),
        (0xb4 => bank0_info0_page_cfg_7: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_7::Register>),
        (0xb8 => bank0_info0_page_cfg_8: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_8::Register>),
        (0xbc => bank0_info0_page_cfg_9: ReadWrite<u32, BANK0_INFO0_PAGE_CFG_9::Register>),
        (0xc0 => bank0_info1_regwen: ReadWrite<u32, BANK0_INFO1_REGWEN::Register>),
        (0xc4 => bank0_info1_page_cfg: ReadWrite<u32, BANK0_INFO1_PAGE_CFG::Register>),
        (0xc8 => bank0_info2_regwen_0: ReadWrite<u32, BANK0_INFO2_REGWEN_0::Register>),
        (0xcc => bank0_info2_regwen_1: ReadWrite<u32, BANK0_INFO2_REGWEN_1::Register>),
        (0xd0 => bank0_info2_page_cfg_0: ReadWrite<u32, BANK0_INFO2_PAGE_CFG_0::Register>),
        (0xd4 => bank0_info2_page_cfg_1: ReadWrite<u32, BANK0_INFO2_PAGE_CFG_1::Register>),
        (0xd8 => bank1_info0_regwen_0: ReadWrite<u32, BANK1_INFO0_REGWEN_0::Register>),
        (0xdc => bank1_info0_regwen_1: ReadWrite<u32, BANK1_INFO0_REGWEN_1::Register>),
        (0xe0 => bank1_info0_regwen_2: ReadWrite<u32, BANK1_INFO0_REGWEN_2::Register>),
        (0xe4 => bank1_info0_regwen_3: ReadWrite<u32, BANK1_INFO0_REGWEN_3::Register>),
        (0xe8 => bank1_info0_regwen_4: ReadWrite<u32, BANK1_INFO0_REGWEN_4::Register>),
        (0xec => bank1_info0_regwen_5: ReadWrite<u32, BANK1_INFO0_REGWEN_5::Register>),
        (0xf0 => bank1_info0_regwen_6: ReadWrite<u32, BANK1_INFO0_REGWEN_6::Register>),
        (0xf4 => bank1_info0_regwen_7: ReadWrite<u32, BANK1_INFO0_REGWEN_7::Register>),
        (0xf8 => bank1_info0_regwen_8: ReadWrite<u32, BANK1_INFO0_REGWEN_8::Register>),
        (0xfc => bank1_info0_regwen_9: ReadWrite<u32, BANK1_INFO0_REGWEN_9::Register>),
        (0x100 => bank1_info0_page_cfg_0: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_0::Register>),
        (0x104 => bank1_info0_page_cfg_1: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_1::Register>),
        (0x108 => bank1_info0_page_cfg_2: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_2::Register>),
        (0x10c => bank1_info0_page_cfg_3: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_3::Register>),
        (0x110 => bank1_info0_page_cfg_4: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_4::Register>),
        (0x114 => bank1_info0_page_cfg_5: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_5::Register>),
        (0x118 => bank1_info0_page_cfg_6: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_6::Register>),
        (0x11c => bank1_info0_page_cfg_7: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_7::Register>),
        (0x120 => bank1_info0_page_cfg_8: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_8::Register>),
        (0x124 => bank1_info0_page_cfg_9: ReadWrite<u32, BANK1_INFO0_PAGE_CFG_9::Register>),
        (0x128 => bank1_info1_regwen: ReadWrite<u32, BANK1_INFO1_REGWEN::Register>),
        (0x12c => bank1_info1_page_cfg: ReadWrite<u32, BANK1_INFO1_PAGE_CFG::Register>),
        (0x130 => bank1_info2_regwen_0: ReadWrite<u32, BANK1_INFO2_REGWEN_0::Register>),
        (0x134 => bank1_info2_regwen_1: ReadWrite<u32, BANK1_INFO2_REGWEN_1::Register>),
        (0x138 => bank1_info2_page_cfg_0: ReadWrite<u32, BANK1_INFO2_PAGE_CFG_0::Register>),
        (0x13c => bank1_info2_page_cfg_1: ReadWrite<u32, BANK1_INFO2_PAGE_CFG_1::Register>),
        (0x140 => bank_cfg_regwen: ReadWrite<u32, BANK_CFG_REGWEN::Register>),
        (0x144 => mp_bank_cfg: ReadWrite<u32, MP_BANK_CFG::Register>),
        (0x148 => op_status: ReadWrite<u32, OP_STATUS::Register>),
        (0x14c => status: ReadOnly<u32, STATUS::Register>),
        (0x150 => err_code_intr_en: ReadWrite<u32, ERR_CODE_INTR_EN::Register>),
        (0x154 => err_code: ReadWrite<u32, ERR_CODE::Register>),
        (0x158 => err_addr: ReadOnly<u32, ERR_ADDR::Register>),
        (0x15c => ecc_single_err_cnt: ReadWrite<u32, ECC_SINGLE_ERR_CNT::Register>),
        (0x160 => ecc_single_err_addr_0: ReadOnly<u32, ECC_SINGLE_ERR_ADDR_0::Register>),
        (0x164 => ecc_single_err_addr_1: ReadOnly<u32, ECC_SINGLE_ERR_ADDR_1::Register>),
        (0x168 => ecc_multi_err_cnt: ReadWrite<u32, ECC_MULTI_ERR_CNT::Register>),
        (0x16c => ecc_multi_err_addr_0: ReadOnly<u32, ECC_MULTI_ERR_ADDR_0::Register>),
        (0x170 => ecc_multi_err_addr_1: ReadOnly<u32, ECC_MULTI_ERR_ADDR_1::Register>),
        (0x174 => phy_err_cfg_regwen: ReadWrite<u32, PHY_ERR_CFG_REGWEN::Register>),
        (0x178 => phy_err_cfg: ReadWrite<u32, PHY_ERR_CFG::Register>),
        (0x17c => phy_alert_cfg: ReadWrite<u32, PHY_ALERT_CFG::Register>),
        (0x180 => phy_status: ReadOnly<u32, PHY_STATUS::Register>),
        (0x184 => scratch: ReadWrite<u32, Scratch::Register>),
        (0x188 => fifo_lvl: ReadWrite<u32, FIFO_LVL::Register>),
        (0x18c => fifo_rst: ReadWrite<u32, FIFO_RST::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        PROG_EMPTY OFFSET(0) NUMBITS(1) [],
        PROG_LVL OFFSET(1) NUMBITS(1) [],
        RD_FULL OFFSET(2) NUMBITS(1) [],
        RD_LVL OFFSET(3) NUMBITS(1) [],
        OP_DONE OFFSET(4) NUMBITS(1) [],
        ERR OFFSET(5) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        PROG_EMPTY OFFSET(0) NUMBITS(1) [],
        PROG_LVL OFFSET(1) NUMBITS(1) [],
        RD_FULL OFFSET(2) NUMBITS(1) [],
        RD_LVL OFFSET(3) NUMBITS(1) [],
        OP_DONE OFFSET(4) NUMBITS(1) [],
        ERR OFFSET(5) NUMBITS(1) [],
    ],
    INTR_TEST [
        PROG_EMPTY OFFSET(0) NUMBITS(1) [],
        PROG_LVL OFFSET(1) NUMBITS(1) [],
        RD_FULL OFFSET(2) NUMBITS(1) [],
        RD_LVL OFFSET(3) NUMBITS(1) [],
        OP_DONE OFFSET(4) NUMBITS(1) [],
        ERR OFFSET(5) NUMBITS(1) [],
    ],
    ALERT_TEST [
        RECOV_ERR OFFSET(0) NUMBITS(1) [],
        RECOV_MP_ERR OFFSET(1) NUMBITS(1) [],
        RECOV_ECC_ERR OFFSET(2) NUMBITS(1) [],
        FATAL_INTG_ERR OFFSET(3) NUMBITS(1) [],
    ],
    FLASH_DISABLE [
        VAL OFFSET(0) NUMBITS(1) [],
    ],
    INIT [
        VAL OFFSET(0) NUMBITS(1) [],
    ],
    CTRL_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    CONTROL [
        START OFFSET(0) NUMBITS(1) [],
        OP OFFSET(4) NUMBITS(2) [
            READ = 0,
            PROG = 1,
            ERASE = 2,
        ],
        PROG_SEL OFFSET(6) NUMBITS(1) [
            NORMAL_PROGRAM = 0,
            PROGRAM_REPAIR = 1,
        ],
        ERASE_SEL OFFSET(7) NUMBITS(1) [
            PAGE_ERASE = 0,
            BANK_ERASE = 1,
        ],
        PARTITION_SEL OFFSET(8) NUMBITS(1) [],
        INFO_SEL OFFSET(9) NUMBITS(2) [],
        NUM OFFSET(16) NUMBITS(12) [],
    ],
    ADDR [
        START OFFSET(0) NUMBITS(32) [],
    ],
    PROG_TYPE_EN [
        NORMAL OFFSET(0) NUMBITS(1) [],
        REPAIR OFFSET(1) NUMBITS(1) [],
    ],
    ERASE_SUSPEND [
        REQ OFFSET(0) NUMBITS(1) [],
    ],
    REGION_CFG_REGWEN_0 [
        REGION_0 OFFSET(0) NUMBITS(1) [
            REGION_LOCKED = 0,
            REGION_ENABLED = 1,
        ],
    ],
    REGION_CFG_REGWEN_1 [
        REGION_1 OFFSET(0) NUMBITS(1) [],
    ],
    REGION_CFG_REGWEN_2 [
        REGION_2 OFFSET(0) NUMBITS(1) [],
    ],
    REGION_CFG_REGWEN_3 [
        REGION_3 OFFSET(0) NUMBITS(1) [],
    ],
    REGION_CFG_REGWEN_4 [
        REGION_4 OFFSET(0) NUMBITS(1) [],
    ],
    REGION_CFG_REGWEN_5 [
        REGION_5 OFFSET(0) NUMBITS(1) [],
    ],
    REGION_CFG_REGWEN_6 [
        REGION_6 OFFSET(0) NUMBITS(1) [],
    ],
    REGION_CFG_REGWEN_7 [
        REGION_7 OFFSET(0) NUMBITS(1) [],
    ],
    MP_REGION_CFG_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
        RD_EN_0 OFFSET(1) NUMBITS(1) [],
        PROG_EN_0 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_0 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_0 OFFSET(4) NUMBITS(1) [],
        ECC_EN_0 OFFSET(5) NUMBITS(1) [],
        HE_EN_0 OFFSET(6) NUMBITS(1) [],
        BASE_0 OFFSET(8) NUMBITS(9) [],
        SIZE_0 OFFSET(17) NUMBITS(10) [],
    ],
    MP_REGION_CFG_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
        RD_EN_1 OFFSET(1) NUMBITS(1) [],
        PROG_EN_1 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_1 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_1 OFFSET(4) NUMBITS(1) [],
        ECC_EN_1 OFFSET(5) NUMBITS(1) [],
        HE_EN_1 OFFSET(6) NUMBITS(1) [],
        BASE_1 OFFSET(8) NUMBITS(9) [],
        SIZE_1 OFFSET(17) NUMBITS(10) [],
    ],
    MP_REGION_CFG_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
        RD_EN_2 OFFSET(1) NUMBITS(1) [],
        PROG_EN_2 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_2 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_2 OFFSET(4) NUMBITS(1) [],
        ECC_EN_2 OFFSET(5) NUMBITS(1) [],
        HE_EN_2 OFFSET(6) NUMBITS(1) [],
        BASE_2 OFFSET(8) NUMBITS(9) [],
        SIZE_2 OFFSET(17) NUMBITS(10) [],
    ],
    MP_REGION_CFG_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
        RD_EN_3 OFFSET(1) NUMBITS(1) [],
        PROG_EN_3 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_3 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_3 OFFSET(4) NUMBITS(1) [],
        ECC_EN_3 OFFSET(5) NUMBITS(1) [],
        HE_EN_3 OFFSET(6) NUMBITS(1) [],
        BASE_3 OFFSET(8) NUMBITS(9) [],
        SIZE_3 OFFSET(17) NUMBITS(10) [],
    ],
    MP_REGION_CFG_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
        RD_EN_4 OFFSET(1) NUMBITS(1) [],
        PROG_EN_4 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_4 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_4 OFFSET(4) NUMBITS(1) [],
        ECC_EN_4 OFFSET(5) NUMBITS(1) [],
        HE_EN_4 OFFSET(6) NUMBITS(1) [],
        BASE_4 OFFSET(8) NUMBITS(9) [],
        SIZE_4 OFFSET(17) NUMBITS(10) [],
    ],
    MP_REGION_CFG_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
        RD_EN_5 OFFSET(1) NUMBITS(1) [],
        PROG_EN_5 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_5 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_5 OFFSET(4) NUMBITS(1) [],
        ECC_EN_5 OFFSET(5) NUMBITS(1) [],
        HE_EN_5 OFFSET(6) NUMBITS(1) [],
        BASE_5 OFFSET(8) NUMBITS(9) [],
        SIZE_5 OFFSET(17) NUMBITS(10) [],
    ],
    MP_REGION_CFG_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
        RD_EN_6 OFFSET(1) NUMBITS(1) [],
        PROG_EN_6 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_6 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_6 OFFSET(4) NUMBITS(1) [],
        ECC_EN_6 OFFSET(5) NUMBITS(1) [],
        HE_EN_6 OFFSET(6) NUMBITS(1) [],
        BASE_6 OFFSET(8) NUMBITS(9) [],
        SIZE_6 OFFSET(17) NUMBITS(10) [],
    ],
    MP_REGION_CFG_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
        RD_EN_7 OFFSET(1) NUMBITS(1) [],
        PROG_EN_7 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_7 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_7 OFFSET(4) NUMBITS(1) [],
        ECC_EN_7 OFFSET(5) NUMBITS(1) [],
        HE_EN_7 OFFSET(6) NUMBITS(1) [],
        BASE_7 OFFSET(8) NUMBITS(9) [],
        SIZE_7 OFFSET(17) NUMBITS(10) [],
    ],
    DEFAULT_REGION [
        RD_EN OFFSET(0) NUMBITS(1) [],
        PROG_EN OFFSET(1) NUMBITS(1) [],
        ERASE_EN OFFSET(2) NUMBITS(1) [],
        SCRAMBLE_EN OFFSET(3) NUMBITS(1) [],
        ECC_EN OFFSET(4) NUMBITS(1) [],
        HE_EN OFFSET(5) NUMBITS(1) [],
    ],
    BANK0_INFO0_REGWEN_0 [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE_LOCKED = 0,
            PAGE_ENABLED = 1,
        ],
    ],
    BANK0_INFO0_REGWEN_1 [
        REGION_1 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO0_REGWEN_2 [
        REGION_2 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO0_REGWEN_3 [
        REGION_3 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO0_REGWEN_4 [
        REGION_4 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO0_REGWEN_5 [
        REGION_5 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO0_REGWEN_6 [
        REGION_6 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO0_REGWEN_7 [
        REGION_7 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO0_REGWEN_8 [
        REGION_8 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO0_REGWEN_9 [
        REGION_9 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
        RD_EN_0 OFFSET(1) NUMBITS(1) [],
        PROG_EN_0 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_0 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_0 OFFSET(4) NUMBITS(1) [],
        ECC_EN_0 OFFSET(5) NUMBITS(1) [],
        HE_EN_0 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
        RD_EN_1 OFFSET(1) NUMBITS(1) [],
        PROG_EN_1 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_1 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_1 OFFSET(4) NUMBITS(1) [],
        ECC_EN_1 OFFSET(5) NUMBITS(1) [],
        HE_EN_1 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
        RD_EN_2 OFFSET(1) NUMBITS(1) [],
        PROG_EN_2 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_2 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_2 OFFSET(4) NUMBITS(1) [],
        ECC_EN_2 OFFSET(5) NUMBITS(1) [],
        HE_EN_2 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
        RD_EN_3 OFFSET(1) NUMBITS(1) [],
        PROG_EN_3 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_3 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_3 OFFSET(4) NUMBITS(1) [],
        ECC_EN_3 OFFSET(5) NUMBITS(1) [],
        HE_EN_3 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
        RD_EN_4 OFFSET(1) NUMBITS(1) [],
        PROG_EN_4 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_4 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_4 OFFSET(4) NUMBITS(1) [],
        ECC_EN_4 OFFSET(5) NUMBITS(1) [],
        HE_EN_4 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
        RD_EN_5 OFFSET(1) NUMBITS(1) [],
        PROG_EN_5 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_5 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_5 OFFSET(4) NUMBITS(1) [],
        ECC_EN_5 OFFSET(5) NUMBITS(1) [],
        HE_EN_5 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
        RD_EN_6 OFFSET(1) NUMBITS(1) [],
        PROG_EN_6 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_6 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_6 OFFSET(4) NUMBITS(1) [],
        ECC_EN_6 OFFSET(5) NUMBITS(1) [],
        HE_EN_6 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
        RD_EN_7 OFFSET(1) NUMBITS(1) [],
        PROG_EN_7 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_7 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_7 OFFSET(4) NUMBITS(1) [],
        ECC_EN_7 OFFSET(5) NUMBITS(1) [],
        HE_EN_7 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
        RD_EN_8 OFFSET(1) NUMBITS(1) [],
        PROG_EN_8 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_8 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_8 OFFSET(4) NUMBITS(1) [],
        ECC_EN_8 OFFSET(5) NUMBITS(1) [],
        HE_EN_8 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO0_PAGE_CFG_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
        RD_EN_9 OFFSET(1) NUMBITS(1) [],
        PROG_EN_9 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_9 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_9 OFFSET(4) NUMBITS(1) [],
        ECC_EN_9 OFFSET(5) NUMBITS(1) [],
        HE_EN_9 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO1_REGWEN [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE_LOCKED = 0,
            PAGE_ENABLED = 1,
        ],
    ],
    BANK0_INFO1_PAGE_CFG [
        EN_0 OFFSET(0) NUMBITS(1) [],
        RD_EN_0 OFFSET(1) NUMBITS(1) [],
        PROG_EN_0 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_0 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_0 OFFSET(4) NUMBITS(1) [],
        ECC_EN_0 OFFSET(5) NUMBITS(1) [],
        HE_EN_0 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO2_REGWEN_0 [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE_LOCKED = 0,
            PAGE_ENABLED = 1,
        ],
    ],
    BANK0_INFO2_REGWEN_1 [
        REGION_1 OFFSET(0) NUMBITS(1) [],
    ],
    BANK0_INFO2_PAGE_CFG_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
        RD_EN_0 OFFSET(1) NUMBITS(1) [],
        PROG_EN_0 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_0 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_0 OFFSET(4) NUMBITS(1) [],
        ECC_EN_0 OFFSET(5) NUMBITS(1) [],
        HE_EN_0 OFFSET(6) NUMBITS(1) [],
    ],
    BANK0_INFO2_PAGE_CFG_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
        RD_EN_1 OFFSET(1) NUMBITS(1) [],
        PROG_EN_1 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_1 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_1 OFFSET(4) NUMBITS(1) [],
        ECC_EN_1 OFFSET(5) NUMBITS(1) [],
        HE_EN_1 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_REGWEN_0 [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE_LOCKED = 0,
            PAGE_ENABLED = 1,
        ],
    ],
    BANK1_INFO0_REGWEN_1 [
        REGION_1 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO0_REGWEN_2 [
        REGION_2 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO0_REGWEN_3 [
        REGION_3 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO0_REGWEN_4 [
        REGION_4 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO0_REGWEN_5 [
        REGION_5 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO0_REGWEN_6 [
        REGION_6 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO0_REGWEN_7 [
        REGION_7 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO0_REGWEN_8 [
        REGION_8 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO0_REGWEN_9 [
        REGION_9 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
        RD_EN_0 OFFSET(1) NUMBITS(1) [],
        PROG_EN_0 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_0 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_0 OFFSET(4) NUMBITS(1) [],
        ECC_EN_0 OFFSET(5) NUMBITS(1) [],
        HE_EN_0 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
        RD_EN_1 OFFSET(1) NUMBITS(1) [],
        PROG_EN_1 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_1 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_1 OFFSET(4) NUMBITS(1) [],
        ECC_EN_1 OFFSET(5) NUMBITS(1) [],
        HE_EN_1 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_2 [
        EN_2 OFFSET(0) NUMBITS(1) [],
        RD_EN_2 OFFSET(1) NUMBITS(1) [],
        PROG_EN_2 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_2 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_2 OFFSET(4) NUMBITS(1) [],
        ECC_EN_2 OFFSET(5) NUMBITS(1) [],
        HE_EN_2 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_3 [
        EN_3 OFFSET(0) NUMBITS(1) [],
        RD_EN_3 OFFSET(1) NUMBITS(1) [],
        PROG_EN_3 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_3 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_3 OFFSET(4) NUMBITS(1) [],
        ECC_EN_3 OFFSET(5) NUMBITS(1) [],
        HE_EN_3 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_4 [
        EN_4 OFFSET(0) NUMBITS(1) [],
        RD_EN_4 OFFSET(1) NUMBITS(1) [],
        PROG_EN_4 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_4 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_4 OFFSET(4) NUMBITS(1) [],
        ECC_EN_4 OFFSET(5) NUMBITS(1) [],
        HE_EN_4 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_5 [
        EN_5 OFFSET(0) NUMBITS(1) [],
        RD_EN_5 OFFSET(1) NUMBITS(1) [],
        PROG_EN_5 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_5 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_5 OFFSET(4) NUMBITS(1) [],
        ECC_EN_5 OFFSET(5) NUMBITS(1) [],
        HE_EN_5 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_6 [
        EN_6 OFFSET(0) NUMBITS(1) [],
        RD_EN_6 OFFSET(1) NUMBITS(1) [],
        PROG_EN_6 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_6 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_6 OFFSET(4) NUMBITS(1) [],
        ECC_EN_6 OFFSET(5) NUMBITS(1) [],
        HE_EN_6 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_7 [
        EN_7 OFFSET(0) NUMBITS(1) [],
        RD_EN_7 OFFSET(1) NUMBITS(1) [],
        PROG_EN_7 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_7 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_7 OFFSET(4) NUMBITS(1) [],
        ECC_EN_7 OFFSET(5) NUMBITS(1) [],
        HE_EN_7 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_8 [
        EN_8 OFFSET(0) NUMBITS(1) [],
        RD_EN_8 OFFSET(1) NUMBITS(1) [],
        PROG_EN_8 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_8 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_8 OFFSET(4) NUMBITS(1) [],
        ECC_EN_8 OFFSET(5) NUMBITS(1) [],
        HE_EN_8 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO0_PAGE_CFG_9 [
        EN_9 OFFSET(0) NUMBITS(1) [],
        RD_EN_9 OFFSET(1) NUMBITS(1) [],
        PROG_EN_9 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_9 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_9 OFFSET(4) NUMBITS(1) [],
        ECC_EN_9 OFFSET(5) NUMBITS(1) [],
        HE_EN_9 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO1_REGWEN [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE_LOCKED = 0,
            PAGE_ENABLED = 1,
        ],
    ],
    BANK1_INFO1_PAGE_CFG [
        EN_0 OFFSET(0) NUMBITS(1) [],
        RD_EN_0 OFFSET(1) NUMBITS(1) [],
        PROG_EN_0 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_0 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_0 OFFSET(4) NUMBITS(1) [],
        ECC_EN_0 OFFSET(5) NUMBITS(1) [],
        HE_EN_0 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO2_REGWEN_0 [
        REGION_0 OFFSET(0) NUMBITS(1) [
            PAGE_LOCKED = 0,
            PAGE_ENABLED = 1,
        ],
    ],
    BANK1_INFO2_REGWEN_1 [
        REGION_1 OFFSET(0) NUMBITS(1) [],
    ],
    BANK1_INFO2_PAGE_CFG_0 [
        EN_0 OFFSET(0) NUMBITS(1) [],
        RD_EN_0 OFFSET(1) NUMBITS(1) [],
        PROG_EN_0 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_0 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_0 OFFSET(4) NUMBITS(1) [],
        ECC_EN_0 OFFSET(5) NUMBITS(1) [],
        HE_EN_0 OFFSET(6) NUMBITS(1) [],
    ],
    BANK1_INFO2_PAGE_CFG_1 [
        EN_1 OFFSET(0) NUMBITS(1) [],
        RD_EN_1 OFFSET(1) NUMBITS(1) [],
        PROG_EN_1 OFFSET(2) NUMBITS(1) [],
        ERASE_EN_1 OFFSET(3) NUMBITS(1) [],
        SCRAMBLE_EN_1 OFFSET(4) NUMBITS(1) [],
        ECC_EN_1 OFFSET(5) NUMBITS(1) [],
        HE_EN_1 OFFSET(6) NUMBITS(1) [],
    ],
    BANK_CFG_REGWEN [
        BANK OFFSET(0) NUMBITS(1) [
            BANK_LOCKED = 0,
            BANK_ENABLED = 1,
        ],
    ],
    MP_BANK_CFG [
        ERASE_EN_0 OFFSET(0) NUMBITS(1) [],
        ERASE_EN_1 OFFSET(1) NUMBITS(1) [],
    ],
    OP_STATUS [
        DONE OFFSET(0) NUMBITS(1) [],
        ERR OFFSET(1) NUMBITS(1) [],
    ],
    STATUS [
        RD_FULL OFFSET(0) NUMBITS(1) [],
        RD_EMPTY OFFSET(1) NUMBITS(1) [],
        PROG_FULL OFFSET(2) NUMBITS(1) [],
        PROG_EMPTY OFFSET(3) NUMBITS(1) [],
        INIT_WIP OFFSET(4) NUMBITS(1) [],
    ],
    ERR_CODE_INTR_EN [
        FLASH_ERR_EN OFFSET(0) NUMBITS(1) [],
        FLASH_ALERT_EN OFFSET(1) NUMBITS(1) [],
        OOB_ERR OFFSET(2) NUMBITS(1) [],
        MP_ERR OFFSET(3) NUMBITS(1) [],
        ECC_SINGLE_ERR OFFSET(4) NUMBITS(1) [],
        ECC_MULTI_ERR OFFSET(5) NUMBITS(1) [],
    ],
    ERR_CODE [
        FLASH_ERR OFFSET(0) NUMBITS(1) [],
        FLASH_ALERT OFFSET(1) NUMBITS(1) [],
        OOB_ERR OFFSET(2) NUMBITS(1) [],
        MP_ERR OFFSET(3) NUMBITS(1) [],
        ECC_SINGLE_ERR OFFSET(4) NUMBITS(1) [],
        ECC_MULTI_ERR OFFSET(5) NUMBITS(1) [],
    ],
    ERR_ADDR [
        ERR_ADDR OFFSET(0) NUMBITS(9) [],
    ],
    ECC_SINGLE_ERR_CNT [
        ECC_SINGLE_ERR_CNT OFFSET(0) NUMBITS(8) [],
    ],
    ECC_SINGLE_ERR_ADDR_0 [
        ECC_SINGLE_ERR_ADDR_0 OFFSET(0) NUMBITS(20) [],
    ],
    ECC_SINGLE_ERR_ADDR_1 [
        ECC_SINGLE_ERR_ADDR_1 OFFSET(0) NUMBITS(20) [],
    ],
    ECC_MULTI_ERR_CNT [
        ECC_MULTI_ERR_CNT OFFSET(0) NUMBITS(8) [],
    ],
    ECC_MULTI_ERR_ADDR_0 [
        ECC_MULTI_ERR_ADDR_0 OFFSET(0) NUMBITS(20) [],
    ],
    ECC_MULTI_ERR_ADDR_1 [
        ECC_MULTI_ERR_ADDR_1 OFFSET(0) NUMBITS(20) [],
    ],
    PHY_ERR_CFG_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    PHY_ERR_CFG [
        ECC_MULTI_ERR_DATA_EN OFFSET(0) NUMBITS(1) [],
    ],
    PHY_ALERT_CFG [
        ALERT_ACK OFFSET(0) NUMBITS(1) [],
        ALERT_TRIG OFFSET(1) NUMBITS(1) [],
    ],
    PHY_STATUS [
        INIT_WIP OFFSET(0) NUMBITS(1) [],
        PROG_NORMAL_AVAIL OFFSET(1) NUMBITS(1) [],
        PROG_REPAIR_AVAIL OFFSET(2) NUMBITS(1) [],
    ],
    SCRATCH [
        DATA OFFSET(0) NUMBITS(32) [],
    ],
    FIFO_LVL [
        PROG OFFSET(0) NUMBITS(5) [],
        RD OFFSET(8) NUMBITS(5) [],
    ],
    FIFO_RST [
        EN OFFSET(0) NUMBITS(1) [],
    ],
];

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

// Number of alerts
pub const FLASH_CTRL_PARAM_NUM_ALERTS: u32 = 4;

// Register width
pub const FLASH_CTRL_PARAM_REG_WIDTH: u32 = 32;

// Memory region registers configuration enable. (common parameters)
pub const FLASH_CTRL_REGION_CFG_REGWEN_REGION_FIELD_WIDTH: u32 = 1;
pub const FLASH_CTRL_REGION_CFG_REGWEN_REGION_FIELDS_PER_REG: u32 = 32;
pub const FLASH_CTRL_REGION_CFG_REGWEN_MULTIREG_COUNT: u32 = 8;

// Memory region registers configuration enable. (common parameters)
pub const FLASH_CTRL_BANK0_INFO0_REGWEN_REGION_FIELD_WIDTH: u32 = 1;
pub const FLASH_CTRL_BANK0_INFO0_REGWEN_REGION_FIELDS_PER_REG: u32 = 32;
pub const FLASH_CTRL_BANK0_INFO0_REGWEN_MULTIREG_COUNT: u32 = 10;

// Memory region registers configuration enable. (common parameters)
pub const FLASH_CTRL_BANK0_INFO1_REGWEN_REGION_FIELD_WIDTH: u32 = 1;
pub const FLASH_CTRL_BANK0_INFO1_REGWEN_REGION_FIELDS_PER_REG: u32 = 32;
pub const FLASH_CTRL_BANK0_INFO1_REGWEN_MULTIREG_COUNT: u32 = 1;

// Memory region registers configuration enable. (common parameters)
pub const FLASH_CTRL_BANK0_INFO2_REGWEN_REGION_FIELD_WIDTH: u32 = 1;
pub const FLASH_CTRL_BANK0_INFO2_REGWEN_REGION_FIELDS_PER_REG: u32 = 32;
pub const FLASH_CTRL_BANK0_INFO2_REGWEN_MULTIREG_COUNT: u32 = 2;

// Memory region registers configuration enable. (common parameters)
pub const FLASH_CTRL_BANK1_INFO0_REGWEN_REGION_FIELD_WIDTH: u32 = 1;
pub const FLASH_CTRL_BANK1_INFO0_REGWEN_REGION_FIELDS_PER_REG: u32 = 32;
pub const FLASH_CTRL_BANK1_INFO0_REGWEN_MULTIREG_COUNT: u32 = 10;

// Memory region registers configuration enable. (common parameters)
pub const FLASH_CTRL_BANK1_INFO1_REGWEN_REGION_FIELD_WIDTH: u32 = 1;
pub const FLASH_CTRL_BANK1_INFO1_REGWEN_REGION_FIELDS_PER_REG: u32 = 32;
pub const FLASH_CTRL_BANK1_INFO1_REGWEN_MULTIREG_COUNT: u32 = 1;

// Memory region registers configuration enable. (common parameters)
pub const FLASH_CTRL_BANK1_INFO2_REGWEN_REGION_FIELD_WIDTH: u32 = 1;
pub const FLASH_CTRL_BANK1_INFO2_REGWEN_REGION_FIELDS_PER_REG: u32 = 32;
pub const FLASH_CTRL_BANK1_INFO2_REGWEN_MULTIREG_COUNT: u32 = 2;

// Memory properties bank configuration (common parameters)
pub const FLASH_CTRL_MP_BANK_CFG_ERASE_EN_FIELD_WIDTH: u32 = 1;
pub const FLASH_CTRL_MP_BANK_CFG_ERASE_EN_FIELDS_PER_REG: u32 = 32;
pub const FLASH_CTRL_MP_BANK_CFG_MULTIREG_COUNT: u32 = 1;

// Latest single bit error address (correctable) (common parameters)
pub const FLASH_CTRL_ECC_SINGLE_ERR_ADDR_ECC_SINGLE_ERR_ADDR_FIELD_WIDTH: u32 = 20;
pub const FLASH_CTRL_ECC_SINGLE_ERR_ADDR_ECC_SINGLE_ERR_ADDR_FIELDS_PER_REG: u32 = 1;
pub const FLASH_CTRL_ECC_SINGLE_ERR_ADDR_MULTIREG_COUNT: u32 = 2;

// Latest multi bit error address (uncorrectable) (common parameters)
pub const FLASH_CTRL_ECC_MULTI_ERR_ADDR_ECC_MULTI_ERR_ADDR_FIELD_WIDTH: u32 = 20;
pub const FLASH_CTRL_ECC_MULTI_ERR_ADDR_ECC_MULTI_ERR_ADDR_FIELDS_PER_REG: u32 = 1;
pub const FLASH_CTRL_ECC_MULTI_ERR_ADDR_MULTIREG_COUNT: u32 = 2;

// Memory area: Flash program FIFO.
pub const FLASH_CTRL_PROG_FIFO_REG_OFFSET: usize = 0x190;
pub const FLASH_CTRL_PROG_FIFO_SIZE_WORDS: u32 = 1;
pub const FLASH_CTRL_PROG_FIFO_SIZE_BYTES: u32 = 4;
// Memory area: Flash read FIFO.
pub const FLASH_CTRL_RD_FIFO_REG_OFFSET: usize = 0x194;
pub const FLASH_CTRL_RD_FIFO_SIZE_WORDS: u32 = 1;
pub const FLASH_CTRL_RD_FIFO_SIZE_BYTES: u32 = 4;
// End generated register constants for FLASH_CTRL

