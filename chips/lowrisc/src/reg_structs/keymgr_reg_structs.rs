// Generated register struct for KEYMGR

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
    pub KeymgrRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => cfg_regwen: ReadOnly<u32, CFG_REGWEN::Register>),
        (0x14 => control: ReadWrite<u32, CONTROL::Register>),
        (0x18 => sideload_clear: ReadWrite<u32, SIDELOAD_CLEAR::Register>),
        (0x1c => reseed_interval_shadowed: ReadWrite<u32, RESEED_INTERVAL_SHADOWED::Register>),
        (0x20 => sw_binding_regwen: ReadWrite<u32, SW_BINDING_REGWEN::Register>),
        (0x24 => sealing_sw_binding_0: ReadWrite<u32, SEALING_SW_BINDING_0::Register>),
        (0x28 => sealing_sw_binding_1: ReadWrite<u32, SEALING_SW_BINDING_1::Register>),
        (0x2c => sealing_sw_binding_2: ReadWrite<u32, SEALING_SW_BINDING_2::Register>),
        (0x30 => sealing_sw_binding_3: ReadWrite<u32, SEALING_SW_BINDING_3::Register>),
        (0x34 => sealing_sw_binding_4: ReadWrite<u32, SEALING_SW_BINDING_4::Register>),
        (0x38 => sealing_sw_binding_5: ReadWrite<u32, SEALING_SW_BINDING_5::Register>),
        (0x3c => sealing_sw_binding_6: ReadWrite<u32, SEALING_SW_BINDING_6::Register>),
        (0x40 => sealing_sw_binding_7: ReadWrite<u32, SEALING_SW_BINDING_7::Register>),
        (0x44 => attest_sw_binding_0: ReadWrite<u32, ATTEST_SW_BINDING_0::Register>),
        (0x48 => attest_sw_binding_1: ReadWrite<u32, ATTEST_SW_BINDING_1::Register>),
        (0x4c => attest_sw_binding_2: ReadWrite<u32, ATTEST_SW_BINDING_2::Register>),
        (0x50 => attest_sw_binding_3: ReadWrite<u32, ATTEST_SW_BINDING_3::Register>),
        (0x54 => attest_sw_binding_4: ReadWrite<u32, ATTEST_SW_BINDING_4::Register>),
        (0x58 => attest_sw_binding_5: ReadWrite<u32, ATTEST_SW_BINDING_5::Register>),
        (0x5c => attest_sw_binding_6: ReadWrite<u32, ATTEST_SW_BINDING_6::Register>),
        (0x60 => attest_sw_binding_7: ReadWrite<u32, ATTEST_SW_BINDING_7::Register>),
        (0x64 => salt_0: ReadWrite<u32, Salt_0::Register>),
        (0x68 => salt_1: ReadWrite<u32, Salt_1::Register>),
        (0x6c => salt_2: ReadWrite<u32, Salt_2::Register>),
        (0x70 => salt_3: ReadWrite<u32, Salt_3::Register>),
        (0x74 => salt_4: ReadWrite<u32, Salt_4::Register>),
        (0x78 => salt_5: ReadWrite<u32, Salt_5::Register>),
        (0x7c => salt_6: ReadWrite<u32, Salt_6::Register>),
        (0x80 => salt_7: ReadWrite<u32, Salt_7::Register>),
        (0x84 => key_version: ReadWrite<u32, KEY_VERSION::Register>),
        (0x88 => max_creator_key_ver_regwen: ReadWrite<u32, MAX_CREATOR_KEY_VER_REGWEN::Register>),
        (0x8c => max_creator_key_ver_shadowed: ReadWrite<u32, MAX_CREATOR_KEY_VER_SHADOWED::Register>),
        (0x90 => max_owner_int_key_ver_regwen: ReadWrite<u32, MAX_OWNER_INT_KEY_VER_REGWEN::Register>),
        (0x94 => max_owner_int_key_ver_shadowed: ReadWrite<u32, MAX_OWNER_INT_KEY_VER_SHADOWED::Register>),
        (0x98 => max_owner_key_ver_regwen: ReadWrite<u32, MAX_OWNER_KEY_VER_REGWEN::Register>),
        (0x9c => max_owner_key_ver_shadowed: ReadWrite<u32, MAX_OWNER_KEY_VER_SHADOWED::Register>),
        (0xa0 => sw_share0_output_0: ReadWrite<u32, SW_SHARE0_OUTPUT_0::Register>),
        (0xa4 => sw_share0_output_1: ReadWrite<u32, SW_SHARE0_OUTPUT_1::Register>),
        (0xa8 => sw_share0_output_2: ReadWrite<u32, SW_SHARE0_OUTPUT_2::Register>),
        (0xac => sw_share0_output_3: ReadWrite<u32, SW_SHARE0_OUTPUT_3::Register>),
        (0xb0 => sw_share0_output_4: ReadWrite<u32, SW_SHARE0_OUTPUT_4::Register>),
        (0xb4 => sw_share0_output_5: ReadWrite<u32, SW_SHARE0_OUTPUT_5::Register>),
        (0xb8 => sw_share0_output_6: ReadWrite<u32, SW_SHARE0_OUTPUT_6::Register>),
        (0xbc => sw_share0_output_7: ReadWrite<u32, SW_SHARE0_OUTPUT_7::Register>),
        (0xc0 => sw_share1_output_0: ReadWrite<u32, SW_SHARE1_OUTPUT_0::Register>),
        (0xc4 => sw_share1_output_1: ReadWrite<u32, SW_SHARE1_OUTPUT_1::Register>),
        (0xc8 => sw_share1_output_2: ReadWrite<u32, SW_SHARE1_OUTPUT_2::Register>),
        (0xcc => sw_share1_output_3: ReadWrite<u32, SW_SHARE1_OUTPUT_3::Register>),
        (0xd0 => sw_share1_output_4: ReadWrite<u32, SW_SHARE1_OUTPUT_4::Register>),
        (0xd4 => sw_share1_output_5: ReadWrite<u32, SW_SHARE1_OUTPUT_5::Register>),
        (0xd8 => sw_share1_output_6: ReadWrite<u32, SW_SHARE1_OUTPUT_6::Register>),
        (0xdc => sw_share1_output_7: ReadWrite<u32, SW_SHARE1_OUTPUT_7::Register>),
        (0xe0 => working_state: ReadOnly<u32, WORKING_STATE::Register>),
        (0xe4 => op_status: ReadWrite<u32, OP_STATUS::Register>),
        (0xe8 => err_code: ReadWrite<u32, ERR_CODE::Register>),
        (0xec => fault_status: ReadOnly<u32, FAULT_STATUS::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        OP_DONE OFFSET(0) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        OP_DONE OFFSET(0) NUMBITS(1) [],
    ],
    INTR_TEST [
        OP_DONE OFFSET(0) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT_ERR OFFSET(0) NUMBITS(1) [],
        RECOV_OPERATION_ERR OFFSET(1) NUMBITS(1) [],
    ],
    CFG_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    CONTROL [
        START OFFSET(0) NUMBITS(1) [
            VALID_STATE = 1,
        ],
        OPERATION OFFSET(4) NUMBITS(3) [
            ADVANCE = 0,
            GENERATE_ID = 1,
            GENERATE_SW_OUTPUT = 2,
            GENERATE_HW_OUTPUT = 3,
            DISABLE = 4,
        ],
        CDI_SEL OFFSET(7) NUMBITS(1) [
            SEALING_CDI = 0,
            ATTESTATION_CDI = 1,
        ],
        DEST_SEL OFFSET(12) NUMBITS(3) [
            NONE = 0,
            AES = 1,
            KMAC = 2,
            OTBN = 3,
        ],
    ],
    SIDELOAD_CLEAR [
        VAL OFFSET(0) NUMBITS(3) [
            NONE = 0,
            AES = 1,
            KMAC = 2,
            OTBN = 3,
        ],
    ],
    RESEED_INTERVAL_SHADOWED [
        VAL OFFSET(0) NUMBITS(16) [],
    ],
    SW_BINDING_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    SEALING_SW_BINDING_0 [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    SEALING_SW_BINDING_1 [
        VAL_1 OFFSET(0) NUMBITS(32) [],
    ],
    SEALING_SW_BINDING_2 [
        VAL_2 OFFSET(0) NUMBITS(32) [],
    ],
    SEALING_SW_BINDING_3 [
        VAL_3 OFFSET(0) NUMBITS(32) [],
    ],
    SEALING_SW_BINDING_4 [
        VAL_4 OFFSET(0) NUMBITS(32) [],
    ],
    SEALING_SW_BINDING_5 [
        VAL_5 OFFSET(0) NUMBITS(32) [],
    ],
    SEALING_SW_BINDING_6 [
        VAL_6 OFFSET(0) NUMBITS(32) [],
    ],
    SEALING_SW_BINDING_7 [
        VAL_7 OFFSET(0) NUMBITS(32) [],
    ],
    ATTEST_SW_BINDING_0 [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    ATTEST_SW_BINDING_1 [
        VAL_1 OFFSET(0) NUMBITS(32) [],
    ],
    ATTEST_SW_BINDING_2 [
        VAL_2 OFFSET(0) NUMBITS(32) [],
    ],
    ATTEST_SW_BINDING_3 [
        VAL_3 OFFSET(0) NUMBITS(32) [],
    ],
    ATTEST_SW_BINDING_4 [
        VAL_4 OFFSET(0) NUMBITS(32) [],
    ],
    ATTEST_SW_BINDING_5 [
        VAL_5 OFFSET(0) NUMBITS(32) [],
    ],
    ATTEST_SW_BINDING_6 [
        VAL_6 OFFSET(0) NUMBITS(32) [],
    ],
    ATTEST_SW_BINDING_7 [
        VAL_7 OFFSET(0) NUMBITS(32) [],
    ],
    SALT_0 [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    SALT_1 [
        VAL_1 OFFSET(0) NUMBITS(32) [],
    ],
    SALT_2 [
        VAL_2 OFFSET(0) NUMBITS(32) [],
    ],
    SALT_3 [
        VAL_3 OFFSET(0) NUMBITS(32) [],
    ],
    SALT_4 [
        VAL_4 OFFSET(0) NUMBITS(32) [],
    ],
    SALT_5 [
        VAL_5 OFFSET(0) NUMBITS(32) [],
    ],
    SALT_6 [
        VAL_6 OFFSET(0) NUMBITS(32) [],
    ],
    SALT_7 [
        VAL_7 OFFSET(0) NUMBITS(32) [],
    ],
    KEY_VERSION [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    MAX_CREATOR_KEY_VER_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    MAX_CREATOR_KEY_VER_SHADOWED [
        VAL OFFSET(0) NUMBITS(32) [],
    ],
    MAX_OWNER_INT_KEY_VER_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    MAX_OWNER_INT_KEY_VER_SHADOWED [
        VAL OFFSET(0) NUMBITS(32) [],
    ],
    MAX_OWNER_KEY_VER_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    MAX_OWNER_KEY_VER_SHADOWED [
        VAL OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE0_OUTPUT_0 [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE0_OUTPUT_1 [
        VAL_1 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE0_OUTPUT_2 [
        VAL_2 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE0_OUTPUT_3 [
        VAL_3 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE0_OUTPUT_4 [
        VAL_4 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE0_OUTPUT_5 [
        VAL_5 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE0_OUTPUT_6 [
        VAL_6 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE0_OUTPUT_7 [
        VAL_7 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE1_OUTPUT_0 [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE1_OUTPUT_1 [
        VAL_1 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE1_OUTPUT_2 [
        VAL_2 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE1_OUTPUT_3 [
        VAL_3 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE1_OUTPUT_4 [
        VAL_4 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE1_OUTPUT_5 [
        VAL_5 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE1_OUTPUT_6 [
        VAL_6 OFFSET(0) NUMBITS(32) [],
    ],
    SW_SHARE1_OUTPUT_7 [
        VAL_7 OFFSET(0) NUMBITS(32) [],
    ],
    WORKING_STATE [
        STATE OFFSET(0) NUMBITS(3) [
            RESET = 0,
            INIT = 1,
            CREATOR_ROOT_KEY = 2,
            OWNER_INTERMEDIATE_KEY = 3,
            OWNER_KEY = 4,
            DISABLED = 5,
            INVALID = 6,
        ],
    ],
    OP_STATUS [
        STATUS OFFSET(0) NUMBITS(2) [
            IDLE = 0,
            WIP = 1,
            DONE_SUCCESS = 2,
            DONE_ERROR = 3,
        ],
    ],
    ERR_CODE [
        INVALID_OP OFFSET(0) NUMBITS(1) [],
        INVALID_KMAC_INPUT OFFSET(1) NUMBITS(1) [],
        INVALID_SHADOW_UPDATE OFFSET(2) NUMBITS(1) [],
    ],
    FAULT_STATUS [
        CMD OFFSET(0) NUMBITS(1) [],
        KMAC_FSM OFFSET(1) NUMBITS(1) [],
        KMAC_OP OFFSET(2) NUMBITS(1) [],
        KMAC_OUT OFFSET(3) NUMBITS(1) [],
        REGFILE_INTG OFFSET(4) NUMBITS(1) [],
        SHADOW OFFSET(5) NUMBITS(1) [],
        CTRL_FSM_INTG OFFSET(6) NUMBITS(1) [],
        CTRL_FSM_CNT OFFSET(7) NUMBITS(1) [],
    ],
];

// Number of Registers for SW inputs (Salt)
pub const KEYMGR_PARAM_NUM_SALT_REG: u32 = 8;

// Number of Registers for SW inputs (SW binding)
pub const KEYMGR_PARAM_NUM_SW_BINDING_REG: u32 = 8;

// Number of Registers for SW outputs
pub const KEYMGR_PARAM_NUM_OUT_REG: u32 = 8;

// Number of Registers for key version
pub const KEYMGR_PARAM_NUM_KEY_VERSION: u32 = 1;

// Number of alerts
pub const KEYMGR_PARAM_NUM_ALERTS: u32 = 2;

// Register width
pub const KEYMGR_PARAM_REG_WIDTH: u32 = 32;

// Software binding input to sealing portion of the key manager.
pub const KEYMGR_SEALING_SW_BINDING_VAL_FIELD_WIDTH: u32 = 32;
pub const KEYMGR_SEALING_SW_BINDING_VAL_FIELDS_PER_REG: u32 = 1;
pub const KEYMGR_SEALING_SW_BINDING_MULTIREG_COUNT: u32 = 8;

// Software binding input to the attestation portion of the key manager.
pub const KEYMGR_ATTEST_SW_BINDING_VAL_FIELD_WIDTH: u32 = 32;
pub const KEYMGR_ATTEST_SW_BINDING_VAL_FIELDS_PER_REG: u32 = 1;
pub const KEYMGR_ATTEST_SW_BINDING_MULTIREG_COUNT: u32 = 8;

// Salt value used as part of output generation (common parameters)
pub const KEYMGR_SALT_VAL_FIELD_WIDTH: u32 = 32;
pub const KEYMGR_SALT_VAL_FIELDS_PER_REG: u32 = 1;
pub const KEYMGR_SALT_MULTIREG_COUNT: u32 = 8;

// Version used as part of output generation (common parameters)
pub const KEYMGR_KEY_VERSION_VAL_FIELD_WIDTH: u32 = 32;
pub const KEYMGR_KEY_VERSION_VAL_FIELDS_PER_REG: u32 = 1;
pub const KEYMGR_KEY_VERSION_MULTIREG_COUNT: u32 = 1;

// Key manager software output.
pub const KEYMGR_SW_SHARE0_OUTPUT_VAL_FIELD_WIDTH: u32 = 32;
pub const KEYMGR_SW_SHARE0_OUTPUT_VAL_FIELDS_PER_REG: u32 = 1;
pub const KEYMGR_SW_SHARE0_OUTPUT_MULTIREG_COUNT: u32 = 8;

// Key manager software output.
pub const KEYMGR_SW_SHARE1_OUTPUT_VAL_FIELD_WIDTH: u32 = 32;
pub const KEYMGR_SW_SHARE1_OUTPUT_VAL_FIELDS_PER_REG: u32 = 1;
pub const KEYMGR_SW_SHARE1_OUTPUT_MULTIREG_COUNT: u32 = 8;

// End generated register constants for KEYMGR

