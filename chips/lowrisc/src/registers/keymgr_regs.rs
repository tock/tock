// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for keymgr.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/keymgr/data/keymgr.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of Registers for SW inputs (Salt)
pub const KEYMGR_PARAM_NUM_SALT_REG: u32 = 8;
/// Number of Registers for SW inputs (SW binding)
pub const KEYMGR_PARAM_NUM_SW_BINDING_REG: u32 = 8;
/// Number of Registers for SW outputs
pub const KEYMGR_PARAM_NUM_OUT_REG: u32 = 8;
/// Number of Registers for key version
pub const KEYMGR_PARAM_NUM_KEY_VERSION: u32 = 1;
/// Number of alerts
pub const KEYMGR_PARAM_NUM_ALERTS: u32 = 2;
/// Register width
pub const KEYMGR_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub KeymgrRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// Key manager configuration enable
        (0x0010 => pub(crate) cfg_regwen: ReadWrite<u32, CFG_REGWEN::Register>),
        /// Key manager operation start
        (0x0014 => pub(crate) start: ReadWrite<u32, START::Register>),
        /// Key manager operation controls
        (0x0018 => pub(crate) control_shadowed: ReadWrite<u32, CONTROL_SHADOWED::Register>),
        /// sideload key slots clear
        (0x001c => pub(crate) sideload_clear: ReadWrite<u32, SIDELOAD_CLEAR::Register>),
        /// regwen for reseed interval
        (0x0020 => pub(crate) reseed_interval_regwen: ReadWrite<u32, RESEED_INTERVAL_REGWEN::Register>),
        /// Reseed interval for key manager entropy reseed
        (0x0024 => pub(crate) reseed_interval_shadowed: ReadWrite<u32, RESEED_INTERVAL_SHADOWED::Register>),
        /// Register write enable for SOFTWARE_BINDING
        (0x0028 => pub(crate) sw_binding_regwen: ReadWrite<u32, SW_BINDING_REGWEN::Register>),
        /// Software binding input to sealing portion of the key manager.
        (0x002c => pub(crate) sealing_sw_binding: [ReadWrite<u32, SEALING_SW_BINDING::Register>; 8]),
        /// Software binding input to the attestation portion of the key manager.
        (0x004c => pub(crate) attest_sw_binding: [ReadWrite<u32, ATTEST_SW_BINDING::Register>; 8]),
        /// Salt value used as part of output generation
        (0x006c => pub(crate) salt: [ReadWrite<u32, SALT::Register>; 8]),
        /// Version used as part of output generation
        (0x008c => pub(crate) key_version: [ReadWrite<u32, KEY_VERSION::Register>; 1]),
        /// Register write enable for MAX_CREATOR_KEY_VERSION
        (0x0090 => pub(crate) max_creator_key_ver_regwen: ReadWrite<u32, MAX_CREATOR_KEY_VER_REGWEN::Register>),
        /// Max creator key version
        (0x0094 => pub(crate) max_creator_key_ver_shadowed: ReadWrite<u32, MAX_CREATOR_KEY_VER_SHADOWED::Register>),
        /// Register write enable for MAX_OWNER_INT_KEY_VERSION
        (0x0098 => pub(crate) max_owner_int_key_ver_regwen: ReadWrite<u32, MAX_OWNER_INT_KEY_VER_REGWEN::Register>),
        /// Max owner intermediate key version
        (0x009c => pub(crate) max_owner_int_key_ver_shadowed: ReadWrite<u32, MAX_OWNER_INT_KEY_VER_SHADOWED::Register>),
        /// Register write enable for MAX_OWNER_KEY_VERSION
        (0x00a0 => pub(crate) max_owner_key_ver_regwen: ReadWrite<u32, MAX_OWNER_KEY_VER_REGWEN::Register>),
        /// Max owner key version
        (0x00a4 => pub(crate) max_owner_key_ver_shadowed: ReadWrite<u32, MAX_OWNER_KEY_VER_SHADOWED::Register>),
        /// Key manager software output.
        (0x00a8 => pub(crate) sw_share0_output: [ReadWrite<u32, SW_SHARE0_OUTPUT::Register>; 8]),
        /// Key manager software output.
        (0x00c8 => pub(crate) sw_share1_output: [ReadWrite<u32, SW_SHARE1_OUTPUT::Register>; 8]),
        /// Key manager working state.
        (0x00e8 => pub(crate) working_state: ReadWrite<u32, WORKING_STATE::Register>),
        /// Key manager status.
        (0x00ec => pub(crate) op_status: ReadWrite<u32, OP_STATUS::Register>),
        /// Key manager error code.
        (0x00f0 => pub(crate) err_code: ReadWrite<u32, ERR_CODE::Register>),
        /// This register represents both synchronous and asynchronous fatal faults.
        (0x00f4 => pub(crate) fault_status: ReadWrite<u32, FAULT_STATUS::Register>),
        /// The register holds some debug information that may be convenient if keymgr
        (0x00f8 => pub(crate) debug: ReadWrite<u32, DEBUG::Register>),
        (0x00fc => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
        OP_DONE OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        RECOV_OPERATION_ERR OFFSET(0) NUMBITS(1) [],
        FATAL_FAULT_ERR OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) CFG_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) START [
        EN OFFSET(0) NUMBITS(1) [
            VALID_STATE = 1,
        ],
    ],
    pub(crate) CONTROL_SHADOWED [
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
        DEST_SEL OFFSET(12) NUMBITS(2) [
            NONE = 0,
            AES = 1,
            KMAC = 2,
            OTBN = 3,
        ],
    ],
    pub(crate) SIDELOAD_CLEAR [
        VAL OFFSET(0) NUMBITS(3) [
            NONE = 0,
            AES = 1,
            KMAC = 2,
            OTBN = 3,
        ],
    ],
    pub(crate) RESEED_INTERVAL_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) RESEED_INTERVAL_SHADOWED [
        VAL OFFSET(0) NUMBITS(16) [],
    ],
    pub(crate) SW_BINDING_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) SEALING_SW_BINDING [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) ATTEST_SW_BINDING [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) SALT [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) KEY_VERSION [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) MAX_CREATOR_KEY_VER_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) MAX_CREATOR_KEY_VER_SHADOWED [
        VAL OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) MAX_OWNER_INT_KEY_VER_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) MAX_OWNER_INT_KEY_VER_SHADOWED [
        VAL OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) MAX_OWNER_KEY_VER_REGWEN [
        EN OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) MAX_OWNER_KEY_VER_SHADOWED [
        VAL OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) SW_SHARE0_OUTPUT [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) SW_SHARE1_OUTPUT [
        VAL_0 OFFSET(0) NUMBITS(32) [],
    ],
    pub(crate) WORKING_STATE [
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
    pub(crate) OP_STATUS [
        STATUS OFFSET(0) NUMBITS(2) [
            IDLE = 0,
            WIP = 1,
            DONE_SUCCESS = 2,
            DONE_ERROR = 3,
        ],
    ],
    pub(crate) ERR_CODE [
        INVALID_OP OFFSET(0) NUMBITS(1) [],
        INVALID_KMAC_INPUT OFFSET(1) NUMBITS(1) [],
        INVALID_SHADOW_UPDATE OFFSET(2) NUMBITS(1) [],
    ],
    pub(crate) FAULT_STATUS [
        CMD OFFSET(0) NUMBITS(1) [],
        KMAC_FSM OFFSET(1) NUMBITS(1) [],
        KMAC_DONE OFFSET(2) NUMBITS(1) [],
        KMAC_OP OFFSET(3) NUMBITS(1) [],
        KMAC_OUT OFFSET(4) NUMBITS(1) [],
        REGFILE_INTG OFFSET(5) NUMBITS(1) [],
        SHADOW OFFSET(6) NUMBITS(1) [],
        CTRL_FSM_INTG OFFSET(7) NUMBITS(1) [],
        CTRL_FSM_CHK OFFSET(8) NUMBITS(1) [],
        CTRL_FSM_CNT OFFSET(9) NUMBITS(1) [],
        RESEED_CNT OFFSET(10) NUMBITS(1) [],
        SIDE_CTRL_FSM OFFSET(11) NUMBITS(1) [],
        SIDE_CTRL_SEL OFFSET(12) NUMBITS(1) [],
        KEY_ECC OFFSET(13) NUMBITS(1) [],
    ],
    pub(crate) DEBUG [
        INVALID_CREATOR_SEED OFFSET(0) NUMBITS(1) [],
        INVALID_OWNER_SEED OFFSET(1) NUMBITS(1) [],
        INVALID_DEV_ID OFFSET(2) NUMBITS(1) [],
        INVALID_HEALTH_STATE OFFSET(3) NUMBITS(1) [],
        INVALID_KEY_VERSION OFFSET(4) NUMBITS(1) [],
        INVALID_KEY OFFSET(5) NUMBITS(1) [],
        INVALID_DIGEST OFFSET(6) NUMBITS(1) [],
    ],
];

// End generated register constants for keymgr
