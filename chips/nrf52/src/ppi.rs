// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Programmable peripheral interconnect, nRF52
//!
//! Chapter 20 of the nRF52832 Objective Product Specification v0.6.3:
//!
//! The PPI provides a mechanism to automatically trigger a task in one peripheral
//! as a result of an event occurring in another peripheral. A task is connected to
//! an event through a PPI channel.
//! The PPI channel is composed of three end point registers, one event end point (EEP)
//! and two task end points (TEP).
//! A peripheral task is connected to a TEP using the address of the task register
//! associated with the task. Similarly, a peripheral event is connected to an EEP using
//! the address of the event register associated with the event.
//!
//! Pre-programmed Channels
//! (Channel EEP TEP):
//!
//! * 20        `TIMER0->EVENTS_COMPARE[0]`       `RADIO->TASKS_TXEN`
//! * 21        `TIMER0->EVENTS_COMPARE[0]`       `RADIO->TASKS_RXEN`
//! * 22        `TIMER0->EVENTS_COMPARE[1]`       `RADIO->TASKS_DISABLE`
//! * 23        `RADIO->EVENTS_BCMATCH`           `AAR->TASKS_START`
//! * 24        `RADIO->EVENTS_READY`             `CCM->TASKS_KSGEN`
//! * 25        `RADIO->EVENTS_ADDRESS`           `CCM->TASKS_CRYPT`
//! * 26        `RADIO->EVENTS_ADDRESS`           `TIMER0->TASKS_CAPTURE[1]`
//! * 27        `RADIO->EVENTS_END`               `TIMER0->TASKS_CAPTURE[2]`
//! * 28        `RTC0->EVENTS_COMPARE[0]`         `RADIO->TASKS_TXEN`
//! * 29        `RTC0->EVENTS_COMPARE[0]`         `RADIO->TASKS_RXEN`
//! * 30        `RTC0->EVENTS_COMPARE[0]`         `TIMER0->TASKS_CLEAR`
//! * 31        `RTC0->EVENTS_COMPARE[0]`         `TIMER0->TASKS_START`
//!
//! Authors
//! ---------
//! * Johan Lindskogen
//! * Francine Mäkelä
//! * Date: May 04, 2018

use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{FieldValue, ReadWrite, register_bitfields, register_structs};

register_structs! {
    pub PpiRegisters {
        (0x000 => tasks_chg0_en: ReadWrite<u32, Control::Register>),
        (0x004 => tasks_chg0_dis: ReadWrite<u32, Control::Register>),
        (0x008 => tasks_chg1_en: ReadWrite<u32, Control::Register>),
        (0x00C => tasks_chg1_dis: ReadWrite<u32, Control::Register>),
        (0x010 => tasks_chg2_en: ReadWrite<u32, Control::Register>),
        (0x014 => tasks_chg2_dis: ReadWrite<u32, Control::Register>),
        (0x018 => tasks_chg3_en: ReadWrite<u32, Control::Register>),
        (0x01C => tasks_chg3_dis: ReadWrite<u32, Control::Register>),
        (0x020 => tasks_chg4_en: ReadWrite<u32, Control::Register>),
        (0x024 => tasks_chg4_dis: ReadWrite<u32, Control::Register>),
        (0x028 => tasks_chg5_en: ReadWrite<u32, Control::Register>),
        (0x02C => tasks_chg5_dis: ReadWrite<u32, Control::Register>),
        (0x030 => _reserved1),
        (0x500 => chen: ReadWrite<u32, Channel::Register>),
        (0x504 => chenset: ReadWrite<u32, Channel::Register>),
        (0x508 => chenclr: ReadWrite<u32, Channel::Register>),
        (0x50C => ch0_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x510 => ch0_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x514 => ch1_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x518 => ch1_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x51C => ch2_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x520 => ch2_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x524 => ch3_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x528 => ch3_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x52C => ch4_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x530 => ch4_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x534 => ch5_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x538 => ch5_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x53C => ch6_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x540 => ch6_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x544 => ch7_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x548 => ch7_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x54C => ch8_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x550 => ch8_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x554 => ch9_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x558 => ch9_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x55C => ch10_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x560 => ch10_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x564 => ch11_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x568 => ch11_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x56C => ch12_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x570 => ch12_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x574 => ch13_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x578 => ch13_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x57C => ch14_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x580 => ch14_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x584 => ch15_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x588 => ch15_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x58C => ch16_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x590 => ch16_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x594 => ch17_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x598 => ch17_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x59C => ch18_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x5A0 => ch18_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x5A4 => ch19_eep: ReadWrite<u32, EventEndPoint::Register>),
        (0x5A8 => ch19_tep: ReadWrite<u32, TaskEndPoint::Register>),
        (0x5AC => _reserved2),
        (0x7FC => chg: [ReadWrite<u32, Channel::Register>; 6]),
        (0x814 => _reserved3),
        (0x90C => fork_tep: [ReadWrite<u32, TaskEndPoint::Register>; 32]),
        (0x98C => @END),
    }
}

register_bitfields! [u32,
    Control [
        ENABLE OFFSET(0) NUMBITS(1)
    ],
    pub Channel [
         CH0 OFFSET(0) NUMBITS(1),
         CH1 OFFSET(1) NUMBITS(1),
         CH2 OFFSET(2) NUMBITS(1),
         CH3 OFFSET(3) NUMBITS(1),
         CH4 OFFSET(4) NUMBITS(1),
         CH5 OFFSET(5) NUMBITS(1),
         CH6 OFFSET(6) NUMBITS(1),
         CH7 OFFSET(7) NUMBITS(1),
         CH8 OFFSET(8) NUMBITS(1),
         CH9 OFFSET(9) NUMBITS(1),
         CH10 OFFSET(10) NUMBITS(1),
         CH11 OFFSET(11) NUMBITS(1),
         CH12 OFFSET(12) NUMBITS(1),
         CH13 OFFSET(13) NUMBITS(1),
         CH14 OFFSET(14) NUMBITS(1),
         CH15 OFFSET(15) NUMBITS(1),
         CH16 OFFSET(16) NUMBITS(1),
         CH17 OFFSET(17) NUMBITS(1),
         CH18 OFFSET(18) NUMBITS(1),
         CH19 OFFSET(19) NUMBITS(1),
         CH20 OFFSET(20) NUMBITS(1),
         CH21 OFFSET(21) NUMBITS(1),
         CH22 OFFSET(22) NUMBITS(1),
         CH23 OFFSET(23) NUMBITS(1),
         CH24 OFFSET(24) NUMBITS(1),
         CH25 OFFSET(25) NUMBITS(1),
         CH26 OFFSET(26) NUMBITS(1),
         CH27 OFFSET(27) NUMBITS(1),
         CH28 OFFSET(28) NUMBITS(1),
         CH29 OFFSET(29) NUMBITS(1),
         CH30 OFFSET(30) NUMBITS(1),
         CH31 OFFSET(31) NUMBITS(1)
    ],
    TaskEndPoint [
        ADDRESS OFFSET(0) NUMBITS(32)
    ],
    EventEndPoint [
        ADDRESS OFFSET(0) NUMBITS(32)
    ]
];

pub struct Ppi {
    registers: StaticRef<PpiRegisters>,
}

impl Ppi {
    pub const fn new(registers: StaticRef<PpiRegisters>) -> Ppi {
        Ppi { registers }
    }

    pub fn enable(&self, channels: FieldValue<u32, Channel::Register>) {
        self.registers.chenset.write(channels);
    }

    pub fn disable(&self, channels: FieldValue<u32, Channel::Register>) {
        self.registers.chenclr.write(channels);
    }
}
