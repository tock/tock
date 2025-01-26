// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! HIL for Date Time interface

use crate::ErrorCode;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DateTimeValues {
    pub year: u16,
    pub month: Month,
    pub day: u8,
    pub day_of_week: DayOfWeek,
    pub hour: u8,
    pub minute: u8,
    pub seconds: u8,
}

/// Interface for reading and setting the current time
pub trait DateTime<'a> {
    /// Request driver to return date and time
    ///
    /// When successful, this function will be followed by the callback
    /// `callback_get_date` which provides the actual date and time
    /// or an error.
    fn get_date_time(&self) -> Result<(), ErrorCode>;

    /// Sets the current date and time
    ///
    /// When successful this function call must be followed by a call
    /// to `callback_set_date`.
    fn set_date_time(&self, date_time: DateTimeValues) -> Result<(), ErrorCode>;

    /// Sets a client that calls the callback function when date and time is requested
    fn set_client(&self, client: &'a dyn DateTimeClient);
}

/// Callback handler for when current date is read or set.
pub trait DateTimeClient {
    /// Called when a date time reading has completed.
    /// Takes `Ok(DateTime)` of current date and passes it when scheduling an upcall.
    /// If an error is encountered it takes an `Err(ErrorCode)`
    fn get_date_time_done(&self, datetime: Result<DateTimeValues, ErrorCode>);

    /// Called when a date is set
    /// Takes `Ok(())` if time is set correctly.
    /// Takes  `Err(ErrorCode)` in case of an error
    fn set_date_time_done(&self, result: Result<(), ErrorCode>);
}
