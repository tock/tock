// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Real Time Clock (RTC) driver for STM32f429zi.
//!
//! Author: Remus Rughinis <remus.rughinis.007@gmail.com>
//!
//! # Hardware Interface Layer (HIL)
//!
//! The driver implements Date_Time HIL. The following features are available when using
//! the driver through HIL:
//!
//! + Set time from which real time clock should start counting
//! + Read current time from the RTC registers
//!

use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::date_time;
use kernel::hil::date_time::{DateTimeClient, DateTimeValues, DayOfWeek, Month};
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use stm32f4xx::clocks::{phclk, Stm32f4Clocks};

/// Register block to control RTC
#[repr(C)]
pub struct RtcRegisters {
    /// The RTC_TR is the calendar time shadow register. This register must be written in initialization mode only.
    rtc_tr: ReadWrite<u32, RTC_TR::Register>,
    /// The RTC_DR is the calendar date shadow register. This register must be written in initialization mode only.
    rtc_dr: ReadWrite<u32, RTC_DR::Register>,
    /// RTC control register
    rtc_cr: ReadWrite<u32, RTC_CR::Register>,
    /// RTC initialization and status register
    rtc_isr: ReadWrite<u32, RTC_ISR::Register>,
    /// RTC prescaler register
    rtc_prer: ReadWrite<u32, RTC_PRER::Register>,
    /// RTC wakeup timer register
    rtc_wutr: ReadWrite<u32, RTC_WUTR::Register>,
    /// RTC calibration register
    rtc_calibr: ReadWrite<u32, RTC_CALIBR::Register>,
    /// RTC alarm A register
    rtc_alrmar: ReadWrite<u32, RTC_ALRMAR::Register>,
    /// RTC alarm B register
    rtc_alrmbr: ReadWrite<u32, RTC_ALRMBR::Register>,
    /// RTC write protection register
    rtc_wpr: ReadWrite<u32, RTC_WPR::Register>,
    /// RTC sub second register
    rtc_ssr: ReadWrite<u32, RTC_SSR::Register>,
    /// RTC shift control register
    rtc_shiftr: ReadWrite<u32, RTC_SHIFTR::Register>,
    /// RTC time stamp time register
    rtc_tstr: ReadWrite<u32, RTC_TSTR::Register>,
    /// RTC time stamp date register
    rtc_tsdr: ReadWrite<u32, RTC_TSDR::Register>,
    /// RTC time stamp sub second register
    rtc_tsssr: ReadWrite<u32, RTC_TSSSR::Register>,
    /// RTC calibration register
    rtc_calr: ReadWrite<u32, RTC_CALR::Register>,
    /// RTC tamper and alternate function configuration register
    rtc_tafcr: ReadWrite<u32, RTC_TAFCR::Register>,
    /// RTC alarm A sub second register
    rtc_alrmassr: ReadWrite<u32, RTC_ALRMASSR::Register>,
    /// RTC alarm B sub second register
    rtc_alrmbssr: ReadWrite<u32, RTC_ALRMBSSR::Register>,

    /// The application can write or read data to and from these registers
    rtc_bkpxr: [ReadWrite<u32, RTC_BKPXR::Register>; 19],
}

register_bitfields![u32,
RTC_TR[
    /// AM/PM notation. 0: AM or 24-hour format, 1: PM
    PM OFFSET(22) NUMBITS(1) [],
    /// Hour tens in BCD format
    HT OFFSET(20) NUMBITS(2) [],
    /// Hour units in BCD format
    HU OFFSET(16) NUMBITS(4) [],
    /// Minute tens in BCD format
    MNT OFFSET(12) NUMBITS(3) [],
    /// Minute units in BCD format
    MNU OFFSET(8) NUMBITS(4) [],
    /// Second tens in BCD format
    ST OFFSET(4) NUMBITS(3) [],
    /// Second units in BCD format
    SU OFFSET(0) NUMBITS(4) [],
],
RTC_DR[
    /// Year tens in BCD format
    YT OFFSET(20) NUMBITS(4) [],
    /// Year units in BCD format
    YU OFFSET(16) NUMBITS(4) [],
    /// Week day units. 000: forbidden, 001: Monday ... 111: Sunday
    WDU OFFSET(13) NUMBITS(3) [],
    ///Month tens in BCD format
    MT OFFSET(12) NUMBITS(1) [],
    /// Month units in BCD format
    MU OFFSET(8) NUMBITS(4) [],
    /// Date tens in BCD format
    DT OFFSET(4) NUMBITS(2) [],
    /// Date units in BCD format
    DU OFFSET(0) NUMBITS(4) [],
],
RTC_CR[
    /// Calibration output enable, enables the RTC_CALIB output
    COE OFFSET(23) NUMBITS(1) [],
    /// Output selection, used to select the flag to be routed to RTC_ALARM output
    OSEL OFFSET(21) NUMBITS(2) [],
    /// Output polarity, used to configure the polarity of RTC_ALARM output
    POL OFFSET(20) NUMBITS(1) [],
    /// Calibration output selection
    COSEL OFFSET(19) NUMBITS(1) [],
    /// Backup, memorizes whether daylight saving time change has been performed or not
    BKP OFFSET(18) NUMBITS(1) [],
    /// Subtract 1 hour (winter time change)
    SUB1H OFFSET(17) NUMBITS(1) [],
    /// Add 1 hour (summer time change)
    ADD1H OFFSET(16) NUMBITS(1) [],
    /// Timestamp interrupt enable
    TSIE OFFSET(15) NUMBITS(1) [],
    /// Wakeup timer interrupt enable
    WUTIE OFFSET(14) NUMBITS(1) [],
    /// Alarm B interrupt enable
    ALRBIE OFFSET(13) NUMBITS(1) [],
    /// Alarm A interrupt enable
    ALRAIE OFFSET(12) NUMBITS(1) [],
    /// Time stamp enable
    TSE OFFSET(11) NUMBITS(1) [],
    /// Wakeup timer enable
    WUTE OFFSET(10) NUMBITS(1) [],
    /// Alarm B enable
    ALRBE OFFSET(9) NUMBITS(1) [],
    /// Alarm A enable
    ALRAE OFFSET(8) NUMBITS(1) [],
    /// Coarse digital calibration enable
    DCE OFFSET(7) NUMBITS(1) [],
    /// Hour format
    FMT OFFSET(6) NUMBITS(1) [],
    /// Bypass the shadow registers
    BYPSHAD OFFSET(5) NUMBITS(1) [],
    /// Reference clock detection enable (50 or 60 Hz)
    REFCKON OFFSET(4) NUMBITS(1) [],
    /// Timestamp event active edge
    TSEDGE OFFSET(3) NUMBITS(1) [],
    /// Wakeup clock selection
    WUCKSEL OFFSET(0) NUMBITS(3) [],
],
RTC_ISR[
    /// Recalibration Pending Flag
    RECALPF OFFSET(16) NUMBITS(1) [],
    /// TAMPER2 detection flag
    TAMP2F OFFSET(14) NUMBITS(1) [],
    /// TAMPER detection flag
    TAMP1F OFFSET(13) NUMBITS(1) [],
    /// Timestamp overflow flag
    TSOVF OFFSET(12) NUMBITS(1) [],
    /// Timestamp flag
    TSF OFFSET(11) NUMBITS(1) [],
    /// Wakeup timer flag
    WUTF OFFSET(10) NUMBITS(1) [],
    /// Alarm B flag
    ALRBF OFFSET(9) NUMBITS(1) [],
    /// Alarm A flag
    ALRAF OFFSET(8) NUMBITS(1) [],
    /// Initialization mode
    INIT OFFSET(7) NUMBITS(1) [],
    /// Initialization flag
    INITF OFFSET(6) NUMBITS(1) [],
    /// Registers synchronization flag
    RSF OFFSET(5) NUMBITS(1) [],
    /// Initialization status flag
    INITS OFFSET(4) NUMBITS(1) [],
    /// Shift operation pending
    SHPF OFFSET(3) NUMBITS(1) [],
    /// Wakeup timer write flag
    WUTWF OFFSET(2) NUMBITS(1) [],
    /// Alarm B write flag
    ALRBWF OFFSET(1) NUMBITS(1) [],
    /// Alarm A write flag
    ALRAWF OFFSET(0) NUMBITS(1) [],
],
RTC_PRER[
    /// Asynchronous precaler factor
    PREDIV_A OFFSET(16) NUMBITS(7) [],
    /// Synchronous prescaler factor
    PREDIV_S OFFSET(0) NUMBITS(15) [],
],
RTC_WUTR[
    /// Wakeup auto-reload value bits
    WUT OFFSET(0) NUMBITS(16) [],
],
RTC_CALIBR[
    /// Digital calibration sign
    DCS OFFSET(7) NUMBITS(1) [],
    /// Digital calibration
    DC OFFSET(0) NUMBITS(5) [],
],
RTC_ALRMAR[
    /// Alarm A date mask
    MSK4 OFFSET(31) NUMBITS(1) [],
    /// Week day selection
    WDSEL OFFSET(30) NUMBITS(1) [],
    /// Date tens in BCD format
    DT OFFSET(28) NUMBITS(2) [],
    /// Date units in BCD format
    DU OFFSET(24) NUMBITS(4) [],
    /// Alarm A hours mask
    MSK3 OFFSET(23) NUMBITS(1) [],
    /// AM/PM notation
    PM OFFSET(22) NUMBITS(1) [],
    /// Hour tens in BCD format
    HT OFFSET(20) NUMBITS(2) [],
    /// Hour units in BCD format
    HU OFFSET(16) NUMBITS(4) [],
    /// Alarm A minutes mask
    MSK2 OFFSET(15) NUMBITS(1) [],
    /// Minute tens in BCD format
    MNT OFFSET(12) NUMBITS(3) [],
    /// Minute units in BCD format
    MNU OFFSET(8) NUMBITS(4) [],
    /// Alarm A seconds mask
    MSK1 OFFSET(7) NUMBITS(1) [],
    /// Second tens in BCD format
    ST OFFSET(4) NUMBITS(3) [],
    /// Second units in BCD format
    SU OFFSET(0) NUMBITS(4) [],
],
RTC_ALRMBR[
    /// Alarm B date mask
    MSK4 OFFSET(31) NUMBITS(1) [],
    /// Week day selection
    WDSEL OFFSET(30) NUMBITS(1) [],
    /// Date tens in BCD format
    DT OFFSET(28) NUMBITS(2) [],
    /// Date units in BCD format
    DU OFFSET(24) NUMBITS(4) [],
    /// Alarm B hours mask
    MSK3 OFFSET(23) NUMBITS(1) [],
    /// AM/PM notation
    PM OFFSET(22) NUMBITS(1) [],
    /// Hour tens in BCD format
    HT OFFSET(20) NUMBITS(2) [],
    /// Hour units in BCD format
    HU OFFSET(16) NUMBITS(4) [],
    /// Alarm B minutes mask
    MSK2 OFFSET(15) NUMBITS(1) [],
    /// Minute tens in BCD format
    MNT OFFSET(12) NUMBITS(3) [],
    /// Minute units in BCD format
    MNU OFFSET(8) NUMBITS(4) [],
    /// Alarm B seconds mask
    MSK1 OFFSET(7) NUMBITS(1) [],
    /// Second tens in BCD format
    ST OFFSET(4) NUMBITS(3) [],
    /// Second units in BCD format
    SU OFFSET(0) NUMBITS(4) [],
],
RTC_WPR[
    /// Write protection key
    KEY OFFSET(0) NUMBITS(8) [],
],
RTC_SSR[
    /// Sub second value
    SS OFFSET(0) NUMBITS(16) [],
],
RTC_SHIFTR[
    /// Add one second
    ADD1S OFFSET(31) NUMBITS(1) [],
    /// Subtract a fraction of a second
    SUBFS OFFSET(0) NUMBITS(15) [],
],
RTC_TSTR[
    /// AM/PM notation
    PM OFFSET(22) NUMBITS(1) [],
    /// Hour tens in BCD format
    HT OFFSET(20) NUMBITS(2) [],
    /// Hour units in BCD format
    HU OFFSET(16) NUMBITS(4) [],
    /// Minute tens in BCD format
    MNT OFFSET(12) NUMBITS(3) [],
    /// Minute units in BCD format
    MNU OFFSET(8) NUMBITS(4) [],
    /// Second tens in BCD format
    ST OFFSET(4) NUMBITS(3) [],
    /// Second units in BCD format
    STU OFFSET(0) NUMBITS(4) [],
],
RTC_TSDR[
    /// Week day units. 000: forbidden, 001: Monday ... 111: Sunday
    WDU OFFSET(13) NUMBITS(3) [],
    ///Month tens in BCD format
    MT OFFSET(12) NUMBITS(1) [],
    /// Month units in BCD format
    MU OFFSET(8) NUMBITS(4) [],
    /// Date tens in BCD format
    DT OFFSET(4) NUMBITS(2) [],
    /// Date units in BCD format
    DU OFFSET(0) NUMBITS(4) [],
],
RTC_TSSSR[
    /// Sub second value
    SS OFFSET(0) NUMBITS(16) [],
],
RTC_CALR[
    /// Increase frequency of RTC by 488.0 ppm
    CALP OFFSET(15) NUMBITS(1) [],
    /// Use an 8-second calibration cycle period
    CALW8 OFFSET(14) NUMBITS(1) [],
    /// Use a 16-second calibration cycle period
    CALW16 OFFSET(13) NUMBITS(1) [],
    /// Calibration minus
    CALM OFFSET(0) NUMBITS(9) [],
],
RTC_TAFCR[
    /// RTC_ALARM output type
    ALARMOUTTYPE OFFSET(18) NUMBITS(1) [],
    /// TIMESTAMP mapping
    TSINSEL OFFSET(17) NUMBITS(1) [],
    /// TAMPER1 mapping
    TAMP1INSEL OFFSET(16) NUMBITS(1) [],
    /// TAMPER pull-up disable
    TAMPPUDIS OFFSET(15) NUMBITS(1) [],
    /// Tamper prechange duration
    TAMPPRCH OFFSET(13) NUMBITS(2) [],
    /// Tamper filter count
    TAMPFLT OFFSET(11) NUMBITS(2) [],
    /// Tamper sampling frequency
    TAMPFREQ OFFSET(8) NUMBITS(3) [],
    /// Activate timestamp on tamper detection event
    TAMPTS OFFSET(7) NUMBITS(1) [],
    /// Active level for tamper 2
    TAMP2TRG OFFSET(4) NUMBITS(1) [],
    /// Tamper 2 detection enable
    TAMP2E OFFSET(3) NUMBITS(1) [],
    /// Tamper interrupt enable
    TAMPIE OFFSET(2) NUMBITS(1) [],
    /// Active level for tamper 1
    TAMP1TRG OFFSET(1) NUMBITS(1) [],
    /// Tamper 1 detection enable
    TAMP1E OFFSET(0) NUMBITS(1) [],
],
RTC_ALRMASSR[
    /// Mask the most-significant bits starting at this bit
    MASKSS OFFSET(24) NUMBITS(4) [],
    /// Sub seconds value
    SS OFFSET(0) NUMBITS(15) [],
],
RTC_ALRMBSSR[
    /// Mask the most-significant bits starting at this bit
    MASKSS OFFSET(24) NUMBITS(4) [],
    /// Sub seconds value
    SS OFFSET(0) NUMBITS(15) [],
],
RTC_BKPXR[
    /// The application can write or read data to and from these registers
    BKP OFFSET(0) NUMBITS(32) [],
],
];

pub struct Rtc<'a> {
    registers: StaticRef<RtcRegisters>,
    client: OptionalCell<&'a dyn date_time::DateTimeClient>,
    pub clock: phclk::PeripheralClock<'a>,
    pub pwr_clock: phclk::PeripheralClock<'a>,
    time: Cell<DateTimeValues>,

    deferred_call: DeferredCall,
    deferred_call_task: OptionalCell<DeferredCallTask>,
}

#[derive(Clone, Copy)]
enum DeferredCallTask {
    Get,
    Set,
}

impl<'a> DeferredCallClient for Rtc<'a> {
    fn handle_deferred_call(&self) {
        self.deferred_call_task.take().map(|value| match value {
            DeferredCallTask::Get => self
                .client
                .map(|client| client.get_date_time_done(Ok(self.time.get()))),
            DeferredCallTask::Set => self.client.map(|client| client.set_date_time_done(Ok(()))),
        });
    }
    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

const RTC_BASE: StaticRef<RtcRegisters> =
    unsafe { StaticRef::new(0x40002800 as *const RtcRegisters) };

impl<'a> Rtc<'a> {
    pub fn new(clocks: &'a dyn Stm32f4Clocks) -> Rtc<'a> {
        Rtc {
            registers: RTC_BASE,
            client: OptionalCell::empty(),
            clock: phclk::PeripheralClock::new(phclk::PeripheralClockType::RTC, clocks),
            pwr_clock: phclk::PeripheralClock::new(phclk::PeripheralClockType::PWR, clocks),
            time: Cell::new(DateTimeValues {
                year: 0,
                month: Month::January,
                day: 1,
                day_of_week: DayOfWeek::Sunday,
                hour: 0,
                minute: 0,
                seconds: 0,
            }),
            deferred_call: DeferredCall::new(),
            deferred_call_task: OptionalCell::empty(),
        }
    }

    fn dotw_try_from_u32(dotw: u32) -> Result<DayOfWeek, ErrorCode> {
        match dotw {
            1 => Ok(DayOfWeek::Monday),
            2 => Ok(DayOfWeek::Tuesday),
            3 => Ok(DayOfWeek::Wednesday),
            4 => Ok(DayOfWeek::Thursday),
            5 => Ok(DayOfWeek::Friday),
            6 => Ok(DayOfWeek::Saturday),
            7 => Ok(DayOfWeek::Sunday),
            _ => Err(ErrorCode::INVAL),
        }
    }

    fn dotw_into_u32(dotw: DayOfWeek) -> u32 {
        match dotw {
            DayOfWeek::Monday => 1,
            DayOfWeek::Tuesday => 2,
            DayOfWeek::Wednesday => 3,
            DayOfWeek::Thursday => 4,
            DayOfWeek::Friday => 5,
            DayOfWeek::Saturday => 6,
            DayOfWeek::Sunday => 7,
        }
    }

    fn month_try_from_u32(month_num: u32) -> Result<Month, ErrorCode> {
        match month_num {
            1 => Ok(Month::January),
            2 => Ok(Month::February),
            3 => Ok(Month::March),
            4 => Ok(Month::April),
            5 => Ok(Month::May),
            6 => Ok(Month::June),
            7 => Ok(Month::July),
            8 => Ok(Month::August),
            9 => Ok(Month::September),
            10 => Ok(Month::October),
            11 => Ok(Month::November),
            12 => Ok(Month::December),
            _ => Err(ErrorCode::INVAL),
        }
    }

    fn month_into_u32(month: Month) -> u32 {
        match month {
            Month::January => 1,
            Month::February => 2,
            Month::March => 3,
            Month::April => 4,
            Month::May => 5,
            Month::June => 6,
            Month::July => 7,
            Month::August => 8,
            Month::September => 9,
            Month::October => 10,
            Month::November => 11,
            Month::December => 12,
        }
    }

    #[inline(never)]
    // This function is marked as #[inline(never)] in order to aid with the debugging process when
    // disabling board memory protection
    /// Bypass write protection
    fn bypass_write_protection(&self) {
        self.registers.rtc_wpr.modify(RTC_WPR::KEY.val(0xCA)); // Equivalent to 0xCA
        self.registers.rtc_wpr.modify(RTC_WPR::KEY.val(0x53)); // Equivalent to 0x53
    }

    #[inline(never)]
    // This function is marked as #[inline(never)] in order to aid with the debugging process when
    // enabling board memory protection
    /// Reactivate write protection
    fn enable_write_protection(&self) {
        self.registers.rtc_wpr.modify(RTC_WPR::KEY.val(0x42)); // Equivalent to 0x42
    }

    fn date_time_setup(&self, datetime: date_time::DateTimeValues) -> Result<(), ErrorCode> {
        let month_num = Rtc::month_into_u32(datetime.month);
        let dotw_num = Rtc::dotw_into_u32(datetime.day_of_week);

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

        self.registers.rtc_dr.modify(
            RTC_DR::YT.val((datetime.year % 100) as u32 / 10)
                + RTC_DR::YU.val((datetime.year % 100) as u32 % 10)
                + RTC_DR::MT.val(month_num / 10)
                + RTC_DR::MU.val(month_num % 10)
                + RTC_DR::DT.val(datetime.day as u32 / 10)
                + RTC_DR::DU.val(datetime.day as u32 % 10)
                + RTC_DR::WDU.val(dotw_num),
        );

        self.registers.rtc_tr.modify(
            RTC_TR::HT.val(datetime.hour as u32 / 10)
                + RTC_TR::HU.val(datetime.hour as u32 % 10)
                + RTC_TR::MNT.val(datetime.minute as u32 / 10)
                + RTC_TR::MNU.val(datetime.minute as u32 % 10)
                + RTC_TR::ST.val(datetime.seconds as u32 / 10)
                + RTC_TR::SU.val(datetime.seconds as u32 % 10),
        );

        Ok(())
    }

    pub fn enter_init_mode(&self) -> Result<(), ErrorCode> {
        self.bypass_write_protection();
        self.registers.rtc_isr.modify(RTC_ISR::INIT::SET);

        let mut cycle_counter = 100000;
        while cycle_counter > 0 && !self.registers.rtc_isr.is_set(RTC_ISR::INITF) {
            cycle_counter -= 1;
            // wait until initialization phase mode is entered
        }
        if cycle_counter <= 0 {
            return Err(ErrorCode::FAIL);
        }
        Ok(())
    }
    pub fn exit_init_mode(&self) -> Result<(), ErrorCode> {
        self.registers.rtc_isr.modify(RTC_ISR::INIT::CLEAR);
        let mut cycle_counter = 100000;
        while cycle_counter > 0 && !self.registers.rtc_isr.is_set(RTC_ISR::RSF) {
            cycle_counter -= 1;
        }
        if cycle_counter <= 0 {
            return Err(ErrorCode::FAIL);
        }

        self.enable_write_protection();
        Ok(())
    }

    pub fn rtc_init(&self) -> Result<(), ErrorCode> {
        self.enter_init_mode()?;

        self.registers
            .rtc_prer
            .modify(RTC_PRER::PREDIV_A.val(128 - 1));
        self.registers
            .rtc_prer
            .modify(RTC_PRER::PREDIV_S.val(256 - 1));

        // 0: 24-hour format, 1: AM/PM format
        self.registers.rtc_cr.modify(RTC_CR::FMT.val(0));

        let datetime = date_time::DateTimeValues {
            year: 0,
            month: Month::January,
            day: 1,
            day_of_week: DayOfWeek::Monday,

            hour: 0,
            minute: 0,
            seconds: 0,
        };
        self.date_time_setup(datetime)?;

        self.exit_init_mode()?;
        Ok(())
    }

    pub fn enable_clock(&self) {
        self.pwr_clock.enable();

        // Enable access to the backup domain
        match crate::pwr::enable_backup_access() {
            Err(e) => panic!("{:?}", e),
            _ => (),
        }

        self.clock.enable();
    }
}

impl<'a> date_time::DateTime<'a> for Rtc<'a> {
    fn get_date_time(&self) -> Result<(), ErrorCode> {
        match self.deferred_call_task.take() {
            Some(DeferredCallTask::Set) => {
                self.deferred_call_task.insert(Some(DeferredCallTask::Set));
                return Err(ErrorCode::BUSY);
            }
            Some(DeferredCallTask::Get) => {
                self.deferred_call_task.insert(Some(DeferredCallTask::Get));
                return Err(ErrorCode::ALREADY);
            }
            _ => (),
        }

        let month_num =
            self.registers.rtc_dr.read(RTC_DR::MT) * 10 + self.registers.rtc_dr.read(RTC_DR::MU);
        let month_name = Rtc::month_try_from_u32(month_num)?;

        let dotw_num = self.registers.rtc_dr.read(RTC_DR::WDU);
        let dotw_name = Rtc::dotw_try_from_u32(dotw_num)?;

        let datetime = date_time::DateTimeValues {
            hour: (self.registers.rtc_tr.read(RTC_TR::HT) * 10
                + self.registers.rtc_tr.read(RTC_TR::HU)) as u8,
            minute: (self.registers.rtc_tr.read(RTC_TR::MNT) * 10
                + self.registers.rtc_tr.read(RTC_TR::MNU)) as u8,
            seconds: (self.registers.rtc_tr.read(RTC_TR::ST) * 10
                + self.registers.rtc_tr.read(RTC_TR::SU)) as u8,

            year: (self.registers.rtc_dr.read(RTC_DR::YT) * 10
                + self.registers.rtc_dr.read(RTC_DR::YU)) as u16,
            month: month_name,
            day: (self.registers.rtc_dr.read(RTC_DR::DT) * 10
                + self.registers.rtc_dr.read(RTC_DR::DU)) as u8,
            day_of_week: dotw_name,
        };

        self.time.replace(datetime);

        self.deferred_call_task.insert(Some(DeferredCallTask::Get));
        self.deferred_call.set();

        Ok(())
    }

    fn set_date_time(&self, date_time: date_time::DateTimeValues) -> Result<(), ErrorCode> {
        match self.deferred_call_task.take() {
            Some(DeferredCallTask::Set) => {
                self.deferred_call_task.insert(Some(DeferredCallTask::Set));
                return Err(ErrorCode::ALREADY);
            }
            Some(DeferredCallTask::Get) => {
                self.deferred_call_task.insert(Some(DeferredCallTask::Get));
                return Err(ErrorCode::BUSY);
            }
            _ => (),
        }

        self.enter_init_mode()?;
        self.date_time_setup(date_time)?;
        self.exit_init_mode()?;

        self.deferred_call_task.insert(Some(DeferredCallTask::Set));
        self.deferred_call.set();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn DateTimeClient) {
        self.client.set(client);
    }
}
