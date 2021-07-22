use core::cell::Cell;
use kernel::{CommandReturn, Driver, ErrorCode,Grant, ProcessId, Upcall};
use kernel::hil::time::{DateTime as HilDateTime,Month, DayOfWeek, RtcClient, Rtc };  
use crate::driver::NUM;
use kernel::common::cells::OptionalCell;

pub const DRIVER_NUM: usize = NUM::Rtc as usize;

pub enum DateTimeCommand {
    Exists,
    ReadDateTime,
}

#[derive(Default)]
pub struct AppData {
    callback: Upcall,
    subscribed: bool,
}


pub struct DateTime<'a>{
    date_time: &'a dyn Rtc,
    apps: Grant<AppData>,
    in_progress: Cell<bool>,
    process_id: OptionalCell<ProcessId>,
}

impl<'a> Driver for DateTime<'a>{

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Upcall,
        app_id: ProcessId,
    ) -> Result<Upcall, (Upcall, ErrorCode)> {
        match subscribe_num {
            // subscribe to rtc reading with callback
            0 => self.configure_callback(callback, app_id),
            _ => Err((callback, ErrorCode::NOSUPPORT)),
        }
    }




    fn command(&self,command_number:usize,r2:usize,r3:usize,process_id: ProcessId)->CommandReturn{
        match command_number{
            0=> CommandReturn::success(),
            1=>{
                self.enqueue_command(DateTimeCommand::ReadDateTime, process_id)
                
               // CommandReturn::success()
            },
            _=>{
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
        }
    }
/*
    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::procs::Error> {
        self.apps.enter(processid, |_, _| {})
    }
    */
}

impl<'a> DateTime<'a>{
    pub fn new(date_time: &'a dyn Rtc,
    grant: Grant<AppData>
    ) -> DateTime{
            DateTime{
                date_time: date_time,
                apps: grant,
                in_progress: Cell::new(false),
                process_id: OptionalCell::empty(),
            
            }
    }

    fn call_driver(&self, command: DateTimeCommand) -> CommandReturn {
        match command {
            DateTimeCommand::ReadDateTime =>{ 
                let date_result = self.date_time.get_date_time();
                match date_result{

                    Result::Ok(date)=>{
                        let month = match date.month{
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
                        //let mut year_month_dotm = (date.year * 10000 + month)*100 + date.day;
                        let mut year_month_dotm = date.year << 12;
                        year_month_dotm += month <<4;
                        year_month_dotm += date.day << 5;


                        let dotw = match date.day_of_week{
                            DayOfWeek::Sunday => 0,
                            DayOfWeek::Monday => 1,
                            DayOfWeek::Tuesday => 2,
                            DayOfWeek::Wednesday => 3,
                            DayOfWeek::Thursday => 4,
                            DayOfWeek::Friday => 5,
                            DayOfWeek::Saturday => 6,
                        };

                        let mut dotw_hour_min_sec = ((dotw*10 + date.hour)*100 + date.minute)*100 + date.seconds;

                        self.callback(year_month_dotm, dotw_hour_min_sec);

                        CommandReturn::success()
                    }
                    Result::Err(e)=>{
                        CommandReturn::failure(ErrorCode::FAIL)
                    }
                }

            },
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn configure_callback(
        &self,
        mut callback: Upcall,
        app_id: ProcessId,
    ) -> Result<Upcall, (Upcall, ErrorCode)> {
        let res = self.apps.enter(app_id, |app| {
                mem::swap(&mut app.callback, &mut callback); //dont know what this does
            })
            .map_err(ErrorCode::from);
        

        if let Err(e) = res {
            Err((callback, e))
        } else {
            //self.callback()
            Ok(callback)
        }
    }

    fn enqueue_command(
            &self,
            command: DateTimeCommand,

            appid: ProcessId,
        ) -> CommandReturn {
                self.apps.enter(appid, |app| {
                    if !self.in_progress.get() {

                        app.subscribed = true;
                        self.in_progress.set(true);
                        self.call_driver(command)

                    } else {
                        CommandReturn::failure(ErrorCode::BUSY)
                    }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }
}


impl RtcClient for DateTime<'_> {
    fn callback(&self, year_month_dotm: u32, dotw_hour_min_sec: u32){
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    self.in_progress.set(false);
                    app.subscribed = false;
                    upcalls.schedule_upcall(0, year_month_dotm, dotw_hour_min_sec, 0).ok();
                }
            });
        }
    }
}