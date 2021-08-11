use core::convert::{TryFrom, TryInto};
use kernel::common::cells::OptionalCell;
use kernel::common::registers::interfaces::{ReadWriteable, Readable};
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil::time;
use kernel::hil::time::{DayOfWeek, Month, RtcClient};
use kernel::ErrorCode;

use crate::clocks;

register_structs! {
    /// Register block to control RTC
    RtcRegisters {
        /// Divider minus 1 for the 1 second counter. Safe to change the value when RTC is n
        (0x000 => clkdiv_m1: ReadWrite<u32, CLKDIV_M1::Register>),
        /// RTC setup register 0
        (0x004 => setup_0: ReadWrite<u32, SETUP_0::Register>),
        /// RTC setup register 1
        (0x008 => setup_1: ReadWrite<u32, SETUP_1::Register>),
        /// RTC Control and status
        (0x00C => ctrl: ReadWrite<u32, CTRL::Register>),
        /// Interrupt setup register 0
        (0x010 => irq_setup_0: ReadWrite<u32, IRQ_SETUP_0::Register>),
        /// Interrupt setup register 1
        (0x014 => irq_setup_1: ReadWrite<u32, IRQ_SETUP_1::Register>),
        /// RTC register 1.
        (0x018 => rtc_1: ReadWrite<u32, RTC_1::Register>),
        /// RTC register 0\n
/// Read this before RTC 1!
        (0x01C => rtc_0: ReadWrite<u32, RTC_0::Register>),
        /// Raw Interrupts
        (0x020 => intr: ReadWrite<u32>),
        /// Interrupt Enable
        (0x024 => inte: ReadWrite<u32>),
        /// Interrupt Force
        (0x028 => intf: ReadWrite<u32>),
        /// Interrupt status after masking & forcing
        (0x02C => ints: ReadWrite<u32>),
        (0x030 => @END),
    }
}
register_bitfields![u32,
CLKDIV_M1 [

    CLKDIV_M OFFSET(0) NUMBITS(16) []
],
SETUP_0 [
    /// Year
    YEAR OFFSET(12) NUMBITS(12) [],
    /// Month (1..12)
    MONTH OFFSET(8) NUMBITS(4) [],
    /// Day of the month (1..31)
    DAY OFFSET(0) NUMBITS(5) []
],
SETUP_1 [
    /// Day of the week: 1-Monday...0-Sunday ISO 8601 mod 7
    DOTW OFFSET(24) NUMBITS(3) [],
    /// Hours
    HOUR OFFSET(16) NUMBITS(5) [],
    /// Minutes
    MIN OFFSET(8) NUMBITS(6) [],
    /// Seconds
    SEC OFFSET(0) NUMBITS(6) []
],
CTRL [
    /// If set, leapyear is forced off.\n
/// Useful for years divisible by 100 but not by 400
    FORCE_NOTLEAPYEAR OFFSET(8) NUMBITS(1) [],
    /// Load RTC
    LOAD OFFSET(4) NUMBITS(1) [],
    /// RTC enabled (running)
    RTC_ACTIVE OFFSET(1) NUMBITS(1) [],
    /// Enable RTC
    RTC_ENABLE OFFSET(0) NUMBITS(1) []
],
IRQ_SETUP_0 [

    MATCH_ACTIVE OFFSET(29) NUMBITS(1) [],
    /// Global match enable. Don't change any other value while this one is enabled
    MATCH_ENA OFFSET(28) NUMBITS(1) [],
    /// Enable year matching
    YEAR_ENA OFFSET(26) NUMBITS(1) [],
    /// Enable month matching
    MONTH_ENA OFFSET(25) NUMBITS(1) [],
    /// Enable day matching
    DAY_ENA OFFSET(24) NUMBITS(1) [],
    /// Year
    YEAR OFFSET(12) NUMBITS(12) [],
    /// Month (1..12)
    MONTH OFFSET(8) NUMBITS(4) [],
    /// Day of the month (1..31)
    DAY OFFSET(0) NUMBITS(5) []
],
IRQ_SETUP_1 [
    /// Enable day of the week matching
    DOTW_ENA OFFSET(31) NUMBITS(1) [],
    /// Enable hour matching
    HOUR_ENA OFFSET(30) NUMBITS(1) [],
    /// Enable minute matching
    MIN_ENA OFFSET(29) NUMBITS(1) [],
    /// Enable second matching
    SEC_ENA OFFSET(28) NUMBITS(1) [],
    /// Day of the week
    DOTW OFFSET(24) NUMBITS(3) [],
    /// Hours
    HOUR OFFSET(16) NUMBITS(5) [],
    /// Minutes
    MIN OFFSET(8) NUMBITS(6) [],
    /// Seconds
    SEC OFFSET(0) NUMBITS(6) []
],
RTC_1 [
    /// Year
    YEAR OFFSET(12) NUMBITS(12) [],
    /// Month (1..12)
    MONTH OFFSET(8) NUMBITS(4) [],
    /// Day of the month (1..31)
    DAY OFFSET(0) NUMBITS(5) []
],
RTC_0 [
    /// Day of the week
    DOTW OFFSET(24) NUMBITS(3) [],
    /// Hours
    HOUR OFFSET(16) NUMBITS(5) [],
    /// Minutes
    MIN OFFSET(8) NUMBITS(6) [],
    /// Seconds
    SEC OFFSET(0) NUMBITS(6) []
],
INTR [

    RTC OFFSET(0) NUMBITS(1) []
],
INTE [

    RTC OFFSET(0) NUMBITS(1) []
],
INTF [

    RTC OFFSET(0) NUMBITS(1) []
],
INTS [

    RTC OFFSET(0) NUMBITS(1) []
]
];
const RTC_BASE: StaticRef<RtcRegisters> =
    unsafe { StaticRef::new(0x4005C000 as *const RtcRegisters) };

pub struct Rtc<'a> {
    registers: StaticRef<RtcRegisters>,
    client: OptionalCell<&'a dyn kernel::hil::time::RtcClient>,
    clocks: OptionalCell<&'a clocks::Clocks>
}

impl<'a> Rtc<'a> {
    pub const fn new() -> Rtc<'a> {
        Rtc {
            registers: RTC_BASE,
            client: OptionalCell::empty(),
            clocks: OptionalCell::empty(),
        }
    }

    pub fn set_clocks (&self, clocks: &'a clocks::Clocks) {
        self.clocks.replace(clocks);
    }

    fn date_time_setup(&self, datetime: time::DateTime) -> Result<(), ErrorCode> {
        let month_val: usize = match datetime.month.try_into() {
            Result::Ok(t) => t,
            Result::Err(()) => {
                return Err(ErrorCode::FAIL);
            }
        };

        let day_val: usize = match datetime.day_of_week.try_into() {
            Result::Ok(t) => t,
            Result::Err(()) => {
                return Err(ErrorCode::FAIL);
            }
        };

        if !(datetime.year <= 4095) {
            return Err(ErrorCode::INVAL);
        }

        if !(datetime.day >= 1 && datetime.day <= 31) {
            return Err(ErrorCode::INVAL);
        }

        if !(datetime.hour <= 23) {
            return Err(ErrorCode::INVAL);
        }
        if !(datetime.minute <= 59) {
            return Err(ErrorCode::INVAL);
        }
        if !(datetime.seconds <= 59) {
            return Err(ErrorCode::INVAL);
        }

        self.registers
            .setup_0
            .modify(SETUP_0::YEAR.val(datetime.year));
        self.registers
            .setup_0
            .modify(SETUP_0::MONTH.val(month_val as u32));
        self.registers
            .setup_0
            .modify(SETUP_0::DAY.val(datetime.day));

        self.registers
            .setup_1
            .modify(SETUP_1::DOTW.val(day_val as u32));
        self.registers
            .setup_1
            .modify(SETUP_1::HOUR.val(datetime.hour));
        self.registers
            .setup_1
            .modify(SETUP_1::MIN.val(datetime.minute));
        self.registers
            .setup_1
            .modify(SETUP_1::SEC.val(datetime.seconds));

        Ok(())
    }

    fn set_initial(&self, datetime: time::DateTime) -> Result<(), ErrorCode> {
        //TODO check datetime validity before setup

        let mut hw_ctrl: u32;

        self.registers.ctrl.modify(CTRL::RTC_ENABLE.val(0));
        hw_ctrl = self.registers.ctrl.read(CTRL::RTC_ENABLE);

        while hw_ctrl & self.registers.ctrl.read(CTRL::RTC_ACTIVE) > 0 {
            debug!("rtc is running");
        }

        match self.date_time_setup(datetime) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        };
        self.registers.ctrl.modify(CTRL::LOAD::SET);
        self.registers.ctrl.modify(CTRL::RTC_ENABLE.val(1));
        hw_ctrl = self.registers.ctrl.read(CTRL::RTC_ENABLE);

        while !((hw_ctrl & self.registers.ctrl.read(CTRL::RTC_ACTIVE)) > 0) {
            debug!("rtc is NOT running");
        }

        Ok(())
    }

    pub fn rtc_init(&self) {
        let mut rtc_freq= self.clocks.map_or(46875, |clocks| clocks.get_frequency(clocks::Clock::Rtc));

        rtc_freq = rtc_freq - 1;

        self.registers
            .clkdiv_m1
            .modify(CLKDIV_M1::CLKDIV_M.val(rtc_freq));
    }
}

impl<'a> time::Rtc<'a> for Rtc<'a> {
    fn get_date_time(&self) -> Result<Option<time::DateTime>, ErrorCode> {
        let month_num: u32 = self.registers.setup_0.read(SETUP_0::MONTH);
        let month_name: Month = match time::Month::try_from(month_num as usize) {
            Result::Ok(t) => t,
            Result::Err(()) => {
                return Err(ErrorCode::FAIL);
            }
        };
        let dotw_num = self.registers.setup_1.read(SETUP_1::DOTW);
        let dotw = match DayOfWeek::try_from(dotw_num as usize) {
            Result::Ok(t) => t,
            Result::Err(()) => {
                return Err(ErrorCode::FAIL);
            }
        };

        let datetime = time::DateTime {
            hour: self.registers.rtc_0.read(RTC_0::HOUR),
            minute: self.registers.rtc_0.read(RTC_0::MIN),
            seconds: self.registers.rtc_0.read(RTC_0::SEC),

            year: self.registers.rtc_1.read(RTC_1::YEAR),
            month: month_name,
            day: self.registers.rtc_1.read(RTC_1::DAY),
            day_of_week: dotw,
        };

        self.client
            .map(|client| client.callback_get_date(Ok(datetime)));

        Ok(None)
    }

    fn set_date_time(
        &self,
        date_time: time::DateTime,
    ) -> Result<Option<time::DateTime>, ErrorCode> {
        self.set_initial(date_time)?;
        self.client.map(|client| client.callback_set_date(Ok(())));
        Ok(None)
    }

    fn set_client(&self, client: &'a dyn RtcClient) {
        self.client.set(client);
    }
}
