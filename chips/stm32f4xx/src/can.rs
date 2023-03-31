// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022
// Copyright OxidOS Automotive SRL 2022
//
// Author: Teona Severin <teona.severin@oxidos.io>

//! Low-level CAN driver for STM32F4XX chips
//!

use crate::rcc;
use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::can::{self, StandardBitTiming};
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

pub const BRP_MIN_STM32: u32 = 0;
pub const BRP_MAX_STM32: u32 = 1023;

pub const TX_MAILBOX_COUNT: usize = 3;
pub const RX_MAILBOX_COUNT: usize = 2;
pub const FILTER_COUNT: usize = 56;

register_structs! {
    pub Registers {
        /// CAN control and status registers
        (0x000 => can_mcr: ReadWrite<u32, CAN_MCR::Register>),
        /// CAN master status register
        (0x004 => can_msr: ReadWrite<u32, CAN_MSR::Register>),
        /// CAN transmit status register
        (0x008 => can_tsr: ReadWrite<u32, CAN_TSR::Register>),
        /// CAN receive FIFO 0 register
        (0x00c => can_rf0r: ReadWrite<u32, CAN_RF0R::Register>),
        /// CAN receive FIFO 1 registers
        (0x010 => can_rf1r: ReadWrite<u32, CAN_RF1R::Register>),
        /// CAN interrupt enable register
        (0x014 => can_ier: ReadWrite<u32, CAN_IER::Register>),
        /// CAN error status register
        (0x018 => can_esr: ReadWrite<u32, CAN_ESR::Register>),
        /// CAN bit timing register
        (0x01c => can_btr: ReadWrite<u32, CAN_BTR::Register>),
        (0x020 => _reserved0),
        ///
        ///
        /// CAN MAILBOX REGISTERS
        ///
        /// CAN TX mailbox identifier registers
        (0x180 => can_tx_mailbox: [TransmitMailBox; TX_MAILBOX_COUNT]),
        /// CAN RX mailbox identifier registers
        (0x1b0 => can_rx_mailbox: [ReceiveMailBox; RX_MAILBOX_COUNT]),
        (0x1d0 => _reserved1),
        ///
        ///
        /// CAN FILTER REGISTERS
        ///
        ///
        /// CAN filter master register
        (0x200 => can_fmr: ReadWrite<u32, CAN_FMR::Register>),
        /// CAN filter mode register
        (0x204 => can_fm1r: ReadWrite<u32, CAN_FM1R::Register>),
        (0x208 => _reserved2),
        /// CAN filter scale register
        (0x20c => can_fs1r: ReadWrite<u32, CAN_FS1R::Register>),
        (0x210 => _reserved3),
        /// CAN filter FIFO assignment register
        (0x214 => can_ffa1r: ReadWrite<u32, CAN_FFA1R::Register>),
        (0x218 => _reserved4),
        /// CAN filter activation register
        (0x21c => can_fa1r: ReadWrite<u32, CAN_FA1R::Register>),
        (0x220 => _reserved5),
        /// Filter bank 0-27 for register 1-2
        (0x240 => can_firx: [ReadWrite<u32, CAN_FiRx::Register>; FILTER_COUNT]),
        (0x320 => @END),
    },

    TransmitMailBox {
        (0x00 => can_tir: ReadWrite<u32, CAN_TIxR::Register>),
        (0x04 => can_tdtr: ReadWrite<u32, CAN_TDTxR::Register>),
        (0x08 => can_tdlr: ReadWrite<u32, CAN_TDLxR::Register>),
        (0x0c => can_tdhr: ReadWrite<u32, CAN_TDHxR::Register>),
        (0x010 => @END),
    },

    ReceiveMailBox {
        (0x00 => can_rir: ReadWrite<u32, CAN_RIxR::Register>),
        (0x04 => can_rdtr: ReadWrite<u32, CAN_RDTxR::Register>),
        (0x08 => can_rdlr: ReadWrite<u32, CAN_RDLxR::Register>),
        (0x0c => can_rdhr: ReadWrite<u32, CAN_RDHxR::Register>),
        (0x010 => @END),
    }
}

register_bitfields![u32,
    CAN_MCR [
        /// Debug freeze
        DBF OFFSET(16) NUMBITS(1) [],
        /// bcXAN software master reset
        RESET OFFSET(15) NUMBITS(1) [],
        /// Time triggered communication mode
        TTCM OFFSET(7) NUMBITS(1) [],
        /// Automatic bus-off management
        ABOM OFFSET(6) NUMBITS(1) [],
        /// Automatic wakeup mode
        AWUM OFFSET(5) NUMBITS(1) [],
        /// No automatic retransmission
        NART OFFSET(4) NUMBITS(1) [],
        /// Receive FIFO locked mode
        RFLM OFFSET(3) NUMBITS(1) [],
        /// Transmit FIFO prioritY
        TXFP OFFSET(2) NUMBITS(1) [],
        /// Sleep mode request
        SLEEP OFFSET(1) NUMBITS(1) [],
        /// Initialization request
        INRQ OFFSET(0) NUMBITS(1) []
    ],
    CAN_MSR [
        /// CAN Rx signal
        RX OFFSET(11) NUMBITS(1) [],
        /// Last sample point
        SAMP OFFSET(10) NUMBITS(1) [],
        /// Receive mode
        RXM OFFSET(9) NUMBITS(1) [],
        /// Transmit mode
        TXM OFFSET(8) NUMBITS(1) [],
        /// Sleep acknowledge interrupt
        SLAKI OFFSET(4) NUMBITS(1) [],
        /// Wakeup interrupt
        WKUI OFFSET(3) NUMBITS(1) [],
        /// Error interrupt
        ERRI OFFSET(2) NUMBITS(1) [],
        /// Sleep acknowledge
        SLAK OFFSET(1) NUMBITS(1) [],
        /// Initialization acknowledge
        INAK OFFSET(0) NUMBITS(1) []
    ],
    CAN_TSR [
        /// Lowest priority flag for mailbox 2
        LOW2 OFFSET(31) NUMBITS(1) [],
        /// Lowest priority flag for mailbox 1
        LOW1 OFFSET(30) NUMBITS(1) [],
        /// Lowest priority flag for mailbox 0
        LOW0 OFFSET(29) NUMBITS(1) [],
        /// Transmit mailbox 2 empty
        TME2 OFFSET(28) NUMBITS(1) [],
        /// Transmit mailbox 1 empty
        TME1 OFFSET(27) NUMBITS(1) [],
        /// Transmit mailbox 0 empty
        TME0 OFFSET(26) NUMBITS(1) [],
        /// Mailbox code
        CODE OFFSET(24) NUMBITS(2) [],
        /// Abort request for mailbox 2
        ABRQ2 OFFSET(23) NUMBITS(1) [],
        /// Transmission error of mailbox 2
        TERR2 OFFSET(19) NUMBITS(1) [],
        /// Arbitration lost for mailbox 2
        ALST2 OFFSET(18) NUMBITS(1) [],
        /// Transmission OK of mailbox 2
        TXOK2 OFFSET(17) NUMBITS(1) [],
        /// Request completed mailbox 2
        RQCP2 OFFSET(16) NUMBITS(1) [],
        /// Abort request for mailbox 1
        ABRQ1 OFFSET(15) NUMBITS(1) [],
        /// Transmission error of mailbox 1
        TERR1 OFFSET(11) NUMBITS(1) [],
        /// Arbitration lost for mailbox 1
        ALST1 OFFSET(10) NUMBITS(1) [],
        /// Transmission OK of mailbox 1
        TXOK1 OFFSET(9) NUMBITS(1) [],
        /// Request completed mailbox 1
        RQCP1 OFFSET(8) NUMBITS(1) [],
        /// Abort request for mailbox 0
        ABRQ0 OFFSET(7) NUMBITS(1) [],
        /// Transmission error of mailbox 0
        TERR0 OFFSET(3) NUMBITS(1) [],
        /// Arbitration lost for mailbox 0
        ALST0 OFFSET(2) NUMBITS(1) [],
        /// Transmission OK of mailbox 0
        TXOK0 OFFSET(1) NUMBITS(1) [],
        /// Request completed mailbox 0
        RQCP0 OFFSET(0) NUMBITS(1) []
    ],
    CAN_RF0R [
        /// Release FIFO 0 output mailbox
        RFOM0 OFFSET(5) NUMBITS(1) [],
        /// FIFO 0 overrun
        FOVR0 OFFSET(4) NUMBITS(1) [],
        /// FIFO 0 full
        FULL0 OFFSET(3) NUMBITS(1) [],
        /// FIFO 0 message pending
        FMP0 OFFSET(0) NUMBITS(2) []
    ],
    CAN_RF1R [
        /// Release FIFO 1 output mailbox
        RFOM1 OFFSET(5) NUMBITS(1) [],
        /// FIFO 1 overrun
        FOVR1 OFFSET(4) NUMBITS(1) [],
        /// FIFO 1 full
        FULL1 OFFSET(3) NUMBITS(1) [],
        /// FIFO 1 message pending
        FMP1 OFFSET(0) NUMBITS(2) []
    ],
    CAN_IER [
        /// Sleep interrupt enable
        SLKIE OFFSET(17) NUMBITS(1) [],
        /// Wakeup interrupt enable
        WKUIE OFFSET(16) NUMBITS(1) [],
        /// Error interrupt enable
        ERRIE OFFSET(15) NUMBITS(1) [],
        /// Last error code interrupt enable
        LECIE OFFSET(11) NUMBITS(1) [],
        /// Bus-off interrupt enable
        BOFIE OFFSET(10) NUMBITS(1) [],
        /// Error passive interrupt enable
        EPVIE OFFSET(9) NUMBITS(1) [],
        /// Error warning interrupt enable
        EWGIE OFFSET(8) NUMBITS(1) [],
        /// FIFO 1 overrun interrupt enable
        FOVIE1 OFFSET(6) NUMBITS(1) [],
        /// FIFO 1 full interrupt enable
        FFIE1 OFFSET(5) NUMBITS(1) [],
        /// FIFO 1 message pending interrupt enable
        FMPIE1 OFFSET(4) NUMBITS(1) [],
        /// FIFO 0 overrun interrupt enable
        FOVIE0 OFFSET(3) NUMBITS(1) [],
        /// FIFO 0 full interrupt enable
        FFIE0 OFFSET(2) NUMBITS(1) [],
        /// FIFO 0 message pending interrupt enable
        FMPIE0 OFFSET(1) NUMBITS(1) [],
        /// Transmit mailbox empty interrupt enable
        TMEIE OFFSET(0) NUMBITS(1) []
    ],
    CAN_ESR [
        /// Receive error counter
        REC OFFSET(24) NUMBITS(8) [],
        /// Least significant byte of the 9-bit transmit error counter
        TEC OFFSET(16) NUMBITS(8) [],
        /// Last error code
        LEC OFFSET(4) NUMBITS(3) [
            NoError = 0,
            StuffError = 1,
            FormError = 2,
            AcknowledgmentError = 3,
            BitRecessiveError = 4,
            BitDominantError = 5,
            CrcError = 6,
            SetBySoftware = 7
        ],
        /// Bus-off flag
        BOFF OFFSET(2) NUMBITS(1) [],
        /// Error passive flag
        EPVF OFFSET(1) NUMBITS(1) [],
        /// Error warning flag
        EWGF OFFSET(0) NUMBITS(1) []
    ],
    CAN_BTR [
        /// Silent mode (debug)
        SILM OFFSET(31) NUMBITS(1) [],
        /// Loop back mode (debug)
        LBKM OFFSET(30) NUMBITS(1) [],
        /// Resynchronization jump width
        SJW OFFSET(24) NUMBITS(2) [],
        /// Time segment 2
        TS2 OFFSET(20) NUMBITS(3) [],
        /// Time segment 1
        TS1 OFFSET(16) NUMBITS(4) [],
        /// Baud rate prescaler
        BRP OFFSET(0) NUMBITS(10) []
    ],
    ///
    ///
    /// CAN mailbox registers
    ///
    ///
    CAN_TIxR [
        /// Standard identifier or extended identifier
        STID OFFSET(21) NUMBITS(11) [],
        /// Extended identifier
        EXID OFFSET(3) NUMBITS(18) [],
        /// Identifier extension
        IDE OFFSET(2) NUMBITS(1) [],
        /// Remote transmission request
        RTR OFFSET(1) NUMBITS(1) [],
        /// Transmit mailbox request
        TXRQ OFFSET(0) NUMBITS(1) []
    ],
    CAN_TDTxR [
        /// Message time stamp
        TIME OFFSET(16) NUMBITS(16) [],
        /// Transmit global time
        TGT OFFSET(8) NUMBITS(1) [],
        /// Data length code
        DLC OFFSET(0) NUMBITS(4) []
    ],
    CAN_TDLxR [
        /// Data byte 3
        DATA3 OFFSET(24) NUMBITS(8) [],
        /// Data byte 2
        DATA2 OFFSET(16) NUMBITS(8) [],
        /// Data byte 1
        DATA1 OFFSET(8) NUMBITS(8) [],
        /// Data byte 0
        DATA0 OFFSET(0) NUMBITS(8) []
    ],
    CAN_TDHxR [
        /// Data byte 7
        DATA7 OFFSET(24) NUMBITS(8) [],
        /// Data byte 6
        DATA6 OFFSET(16) NUMBITS(8) [],
        /// Data byte 5
        DATA5 OFFSET(8) NUMBITS(8) [],
        /// Data byte 4
        DATA4 OFFSET(0) NUMBITS(8) []
    ],
    CAN_RIxR [
        /// Standard identifier or extended identifier
        STID OFFSET(21) NUMBITS(11) [],
        /// Extended identifier
        EXID OFFSET(3) NUMBITS(18) [],
        /// Identifier extension
        IDE OFFSET(2) NUMBITS(1) [],
        /// Remote transmission request
        RTR OFFSET(1) NUMBITS(1) []
    ],
    CAN_RDTxR [
        /// Message time stamp
        TIME OFFSET(16) NUMBITS(16) [],
        /// Filter match index
        FMI OFFSET(8) NUMBITS(8) [],
        /// Data length code
        DLC OFFSET(0) NUMBITS(4) []
    ],
    CAN_RDLxR [
        /// Data byte 3
        DATA3 OFFSET(24) NUMBITS(8) [],
        /// Data byte 2
        DATA2 OFFSET(16) NUMBITS(8) [],
        /// Data byte 1
        DATA1 OFFSET(8) NUMBITS(8) [],
        /// Data byte 0
        DATA0 OFFSET(0) NUMBITS(8) []
    ],
    CAN_RDHxR [
        /// Data byte 7
        DATA7 OFFSET(24) NUMBITS(8) [],
        /// Data byte 6
        DATA6 OFFSET(16) NUMBITS(8) [],
        /// Data byte 5
        DATA5 OFFSET(8) NUMBITS(8) [],
        /// Data byte 4
        DATA4 OFFSET(0) NUMBITS(8) []
    ],
    ///
    ///
    /// CAN filter registers
    ///
    ///
    CAN_FMR [
        /// CAN start bank
        CANSB OFFSET(8) NUMBITS(6) [],
        /// Filter initialization mode
        FINIT OFFSET(0) NUMBITS(1) []
    ],
    /// CAN filter mode register
    CAN_FM1R [
        /// Filter mode
        FBM OFFSET(0) NUMBITS(28) []
    ],
    CAN_FS1R [
        /// Filter scale configuration
        FSC OFFSET(0) NUMBITS(28) []
    ],
    CAN_FFA1R [
        /// Filter FIFO assignment for filter x
        FFA OFFSET(0) NUMBITS(28) []
    ],
    CAN_FA1R [
        /// Filter active
        FACT OFFSET(0) NUMBITS(28) []
    ],
    CAN_FiRx [
        /// Filter bits
        FB OFFSET(0) NUMBITS(32) []
    ]
];

#[derive(Copy, Clone, PartialEq)]
enum CanState {
    Initialization,
    Normal,
    Sleep,
    RunningError(can::Error),
}

// The 4 possbile actions that the deferred call task can do.
#[derive(Copy, Clone, PartialEq)]
enum AsyncAction {
    Enable,
    AbortReceive,
    Disabled,
    EnableError(kernel::ErrorCode),
}

#[repr(u32)]
enum BitSegment1 {
    CanBtrTs1Min = 0b0000,
    CanBtrTs1Max = 0b1111,
}

#[repr(u32)]
enum BitSegment2 {
    CanBtrTs2Min = 0b0000,
    CanBtrTs2Max = 0b0111,
}

#[repr(u32)]
enum SynchronizationJumpWidth {
    CanBtrSjwMin = 0b00,
    CanBtrSjwMax = 0b11,
}

#[derive(Copy, Clone, PartialEq)]
pub enum CanInterruptMode {
    TransmitInterrupt,
    Fifo0Interrupt,
    Fifo1Interrupt,
    ErrorAndStatusChangeInterrupt,
}

impl From<CanState> for can::State {
    fn from(state: CanState) -> Self {
        match state {
            CanState::Initialization | CanState::Sleep => can::State::Disabled,
            CanState::Normal => can::State::Running,
            CanState::RunningError(err) => can::State::Error(err),
        }
    }
}

pub struct Can<'a> {
    registers: StaticRef<Registers>,
    clock: CanClock<'a>,
    can_state: Cell<CanState>,
    error_interrupt_counter: Cell<u32>,
    fifo0_interrupt_counter: Cell<u32>,
    fifo1_interrupt_counter: Cell<u32>,
    failed_messages: Cell<u32>,

    // communication parameters
    automatic_retransmission: Cell<bool>,
    automatic_wake_up: Cell<bool>,
    operating_mode: OptionalCell<can::OperationMode>,
    bit_timing: OptionalCell<can::BitTiming>,

    // clients
    controller_client: OptionalCell<&'static dyn can::ControllerClient>,
    receive_client:
        OptionalCell<&'static dyn can::ReceiveClient<{ can::STANDARD_CAN_PACKET_SIZE }>>,
    transmit_client:
        OptionalCell<&'static dyn can::TransmitClient<{ can::STANDARD_CAN_PACKET_SIZE }>>,

    // buffers for transmission and reception
    rx_buffer: TakeCell<'static, [u8; can::STANDARD_CAN_PACKET_SIZE]>,
    tx_buffer: TakeCell<'static, [u8; can::STANDARD_CAN_PACKET_SIZE]>,

    deferred_call: DeferredCall,
    // deferred call task action
    deferred_action: OptionalCell<AsyncAction>,
}

impl<'a> Can<'a> {
    pub fn new(rcc: &'a rcc::Rcc, registers: StaticRef<Registers>) -> Can<'a> {
        Can {
            registers: registers,
            clock: CanClock(rcc::PeripheralClock::new(
                rcc::PeripheralClockType::APB1(rcc::PCLK1::CAN1),
                rcc,
            )),
            can_state: Cell::new(CanState::Sleep),
            error_interrupt_counter: Cell::new(0),
            fifo0_interrupt_counter: Cell::new(0),
            fifo1_interrupt_counter: Cell::new(0),
            failed_messages: Cell::new(0),
            automatic_retransmission: Cell::new(false),
            automatic_wake_up: Cell::new(false),
            operating_mode: OptionalCell::empty(),
            bit_timing: OptionalCell::empty(),
            controller_client: OptionalCell::empty(),
            receive_client: OptionalCell::empty(),
            transmit_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            tx_buffer: TakeCell::empty(),
            deferred_call: DeferredCall::new(),
            deferred_action: OptionalCell::empty(),
        }
    }

    /// This function is used for busy waiting and checks if the closure
    /// received as an argument returns a true value for `times` times.
    ///
    /// Usage: check is the INAK bit in the CAN_MSR is set for 200_000 times.
    /// ```ignore
    ///    Can::wait_for(200_000, || self.registers.can_msr.is_set(CAN_MSR::INAK))
    /// ```
    fn wait_for(times: usize, f: impl Fn() -> bool) -> bool {
        for _ in 0..times {
            if f() {
                return true;
            }
        }

        false
    }

    /// Enable the peripheral with the stored communication parameters:
    /// bit timing settings and communication mode
    pub fn enable(&self) -> Result<(), kernel::ErrorCode> {
        // leave Sleep Mode
        self.registers.can_mcr.modify(CAN_MCR::SLEEP::CLEAR);

        // request to enter the initialization mode
        self.registers.can_mcr.modify(CAN_MCR::INRQ::SET);

        // After requesting to enter the initialization mode, the driver
        // must wait for ACK from the peripheral - the INAK bit to be set
        // (as explained in RM0090 Reference Manual, Chapter 32.4.1).
        // This is done by checking the INAK bit 20_000 times or until it is set.
        if !Can::wait_for(20000, || self.registers.can_msr.is_set(CAN_MSR::INAK)) {
            return Err(kernel::ErrorCode::FAIL);
        }

        self.can_state.set(CanState::Initialization);

        // After requesting to enter the initialization mode, the driver
        // must wait for ACK from the peripheral - the SLAK bit to be cleared
        // (as explained in RM0090 Reference Manual, Chapter 32.4, Figure 336).
        // This is done by checking the SLAK bit 20_000 times or until it is cleared.
        if !Can::wait_for(20000, || !self.registers.can_msr.is_set(CAN_MSR::SLAK)) {
            return Err(kernel::ErrorCode::FAIL);
        }

        // set communication mode
        self.registers.can_mcr.modify(CAN_MCR::TTCM::CLEAR);
        self.registers.can_mcr.modify(CAN_MCR::ABOM::CLEAR);
        self.registers.can_mcr.modify(CAN_MCR::RFLM::CLEAR);
        self.registers.can_mcr.modify(CAN_MCR::TXFP::CLEAR);

        match self.automatic_retransmission.get() {
            true => self.registers.can_mcr.modify(CAN_MCR::AWUM::SET),
            false => self.registers.can_mcr.modify(CAN_MCR::AWUM::CLEAR),
        }

        match self.automatic_wake_up.get() {
            true => self.registers.can_mcr.modify(CAN_MCR::NART::CLEAR),
            false => self.registers.can_mcr.modify(CAN_MCR::NART::SET),
        }

        if let Some(operating_mode_settings) = self.operating_mode.extract() {
            match operating_mode_settings {
                can::OperationMode::Loopback => self.registers.can_btr.modify(CAN_BTR::LBKM::SET),
                can::OperationMode::Monitoring => self.registers.can_btr.modify(CAN_BTR::SILM::SET),
                can::OperationMode::Freeze => return Err(kernel::ErrorCode::INVAL),
                _ => {}
            }
        }

        // set bit timing mode
        if let Some(bit_timing_settings) = self.bit_timing.extract() {
            self.registers
                .can_btr
                .modify(CAN_BTR::TS1.val(bit_timing_settings.segment1 as u32));
            self.registers
                .can_btr
                .modify(CAN_BTR::TS2.val(bit_timing_settings.segment2 as u32));
            self.registers
                .can_btr
                .modify(CAN_BTR::SJW.val(bit_timing_settings.sync_jump_width as u32));
            self.registers
                .can_btr
                .modify(CAN_BTR::BRP.val(bit_timing_settings.baud_rate_prescaler as u32));
        } else {
            self.enter_sleep_mode();
            return Err(kernel::ErrorCode::INVAL);
        }

        Ok(())
    }

    /// Configure a filter to receive messages
    pub fn config_filter(&self, filter_info: can::FilterParameters, enable: bool) {
        // get position of the filter number
        let filter_number = 1 << filter_info.number;

        // start filter configuration
        self.registers.can_fmr.modify(CAN_FMR::FINIT::SET);

        // request filter number filter_number
        self.registers.can_fa1r.modify(
            CAN_FA1R::FACT.val(self.registers.can_fa1r.read(CAN_FA1R::FACT) & !filter_number),
        );

        // request filter width to be 32 or 16 bits
        match filter_info.scale_bits {
            can::ScaleBits::Bits16 => {
                self.registers.can_fs1r.modify(
                    CAN_FS1R::FSC.val(self.registers.can_fs1r.read(CAN_FS1R::FSC) | filter_number),
                );
            }
            can::ScaleBits::Bits32 => {
                self.registers.can_fs1r.modify(
                    CAN_FS1R::FSC.val(self.registers.can_fs1r.read(CAN_FS1R::FSC) & !filter_number),
                );
            }
        }

        self.registers.can_firx[(filter_info.number as usize) * 2].modify(CAN_FiRx::FB.val(0));
        self.registers.can_firx[(filter_info.number as usize) * 2 + 1].modify(CAN_FiRx::FB.val(0));

        // request filter mode to be mask or list
        match filter_info.identifier_mode {
            can::IdentifierMode::List => {
                self.registers.can_fm1r.modify(
                    CAN_FM1R::FBM.val(self.registers.can_fm1r.read(CAN_FM1R::FBM) | filter_number),
                );
            }
            can::IdentifierMode::Mask => {
                self.registers.can_fm1r.modify(
                    CAN_FM1R::FBM.val(self.registers.can_fm1r.read(CAN_FM1R::FBM) & !filter_number),
                );
            }
        }

        // request fifo0 or fifo1
        if filter_info.fifo_number == 0 {
            self.registers.can_ffa1r.modify(
                CAN_FFA1R::FFA.val(self.registers.can_ffa1r.read(CAN_FFA1R::FFA) & !filter_number),
            );
        } else {
            self.registers.can_ffa1r.modify(
                CAN_FFA1R::FFA.val(self.registers.can_ffa1r.read(CAN_FFA1R::FFA) | filter_number),
            );
        }

        if enable {
            self.registers.can_fa1r.modify(
                CAN_FA1R::FACT.val(self.registers.can_fa1r.read(CAN_FA1R::FACT) | filter_number),
            );
        } else {
            self.registers.can_fa1r.modify(
                CAN_FA1R::FACT.val(self.registers.can_fa1r.read(CAN_FA1R::FACT) & !filter_number),
            );
        }
    }

    pub fn enable_filter_config(&self) {
        // activate the filter configuration
        self.registers.can_fmr.modify(CAN_FMR::FINIT::CLEAR);
    }

    pub fn enter_normal_mode(&self) -> Result<(), kernel::ErrorCode> {
        // request to enter normal mode by clearing INRQ bit
        self.registers.can_mcr.modify(CAN_MCR::INRQ::CLEAR);

        // After requesting to enter the normal mode, the driver
        // must wait for ACK from the peripheral - the INAK bit to be cleared
        // (as explained in RM0090 Reference Manual, Chapter 32.4.2).
        // This is done by checking the INAK bit 20_000 times or until it is cleared.
        if !Can::wait_for(20000, || !self.registers.can_msr.is_set(CAN_MSR::INAK)) {
            return Err(kernel::ErrorCode::FAIL);
        }

        self.can_state.set(CanState::Normal);
        Ok(())
    }

    pub fn enter_sleep_mode(&self) {
        // request to enter sleep mode by setting SLEEP bit
        self.disable_irqs();
        self.registers.can_mcr.modify(CAN_MCR::SLEEP::SET);
        self.can_state.set(CanState::Sleep);
    }

    /// This function sends an 8-byte message
    pub fn send_8byte_message(
        &self,
        id: can::Id,
        dlc: usize,
        rtr: u8,
    ) -> Result<(), kernel::ErrorCode> {
        self.enable_irq(CanInterruptMode::ErrorAndStatusChangeInterrupt);
        if self.can_state.get() == CanState::Normal {
            if let Some(tx_mailbox) = self.find_empty_mailbox() {
                // set extended or standard id in registers
                match id {
                    can::Id::Standard(id) => {
                        self.registers.can_tx_mailbox[tx_mailbox]
                            .can_tir
                            .modify(CAN_TIxR::IDE::CLEAR);
                        self.registers.can_tx_mailbox[tx_mailbox]
                            .can_tir
                            .modify(CAN_TIxR::STID.val(id as u32 & 0xeff));
                        self.registers.can_tx_mailbox[tx_mailbox]
                            .can_tir
                            .modify(CAN_TIxR::EXID.val(0));
                    }
                    can::Id::Extended(id) => {
                        self.registers.can_tx_mailbox[tx_mailbox]
                            .can_tir
                            .modify(CAN_TIxR::IDE::SET);
                        self.registers.can_tx_mailbox[tx_mailbox]
                            .can_tir
                            .modify(CAN_TIxR::STID.val((id & 0xffc0000) >> 18));
                        self.registers.can_tx_mailbox[tx_mailbox]
                            .can_tir
                            .modify(CAN_TIxR::EXID.val(id & 0x003fffff));
                    }
                }
                // write rtr
                self.registers.can_tx_mailbox[tx_mailbox]
                    .can_tir
                    .modify(CAN_TIxR::RTR.val(rtr.into()));
                // write dlc
                self.registers.can_tx_mailbox[tx_mailbox]
                    .can_tdtr
                    .modify(CAN_TDTxR::DLC.val(dlc as u32));
                // write first 4 bytes of the data
                match self.tx_buffer.map(|tx| {
                    self.registers.can_tx_mailbox[tx_mailbox]
                        .can_tdlr
                        .modify(CAN_TDLxR::DATA0.val(tx[0].into()));
                    self.registers.can_tx_mailbox[tx_mailbox]
                        .can_tdlr
                        .modify(CAN_TDLxR::DATA1.val(tx[1].into()));
                    self.registers.can_tx_mailbox[tx_mailbox]
                        .can_tdlr
                        .modify(CAN_TDLxR::DATA2.val(tx[2].into()));
                    self.registers.can_tx_mailbox[tx_mailbox]
                        .can_tdlr
                        .modify(CAN_TDLxR::DATA3.val(tx[3].into()));
                    // write the last 4 bytes of the data
                    self.registers.can_tx_mailbox[tx_mailbox]
                        .can_tdhr
                        .modify(CAN_TDHxR::DATA4.val(tx[4].into()));
                    self.registers.can_tx_mailbox[tx_mailbox]
                        .can_tdhr
                        .modify(CAN_TDHxR::DATA5.val(tx[5].into()));
                    self.registers.can_tx_mailbox[tx_mailbox]
                        .can_tdhr
                        .modify(CAN_TDHxR::DATA6.val(tx[6].into()));
                    self.registers.can_tx_mailbox[tx_mailbox]
                        .can_tdhr
                        .modify(CAN_TDHxR::DATA7.val(tx[7].into()));

                    self.registers.can_tx_mailbox[tx_mailbox]
                        .can_tir
                        .modify(CAN_TIxR::TXRQ::SET);
                }) {
                    Some(_) => Ok(()),
                    None => Err(kernel::ErrorCode::FAIL),
                }
            } else {
                // no mailbox empty
                self.failed_messages.replace(self.failed_messages.get() + 1);
                Err(kernel::ErrorCode::BUSY)
            }
        } else {
            Err(kernel::ErrorCode::OFF)
        }
    }

    pub fn find_empty_mailbox(&self) -> Option<usize> {
        if self.registers.can_tsr.read(CAN_TSR::TME0) == 1 {
            Some(0)
        } else if self.registers.can_tsr.read(CAN_TSR::TME1) == 1 {
            Some(1)
        } else if self.registers.can_tsr.read(CAN_TSR::TME2) == 1 {
            Some(2)
        } else {
            None
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    /// Handle the transmit interrupt. Check the status register for each
    /// transmit mailbox to find out the mailbox that the message was sent from.
    pub fn handle_transmit_interrupt(&self) {
        let mut state = Ok(());
        if self.registers.can_esr.read(CAN_ESR::BOFF) == 1 {
            state = Err(can::Error::BusOff)
        } else {
            if self.registers.can_tsr.read(CAN_TSR::RQCP0) == 1 {
                // check status
                state = if self.registers.can_tsr.read(CAN_TSR::TXOK0) == 1 {
                    Ok(())
                } else if self.registers.can_tsr.read(CAN_TSR::TERR0) == 1 {
                    Err(can::Error::Transmission)
                } else if self.registers.can_tsr.read(CAN_TSR::ALST0) == 1 {
                    Err(can::Error::ArbitrationLost)
                } else {
                    Ok(())
                };
                // mark the interrupt as handled
                self.registers.can_tsr.modify(CAN_TSR::RQCP0::SET);
            }
            if self.registers.can_tsr.read(CAN_TSR::RQCP1) == 1 {
                state = if self.registers.can_tsr.read(CAN_TSR::TXOK1) == 1 {
                    Ok(())
                } else if self.registers.can_tsr.read(CAN_TSR::TERR1) == 1 {
                    Err(can::Error::Transmission)
                } else if self.registers.can_tsr.read(CAN_TSR::ALST1) == 1 {
                    Err(can::Error::ArbitrationLost)
                } else {
                    Ok(())
                };
                // mark the interrupt as handled
                self.registers.can_tsr.modify(CAN_TSR::RQCP1::SET);
            }
            if self.registers.can_tsr.read(CAN_TSR::RQCP2) == 1 {
                state = if self.registers.can_tsr.read(CAN_TSR::TXOK2) == 1 {
                    Ok(())
                } else if self.registers.can_tsr.read(CAN_TSR::TERR2) == 1 {
                    Err(can::Error::Transmission)
                } else if self.registers.can_tsr.read(CAN_TSR::ALST2) == 1 {
                    Err(can::Error::ArbitrationLost)
                } else {
                    Ok(())
                };
                // mark the interrupt as handled
                self.registers.can_tsr.modify(CAN_TSR::RQCP2::SET);
            }
        }

        match state {
            Err(err) => self.can_state.set(CanState::RunningError(err)),
            _ => {}
        }

        self.transmit_client
            .map(|transmit_client| match self.tx_buffer.take() {
                Some(buf) => transmit_client.transmit_complete(state, buf),
                None => {}
            });
    }

    pub fn process_received_message(
        &self,
        rx_mailbox: usize,
    ) -> (can::Id, usize, [u8; can::STANDARD_CAN_PACKET_SIZE]) {
        let message_id = if self.registers.can_rx_mailbox[rx_mailbox]
            .can_rir
            .read(CAN_RIxR::IDE)
            == 0
        {
            can::Id::Standard(
                self.registers.can_rx_mailbox[rx_mailbox]
                    .can_rir
                    .read(CAN_RIxR::STID) as u16,
            )
        } else {
            can::Id::Extended(
                (self.registers.can_rx_mailbox[rx_mailbox]
                    .can_rir
                    .read(CAN_RIxR::STID)
                    << 18)
                    | (self.registers.can_rx_mailbox[rx_mailbox]
                        .can_rir
                        .read(CAN_RIxR::EXID)),
            )
        };
        let message_length = self.registers.can_rx_mailbox[rx_mailbox]
            .can_rdtr
            .read(CAN_RDTxR::DLC) as usize;
        let recv: u64 = ((self.registers.can_rx_mailbox[0].can_rdhr.get() as u64) << 32)
            | (self.registers.can_rx_mailbox[0].can_rdlr.get() as u64);
        let rx_buf = recv.to_le_bytes();
        self.rx_buffer.map(|rx| {
            for i in 0..8 {
                rx[i] = rx_buf[i];
            }
        });

        (message_id, message_length, rx_buf)
    }

    pub fn handle_fifo0_interrupt(&self) {
        if self.registers.can_rf0r.read(CAN_RF0R::FULL0) == 1 {
            self.registers.can_rf0r.modify(CAN_RF0R::FULL0::SET);
        }

        if self.registers.can_rf0r.read(CAN_RF0R::FOVR0) == 1 {
            self.registers.can_rf0r.modify(CAN_RF0R::FOVR0::SET);
        }

        if self.registers.can_rf0r.read(CAN_RF0R::FMP0) != 0 {
            let (message_id, message_length, mut rx_buf) = self.process_received_message(0);

            self.receive_client.map(|receive_client| {
                receive_client.message_received(message_id, &mut rx_buf, message_length, Ok(()))
            });
            self.fifo0_interrupt_counter
                .replace(self.fifo0_interrupt_counter.get() + 1);

            // mark the interrupt as handled
            self.registers.can_rf0r.modify(CAN_RF0R::RFOM0::SET);
        }
    }

    pub fn handle_fifo1_interrupt(&self) {
        if self.registers.can_rf1r.read(CAN_RF1R::FULL1) == 1 {
            self.registers.can_rf1r.modify(CAN_RF1R::FULL1::SET);
        }

        if self.registers.can_rf1r.read(CAN_RF1R::FOVR1) == 1 {
            self.registers.can_rf1r.modify(CAN_RF1R::FOVR1::SET);
        }

        if self.registers.can_rf1r.read(CAN_RF1R::FMP1) != 0 {
            self.fifo1_interrupt_counter
                .replace(self.fifo1_interrupt_counter.get() + 1);
            let (message_id, message_length, mut rx_buf) = self.process_received_message(1);
            self.receive_client.map(|receive_client| {
                receive_client.message_received(message_id, &mut rx_buf, message_length, Ok(()))
            });

            // mark the interrupt as handled
            self.registers.can_rf1r.modify(CAN_RF1R::RFOM1::SET);
        }
    }

    pub fn handle_error_status_interrupt(&self) {
        // Check if there is a status change interrupt
        if self.registers.can_msr.read(CAN_MSR::WKUI) == 1 {
            // mark the interrupt as handled
            self.registers.can_msr.modify(CAN_MSR::WKUI::SET);
        }
        if self.registers.can_msr.read(CAN_MSR::SLAKI) == 1 {
            // mark the interrupt as handled
            self.registers.can_msr.modify(CAN_MSR::SLAKI::SET);
        }

        // Check if there is an error interrupt
        // Warning flag
        if self.registers.can_esr.read(CAN_ESR::EWGF) == 1 {
            self.can_state
                .set(CanState::RunningError(can::Error::Warning));
        }
        // Passive flag
        if self.registers.can_esr.read(CAN_ESR::EPVF) == 1 {
            self.can_state
                .set(CanState::RunningError(can::Error::Passive));
        }
        // Bus-off flag
        if self.registers.can_esr.read(CAN_ESR::BOFF) == 1 {
            self.can_state
                .set(CanState::RunningError(can::Error::BusOff));
        }
        // Last Error Code
        match self.registers.can_esr.read(CAN_ESR::LEC) {
            0x001 => self
                .can_state
                .set(CanState::RunningError(can::Error::Stuff)),
            0x010 => self.can_state.set(CanState::RunningError(can::Error::Form)),
            0x011 => self.can_state.set(CanState::RunningError(can::Error::Ack)),
            0x100 => self
                .can_state
                .set(CanState::RunningError(can::Error::BitRecessive)),
            0x101 => self
                .can_state
                .set(CanState::RunningError(can::Error::BitDominant)),
            0x110 => self.can_state.set(CanState::RunningError(can::Error::Crc)),
            0x111 => self
                .can_state
                .set(CanState::RunningError(can::Error::SetBySoftware)),
            _ => {}
        }

        self.error_interrupt_counter
            .replace(self.error_interrupt_counter.get() + 1);

        match self.can_state.get() {
            CanState::RunningError(err) => {
                self.controller_client.map(|controller_client| {
                    controller_client.state_changed(kernel::hil::can::State::Error(err));
                });
            }
            _ => {}
        }
    }

    pub fn enable_irq(&self, interrupt: CanInterruptMode) {
        match interrupt {
            CanInterruptMode::TransmitInterrupt => {
                self.registers.can_ier.modify(CAN_IER::TMEIE::SET);
            }
            CanInterruptMode::Fifo0Interrupt => {
                self.registers.can_ier.modify(CAN_IER::FMPIE0::SET);
                self.registers.can_ier.modify(CAN_IER::FFIE0::SET);
                self.registers.can_ier.modify(CAN_IER::FOVIE0::SET);
            }
            CanInterruptMode::Fifo1Interrupt => {
                self.registers.can_ier.modify(CAN_IER::FMPIE1::SET);
                self.registers.can_ier.modify(CAN_IER::FFIE1::SET);
                self.registers.can_ier.modify(CAN_IER::FOVIE1::SET);
            }
            CanInterruptMode::ErrorAndStatusChangeInterrupt => {
                self.registers.can_ier.modify(CAN_IER::ERRIE::SET);
                self.registers.can_ier.modify(CAN_IER::EWGIE::SET);
                self.registers.can_ier.modify(CAN_IER::EPVIE::SET);
                self.registers.can_ier.modify(CAN_IER::BOFIE::SET);
                self.registers.can_ier.modify(CAN_IER::LECIE::SET);
                self.registers.can_ier.modify(CAN_IER::WKUIE::SET);
                self.registers.can_ier.modify(CAN_IER::SLKIE::SET);
            }
        }
    }

    pub fn disable_irq(&self, interrupt: CanInterruptMode) {
        match interrupt {
            CanInterruptMode::TransmitInterrupt => {
                self.registers.can_ier.modify(CAN_IER::TMEIE::CLEAR);
            }
            CanInterruptMode::Fifo0Interrupt => {
                self.registers.can_ier.modify(CAN_IER::FMPIE0::CLEAR);
                self.registers.can_ier.modify(CAN_IER::FFIE0::CLEAR);
                self.registers.can_ier.modify(CAN_IER::FOVIE0::CLEAR);
            }
            CanInterruptMode::Fifo1Interrupt => {
                self.registers.can_ier.modify(CAN_IER::FMPIE1::CLEAR);
                self.registers.can_ier.modify(CAN_IER::FFIE1::CLEAR);
                self.registers.can_ier.modify(CAN_IER::FOVIE1::CLEAR);
            }
            CanInterruptMode::ErrorAndStatusChangeInterrupt => {
                self.registers.can_ier.modify(CAN_IER::ERRIE::CLEAR);
                self.registers.can_ier.modify(CAN_IER::EWGIE::CLEAR);
                self.registers.can_ier.modify(CAN_IER::EPVIE::CLEAR);
                self.registers.can_ier.modify(CAN_IER::BOFIE::CLEAR);
                self.registers.can_ier.modify(CAN_IER::LECIE::CLEAR);
                self.registers.can_ier.modify(CAN_IER::WKUIE::CLEAR);
                self.registers.can_ier.modify(CAN_IER::SLKIE::CLEAR);
            }
        }
    }

    pub fn enable_irqs(&self) {
        self.enable_irq(CanInterruptMode::TransmitInterrupt);
        self.enable_irq(CanInterruptMode::Fifo0Interrupt);
        self.enable_irq(CanInterruptMode::Fifo1Interrupt);
        self.enable_irq(CanInterruptMode::ErrorAndStatusChangeInterrupt);
    }

    pub fn disable_irqs(&self) {
        self.disable_irq(CanInterruptMode::TransmitInterrupt);
        self.disable_irq(CanInterruptMode::Fifo0Interrupt);
        self.disable_irq(CanInterruptMode::Fifo1Interrupt);
        self.disable_irq(CanInterruptMode::ErrorAndStatusChangeInterrupt);
    }
}

impl DeferredCallClient for Can<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self)
    }

    fn handle_deferred_call(&self) {
        match self.deferred_action.take() {
            Some(action) => match action {
                AsyncAction::Enable => {
                    if let Err(enable_err) = self.enter_normal_mode() {
                        self.controller_client.map(|controller_client| {
                            controller_client.state_changed(self.can_state.get().into());
                            controller_client.enabled(Err(enable_err));
                        });
                    }
                    self.controller_client.map(|controller_client| {
                        controller_client.state_changed(can::State::Running);
                        controller_client.enabled(Ok(()));
                    });
                }
                AsyncAction::AbortReceive => {
                    if let Some(rx) = self.rx_buffer.take() {
                        self.receive_client
                            .map(|receive_client| receive_client.stopped(rx));
                    }
                }
                AsyncAction::Disabled => {
                    self.controller_client.map(|controller_client| {
                        controller_client.state_changed(self.can_state.get().into());
                        controller_client.disabled(Ok(()));
                    });
                }
                AsyncAction::EnableError(err) => {
                    self.controller_client.map(|controller_client| {
                        controller_client.state_changed(self.can_state.get().into());
                        controller_client.enabled(Err(err));
                    });
                }
            },
            // todo no action set
            None => todo!(),
        }
    }
}

struct CanClock<'a>(rcc::PeripheralClock<'a>);

impl ClockInterface for CanClock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}

impl<'a> can::Configure for Can<'_> {
    const MIN_BIT_TIMINGS: can::BitTiming = can::BitTiming {
        segment1: BitSegment1::CanBtrTs1Min as u8,
        segment2: BitSegment2::CanBtrTs2Min as u8,
        propagation: 0,
        sync_jump_width: SynchronizationJumpWidth::CanBtrSjwMin as u32,
        baud_rate_prescaler: BRP_MIN_STM32,
    };

    const MAX_BIT_TIMINGS: can::BitTiming = can::BitTiming {
        segment1: BitSegment1::CanBtrTs1Max as u8,
        segment2: BitSegment2::CanBtrTs2Max as u8,
        propagation: 0,
        sync_jump_width: SynchronizationJumpWidth::CanBtrSjwMax as u32,
        baud_rate_prescaler: BRP_MAX_STM32,
    };

    const SYNC_SEG: u8 = 1;

    fn set_bitrate(&self, bitrate: u32) -> Result<(), kernel::ErrorCode> {
        let bit_timing = Self::bit_timing_for_bitrate(16_000_000, bitrate)?;
        self.set_bit_timing(bit_timing)
    }

    fn set_bit_timing(&self, bit_timing: can::BitTiming) -> Result<(), kernel::ErrorCode> {
        match self.can_state.get() {
            CanState::Sleep => {
                self.bit_timing.set(bit_timing);
                Ok(())
            }
            CanState::Normal | CanState::Initialization | CanState::RunningError(_) => {
                Err(kernel::ErrorCode::BUSY)
            }
        }
    }

    fn set_operation_mode(&self, mode: can::OperationMode) -> Result<(), kernel::ErrorCode> {
        match self.can_state.get() {
            CanState::Sleep => {
                self.operating_mode.set(mode);
                Ok(())
            }
            CanState::Normal | CanState::Initialization | CanState::RunningError(_) => {
                Err(kernel::ErrorCode::BUSY)
            }
        }
    }

    fn get_bit_timing(&self) -> Result<can::BitTiming, kernel::ErrorCode> {
        if let Some(bit_timing) = self.bit_timing.extract() {
            Ok(bit_timing)
        } else {
            Err(kernel::ErrorCode::INVAL)
        }
    }

    fn get_operation_mode(&self) -> Result<can::OperationMode, kernel::ErrorCode> {
        if let Some(operation_mode) = self.operating_mode.extract() {
            Ok(operation_mode)
        } else {
            Err(kernel::ErrorCode::INVAL)
        }
    }

    fn set_automatic_retransmission(&self, automatic: bool) -> Result<(), kernel::ErrorCode> {
        match self.can_state.get() {
            CanState::Sleep => {
                self.automatic_retransmission.replace(automatic);
                Ok(())
            }
            CanState::Normal | CanState::Initialization | CanState::RunningError(_) => {
                Err(kernel::ErrorCode::BUSY)
            }
        }
    }

    fn set_wake_up(&self, wake_up: bool) -> Result<(), kernel::ErrorCode> {
        match self.can_state.get() {
            CanState::Sleep => {
                self.automatic_wake_up.replace(wake_up);
                Ok(())
            }
            CanState::Normal | CanState::Initialization | CanState::RunningError(_) => {
                Err(kernel::ErrorCode::BUSY)
            }
        }
    }

    fn get_automatic_retransmission(&self) -> Result<bool, kernel::ErrorCode> {
        Ok(self.automatic_retransmission.get())
    }

    fn get_wake_up(&self) -> Result<bool, kernel::ErrorCode> {
        Ok(self.automatic_wake_up.get())
    }

    fn receive_fifo_count(&self) -> usize {
        2
    }
}

impl<'a> can::Controller for Can<'_> {
    fn set_client(&self, client: Option<&'static dyn can::ControllerClient>) {
        if let Some(client) = client {
            self.controller_client.replace(client);
        } else {
            self.controller_client.clear();
        }
    }

    fn enable(&self) -> Result<(), kernel::ErrorCode> {
        match self.can_state.get() {
            CanState::Sleep => {
                if self.bit_timing.is_none() || self.operating_mode.is_none() {
                    Err(kernel::ErrorCode::INVAL)
                } else {
                    let r = self.enable();
                    // there is another deferred action that must be completed
                    if self.deferred_action.is_some() {
                        Err(kernel::ErrorCode::BUSY)
                    } else {
                        // set an Enable or an EnableError deferred action
                        match r {
                            Ok(_) => {
                                self.deferred_action.set(AsyncAction::Enable);
                            }
                            Err(err) => {
                                self.deferred_action.set(AsyncAction::EnableError(err));
                            }
                        }
                        self.deferred_call.set();
                        r
                    }
                }
            }
            CanState::Normal | CanState::Initialization => Err(kernel::ErrorCode::ALREADY),
            CanState::RunningError(_) => Err(kernel::ErrorCode::FAIL),
        }
    }

    fn disable(&self) -> Result<(), kernel::ErrorCode> {
        match self.can_state.get() {
            CanState::Normal | CanState::RunningError(_) => {
                self.enter_sleep_mode();
                if self.deferred_action.is_some() {
                    // there is another deferred action that must be completed
                    return Err(kernel::ErrorCode::BUSY);
                } else {
                    // set a Disable deferred action
                    self.deferred_action.set(AsyncAction::Disabled);
                    self.deferred_call.set();
                }
                Ok(())
            }
            CanState::Sleep | CanState::Initialization => Err(kernel::ErrorCode::OFF),
        }
    }

    fn get_state(&self) -> Result<can::State, kernel::ErrorCode> {
        Ok(self.can_state.get().into())
    }
}

impl<'a> can::Transmit<{ can::STANDARD_CAN_PACKET_SIZE }> for Can<'_> {
    fn set_client(
        &self,
        client: Option<&'static dyn can::TransmitClient<{ can::STANDARD_CAN_PACKET_SIZE }>>,
    ) {
        if let Some(client) = client {
            self.transmit_client.set(client);
        } else {
            self.transmit_client.clear();
        }
    }

    fn send(
        &self,
        id: can::Id,
        buffer: &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
        len: usize,
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
        ),
    > {
        match self.can_state.get() {
            CanState::Normal | CanState::RunningError(_) => {
                self.tx_buffer.replace(buffer);
                self.enable_irq(CanInterruptMode::TransmitInterrupt);
                self.can_state.set(CanState::Normal);
                match self.send_8byte_message(id, len, 0) {
                    Ok(_) => Ok(()),
                    Err(err) => Err((err, self.tx_buffer.take().unwrap())),
                }
            }
            CanState::Sleep | CanState::Initialization => Err((kernel::ErrorCode::OFF, buffer)),
        }
    }
}

impl<'a> can::Receive<{ can::STANDARD_CAN_PACKET_SIZE }> for Can<'_> {
    fn set_client(
        &self,
        client: Option<&'static dyn can::ReceiveClient<{ can::STANDARD_CAN_PACKET_SIZE }>>,
    ) {
        if let Some(client) = client {
            self.receive_client.set(client);
        } else {
            self.receive_client.clear();
        }
    }

    fn start_receive_process(
        &self,
        buffer: &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
        ),
    > {
        match self.can_state.get() {
            CanState::Normal | CanState::RunningError(_) => {
                self.can_state.set(CanState::Normal);
                self.config_filter(
                    can::FilterParameters {
                        number: 0,
                        scale_bits: can::ScaleBits::Bits32,
                        identifier_mode: can::IdentifierMode::Mask,
                        fifo_number: 0,
                    },
                    true,
                );
                self.config_filter(
                    can::FilterParameters {
                        number: 1,
                        scale_bits: can::ScaleBits::Bits32,
                        identifier_mode: can::IdentifierMode::Mask,
                        fifo_number: 1,
                    },
                    true,
                );
                self.enable_filter_config();
                self.enable_irq(CanInterruptMode::Fifo0Interrupt);
                self.enable_irq(CanInterruptMode::Fifo1Interrupt);
                self.rx_buffer.put(Some(buffer));
                Ok(())
            }
            CanState::Sleep | CanState::Initialization => Err((kernel::ErrorCode::OFF, buffer)),
        }
    }

    fn stop_receive(&self) -> Result<(), kernel::ErrorCode> {
        match self.can_state.get() {
            CanState::Normal | CanState::RunningError(_) => {
                self.can_state.set(CanState::Normal);
                self.config_filter(
                    can::FilterParameters {
                        number: 0,
                        scale_bits: can::ScaleBits::Bits32,
                        identifier_mode: can::IdentifierMode::Mask,
                        fifo_number: 0,
                    },
                    false,
                );
                self.config_filter(
                    can::FilterParameters {
                        number: 1,
                        scale_bits: can::ScaleBits::Bits32,
                        identifier_mode: can::IdentifierMode::Mask,
                        fifo_number: 1,
                    },
                    false,
                );
                self.enable_filter_config();
                self.disable_irq(CanInterruptMode::Fifo0Interrupt);
                self.disable_irq(CanInterruptMode::Fifo1Interrupt);
                // there is another deferred action that must be completed
                if self.deferred_action.is_some() {
                    Err(kernel::ErrorCode::BUSY)
                // the chip does not own the buffer from the capsule
                } else if self.rx_buffer.is_none() {
                    Err(kernel::ErrorCode::SIZE)
                } else {
                    // set a AbortReceive deferred action
                    self.deferred_action.set(AsyncAction::AbortReceive);
                    self.deferred_call.set();
                    Ok(())
                }
            }
            CanState::Sleep | CanState::Initialization => Err(kernel::ErrorCode::OFF),
        }
    }
}
