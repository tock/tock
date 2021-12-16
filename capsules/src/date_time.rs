//! Real Time Clock driver
//!
//! Allows handling of the current date and time
//!
//! Author: Irina Bradu <irinabradu.a@gmail.com>
//!
//!
//! Usage
//! -----
//!
//! ```rust
//!  let grant_dt = create_capability!(capabilities::MemoryAllocationCapability);
//!  let grant_date_time = board_kernel.create_grant(capsules::date_time::DRIVER_NUM, &grant_dt);
//!
//!  let date_time = static_init!(
//!     capsules::date_time::DateTime<'static>,
//!     capsules::date_time::DateTime::new(&peripherals.rtc, grant_date_time)
//!  );
//!  kernel::hil::date_time::DateTime::set_client(&peripherals.rtc, date_time);
//! ```

use crate::driver::NUM;
use core::cell::Cell;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::date_time;

use kernel::errorcode::into_statuscode;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::registers::{register_bitfields, LocalRegisterCopy};
use kernel::{ErrorCode, ProcessId};

pub const DRIVER_NUM: usize = NUM::DateTime as usize;

pub enum DateTimeCommand {
    ReadDateTime,
    SetDateTime,
}

#[derive(Default, Clone, Copy)]
pub struct AppData {
    subscribed: bool,
}

pub struct DateTime<'a> {
    date_time: &'a dyn date_time::DateTime<'a>,
    apps: Grant<AppData, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    in_progress: Cell<bool>,
}

register_bitfields![u32,
    YEAR_MONTH_DOTM[
        YEAR OFFSET(9) NUMBITS(12) [],

        MONTH OFFSET(5) NUMBITS(4) [],

        DAY OFFSET(0) NUMBITS(5) [],

    ],
    DOTW_HOUR_MIN_SEC[

        DOTW OFFSET(17) NUMBITS(3) [],

        HOUR OFFSET(12) NUMBITS(5) [],

        MIN OFFSET(6) NUMBITS(6) [],

        SEC OFFSET(0) NUMBITS(6) []
    ]
];

impl<'a> DateTime<'a> {
    pub fn new(
        date_time: &'a dyn date_time::DateTime<'a>,
        grant: Grant<AppData, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> DateTime<'a> {
        DateTime {
            date_time,
            apps: grant,
            in_progress: Cell::new(false),
        }
    }

    fn month_as_u32(&self, month: date_time::Month) -> u32 {
        match month {
            date_time::Month::January => 1,
            date_time::Month::February => 2,
            date_time::Month::March => 3,
            date_time::Month::April => 4,
            date_time::Month::May => 5,
            date_time::Month::June => 6,
            date_time::Month::July => 7,
            date_time::Month::August => 8,
            date_time::Month::September => 9,
            date_time::Month::October => 10,
            date_time::Month::November => 11,
            date_time::Month::December => 12,
        }
    }

    fn u32_as_month(&self, month_num: u32) -> Result<date_time::Month, ErrorCode> {
        match month_num {
            1 => Ok(date_time::Month::January),
            2 => Ok(date_time::Month::February),
            3 => Ok(date_time::Month::March),
            4 => Ok(date_time::Month::April),
            5 => Ok(date_time::Month::May),
            6 => Ok(date_time::Month::June),
            7 => Ok(date_time::Month::July),
            8 => Ok(date_time::Month::August),
            9 => Ok(date_time::Month::September),
            10 => Ok(date_time::Month::October),
            11 => Ok(date_time::Month::November),
            12 => Ok(date_time::Month::December),
            _ => Err(ErrorCode::INVAL),
        }
    }

    fn dotw_as_u32(&self, dotw: date_time::DayOfWeek) -> u32 {
        match dotw {
            date_time::DayOfWeek::Sunday => 0,
            date_time::DayOfWeek::Monday => 1,
            date_time::DayOfWeek::Tuesday => 2,
            date_time::DayOfWeek::Wednesday => 3,
            date_time::DayOfWeek::Thursday => 4,
            date_time::DayOfWeek::Friday => 5,
            date_time::DayOfWeek::Saturday => 6,
        }
    }

    fn u32_as_dotw(&self, dotw_num: u32) -> Result<date_time::DayOfWeek, ErrorCode> {
        match dotw_num {
            0 => Ok(date_time::DayOfWeek::Sunday),
            1 => Ok(date_time::DayOfWeek::Monday),
            2 => Ok(date_time::DayOfWeek::Tuesday),
            3 => Ok(date_time::DayOfWeek::Wednesday),
            4 => Ok(date_time::DayOfWeek::Thursday),
            5 => Ok(date_time::DayOfWeek::Friday),
            6 => Ok(date_time::DayOfWeek::Saturday),
            _ => Err(ErrorCode::INVAL),
        }
    }

    /// Transforms Date structure (year, month, dotm, dotw, hour, minute, seconds)
    /// into two u32 numbers:
    ///         first number (year<<month<<day_of_the_month):
    ///                 -last 5 bits store the day_of_the_month
    ///                 -previous 4 bits store the month
    ///                 -previous 12 bits store the year
    ///         second number (day_of_the_week, hour, minute, seconds):
    ///                 -last 6 bits store the seconds
    ///                 -previous 6 store the minute
    ///                 -previous 5 store the hour
    ///                 -previous 3 store the day_of_the_week
    ///the two u32 numbers are returned as a tuple
    fn date_as_u32_tuple(&self, date: date_time::Date) -> Result<(u32, u32), ErrorCode> {
        let month = self.month_as_u32(date.month);

        let mut year_month_dotm: LocalRegisterCopy<u32, YEAR_MONTH_DOTM::Register> =
            LocalRegisterCopy::new(0);

        year_month_dotm.modify(YEAR_MONTH_DOTM::YEAR.val(date.year as u32));
        year_month_dotm.modify(YEAR_MONTH_DOTM::MONTH.val(month as u32));
        year_month_dotm.modify(YEAR_MONTH_DOTM::DAY.val(date.day as u32));

        let dotw = self.dotw_as_u32(date.day_of_week);

        let mut dotw_hour_min_sec: LocalRegisterCopy<u32, DOTW_HOUR_MIN_SEC::Register> =
            LocalRegisterCopy::new(0);

        dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::DOTW.val(dotw as u32));
        dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::HOUR.val(date.hour as u32));
        dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::MIN.val(date.minute as u32));
        dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::SEC.val(date.seconds as u32));

        Ok((year_month_dotm.get(), dotw_hour_min_sec.get()))
    }

    fn call_driver(&self, command: DateTimeCommand, r2: usize, r3: usize) -> CommandReturn {
        match command {
            DateTimeCommand::ReadDateTime => {
                let date_result = self.date_time.get_date_time();
                match date_result {
                    Result::Ok(()) => {
                        self.in_progress.set(true);
                        CommandReturn::success()
                    }
                    Result::Err(e) => CommandReturn::failure(e),
                }
            }
            DateTimeCommand::SetDateTime => {
                let year_month_dotm: LocalRegisterCopy<u32, YEAR_MONTH_DOTM::Register> =
                    LocalRegisterCopy::new(r2 as u32);
                let dotw_hour_min_sec: LocalRegisterCopy<u32, DOTW_HOUR_MIN_SEC::Register> =
                    LocalRegisterCopy::new(r3 as u32);

                let date = date_time::Date {
                    year: year_month_dotm.read(YEAR_MONTH_DOTM::YEAR) as u16,
                    month: match self.u32_as_month(year_month_dotm.read(YEAR_MONTH_DOTM::MONTH)) {
                        Result::Ok(t) => t,
                        Result::Err(e) => {
                            return CommandReturn::failure(e);
                        }
                    },
                    day: year_month_dotm.read(YEAR_MONTH_DOTM::DAY) as u8,
                    day_of_week: match self
                        .u32_as_dotw(dotw_hour_min_sec.read(DOTW_HOUR_MIN_SEC::DOTW))
                    {
                        Result::Ok(t) => t,
                        Result::Err(e) => {
                            return CommandReturn::failure(e);
                        }
                    },
                    hour: dotw_hour_min_sec.read(DOTW_HOUR_MIN_SEC::HOUR) as u8,
                    minute: dotw_hour_min_sec.read(DOTW_HOUR_MIN_SEC::MIN) as u8,
                    seconds: dotw_hour_min_sec.read(DOTW_HOUR_MIN_SEC::SEC) as u8,
                };

                let get_date_result = self.date_time.set_date_time(date);

                match get_date_result {
                    Result::Ok(()) => {
                        self.in_progress.set(true);
                        CommandReturn::success()
                    }
                    Result::Err(e) => CommandReturn::failure(e),
                }
            }
        }
    }

    fn enqueue_command(
        &self,
        command: DateTimeCommand,
        year_month_dotm: u32,
        dotw_hour_min_sec: u32,
        appid: ProcessId,
    ) -> CommandReturn {
        if !self.in_progress.get() {
            let grant_enter_res = self.apps.enter(appid, |app, _| {
                app.subscribed = true;
            });
            match grant_enter_res {
                Ok(()) => self
                    .call_driver(
                        command,
                        year_month_dotm as usize,
                        dotw_hour_min_sec as usize,
                    )
                    .into(),
                Err(_e) => CommandReturn::failure(ErrorCode::FAIL),
            }
        } else {
            let grant_enter_res = self.apps.enter(appid, |app, _| {
                app.subscribed = true;
            });

            match grant_enter_res {
                Ok(()) => CommandReturn::success(),
                Err(_e) => CommandReturn::failure(ErrorCode::FAIL),
            }
        }
    }
}

impl date_time::DateTimeClient for DateTime<'_> {
    fn callback_get_date(&self, datetime: Result<date_time::Date, ErrorCode>) {
        self.in_progress.set(false);
        let mut upcall_status: usize = into_statuscode(Ok(()));
        let mut upcall_r1: usize = 0;
        let mut upcall_r2: usize = 0;

        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    app.subscribed = false;
                    match datetime {
                        Result::Ok(date) => {
                            let (year_month_dotm, dotw_hour_min_sec) =
                                match self.date_as_u32_tuple(date) {
                                    Result::Ok(t) => t,
                                    Result::Err(e) => {
                                        upcall_status = into_statuscode(Result::Err(e));
                                        (0, 0)
                                    }
                                };

                            upcall_r1 = year_month_dotm as usize;
                            upcall_r2 = dotw_hour_min_sec as usize;
                        }
                        Result::Err(e) => {
                            upcall_status = into_statuscode(Result::Err(e));
                        }
                    }

                    upcalls
                        .schedule_upcall(0, (upcall_status, upcall_r1, upcall_r2))
                        .ok();
                }
            });
        }
    }

    fn callback_set_date(&self, result: Result<(), ErrorCode>) {
        self.in_progress.set(false);

        for cntr in self.apps.iter() {
            //let mut upcall_status = UPCALL_OK;
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    app.subscribed = false;

                    upcalls
                        .schedule_upcall(0, (into_statuscode(result) as usize, 0, 0))
                        .ok();
                }
            });
        }
    }
}

impl<'a> SyscallDriver for DateTime<'a> {
    fn command(
        &self,
        command_number: usize,
        r2: usize,
        r3: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            1 => self.enqueue_command(
                DateTimeCommand::ReadDateTime,
                r2 as u32,
                r3 as u32,
                process_id,
            ),
            2 => self.enqueue_command(
                DateTimeCommand::SetDateTime,
                r2 as u32,
                r3 as u32,
                process_id,
            ),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
