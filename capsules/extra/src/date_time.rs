// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Real Time Clock driver
//!
//! Allows handling of the current date and time
//!
//! Authors: Irina Bradu <irinabradu.a@gmail.com>
//!          Remus Rughinis <remus.rughinis.007@gmail.com>
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//!  let grant_dt = create_capability!(capabilities::MemoryAllocationCapability);
//!  let grant_date_time = board_kernel.create_grant(capsules::date_time::DRIVER_NUM, &grant_dt);
//!
//!  let date_time = static_init!(
//!     capsules::date_time::DateTime<'static>,
//!     capsules::date_time::DateTime::new(&peripherals.rtc, grant_date_time)
//!  );
//!  kernel::hil::date_time::DateTime::set_client(&peripherals.rtc, date_time);
//! ```
//!
//! A DateTimeValues structure can be transformed to and from a u32 tuple in the following way:
//!         first number (year, month, day_of_the_month):
//!                 -last 5 bits store the day_of_the_month
//!                 -previous 4 bits store the month
//!                 -previous 12 bits store the year
//!         second number (day_of_the_week, hour, minute, seconds):
//!                 -last 6 bits store the seconds
//!                 -previous 6 store the minute
//!                 -previous 5 store the hour
//!                 -previous 3 store the day_of_the_week

use capsules_core::driver::NUM;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::date_time;

use kernel::errorcode::into_statuscode;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};

pub const DRIVER_NUM: usize = NUM::DateTime as usize;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DateTimeCommand {
    ReadDateTime,
    SetDateTime(u32, u32),
}

#[derive(Default)]
pub struct AppData {
    task: Option<DateTimeCommand>,
}

pub struct DateTimeCapsule<'a, DateTime: date_time::DateTime<'a>> {
    date_time: &'a DateTime,
    apps: Grant<AppData, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    in_progress: OptionalCell<ProcessId>,
}

fn month_as_u32(month: date_time::Month) -> u32 {
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

fn u32_as_month(month_num: u32) -> Result<date_time::Month, ErrorCode> {
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

fn dotw_as_u32(dotw: date_time::DayOfWeek) -> u32 {
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

fn u32_as_dotw(dotw_num: u32) -> Result<date_time::DayOfWeek, ErrorCode> {
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

/// Transforms two u32 numbers into a DateTimeValues structure (year, month, dotm, dotw, hour, minute, seconds)
/// Check file documentation for details on how the u32 tuple stores data
fn date_from_u32_tuple(date: u32, time: u32) -> Result<date_time::DateTimeValues, ErrorCode> {
    let month_num = date % (1 << 9) / (1 << 5);
    let month_name = u32_as_month(month_num)?;

    let dotw_num = time % (1 << 20) / (1 << 17);
    let dotw_name = u32_as_dotw(dotw_num)?;

    let date_result = date_time::DateTimeValues {
        year: (date % (1 << 21) / (1 << 9)) as u16,
        month: month_name,
        day: (date % (1 << 5)) as u8,

        day_of_week: dotw_name,
        hour: (time % (1 << 17) / (1 << 12)) as u8,
        minute: (time % (1 << 12) / (1 << 6)) as u8,
        seconds: (time % (1 << 6)) as u8,
    };

    if !(date_result.day <= 31) {
        return Err(ErrorCode::INVAL);
    }
    if !(date_result.hour <= 24) {
        return Err(ErrorCode::INVAL);
    }
    if !(date_result.minute <= 60) {
        return Err(ErrorCode::INVAL);
    }
    if !(date_result.seconds <= 60) {
        return Err(ErrorCode::INVAL);
    }

    Ok(date_result)
}

/// Transforms DateTimeValues structure (year, month, dotm, dotw, hour, minute, seconds) into two u32 numbers
/// Check file documentation for details on how the u32 numbers stores data
/// The two u32 numbers are returned as a tuple
fn date_as_u32_tuple(set_date: date_time::DateTimeValues) -> Result<(u32, u32), ErrorCode> {
    let month = month_as_u32(set_date.month);
    let dotw = dotw_as_u32(set_date.day_of_week);

    let date =
        set_date.year as u32 * (1 << 9) as u32 + month * (1 << 5) as u32 + set_date.day as u32;
    let time = dotw * (1 << 17) as u32
        + set_date.hour as u32 * (1 << 12) as u32
        + set_date.minute as u32 * (1 << 6) as u32
        + set_date.seconds as u32;

    Ok((date, time))
}

impl<'a, DateTime: date_time::DateTime<'a>> DateTimeCapsule<'a, DateTime> {
    pub fn new(
        date_time: &'a DateTime,
        grant: Grant<AppData, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> DateTimeCapsule<'a, DateTime> {
        DateTimeCapsule {
            date_time,
            apps: grant,
            in_progress: OptionalCell::empty(),
        }
    }

    fn call_driver(&self, command: DateTimeCommand, processid: ProcessId) -> Result<(), ErrorCode> {
        match command {
            DateTimeCommand::ReadDateTime => {
                let date_result = self.date_time.get_date_time();
                match date_result {
                    Result::Ok(()) => {
                        self.in_progress.set(processid);
                        Ok(())
                    }
                    Result::Err(e) => Err(e),
                }
            }
            DateTimeCommand::SetDateTime(r2, r3) => {
                let date = match date_from_u32_tuple(r2, r3) {
                    Result::Ok(d) => d,
                    Result::Err(e) => {
                        return Err(e);
                    }
                };

                let get_date_result = self.date_time.set_date_time(date);

                match get_date_result {
                    Result::Ok(()) => {
                        self.in_progress.set(processid);
                        Ok(())
                    }
                    Result::Err(e) => Err(e),
                }
            }
        }
    }

    fn enqueue_command(&self, command: DateTimeCommand, processid: ProcessId) -> CommandReturn {
        let grant_enter_res = self.apps.enter(processid, |app, _| {
            if !(app.task.is_none()) {
                CommandReturn::failure(ErrorCode::BUSY)
            } else {
                app.task = Some(command);
                CommandReturn::success()
            }
        });

        // If no command is currently run, run the current command
        if self.in_progress.is_none() {
            match grant_enter_res {
                Ok(_) => match self.call_driver(command, processid) {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                },
                Err(_e) => CommandReturn::failure(ErrorCode::FAIL),
            }
        } else {
            match grant_enter_res {
                Ok(_) => CommandReturn::success(),
                Err(_e) => CommandReturn::failure(ErrorCode::FAIL),
            }
        }
    }

    fn queue_next_command(&self) {
        self.apps.iter().find_map(|grant| {
            let processid = grant.processid();
            grant.enter(|app, kernel| {
                app.task.map_or(None, |command| {
                    let command_return = self.call_driver(command, processid);
                    match command_return {
                        Ok(()) => Some(()),
                        Err(e) => {
                            let upcall_status = into_statuscode(Err(e));
                            kernel.schedule_upcall(0, (upcall_status, 0, 0)).ok();
                            None
                        }
                    }
                })
            })
        });
    }
}

impl<'a, DateTime: date_time::DateTime<'a>> date_time::DateTimeClient
    for DateTimeCapsule<'a, DateTime>
{
    fn get_date_time_done(&self, datetime: Result<date_time::DateTimeValues, ErrorCode>) {
        self.in_progress.clear();
        let mut upcall_status: usize = into_statuscode(Ok(()));
        let mut upcall_r1: usize = 0;
        let mut upcall_r2: usize = 0;

        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.task == Some(DateTimeCommand::ReadDateTime) {
                    app.task = None;
                    match datetime {
                        Result::Ok(date) => {
                            let (year_month_dotm, dotw_hour_min_sec) = match date_as_u32_tuple(date)
                            {
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

        self.queue_next_command();
    }

    fn set_date_time_done(&self, result: Result<(), ErrorCode>) {
        // in_progress.take() also sets the OptionalCell to None
        let processid = self.in_progress.take().unwrap();
        let _enter_grant = self.apps.enter(processid, |app, upcalls| {
            app.task = None;

            upcalls
                .schedule_upcall(0, (into_statuscode(result), 0, 0))
                .ok();
        });

        self.queue_next_command();
    }
}

impl<'a, DateTime: date_time::DateTime<'a>> SyscallDriver for DateTimeCapsule<'a, DateTime> {
    fn command(
        &self,
        command_number: usize,
        r2: usize,
        r3: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            1 => self.enqueue_command(DateTimeCommand::ReadDateTime, process_id),
            2 => self.enqueue_command(
                DateTimeCommand::SetDateTime(r2 as u32, r3 as u32),
                process_id,
            ),
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
