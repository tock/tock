// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Deterministic LF1 UART test suite for the `test-harness` image.
//!
//! The suite needs no loopback wire. It verifies one buffered transmit and two
//! immediate deferred aborts, writes one LF0 polling result line, then halts.

use core::cell::Cell;

use capsules_core::test::capsule_test::{CapsuleTest, CapsuleTestClient};
use kernel::hil::uart::{self, Receive, Transmit};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{static_init, ErrorCode};
use nxp_s32g3::linflexd::transmit_lf0_sync;
use nxp_s32g3::linflexd::LinFlexD;

const TX_LEN: usize = 16;
const RX_LEN: usize = 4;
const RESULT_PASS: &str = "S32G3_UART_TEST result=PASS tx=PASS tx_abort=PASS rx_abort=PASS\r\n";
const RESULT_FAIL_TT: &str = "S32G3_UART_TEST result=FAIL tx=FAIL tx_abort=FAIL rx_abort=PASS\r\n";
const RESULT_FAIL_TR: &str = "S32G3_UART_TEST result=FAIL tx=FAIL tx_abort=PASS rx_abort=FAIL\r\n";
const RESULT_FAIL_T: &str = "S32G3_UART_TEST result=FAIL tx=FAIL tx_abort=PASS rx_abort=PASS\r\n";
const RESULT_FAIL_AT: &str = "S32G3_UART_TEST result=FAIL tx=PASS tx_abort=FAIL rx_abort=FAIL\r\n";
const RESULT_FAIL_A: &str = "S32G3_UART_TEST result=FAIL tx=PASS tx_abort=FAIL rx_abort=PASS\r\n";
const RESULT_FAIL_R: &str = "S32G3_UART_TEST result=FAIL tx=PASS tx_abort=PASS rx_abort=FAIL\r\n";
const RESULT_FAIL_ALL: &str = "S32G3_UART_TEST result=FAIL tx=FAIL tx_abort=FAIL rx_abort=FAIL\r\n";
const PANIC_RESOURCES_BOUND: &str =
    "S32G3_S12 panic_resources chip=bound processes=bound printer=bound\r\n";

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    TxSuccess,
    TxAbort,
    RxAbort,
    Done,
}

#[derive(Clone, Copy)]
struct SuiteResult {
    phase: Phase,
    tx_ok: bool,
    tx_abort_ok: bool,
    rx_abort_ok: bool,
    tx_abort_preconditions_ok: bool,
    rx_abort_preconditions_ok: bool,
    emitted: bool,
}

impl SuiteResult {
    const fn new() -> Self {
        Self {
            phase: Phase::TxSuccess,
            tx_ok: false,
            tx_abort_ok: false,
            rx_abort_ok: false,
            tx_abort_preconditions_ok: false,
            rx_abort_preconditions_ok: false,
            emitted: false,
        }
    }

    fn tx_success(&mut self, rval: Result<(), ErrorCode>, tx_len: usize) -> bool {
        if self.phase != Phase::TxSuccess {
            return false;
        }
        self.tx_ok = rval == Ok(()) && tx_len == TX_LEN;
        self.phase = Phase::TxAbort;
        true
    }

    fn tx_abort(
        &mut self,
        preconditions_ok: bool,
        rval: Result<(), ErrorCode>,
        tx_len: usize,
    ) -> bool {
        if self.phase != Phase::TxAbort {
            return false;
        }
        self.tx_abort_preconditions_ok = preconditions_ok;
        self.tx_abort_ok = preconditions_ok && rval == Err(ErrorCode::CANCEL) && tx_len == 0;
        self.phase = Phase::RxAbort;
        true
    }

    fn rx_abort(
        &mut self,
        preconditions_ok: bool,
        rval: Result<(), ErrorCode>,
        rx_len: usize,
        error: uart::Error,
    ) -> bool {
        if self.phase != Phase::RxAbort {
            return false;
        }
        self.rx_abort_preconditions_ok = preconditions_ok;
        self.rx_abort_ok = preconditions_ok
            && rval == Err(ErrorCode::CANCEL)
            && rx_len == 0
            && error == uart::Error::Aborted;
        self.phase = Phase::Done;
        true
    }

    fn fail_current(&mut self) {
        match self.phase {
            Phase::TxSuccess => self.tx_ok = false,
            Phase::TxAbort => self.tx_abort_ok = false,
            Phase::RxAbort => self.rx_abort_ok = false,
            Phase::Done => return,
        }
        self.phase = match self.phase {
            Phase::TxSuccess => Phase::TxAbort,
            Phase::TxAbort => Phase::RxAbort,
            Phase::RxAbort | Phase::Done => Phase::Done,
        };
    }

    fn take_result_line(&mut self) -> Option<&'static str> {
        if self.phase != Phase::Done || self.emitted {
            return None;
        }
        self.emitted = true;
        Some(match (self.tx_ok, self.tx_abort_ok, self.rx_abort_ok) {
            (true, true, true) => RESULT_PASS,
            (false, false, true) => RESULT_FAIL_TT,
            (false, true, false) => RESULT_FAIL_TR,
            (false, true, true) => RESULT_FAIL_T,
            (true, false, false) => RESULT_FAIL_AT,
            (true, false, true) => RESULT_FAIL_A,
            (true, true, false) => RESULT_FAIL_R,
            (false, false, false) => RESULT_FAIL_ALL,
        })
    }
}

struct UartTestSuite {
    uart: OptionalCell<&'static LinFlexD<'static>>,
    client: OptionalCell<&'static dyn CapsuleTestClient>,
    result: Cell<SuiteResult>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    tx_overlap: TakeCell<'static, [u8]>,
    rx_overlap: TakeCell<'static, [u8]>,
}

impl UartTestSuite {
    const fn new(client: &'static dyn CapsuleTestClient) -> Self {
        Self {
            uart: OptionalCell::empty(),
            client: OptionalCell::new(client),
            result: Cell::new(SuiteResult::new()),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            tx_overlap: TakeCell::empty(),
            rx_overlap: TakeCell::empty(),
        }
    }
    fn run_tx_abort(&self) {
        self.uart.map(|uart| {
            self.tx_buf.take().map(|buf| {
                for byte in buf.iter_mut() {
                    *byte = 0xAA;
                }
                match uart.transmit_buffer(buf, TX_LEN) {
                    Ok(()) => {
                        let mut result = self.result.get();
                        result.tx_abort_preconditions_ok = uart.transmit_abort() == Err(ErrorCode::BUSY);
                        if result.tx_abort_preconditions_ok {
                            self.tx_overlap.take().map(|overlap| {
                                match uart.transmit_buffer(overlap, TX_LEN) {
                                    Err((ErrorCode::BUSY, overlap)) => {
                                        self.tx_overlap.replace(overlap);
                                    }
                                    Err((_error, overlap)) => {
                                        self.tx_overlap.replace(overlap);
                                        result.tx_abort_preconditions_ok = false;
                                    }
                                    Ok(()) => {
                                        result.tx_abort_preconditions_ok = false;
                                    }
                                }
                            });
                        }
                        self.result.set(result);
                    }
                    Err((_error, buf)) => {
                        self.tx_buf.replace(buf);
                        self.fail_current();
                    }
                }
            });
        });
    }

    fn start_phase(&self) {
        match self.result.get().phase {
            Phase::TxSuccess => self.run_tx_success(),
            Phase::TxAbort => self.run_tx_abort(),
            Phase::RxAbort => self.run_rx_abort(),
            Phase::Done => self.report(),
        }
    }

    fn fail_current(&self) {
        let mut result = self.result.get();
        result.fail_current();
        self.result.set(result);
        self.start_phase();
    }

    fn run_tx_success(&self) {
        self.uart.map(|uart| {
            self.tx_buf.take().map(|buf| {
                for (index, byte) in buf.iter_mut().enumerate() {
                    *byte = b'A' + (index as u8 % 26);
                }
                if let Err((_error, buf)) = uart.transmit_buffer(buf, TX_LEN) {
                    self.tx_buf.replace(buf);
                    self.fail_current();
                }
            });
        });
    }


    fn run_rx_abort(&self) {
        self.uart.map(|uart| {
            self.rx_buf.take().map(|buf| match uart.receive_buffer(buf, RX_LEN) {
                Ok(()) => {
                    let mut result = self.result.get();
                    result.rx_abort_preconditions_ok = uart.receive_abort() == Err(ErrorCode::BUSY);
                    if result.rx_abort_preconditions_ok {
                        self.rx_overlap.take().map(|overlap| {
                            match uart.receive_buffer(overlap, RX_LEN) {
                                Err((ErrorCode::BUSY, overlap)) => {
                                    self.rx_overlap.replace(overlap);
                                }
                                Err((_error, overlap)) => {
                                    self.rx_overlap.replace(overlap);
                                    result.rx_abort_preconditions_ok = false;
                                }
                                Ok(()) => {
                                    result.rx_abort_preconditions_ok = false;
                                }
                            }
                        });
                    }
                    self.result.set(result);
                }
                Err((_error, buf)) => {
                    self.rx_buf.replace(buf);
                    self.fail_current();
                }
            });
        });
    }

    fn report(&self) {
        let mut result = self.result.get();
        if let Some(line) = result.take_result_line() {
            self.result.set(result);
            let resources_bound = crate::PANIC_RESOURCES.get().is_some_and(|resources| {
                resources.chip.is_some()
                    && resources.processes.is_some()
                    && resources.printer.is_some()
            });
            transmit_lf0_sync(if resources_bound {
                PANIC_RESOURCES_BOUND.as_bytes()
            } else {
                b"S32G3_S12 panic_resources result=UNBOUND\r\n"
            });
            transmit_lf0_sync(line.as_bytes());
            self.client.map(|client| client.done(Ok(())));
        }
    }
}

impl CapsuleTest for UartTestSuite {
    fn set_client(&self, client: &'static dyn CapsuleTestClient) {
        self.client.set(client);
    }
}

impl uart::TransmitClient for UartTestSuite {
    fn transmitted_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        rval: Result<(), ErrorCode>,
    ) {
        self.tx_buf.replace(tx_buffer);
        let mut result = self.result.get();
        let advanced = match result.phase {
            Phase::TxSuccess => result.tx_success(rval, tx_len),
            Phase::TxAbort => result.tx_abort(result.tx_abort_preconditions_ok, rval, tx_len),
            Phase::RxAbort | Phase::Done => false,
        };
        self.result.set(result);
        if advanced {
            self.start_phase();
        }
    }

    fn transmitted_word(&self, _rval: Result<(), ErrorCode>) {}
}

impl uart::ReceiveClient for UartTestSuite {
    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rval: Result<(), ErrorCode>,
        error: uart::Error,
    ) {
        self.rx_buf.replace(rx_buffer);
        let mut result = self.result.get();
        let advanced = result.rx_abort(result.rx_abort_preconditions_ok, rval, rx_len, error);
        self.result.set(result);
        if advanced {
            self.start_phase();
        }
    }
}

/// Start the deterministic suite after LF1 is configured and deferred calls exist.
///
/// # Safety
/// Must be called exactly once during board startup, after the board's linflex modules
/// have been initialized.
pub unsafe fn run(lf1: &'static LinFlexD<'static>, client: &'static dyn CapsuleTestClient) {
    let tx_buf = static_init!([u8; TX_LEN], [0; TX_LEN]);
    let rx_buf = static_init!([u8; RX_LEN], [0; RX_LEN]);
    let tx_overlap = static_init!([u8; TX_LEN], [0; TX_LEN]);
    let rx_overlap = static_init!([u8; RX_LEN], [0; RX_LEN]);

    let suite = static_init!(UartTestSuite, UartTestSuite::new(client));
    suite.uart.set(lf1);
    suite.tx_buf.replace(tx_buf);
    suite.rx_buf.replace(rx_buf);
    suite.tx_overlap.replace(tx_overlap);
    suite.rx_overlap.replace(rx_overlap);
    lf1.set_transmit_client(suite);
    lf1.set_receive_client(suite);
    suite.start_phase();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_success_requires_ok_and_exact_length() {
        let mut wrong_result = SuiteResult::new();
        assert!(wrong_result.tx_success(Err(ErrorCode::FAIL), TX_LEN));
        assert!(!wrong_result.tx_ok);

        let mut wrong_length = SuiteResult::new();
        assert!(wrong_length.tx_success(Ok(()), TX_LEN - 1));
        assert!(!wrong_length.tx_ok);
    }

    #[test]
    fn rejects_tx_abort_without_cancel_zero_or_preconditions() {
        let cases = [
            (false, Err(ErrorCode::CANCEL), 0),
            (true, Ok(()), 0),
            (true, Err(ErrorCode::CANCEL), 1),
        ];

        for (preconditions_ok, rval, len) in cases {
            let mut result = SuiteResult::new();
            assert!(result.tx_success(Ok(()), TX_LEN));
            assert!(result.tx_abort(preconditions_ok, rval, len));
            assert!(!result.tx_abort_ok);
        }
    }

    #[test]
    fn rejects_rx_abort_without_cancel_zero_aborted_or_preconditions() {
        let cases = [
            (false, Err(ErrorCode::CANCEL), 0, uart::Error::Aborted),
            (true, Ok(()), 0, uart::Error::Aborted),
            (true, Err(ErrorCode::CANCEL), 1, uart::Error::Aborted),
            (true, Err(ErrorCode::CANCEL), 0, uart::Error::None),
        ];

        for (preconditions_ok, rval, len, error) in cases {
            let mut result = SuiteResult::new();
            assert!(result.tx_success(Ok(()), TX_LEN));
            assert!(result.tx_abort(true, Err(ErrorCode::CANCEL), 0));
            assert!(result.rx_abort(preconditions_ok, rval, len, error));
            assert!(!result.rx_abort_ok);
        }
    }

    #[test]
    fn ignores_out_of_order_and_duplicate_callbacks() {
        let mut result = SuiteResult::new();
        assert!(!result.tx_abort(true, Err(ErrorCode::CANCEL), 0));
        assert!(!result.rx_abort(true, Err(ErrorCode::CANCEL), 0, uart::Error::Aborted));
        assert!(result.tx_success(Ok(()), TX_LEN));
        assert!(result.tx_abort(true, Err(ErrorCode::CANCEL), 0));
        assert!(result.rx_abort(true, Err(ErrorCode::CANCEL), 0, uart::Error::Aborted));
        assert!(!result.tx_success(Ok(()), TX_LEN));
        assert!(!result.tx_abort(true, Err(ErrorCode::CANCEL), 0));
        assert!(!result.rx_abort(true, Err(ErrorCode::CANCEL), 0, uart::Error::Aborted));
        assert_eq!(
            result.take_result_line(),
            Some("S32G3_UART_TEST result=PASS tx=PASS tx_abort=PASS rx_abort=PASS\r\n")
        );
        assert_eq!(result.take_result_line(), None);
    }

    #[test]
    fn maps_each_failure_combination_to_its_single_expected_line() {
        let cases = [
            (false, false, false, RESULT_FAIL_ALL),
            (false, false, true, RESULT_FAIL_TT),
            (false, true, false, RESULT_FAIL_TR),
            (false, true, true, RESULT_FAIL_T),
            (true, false, false, RESULT_FAIL_AT),
            (true, false, true, RESULT_FAIL_A),
            (true, true, false, RESULT_FAIL_R),
        ];

        for (tx_ok, tx_abort_ok, rx_abort_ok, expected) in cases {
            let mut result = SuiteResult::new();
            result.phase = Phase::Done;
            result.tx_ok = tx_ok;
            result.tx_abort_ok = tx_abort_ok;
            result.rx_abort_ok = rx_abort_ok;
            assert_eq!(result.take_result_line(), Some(expected));
        }
    }
}
