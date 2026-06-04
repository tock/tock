// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Sequential UART test suite for LinFlexD on LF0.
//!
//! Runs four test phases in sequence, advancing via callbacks:
//!   Phase 1: RX buffer → wait for 4 bytes from human (press 4 keys on LF0)
//!   Phase 2: TX buffer → success callback
//!   Phase 3: TX buffer → immediate abort → CANCEL callback via deferred call
//!   Phase 4: RX buffer → immediate abort → CANCEL callback via deferred call
//!
//! To run, call from `lib.rs::start()`:
//! ```rust,ignore
//!     tests::uart_suite::run(uart_test);
//! ```
//!
//! Results are reported on the debug console (LF0).

use core::cell::Cell;
use core::ptr::addr_of_mut;

use kernel::debug;
use kernel::hil::uart::{self, Receive, Transmit};
use kernel::static_init;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

use nxp_s32g3::linflexd::LinFlexD;

// ---------------------------------------------------------------------------
// Test phases
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Loopback,
    RxSuccess,
    TxSuccess,
    TxAbort,
    RxAbort,
    Done,
}

// ---------------------------------------------------------------------------
// Test harness struct
// ---------------------------------------------------------------------------

struct UartTestSuite {
    uart: OptionalCell<&'static LinFlexD<'static>>,
    phase: Cell<Phase>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    passed: Cell<u32>,
    failed: Cell<u32>,
}

impl UartTestSuite {
    const fn new() -> Self {
        Self {
            uart: OptionalCell::empty(),
            phase: Cell::new(if USE_LOOPBACK {
                Phase::Loopback
            } else {
                Phase::RxSuccess
            }),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            passed: Cell::new(0),
            failed: Cell::new(0),
        }
    }

    fn pass(&self, msg: &str) {
        debug!("  [PASS] {}", msg);
        self.passed.set(self.passed.get() + 1);
    }

    fn fail(&self, msg: &str) {
        debug!("  [FAIL] {}", msg);
        self.failed.set(self.failed.get() + 1);
    }

    fn start_next_phase(&self) {
        match self.phase.get() {
            Phase::Loopback => self.run_loopback(),
            Phase::RxSuccess => self.run_rx_success(),
            Phase::TxSuccess => self.run_tx_success(),
            Phase::TxAbort => self.run_tx_abort(),
            Phase::RxAbort => self.run_rx_abort(),
            Phase::Done => self.report(),
        }
    }

    fn advance(&self) {
        let next = match self.phase.get() {
            Phase::Loopback => Phase::RxSuccess,
            Phase::RxSuccess => Phase::TxSuccess,
            Phase::TxSuccess => Phase::TxAbort,
            Phase::TxAbort => Phase::RxAbort,
            Phase::RxAbort => Phase::Done,
            Phase::Done => Phase::Done,
        };
        self.phase.set(next);
        self.start_next_phase();
    }

    fn report(&self) {
        debug!("========================================");
        debug!(
            "[UART SUITE] {} passed, {} failed",
            self.passed.get(),
            self.failed.get()
        );
        if self.failed.get() == 0 {
            debug!("[UART SUITE] ALL TESTS PASSED");
        } else {
            debug!("[UART SUITE] SOME TESTS FAILED");
        }
        debug!("========================================");
    }

    // ------ Phase Loopback: TX→RX loopback (self-checked) ------

    fn run_loopback(&self) {
        debug!("[Phase Loopback] TX→RX loopback...");
        self.uart.map(|uart| {
            self.tx_buf.take().map(|buf| {
                let pattern = b"ABCD";
                for (i, b) in buf.iter_mut().enumerate() {
                    *b = pattern[i % pattern.len()];
                }
                match uart.transmit_buffer(buf, RX_LEN) {
                    Ok(()) => {} // Wait for TX callback to trigger RX.
                    Err((code, buf)) => {
                        self.fail("loopback transmit_buffer returned error");
                        debug!("         error: {:?}", code);
                        self.tx_buf.replace(buf);
                        self.advance();
                    }
                }
            });
        });
    }

    // ------ Phase 1: RX success (human interaction) ------

    fn run_rx_success(&self) {
        debug!(
            "[Phase 1] RX buffer success — press {} keys on LF0...",
            RX_LEN
        );
        self.uart.map(|uart| {
            self.rx_buf.take().map(|buf| {
                match uart.receive_buffer(buf, RX_LEN) {
                    Ok(()) => {} // Wait for callback.
                    Err((code, buf)) => {
                        self.fail("receive_buffer returned error");
                        debug!("         error: {:?}", code);
                        self.rx_buf.replace(buf);
                        self.advance();
                    }
                }
            });
        });
    }

    // ------ Phase 2: TX success ------

    fn run_tx_success(&self) {
        debug!("[Phase 2] TX buffer success...");
        self.uart.map(|uart| {
            self.tx_buf.take().map(|buf| {
                for (i, b) in buf.iter_mut().enumerate() {
                    *b = b'A' + (i as u8 % 26);
                }
                match uart.transmit_buffer(buf, TX_LEN) {
                    Ok(()) => {}
                    Err((code, buf)) => {
                        self.fail("transmit_buffer returned error");
                        debug!("         error: {:?}", code);
                        self.tx_buf.replace(buf);
                        self.advance();
                    }
                }
            });
        });
    }

    // ------ Phase 3: TX abort ------

    fn run_tx_abort(&self) {
        debug!("[Phase 3] TX buffer abort...");
        self.uart.map(|uart| {
            self.tx_buf.take().map(|buf| {
                for b in buf.iter_mut() {
                    *b = 0xAA;
                }
                match uart.transmit_buffer(buf, TX_LEN) {
                    Ok(()) => {
                        let r = uart.transmit_abort();
                        if r != Err(ErrorCode::BUSY) {
                            self.fail("transmit_abort did not return Err(BUSY)");
                            debug!("         got: {:?}", r);
                        }
                    }
                    Err((code, buf)) => {
                        self.fail("transmit_buffer returned error");
                        debug!("         error: {:?}", code);
                        self.tx_buf.replace(buf);
                        self.advance();
                    }
                }
            });
        });
    }

    // ------ Phase 4: RX abort ------

    fn run_rx_abort(&self) {
        debug!("[Phase 4] RX buffer abort...");
        self.uart.map(|uart| {
            self.rx_buf
                .take()
                .map(|buf| match uart.receive_buffer(buf, RX_LEN) {
                    Ok(()) => {
                        let r = uart.receive_abort();
                        if r != Err(ErrorCode::BUSY) {
                            self.fail("receive_abort did not return Err(BUSY)");
                            debug!("         got: {:?}", r);
                        }
                    }
                    Err((code, buf)) => {
                        self.fail("receive_buffer returned error");
                        debug!("         error: {:?}", code);
                        self.rx_buf.replace(buf);
                        self.advance();
                    }
                });
        });
    }
}

// ---------------------------------------------------------------------------
// TransmitClient — handles TX completions
// ---------------------------------------------------------------------------

impl uart::TransmitClient for UartTestSuite {
    fn transmitted_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        rval: Result<(), ErrorCode>,
    ) {
        self.tx_buf.replace(tx_buffer);
        match self.phase.get() {
            Phase::Loopback => {
                if rval == Ok(()) && tx_len == RX_LEN {
                    self.pass("loopback TX completed, starting RX");
                    self.uart.map(|uart| {
                        self.rx_buf
                            .take()
                            .map(|buf| match uart.receive_buffer(buf, RX_LEN) {
                                Ok(()) => {}
                                Err((code, buf)) => {
                                    self.fail("loopback receive_buffer returned error");
                                    debug!("         error: {:?}", code);
                                    self.rx_buf.replace(buf);
                                    self.advance();
                                }
                            });
                    });
                } else {
                    self.fail("loopback TX unexpected result");
                    debug!("         rval={:?} len={}", rval, tx_len);
                    self.advance();
                }
            }
            Phase::TxSuccess => {
                if rval == Ok(()) && tx_len == TX_LEN {
                    self.pass("TX buffer completed successfully");
                } else {
                    self.fail("TX buffer unexpected result");
                    debug!("         rval={:?} len={}", rval, tx_len);
                }
                self.advance();
            }
            Phase::TxAbort => {
                if rval == Err(ErrorCode::CANCEL) {
                    self.pass("TX abort delivered Err(CANCEL) via deferred call");
                } else if rval == Ok(()) {
                    self.pass("TX completed before abort (race OK)");
                } else {
                    self.fail("TX abort unexpected result");
                    debug!("         rval={:?}", rval);
                }
                self.advance();
            }
            _ => {}
        }
    }

    fn transmitted_word(&self, _rval: Result<(), ErrorCode>) {}
}

// ---------------------------------------------------------------------------
// ReceiveClient — handles RX completions
// ---------------------------------------------------------------------------

impl uart::ReceiveClient for UartTestSuite {
    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rval: Result<(), ErrorCode>,
        error: uart::Error,
    ) {
        self.rx_buf.replace(rx_buffer);
        match self.phase.get() {
            Phase::Loopback => {
                if rval == Ok(()) && rx_len == RX_LEN && error == uart::Error::None {
                    let expected = b"ABCD";
                    self.rx_buf.map(|buf| {
                        if &buf[..RX_LEN] == expected {
                            self.pass("loopback RX matches expected pattern");
                        } else {
                            self.failed.set(self.failed.get() + 1);
                            debug!(
                                "  [FAIL] RX mismatch: expected [{:#04x}, {:#04x}, {:#04x}, {:#04x}] got [{:#04x}, {:#04x}, {:#04x}, {:#04x}]",
                                expected[0], expected[1], expected[2], expected[3],
                                buf[0], buf[1], buf[2], buf[3]
                            );
                        }
                    });
                } else {
                    self.fail("loopback RX unexpected result");
                    debug!("         rval={:?} len={} error={:?}", rval, rx_len, error);
                }
                self.advance();
            }
            Phase::RxSuccess => {
                if rval == Ok(()) && rx_len == RX_LEN && error == uart::Error::None {
                    self.pass("RX buffer received successfully");
                    self.rx_buf.map(|buf| {
                        for (i, &b) in buf[..RX_LEN].iter().enumerate() {
                            if b < 0x20 || b > 0x7E {
                                debug!("[WARN] non-printable byte {:#04x} at offset {}", b, i);
                            }
                        }
                        debug!(
                            "         received: [{:#04x}, {:#04x}, {:#04x}, {:#04x}]",
                            buf[0], buf[1], buf[2], buf[3]
                        );
                    });
                } else {
                    self.fail("RX buffer unexpected result");
                    debug!("         rval={:?} len={} error={:?}", rval, rx_len, error);
                }
                self.advance();
            }
            Phase::RxAbort => {
                if rval == Err(ErrorCode::CANCEL) && error == uart::Error::Aborted {
                    self.pass("RX abort delivered Err(CANCEL) + Aborted via deferred call");
                } else {
                    self.fail("RX abort unexpected result");
                    debug!("         rval={:?} error={:?}", rval, error);
                }
                self.advance();
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TX_LEN: usize = 16;
const RX_LEN: usize = 4;
const USE_LOOPBACK: bool = false;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run the sequential UART test suite on the given LinFlexD instance.
///
/// # Safety
///
/// Must only be called once from `start()` after the UART is configured.
pub unsafe fn run(lf1: &'static LinFlexD<'static>) {
    static mut TX_BUF: [u8; TX_LEN] = [0; TX_LEN];
    static mut RX_BUF: [u8; RX_LEN] = [0; RX_LEN];

    let suite = static_init!(UartTestSuite, UartTestSuite::new());
    suite.uart.set(lf1);
    suite.tx_buf.replace(&mut *addr_of_mut!(TX_BUF));
    suite.rx_buf.replace(&mut *addr_of_mut!(RX_BUF));

    lf1.set_transmit_client(suite);
    lf1.set_receive_client(suite);

    debug!("========================================");
    debug!("[UART SUITE] Starting LinFlexD test suite on LF0 @ 115200");
    debug!("[UART SUITE] Phase 1 requires human input: press 4 keys on LF0");
    debug!("========================================");

    suite.start_next_phase();
}
