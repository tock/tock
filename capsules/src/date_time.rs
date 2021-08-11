use crate::driver::NUM;
use core::cell::Cell;
use kernel::common::registers::{register_bitfields, LocalRegisterCopy};
use kernel::debug;
use kernel::hil::time::{DateTime as HilDateTime, Rtc, RtcClient, DayOfWeek as HilDayOfWeek, Month as HilMonth};
use kernel::{CommandReturn, Driver, ErrorCode, Grant, ProcessId};
use core::convert::{TryInto, TryFrom};

pub const DRIVER_NUM: usize = NUM::Rtc as usize;

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

    fn call_driver(&self, command: DateTimeCommand, r2: usize, r3: usize) -> CommandReturn {
        match command {
            DateTimeCommand::ReadDateTime => {
                let date_result = self.date_time.get_date_time();
                match date_result {
                    Result::Ok(d) => {
                        match d {
                            Some(date) => {
                                //sync

                                self.callback_get_date(Ok(date));

                                CommandReturn::success()
                            }

                            //async
                            None => CommandReturn::success(),
                        }
                    }
                    Result::Err(_e) => CommandReturn::failure(ErrorCode::FAIL),
                }
            }
            DateTimeCommand::SetDateTime =>{
                let year_month_dotm:LocalRegisterCopy<u32, YEAR_MONTH_DOTM::Register>= LocalRegisterCopy::new(r2 as u32);
                let dotw_hour_min_sec:LocalRegisterCopy<u32, DOTW_HOUR_MIN_SEC::Register>=LocalRegisterCopy::new(r3 as u32);


                let date = HilDateTime{

                    year: year_month_dotm.read(YEAR_MONTH_DOTM::YEAR),
                    month: match HilMonth::try_from(year_month_dotm.read(YEAR_MONTH_DOTM::MONTH) as usize){
                        Result::Ok(t) => t,
                        Result::Err(())=> {return CommandReturn::failure(ErrorCode::INVAL);},
                    },
                    day: year_month_dotm.read(YEAR_MONTH_DOTM::DAY),
                    day_of_week: match HilDayOfWeek::try_from(dotw_hour_min_sec.read(DOTW_HOUR_MIN_SEC::DOTW) as usize){
                        Result::Ok(t) => t,
                        Result::Err(())=> {return CommandReturn::failure(ErrorCode::INVAL);},
                    },
                    hour: dotw_hour_min_sec.read(DOTW_HOUR_MIN_SEC::HOUR),
                    minute: dotw_hour_min_sec.read(DOTW_HOUR_MIN_SEC::MIN),
                    seconds: dotw_hour_min_sec.read(DOTW_HOUR_MIN_SEC::SEC)
                };

                let  get_date_result = self.date_time.set_date_time(date);

                match get_date_result{
                    Result::Ok(d) =>{
                        match d {
                            Some(_date) =>{


                                self.callback_set_date(Ok(()));
                                CommandReturn::success()
                            },
                            None => CommandReturn::success()
                        }
                    },
                    Result::Err(e)=> CommandReturn::failure(e)

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
    fn callback_get_date(&self, datetime: Result<HilDateTime, ErrorCode>) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                app.subscribed = true;
                if app.subscribed {
                    self.in_progress.set(false);
                    app.subscribed = false;
                    match datetime {
                        Result::Ok(date) => {

                            let month:usize = match date.month.try_into(){
                                Result::Ok(t)=>t,
                                Result::Err(())=>{return ();}
                            };

                            let dotw:usize = match date.day_of_week.try_into(){
                                Result::Ok(t) => t,
                                Result::Err(())=> {return ();}
                            };



                            let mut year_month_dotm:LocalRegisterCopy<u32, YEAR_MONTH_DOTM::Register>= LocalRegisterCopy::new(0);
                            let mut dotw_hour_min_sec:LocalRegisterCopy<u32, DOTW_HOUR_MIN_SEC::Register>=LocalRegisterCopy::new(0);

                            year_month_dotm.modify(YEAR_MONTH_DOTM::YEAR.val(date.year));
                            year_month_dotm.modify(YEAR_MONTH_DOTM::MONTH.val(month as u32));
                            year_month_dotm.modify(YEAR_MONTH_DOTM::DAY.val(date.day));

                            dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::DOTW.val(dotw as u32));
                            dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::HOUR.val(date.hour));
                            dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::MIN.val(date.minute));
                            dotw_hour_min_sec.modify(DOTW_HOUR_MIN_SEC::SEC.val(date.seconds));


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

    fn callback_set_date(&self, result: Result<(), ErrorCode>) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                app.subscribed = true;
                if app.subscribed {
                    self.in_progress.set(false);
                    app.subscribed = false;
                    match result {
                        Result::Ok(()) => {


                            upcalls
                                .schedule_upcall(
                                    0,
                                    0,
                                    0,
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
            2 => self.enqueue_command(
                DateTimeCommand::SetDateTime,
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
