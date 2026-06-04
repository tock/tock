// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Callback-driven STM harness for the `test-harness` image.
//!
//! The suite uses two `VirtualMuxAlarm`s. The board's `MuxAlarm` remains the
//! sole STM client; one virtual alarm drives the suite and the other observes
//! that a disarmed deadline cannot invoke the suite callback.

use core::cell::Cell;

use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient};
use capsules_core::virtualizers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::hil::time::{Alarm, AlarmClient, Ticks, Ticks32, Time};
use kernel::static_init;
use kernel::utilities::cells::OptionalCell;
use nxp_s32g3::stm::{Stm, STM_FREQUENCY_HZ};
use nxp_s32g3::trace_sync;

const TEN_MILLISECONDS_TICKS: u32 = STM_FREQUENCY_HZ / 100;
const ONE_HUNDRED_MILLISECONDS_TICKS: u32 = STM_FREQUENCY_HZ / 10;
const ONE_HUNDRED_MILLISECONDS_CALLBACKS: u8 = 10;

const MIN_DELTA_ARMED: &str = "S32G3_STM_TEST step=min result=ARMED";
const MIN_DELTA_PASS: &str = "S32G3_STM_TEST step=min result=PASS";
const TEN_MILLISECONDS_PASS: &str = "S32G3_STM_TEST step=10ms result=PASS";
const DISARM_PASS: &str = "S32G3_STM_TEST step=disarm result=PASS count=0";
const DISARM_FAIL: &str = "S32G3_STM_TEST step=disarm result=FAIL";
const COMPLETE: &str = "S32G3_STM_TEST step=complete result=PASS";
const HOST_TIMING_READY: &str = "S32G3_STM_TEST step=host_timing result=READY";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    MinDelta,
    TenMilliseconds,
    SeededWrap { before: u32 },
    DisarmVerify,
    OneHundredMilliseconds(u8),
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OneHundredMillisecondsCallbackAction {
    Rearm,
    Emit,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SuiteState {
    phase: Phase,
    completed: bool,
}

impl SuiteState {
    const fn new() -> Self {
        Self {
            phase: Phase::MinDelta,
            completed: false,
        }
    }

    fn advance_after_min_delta(&mut self) {
        self.phase = Phase::TenMilliseconds;
    }

    fn begin_disarm_verify(&mut self) {
        self.phase = Phase::DisarmVerify;
    }

    fn begin_one_hundred_millisecond_callbacks(&mut self) {
        self.phase = Phase::OneHundredMilliseconds(0);
    }

    fn advance_after_one_hundred_milliseconds(&mut self) -> bool {
        let Phase::OneHundredMilliseconds(callbacks) = self.phase else {
            return false;
        };
        if callbacks + 1 == ONE_HUNDRED_MILLISECONDS_CALLBACKS {
            self.phase = Phase::Complete;
            true
        } else {
            self.phase = Phase::OneHundredMilliseconds(callbacks + 1);
            false
        }
    }

    fn complete_once(&mut self) -> bool {
        if self.phase != Phase::Complete || self.completed {
            return false;
        }
        self.completed = true;
        true
    }

    fn one_hundred_millisecond_callback_actions(
        done: bool,
    ) -> &'static [OneHundredMillisecondsCallbackAction] {
        if done {
            &[
                OneHundredMillisecondsCallbackAction::Emit,
                OneHundredMillisecondsCallbackAction::Complete,
            ]
        } else {
            &[
                OneHundredMillisecondsCallbackAction::Rearm,
                OneHundredMillisecondsCallbackAction::Emit,
            ]
        }
    }
}

struct DisarmObserver {
    suite: Cell<Option<&'static StmSuite>>,
}

impl DisarmObserver {
    const fn new() -> Self {
        Self {
            suite: Cell::new(None),
        }
    }

    fn set_suite(&self, suite: &'static StmSuite) {
        self.suite.set(Some(suite));
    }
}

impl AlarmClient for DisarmObserver {
    fn alarm(&self) {
        if let Some(suite) = self.suite.get() {
            suite.verify_disarmed_deadline();
        }
    }
}

struct StmSuite {
    alarm: &'static VirtualMuxAlarm<'static, Stm<'static>>,
    observer_alarm: &'static VirtualMuxAlarm<'static, Stm<'static>>,
    stm: &'static Stm<'static>,
    client: OptionalCell<&'static dyn CapsuleTestClient>,
    state: Cell<SuiteState>,
    disarmed_target_callbacks: Cell<u8>,
}

impl StmSuite {
    const fn new(
        alarm: &'static VirtualMuxAlarm<'static, Stm<'static>>,
        observer_alarm: &'static VirtualMuxAlarm<'static, Stm<'static>>,
        stm: &'static Stm<'static>,
        client: &'static dyn CapsuleTestClient,
    ) -> Self {
        Self {
            alarm,
            observer_alarm,
            stm,
            client: OptionalCell::new(client),
            state: Cell::new(SuiteState::new()),
            disarmed_target_callbacks: Cell::new(0),
        }
    }

    fn begin_seeded_wrap(&self) {
        let before = u32::MAX - 8;
        self.stm.seed_counter_for_test(before);
        let mut state = self.state.get();
        state.phase = Phase::SeededWrap { before };
        self.state.set(state);
        self.arm(Ticks32::from(16));
    }

    fn arm(&self, dt: Ticks32) {
        self.alarm.set_alarm(self.alarm.now(), dt);
    }

    fn start(&self) {
        trace_sync!("{}", MIN_DELTA_ARMED);
        self.arm(self.alarm.minimum_dt());
    }

    fn begin_disarm_verifier(&self) {
        let target_deadline = Ticks32::from(TEN_MILLISECONDS_TICKS);
        self.disarmed_target_callbacks.set(0);
        self.arm(target_deadline);
        let disarmed = self.alarm.disarm().is_ok() && !self.alarm.is_armed();
        if !disarmed {
            trace_sync!("{}", DISARM_FAIL);
        }
        self.observer_alarm.set_alarm(
            self.observer_alarm.now(),
            target_deadline.wrapping_add(Ticks32::from(TEN_MILLISECONDS_TICKS)),
        );
    }

    fn verify_disarmed_deadline(&self) {
        let mut state = self.state.get();
        if state.phase != Phase::DisarmVerify {
            return;
        }
        let passed = self.disarmed_target_callbacks.get() == 0;
        trace_sync!("{}", if passed { DISARM_PASS } else { DISARM_FAIL });
        state.begin_one_hundred_millisecond_callbacks();
        self.state.set(state);
        trace_sync!("{}", HOST_TIMING_READY);
        self.arm(Ticks32::from(ONE_HUNDRED_MILLISECONDS_TICKS));
    }
}

impl AlarmClient for StmSuite {
    fn alarm(&self) {
        let mut state = self.state.get();
        match state.phase {
            Phase::MinDelta => {
                trace_sync!("{}", MIN_DELTA_PASS);
                state.advance_after_min_delta();
                self.state.set(state);
                self.arm(Ticks32::from(TEN_MILLISECONDS_TICKS));
            }
            Phase::TenMilliseconds => {
                trace_sync!("{}", TEN_MILLISECONDS_PASS);
                self.begin_seeded_wrap();
            }
            Phase::SeededWrap { before } => {
                let after = self.alarm.now().into_u32();
                trace_sync!(
                    "S32G3_STM_TEST step=wrap result={} before={} after={}",
                    if after < before { "PASS" } else { "FAIL" },
                    before,
                    after
                );
                state.begin_disarm_verify();
                self.state.set(state);
                self.begin_disarm_verifier();
            }
            Phase::DisarmVerify => {
                self.disarmed_target_callbacks
                    .set(self.disarmed_target_callbacks.get().saturating_add(1));
            }
            Phase::OneHundredMilliseconds(callback) => {
                let ticks = self.alarm.now().into_u32();
                let done = state.advance_after_one_hundred_milliseconds();
                self.state.set(state);
                for action in SuiteState::one_hundred_millisecond_callback_actions(done) {
                    match action {
                        OneHundredMillisecondsCallbackAction::Rearm => {
                            // Arm from the ISR timestamp before synchronous UART output
                            // so log serialization cannot extend the next interval.
                            self.arm(Ticks32::from(ONE_HUNDRED_MILLISECONDS_TICKS));
                        }
                        OneHundredMillisecondsCallbackAction::Emit => {
                            trace_sync!(
                                "S32G3_STM_TEST step=100ms result=PASS callback={} ticks={}",
                                callback + 1,
                                ticks
                            );
                        }
                        OneHundredMillisecondsCallbackAction::Complete => {
                            trace_sync!("{}", COMPLETE);
                            let mut state = self.state.get();
                            if state.complete_once() {
                                self.state.set(state);
                                self.client.map(|client| client.done(Ok(())));
                            }
                        }
                    }
                }
            }
            Phase::Complete => {}
        }
    }
}

impl CapsuleTest for StmSuite {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}
/// Begin the STM suite without blocking before `kernel_loop`.
///
/// The launcher is notified once after the 10th 100-millisecond alarm callback.
/// `stm` provides the test-harness-only safe counter seed for wrap coverage.
///
/// # Safety
/// Must be called exactly once during board startup, after the board's STM
/// `MuxAlarm` has been initialized.
pub unsafe fn run(
    mux_alarm: &'static MuxAlarm<'static, Stm<'static>>,
    stm: &'static Stm<'static>,
    client: &'static dyn CapsuleTestClient,
) {
    let alarm = static_init!(
        VirtualMuxAlarm<'static, Stm<'static>>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    alarm.setup();

    let observer_alarm = static_init!(
        VirtualMuxAlarm<'static, Stm<'static>>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    observer_alarm.setup();

    let observer = static_init!(DisarmObserver, DisarmObserver::new());
    observer_alarm.set_alarm_client(observer);

    let suite = static_init!(StmSuite, StmSuite::new(alarm, observer_alarm, stm, client));
    alarm.set_alarm_client(suite);
    observer.set_suite(suite);
    suite.start();
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn state_advances_through_callbacks_and_completes_once() {
        let mut state = SuiteState::new();
        state.advance_after_min_delta();
        assert_eq!(state.phase, Phase::TenMilliseconds);
        state.begin_disarm_verify();
        assert_eq!(state.phase, Phase::DisarmVerify);
        state.begin_one_hundred_millisecond_callbacks();

        for callback in 0..(ONE_HUNDRED_MILLISECONDS_CALLBACKS - 1) {
            assert_eq!(state.phase, Phase::OneHundredMilliseconds(callback));
            assert!(!state.advance_after_one_hundred_milliseconds());
        }
        assert!(state.advance_after_one_hundred_milliseconds());
        assert_eq!(state.phase, Phase::Complete);
        assert!(state.complete_once());
        assert!(!state.complete_once());
    }

    #[test]
    fn nonfinal_callback_rearms_before_its_synchronous_record() {
        assert_eq!(
            SuiteState::one_hundred_millisecond_callback_actions(false),
            &[
                OneHundredMillisecondsCallbackAction::Rearm,
                OneHundredMillisecondsCallbackAction::Emit
            ]
        );
        assert_eq!(
            SuiteState::one_hundred_millisecond_callback_actions(true),
            &[
                OneHundredMillisecondsCallbackAction::Emit,
                OneHundredMillisecondsCallbackAction::Complete
            ]
        );
    }
}
