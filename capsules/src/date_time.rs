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
//!  kernel::hil::time::Rtc::set_client(&peripherals.rtc, date_time);
//! ```

use crate::driver::NUM;
use core::cell::Cell;
use kernel::grant::Grant;
use kernel::hil::time::{
    DateTime as HilDateTime, DayOfWeek as HilDayOfWeek, Month as HilMonth, Rtc, RtcClient,
};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::registers::{register_bitfields, LocalRegisterCopy};
use kernel::{ErrorCode, ProcessId};

pub const DRIVER_NUM: usize = NUM::Rtc as usize;

pub const UPCALL_OK: u32 = 1;
pub const UPCALL_ERR: u32 = 0;

pub enum DateTimeCommand {
    Exists,
    ReadDateTime,
    SetDateTime,
}

#[derive(Default, Clone, Copy)]
pub struct AppData {
    subscribed: bool,
}

pub struct DateTime<'a> {
    date_time: &'a dyn Rtc<'a>,
    apps: Grant<AppData, 1>,
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
    pub fn new(date_time: &'a dyn Rtc<'a>, grant: Grant<AppData, 1>) -> DateTime<'a> {
        DateTime {
            date_time,
            apps: grant,
            in_progress: Cell::new(false),
        }
    }

    fn month_as_u32(&self, month: HilMonth) -> u32 {
        match month {
            HilMonth::January => 1,
            HilMonth::February => 2,
            HilMonth::March => 3,
            HilMonth::April => 4,
            HilMonth::May => 5,
            HilMonth::June => 6,
            HilMonth::July => 7,
            HilMonth::August => 8,
            HilMonth::September => 9,
            HilMonth::October => 10,
            HilMonth::November => 11,
            HilMonth::December => 12,
        }
    }

    fn u32_as_month(&self, month_num: u32) -> Result<HilMonth, ErrorCode> {
        match month_num {
            1 => Ok(HilMonth::January),
            2 => Ok(HilMonth::February),
            3 => Ok(HilMonth::March),
            4 => Ok(HilMonth::April),
            5 => Ok(HilMonth::May),
            6 => Ok(HilMonth::June),
            7 => Ok(HilMonth::July),
            8 => Ok(HilMonth::August),
            9 => Ok(HilMonth::September),
            10 => Ok(HilMonth::October),
            11 => Ok(HilMonth::November),
            12 => Ok(HilMonth::December),
            _ => Err(ErrorCode::INVAL),
        }
    }

    fn dotw_as_u32(&self, dotw: HilDayOfWeek) -> u32 {
        match dotw {
            HilDayOfWeek::Sunday => 0,
            HilDayOfWeek::Monday => 1,
            HilDayOfWeek::Tuesday => 2,
            HilDayOfWeek::Wednesday => 3,
            HilDayOfWeek::Thursday => 4,
            HilDayOfWeek::Friday => 5,
            HilDayOfWeek::Saturday => 6,
        }
    }

    fn u32_as_dotw(&self, dotw_num: u32) -> Result<HilDayOfWeek, ErrorCode> {
        match dotw_num {
            0 => Ok(HilDayOfWeek::Sunday),
            1 => Ok(HilDayOfWeek::Monday),
            2 => Ok(HilDayOfWeek::Tuesday),
            3 => Ok(HilDayOfWeek::Wednesday),
            4 => Ok(HilDayOfWeek::Thursday),
            5 => Ok(HilDayOfWeek::Friday),
            6 => Ok(HilDayOfWeek::Saturday),
            _ => Err(ErrorCode::INVAL),
        }
    }

    fn date_as_u32_tuple(&self, date: HilDateTime) -> Result<(u32, u32), ErrorCode> {
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
                    Result::Ok(()) => CommandReturn::success(),
                    Result::Err(e) => CommandReturn::failure(e),
                }
            }
            DateTimeCommand::SetDateTime => {
                let year_month_dotm: LocalRegisterCopy<u32, YEAR_MONTH_DOTM::Register> =
                    LocalRegisterCopy::new(r2 as u32);
                let dotw_hour_min_sec: LocalRegisterCopy<u32, DOTW_HOUR_MIN_SEC::Register> =
                    LocalRegisterCopy::new(r3 as u32);

                let date = HilDateTime {
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
                    Result::Ok(()) => CommandReturn::success(),
                    Result::Err(e) => CommandReturn::failure(e),
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
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
            self.in_progress.set(true);

            let grant_enter_res = self.apps.enter(appid, |app, _| {
                app.subscribed = true;
            });

            match grant_enter_res {
                Ok(()) => {}
                Err(_e) => {
                    return CommandReturn::failure(ErrorCode::FAIL);
                }
            }

            self.call_driver(
                command,
                year_month_dotm as usize,
                dotw_hour_min_sec as usize,
            )
            .into()
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

impl RtcClient for DateTime<'_> {
    fn callback_get_date(&self, datetime: Result<HilDateTime, ErrorCode>) {
        self.in_progress.set(false);
        let mut upcall_status: u32 = UPCALL_OK;
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
                                    Result::Err(_e) => {
                                        upcall_status = UPCALL_ERR;
                                        (0, 0)
                                    }
                                };

                            upcall_r1 = year_month_dotm as usize;
                            upcall_r2 = dotw_hour_min_sec as usize;
                        }
                        Result::Err(_e) => {
                            upcall_status = UPCALL_ERR;
                        }
                    }

                    upcalls
                        .schedule_upcall(0, (upcall_status as usize, upcall_r1, upcall_r2))
                        .ok();
                }
            });
        }
    }

    fn callback_set_date(&self, result: Result<(), ErrorCode>) {
        self.in_progress.set(false);

        for cntr in self.apps.iter() {
            let mut upcall_status = UPCALL_OK;
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    app.subscribed = false;
                    match result {
                        Result::Ok(()) => {}
                        Result::Err(_e) => {
                            upcall_status = UPCALL_ERR;
                        }
                    }
                    upcalls
                        .schedule_upcall(0, (upcall_status as usize, 0, 0))
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
