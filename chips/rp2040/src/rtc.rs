use kernel::common::StaticRef;
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::hil::time;    
use kernel::ErrorCode;
use kernel::debug;
use kernel::common::registers::interfaces::{Readable, ReadWriteable, Writeable};
use kernel::hil::time::RtcClient;
use kernel::common::cells::OptionalCell;

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
}


impl<'a> Rtc<'a>{
    pub const fn new() -> Rtc<'a> {
        Rtc {
            registers: RTC_BASE,
            client: OptionalCell::empty(),
        }
    }

    fn date_time_setup(&self, datetime:time::DateTime) -> Result<(),ErrorCode>{

        let month_val:u32 =
            match datetime.month{
                time::Month::January => 1,
                time::Month::February => 2,
                time::Month::March => 3,
                time::Month::April => 4,
                time::Month::May => 5,
                time::Month::June => 6,
                time::Month::July => 7,
                time::Month::August => 8,
                time::Month::September => 9,
                time::Month::October => 10,
                time::Month::November => 11,
                time::Month::December => 12,
            };

        let day_val:u32 =
            match datetime.day_of_week{
                time::DayOfWeek::Sunday => 0,
                time::DayOfWeek::Monday => 1,
                time::DayOfWeek::Tuesday => 2,
                time::DayOfWeek::Wednesday => 3,
                time::DayOfWeek::Thursday => 4,
                time::DayOfWeek::Friday => 5,
                time::DayOfWeek::Saturday =>6,
            };


        self.registers.setup_0.modify(SETUP_0::YEAR.val(datetime.year));
        self.registers.setup_0.modify(SETUP_0::MONTH.val(month_val));
        self.registers.setup_0.modify(SETUP_0::DAY.val(datetime.day));

        self.registers.setup_1.modify(SETUP_1::DOTW.val(day_val));
        self.registers.setup_1.modify(SETUP_1::HOUR.val(datetime.hour));
        self.registers.setup_1.modify(SETUP_1::MIN.val(datetime.minute));
        self.registers.setup_1.modify(SETUP_1::SEC.val(datetime.seconds));

        Ok(())

    }

    pub fn set_initial(&self, datetime:time::DateTime) -> Result<(),ErrorCode>{
        let mut hw_ctrl:u32;

        self.registers.ctrl.modify(CTRL::RTC_ENABLE.val(0));
        hw_ctrl = self.registers.ctrl.read(CTRL::RTC_ENABLE);

        while hw_ctrl & self.registers.ctrl.read(CTRL::RTC_ACTIVE)>0 {
            debug!("is running");
        }

        self.date_time_setup(datetime);
        self.registers.ctrl.modify(CTRL::LOAD::SET);
        self.registers.ctrl.modify(CTRL::RTC_ENABLE.val(1));
        hw_ctrl = self.registers.ctrl.read(CTRL::RTC_ENABLE);

        while !((hw_ctrl & self.registers.ctrl.read(CTRL::RTC_ACTIVE))>0) {
            debug!("is NOT running");
        }

        Ok(())

    }


    //pub fn get_leap_year(){}

    
    pub fn rtc_init(&self){
        let mut rtc_freq:u32 = 48_000_000;

        rtc_freq = rtc_freq - 1;


        self.registers.clkdiv_m1.modify(CLKDIV_M1::CLKDIV_M.val(rtc_freq));
        
    }



}



impl<'a> time::Rtc<'a> for Rtc<'a> {
    fn get_date_time (&self) -> Result<Option<time::DateTime>, ErrorCode>{


        let month_name: time::Month;
        match self.get_month(){
            Ok(v) => {month_name = v;},
            Err(e) => {debug!("error settng month {:?}",e); return Err(e);},
        };

        let dotw : time::DayOfWeek;
        match self.get_day_of_week(){
            Ok(v) => {dotw = v;},
            Err(e) => {debug!("error settng day of week {:?}",e); return Err(e);}
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


       // debug!("year: {}   day:{}   \n hour:{}   minute:{}  seconds:{}",datetime.year,datetime.day,datetime.hour, datetime.minute, datetime.seconds);



        //self.client.map(|client| client.callback(Ok(datetime)));
        self.client.map(|client| client.callback(Ok(datetime)));

        Ok(
            //Some()
            None
        )

    }

    fn set_date_time (&self, date_time: time::DateTime) -> Result<(), ErrorCode>{

       self.set_year(date_time.year)?;
       self.set_month(date_time.month)?;
       self.set_day_of_month(date_time.day)?;
       self.set_day_of_week(date_time.day_of_week)?;
       self.set_hour(date_time.hour)?;
       self.set_minute(date_time.minute)?;
       self.set_seconds(date_time.seconds)?;
       Ok(())
    }


    fn get_year(&self) -> Result<u32, ErrorCode>{
        Ok(self.registers.setup_0.read(SETUP_0::YEAR))
    }

    fn set_year (&self, year:u32) -> Result<(), ErrorCode>{
        self.registers.setup_0.modify(SETUP_0::YEAR.val(year));
        Ok(())
    }


    fn get_month(&self) -> Result<time::Month, ErrorCode>{
      let month_num: u32 = self.registers.setup_0.read(SETUP_0::MONTH);
      match month_num{
          1 => Ok(time::Month::January),
          2 => Ok(time::Month::February),
          3 => Ok(time::Month::March),
          4 => Ok(time::Month::April),
          5 => Ok(time::Month::May),
          6 => Ok(time::Month::June),
          7 => Ok(time::Month::July),
          8 => Ok(time::Month::August),
          9 => Ok(time::Month::September),
          10 => Ok(time::Month::October),
          11 => Ok(time::Month::November),
          12 => Ok(time::Month::December),
          _=> Err(ErrorCode::FAIL),
      }
    }

    fn set_month(&self, month: time::Month) -> Result<(), ErrorCode>{
        let month_val =
        match month{
            time::Month::January => 1,
            time::Month::February => 2,
            time::Month::March => 3,
            time::Month::April => 4,
            time::Month::May => 5,
            time::Month::June => 6,
            time::Month::July => 7,
            time::Month::August => 8,
            time::Month::September => 9,
            time::Month::October => 10,
            time::Month::November => 11,
            time::Month::December => 12,
        };
        self.registers.setup_0.modify(SETUP_0::MONTH.val(month_val));
        Ok(())
    }

    fn get_day_of_month(&self) -> Result<u32, ErrorCode>{
        Ok(self.registers.setup_0.read(SETUP_0::DAY))

    }


    fn set_day_of_month(&self, day:u32) -> Result<(), ErrorCode>{
        self.registers.setup_0.modify(SETUP_0::DAY.val(day));
        Ok(())
    }

    fn get_day_of_week(&self) -> Result<time::DayOfWeek, ErrorCode>{
       match self.registers.setup_1.read(SETUP_1::DOTW){
           0 => Ok(time::DayOfWeek::Sunday),
           1 => Ok(time::DayOfWeek::Monday),
           2 => Ok(time::DayOfWeek::Tuesday),
           3 => Ok(time::DayOfWeek::Wednesday),
           4 => Ok(time::DayOfWeek::Thursday),
           5 => Ok(time::DayOfWeek::Friday),
           6 => Ok(time::DayOfWeek::Saturday),
           _=> Err(ErrorCode::FAIL),
       }
    }
    fn set_day_of_week(&self, day_of_week: time::DayOfWeek) -> Result<(), ErrorCode>{
        let day_val =
        match day_of_week{
            time::DayOfWeek::Sunday => 0,
            time::DayOfWeek::Monday => 1,
            time::DayOfWeek::Tuesday => 2,
            time::DayOfWeek::Wednesday => 3,
            time::DayOfWeek::Thursday => 4,
            time::DayOfWeek::Friday => 5,
            time::DayOfWeek::Saturday =>6,
        };
        self.registers.setup_1.modify(SETUP_1::DOTW.val(day_val));
        Ok(())

    }

    fn get_hour(&self) -> Result<u32, ErrorCode>{
        Ok(self.registers.setup_1.read(SETUP_1::HOUR))
    }
    fn set_hour(&self, hour: u32) -> Result<(), ErrorCode>{
        self.registers.setup_1.modify(SETUP_1::HOUR.val(hour));
        Ok(())
    }

    fn get_minute(&self) -> Result<u32, ErrorCode>{
        Ok(self.registers.setup_1.read(SETUP_1::MIN))
    }
    fn set_minute(&self,minute: u32) -> Result<(), ErrorCode>{
        self.registers.setup_1.modify(SETUP_1::MIN.val(minute));
        Ok(())
    }

    fn get_seconds(&self) -> Result<u32, ErrorCode>{
        Ok(self.registers.setup_1.read(SETUP_1::SEC))
    }


    fn set_seconds(&self, seconds: u32) -> Result<(), ErrorCode>{
        self.registers.setup_1.modify(SETUP_1::SEC.val(seconds));
        Ok(())
    }

    fn set_client(&self, client: &'a dyn RtcClient) {
        self.client.set(client);
    }
}