// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Universal Serial Bus Device for Raspberry Pi Pico

//! Authors: Cosmin Radu <cosmin.radu@wyliodrin.com>
//!          Teodora Miu <teodora.miu01@gmail.com>

use crate::gpio::RPGpioPin;
use core::cell::Cell;
use kernel::hil;
use kernel::hil::usb::TransferType;
use kernel::utilities::cells::{OptionalCell, VolatileCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

macro_rules! internal_err {
    [ $( $arg:expr ),+ ] => {
        panic!($( $arg ),+)
    };
}

register_structs! {
    Ep_ctrl {
        (0x00 => ep_in_ctrl: ReadWrite<u32, EP_CONTROL::Register>),
        (0x04 => ep_out_ctrl: ReadWrite<u32, EP_CONTROL::Register>),
        (0x08 => @END),
    }
}

register_structs! {
    Ep_buf_ctrl {
        (0x00 => ep_in_buf_ctrl: ReadWrite<u32, EP_BUFFER_CONTROL::Register>),
        (0x04 => ep_out_buf_ctrl: ReadWrite<u32, EP_BUFFER_CONTROL::Register>),
        (0x08 => @END),
    }
}

register_structs! {
    /// USB FS/LS controller device registers
    Usbctrl_DPSRAM {
        /// Device address and endpoint control
        (0x00 => setup_h: ReadWrite<u32, SETUP_H::Register>),
        (0x04 => setup_l: ReadWrite<u32, SETUP_L::Register>),
        (0x08 => ep_ctrl: [Ep_ctrl; 15]),
        (0x80 => ep_buf_ctrl: [Ep_buf_ctrl; 16]),
        (0x100 => ep0_buffer0: [VolatileCell<u8>; 0x40]),
        (0x140 => optional_ep0_buffer0: [VolatileCell<u8>; 0x40]),
        (0x180 => buffers: [VolatileCell<u8>; 4096-0x180]),
        (0x1000 => @END),
    }
}

register_structs! {
    /// USB FS/LS controller device registers
    Usbctrl_RegsRegisters {
        /// Device address and endpoint control
        (0x000 => addr_endp: ReadWrite<u32, ADDR_ENDP::Register>),
        /// Interrupt endpoint 1. Only valid for HOST mode.
        (0x004 => addr_endp1: ReadWrite<u32, ADDR_ENDP1::Register>),
        /// Interrupt endpoint 2. Only valid for HOST mode.
        (0x008 => addr_endp2: ReadWrite<u32, ADDR_ENDP2::Register>),
        /// Interrupt endpoint 3. Only valid for HOST mode.
        (0x00C => addr_endp3: ReadWrite<u32, ADDR_ENDP3::Register>),
        /// Interrupt endpoint 4. Only valid for HOST mode.
        (0x010 => addr_endp4: ReadWrite<u32, ADDR_ENDP4::Register>),
        /// Interrupt endpoint 5. Only valid for HOST mode.
        (0x014 => addr_endp5: ReadWrite<u32, ADDR_ENDP5::Register>),
        /// Interrupt endpoint 6. Only valid for HOST mode.
        (0x018 => addr_endp6: ReadWrite<u32, ADDR_ENDP6::Register>),
        /// Interrupt endpoint 7. Only valid for HOST mode.
        (0x01C => addr_endp7: ReadWrite<u32, ADDR_ENDP7::Register>),
        /// Interrupt endpoint 8. Only valid for HOST mode.
        (0x020 => addr_endp8: ReadWrite<u32, ADDR_ENDP8::Register>),
        /// Interrupt endpoint 9. Only valid for HOST mode.
        (0x024 => addr_endp9: ReadWrite<u32, ADDR_ENDP9::Register>),
        /// Interrupt endpoint 10. Only valid for HOST mode.
        (0x028 => addr_endp10: ReadWrite<u32, ADDR_ENDP10::Register>),
        /// Interrupt endpoint 11. Only valid for HOST mode.
        (0x02C => addr_endp11: ReadWrite<u32, ADDR_ENDP11::Register>),
        /// Interrupt endpoint 12. Only valid for HOST mode.
        (0x030 => addr_endp12: ReadWrite<u32, ADDR_ENDP12::Register>),
        /// Interrupt endpoint 13. Only valid for HOST mode.
        (0x034 => addr_endp13: ReadWrite<u32, ADDR_ENDP13::Register>),
        /// Interrupt endpoint 14. Only valid for HOST mode.
        (0x038 => addr_endp14: ReadWrite<u32, ADDR_ENDP14::Register>),
        /// Interrupt endpoint 15. Only valid for HOST mode.
        (0x03C => addr_endp15: ReadWrite<u32, ADDR_ENDP15::Register>),
        /// Main control register
        (0x040 => main_ctrl: ReadWrite<u32, MAIN_CTRL::Register>),
        /// Set the SOF (Start of Frame) frame number in the host controller.
        /// The SOF packet is sent every 1ms and the host will increment the
        /// frame number by 1 each time.
        (0x044 => sof_wr: ReadWrite<u32, SOF_WR::Register>),
        /// Read the last SOF (Start of Frame) frame number seen. In device
        /// mode the last SOF received from the host. In host mode the last
        /// SOF sent by the host.
        (0x048 => sof_rd: ReadWrite<u32, SOF_RD::Register>),
        /// SIE control register
        (0x04C => sie_ctrl: ReadWrite<u32, SIE_CTRL::Register>),
        /// SIE status register
        (0x050 => sie_status: ReadWrite<u32, SIE_STATUS::Register>),
        /// interrupt endpoint control register
        (0x054 => int_ep_ctrl: ReadWrite<u32, INT_EP_CTRL::Register>),
        /// Buffer status register. A bit set here indicates that a buffer has
        /// completed on the endpoint (if the buffer interrupt is enabled). It
        /// is possible for 2 buffers to be completed, so clearing the buffer
        /// status bit may instantly re set it on the next clock cycle.
        (0x058 => buff_status: ReadWrite<u32, BUFF_STATUS::Register>),
        /// Which of the double buffers should be handled. Only valid if
        /// using an interrupt per buffer (i.e. not per 2 buffers). Not valid for
        /// host interrupt endpoint polling because they are only single
        /// buffered.
        (0x05C => buff_cpu_should_handle: ReadWrite<u32, BUFF_CPU_SHOULD_HANDLE::Register>),
        /// Device only: Can be set to ignore the buffer control register for
        /// this endpoint in case you would like to revoke a buffer. A NAK
        /// will be sent for every access to the endpoint until this bit is
        /// cleared. A corresponding bit in EP_ABORT_DONE is set when it is safe
        /// to modify the buffer control register.
        (0x060 => ep_abort: ReadWrite<u32, EP_ABORT::Register>),
        /// Device only: Used in conjunction with EP_ABORT. Set once an
        /// endpoint is idle so the programmer knows it is safe to modify the
        /// buffer control register.
        (0x064 => ep_abort_done: ReadWrite<u32, EP_ABORT_DONE::Register>),
        /// Device: this bit must be set in conjunction with the STALL bit in the
        /// buffer control register to send a STALL on EP0. The device
        /// controller clears these bits when a SETUP packet is received
        /// because the USB spec requires that a STALL condition is cleared
        /// when a SETUP packet is received.
        (0x068 => ep_stall_arm: ReadWrite<u32, EP_STALL_ARM::Register>),
        /// Used by the host controller. Sets the wait time in microseconds
        /// before trying again if the device replies with a NAK.
        (0x06C => nak_poll: ReadWrite<u32, NAK_POLL::Register>),
        /// Device: bits are set when the IRQ_ON_NAK or IRQ_ON_STALL bits are
        /// set. For EP0 this comes from SIE_CTRL. For all other endpoints it
        /// comes from the endpoint control register.
        (0x070 => ep_status_stall_nak: ReadWrite<u32, EP_STATUS_STALL_NAK::Register>),
        /// Where to connect the USB controller. Should be to_phy by default.
        (0x074 => usb_muxing: ReadWrite<u32, USB_MUXING::Register>),
        /// Overrides for the power signals in the event that the VBUS
        /// signals are not hooked up to GPIO. Set the value of the override
        /// and then the override enable to switch over to the override value.
        (0x078 => usb_pwr: ReadWrite<u32, USB_PWR::Register>),
        /// This register allows for direct control of the USB phy. Use in
        /// conjunction with usbphy_direct_override register to enable each
        /// override bit.
        (0x07C => usbphy_direct: ReadWrite<u32, USBPHY_DIRECT::Register>),
        /// Override enable for each control in usbphy_direct
        (0x080 => usbphy_direct_override: ReadWrite<u32, USBPHY_DIRECT_OVERRIDE::Register>),
        /// Used to adjust trim values of USB phy pull down resistors.
        (0x084 => usbphy_trim: ReadWrite<u32, USBPHY_TRIM::Register>),
        (0x088 => _reserved0),
        /// Raw Interrupts
        (0x08C => intr: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable
        (0x090 => inte: ReadWrite<u32, INTE::Register>),
        /// Interrupt Force
        (0x094 => intf: ReadWrite<u32, INTF::Register>),
        /// Interrupt status after masking & forcing
        (0x098 => ints: ReadWrite<u32, INTS::Register>),
        (0x09C => @END),
    }
}
register_bitfields![u32,
ADDR_ENDP [
    /// Device endpoint to send data to. Only valid for HOST mode.
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// In device mode, the address that the device should
    /// respond to. Set in response to a SET_ADDR setup packet
    /// from the host. In host mode set to the address of the
    /// device to communicate with.
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP1 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP2 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP3 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP4 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP5 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP6 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP7 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP8 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP9 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP10 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP11 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP12 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP13 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP14 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
ADDR_ENDP15 [
    /// Interrupt EP requires preamble (is a low speed device on a full speed hub)
    INTEP_PREAMBLE OFFSET(26) NUMBITS(1) [],
    /// Direction of the interrupt endpoint. In=0, Out=1
    INTEP_DIR OFFSET(25) NUMBITS(1) [],
    /// Endpoint number of the interrupt endpoint
    ENDPOINT OFFSET(16) NUMBITS(4) [],
    /// Device address
    ADDRESS OFFSET(0) NUMBITS(7) []
],
MAIN_CTRL [
    /// Reduced timings for simulation
    SIM_TIMING OFFSET(31) NUMBITS(1) [],
    /// Device mode = 0, Host mode = 1
    HOST_NDEVICE OFFSET(1) NUMBITS(1) [],
    /// Enable controller
    CONTROLLER_EN OFFSET(0) NUMBITS(1) []
],
SOF_WR [

    COUNT OFFSET(0) NUMBITS(11) []
],
SOF_RD [

    COUNT OFFSET(0) NUMBITS(11) []
],
SIE_CTRL [
    /// Device: Set bit in EP_STATUS_STALL_NAK when EP0 sends a STALL
    EP0_INT_STALL OFFSET(31) NUMBITS(1) [],
    /// Device: EP0 single buffered = 0, double buffered = 1
    EP0_DOUBLE_BUF OFFSET(30) NUMBITS(1) [],
    /// Device: Set bit in BUFF_STATUS for every buffer completed on EP0
    EP0_INT_1BUF OFFSET(29) NUMBITS(1) [],
    /// Device: Set bit in BUFF_STATUS for every 2 buffers completed on EP0
    EP0_INT_2BUF OFFSET(28) NUMBITS(1) [],
    /// Device: Set bit in EP_STATUS_STALL_NAK when EP0 sends a NAK
    EP0_INT_NAK OFFSET(27) NUMBITS(1) [],
    /// Direct bus drive enable
    DIRECT_EN OFFSET(26) NUMBITS(1) [],
    /// Direct control of DP
    DIRECT_DP OFFSET(25) NUMBITS(1) [],
    /// Direct control of DM
    DIRECT_DM OFFSET(24) NUMBITS(1) [],
    /// Power down bus transceiver
    TRANSCEIVER_PD OFFSET(18) NUMBITS(1) [],
    /// Device: Pull-up strength (0=1K2, 1=2k3)
    RPU_OPT OFFSET(17) NUMBITS(1) [],
    /// Device: Enable pull up resistor
    PULLUP_EN OFFSET(16) NUMBITS(1) [],
    /// Host: Enable pull down resistors
    PULLDOWN_EN OFFSET(15) NUMBITS(1) [],
    /// Host: Reset bus
    RESET_BUS OFFSET(13) NUMBITS(1) [],
    /// Device: Remote wakeup. Device can initiate its own resume after suspend.
    RESUME OFFSET(12) NUMBITS(1) [],
    /// Host: Enable VBUS
    VBUS_EN OFFSET(11) NUMBITS(1) [],
    /// Host: Enable keep alive packet (for low speed bus)
    KEEP_ALIVE_EN OFFSET(10) NUMBITS(1) [],
    /// Host: Enable SOF generation (for full speed bus)
    SOF_EN OFFSET(9) NUMBITS(1) [],
    /// Host: Delay packet(s) until after SOF
    SOF_SYNC OFFSET(8) NUMBITS(1) [],
    /// Host: Preable enable for LS device on FS hub
    PREAMBLE_EN OFFSET(6) NUMBITS(1) [],
    /// Host: Stop transaction
    STOP_TRANS OFFSET(4) NUMBITS(1) [],
    /// Host: Receive transaction (IN to host)
    RECEIVE_DATA OFFSET(3) NUMBITS(1) [],
    /// Host: Send transaction (OUT from host)
    SEND_DATA OFFSET(2) NUMBITS(1) [],
    /// Host: Send Setup packet
    SEND_SETUP OFFSET(1) NUMBITS(1) [],
    /// Host: Start transaction
    START_TRANS OFFSET(0) NUMBITS(1) []
],
SIE_STATUS [
    /// Data Sequence Error.
    /// The device can raise a sequence error in the following conditions:
    /// * A SETUP packet is received followed by a DATA1 packet (data phase
    /// should always be DATA0) * An OUT packet is received from the host but
    /// doesn't match the data pid in the buffer control register read from DPSRAM
    /// The host can raise a data sequence error in the following conditions:
    /// * An IN packet from the device has the wrong data PID
    DATA_SEQ_ERROR OFFSET(31) NUMBITS(1) [],
    /// ACK received. Raised by both host and device.
    ACK_REC OFFSET(30) NUMBITS(1) [],
    /// Host: STALL received
    STALL_REC OFFSET(29) NUMBITS(1) [],
    /// Host: NAK received
    NAK_REC OFFSET(28) NUMBITS(1) [],
    /// RX timeout is raised by both the host and device if an ACK
    /// is not received in the maximum time specified by the USB
    /// spec.
    RX_TIMEOUT OFFSET(27) NUMBITS(1) [],
    /// RX overflow is raised by the Serial RX engine if the incoming data is too fast.
    RX_OVERFLOW OFFSET(26) NUMBITS(1) [],
    /// Bit Stuff Error. Raised by the Serial RX engine.
    BIT_STUFF_ERROR OFFSET(25) NUMBITS(1) [],
    /// CRC Error. Raised by the Serial RX engine.
    CRC_ERROR OFFSET(24) NUMBITS(1) [],
    /// Device: bus reset received
    BUS_RESET OFFSET(19) NUMBITS(1) [],
    /// Transaction complete.
    /// Raised by device if:
    /// * An IN or OUT packet is sent with the LAST_BUFF bit set in
    /// the buffer control register
    /// Raised by host if:
    /// * A setup packet is sent when no data in or data out
    /// transaction follows * An IN packet is received and the
    /// LAST_BUFF bit is set in the buffer control register * An IN
    /// packet is received with zero length * An OUT packet is
    /// sent and the LAST_BUFF bit is set
    TRANS_COMPLETE OFFSET(18) NUMBITS(1) [],
    /// Device: Setup packet received
    SETUP_REC OFFSET(17) NUMBITS(1) [],
    /// Device: connected
    CONNECTED OFFSET(16) NUMBITS(1) [],
    /// Host: Device has initiated a remote resume. Device: host has initiated a resume.
    RESUME OFFSET(11) NUMBITS(1) [],
    /// VBUS over current detected
    VBUS_OVER_CURR OFFSET(10) NUMBITS(1) [],
    /// Host: device speed. Disconnected = 00, LS = 01, FS = 10
    SPEED OFFSET(8) NUMBITS(2) [],
    /// Bus in suspended state. Valid for device and host. Host
    /// and device will go into suspend if neither Keep Alive / SOF
    /// frames are enabled.
    SUSPENDED OFFSET(4) NUMBITS(1) [],
    /// USB bus line state
    LINE_STATE OFFSET(2) NUMBITS(2) [],
    /// Device: VBUS Detected
    VBUS_DETECTED OFFSET(0) NUMBITS(1) []
],
INT_EP_CTRL [
    /// Host: Enable interrupt endpoint 1 -> 15
    INT_EP_ACTIVE OFFSET(1) NUMBITS(15) []
],
BUFF_STATUS [

    EP15_OUT OFFSET(31) NUMBITS(1) [],

    EP15_IN OFFSET(30) NUMBITS(1) [],

    EP14_OUT OFFSET(29) NUMBITS(1) [],

    EP14_IN OFFSET(28) NUMBITS(1) [],

    EP13_OUT OFFSET(27) NUMBITS(1) [],

    EP13_IN OFFSET(26) NUMBITS(1) [],

    EP12_OUT OFFSET(25) NUMBITS(1) [],

    EP12_IN OFFSET(24) NUMBITS(1) [],

    EP11_OUT OFFSET(23) NUMBITS(1) [],

    EP11_IN OFFSET(22) NUMBITS(1) [],

    EP10_OUT OFFSET(21) NUMBITS(1) [],

    EP10_IN OFFSET(20) NUMBITS(1) [],

    EP9_OUT OFFSET(19) NUMBITS(1) [],

    EP9_IN OFFSET(18) NUMBITS(1) [],

    EP8_OUT OFFSET(17) NUMBITS(1) [],

    EP8_IN OFFSET(16) NUMBITS(1) [],

    EP7_OUT OFFSET(15) NUMBITS(1) [],

    EP7_IN OFFSET(14) NUMBITS(1) [],

    EP6_OUT OFFSET(13) NUMBITS(1) [],

    EP6_IN OFFSET(12) NUMBITS(1) [],

    EP5_OUT OFFSET(11) NUMBITS(1) [],

    EP5_IN OFFSET(10) NUMBITS(1) [],

    EP4_OUT OFFSET(9) NUMBITS(1) [],

    EP4_IN OFFSET(8) NUMBITS(1) [],

    EP3_OUT OFFSET(7) NUMBITS(1) [],

    EP3_IN OFFSET(6) NUMBITS(1) [],

    EP2_OUT OFFSET(5) NUMBITS(1) [],

    EP2_IN OFFSET(4) NUMBITS(1) [],

    EP1_OUT OFFSET(3) NUMBITS(1) [],

    EP1_IN OFFSET(2) NUMBITS(1) [],

    EP0_OUT OFFSET(1) NUMBITS(1) [],

    EP0_IN OFFSET(0) NUMBITS(1) []
],
BUFF_CPU_SHOULD_HANDLE [

    EP15_OUT OFFSET(31) NUMBITS(1) [],

    EP15_IN OFFSET(30) NUMBITS(1) [],

    EP14_OUT OFFSET(29) NUMBITS(1) [],

    EP14_IN OFFSET(28) NUMBITS(1) [],

    EP13_OUT OFFSET(27) NUMBITS(1) [],

    EP13_IN OFFSET(26) NUMBITS(1) [],

    EP12_OUT OFFSET(25) NUMBITS(1) [],

    EP12_IN OFFSET(24) NUMBITS(1) [],

    EP11_OUT OFFSET(23) NUMBITS(1) [],

    EP11_IN OFFSET(22) NUMBITS(1) [],

    EP10_OUT OFFSET(21) NUMBITS(1) [],

    EP10_IN OFFSET(20) NUMBITS(1) [],

    EP9_OUT OFFSET(19) NUMBITS(1) [],

    EP9_IN OFFSET(18) NUMBITS(1) [],

    EP8_OUT OFFSET(17) NUMBITS(1) [],

    EP8_IN OFFSET(16) NUMBITS(1) [],

    EP7_OUT OFFSET(15) NUMBITS(1) [],

    EP7_IN OFFSET(14) NUMBITS(1) [],

    EP6_OUT OFFSET(13) NUMBITS(1) [],

    EP6_IN OFFSET(12) NUMBITS(1) [],

    EP5_OUT OFFSET(11) NUMBITS(1) [],

    EP5_IN OFFSET(10) NUMBITS(1) [],

    EP4_OUT OFFSET(9) NUMBITS(1) [],

    EP4_IN OFFSET(8) NUMBITS(1) [],

    EP3_OUT OFFSET(7) NUMBITS(1) [],

    EP3_IN OFFSET(6) NUMBITS(1) [],

    EP2_OUT OFFSET(5) NUMBITS(1) [],

    EP2_IN OFFSET(4) NUMBITS(1) [],

    EP1_OUT OFFSET(3) NUMBITS(1) [],

    EP1_IN OFFSET(2) NUMBITS(1) [],

    EP0_OUT OFFSET(1) NUMBITS(1) [],

    EP0_IN OFFSET(0) NUMBITS(1) []
],
EP_ABORT [

    EP15_OUT OFFSET(31) NUMBITS(1) [],

    EP15_IN OFFSET(30) NUMBITS(1) [],

    EP14_OUT OFFSET(29) NUMBITS(1) [],

    EP14_IN OFFSET(28) NUMBITS(1) [],

    EP13_OUT OFFSET(27) NUMBITS(1) [],

    EP13_IN OFFSET(26) NUMBITS(1) [],

    EP12_OUT OFFSET(25) NUMBITS(1) [],

    EP12_IN OFFSET(24) NUMBITS(1) [],

    EP11_OUT OFFSET(23) NUMBITS(1) [],

    EP11_IN OFFSET(22) NUMBITS(1) [],

    EP10_OUT OFFSET(21) NUMBITS(1) [],

    EP10_IN OFFSET(20) NUMBITS(1) [],

    EP9_OUT OFFSET(19) NUMBITS(1) [],

    EP9_IN OFFSET(18) NUMBITS(1) [],

    EP8_OUT OFFSET(17) NUMBITS(1) [],

    EP8_IN OFFSET(16) NUMBITS(1) [],

    EP7_OUT OFFSET(15) NUMBITS(1) [],

    EP7_IN OFFSET(14) NUMBITS(1) [],

    EP6_OUT OFFSET(13) NUMBITS(1) [],

    EP6_IN OFFSET(12) NUMBITS(1) [],

    EP5_OUT OFFSET(11) NUMBITS(1) [],

    EP5_IN OFFSET(10) NUMBITS(1) [],

    EP4_OUT OFFSET(9) NUMBITS(1) [],

    EP4_IN OFFSET(8) NUMBITS(1) [],

    EP3_OUT OFFSET(7) NUMBITS(1) [],

    EP3_IN OFFSET(6) NUMBITS(1) [],

    EP2_OUT OFFSET(5) NUMBITS(1) [],

    EP2_IN OFFSET(4) NUMBITS(1) [],

    EP1_OUT OFFSET(3) NUMBITS(1) [],

    EP1_IN OFFSET(2) NUMBITS(1) [],

    EP0_OUT OFFSET(1) NUMBITS(1) [],

    EP0_IN OFFSET(0) NUMBITS(1) []
],
EP_ABORT_DONE [

    EP15_OUT OFFSET(31) NUMBITS(1) [],

    EP15_IN OFFSET(30) NUMBITS(1) [],

    EP14_OUT OFFSET(29) NUMBITS(1) [],

    EP14_IN OFFSET(28) NUMBITS(1) [],

    EP13_OUT OFFSET(27) NUMBITS(1) [],

    EP13_IN OFFSET(26) NUMBITS(1) [],

    EP12_OUT OFFSET(25) NUMBITS(1) [],

    EP12_IN OFFSET(24) NUMBITS(1) [],

    EP11_OUT OFFSET(23) NUMBITS(1) [],

    EP11_IN OFFSET(22) NUMBITS(1) [],

    EP10_OUT OFFSET(21) NUMBITS(1) [],

    EP10_IN OFFSET(20) NUMBITS(1) [],

    EP9_OUT OFFSET(19) NUMBITS(1) [],

    EP9_IN OFFSET(18) NUMBITS(1) [],

    EP8_OUT OFFSET(17) NUMBITS(1) [],

    EP8_IN OFFSET(16) NUMBITS(1) [],

    EP7_OUT OFFSET(15) NUMBITS(1) [],

    EP7_IN OFFSET(14) NUMBITS(1) [],

    EP6_OUT OFFSET(13) NUMBITS(1) [],

    EP6_IN OFFSET(12) NUMBITS(1) [],

    EP5_OUT OFFSET(11) NUMBITS(1) [],

    EP5_IN OFFSET(10) NUMBITS(1) [],

    EP4_OUT OFFSET(9) NUMBITS(1) [],

    EP4_IN OFFSET(8) NUMBITS(1) [],

    EP3_OUT OFFSET(7) NUMBITS(1) [],

    EP3_IN OFFSET(6) NUMBITS(1) [],

    EP2_OUT OFFSET(5) NUMBITS(1) [],

    EP2_IN OFFSET(4) NUMBITS(1) [],

    EP1_OUT OFFSET(3) NUMBITS(1) [],

    EP1_IN OFFSET(2) NUMBITS(1) [],

    EP0_OUT OFFSET(1) NUMBITS(1) [],

    EP0_IN OFFSET(0) NUMBITS(1) []
],
EP_STALL_ARM [

    EP0_OUT OFFSET(1) NUMBITS(1) [],

    EP0_IN OFFSET(0) NUMBITS(1) []
],
NAK_POLL [
    /// NAK polling interval for a full speed device
    DELAY_FS OFFSET(16) NUMBITS(10) [],
    /// NAK polling interval for a low speed device
    DELAY_LS OFFSET(0) NUMBITS(10) []
],
EP_STATUS_STALL_NAK [

    EP15_OUT OFFSET(31) NUMBITS(1) [],

    EP15_IN OFFSET(30) NUMBITS(1) [],

    EP14_OUT OFFSET(29) NUMBITS(1) [],

    EP14_IN OFFSET(28) NUMBITS(1) [],

    EP13_OUT OFFSET(27) NUMBITS(1) [],

    EP13_IN OFFSET(26) NUMBITS(1) [],

    EP12_OUT OFFSET(25) NUMBITS(1) [],

    EP12_IN OFFSET(24) NUMBITS(1) [],

    EP11_OUT OFFSET(23) NUMBITS(1) [],

    EP11_IN OFFSET(22) NUMBITS(1) [],

    EP10_OUT OFFSET(21) NUMBITS(1) [],

    EP10_IN OFFSET(20) NUMBITS(1) [],

    EP9_OUT OFFSET(19) NUMBITS(1) [],

    EP9_IN OFFSET(18) NUMBITS(1) [],

    EP8_OUT OFFSET(17) NUMBITS(1) [],

    EP8_IN OFFSET(16) NUMBITS(1) [],

    EP7_OUT OFFSET(15) NUMBITS(1) [],

    EP7_IN OFFSET(14) NUMBITS(1) [],

    EP6_OUT OFFSET(13) NUMBITS(1) [],

    EP6_IN OFFSET(12) NUMBITS(1) [],

    EP5_OUT OFFSET(11) NUMBITS(1) [],

    EP5_IN OFFSET(10) NUMBITS(1) [],

    EP4_OUT OFFSET(9) NUMBITS(1) [],

    EP4_IN OFFSET(8) NUMBITS(1) [],

    EP3_OUT OFFSET(7) NUMBITS(1) [],

    EP3_IN OFFSET(6) NUMBITS(1) [],

    EP2_OUT OFFSET(5) NUMBITS(1) [],

    EP2_IN OFFSET(4) NUMBITS(1) [],

    EP1_OUT OFFSET(3) NUMBITS(1) [],

    EP1_IN OFFSET(2) NUMBITS(1) [],

    EP0_OUT OFFSET(1) NUMBITS(1) [],

    EP0_IN OFFSET(0) NUMBITS(1) []
],
USB_MUXING [

    SOFTCON OFFSET(3) NUMBITS(1) [],

    TO_DIGITAL_PAD OFFSET(2) NUMBITS(1) [],

    TO_EXTPHY OFFSET(1) NUMBITS(1) [],

    TO_PHY OFFSET(0) NUMBITS(1) []
],
USB_PWR [

    OVERCURR_DETECT_EN OFFSET(5) NUMBITS(1) [],

    OVERCURR_DETECT OFFSET(4) NUMBITS(1) [],

    VBUS_DETECT_OVERRIDE_EN OFFSET(3) NUMBITS(1) [],

    VBUS_DETECT OFFSET(2) NUMBITS(1) [],

    VBUS_EN_OVERRIDE_EN OFFSET(1) NUMBITS(1) [],

    VBUS_EN OFFSET(0) NUMBITS(1) []
],
USBPHY_DIRECT [
    /// DM over voltage
    DM_OVV OFFSET(22) NUMBITS(1) [],
    /// DP over voltage
    DP_OVV OFFSET(21) NUMBITS(1) [],
    /// DM overcurrent
    DM_OVCN OFFSET(20) NUMBITS(1) [],
    /// DP overcurrent
    DP_OVCN OFFSET(19) NUMBITS(1) [],
    /// DPM pin state
    RX_DM OFFSET(18) NUMBITS(1) [],
    /// DPP pin state
    RX_DP OFFSET(17) NUMBITS(1) [],
    /// Differential RX
    RX_DD OFFSET(16) NUMBITS(1) [],
    /// TX_DIFFMODE=0: Single ended mode
    /// TX_DIFFMODE=1: Differential drive mode (TX_DM, TX_DM_OE ignored)
    TX_DIFFMODE OFFSET(15) NUMBITS(1) [],
    /// TX_FSSLEW=0: Low speed slew rate
    /// TX_FSSLEW=1: Full speed slew rate
    TX_FSSLEW OFFSET(14) NUMBITS(1) [],
    /// TX power down override (if override enable is set). 1 = powered down.
    TX_PD OFFSET(13) NUMBITS(1) [],
    /// RX power down override (if override enable is set). 1 = powered down.
    RX_PD OFFSET(12) NUMBITS(1) [],
    /// Output data. TX_DIFFMODE=1, Ignored
    /// TX_DIFFMODE=0, Drives DPM only. TX_DM_OE=1 to
    /// enable drive. DPM=TX_DM
    TX_DM OFFSET(11) NUMBITS(1) [],
    /// Output data. If TX_DIFFMODE=1, Drives DPP/DPM diff
    /// pair. TX_DP_OE=1 to enable drive. DPP=TX_DP,
    /// DPM=~TX_DP
    /// If TX_DIFFMODE=0, Drives DPP only. TX_DP_OE=1 to
    /// enable drive. DPP=TX_DP
    TX_DP OFFSET(10) NUMBITS(1) [],
    /// Output enable. If TX_DIFFMODE=1, Ignored.
    /// If TX_DIFFMODE=0, OE for DPM only. 0 - DPM in Hi-Z
    /// state; 1 - DPM driving
    TX_DM_OE OFFSET(9) NUMBITS(1) [],
    /// Output enable. If TX_DIFFMODE=1, Ignored.
    /// If TX_DIFFMODE=0, OE for DPM only. 0 - DPM in Hi-Z
    /// state; 1 - DPM driving
    TX_DP_OE OFFSET(8) NUMBITS(1) [],
    /// DM pull down enable
    DM_PULLDN_EN OFFSET(6) NUMBITS(1) [],
    /// DM pull up enable
    DM_PULLUP_EN OFFSET(5) NUMBITS(1) [],
    /// Enable the second DM pull up resistor. 0 - Pull = Rpu2; 1 - Pull = Rpu1 + Rpu2
    DM_PULLUP_HISEL OFFSET(4) NUMBITS(1) [],
    /// DP pull down enable
    DP_PULLDN_EN OFFSET(2) NUMBITS(1) [],
    /// DP pull up enable
    DP_PULLUP_EN OFFSET(1) NUMBITS(1) [],
    /// Enable the second DP pull up resistor. 0 - Pull = Rpu2; 1 - Pull = Rpu1 + Rpu2
    DP_PULLUP_HISEL OFFSET(0) NUMBITS(1) []
],
USBPHY_DIRECT_OVERRIDE [

    TX_DIFFMODE_OVERRIDE_EN OFFSET(15) NUMBITS(1) [],

    DM_PULLUP_OVERRIDE_EN OFFSET(12) NUMBITS(1) [],

    TX_FSSLEW_OVERRIDE_EN OFFSET(11) NUMBITS(1) [],

    TX_PD_OVERRIDE_EN OFFSET(10) NUMBITS(1) [],

    RX_PD_OVERRIDE_EN OFFSET(9) NUMBITS(1) [],

    TX_DM_OVERRIDE_EN OFFSET(8) NUMBITS(1) [],

    TX_DP_OVERRIDE_EN OFFSET(7) NUMBITS(1) [],

    TX_DM_OE_OVERRIDE_EN OFFSET(6) NUMBITS(1) [],

    TX_DP_OE_OVERRIDE_EN OFFSET(5) NUMBITS(1) [],

    DM_PULLDN_EN_OVERRIDE_EN OFFSET(4) NUMBITS(1) [],

    DP_PULLDN_EN_OVERRIDE_EN OFFSET(3) NUMBITS(1) [],

    DP_PULLUP_EN_OVERRIDE_EN OFFSET(2) NUMBITS(1) [],

    DM_PULLUP_HISEL_OVERRIDE_EN OFFSET(1) NUMBITS(1) [],

    DP_PULLUP_HISEL_OVERRIDE_EN OFFSET(0) NUMBITS(1) []
],
USBPHY_TRIM [
    /// Value to drive to USB PHY
    /// DM pulldown resistor trim control
    // Experimental data suggests that the reset value will work,
    // but this register allows adjustment if required
    DM_PULLDN_TRIM OFFSET(8) NUMBITS(5) [],
    /// Value to drive to USB PHY
    /// DP pulldown resistor trim control
    /// Experimental data suggests that the reset value will work,
    /// but this register allows adjustment if required
    DP_PULLDN_TRIM OFFSET(0) NUMBITS(5) []
],
INTR [
    /// Raised when any bit in EP_STATUS_STALL_NAK is set.
    /// Clear by clearing all bits in EP_STATUS_STALL_NAK.
    EP_STALL_NAK OFFSET(19) NUMBITS(1) [],
    /// Raised when any bit in ABORT_DONE is set. Clear by
    /// clearing all bits in ABORT_DONE.
    ABORT_DONE OFFSET(18) NUMBITS(1) [],
    /// Set every time the device receives a SOF (Start of Frame)
    /// packet. Cleared by reading SOF_RD
    DEV_SOF OFFSET(17) NUMBITS(1) [],
    /// Device. Source: SIE_STATUS.SETUP_REC
    SETUP_REQ OFFSET(16) NUMBITS(1) [],
    /// Set when the device receives a resume from the host.
    /// Cleared by writing to SIE_STATUS.RESUME
    DEV_RESUME_FROM_HOST OFFSET(15) NUMBITS(1) [],
    /// Set when the device suspend state changes. Cleared by
    /// writing to SIE_STATUS.SUSPENDED
    DEV_SUSPEND OFFSET(14) NUMBITS(1) [],
    /// Set when the device connection state changes. Cleared by
    /// writing to SIE_STATUS.CONNECTED
    DEV_CONN_DIS OFFSET(13) NUMBITS(1) [],
    /// Source: SIE_STATUS.BUS_RESET
    BUS_RESET OFFSET(12) NUMBITS(1) [],
    /// Source: SIE_STATUS.VBUS_DETECT
    VBUS_DETECT OFFSET(11) NUMBITS(1) [],
    /// Source: SIE_STATUS.STALL_REC
    STALL OFFSET(10) NUMBITS(1) [],
    /// Source: SIE_STATUS.CRC_ERROR
    ERROR_CRC OFFSET(9) NUMBITS(1) [],
    /// Source: SIE_STATUS.BIT_STUFF_ERROR
    ERROR_BIT_STUFF OFFSET(8) NUMBITS(1) [],
    /// Source: SIE_STATUS.RX_OVERFLOW
    ERROR_RX_OVERFLOW OFFSET(7) NUMBITS(1) [],
    /// Source: SIE_STATUS.RX_TIMEOUT
    ERROR_RX_TIMEOUT OFFSET(6) NUMBITS(1) [],
    /// Source: SIE_STATUS.DATA_SEQ_ERROR
    ERROR_DATA_SEQ OFFSET(5) NUMBITS(1) [],
    /// Raised when any bit in BUFF_STATUS is set. Clear by
    /// clearing all bits in BUFF_STATUS.
    BUFF_STATUS OFFSET(4) NUMBITS(1) [],
    /// Raised every time SIE_STATUS.TRANS_COMPLETE is set.
    /// Clear by writing to this bit.
    TRANS_COMPLETE OFFSET(3) NUMBITS(1) [],
    /// Host: raised every time the host sends a SOF (Start of
    /// Frame). Cleared by reading SOF_RD
    HOST_SOF OFFSET(2) NUMBITS(1) [],
    /// Host: raised when a device wakes up the host. Cleared by
    /// writing to SIE_STATUS.RESUME
    HOST_RESUME OFFSET(1) NUMBITS(1) [],
    /// Host: raised when a device is connected or disconnected
    /// (i.e. when SIE_STATUS.SPEED changes). Cleared by
    /// writing to SIE_STATUS.SPEED
    HOST_CONN_DIS OFFSET(0) NUMBITS(1) []
],
INTE [
    /// Raised when any bit in EP_STATUS_STALL_NAK is set.
    /// Clear by clearing all bits in EP_STATUS_STALL_NAK.
    EP_STALL_NAK OFFSET(19) NUMBITS(1) [],
    /// Raised when any bit in ABORT_DONE is set. Clear by
    /// clearing all bits in ABORT_DONE.
    ABORT_DONE OFFSET(18) NUMBITS(1) [],
    /// Set every time the device receives a SOF (Start of Frame)
    /// packet. Cleared by reading SOF_RD
    DEV_SOF OFFSET(17) NUMBITS(1) [],
    /// Device. Source: SIE_STATUS.SETUP_REC
    SETUP_REQ OFFSET(16) NUMBITS(1) [],
    /// Set when the device receives a resume from the host.
    /// Cleared by writing to SIE_STATUS.RESUME
    DEV_RESUME_FROM_HOST OFFSET(15) NUMBITS(1) [],
    /// Set when the device suspend state changes. Cleared by
    /// writing to SIE_STATUS.SUSPENDED
    DEV_SUSPEND OFFSET(14) NUMBITS(1) [],
    /// Set when the device connection state changes. Cleared by
    /// writing to SIE_STATUS.CONNECTED
    DEV_CONN_DIS OFFSET(13) NUMBITS(1) [],
    /// Source: SIE_STATUS.BUS_RESET
    BUS_RESET OFFSET(12) NUMBITS(1) [],
    /// Source: SIE_STATUS.VBUS_DETECT
    VBUS_DETECT OFFSET(11) NUMBITS(1) [],
    /// Source: SIE_STATUS.STALL_REC
    STALL OFFSET(10) NUMBITS(1) [],
    /// Source: SIE_STATUS.CRC_ERROR
    ERROR_CRC OFFSET(9) NUMBITS(1) [],
    /// Source: SIE_STATUS.BIT_STUFF_ERROR
    ERROR_BIT_STUFF OFFSET(8) NUMBITS(1) [],
    /// Source: SIE_STATUS.RX_OVERFLOW
    ERROR_RX_OVERFLOW OFFSET(7) NUMBITS(1) [],
    /// Source: SIE_STATUS.RX_TIMEOUT
    ERROR_RX_TIMEOUT OFFSET(6) NUMBITS(1) [],
    /// Source: SIE_STATUS.DATA_SEQ_ERROR
    ERROR_DATA_SEQ OFFSET(5) NUMBITS(1) [],
    /// Raised when any bit in BUFF_STATUS is set. Clear by
    /// clearing all bits in BUFF_STATUS.
    BUFF_STATUS OFFSET(4) NUMBITS(1) [],
    /// Raised every time SIE_STATUS.TRANS_COMPLETE is set.
    /// Clear by writing to this bit.
    TRANS_COMPLETE OFFSET(3) NUMBITS(1) [],
    /// Host: raised every time the host sends a SOF (Start of
    /// Frame). Cleared by reading SOF_RD
    HOST_SOF OFFSET(2) NUMBITS(1) [],
    /// Host: raised when a device wakes up the host. Cleared by
    /// writing to SIE_STATUS.RESUME
    HOST_RESUME OFFSET(1) NUMBITS(1) [],
    /// Host: raised when a device is connected or disconnected
    /// (i.e. when SIE_STATUS.SPEED changes). Cleared by
    /// writing to SIE_STATUS.SPEED
    HOST_CONN_DIS OFFSET(0) NUMBITS(1) []
],
INTF [
    /// Raised when any bit in EP_STATUS_STALL_NAK is set.
    /// Clear by clearing all bits in EP_STATUS_STALL_NAK.
    EP_STALL_NAK OFFSET(19) NUMBITS(1) [],
    /// Raised when any bit in ABORT_DONE is set. Clear by
    /// clearing all bits in ABORT_DONE.
    ABORT_DONE OFFSET(18) NUMBITS(1) [],
    /// Set every time the device receives a SOF (Start of Frame)
    /// packet. Cleared by reading SOF_RD
    DEV_SOF OFFSET(17) NUMBITS(1) [],
    /// Device. Source: SIE_STATUS.SETUP_REC
    SETUP_REQ OFFSET(16) NUMBITS(1) [],
    /// Set when the device receives a resume from the host.
    /// Cleared by writing to SIE_STATUS.RESUME
    DEV_RESUME_FROM_HOST OFFSET(15) NUMBITS(1) [],
    /// Set when the device suspend state changes. Cleared by
    /// writing to SIE_STATUS.SUSPENDED
    DEV_SUSPEND OFFSET(14) NUMBITS(1) [],
    /// Set when the device connection state changes. Cleared by
    /// writing to SIE_STATUS.CONNECTED
    DEV_CONN_DIS OFFSET(13) NUMBITS(1) [],
    /// Source: SIE_STATUS.BUS_RESET
    BUS_RESET OFFSET(12) NUMBITS(1) [],
    /// Source: SIE_STATUS.VBUS_DETECT
    VBUS_DETECT OFFSET(11) NUMBITS(1) [],
    /// Source: SIE_STATUS.STALL_REC
    STALL OFFSET(10) NUMBITS(1) [],
    /// Source: SIE_STATUS.CRC_ERROR
    ERROR_CRC OFFSET(9) NUMBITS(1) [],
    /// Source: SIE_STATUS.BIT_STUFF_ERROR
    ERROR_BIT_STUFF OFFSET(8) NUMBITS(1) [],
    /// Source: SIE_STATUS.RX_OVERFLOW
    ERROR_RX_OVERFLOW OFFSET(7) NUMBITS(1) [],
    /// Source: SIE_STATUS.RX_TIMEOUT
    ERROR_RX_TIMEOUT OFFSET(6) NUMBITS(1) [],
    /// Source: SIE_STATUS.DATA_SEQ_ERROR
    ERROR_DATA_SEQ OFFSET(5) NUMBITS(1) [],
    /// Raised when any bit in BUFF_STATUS is set. Clear by
    /// clearing all bits in BUFF_STATUS.
    BUFF_STATUS OFFSET(4) NUMBITS(1) [],
    /// Raised every time SIE_STATUS.TRANS_COMPLETE is set.
    /// Clear by writing to this bit.
    TRANS_COMPLETE OFFSET(3) NUMBITS(1) [],
    /// Host: raised every time the host sends a SOF (Start of
    /// Frame). Cleared by reading SOF_RD
    HOST_SOF OFFSET(2) NUMBITS(1) [],
    /// Host: raised when a device wakes up the host. Cleared by
    /// writing to SIE_STATUS.RESUME
    HOST_RESUME OFFSET(1) NUMBITS(1) [],
    /// Host: raised when a device is connected or disconnected
    /// (i.e. when SIE_STATUS.SPEED changes). Cleared by
    /// writing to SIE_STATUS.SPEED
    HOST_CONN_DIS OFFSET(0) NUMBITS(1) []
],
INTS [
    /// Raised when any bit in EP_STATUS_STALL_NAK is set.
    /// Clear by clearing all bits in EP_STATUS_STALL_NAK.
    EP_STALL_NAK OFFSET(19) NUMBITS(1) [],
    /// Raised when any bit in ABORT_DONE is set. Clear by
    /// clearing all bits in ABORT_DONE.
    ABORT_DONE OFFSET(18) NUMBITS(1) [],
    /// Set every time the device receives a SOF (Start of Frame)
    /// packet. Cleared by reading SOF_RD
    DEV_SOF OFFSET(17) NUMBITS(1) [],
    /// Device. Source: SIE_STATUS.SETUP_REC
    SETUP_REQ OFFSET(16) NUMBITS(1) [],
    /// Set when the device receives a resume from the host.
    /// Cleared by writing to SIE_STATUS.RESUME
    DEV_RESUME_FROM_HOST OFFSET(15) NUMBITS(1) [],
    /// Set when the device suspend state changes. Cleared by
    /// writing to SIE_STATUS.SUSPENDED
    DEV_SUSPEND OFFSET(14) NUMBITS(1) [],
    /// Set when the device connection state changes. Cleared by
    /// writing to SIE_STATUS.CONNECTED
    DEV_CONN_DIS OFFSET(13) NUMBITS(1) [],
    /// Source: SIE_STATUS.BUS_RESET
    BUS_RESET OFFSET(12) NUMBITS(1) [],
    /// Source: SIE_STATUS.VBUS_DETECT
    VBUS_DETECT OFFSET(11) NUMBITS(1) [],
    /// Source: SIE_STATUS.STALL_REC
    STALL OFFSET(10) NUMBITS(1) [],
    /// Source: SIE_STATUS.CRC_ERROR
    ERROR_CRC OFFSET(9) NUMBITS(1) [],
    /// Source: SIE_STATUS.BIT_STUFF_ERROR
    ERROR_BIT_STUFF OFFSET(8) NUMBITS(1) [],
    /// Source: SIE_STATUS.RX_OVERFLOW
    ERROR_RX_OVERFLOW OFFSET(7) NUMBITS(1) [],
    /// Source: SIE_STATUS.RX_TIMEOUT
    ERROR_RX_TIMEOUT OFFSET(6) NUMBITS(1) [],
    /// Source: SIE_STATUS.DATA_SEQ_ERROR
    ERROR_DATA_SEQ OFFSET(5) NUMBITS(1) [],
    /// Raised when any bit in BUFF_STATUS is set. Clear by
    /// clearing all bits in BUFF_STATUS.
    BUFF_STATUS OFFSET(4) NUMBITS(1) [],
    /// Raised every time SIE_STATUS.TRANS_COMPLETE is set.
    /// Clear by writing to this bit.
    TRANS_COMPLETE OFFSET(3) NUMBITS(1) [],
    /// Host: raised every time the host sends a SOF (Start of
    /// Frame). Cleared by reading SOF_RD
    HOST_SOF OFFSET(2) NUMBITS(1) [],
    /// Host: raised when a device wakes up the host. Cleared by
    /// writing to SIE_STATUS.RESUME
    HOST_RESUME OFFSET(1) NUMBITS(1) [],
    /// Host: raised when a device is connected or disconnected
    /// (i.e. when SIE_STATUS.SPEED changes). Cleared by
    /// writing to SIE_STATUS.SPEED
    HOST_CONN_DIS OFFSET(0) NUMBITS(1) []
]
];

register_bitfields![u32,
    RequestType [
        RECIPIENT OFFSET(0) NUMBITS(5) [
            Device = 0,
            Interface = 1,
            Endpoint = 2,
            Other = 3
        ],
        TYPE OFFSET(5) NUMBITS(2) [
            Standard = 0,
            Class = 1,
            Vendor = 2
        ],
        DIRECTION OFFSET(7) NUMBITS(1) [
            HostToDevice = 0,
            DeviceToHost = 1
        ]
    ],
    SETUP_H [
        BM_REQUEST_TYPE OFFSET(0) NUMBITS(8) [],
        B_REQUEST OFFSET(8) NUMBITS(8) [
            GET_ADDRESS = 0x05,
            GET_DESCRIPTOR = 0x07,
            GET_CONFIGURATION = 0x09,
        ],
        W_VALUE_L OFFSET(16) NUMBITS(8) [],
        W_VALUE_H OFFSET(24) NUMBITS(8) [],
    ],
    SETUP_L [
        W_INDEX_L OFFSET(0) NUMBITS(8) [],
        W_INDEX_H OFFSET(8) NUMBITS(8) [],
        W_LENGTH_L OFFSET(16) NUMBITS(8) [],
        W_LENGTH_H OFFSET(24) NUMBITS(8) [],
    ],
    EP_CONTROL [
        ENDPOINT_ENABLE OFFSET(31) NUMBITS(1) [],
        DOUBLE_BUFFERED OFFSET(30) NUMBITS(1) [],
        INTERRUPT_SINGLE_BIT OFFSET(29) NUMBITS(1) [],
        INTERRUPT_DOUBLE_BIT OFFSET(28) NUMBITS(1) [],
        ENDPOINT_TYPE OFFSET(26) NUMBITS(2) [
            CONTROL = 0,
            ISO = 1,
            BULK = 2,
            INT = 3
        ],
        INT_STALL OFFSET(17) NUMBITS(1) [],
        INT_NAK OFFSET(16) NUMBITS(1) [],
        ADDR_BASE OFFSET(0) NUMBITS(16) [],
    ],
    EP_BUFFER_CONTROL [
        BUFFER1_FULL OFFSET(31) NUMBITS(1) [],
        LAST_BUFFER1 OFFSET(30) NUMBITS(1) [],
        DATA_PID1 OFFSET(29) NUMBITS(1) [],
        DOUBLE_BUFFERED_OFFSET_ISO OFFSET(27) NUMBITS(2) [
            OFFSET_128 = 0,
            OFFSET_256 = 1,
            OFFSET_512 = 2,
            OFFSET_1024 = 3,
        ],
        AVAILABLE1 OFFSET(26) NUMBITS(1) [],
        TRANSFER_LENGTH1 OFFSET(16) NUMBITS(10) [],
        BUFFER0_FULL OFFSET(15) NUMBITS(1) [],
        LAST_BUFFER0 OFFSET(14) NUMBITS(1) [],
        DATA_PID0 OFFSET(13) NUMBITS(1) [],
        RESET_BUFFER OFFSET(12) NUMBITS(1) [],
        STALL OFFSET(11) NUMBITS(1) [],
        AVAILABLE0 OFFSET(10) NUMBITS(1) [],
        TRANSFER_LENGTH0 OFFSET(0) NUMBITS(10) [],
    ]
];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UsbState {
    Disabled,
    Started,
    Initialized,
    PoweredOn,
    Attached,
    Configured,
}

#[derive(Copy, Clone, Debug)]
pub enum EndpointState {
    Disabled,
    Ctrl(CtrlState),
    Bulk(TransferType, Option<BulkInState>, Option<BulkOutState>),
}

impl EndpointState {
    fn ctrl_state(self) -> CtrlState {
        match self {
            EndpointState::Ctrl(state) => state,
            _ => panic!("Expected EndpointState::Ctrl"),
        }
    }

    fn bulk_state(self) -> (TransferType, Option<BulkInState>, Option<BulkOutState>) {
        match self {
            EndpointState::Bulk(transfer_type, in_state, out_state) => {
                (transfer_type, in_state, out_state)
            }
            _ => panic!("Expected EndpointState::Bulk"),
        }
    }
}

/// State of the control endpoint (endpoint 0).
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CtrlState {
    /// Control endpoint is idle, and waiting for a command from the host.
    Init,
    /// Control endpoint has started an IN transfer.
    ReadIn,
    /// Control endpoint has moved to the status phase.
    ReadStatus,
    /// Control endpoint is handling a control write (OUT) transfer.
    WriteOut,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BulkInState {
    // The endpoint is ready to perform transactions.
    Init,
    // There is a pending IN packet transfer on this endpoint.
    InData,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BulkOutState {
    // The endpoint is ready to perform transactions.
    Init,
    // There is a pending OUT packet in this endpoint's buffer, to be read by
    // the client application.
    OutDelay,
    // There is a pending EPDATA to reply to. Store the size right after the
    // EPDATA event.
    OutData { size: u32 },
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EndpointType {
    NONE,
    IN,
    OUT,
}

pub struct Endpoint<'a> {
    slice_in: OptionalCell<&'a [VolatileCell<u8>]>,
    slice_out: OptionalCell<&'a [VolatileCell<u8>]>,
    state: Cell<EndpointState>,
    // Whether a transfer is requested on this IN endpoint.
    request_transmit_in: Cell<bool>,
    // Whether a transfer is requested on this OUT endpoint.
    request_transmit_out: Cell<bool>,
    direction: Cell<EndpointType>,
}

impl Endpoint<'_> {
    const fn new() -> Self {
        Endpoint {
            slice_in: OptionalCell::empty(),
            slice_out: OptionalCell::empty(),
            state: Cell::new(EndpointState::Disabled),
            request_transmit_in: Cell::new(false),
            request_transmit_out: Cell::new(false),
            direction: Cell::new(EndpointType::NONE),
        }
    }
}

const USBCTRL_DPSRAM: StaticRef<Usbctrl_DPSRAM> =
    unsafe { StaticRef::new(0x50100000 as *const Usbctrl_DPSRAM) };

const USBCTRL_REGS_BASE: StaticRef<Usbctrl_RegsRegisters> =
    unsafe { StaticRef::new(0x50110000 as *const Usbctrl_RegsRegisters) };

pub const N_ENDPOINTS: usize = 16;

pub struct UsbCtrl<'a> {
    dpsram: StaticRef<Usbctrl_DPSRAM>,
    registers: StaticRef<Usbctrl_RegsRegisters>,
    state: OptionalCell<UsbState>,
    client: OptionalCell<&'a dyn hil::usb::Client<'a>>,
    descriptors: [Endpoint<'a>; N_ENDPOINTS],
    should_set_address: VolatileCell<bool>,
    address: VolatileCell<u32>,
    next_pid_in: [VolatileCell<u8>; 16],
    next_pid_out: [VolatileCell<u8>; 16],
    errata_pin: OptionalCell<&'a RPGpioPin<'a>>,
    counter: VolatileCell<u32>,
}

impl<'a> UsbCtrl<'a> {
    pub const fn new() -> Self {
        Self {
            dpsram: USBCTRL_DPSRAM,
            registers: USBCTRL_REGS_BASE,
            client: OptionalCell::empty(),
            state: OptionalCell::new(UsbState::Disabled),
            descriptors: [
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
            ],
            should_set_address: VolatileCell::new(false),
            address: VolatileCell::new(0),
            next_pid_in: [
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
            ],
            next_pid_out: [
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
                VolatileCell::new(0),
            ],
            errata_pin: OptionalCell::empty(),
            counter: VolatileCell::new(0),
        }
    }

    fn nop_wait(&self) {
        for _i in 0..100 {
            cortexm0p::support::nop()
        }
    }

    // This is errata RP2040-E5 https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf#RP2040-E5
    pub fn set_gpio(&self, gpio_pin: &'a RPGpioPin<'a>) {
        self.errata_pin.set(gpio_pin);
    }

    pub fn enable(&self) {
        self.registers
            .usb_muxing
            .modify(USB_MUXING::TO_PHY::SET + USB_MUXING::SOFTCON::SET);
        self.registers
            .usb_pwr
            .modify(USB_PWR::VBUS_DETECT::SET + USB_PWR::VBUS_DETECT_OVERRIDE_EN::SET);
        self.registers.main_ctrl.modify(
            MAIN_CTRL::CONTROLLER_EN::SET
                + MAIN_CTRL::HOST_NDEVICE::CLEAR
                + MAIN_CTRL::SIM_TIMING::CLEAR,
        );

        self.apply_errata_e5();
        self.state.set(UsbState::Started);
    }

    pub fn get_state(&self) -> UsbState {
        self.state.unwrap_or_panic()
    }

    // Allows the peripheral to be enumerated by the USB master
    fn start(&self) {
        if self.get_state() == UsbState::Disabled {
            self.enable();
            self.registers
                .inte
                .modify(INTE::SETUP_REQ::SET + INTE::BUFF_STATUS::SET + INTE::BUS_RESET::SET);
            self.registers
                .sie_ctrl
                .modify(SIE_CTRL::EP0_DOUBLE_BUF::CLEAR + SIE_CTRL::EP0_INT_1BUF::SET);
        }
    }

    pub fn enable_pullup(&self) {
        if self.get_state() == UsbState::Started {
            self.registers.sie_ctrl.modify(SIE_CTRL::PULLUP_EN::SET);
        }
        self.state.set(UsbState::Attached);
    }

    pub fn disable_pullup(&self) {
        self.state.set(UsbState::Started);
        self.registers.sie_ctrl.modify(SIE_CTRL::PULLUP_EN::CLEAR);
    }

    fn enable_in_endpoint_(&self, transfer_type: TransferType, endpoint: usize) {
        self.descriptors[endpoint].state.set(match endpoint {
            0 => {
                self.dpsram.ep_buf_ctrl[endpoint].ep_in_buf_ctrl.set(0);
                self.dpsram.ep_buf_ctrl[endpoint]
                    .ep_in_buf_ctrl
                    .modify(EP_BUFFER_CONTROL::DATA_PID0::SET + EP_BUFFER_CONTROL::AVAILABLE0::SET);
                EndpointState::Ctrl(CtrlState::Init)
            }
            1..=N_ENDPOINTS => {
                self.dpsram.ep_ctrl[endpoint - 1].ep_in_ctrl.write(
                    EP_CONTROL::ENDPOINT_ENABLE::SET
                        + EP_CONTROL::DOUBLE_BUFFERED::CLEAR
                        + EP_CONTROL::ENDPOINT_TYPE::BULK
                        + EP_CONTROL::INTERRUPT_SINGLE_BIT::SET
                        + EP_CONTROL::INTERRUPT_DOUBLE_BIT::CLEAR
                        + EP_CONTROL::ADDR_BASE.val((0x180 + 64 * (endpoint - 1)) as u32),
                );
                self.dpsram.ep_buf_ctrl[endpoint].ep_in_buf_ctrl.set(0);
                self.descriptors[endpoint].direction.set(EndpointType::IN);
                EndpointState::Bulk(transfer_type, Some(BulkInState::Init), None)
            }
            _ => unreachable!("unexisting endpoint"),
        });
    }

    fn enable_out_endpoint_(&self, transfer_type: TransferType, endpoint: usize) {
        self.descriptors[endpoint].state.set(match endpoint {
            0 => {
                self.dpsram.ep_buf_ctrl[endpoint].ep_out_buf_ctrl.set(0);
                self.dpsram.ep_buf_ctrl[endpoint]
                    .ep_out_buf_ctrl
                    .modify(EP_BUFFER_CONTROL::DATA_PID0::SET);
                EndpointState::Ctrl(CtrlState::Init)
            }
            1..=N_ENDPOINTS => {
                self.dpsram.ep_ctrl[endpoint].ep_out_ctrl.set(0);
                self.dpsram.ep_ctrl[endpoint - 1].ep_out_ctrl.modify(
                    EP_CONTROL::ENDPOINT_ENABLE::SET
                        + EP_CONTROL::DOUBLE_BUFFERED::CLEAR
                        + EP_CONTROL::ENDPOINT_TYPE::BULK
                        + EP_CONTROL::INTERRUPT_SINGLE_BIT::SET
                        + EP_CONTROL::INTERRUPT_DOUBLE_BIT::CLEAR
                        + EP_CONTROL::ADDR_BASE.val((0x180 + 64 * (endpoint - 1)) as u32),
                );
                self.dpsram.ep_buf_ctrl[endpoint].ep_out_buf_ctrl.set(0);
                self.descriptors[endpoint].direction.set(EndpointType::OUT);
                EndpointState::Bulk(transfer_type, None, Some(BulkOutState::Init))
            }
            _ => unreachable!("unexisting endpoint"),
        });
    }

    fn apply_errata_e5(&self) {
        self.errata_pin.map(|p| {
            let (prev_ctrl, prev_pad) = p.start_usb_errata();
            self.registers.usb_muxing.set(0);
            self.registers
                .usb_muxing
                .modify(USB_MUXING::TO_DIGITAL_PAD::SET + USB_MUXING::SOFTCON::SET);
            for _i in 0..106400 {
                cortexm0p::support::nop()
            }
            self.registers.usb_muxing.set(0);
            self.registers
                .usb_muxing
                .modify(USB_MUXING::TO_PHY::SET + USB_MUXING::SOFTCON::SET);
            p.finish_usb_errata(prev_ctrl, prev_pad);
        });
    }

    fn handle_bus_reset(&self) {
        for (ep, desc) in self.descriptors.iter().enumerate() {
            match desc.state.get() {
                EndpointState::Disabled => {}
                EndpointState::Ctrl(_) => desc.state.set(EndpointState::Ctrl(CtrlState::Init)),
                EndpointState::Bulk(transfer_type, in_state, out_state) => {
                    desc.state.set(EndpointState::Bulk(
                        transfer_type,
                        in_state.map(|_| BulkInState::Init),
                        out_state.map(|_| BulkOutState::Init),
                    ));
                    if out_state.is_some() {
                        self.dpsram.ep_buf_ctrl[ep].ep_out_buf_ctrl.set(0);
                        self.dpsram.ep_buf_ctrl[ep].ep_out_buf_ctrl.modify(
                            EP_BUFFER_CONTROL::AVAILABLE0::SET
                                + EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(64_u32),
                        );
                    }
                }
            }
            self.next_pid_in[ep].set(0);
            self.next_pid_out[ep].set(0);
            desc.request_transmit_in.set(false);
            desc.request_transmit_out.set(false);
        }
        self.dpsram.ep_buf_ctrl[0]
            .ep_out_buf_ctrl
            .modify(EP_BUFFER_CONTROL::DATA_PID0::SET);

        self.dpsram.ep_buf_ctrl[0].ep_in_buf_ctrl.modify(
            EP_BUFFER_CONTROL::AVAILABLE0::SET
                + EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(64)
                + EP_BUFFER_CONTROL::DATA_PID0::CLEAR,
        );
        self.registers.buff_status.set(0);
        self.registers.addr_endp.modify(ADDR_ENDP::ADDRESS.val(0));

        self.address.set(0);
        self.client.map(|client| {
            client.bus_reset();
        });
        self.registers.sie_status.modify(SIE_STATUS::BUS_RESET::SET);
    }

    pub fn handle_interrupt(&self) {
        if self.registers.ints.is_set(INTS::BUS_RESET) {
            self.handle_bus_reset();
        }

        if self.registers.buff_status.get() != 0 {
            self.handle_buff_status();
        }

        self.process_requests();

        if self.registers.ints.is_set(INTS::SETUP_REQ) {
            self.registers.sie_status.modify(SIE_STATUS::SETUP_REC::SET);
            self.usb_handle_setup_packet();
        }
    }

    fn handle_buff_status(&self) {
        // Endpoint 0
        if self.registers.buff_status.is_set(BUFF_STATUS::EP0_IN) {
            self.handle_ep0datadone();
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP0_OUT) {
            self.handle_endepout(0);
        }
        // Endpoint 1
        if self.registers.buff_status.is_set(BUFF_STATUS::EP1_IN) {
            self.handle_endepin(1);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP1_OUT) {
            self.handle_epdata_out(1);
        }
        // Endpoint 2
        if self.registers.buff_status.is_set(BUFF_STATUS::EP2_IN) {
            self.handle_epdata_in(2);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP2_OUT) {
            self.handle_epdata_out(2);
        }
        // Endpoint 3
        if self.registers.buff_status.is_set(BUFF_STATUS::EP3_IN) {
            self.handle_endepin(3);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP3_OUT) {
            self.handle_epdata_out(3);
        }
        // Endpoint 4
        if self.registers.buff_status.is_set(BUFF_STATUS::EP4_IN) {
            self.handle_epdata_in(4);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP4_OUT) {
            self.handle_epdata_out(4);
        }
        // Endpoint 5
        if self.registers.buff_status.is_set(BUFF_STATUS::EP5_IN) {
            self.handle_epdata_in(5);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP5_OUT) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP5_OUT::CLEAR);
            self.handle_epdata_out(5);
        }
        // Endpoint 6
        if self.registers.buff_status.is_set(BUFF_STATUS::EP6_IN) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP6_IN::CLEAR);
            self.handle_epdata_in(6);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP6_OUT) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP6_OUT::CLEAR);
            self.handle_epdata_out(6);
        }
        // Endpoint 7
        if self.registers.buff_status.is_set(BUFF_STATUS::EP7_IN) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP7_IN::CLEAR);
            self.handle_epdata_in(7);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP7_OUT) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP7_OUT::CLEAR);
            self.handle_epdata_out(7);
        }
        // Endpoint 8
        if self.registers.buff_status.is_set(BUFF_STATUS::EP8_IN) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP8_IN::CLEAR);
            self.handle_epdata_in(8);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP8_OUT) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP8_OUT::CLEAR);
            self.handle_epdata_out(8);
        }
        // Endpoint 9
        if self.registers.buff_status.is_set(BUFF_STATUS::EP9_IN) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP9_IN::CLEAR);
            self.handle_epdata_in(9);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP9_OUT) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP9_OUT::CLEAR);
            self.handle_epdata_out(9);
        }
        // Endpoint 10
        if self.registers.buff_status.is_set(BUFF_STATUS::EP10_IN) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP10_IN::CLEAR);
            self.handle_epdata_in(10);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP10_OUT) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP10_OUT::CLEAR);
            self.handle_epdata_out(10);
        }
        // Endpoint 11
        if self.registers.buff_status.is_set(BUFF_STATUS::EP11_IN) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP11_IN::CLEAR);
            self.handle_epdata_in(11);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP11_OUT) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP11_OUT::CLEAR);
            self.handle_epdata_out(11);
        }
        // Endpoint 12
        if self.registers.buff_status.is_set(BUFF_STATUS::EP12_IN) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP12_IN::CLEAR);
            self.handle_epdata_in(12);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP12_OUT) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP12_OUT::CLEAR);
            self.handle_epdata_out(12);
        }
        // Endpoint 13
        if self.registers.buff_status.is_set(BUFF_STATUS::EP13_IN) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP13_IN::CLEAR);
            self.handle_epdata_in(13);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP13_OUT) {
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP13_OUT::CLEAR);
            self.handle_epdata_out(13);
        }
        // Endpoint 14
        if self.registers.buff_status.is_set(BUFF_STATUS::EP14_IN) {
            self.handle_epdata_in(14);
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP14_IN::CLEAR);
        }
        if self.registers.buff_status.is_set(BUFF_STATUS::EP14_OUT) {
            self.handle_epdata_out(14);
            self.registers
                .buff_status
                .modify(BUFF_STATUS::EP14_OUT::CLEAR);
        }
        // Endpoint 15
        if self.registers.buff_status.is_set(BUFF_STATUS::EP15_IN) {
            self.handle_epdata_in(15);
        }
        self.registers
            .buff_status
            .modify(BUFF_STATUS::EP15_IN::CLEAR);

        if self.registers.buff_status.is_set(BUFF_STATUS::EP15_OUT) {
            self.handle_epdata_out(15);
        }
        self.registers
            .buff_status
            .modify(BUFF_STATUS::EP15_OUT::CLEAR);

        self.registers.buff_status.set(0);
    }

    fn process_requests(&self) {
        for (endpoint, desc) in self.descriptors.iter().enumerate() {
            if desc.request_transmit_in.take() {
                if endpoint == 0 {
                    self.transmit_in_ep0();
                } else {
                    self.transmit_in(endpoint);
                }
            }
            if desc.request_transmit_out.take() {
                if endpoint == 0 {
                    self.transmit_out_ep0();
                } else {
                    self.transmit_out(endpoint);
                }
            }
        }
    }

    fn handle_epdata_out(&self, ep: usize) {
        let (transfer_type, in_state, out_state) = self.descriptors[ep].state.get().bulk_state();
        assert!(out_state.is_some());

        // We need to read the size at this point in the process. At this point
        // the USB hardware has received the data, but we need to
        // copy the data to memory. Later on the EPOUT.SIZE register can
        // be overwritten, particularly if the host is sending OUT
        // transactions quickly.
        let ep_size = self.dpsram.ep_buf_ctrl[ep]
            .ep_out_buf_ctrl
            .read(EP_BUFFER_CONTROL::TRANSFER_LENGTH0);

        match out_state.unwrap() {
            BulkOutState::Init => {
                // The endpoint is ready to receive data. Request a transmit_out.
                self.descriptors[ep].request_transmit_out.set(true);
            }
            BulkOutState::OutDelay => {
                // The endpoint will be resumed later by the client application with transmit_out().
            }
            BulkOutState::OutData { size: _ } => {
                self.descriptors[ep].request_transmit_out.set(true);
            }
        }
        // Indicate that the endpoint now has data available.
        self.descriptors[ep].state.set(EndpointState::Bulk(
            transfer_type,
            in_state,
            Some(BulkOutState::OutData { size: ep_size }),
        ));
    }

    fn handle_epdata_in(&self, endpoint: usize) {
        let (transfer_type, in_state, out_state) =
            self.descriptors[endpoint].state.get().bulk_state();
        assert!(in_state.is_some());
        match in_state.unwrap() {
            BulkInState::InData => {
                // Totally expected state. Nothing to do.
                self.client
                    .map(|client| client.packet_transmitted(endpoint));
                self.descriptors[endpoint].state.set(EndpointState::Bulk(
                    transfer_type,
                    Some(BulkInState::Init),
                    out_state,
                ));
            }
            BulkInState::Init => {}
        }
    }

    fn usb_handle_setup_packet(&self) {
        let endpoint = 0;

        // We are idle, and ready for any control transfer.
        let state = self.descriptors[endpoint].state.get().ctrl_state();
        match state {
            CtrlState::Init => {
                let ep_buf = &self.descriptors[endpoint].slice_out;
                let ep_buf = ep_buf.unwrap_or_panic();
                if ep_buf.len() < 8 {
                    panic!("EP0 DMA buffer length < 8");
                }

                // Re-construct the SETUP packet from various registers. The
                // client's ctrl_setup() will parse it as a SetupData
                // descriptor.
                ep_buf[0].set(self.dpsram.setup_h.read(SETUP_H::BM_REQUEST_TYPE) as u8);
                ep_buf[1].set(self.dpsram.setup_h.read(SETUP_H::B_REQUEST) as u8);
                ep_buf[2].set(self.dpsram.setup_h.read(SETUP_H::W_VALUE_L) as u8);
                ep_buf[3].set(self.dpsram.setup_h.read(SETUP_H::W_VALUE_H) as u8);
                ep_buf[4].set(self.dpsram.setup_l.read(SETUP_L::W_INDEX_L) as u8);
                ep_buf[5].set(self.dpsram.setup_l.read(SETUP_L::W_INDEX_H) as u8);
                ep_buf[6].set(self.dpsram.setup_l.read(SETUP_L::W_LENGTH_L) as u8);
                ep_buf[7].set(self.dpsram.setup_l.read(SETUP_L::W_LENGTH_H) as u8);

                let size = self.dpsram.setup_l.read(SETUP_L::W_LENGTH_L)
                    + (self.dpsram.setup_l.read(SETUP_L::W_LENGTH_H) << 8);
                self.client.map(|client| {
                    // Notify the client that the ctrl setup event has occurred.
                    // Allow it to configure any data we need to send back.
                    match client.ctrl_setup(endpoint) {
                        hil::usb::CtrlSetupResult::OkSetAddress => {
                            self.should_set_address.set(true);
                            self.send_empty_in(endpoint);
                            self.descriptors[0]
                                .state
                                .set(EndpointState::Ctrl(CtrlState::ReadStatus));
                        }
                        hil::usb::CtrlSetupResult::Ok => {
                            // Setup request is successful.
                            if size == 0 {
                                // Directly handle a 0 length setup request.
                                self.send_empty_in(endpoint);
                            } else {
                                match self.dpsram.setup_h.read(SETUP_H::BM_REQUEST_TYPE) >> 7 {
                                    0 => {
                                        self.send_empty_in(endpoint);

                                        self.transmit_out_ep0();
                                    }
                                    1 => {
                                        self.descriptors[endpoint]
                                            .state
                                            .set(EndpointState::Ctrl(CtrlState::ReadIn));
                                        // Transmit first packet.
                                        self.next_pid_in[endpoint].set(1);
                                        self.transmit_in_ep0();
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                }
                            }
                        }
                        _err => {
                            // An error occurred, we stall the endpoint.
                            self.registers
                                .ep_stall_arm
                                .modify(EP_STALL_ARM::EP0_IN::SET);
                            self.dpsram.ep_buf_ctrl[0]
                                .ep_in_buf_ctrl
                                .modify(EP_BUFFER_CONTROL::STALL::SET);
                        }
                    }
                });
            }

            CtrlState::ReadIn | CtrlState::ReadStatus | CtrlState::WriteOut => {
                // Unexpected state to receive a SETUP packet. Let's STALL the endpoint.
                self.registers.sie_ctrl.write(SIE_CTRL::EP0_INT_STALL::SET);
            }
        }
    }

    fn handle_ep0datadone(&self) {
        let endpoint = 0;
        let state = self.descriptors[endpoint].state.get().ctrl_state();

        match state {
            CtrlState::ReadIn => {
                self.transmit_in_ep0();
            }

            CtrlState::ReadStatus => {
                self.complete_ctrl_status();
            }

            CtrlState::WriteOut => {
                // We just completed the Setup stage for a CTRL WRITE transfer.
                self.transmit_out_ep0();
            }

            CtrlState::Init => {
                self.send_empty_in(0);
                self.complete_ctrl_status();
            }
        }

        self.nop_wait();

        self.dpsram.ep_buf_ctrl[0]
            .ep_in_buf_ctrl
            .modify(EP_BUFFER_CONTROL::AVAILABLE0::SET);
    }

    fn handle_endepin(&self, endpoint: usize) {
        match endpoint {
            0 => {}
            1..=N_ENDPOINTS => {
                let (transfer_type, _in_state, out_state) =
                    self.descriptors[endpoint].state.get().bulk_state();
                self.descriptors[endpoint].state.set(EndpointState::Bulk(
                    transfer_type,
                    Some(BulkInState::InData),
                    out_state,
                ));
            }
            _ => panic!("unexisting endpoint"),
        }

        // Nothing else to do. Wait for the EPDATA event.
    }

    fn handle_endepout(&self, endpoint: usize) {
        match endpoint {
            0 => {
                // We got data on the control endpoint during a CTRL WRITE
                // transfer. Let the client handle the data, and then finish up
                // the control write by moving to the status stage.

                // Now we can handle it and pass it to the client to see
                // what the client returns.

                if self.dpsram.ep0_buffer0[0].get() == 128
                    && self.dpsram.ep0_buffer0[1].get() == 37
                    && self.dpsram.ep0_buffer0[2].get() == 0
                {
                    self.dpsram.ep0_buffer0[0].set(0);
                    self.dpsram.ep0_buffer0[1].set(194);
                    self.dpsram.ep0_buffer0[2].set(1);
                }

                self.transmit_out_ep0();
                self.client.map(|client| {
                    match client.ctrl_out(
                        endpoint,
                        self.dpsram.ep_buf_ctrl[endpoint]
                            .ep_out_buf_ctrl
                            .read(EP_BUFFER_CONTROL::TRANSFER_LENGTH0),
                    ) {
                        hil::usb::CtrlOutResult::Ok => {
                            // We only handle the simple case where we have
                            // received all of the data we need to.
                            self.complete_ctrl_status();
                        }
                        hil::usb::CtrlOutResult::Delay => {}
                        _ => {
                            // Respond with STALL to any following transactions
                            // in this request
                            self.registers
                                .ep_stall_arm
                                .modify(EP_STALL_ARM::EP0_OUT::SET);
                            self.dpsram.ep_buf_ctrl[0]
                                .ep_in_buf_ctrl
                                .modify(EP_BUFFER_CONTROL::STALL::SET);
                        }
                    };
                });
            }
            1..=N_ENDPOINTS => {
                // Notify the client about the new packet.
                let (transfer_type, in_state, out_state) =
                    self.descriptors[endpoint].state.get().bulk_state();

                let packet_bytes = if let Some(BulkOutState::OutData { size }) = out_state {
                    size
                } else {
                    0
                };

                self.client.map(|client| {
                    let result = client.packet_out(transfer_type, endpoint, packet_bytes);
                    let new_out_state = match result {
                        hil::usb::OutResult::Ok => {
                            if self.dpsram.ep_buf_ctrl[endpoint]
                                .ep_out_buf_ctrl
                                .read(EP_BUFFER_CONTROL::DATA_PID0)
                                == 0
                            {
                                self.dpsram.ep_buf_ctrl[endpoint].ep_out_buf_ctrl.modify(
                                    EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(packet_bytes)
                                        + EP_BUFFER_CONTROL::DATA_PID0::SET
                                        + EP_BUFFER_CONTROL::BUFFER0_FULL::CLEAR,
                                );
                                self.next_pid_out[endpoint].set(0);
                            } else {
                                self.dpsram.ep_buf_ctrl[endpoint].ep_out_buf_ctrl.modify(
                                    EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(packet_bytes)
                                        + EP_BUFFER_CONTROL::DATA_PID0::CLEAR
                                        + EP_BUFFER_CONTROL::BUFFER0_FULL::CLEAR,
                                );
                                self.next_pid_out[endpoint].set(1);
                            }
                            self.nop_wait();
                            self.dpsram.ep_buf_ctrl[endpoint]
                                .ep_out_buf_ctrl
                                .modify(EP_BUFFER_CONTROL::AVAILABLE0::SET);
                            BulkOutState::Init
                        }

                        hil::usb::OutResult::Delay => {
                            // We can't send the packet now. Wait for a resume_out call from the client.
                            BulkOutState::OutDelay
                        }

                        hil::usb::OutResult::Error => {
                            self.registers
                                .ep_stall_arm
                                .modify(EP_STALL_ARM::EP0_OUT::SET);
                            self.dpsram.ep_buf_ctrl[endpoint]
                                .ep_out_buf_ctrl
                                .modify(EP_BUFFER_CONTROL::STALL::SET);
                            BulkOutState::Init
                        }
                    };
                    self.descriptors[endpoint].state.set(EndpointState::Bulk(
                        transfer_type,
                        in_state,
                        Some(new_out_state),
                    ));
                });
            }
            _ => unreachable!("unexisting endpoint"),
        }
    }

    fn transmit_in_ep0(&self) {
        let endpoint = 0;
        self.client.map(|client| {
            match client.ctrl_in(endpoint) {
                hil::usb::CtrlInResult::Packet(size, last) => {
                    if size == 0 {
                        internal_err!("Empty ctrl packet?");
                    }
                    let slice = self.descriptors[endpoint].slice_in.unwrap_or_panic();

                    for idx in 0..size {
                        self.dpsram.ep0_buffer0[idx].set(slice[idx].get());
                    }

                    if self.next_pid_in[endpoint].get() == 1 {
                        self.dpsram.ep_buf_ctrl[endpoint].ep_in_buf_ctrl.modify(
                            EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(size as u32)
                                + EP_BUFFER_CONTROL::BUFFER0_FULL::SET
                                + EP_BUFFER_CONTROL::DATA_PID0::SET,
                        );
                        self.next_pid_in[endpoint].set(0);
                    } else {
                        self.dpsram.ep_buf_ctrl[endpoint].ep_in_buf_ctrl.modify(
                            EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(size as u32)
                                + EP_BUFFER_CONTROL::BUFFER0_FULL::SET
                                + EP_BUFFER_CONTROL::DATA_PID0::CLEAR,
                        );
                        self.next_pid_in[endpoint].set(1);
                    }
                    self.nop_wait();
                    self.dpsram.ep_buf_ctrl[endpoint]
                        .ep_in_buf_ctrl
                        .modify(EP_BUFFER_CONTROL::AVAILABLE0::SET);
                    if last {
                        self.transmit_out_ep0();
                        self.complete_ctrl_status();
                    }
                }

                hil::usb::CtrlInResult::Delay => {
                    self.registers.sie_ctrl.write(SIE_CTRL::EP0_INT_NAK::SET);
                }

                hil::usb::CtrlInResult::Error => {
                    // An error occurred, we STALL
                    self.registers
                        .ep_stall_arm
                        .modify(EP_STALL_ARM::EP0_IN::SET);
                    self.registers.sie_ctrl.write(SIE_CTRL::EP0_INT_STALL::SET);
                    self.descriptors[endpoint]
                        .state
                        .set(EndpointState::Ctrl(CtrlState::Init));
                }
            };
        });
    }

    fn transmit_out_ep0(&self) {
        let endpoint = 0;

        let slice = self.descriptors[endpoint].slice_out.unwrap_or_panic();

        for idx in 0..self.dpsram.ep_buf_ctrl[endpoint]
            .ep_out_buf_ctrl
            .read(EP_BUFFER_CONTROL::TRANSFER_LENGTH0) as usize
        {
            slice[idx].set(self.dpsram.ep0_buffer0[idx].get());
        }

        if self.dpsram.ep_buf_ctrl[endpoint]
            .ep_out_buf_ctrl
            .read(EP_BUFFER_CONTROL::DATA_PID0)
            == 0
        {
            self.dpsram.ep_buf_ctrl[endpoint].ep_out_buf_ctrl.modify(
                EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(slice.len() as u32)
                    + EP_BUFFER_CONTROL::DATA_PID0::SET
                    + EP_BUFFER_CONTROL::BUFFER0_FULL::CLEAR,
            );
            self.next_pid_out[endpoint].set(0);
            self.nop_wait();
            self.dpsram.ep_buf_ctrl[endpoint]
                .ep_out_buf_ctrl
                .modify(EP_BUFFER_CONTROL::AVAILABLE0::SET);
        } else {
            self.dpsram.ep_buf_ctrl[endpoint].ep_out_buf_ctrl.modify(
                EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(slice.len() as u32)
                    + EP_BUFFER_CONTROL::DATA_PID0::CLEAR
                    + EP_BUFFER_CONTROL::BUFFER0_FULL::CLEAR,
            );
            self.next_pid_out[endpoint].set(1);
            self.nop_wait();
            self.dpsram.ep_buf_ctrl[endpoint]
                .ep_out_buf_ctrl
                .modify(EP_BUFFER_CONTROL::AVAILABLE0::SET);
        }
    }

    fn complete_ctrl_status(&self) {
        let endpoint = 0;
        self.client.map(|client| {
            client.ctrl_status(endpoint);
            if self.should_set_address.get() {
                self.should_set_address.set(false);
            }
            client.ctrl_status_complete(endpoint);
            self.descriptors[endpoint]
                .state
                .set(EndpointState::Ctrl(CtrlState::Init));
        });
    }

    fn transmit_in(&self, endpoint: usize) {
        self.client.map(|client| {
            let (transfer_type, in_state, out_state) =
                self.descriptors[endpoint].state.get().bulk_state();
            assert_eq!(in_state, Some(BulkInState::Init));

            let result = client.packet_in(transfer_type, endpoint);

            let new_in_state = match result {
                hil::usb::InResult::Packet(size) => {
                    let slice = self.descriptors[endpoint].slice_in.unwrap_or_panic();

                    self.counter.set(self.counter.get() + 1);
                    for idx in 0..size {
                        self.dpsram.buffers[(64 * (endpoint - 1)) + idx].set(slice[idx].get());
                    }
                    if self.next_pid_in[endpoint].get() == 1 {
                        self.dpsram.ep_buf_ctrl[endpoint].ep_in_buf_ctrl.modify(
                            EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(size as u32)
                                + EP_BUFFER_CONTROL::BUFFER0_FULL::SET
                                + EP_BUFFER_CONTROL::DATA_PID0::SET,
                        );
                        self.next_pid_in[endpoint].set(0);
                    } else {
                        self.dpsram.ep_buf_ctrl[endpoint].ep_in_buf_ctrl.modify(
                            EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(size as u32)
                                + EP_BUFFER_CONTROL::BUFFER0_FULL::SET
                                + EP_BUFFER_CONTROL::DATA_PID0::CLEAR,
                        );
                        self.next_pid_in[endpoint].set(1);
                    }
                    self.nop_wait();
                    self.dpsram.ep_buf_ctrl[endpoint]
                        .ep_in_buf_ctrl
                        .modify(EP_BUFFER_CONTROL::AVAILABLE0::SET);
                    self.descriptors[endpoint].request_transmit_in.set(false);
                    BulkInState::InData
                }

                hil::usb::InResult::Delay => {
                    // No packet to send now. Wait for a resume call from the client.
                    BulkInState::Init
                }

                hil::usb::InResult::Error => {
                    self.dpsram.ep_buf_ctrl[endpoint]
                        .ep_in_buf_ctrl
                        .modify(EP_BUFFER_CONTROL::STALL::SET);
                    BulkInState::Init
                }
            };
            self.descriptors[endpoint].state.set(EndpointState::Bulk(
                transfer_type,
                Some(new_in_state),
                out_state,
            ));
        });
    }

    fn send_empty_in(&self, endpoint: usize) {
        match endpoint {
            0 => {
                self.dpsram.ep_buf_ctrl[0].ep_in_buf_ctrl.modify(
                    EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(0)
                        + EP_BUFFER_CONTROL::BUFFER0_FULL::SET
                        + EP_BUFFER_CONTROL::DATA_PID0::SET,
                );
                self.next_pid_in[endpoint].set(1);
                self.nop_wait();
                self.dpsram.ep_buf_ctrl[0]
                    .ep_in_buf_ctrl
                    .modify(EP_BUFFER_CONTROL::AVAILABLE0::SET);
            }
            1..=N_ENDPOINTS => {
                if self.dpsram.ep_buf_ctrl[endpoint]
                    .ep_in_buf_ctrl
                    .read(EP_BUFFER_CONTROL::DATA_PID0)
                    == 0
                {
                    self.dpsram.ep_buf_ctrl[endpoint].ep_in_buf_ctrl.modify(
                        EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(0)
                            + EP_BUFFER_CONTROL::BUFFER0_FULL::SET
                            + EP_BUFFER_CONTROL::DATA_PID0::SET,
                    );
                } else {
                    self.dpsram.ep_buf_ctrl[endpoint].ep_in_buf_ctrl.modify(
                        EP_BUFFER_CONTROL::TRANSFER_LENGTH0.val(0)
                            + EP_BUFFER_CONTROL::BUFFER0_FULL::SET
                            + EP_BUFFER_CONTROL::DATA_PID0::SET,
                    );
                }
                self.nop_wait();
                self.dpsram.ep_buf_ctrl[endpoint]
                    .ep_in_buf_ctrl
                    .modify(EP_BUFFER_CONTROL::AVAILABLE0::SET);
            }
            _ => unreachable!("unexisting endpoint"),
        }
    }

    fn transmit_out(&self, endpoint: usize) {
        let size = self.dpsram.ep_buf_ctrl[endpoint]
            .ep_out_buf_ctrl
            .read(EP_BUFFER_CONTROL::TRANSFER_LENGTH0);

        let slice = self.descriptors[endpoint].slice_out.unwrap_or_panic();

        for idx in 0..size as usize {
            slice[idx].set(self.dpsram.buffers[(64 * (endpoint - 1)) + idx].get());
        }
        let (transfer_type, in_state, out_state) =
            self.descriptors[endpoint].state.get().bulk_state();
        // Starting the receiving can only happen in the OutData state, i.e. after an EPDATA event.
        assert!(matches!(out_state, Some(BulkOutState::OutData { .. })));
        self.descriptors[endpoint].request_transmit_out.set(false);
        let size = if let Some(BulkOutState::OutData { size }) = out_state {
            size
        } else {
            0
        };

        self.descriptors[endpoint].state.set(EndpointState::Bulk(
            transfer_type,
            in_state,
            Some(BulkOutState::OutData { size }),
        ));

        self.handle_endepout(endpoint);
    }
}

impl<'a> hil::usb::UsbController<'a> for UsbCtrl<'a> {
    fn set_client(&self, client: &'a dyn hil::usb::Client<'a>) {
        self.client.set(client);
    }

    fn endpoint_set_ctrl_buffer(&self, buf: &'a [VolatileCell<u8>]) {
        if buf.len() < 8 {
            panic!("Endpoint buffer must be at least 8 bytes");
        }
        if !buf.len().is_power_of_two() {
            panic!("Buffer size must be a power of 2");
        }
        self.descriptors[0].slice_in.set(buf);
        self.descriptors[0].slice_out.set(buf);
    }

    fn endpoint_set_in_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]) {
        if buf.len() < 8 {
            panic!("Endpoint buffer must be at least 8 bytes");
        }
        if !buf.len().is_power_of_two() {
            panic!("Buffer size must be a power of 2");
        }
        if endpoint == 0 || endpoint >= N_ENDPOINTS {
            panic!("Endpoint number is invalid");
        }
        self.descriptors[endpoint].slice_in.set(buf);
    }

    fn endpoint_set_out_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]) {
        if buf.len() < 8 {
            panic!("Endpoint buffer must be at least 8 bytes");
        }
        if !buf.len().is_power_of_two() {
            panic!("Buffer size must be a power of 2");
        }
        if endpoint == 0 || endpoint >= N_ENDPOINTS {
            panic!("Endpoint number is invalid");
        }
        self.descriptors[endpoint].slice_out.set(buf);
    }

    fn enable_as_device(&self, speed: hil::usb::DeviceSpeed) {
        match speed {
            hil::usb::DeviceSpeed::Low => internal_err!("Low speed is not supported"),
            hil::usb::DeviceSpeed::Full => {}
        }
        self.start();
    }

    fn attach(&self) {
        self.enable_pullup();
    }

    fn detach(&self) {
        self.disable_pullup();
    }

    fn set_address(&self, addr: u16) {
        self.address.set(addr as u32);
    }

    fn enable_address(&self) {
        self.registers
            .addr_endp
            .modify(ADDR_ENDP::ADDRESS.val(self.address.get()));
    }

    fn endpoint_in_enable(&self, transfer_type: TransferType, endpoint: usize) {
        match transfer_type {
            TransferType::Control => {
                panic!("There is no IN control endpoint");
            }
            TransferType::Bulk | TransferType::Interrupt => {
                if endpoint == 0 || endpoint >= N_ENDPOINTS {
                    panic!("Bulk/Interrupt endpoints are endpoints 1 to 7");
                }
                self.enable_in_endpoint_(transfer_type, endpoint);
            }
            TransferType::Isochronous => unimplemented!("isochronous endpoint"),
        }
    }

    fn endpoint_out_enable(&self, transfer_type: TransferType, endpoint: usize) {
        match transfer_type {
            TransferType::Control => {
                if endpoint != 0 {
                    panic!("Only endpoint 0 can be a control endpoint");
                }
                self.enable_out_endpoint_(transfer_type, endpoint);
            }
            TransferType::Bulk | TransferType::Interrupt => {
                if endpoint == 0 || endpoint >= N_ENDPOINTS {
                    panic!("Bulk/Interrupt endpoints are endpoints 1 to 7");
                }
                self.enable_out_endpoint_(transfer_type, endpoint);
            }
            TransferType::Isochronous => unimplemented!("isochronous endpoint"),
        }
    }

    fn endpoint_in_out_enable(&self, transfer_type: TransferType, endpoint: usize) {
        match transfer_type {
            TransferType::Control => {
                panic!("There is no IN control endpoint");
            }
            TransferType::Bulk | TransferType::Interrupt => {
                if endpoint == 0 || endpoint >= N_ENDPOINTS {
                    panic!("Bulk/Interrupt endpoints are endpoints 1 to 7");
                }
            }
            TransferType::Isochronous => unimplemented!("isochronous endpoint"),
        }
    }

    fn endpoint_resume_in(&self, endpoint: usize) {
        // Get the state of the endpoint that the upper layer requested to start
        // an IN transfer with for our state machine.
        let (_, in_state, _) = self.descriptors[endpoint].state.get().bulk_state();
        // If the state is `None`, this endpoint is not configured and should
        // not have been used to call `endpoint_resume_in()`.
        assert!(in_state.is_some());

        // If there is an active request, or we are waiting on finishing up
        // a previous IN transfer, we queue this request and it will be serviced
        // after those complete.
        if in_state == Some(BulkInState::Init) {
            // If we aren't waiting on anything, trigger the transaction now.
            self.transmit_in(endpoint);
        } else {
            self.descriptors[endpoint].request_transmit_in.set(true);
        }
    }

    fn endpoint_resume_out(&self, endpoint: usize) {
        let (transfer_type, in_state, out_state) =
            self.descriptors[endpoint].state.get().bulk_state();
        assert!(out_state.is_some());

        match out_state.unwrap() {
            BulkOutState::OutDelay => {
                // The endpoint has now finished processing the last ENDEPOUT. No EPDATA event
                // happened in the meantime, so the state is now back to Init.
                self.descriptors[endpoint].state.set(EndpointState::Bulk(
                    transfer_type,
                    in_state,
                    Some(BulkOutState::Init),
                ));
            }
            BulkOutState::OutData { size: _ } => {
                // Although the client reported a delay before, an EPDATA event has
                // happened in the meantime. This pending transaction will now
                // continue in transmit_out().
                self.transmit_out(endpoint);
            }
            BulkOutState::Init => {
                internal_err!("Unexpected state: {:?}", out_state);
            }
        }
    }
}
