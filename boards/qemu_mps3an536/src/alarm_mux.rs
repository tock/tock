// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock syscall driver capsule for Alarms, which issue callbacks when
//! a point in time has been reached.

// Note: This module implements a generic timer multiplexer for the
// Cortex R52 (and for this timer only). It is:
//  - totally redundant with the exisitng multiplexer
//  - probably buggy
//  - very basic thx to the non-wrapping 64-bits counter
//  - unoptimized (still better than the existing one)
//  - only tested with Alarm::sleep_for() libtock-rs
// The rationale for adding this module was that debugging the generic
// timer driver was too complicated with the existing multiplexer.

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::time::{self, Alarm, Ticks, Time};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};
use an536_inst::FREQ_ACLK;
use cortexr52::gentimer::GenTimer;

const ALARM_CALLBACK_NUM: usize = 0;
const NUM_UPCALLS: u8 = 1;

pub struct AlarmMux<'a> {
    alarm: &'a GenTimer<'a>,
    grant:
        Grant<u64, UpcallCount<NUM_UPCALLS>, AllowRoCount<0>, AllowRwCount<0>>,
}

impl<'a> AlarmMux<'a> {
    pub const fn new(
        alarm: &'a GenTimer<'a>,
        grant: Grant<
            u64,
            UpcallCount<NUM_UPCALLS>,
            AllowRoCount<0>,
            AllowRwCount<0>,
        >,
    ) -> AlarmMux<'a> {
        AlarmMux {
            alarm,
            grant,
        }
    }

    // must be called after new() and can be chained: .new().init()
    pub fn init(&'a self) -> &'a Self {
        self.alarm.set_alarm_client(self);
        for app in self.grant.iter() {
            app.enter(|grant_data, _| {
                **grant_data = u64::MAX;
            });
        }
        self
    }

    fn setup_alarm(&self) {
        let mut min = u64::MAX;
        let now = self.alarm.now64();

        for app in self.grant.iter() {
            app.enter(|grant_data, upcalls| {
                let exp = **grant_data;
                if exp <= now {
                    let _ = upcalls.schedule_upcall(
                        ALARM_CALLBACK_NUM,
                        ( 123, 456, 789, ), // Don't care
                    );
                    **grant_data = u64::MAX;

                } else if exp < min {
                    min = exp;
                }
            });
        }

        if min != u64::MAX {
            if min != self.alarm.get_alarm_u64() {
                // alarm will fired even is set after current time
                self.alarm.set_alarm_u64(min);
            }
        } else {
            let _ = self.alarm.disarm();
        }
    }
}

impl SyscallDriver for AlarmMux<'_> {
    fn command(
        &self,
        cmd_type: usize,
        data: usize,
        _data2: usize,
        caller_id: ProcessId,
    ) -> CommandReturn {
        self.grant
            .enter(caller_id, |exp, _upcalls| {
                match cmd_type {
                    0 => (CommandReturn::success(), false),
                    1 => (CommandReturn::success_u32(FREQ_ACLK), false),
                    2 => (CommandReturn::success_u32(self.alarm.now().into_u32_left_justified()), false),
                    5 => {
                        **exp = self.alarm.now64() + data as u64;
                        (CommandReturn::success_u32(123), true) // 123: Don't care
                    }
                    _ => (CommandReturn::failure(ErrorCode::NOSUPPORT), false),
                }
            })
            .map_or_else(
                |err| CommandReturn::failure(err.into()),
                |(retval, rearm_timer)| {
                    if rearm_timer {
                        self.setup_alarm();
                    }
                    retval
                },
            )
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grant.enter(processid, |_, _| {})
    }
}

impl time::AlarmClient for AlarmMux<'_> {
    fn alarm(&self) {
        self.setup_alarm();
    }
}

