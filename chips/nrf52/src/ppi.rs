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
//!     * 20        TIMER0->EVENTS_COMPARE[0]       RADIO->TASKS_TXEN
//!     * 21        TIMER0->EVENTS_COMPARE[0]       RADIO->TASKS_RXEN
//!     * 22        TIMER0->EVENTS_COMPARE[1]       RADIO->TASKS_DISABLE
//!     * 23        RADIO->EVENTS_BCMATCH           AAR->TASKS_START
//!     * 24        RADIO->EVENTS_READY             CCM->TASKS_KSGEN
//!     * 25        RADIO->EVENTS_ADDRESS           CCM->TASKS_CRYPT
//!     * 26        RADIO->EVENTS_ADDRESS           TIMER0->TASKS_CAPTURE[1]
//!     * 27        RADIO->EVENTS_END               TIMER0->TASKS_CAPTURE[2]
//!     * 28        RTC0->EVENTS_COMPARE[0]         RADIO->TASKS_TXEN
//!     * 29        RTC0->EVENTS_COMPARE[0]         RADIO->TASKS_RXEN
//!     * 30        RTC0->EVENTS_COMPARE[0]         TIMER0->TASKS_CLEAR
//!     * 31        RTC0->EVENTS_COMPARE[0]         TIMER0->TASKS_START
//!
//! Authors
//! ---------
//! * Johan Lindskogen
//! * Francine Mäkelä
//! * Date: May 04, 2018

use kernel::common::registers::{FieldValue, ReadWrite};
use kernel::common::StaticRef;

const PPI_BASE: StaticRef<PpiRegisters> =
    unsafe { StaticRef::new(0x4001F000 as *const PpiRegisters) };

#[repr(C)]
struct PpiRegisters {
    tasks_chg0_en: ReadWrite<u32, Control::Register>,
    tasks_chg0_dis: ReadWrite<u32, Control::Register>,
    tasks_chg1_en: ReadWrite<u32, Control::Register>,
    tasks_chg1_dis: ReadWrite<u32, Control::Register>,
    tasks_chg2_en: ReadWrite<u32, Control::Register>,
    tasks_chg2_dis: ReadWrite<u32, Control::Register>,
    tasks_chg3_en: ReadWrite<u32, Control::Register>,
    tasks_chg3_dis: ReadWrite<u32, Control::Register>,
    tasks_chg4_en: ReadWrite<u32, Control::Register>,
    tasks_chg4_dis: ReadWrite<u32, Control::Register>,
    tasks_chg5_en: ReadWrite<u32, Control::Register>,
    tasks_chg5_dis: ReadWrite<u32, Control::Register>,
    _reserved1: [u32; 308],
    chen: ReadWrite<u32, Channel::Register>,
    chenset: ReadWrite<u32, Channel::Register>,
    chenclr: ReadWrite<u32, Channel::Register>,
    ch0_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch0_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch1_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch1_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch2_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch2_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch3_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch3_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch4_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch4_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch5_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch5_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch6_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch6_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch7_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch7_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch8_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch8_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch9_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch9_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch10_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch10_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch11_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch11_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch12_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch12_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch13_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch13_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch14_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch14_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch15_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch15_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch16_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch16_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch17_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch17_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch18_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch18_tep: ReadWrite<u32, TaskEndPoint::Register>,
    ch19_eep: ReadWrite<u32, EventEndPoint::Register>,
    ch19_tep: ReadWrite<u32, TaskEndPoint::Register>,
    _reserved2: [u32; 148],
    chg: [ReadWrite<u32, Channel::Register>; 6],
    _reserved3: [u32; 62],
    fork_tep: [ReadWrite<u32, TaskEndPoint::Register>; 32],
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
        ADDRESS OFFSET(0) NUMBITS(32)
    ],
    EventEndPoint [
        ADDRESS OFFSET(0) NUMBITS(32)
    ]
];

pub struct Ppi {
    registers: StaticRef<PpiRegisters>,
}

pub static mut PPI: Ppi = Ppi::new();

impl Ppi {
    pub const fn new() -> Ppi {
        Ppi {
            registers: PPI_BASE,
        }
    }

    pub fn enable(&self, channels: FieldValue<u32, Channel::Register>) {
        let regs = &*self.registers;
        regs.chenset.write(channels);
    }

    pub fn disable(&self, channels: FieldValue<u32, Channel::Register>) {
        let regs = &*self.registers;
        regs.chenclr.write(channels);
    }
}
