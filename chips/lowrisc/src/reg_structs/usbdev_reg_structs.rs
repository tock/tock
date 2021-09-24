// Generated register struct for usbdev

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
    pub UsbdevRegisters {
        (0x0 => intr_state: ReadWrite<u32, INTR_STATE::Register>),
        (0x4 => intr_enable: ReadWrite<u32, INTR_ENABLE::Register>),
        (0x8 => intr_test: WriteOnly<u32, INTR_TEST::Register>),
        (0xc => alert_test: WriteOnly<u32, ALERT_TEST::Register>),
        (0x10 => usbctrl: ReadWrite<u32, usbctrl::Register>),
        (0x14 => usbstat: ReadOnly<u32, usbstat::Register>),
        (0x18 => avbuffer: WriteOnly<u32, avbuffer::Register>),
        (0x1c => rxfifo: ReadOnly<u32, rxfifo::Register>),
        (0x20 => rxenable_setup: ReadWrite<u32, rxenable_setup::Register>),
        (0x24 => rxenable_out: ReadWrite<u32, rxenable_out::Register>),
        (0x28 => in_sent: ReadWrite<u32, in_sent::Register>),
        (0x2c => stall: ReadWrite<u32, stall::Register>),
        (0x30 => configin_0: ReadWrite<u32, configin_0::Register>),
        (0x34 => configin_1: ReadWrite<u32, configin_1::Register>),
        (0x38 => configin_2: ReadWrite<u32, configin_2::Register>),
        (0x3c => configin_3: ReadWrite<u32, configin_3::Register>),
        (0x40 => configin_4: ReadWrite<u32, configin_4::Register>),
        (0x44 => configin_5: ReadWrite<u32, configin_5::Register>),
        (0x48 => configin_6: ReadWrite<u32, configin_6::Register>),
        (0x4c => configin_7: ReadWrite<u32, configin_7::Register>),
        (0x50 => configin_8: ReadWrite<u32, configin_8::Register>),
        (0x54 => configin_9: ReadWrite<u32, configin_9::Register>),
        (0x58 => configin_10: ReadWrite<u32, configin_10::Register>),
        (0x5c => configin_11: ReadWrite<u32, configin_11::Register>),
        (0x60 => iso: ReadWrite<u32, iso::Register>),
        (0x64 => data_toggle_clear: WriteOnly<u32, data_toggle_clear::Register>),
        (0x68 => phy_pins_sense: ReadOnly<u32, phy_pins_sense::Register>),
        (0x6c => phy_pins_drive: ReadWrite<u32, phy_pins_drive::Register>),
        (0x70 => phy_config: ReadWrite<u32, phy_config::Register>),
        (0x74 => wake_config: ReadWrite<u32, wake_config::Register>),
        (0x78 => wake_debug: ReadOnly<u32, wake_debug::Register>),
    }
}

register_bitfields![u32,
    INTR_STATE [
        PKT_RECEIVED OFFSET(0) NUMBITS(1) [],
        PKT_SENT OFFSET(1) NUMBITS(1) [],
        DISCONNECTED OFFSET(2) NUMBITS(1) [],
        HOST_LOST OFFSET(3) NUMBITS(1) [],
        LINK_RESET OFFSET(4) NUMBITS(1) [],
        LINK_SUSPEND OFFSET(5) NUMBITS(1) [],
        LINK_RESUME OFFSET(6) NUMBITS(1) [],
        AV_EMPTY OFFSET(7) NUMBITS(1) [],
        RX_FULL OFFSET(8) NUMBITS(1) [],
        AV_OVERFLOW OFFSET(9) NUMBITS(1) [],
        LINK_IN_ERR OFFSET(10) NUMBITS(1) [],
        RX_CRC_ERR OFFSET(11) NUMBITS(1) [],
        RX_PID_ERR OFFSET(12) NUMBITS(1) [],
        RX_BITSTUFF_ERR OFFSET(13) NUMBITS(1) [],
        FRAME OFFSET(14) NUMBITS(1) [],
        CONNECTED OFFSET(15) NUMBITS(1) [],
        LINK_OUT_ERR OFFSET(16) NUMBITS(1) [],
    ],
    INTR_ENABLE [
        PKT_RECEIVED OFFSET(0) NUMBITS(1) [],
        PKT_SENT OFFSET(1) NUMBITS(1) [],
        DISCONNECTED OFFSET(2) NUMBITS(1) [],
        HOST_LOST OFFSET(3) NUMBITS(1) [],
        LINK_RESET OFFSET(4) NUMBITS(1) [],
        LINK_SUSPEND OFFSET(5) NUMBITS(1) [],
        LINK_RESUME OFFSET(6) NUMBITS(1) [],
        AV_EMPTY OFFSET(7) NUMBITS(1) [],
        RX_FULL OFFSET(8) NUMBITS(1) [],
        AV_OVERFLOW OFFSET(9) NUMBITS(1) [],
        LINK_IN_ERR OFFSET(10) NUMBITS(1) [],
        RX_CRC_ERR OFFSET(11) NUMBITS(1) [],
        RX_PID_ERR OFFSET(12) NUMBITS(1) [],
        RX_BITSTUFF_ERR OFFSET(13) NUMBITS(1) [],
        FRAME OFFSET(14) NUMBITS(1) [],
        CONNECTED OFFSET(15) NUMBITS(1) [],
        LINK_OUT_ERR OFFSET(16) NUMBITS(1) [],
    ],
    INTR_TEST [
        PKT_RECEIVED OFFSET(0) NUMBITS(1) [],
        PKT_SENT OFFSET(1) NUMBITS(1) [],
        DISCONNECTED OFFSET(2) NUMBITS(1) [],
        HOST_LOST OFFSET(3) NUMBITS(1) [],
        LINK_RESET OFFSET(4) NUMBITS(1) [],
        LINK_SUSPEND OFFSET(5) NUMBITS(1) [],
        LINK_RESUME OFFSET(6) NUMBITS(1) [],
        AV_EMPTY OFFSET(7) NUMBITS(1) [],
        RX_FULL OFFSET(8) NUMBITS(1) [],
        AV_OVERFLOW OFFSET(9) NUMBITS(1) [],
        LINK_IN_ERR OFFSET(10) NUMBITS(1) [],
        RX_CRC_ERR OFFSET(11) NUMBITS(1) [],
        RX_PID_ERR OFFSET(12) NUMBITS(1) [],
        RX_BITSTUFF_ERR OFFSET(13) NUMBITS(1) [],
        FRAME OFFSET(14) NUMBITS(1) [],
        CONNECTED OFFSET(15) NUMBITS(1) [],
        LINK_OUT_ERR OFFSET(16) NUMBITS(1) [],
    ],
    ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    USBCTRL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        DEVICE_ADDRESS OFFSET(16) NUMBITS(7) [],
    ],
    USBSTAT [
        FRAME OFFSET(0) NUMBITS(11) [],
        HOST_LOST OFFSET(11) NUMBITS(1) [],
        LINK_STATE OFFSET(12) NUMBITS(3) [
            DISCONNECT = 0,
            POWERED = 1,
            POWERED_SUSPEND = 2,
            ACTIVE = 3,
            SUSPEND = 4,
            ACTIVE_NOSOF = 5,
        ],
        SENSE OFFSET(15) NUMBITS(1) [],
        AV_DEPTH OFFSET(16) NUMBITS(3) [],
        AV_FULL OFFSET(23) NUMBITS(1) [],
        RX_DEPTH OFFSET(24) NUMBITS(3) [],
        RX_EMPTY OFFSET(31) NUMBITS(1) [],
    ],
    AVBUFFER [
        BUFFER OFFSET(0) NUMBITS(5) [],
    ],
    RXFIFO [
        BUFFER OFFSET(0) NUMBITS(5) [],
        SIZE OFFSET(8) NUMBITS(7) [],
        SETUP OFFSET(19) NUMBITS(1) [],
        EP OFFSET(20) NUMBITS(4) [],
    ],
    RXENABLE_SETUP [
        SETUP_0 OFFSET(0) NUMBITS(1) [],
        SETUP_1 OFFSET(1) NUMBITS(1) [],
        SETUP_2 OFFSET(2) NUMBITS(1) [],
        SETUP_3 OFFSET(3) NUMBITS(1) [],
        SETUP_4 OFFSET(4) NUMBITS(1) [],
        SETUP_5 OFFSET(5) NUMBITS(1) [],
        SETUP_6 OFFSET(6) NUMBITS(1) [],
        SETUP_7 OFFSET(7) NUMBITS(1) [],
        SETUP_8 OFFSET(8) NUMBITS(1) [],
        SETUP_9 OFFSET(9) NUMBITS(1) [],
        SETUP_10 OFFSET(10) NUMBITS(1) [],
        SETUP_11 OFFSET(11) NUMBITS(1) [],
    ],
    RXENABLE_OUT [
        OUT_0 OFFSET(0) NUMBITS(1) [],
        OUT_1 OFFSET(1) NUMBITS(1) [],
        OUT_2 OFFSET(2) NUMBITS(1) [],
        OUT_3 OFFSET(3) NUMBITS(1) [],
        OUT_4 OFFSET(4) NUMBITS(1) [],
        OUT_5 OFFSET(5) NUMBITS(1) [],
        OUT_6 OFFSET(6) NUMBITS(1) [],
        OUT_7 OFFSET(7) NUMBITS(1) [],
        OUT_8 OFFSET(8) NUMBITS(1) [],
        OUT_9 OFFSET(9) NUMBITS(1) [],
        OUT_10 OFFSET(10) NUMBITS(1) [],
        OUT_11 OFFSET(11) NUMBITS(1) [],
    ],
    IN_SENT [
        SENT_0 OFFSET(0) NUMBITS(1) [],
        SENT_1 OFFSET(1) NUMBITS(1) [],
        SENT_2 OFFSET(2) NUMBITS(1) [],
        SENT_3 OFFSET(3) NUMBITS(1) [],
        SENT_4 OFFSET(4) NUMBITS(1) [],
        SENT_5 OFFSET(5) NUMBITS(1) [],
        SENT_6 OFFSET(6) NUMBITS(1) [],
        SENT_7 OFFSET(7) NUMBITS(1) [],
        SENT_8 OFFSET(8) NUMBITS(1) [],
        SENT_9 OFFSET(9) NUMBITS(1) [],
        SENT_10 OFFSET(10) NUMBITS(1) [],
        SENT_11 OFFSET(11) NUMBITS(1) [],
    ],
    STALL [
        STALL_0 OFFSET(0) NUMBITS(1) [],
        STALL_1 OFFSET(1) NUMBITS(1) [],
        STALL_2 OFFSET(2) NUMBITS(1) [],
        STALL_3 OFFSET(3) NUMBITS(1) [],
        STALL_4 OFFSET(4) NUMBITS(1) [],
        STALL_5 OFFSET(5) NUMBITS(1) [],
        STALL_6 OFFSET(6) NUMBITS(1) [],
        STALL_7 OFFSET(7) NUMBITS(1) [],
        STALL_8 OFFSET(8) NUMBITS(1) [],
        STALL_9 OFFSET(9) NUMBITS(1) [],
        STALL_10 OFFSET(10) NUMBITS(1) [],
        STALL_11 OFFSET(11) NUMBITS(1) [],
    ],
    CONFIGIN_0 [
        BUFFER_0 OFFSET(0) NUMBITS(5) [],
        SIZE_0 OFFSET(8) NUMBITS(7) [],
        PEND_0 OFFSET(30) NUMBITS(1) [],
        RDY_0 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_1 [
        BUFFER_1 OFFSET(0) NUMBITS(5) [],
        SIZE_1 OFFSET(8) NUMBITS(7) [],
        PEND_1 OFFSET(30) NUMBITS(1) [],
        RDY_1 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_2 [
        BUFFER_2 OFFSET(0) NUMBITS(5) [],
        SIZE_2 OFFSET(8) NUMBITS(7) [],
        PEND_2 OFFSET(30) NUMBITS(1) [],
        RDY_2 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_3 [
        BUFFER_3 OFFSET(0) NUMBITS(5) [],
        SIZE_3 OFFSET(8) NUMBITS(7) [],
        PEND_3 OFFSET(30) NUMBITS(1) [],
        RDY_3 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_4 [
        BUFFER_4 OFFSET(0) NUMBITS(5) [],
        SIZE_4 OFFSET(8) NUMBITS(7) [],
        PEND_4 OFFSET(30) NUMBITS(1) [],
        RDY_4 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_5 [
        BUFFER_5 OFFSET(0) NUMBITS(5) [],
        SIZE_5 OFFSET(8) NUMBITS(7) [],
        PEND_5 OFFSET(30) NUMBITS(1) [],
        RDY_5 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_6 [
        BUFFER_6 OFFSET(0) NUMBITS(5) [],
        SIZE_6 OFFSET(8) NUMBITS(7) [],
        PEND_6 OFFSET(30) NUMBITS(1) [],
        RDY_6 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_7 [
        BUFFER_7 OFFSET(0) NUMBITS(5) [],
        SIZE_7 OFFSET(8) NUMBITS(7) [],
        PEND_7 OFFSET(30) NUMBITS(1) [],
        RDY_7 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_8 [
        BUFFER_8 OFFSET(0) NUMBITS(5) [],
        SIZE_8 OFFSET(8) NUMBITS(7) [],
        PEND_8 OFFSET(30) NUMBITS(1) [],
        RDY_8 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_9 [
        BUFFER_9 OFFSET(0) NUMBITS(5) [],
        SIZE_9 OFFSET(8) NUMBITS(7) [],
        PEND_9 OFFSET(30) NUMBITS(1) [],
        RDY_9 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_10 [
        BUFFER_10 OFFSET(0) NUMBITS(5) [],
        SIZE_10 OFFSET(8) NUMBITS(7) [],
        PEND_10 OFFSET(30) NUMBITS(1) [],
        RDY_10 OFFSET(31) NUMBITS(1) [],
    ],
    CONFIGIN_11 [
        BUFFER_11 OFFSET(0) NUMBITS(5) [],
        SIZE_11 OFFSET(8) NUMBITS(7) [],
        PEND_11 OFFSET(30) NUMBITS(1) [],
        RDY_11 OFFSET(31) NUMBITS(1) [],
    ],
    ISO [
        ISO_0 OFFSET(0) NUMBITS(1) [],
        ISO_1 OFFSET(1) NUMBITS(1) [],
        ISO_2 OFFSET(2) NUMBITS(1) [],
        ISO_3 OFFSET(3) NUMBITS(1) [],
        ISO_4 OFFSET(4) NUMBITS(1) [],
        ISO_5 OFFSET(5) NUMBITS(1) [],
        ISO_6 OFFSET(6) NUMBITS(1) [],
        ISO_7 OFFSET(7) NUMBITS(1) [],
        ISO_8 OFFSET(8) NUMBITS(1) [],
        ISO_9 OFFSET(9) NUMBITS(1) [],
        ISO_10 OFFSET(10) NUMBITS(1) [],
        ISO_11 OFFSET(11) NUMBITS(1) [],
    ],
    DATA_TOGGLE_CLEAR [
        CLEAR_0 OFFSET(0) NUMBITS(1) [],
        CLEAR_1 OFFSET(1) NUMBITS(1) [],
        CLEAR_2 OFFSET(2) NUMBITS(1) [],
        CLEAR_3 OFFSET(3) NUMBITS(1) [],
        CLEAR_4 OFFSET(4) NUMBITS(1) [],
        CLEAR_5 OFFSET(5) NUMBITS(1) [],
        CLEAR_6 OFFSET(6) NUMBITS(1) [],
        CLEAR_7 OFFSET(7) NUMBITS(1) [],
        CLEAR_8 OFFSET(8) NUMBITS(1) [],
        CLEAR_9 OFFSET(9) NUMBITS(1) [],
        CLEAR_10 OFFSET(10) NUMBITS(1) [],
        CLEAR_11 OFFSET(11) NUMBITS(1) [],
    ],
    PHY_PINS_SENSE [
        RX_DP_I OFFSET(0) NUMBITS(1) [],
        RX_DN_I OFFSET(1) NUMBITS(1) [],
        RX_D_I OFFSET(2) NUMBITS(1) [],
        TX_DP_O OFFSET(8) NUMBITS(1) [],
        TX_DN_O OFFSET(9) NUMBITS(1) [],
        TX_D_O OFFSET(10) NUMBITS(1) [],
        TX_SE0_O OFFSET(11) NUMBITS(1) [],
        TX_OE_O OFFSET(12) NUMBITS(1) [],
        SUSPEND_O OFFSET(13) NUMBITS(1) [],
        PWR_SENSE OFFSET(16) NUMBITS(1) [],
    ],
    PHY_PINS_DRIVE [
        DP_O OFFSET(0) NUMBITS(1) [],
        DN_O OFFSET(1) NUMBITS(1) [],
        D_O OFFSET(2) NUMBITS(1) [],
        SE0_O OFFSET(3) NUMBITS(1) [],
        OE_O OFFSET(4) NUMBITS(1) [],
        TX_MODE_SE_O OFFSET(5) NUMBITS(1) [],
        DP_PULLUP_EN_O OFFSET(6) NUMBITS(1) [],
        DN_PULLUP_EN_O OFFSET(7) NUMBITS(1) [],
        SUSPEND_O OFFSET(8) NUMBITS(1) [],
        EN OFFSET(16) NUMBITS(1) [],
    ],
    PHY_CONFIG [
        RX_DIFFERENTIAL_MODE OFFSET(0) NUMBITS(1) [],
        TX_DIFFERENTIAL_MODE OFFSET(1) NUMBITS(1) [],
        EOP_SINGLE_BIT OFFSET(2) NUMBITS(1) [],
        OVERRIDE_PWR_SENSE_EN OFFSET(3) NUMBITS(1) [],
        OVERRIDE_PWR_SENSE_VAL OFFSET(4) NUMBITS(1) [],
        PINFLIP OFFSET(5) NUMBITS(1) [],
        USB_REF_DISABLE OFFSET(6) NUMBITS(1) [],
        TX_OSC_TEST_MODE OFFSET(7) NUMBITS(1) [],
    ],
    WAKE_CONFIG [
        WAKE_EN OFFSET(0) NUMBITS(1) [],
        WAKE_ACK OFFSET(1) NUMBITS(1) [],
    ],
    WAKE_DEBUG [
        STATE OFFSET(0) NUMBITS(3) [],
    ],
];

// Number of endpoints
pub const USBDEV_PARAM_N_ENDPOINTS: u32 = 12;

// Number of alerts
pub const USBDEV_PARAM_NUM_ALERTS: u32 = 1;

// Register width
pub const USBDEV_PARAM_REG_WIDTH: u32 = 32;

// Receive SETUP transaction enable (common parameters)
pub const USBDEV_RXENABLE_SETUP_SETUP_FIELD_WIDTH: u32 = 1;
pub const USBDEV_RXENABLE_SETUP_SETUP_FIELDS_PER_REG: u32 = 32;
pub const USBDEV_RXENABLE_SETUP_MULTIREG_COUNT: u32 = 1;

// Receive OUT transaction enable (common parameters)
pub const USBDEV_RXENABLE_OUT_OUT_FIELD_WIDTH: u32 = 1;
pub const USBDEV_RXENABLE_OUT_OUT_FIELDS_PER_REG: u32 = 32;
pub const USBDEV_RXENABLE_OUT_MULTIREG_COUNT: u32 = 1;

// IN Transaction Sent (common parameters)
pub const USBDEV_IN_SENT_SENT_FIELD_WIDTH: u32 = 1;
pub const USBDEV_IN_SENT_SENT_FIELDS_PER_REG: u32 = 32;
pub const USBDEV_IN_SENT_MULTIREG_COUNT: u32 = 1;

// Endpoint STALL control (common parameters)
pub const USBDEV_STALL_STALL_FIELD_WIDTH: u32 = 1;
pub const USBDEV_STALL_STALL_FIELDS_PER_REG: u32 = 32;
pub const USBDEV_STALL_MULTIREG_COUNT: u32 = 1;

// Endpoint ISO setting (common parameters)
pub const USBDEV_ISO_ISO_FIELD_WIDTH: u32 = 1;
pub const USBDEV_ISO_ISO_FIELDS_PER_REG: u32 = 32;
pub const USBDEV_ISO_MULTIREG_COUNT: u32 = 1;

// Clear the data toggle flag (common parameters)
pub const USBDEV_DATA_TOGGLE_CLEAR_CLEAR_FIELD_WIDTH: u32 = 1;
pub const USBDEV_DATA_TOGGLE_CLEAR_CLEAR_FIELDS_PER_REG: u32 = 32;
pub const USBDEV_DATA_TOGGLE_CLEAR_MULTIREG_COUNT: u32 = 1;

// Memory area: 2 kB packet buffer. Divided into 32 64-byte buffers.
pub const USBDEV_BUFFER_REG_OFFSET: usize = 0x800;
pub const USBDEV_BUFFER_SIZE_WORDS: u32 = 512;
pub const USBDEV_BUFFER_SIZE_BYTES: u32 = 2048;
// End generated register constants for usbdev

