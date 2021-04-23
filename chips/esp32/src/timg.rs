//! TimG Group driver.

use kernel::common::cells::OptionalCell;
use kernel::common::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::common::registers::register_bitfields;
use kernel::common::registers::{register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::time;
use kernel::hil::time::{Alarm, Counter, Ticks64};
use kernel::ErrorCode;

pub const TIMG0_BASE: StaticRef<TimgRegisters> =
    unsafe { StaticRef::new(0x6001_F000 as *const TimgRegisters) };

pub const TIMG1_BASE: StaticRef<TimgRegisters> =
    unsafe { StaticRef::new(0x6002_0000 as *const TimgRegisters) };

/// 80MHz `Frequency`
#[derive(Debug)]
pub struct Freq80MHz;
impl time::Frequency for Freq80MHz {
    fn frequency() -> u32 {
        80_000_000
    }
}

register_structs! {
    pub TimgRegisters {
        (0x000 => t0config: ReadWrite<u32, CONFIG::Register>),
        (0x004 => t0lo: ReadWrite<u32>),
        (0x008 => t0hi: ReadWrite<u32>),
        (0x00C => t0update: ReadWrite<u32>),
        (0x010 => t0alarmlo: ReadWrite<u32>),
        (0x014 => t0alarmhi: ReadWrite<u32>),
        (0x018 => t0loadlo: ReadWrite<u32>),
        (0x01C => _reserved0),
        (0x020 => t0load: ReadWrite<u32>),

        (0x024 => t1config: ReadWrite<u32, CONFIG::Register>),
        (0x028 => t1lo: ReadWrite<u32>),
        (0x02C => t1hi: ReadWrite<u32>),
        (0x030 => t1update: ReadWrite<u32>),
        (0x034 => t1alarmlo: ReadWrite<u32>),
        (0x038 => t1alarmhi: ReadWrite<u32>),
        (0x03C => t1loadlo: ReadWrite<u32>),
        (0x040 => _reserved1),
        (0x044 => t1load: ReadWrite<u32>),

        (0x048 => wdtconfig0: ReadWrite<u32, WDTCONFIG0::Register>),
        (0x04C => wdtconfig1: ReadWrite<u32, WDTCONFIG1::Register>),
        (0x050 => wdtconfig2: ReadWrite<u32>),
        (0x054 => wdtconfig3: ReadWrite<u32>),
        (0x058 => wdtconfig4: ReadWrite<u32>),
        (0x05C => wdtconfig5: ReadWrite<u32>),
        (0x060 => wdtfeed: ReadWrite<u32>),
        (0x064 => wdtwprotect: ReadWrite<u32>),

        (0x068 => rtccalicfg: ReadWrite<u32, RTCCALICFG::Register>),
        (0x06C => rtccalicfg1: ReadWrite<u32, RTCCALICFG1::Register>),

        (0x070 => _reserved2),
        (0x098 => int_ena: ReadWrite<u32, INT::Register>),
        (0x09C => int_raw: ReadWrite<u32, INT::Register>),
        (0x0A0 => int_st: ReadWrite<u32, INT::Register>),
        (0x0A4 => int_clr: ReadWrite<u32, INT::Register>),
        (0x0A8 => @END),
    }
}

register_bitfields![u32,
    CONFIG [
        ALARM_EN OFFSET(10) NUMBITS(1) [],
        LEVEL_INT_EN OFFSET(11) NUMBITS(1) [],
        EDGE_INT_EN OFFSET(12) NUMBITS(1) [],
        DIVIDER OFFSET(13) NUMBITS(16) [],
        AUTORELOAD OFFSET(29) NUMBITS(1) [],
        INCREATE OFFSET(30) NUMBITS(1) [],
        EN OFFSET(31) NUMBITS(1) [],
    ],
    WDTCONFIG0 [
        FLASHBOOT_MOD_EN OFFSET(14) NUMBITS(1) [],
        SYS_RESET_LENGTH OFFSET(15) NUMBITS(3) [],
        CPU_RESET_LENGTH OFFSET(18) NUMBITS(3) [],
        LEVEL_INT_EN OFFSET(21) NUMBITS(1) [],
        EDGE_INT_EN OFFSET(22) NUMBITS(1) [],
        STG3 OFFSET(23) NUMBITS(2) [],
        STG2 OFFSET(25) NUMBITS(2) [],
        STG1 OFFSET(27) NUMBITS(2) [],
        STG0 OFFSET(29) NUMBITS(2) [],
        EN OFFSET(31) NUMBITS(1) [],
    ],
    WDTCONFIG1 [
        CLK_PRESCALE OFFSET(16) NUMBITS(16) [],
    ],
    RTCCALICFG [
        START_CYCLING OFFSET(12) NUMBITS(1) [],
        CLK_SEL OFFSET(13) NUMBITS(2) [],
        RDY OFFSET(15) NUMBITS(1) [],
        MAX OFFSET(16) NUMBITS(15) [],
        START OFFSET(31) NUMBITS(1) [],
    ],
    RTCCALICFG1 [
        VALUE OFFSET(7) NUMBITS(25) [],
    ],
    INT [
        T0 OFFSET(0) NUMBITS(1) [],
        T1 OFFSET(1) NUMBITS(1) [],
        WDT OFFSET(2) NUMBITS(1) [],
    ],
];

pub struct TimG<'a> {
    registers: StaticRef<TimgRegisters>,
    alarm_client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl TimG<'_> {
    pub const fn new(base: StaticRef<TimgRegisters>) -> Self {
        TimG {
            registers: base,
            alarm_client: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
        let _ = self.stop();
        self.alarm_client.map(|client| {
            client.alarm();
        });
    }

    pub fn disable_wdt(&self) {
        self.registers
            .wdtconfig0
            .modify(WDTCONFIG0::EN::CLEAR + WDTCONFIG0::FLASHBOOT_MOD_EN::CLEAR);

        if self.registers.wdtconfig0.is_set(WDTCONFIG0::EN)
            || self
                .registers
                .wdtconfig0
                .is_set(WDTCONFIG0::FLASHBOOT_MOD_EN)
        {
            panic!("Can't disable TIMG WDT");
        }
    }
}

impl time::Time for TimG<'_> {
    type Frequency = Freq80MHz;
    type Ticks = Ticks64;

    fn now(&self) -> Self::Ticks {
        self.registers.t0update.set(0xABC);
        Self::Ticks::from(
            self.registers.t0lo.get() as u64 + ((self.registers.t0hi.get() as u64) << 32),
        )
    }
}

impl<'a> Counter<'a> for TimG<'a> {
    fn set_overflow_client(&'a self, _client: &'a dyn time::OverflowClient) {
        // We have no way to know when this happens
    }

    fn start(&self) -> Result<(), ErrorCode> {
        self.registers.t0config.write(CONFIG::EN::SET);

        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        self.registers.t0config.write(CONFIG::EN::CLEAR);

        Ok(())
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn is_running(&self) -> bool {
        self.registers.t0config.is_set(CONFIG::EN)
    }
}

impl<'a> Alarm<'a> for TimG<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.alarm_client.set(client);
    }

    fn set_alarm(&self, _reference: Self::Ticks, _dt: Self::Ticks) {
        panic!("Unimplemented");
    }

    fn get_alarm(&self) -> Self::Ticks {
        panic!("Unimplemented");
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        panic!("Unimplemented");
    }

    fn is_armed(&self) -> bool {
        panic!("Unimplemented");
    }

    fn minimum_dt(&self) -> Self::Ticks {
        Self::Ticks::from(1 as u64)
    }
}

impl kernel::SchedulerTimer for TimG<'_> {
    fn start(&self, _us: u32) {
        panic!("Unimplemented");
    }

    fn reset(&self) {
        panic!("Unimplemented");
    }

    fn arm(&self) {
        panic!("Unimplemented");
    }

    fn disarm(&self) {
        panic!("Unimplemented");
    }

    fn get_remaining_us(&self) -> Option<u32> {
        panic!("Unimplemented");
    }
}
