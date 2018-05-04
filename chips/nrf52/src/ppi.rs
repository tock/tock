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
//!
//!
//! Pre-programmed Channels:
//! Channel	EEP	                        TEP
//! 20	    TIMER0->EVENTS_COMPARE[0]	RADIO->TASKS_TXEN
//! 21	    TIMER0->EVENTS_COMPARE[0]	RADIO->TASKS_RXEN
//! 22	    TIMER0->EVENTS_COMPARE[1]	RADIO->TASKS_DISABLE
//! 23	    RADIO->EVENTS_BCMATCH	    AAR->TASKS_START
//! 24	    RADIO->EVENTS_READY	        CCM->TASKS_KSGEN
//! 25	    RADIO->EVENTS_ADDRESS	    CCM->TASKS_CRYPT
//! 26	    RADIO->EVENTS_ADDRESS	    TIMER0->TASKS_CAPTURE[1]
//! 27	    RADIO->EVENTS_END	        TIMER0->TASKS_CAPTURE[2]
//! 28	    RTC0->EVENTS_COMPARE[0]	    RADIO->TASKS_TXEN
//! 29	    RTC0->EVENTS_COMPARE[0]	    RADIO->TASKS_RXEN
//! 30	    RTC0->EVENTS_COMPARE[0]	    TIMER0->TASKS_CLEAR
//! 31	    RTC0->EVENTS_COMPARE[0]	    TIMER0->TASKS_START
//!
//! //! Author
//! ---------
//! * Johan Lindskogen
//! * Francine Mäkelä
//! * Date: May 04, 2018

use kernel::common::regs::ReadWrite;
use kernel::common::regs::FieldValue;

pub const PPI_BASE: usize = 0x4001F000;

pub struct PPIRegs {
    pub tasks_chg0_en: ReadWrite<u32, Control::Register>, //0x000
    pub tasks_chg0_dis: ReadWrite<u32, Control::Register>, //0x004
    pub tasks_chg1_en: ReadWrite<u32, Control::Register>, //0x008
    pub tasks_chg1_dis: ReadWrite<u32, Control::Register>, //0x00C
    pub tasks_chg2_en: ReadWrite<u32, Control::Register>, //0x010
    pub tasks_chg2_dis: ReadWrite<u32, Control::Register>, //0x014
    pub tasks_chg3_en: ReadWrite<u32, Control::Register>, //0x018
    pub tasks_chg3_dis: ReadWrite<u32, Control::Register>, //0x01C
    pub tasks_chg4_en: ReadWrite<u32, Control::Register>, //0x020
    pub tasks_chg4_dis: ReadWrite<u32, Control::Register>, //0x024
    pub tasks_chg5_en: ReadWrite<u32, Control::Register>, //0x028
    pub tasks_chg5_dis: ReadWrite<u32, Control::Register>, //0x02C
    _reserved1: [u32; 308], //0x02C - 0x500
    pub chen: ReadWrite<u32, Channel::Register>, //0x500
    pub chenset: ReadWrite<u32, Channel::Register>,  //0x504
    pub chenclr: ReadWrite<u32, Channel::Register>,  //0x508
    pub ch0_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x510
    pub ch0_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x514
    pub ch1_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x518
    pub ch1_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x51C
    pub ch2_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x520
    pub ch2_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x524
    pub ch3_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x528
    pub ch3_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x52C
    pub ch4_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x530
    pub ch4_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x534
    pub ch5_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x538
    pub ch5_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x53C
    pub ch6_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x540
    pub ch6_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x544
    pub ch7_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x548
    pub ch7_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x54C
    pub ch8_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x550
    pub ch8_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x554
    pub ch9_eep: ReadWrite<u32, EventEndPoint::Register>,  //0x558
    pub ch9_tep: ReadWrite<u32, TaskEndPoint::Register>,  //0x55C
    pub ch10_eep: ReadWrite<u32,EventEndPoint::Register>, //0x560
    pub ch10_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x564
    pub ch11_eep: ReadWrite<u32,EventEndPoint::Register>, //0x568
    pub ch11_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x56C
    pub ch12_eep: ReadWrite<u32,EventEndPoint::Register>, //0x570
    pub ch12_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x574
    pub ch13_eep: ReadWrite<u32,EventEndPoint::Register>, //0x578
    pub ch13_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x57C
    pub ch14_eep: ReadWrite<u32,EventEndPoint::Register>, //0x580
    pub ch14_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x584
    pub ch15_eep: ReadWrite<u32,EventEndPoint::Register>, //0x588
    pub ch15_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x58C
    pub ch16_eep: ReadWrite<u32,EventEndPoint::Register>, //0x590
    pub ch16_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x594
    pub ch17_eep: ReadWrite<u32,EventEndPoint::Register>, //0x598
    pub ch17_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x59C
    pub ch18_eep: ReadWrite<u32,EventEndPoint::Register>, //0x5A0
    pub ch18_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x5A4
    pub ch19_eep: ReadWrite<u32,EventEndPoint::Register>, //0x5A8
    pub ch19_tep: ReadWrite<u32, TaskEndPoint::Register>, //0x5AC
    _reserved2: [u32; 148], //0x5AC - 0x800
    pub chg: [ReadWrite<u32, Channel::Register>; 6], //0x800 - 0x814
    _reserved3: [u32; 62],  //0x814 - 0x910
    pub fork_tep: [ReadWrite<u32, TaskEndPoint::Register>; 32],   //0x910 - 0x98C
}

register_bitfields! [u32,
    Control [
        ENABLE OFFSET(0) NUMBITS(1)
    ],
    Channel [
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
        ENABLE OFFSET(0) NUMBITS(1)
    ],
    EventEndPoint [
        ENABLE OFFSET(0) NUMBITS(1)
    ]
];

pub struct PPIStruct {
    regs: *const PPIRegs,
}

pub static mut PPI: PPIStruct = PPIStruct::new();

impl PPIStruct {
    pub const fn new() -> PPIStruct {
        PPIStruct {
            regs: PPI_BASE as *const PPIRegs,
        }
    }

    pub fn enable(&self, channels: FieldValue<u32, Channel::Register>) {
        let regs = unsafe { &*self.regs };
        regs.chenset.write(channels);
    }

    pub fn disable(&self, channels: FieldValue<u32, Channel::Register>) {
        let regs = unsafe { &*self.regs };
        regs.chenclr.write(channels);
    }
}



