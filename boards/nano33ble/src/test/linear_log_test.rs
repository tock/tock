//! Tests the log storage interface in linear mode. For testing in circular mode, see
//! log_test.rs.
//!
//! The testing framework creates a non-circular log storage interface in flash and performs a
//! series of writes and syncs to ensure that the non-circular log properly denies overly-large
//! writes once it is full. For testing all of the general capabilities of the log storage
//! interface, see log_test.rs.
//!
//! To run the test, add the following line to the imix boot sequence:
//! ```
//!     test::linear_log_test::run(mux_alarm, dynamic_deferred_caller);
//! ```
//! and use the `USER` and `RESET` buttons to manually erase the log and reboot the imix,
//! respectively.

use capsules::log;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::cell::Cell;
use kernel::common::cells::{NumericCellExt, TakeCell};
use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
use kernel::debug_verbose;
use kernel::hil::flash;
use kernel::hil::log::{LogRead, LogReadClient, LogWrite, LogWriteClient};
use kernel::hil::time::{Alarm, AlarmClient};
use kernel::static_init;
use kernel::storage_volume;
use kernel::ReturnCode;
use nrf52840::{
    nvmc::{NrfPage, Nvmc},
    rtc::Rtc,
};

// Allocate 4 KiB volume for log storage (the nano33ble page size is 4 KiB).
storage_volume!(LINEAR_TEST_LOG, 4);

pub unsafe fn run(
    mux_alarm: &'static MuxAlarm<'static, Rtc>,
    deferred_caller: &'static DynamicDeferredCall,
    flash_controller: &'static Nvmc,
) {
    // Set up flash controller.
    flash_controller.configure_writeable();
    flash_controller.configure_eraseable();
    let pagebuffer = static_init!(NrfPage, NrfPage::default());

    // Create actual log storage abstraction on top of flash.
    let log = static_init!(
        Log,
        log::Log::new(
            &LINEAR_TEST_LOG,
            &flash_controller,
            pagebuffer,
            deferred_caller,
            false
        )
    );
    flash::HasClient::set_client(flash_controller, log);
    log.initialize_callback_handle(
        deferred_caller
            .register(log)
            .expect("no deferred call slot available for log storage"),
    );

    // Create and run test for log storage.
    let test = static_init!(
        LogTest<VirtualMuxAlarm<'static, Rtc>>,
        LogTest::new(log, &mut BUFFER, VirtualMuxAlarm::new(mux_alarm), &TEST_OPS)
    );
    log.set_read_client(test);
    log.set_append_client(test);
    test.alarm.set_alarm_client(test);

    test.run();
}

static TEST_OPS: [TestOp; 9] = [
    TestOp::Read,
    // Write to first page.
    TestOp::Write(8),
    TestOp::Write(300),
    // Write to next page, too large to fit on first.
    TestOp::Write(304),
    // Write should fail, not enough space remaining.
    TestOp::Write(306),
    // Write should succeed, enough space for a smaller entry.
    TestOp::Write(9),
    // Read back everything to verify and sync.
    TestOp::Read,
    TestOp::Sync,
    // Write should still fail after sync.
    TestOp::Write(308),
];

// Buffer for reading from and writing to in the log tests.
static mut BUFFER: [u8; 310] = [0; 310];
// Time to wait in between log operations.
const WAIT_MS: u32 = 3;

// A single operation within the test.
#[derive(Clone, Copy, PartialEq)]
enum TestOp {
    Read,
    Write(usize),
    Sync,
}

type Log = log::Log<'static, Nvmc>;

struct LogTest<A: Alarm<'static>> {
    log: &'static Log,
    buffer: TakeCell<'static, [u8]>,
    alarm: A,
    ops: &'static [TestOp],
    op_index: Cell<usize>,
}

impl<A: Alarm<'static>> LogTest<A> {
    fn new(
        log: &'static Log,
        buffer: &'static mut [u8],
        alarm: A,
        ops: &'static [TestOp],
    ) -> LogTest<A> {
        debug_verbose!(
            "Log recovered from flash (Start and end entry IDs: {:?} to {:?})",
            log.log_start(),
            log.log_end()
        );

        LogTest {
            log,
            buffer: TakeCell::new(buffer),
            alarm,
            ops,
            op_index: Cell::new(0),
        }
    }

    fn run(&self) {
        let op_index = self.op_index.get();
        if op_index == self.ops.len() {
            debug_verbose!("Linear Log Storage test succeeded!");
            return;
        }

        match self.ops[op_index] {
            TestOp::Read => self.read(),
            TestOp::Write(len) => self.write(len),
            TestOp::Sync => self.sync(),
        }
    }

    fn read(&self) {
        self.buffer.take().map_or_else(
            || panic!("NO BUFFER"),
            move |buffer| {
                // Clear buffer first to make debugging more sane.
                for e in buffer.iter_mut() {
                    *e = 0;
                }

                if let Err((error, original_buffer)) = self.log.read(buffer, buffer.len()) {
                    self.buffer
                        .replace(original_buffer.expect("No buffer returned in error!"));
                    match error {
                        ReturnCode::FAIL => {
                            // No more entries, start writing again.
                            debug_verbose!(
                                "READ DONE: READ OFFSET: {:?} / WRITE OFFSET: {:?}",
                                self.log.next_read_entry_id(),
                                self.log.log_end()
                            );
                            self.op_index.increment();
                            self.run();
                        }
                        ReturnCode::EBUSY => {
                            debug_verbose!("Flash busy, waiting before reattempting read");
                            self.wait();
                        }
                        _ => panic!("READ FAILED: {:?}", error),
                    }
                }
            },
        );
    }

    fn write(&self, len: usize) {
        self.buffer
            .take()
            .map(move |buffer| {
                let expect_write_fail = self.log.log_end() + len > LINEAR_TEST_LOG.len();

                // Set buffer value.
                for i in 0..buffer.len() {
                    buffer[i] = if i < len {
                        len as u8
                    } else {
                        0
                    };
                }

                if let Err((error, original_buffer)) = self.log.append(buffer, len) {
                    self.buffer.replace(original_buffer.expect("No buffer returned in error!"));

                    match error {
                        ReturnCode::FAIL =>
                            if expect_write_fail {
                                debug_verbose!(
                                    "Write failed on {} byte write, as expected",
                                    len
                                );
                                self.op_index.increment();
                                self.run();
                            } else {
                                panic!(
                                    "Write failed unexpectedly on {} byte write (read entry ID: {:?}, append entry ID: {:?})",
                                    len,
                                    self.log.next_read_entry_id(),
                                    self.log.log_end()
                                );
                            }
                        ReturnCode::EBUSY => self.wait(),
                        _ => panic!("WRITE FAILED: {:?}", error),
                    }
                } else if expect_write_fail {
                    panic!(
                        "Write succeeded unexpectedly on {} byte write (read entry ID: {:?}, append entry ID: {:?})",
                        len,
                        self.log.next_read_entry_id(),
                        self.log.log_end()
                    );
                }
            })
            .unwrap();
    }

    fn sync(&self) {
        match self.log.sync() {
            ReturnCode::SUCCESS => (),
            error => panic!("Sync failed: {:?}", error),
        }
    }

    fn wait(&self) {
        let delay = A::ticks_from_ms(WAIT_MS);
        let now = self.alarm.now();
        self.alarm.set_alarm(now, delay);
    }
}

impl<A: Alarm<'static>> LogReadClient for LogTest<A> {
    fn read_done(&self, buffer: &'static mut [u8], length: usize, error: ReturnCode) {
        match error {
            ReturnCode::SUCCESS => {
                // Verify correct value was read.
                assert!(length > 0);
                for i in 0..length {
                    if buffer[i] != length as u8 {
                        panic!(
                            "Read incorrect value {} at index {}, expected {}",
                            buffer[i], i, length
                        );
                    }
                }

                debug_verbose!("Successful read of size {}", length);
                self.buffer.replace(buffer);
                self.wait();
            }
            _ => {
                panic!("Read failed unexpectedly!");
            }
        }
    }

    fn seek_done(&self, _error: ReturnCode) {
        unreachable!();
    }
}

impl<A: Alarm<'static>> LogWriteClient for LogTest<A> {
    fn append_done(
        &self,
        buffer: &'static mut [u8],
        length: usize,
        records_lost: bool,
        error: ReturnCode,
    ) {
        assert!(!records_lost);
        match error {
            ReturnCode::SUCCESS => {
                debug_verbose!("Write succeeded on {} byte write, as expected", length);

                self.buffer.replace(buffer);
                self.op_index.increment();
                self.wait();
            }
            error => panic!("WRITE FAILED IN CALLBACK: {:?}", error),
        }
    }

    fn sync_done(&self, error: ReturnCode) {
        if error == ReturnCode::SUCCESS {
            debug_verbose!(
                "SYNC DONE: READ OFFSET: {:?} / WRITE OFFSET: {:?}",
                self.log.next_read_entry_id(),
                self.log.log_end()
            );
        } else {
            panic!("Sync failed: {:?}", error);
        }

        self.op_index.increment();
        self.run();
    }

    fn erase_done(&self, _error: ReturnCode) {
        unreachable!();
    }
}

impl<A: Alarm<'static>> AlarmClient for LogTest<A> {
    fn alarm(&self) {
        self.run();
    }
}
