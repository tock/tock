// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright lowRISC contributors 2023.

// Generated register constants for usbdev.
// Built for Earlgrey-M2.5.1-RC1-438-gacc67de99
// https://github.com/lowRISC/opentitan/tree/acc67de992ee8de5f2481b1b9580679850d8b5f5
// Tree status: clean
// Build date: 2023-08-08T00:15:38

// Original reference file: hw/ip/usbdev/data/usbdev.hjson
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::{register_bitfields, register_structs};
/// Number of endpoints
pub const USBDEV_PARAM_N_ENDPOINTS: u32 = 12;
/// Number of alerts
pub const USBDEV_PARAM_NUM_ALERTS: u32 = 1;
/// Register width
pub const USBDEV_PARAM_REG_WIDTH: u32 = 32;

register_structs! {
    pub UsbdevRegisters {
        /// Interrupt State Register
        (0x0000 => pub(crate) intr_state: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable Register
        (0x0004 => pub(crate) intr_enable: ReadWrite<u32, INTR::Register>),
        /// Interrupt Test Register
        (0x0008 => pub(crate) intr_test: ReadWrite<u32, INTR::Register>),
        /// Alert Test Register
        (0x000c => pub(crate) alert_test: ReadWrite<u32, ALERT_TEST::Register>),
        /// USB Control
        (0x0010 => pub(crate) usbctrl: ReadWrite<u32, USBCTRL::Register>),
        /// Enable an endpoint to respond to transactions in the downstream direction.
        (0x0014 => pub(crate) ep_out_enable: [ReadWrite<u32, EP_OUT_ENABLE::Register>; 1]),
        /// Enable an endpoint to respond to transactions in the upstream direction.
        (0x0018 => pub(crate) ep_in_enable: [ReadWrite<u32, EP_IN_ENABLE::Register>; 1]),
        /// USB Status
        (0x001c => pub(crate) usbstat: ReadWrite<u32, USBSTAT::Register>),
        /// Available Buffer FIFO
        (0x0020 => pub(crate) avbuffer: ReadWrite<u32, AVBUFFER::Register>),
        /// Received Buffer FIFO
        (0x0024 => pub(crate) rxfifo: ReadWrite<u32, RXFIFO::Register>),
        /// Receive SETUP transaction enable
        (0x0028 => pub(crate) rxenable_setup: [ReadWrite<u32, RXENABLE_SETUP::Register>; 1]),
        /// Receive OUT transaction enable
        (0x002c => pub(crate) rxenable_out: [ReadWrite<u32, RXENABLE_OUT::Register>; 1]),
        /// Set NAK after OUT transactions
        (0x0030 => pub(crate) set_nak_out: [ReadWrite<u32, SET_NAK_OUT::Register>; 1]),
        /// IN Transaction Sent
        (0x0034 => pub(crate) in_sent: [ReadWrite<u32, IN_SENT::Register>; 1]),
        /// OUT Endpoint STALL control
        (0x0038 => pub(crate) out_stall: [ReadWrite<u32, OUT_STALL::Register>; 1]),
        /// IN Endpoint STALL control
        (0x003c => pub(crate) in_stall: [ReadWrite<u32, IN_STALL::Register>; 1]),
        /// Configure IN Transaction
        (0x0040 => pub(crate) configin: [ReadWrite<u32, CONFIGIN::Register>; 12]),
        /// OUT Endpoint isochronous setting
        (0x0070 => pub(crate) out_iso: [ReadWrite<u32, OUT_ISO::Register>; 1]),
        /// IN Endpoint isochronous setting
        (0x0074 => pub(crate) in_iso: [ReadWrite<u32, IN_ISO::Register>; 1]),
        /// Clear the data toggle flag
        (0x0078 => pub(crate) data_toggle_clear: [ReadWrite<u32, DATA_TOGGLE_CLEAR::Register>; 1]),
        /// USB PHY pins sense.
        (0x007c => pub(crate) phy_pins_sense: ReadWrite<u32, PHY_PINS_SENSE::Register>),
        /// USB PHY pins drive.
        (0x0080 => pub(crate) phy_pins_drive: ReadWrite<u32, PHY_PINS_DRIVE::Register>),
        /// USB PHY Configuration
        (0x0084 => pub(crate) phy_config: ReadWrite<u32, PHY_CONFIG::Register>),
        /// USB wake module control for suspend / resume
        (0x0088 => pub(crate) wake_control: ReadWrite<u32, WAKE_CONTROL::Register>),
        /// USB wake module events and debug
        (0x008c => pub(crate) wake_events: ReadWrite<u32, WAKE_EVENTS::Register>),
        (0x0090 => _reserved1),
        /// Memory area: 2 kB packet buffer. Divided into 32 64-byte buffers.
        (0x0800 => pub(crate) buffer: [ReadWrite<u32>; 512]),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
    /// Common Interrupt Offsets
    pub(crate) INTR [
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
        POWERED OFFSET(15) NUMBITS(1) [],
        LINK_OUT_ERR OFFSET(16) NUMBITS(1) [],
    ],
    pub(crate) ALERT_TEST [
        FATAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    pub(crate) USBCTRL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        RESUME_LINK_ACTIVE OFFSET(1) NUMBITS(1) [],
        DEVICE_ADDRESS OFFSET(16) NUMBITS(7) [],
    ],
    pub(crate) EP_OUT_ENABLE [
        ENABLE_0 OFFSET(0) NUMBITS(1) [],
        ENABLE_1 OFFSET(1) NUMBITS(1) [],
        ENABLE_2 OFFSET(2) NUMBITS(1) [],
        ENABLE_3 OFFSET(3) NUMBITS(1) [],
        ENABLE_4 OFFSET(4) NUMBITS(1) [],
        ENABLE_5 OFFSET(5) NUMBITS(1) [],
        ENABLE_6 OFFSET(6) NUMBITS(1) [],
        ENABLE_7 OFFSET(7) NUMBITS(1) [],
        ENABLE_8 OFFSET(8) NUMBITS(1) [],
        ENABLE_9 OFFSET(9) NUMBITS(1) [],
        ENABLE_10 OFFSET(10) NUMBITS(1) [],
        ENABLE_11 OFFSET(11) NUMBITS(1) [],
    ],
    pub(crate) EP_IN_ENABLE [
        ENABLE_0 OFFSET(0) NUMBITS(1) [],
        ENABLE_1 OFFSET(1) NUMBITS(1) [],
        ENABLE_2 OFFSET(2) NUMBITS(1) [],
        ENABLE_3 OFFSET(3) NUMBITS(1) [],
        ENABLE_4 OFFSET(4) NUMBITS(1) [],
        ENABLE_5 OFFSET(5) NUMBITS(1) [],
        ENABLE_6 OFFSET(6) NUMBITS(1) [],
        ENABLE_7 OFFSET(7) NUMBITS(1) [],
        ENABLE_8 OFFSET(8) NUMBITS(1) [],
        ENABLE_9 OFFSET(9) NUMBITS(1) [],
        ENABLE_10 OFFSET(10) NUMBITS(1) [],
        ENABLE_11 OFFSET(11) NUMBITS(1) [],
    ],
    pub(crate) USBSTAT [
        FRAME OFFSET(0) NUMBITS(11) [],
        HOST_LOST OFFSET(11) NUMBITS(1) [],
        LINK_STATE OFFSET(12) NUMBITS(3) [
            DISCONNECTED = 0,
            POWERED = 1,
            POWERED_SUSPENDED = 2,
            ACTIVE = 3,
            SUSPENDED = 4,
            ACTIVE_NOSOF = 5,
            RESUMING = 6,
        ],
        SENSE OFFSET(15) NUMBITS(1) [],
        AV_DEPTH OFFSET(16) NUMBITS(4) [],
        AV_FULL OFFSET(23) NUMBITS(1) [],
        RX_DEPTH OFFSET(24) NUMBITS(4) [],
        RX_EMPTY OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) AVBUFFER [
        BUFFER OFFSET(0) NUMBITS(5) [],
    ],
    pub(crate) RXFIFO [
        BUFFER OFFSET(0) NUMBITS(5) [],
        SIZE OFFSET(8) NUMBITS(7) [],
        SETUP OFFSET(19) NUMBITS(1) [],
        EP OFFSET(20) NUMBITS(4) [],
    ],
    pub(crate) RXENABLE_SETUP [
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
    pub(crate) RXENABLE_OUT [
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
    pub(crate) SET_NAK_OUT [
        ENABLE_0 OFFSET(0) NUMBITS(1) [],
        ENABLE_1 OFFSET(1) NUMBITS(1) [],
        ENABLE_2 OFFSET(2) NUMBITS(1) [],
        ENABLE_3 OFFSET(3) NUMBITS(1) [],
        ENABLE_4 OFFSET(4) NUMBITS(1) [],
        ENABLE_5 OFFSET(5) NUMBITS(1) [],
        ENABLE_6 OFFSET(6) NUMBITS(1) [],
        ENABLE_7 OFFSET(7) NUMBITS(1) [],
        ENABLE_8 OFFSET(8) NUMBITS(1) [],
        ENABLE_9 OFFSET(9) NUMBITS(1) [],
        ENABLE_10 OFFSET(10) NUMBITS(1) [],
        ENABLE_11 OFFSET(11) NUMBITS(1) [],
    ],
    pub(crate) IN_SENT [
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
    pub(crate) OUT_STALL [
        ENDPOINT_0 OFFSET(0) NUMBITS(1) [],
        ENDPOINT_1 OFFSET(1) NUMBITS(1) [],
        ENDPOINT_2 OFFSET(2) NUMBITS(1) [],
        ENDPOINT_3 OFFSET(3) NUMBITS(1) [],
        ENDPOINT_4 OFFSET(4) NUMBITS(1) [],
        ENDPOINT_5 OFFSET(5) NUMBITS(1) [],
        ENDPOINT_6 OFFSET(6) NUMBITS(1) [],
        ENDPOINT_7 OFFSET(7) NUMBITS(1) [],
        ENDPOINT_8 OFFSET(8) NUMBITS(1) [],
        ENDPOINT_9 OFFSET(9) NUMBITS(1) [],
        ENDPOINT_10 OFFSET(10) NUMBITS(1) [],
        ENDPOINT_11 OFFSET(11) NUMBITS(1) [],
    ],
    pub(crate) IN_STALL [
        ENDPOINT_0 OFFSET(0) NUMBITS(1) [],
        ENDPOINT_1 OFFSET(1) NUMBITS(1) [],
        ENDPOINT_2 OFFSET(2) NUMBITS(1) [],
        ENDPOINT_3 OFFSET(3) NUMBITS(1) [],
        ENDPOINT_4 OFFSET(4) NUMBITS(1) [],
        ENDPOINT_5 OFFSET(5) NUMBITS(1) [],
        ENDPOINT_6 OFFSET(6) NUMBITS(1) [],
        ENDPOINT_7 OFFSET(7) NUMBITS(1) [],
        ENDPOINT_8 OFFSET(8) NUMBITS(1) [],
        ENDPOINT_9 OFFSET(9) NUMBITS(1) [],
        ENDPOINT_10 OFFSET(10) NUMBITS(1) [],
        ENDPOINT_11 OFFSET(11) NUMBITS(1) [],
    ],
    pub(crate) CONFIGIN [
        BUFFER_0 OFFSET(0) NUMBITS(5) [],
        SIZE_0 OFFSET(8) NUMBITS(7) [],
        PEND_0 OFFSET(30) NUMBITS(1) [],
        RDY_0 OFFSET(31) NUMBITS(1) [],
    ],
    pub(crate) OUT_ISO [
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
    pub(crate) IN_ISO [
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
    pub(crate) DATA_TOGGLE_CLEAR [
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
    pub(crate) PHY_PINS_SENSE [
        RX_DP_I OFFSET(0) NUMBITS(1) [],
        RX_DN_I OFFSET(1) NUMBITS(1) [],
        RX_D_I OFFSET(2) NUMBITS(1) [],
        TX_DP_O OFFSET(8) NUMBITS(1) [],
        TX_DN_O OFFSET(9) NUMBITS(1) [],
        TX_D_O OFFSET(10) NUMBITS(1) [],
        TX_SE0_O OFFSET(11) NUMBITS(1) [],
        TX_OE_O OFFSET(12) NUMBITS(1) [],
        PWR_SENSE OFFSET(16) NUMBITS(1) [],
    ],
    pub(crate) PHY_PINS_DRIVE [
        DP_O OFFSET(0) NUMBITS(1) [],
        DN_O OFFSET(1) NUMBITS(1) [],
        D_O OFFSET(2) NUMBITS(1) [],
        SE0_O OFFSET(3) NUMBITS(1) [],
        OE_O OFFSET(4) NUMBITS(1) [],
        RX_ENABLE_O OFFSET(5) NUMBITS(1) [],
        DP_PULLUP_EN_O OFFSET(6) NUMBITS(1) [],
        DN_PULLUP_EN_O OFFSET(7) NUMBITS(1) [],
        EN OFFSET(16) NUMBITS(1) [],
    ],
    pub(crate) PHY_CONFIG [
        USE_DIFF_RCVR OFFSET(0) NUMBITS(1) [],
        TX_USE_D_SE0 OFFSET(1) NUMBITS(1) [],
        EOP_SINGLE_BIT OFFSET(2) NUMBITS(1) [],
        PINFLIP OFFSET(5) NUMBITS(1) [],
        USB_REF_DISABLE OFFSET(6) NUMBITS(1) [],
        TX_OSC_TEST_MODE OFFSET(7) NUMBITS(1) [],
    ],
    pub(crate) WAKE_CONTROL [
        SUSPEND_REQ OFFSET(0) NUMBITS(1) [],
        WAKE_ACK OFFSET(1) NUMBITS(1) [],
    ],
    pub(crate) WAKE_EVENTS [
        MODULE_ACTIVE OFFSET(0) NUMBITS(1) [],
        DISCONNECTED OFFSET(8) NUMBITS(1) [],
        BUS_RESET OFFSET(9) NUMBITS(1) [],
    ],
];

// End generated register constants for usbdev
