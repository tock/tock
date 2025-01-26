// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Real Time Clock (RTC) driver for RP2040.
//!
//! Author: Irina Bradu <irinabradu.a@gmail.com>
//!         Remus Rughinis <remus.rughinis.007@gmail.com>
//!
//! # Hardware Interface Layer (HIL)
//!
//! The driver implements Date_Time HIL. The following features are available when using
//! the driver through HIL:
//!
//! + Set time from which real time clock should start counting
//! + Read current time from the RTC registers
//!

use crate::clocks;
use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::date_time;
use kernel::hil::date_time::{DateTimeClient, DateTimeValues, DayOfWeek, Month};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

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
        (0x024 => inte: ReadWrite<u32, INTE::Register>),
        /// Interrupt Force
        (0x028 => intf: ReadWrite<u32, INTF::Register>),
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
    client: OptionalCell<&'a dyn date_time::DateTimeClient>,
    clocks: OptionalCell<&'a clocks::Clocks>,
    time: Cell<DateTimeValues>,

    deferred_call: DeferredCall,
    deferred_call_task: OptionalCell<DeferredCallTask>,
}

#[derive(Clone, Copy)]
enum DeferredCallTask {
    Get,
    Set,
}

impl DeferredCallClient for Rtc<'_> {
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

impl<'a> Rtc<'a> {
    pub fn new() -> Rtc<'a> {
        Rtc {
            registers: RTC_BASE,
            client: OptionalCell::empty(),
            clocks: OptionalCell::empty(),
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

    fn dotw_try_from_u32(&self, dotw: u32) -> Result<DayOfWeek, ErrorCode> {
        match dotw {
            0 => Ok(DayOfWeek::Sunday),
            1 => Ok(DayOfWeek::Monday),
            2 => Ok(DayOfWeek::Tuesday),
            3 => Ok(DayOfWeek::Wednesday),
            4 => Ok(DayOfWeek::Thursday),
            5 => Ok(DayOfWeek::Friday),
            6 => Ok(DayOfWeek::Saturday),
            _ => Err(ErrorCode::INVAL),
        }
    }

    fn dotw_into_u32(&self, dotw: DayOfWeek) -> u32 {
        match dotw {
            DayOfWeek::Sunday => 0,
            DayOfWeek::Monday => 1,
            DayOfWeek::Tuesday => 2,
            DayOfWeek::Wednesday => 3,
            DayOfWeek::Thursday => 4,
            DayOfWeek::Friday => 5,
            DayOfWeek::Saturday => 6,
        }
    }

    fn month_try_from_u32(&self, month_num: u32) -> Result<Month, ErrorCode> {
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

    fn month_into_u32(&self, month: Month) -> u32 {
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

    pub fn handle_set_interrupt(&self) {
        self.client.map(|client| client.set_date_time_done(Ok(())));
    }

    pub fn handle_get_interrupt(&self) {
        self.client
            .map(|client| client.get_date_time_done(Ok(self.time.get())));
    }

    pub fn set_clocks(&self, clocks: &'a clocks::Clocks) {
        self.clocks.replace(clocks);
    }

    fn date_time_setup(&self, datetime: date_time::DateTimeValues) -> Result<(), ErrorCode> {
        let month_val: u32 = self.month_into_u32(datetime.month);
        let day_val: u32 = self.dotw_into_u32(datetime.day_of_week);

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
            .modify(SETUP_0::YEAR.val(datetime.year as u32));
        self.registers.setup_0.modify(SETUP_0::MONTH.val(month_val));
        self.registers
            .setup_0
            .modify(SETUP_0::DAY.val(datetime.day as u32));

        self.registers.setup_1.modify(SETUP_1::DOTW.val(day_val));
        self.registers
            .setup_1
            .modify(SETUP_1::HOUR.val(datetime.hour as u32));
        self.registers
            .setup_1
            .modify(SETUP_1::MIN.val(datetime.minute as u32));
        self.registers
            .setup_1
            .modify(SETUP_1::SEC.val(datetime.seconds as u32));

        self.registers.ctrl.modify(CTRL::LOAD::SET);

        Ok(())
    }

    fn set_initial(&self) -> Result<(), ErrorCode> {
        let mut hw_ctrl: u32;

        self.registers.ctrl.modify(CTRL::RTC_ENABLE.val(0));
        hw_ctrl = self.registers.ctrl.read(CTRL::RTC_ENABLE);

        while hw_ctrl & self.registers.ctrl.read(CTRL::RTC_ACTIVE) > 0 {}

        let datetime = date_time::DateTimeValues {
            year: 1970,
            month: Month::January,
            day: 1,
            day_of_week: DayOfWeek::Sunday,
            hour: 0,
            minute: 0,
            seconds: 0,
        };

        self.date_time_setup(datetime)?;
        self.registers.ctrl.modify(CTRL::LOAD::SET);
        self.registers.ctrl.modify(CTRL::RTC_ENABLE.val(1));
        hw_ctrl = self.registers.ctrl.read(CTRL::RTC_ENABLE);

        while !((hw_ctrl & self.registers.ctrl.read(CTRL::RTC_ACTIVE)) > 0) {
            // wait until rtc starts
        }

        Ok(())
    }

    pub fn rtc_init(&self) -> Result<(), ErrorCode> {
        let mut rtc_freq = self
            .clocks
            .map_or(46875, |clocks| clocks.get_frequency(clocks::Clock::Rtc));

        rtc_freq -= rtc_freq;

        self.registers
            .clkdiv_m1
            .modify(CLKDIV_M1::CLKDIV_M.val(rtc_freq));

        self.set_initial()
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

        let month_num: u32 = self.registers.setup_0.read(SETUP_0::MONTH);
        let month_name: Month = match self.month_try_from_u32(month_num) {
            Result::Ok(t) => t,
            Result::Err(e) => {
                return Err(e);
            }
        };
        let dotw_num = self.registers.setup_1.read(SETUP_1::DOTW);
        let dotw = match self.dotw_try_from_u32(dotw_num) {
            Result::Ok(t) => t,
            Result::Err(e) => {
                return Err(e);
            }
        };

        let datetime = date_time::DateTimeValues {
            hour: self.registers.rtc_0.read(RTC_0::HOUR) as u8,
            minute: self.registers.rtc_0.read(RTC_0::MIN) as u8,
            seconds: self.registers.rtc_0.read(RTC_0::SEC) as u8,

            year: self.registers.rtc_1.read(RTC_1::YEAR) as u16,
            month: month_name,
            day: self.registers.rtc_1.read(RTC_1::DAY) as u8,
            day_of_week: dotw,
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

        self.date_time_setup(date_time)?;

        self.deferred_call_task.insert(Some(DeferredCallTask::Set));
        self.deferred_call.set();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn DateTimeClient) {
        self.client.set(client);
    }
}
