use core::cell::Cell;
use kernel::{CommandReturn, Driver, ErrorCode,Grant, ProcessId};
use kernel::hil::time::{DateTime as HilDateTime,Month, DayOfWeek, RtcClient, Rtc };  
use crate::driver::NUM;
use kernel::common::cells::OptionalCell;
use kernel::debug;

pub const DRIVER_NUM: usize = NUM::Rtc as usize;

pub enum DateTimeCommand {
    Exists,
    ReadDateTime,
}

#[derive(Default, Clone,Copy )]
pub struct AppData {
    subscribed: bool,
}


pub struct DateTime<'a>{
    date_time: &'a dyn Rtc,
    apps: Grant<AppData,1>,
    in_progress: Cell<bool>,
    process_id: OptionalCell<ProcessId>,
}


/*
fn date_to_year_month_dotm(date: HilDateTime)->u32{

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
    year_month_dotm = year_month_dotm + month << 4;
    year_month_dotm = year_month_dotm + date.day << 5;

    return year_month_dotm;
}

fn date_to_dotw_hour_min_sec (date:HilDateTime)->u32{
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
    //self.apps.upcalls.schedule_upcall();

    return dotw_hour_min_sec;
}
*/

impl<'a> DateTime<'a>{

    pub fn new(date_time: &'a dyn Rtc,
    grant: Grant<AppData,1>
    ) -> DateTime{
            DateTime{
                date_time: date_time,
                apps: grant,
                in_progress: Cell::new(false),
                process_id: OptionalCell::empty(),
            
            }
    }

    

    fn call_driver(&self, command: DateTimeCommand, _: usize, _: usize) -> CommandReturn {
        match command {
            DateTimeCommand::ReadDateTime =>{ 
                let date_result = self.date_time.get_date_time();
                match date_result{
                    
                            
                    Result::Ok(d)=>{
                        match d{
                                Some(date) => { //sync
                                
                                    
                                    self.callback(Ok(date));
                                    
                                    return CommandReturn::success();
                                },    

                                
                                //async
                                None => {return CommandReturn::success();}
                        }
                    },
                    Result::Err(e)=>{
                        return CommandReturn::failure(ErrorCode::FAIL);
                    }

                        
                }
            },


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
                self.apps.enter(appid, |app, _| {
                    if !self.in_progress.get() {

                        app.subscribed = true;
                        self.in_progress.set(true); ///>>>
                        self.call_driver(command,year_month_dotm as usize,dotw_hour_min_sec as usize) 
                        
                    } else {
                        CommandReturn::failure(ErrorCode::BUSY)
                    }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }
}


impl RtcClient for DateTime<'_> {
    fn callback(&self, datetime: Result<HilDateTime, ErrorCode>){
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    self.in_progress.set(false);
                    app.subscribed = false;
                    match datetime{
                        Result::Ok(date) => {

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
                            year_month_dotm = year_month_dotm + month << 4;
                            year_month_dotm = year_month_dotm + date.day << 5;
                        
                            

                            
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

                            upcalls.schedule_upcall(0,year_month_dotm as usize, dotw_hour_min_sec as usize, 0).ok();
                        
                        },
                        Result::Err(e) => {debug!("error");}
                    }
                   
                }
            });
        }
    }
}




impl<'a> Driver for DateTime<'a>{


    fn command(&self,command_number:usize,r2:usize,r3:usize,process_id: ProcessId)->CommandReturn{
        match command_number{
            0=> CommandReturn::success(),
            1=>{
                self.enqueue_command(DateTimeCommand::ReadDateTime, r2 as u32 , r3 as u32, process_id)
                
               // CommandReturn::success()
            },
            _=>{
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::procs::Error> {
        self.apps.enter(processid, |_, _| {})
    }
    
}


