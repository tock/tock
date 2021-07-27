use crate::driver::NUM;
use core::cell::Cell;
use kernel::common::registers::{register_bitfields, LocalRegisterCopy};
use kernel::debug;
use kernel::hil::time::{DateTime as HilDateTime, DayOfWeek, Month, Rtc, RtcClient};
use kernel::{CommandReturn, Driver, ErrorCode, Grant, ProcessId};

pub const DRIVER_NUM: usize = NUM::Rtc as usize;

pub enum DateTimeCommand {
    Exists,
    ReadDateTime,
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
            date_time: date_time,
            apps: grant,
            in_progress: Cell::new(false),
        }
    }

    fn call_driver(&self, command: DateTimeCommand, _: usize, _: usize) -> CommandReturn {
        match command {
            DateTimeCommand::ReadDateTime => {
                let date_result = self.date_time.get_date_time();
                match date_result {
                    Result::Ok(d) => {
                        match d {
                            Some(date) => {
                                //sync

                                self.callback(Ok(date));

                                CommandReturn::success()
                            }

                            //async
                            None => CommandReturn::success(),
                        }
                    }
                    Result::Err(_e) => CommandReturn::failure(ErrorCode::FAIL),
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
        _appid: ProcessId,
    ) -> CommandReturn {
        if !self.in_progress.get() {
            //app.subscribed = true;
            self.in_progress.set(true);
            self.call_driver(
                command,
                year_month_dotm as usize,
                dotw_hour_min_sec as usize,
            )
            .into()
        } else {
            CommandReturn::failure(ErrorCode::NOSUPPORT)
        }
    }
}

impl RtcClient for DateTime<'_> {
    fn callback(&self, datetime: Result<HilDateTime, ErrorCode>) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                app.subscribed = true;
                if app.subscribed {
                    self.in_progress.set(false);
                    app.subscribed = false;
                    match datetime {
                        Result::Ok(date) => {

                            let month = match date.month {
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
                            };



                            let dotw: u32 = match date.day_of_week {
                                DayOfWeek::Sunday => 0,
                                DayOfWeek::Monday => 1,
                                DayOfWeek::Tuesday => 2,
                                DayOfWeek::Wednesday => 3,
                                DayOfWeek::Thursday => 4,
                                DayOfWeek::Friday => 5,
                                DayOfWeek::Saturday => 6,
                            };



                            let mut year_month_dotm:LocalRegisterCopy<u32, YEAR_MONTH_DOTM::Register>= LocalRegisterCopy::new(0);
                            let mut dotw_hour_min_sec:LocalRegisterCopy<u32, DOTW_HOUR_MIN_SEC::Register>=LocalRegisterCopy::new(0);

                            year_month_dotm.modify(YEAR_MONTH_DOTM::YEAR.val(date.year));
                            year_month_dotm.modify(YEAR_MONTH_DOTM::MONTH.val(month));
                            year_month_dotm.modify(YEAR_MONTH_DOTM::DAY.val(date.day));

                            dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::DOTW.val(dotw));
                            dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::HOUR.val(date.hour));
                            dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::MIN.val(date.minute));
                            dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::SEC.val(date.seconds));

                            debug!("from capsule year: {}  month:{} day:{}   \n dotw:{} hour:{}   minute:{}  seconds:{}",date.year,month,date.day,dotw,date.hour, date.minute, date.seconds);

                            upcalls
                                .schedule_upcall(
                                    0,
                                    year_month_dotm.get() as usize,
                                    dotw_hour_min_sec.get() as usize,
                                    0,
                                )
                                .ok();
                        }
                        Result::Err(_e) => {
                            debug!("error");
                        }
                    }
                }
            });
        }
    }
}

impl<'a> Driver for DateTime<'a> {
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
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::procs::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
