use kernel::{CommandReturn, Driver, ErrorCode,ProcessId};
use kernel::hil::time;  


pub enum DateTimeCommand {
    Exists,
    ReadDateTime,
}


pub struct DateTime<'a>{
    date_time: &'a dyn hil::time::Rtc,
}

impl Driver for DateTime<'a>{
    fn command(&self,command_number:usize,r2:usize,r3:usize,process_id: ProcessId)->CommandReturn{
        match command_number{
            0=> CommandReturn::success()
            1=>{
                let date = date_time.get_date_time();
                match date{
                    Result::Ok(v)=>{
                        let month = match v.month{
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
                        let mut year_month_dotm = (v.year * 10000 + month)*100 + v.day;

                        let dotw = match v.day_of_week{
                            DayOfWeek::Sunday => 0,
                            DayOfWeek::Monday => 1,
                            DayOfWeek::Tuesday => 2,
                            DayOfWeek::Wednesday => 3,
                            DayOfWeek::Thursday => 4,
                            DayOfWeek::Friday => 5,
                            DayOfWeek::Saturday => 6,
                        };

                        let mut dotw_hour_min_sec = ((dotw*10 + v.hour)*100 + v.minute)*100 + v.seconds;

                        CommandReturn::success(,year_month_dotm,dotw);
                    }
                    Result::Err(e)=>{
                        CommandReturn::failure(ErrorCode::FAIL)
                    }

                }
                CommandReturn::success()
            }
            _=>{
                CommandReturn::failure(ErrorCode::NOSUPPORT)
            }
        }
    }
}

impl<'a> DateTime<'a>{
    pub fn new() -> Self{
        DateTime{

        }
    }

    fn enqueue_command(
        &self,
        command: DateTimeCommand,
        arg1: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        self.apps
            .enter(appid, |app| {
                if !self.busy.get() {
                    app.subscribed = true;
                    self.busy.set(true);
                    self.call_driver(command, arg1)
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }
}