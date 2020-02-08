//! Tests the log storage interface in linear mode. For testing in circular mode, see
//! log_storage_test.rs.
//!
//! The testing framework creates a non-circular log storage interface in flash and performs a
//! series of writes and syncs to ensure that the non-circular log properly denies overly-large
//! writes once it is full. For testing all of the general capabilities of the log storage
//! interface, see storage_test.rs.
//!
//! To run the test, add the following line to the imix boot sequence:
//! ```
//!     linear_storage_test::run_log_storage_linear(mux_alarm, dynamic_deferred_caller);
//! ```
//! and use the `USER` and `RESET` buttons to manually erase the log and reboot the imix,
//! respectively.

use capsules::log_storage;
use capsules::storage_interface::{
    self, LogRead, LogReadClient, LogWrite, LogWriteClient, StorageCookie, StorageLen,
};
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::cell::Cell;
use kernel::common::cells::{NumericCellExt, TakeCell};
use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
use kernel::debug;
use kernel::hil::flash;
use kernel::hil::time::{Alarm, AlarmClient, Frequency};
use kernel::static_init;
use kernel::storage_volume;
use kernel::ReturnCode;
use sam4l::ast::Ast;
use sam4l::flashcalw;

// Allocate 1kiB volume for log storage.
storage_volume!(LINEAR_TEST_LOG, 1);

pub unsafe fn run_log_storage_linear(
    mux_alarm: &'static MuxAlarm<'static, Ast>,
    deferred_caller: &'static DynamicDeferredCall,
) {
    // Set up flash controller.
    flashcalw::FLASH_CONTROLLER.configure();
    static mut PAGEBUFFER: flashcalw::Sam4lPage = flashcalw::Sam4lPage::new();

    // Create actual log storage abstraction on top of flash.
    let log_storage = static_init!(
        LogStorage,
        log_storage::LogStorage::new(
            &LINEAR_TEST_LOG,
            &mut flashcalw::FLASH_CONTROLLER,
            &mut PAGEBUFFER,
            deferred_caller,
            false
        )
    );
    flash::HasClient::set_client(&flashcalw::FLASH_CONTROLLER, log_storage);
    log_storage.initialize_callback_handle(
        deferred_caller
            .register(log_storage)
            .expect("no deferred call slot available for log storage"),
    );

    // Create and run test for log storage.
    let log_storage_test = static_init!(
        LogStorageTest<VirtualMuxAlarm<'static, Ast>>,
        LogStorageTest::new(
            log_storage,
            &mut BUFFER,
            VirtualMuxAlarm::new(mux_alarm),
            &TEST_OPS
        )
    );
    storage_interface::HasClient::set_client(log_storage, log_storage_test);
    log_storage_test.alarm.set_client(log_storage_test);

    log_storage_test.run();
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

// Buffer for reading from and writing to in the storage tests.
static mut BUFFER: [u8; 310] = [0; 310];
// Time to wait in between storage operations.
const WAIT_MS: u32 = 3;

// A single operation within the test.
#[derive(Clone, Copy, PartialEq)]
enum TestOp {
    Read,
    Write(usize),
    Sync,
}

type LogStorage = log_storage::LogStorage<
    'static,
    flashcalw::FLASHCALW,
    LogStorageTest<VirtualMuxAlarm<'static, Ast<'static>>>,
>;
struct LogStorageTest<A: Alarm<'static>> {
    storage: &'static LogStorage,
    buffer: TakeCell<'static, [u8]>,
    alarm: A,
    ops: &'static [TestOp],
    op_index: Cell<usize>,
}

impl<A: Alarm<'static>> LogStorageTest<A> {
    fn new(
        storage: &'static LogStorage,
        buffer: &'static mut [u8],
        alarm: A,
        ops: &'static [TestOp],
    ) -> LogStorageTest<A> {
        debug!(
            "Log recovered from flash (Start and end cookies: {:?} to {:?})",
            storage.current_read_offset(),
            storage.current_append_offset()
        );

        LogStorageTest {
            storage,
            buffer: TakeCell::new(buffer),
            alarm,
            ops,
            op_index: Cell::new(0),
        }
    }

    fn run(&self) {
        let op_index = self.op_index.get();
        if op_index == self.ops.len() {
            debug!("Linear Log Storage test succeeded!");
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

                if let Err((error, original_buffer)) = self.storage.read(buffer, buffer.len()) {
                    self.buffer.replace(original_buffer.expect("No buffer returned in error!"));
                    match error {
                        ReturnCode::FAIL => {
                            // No more entries, start writing again.
                            debug!(
                                "READ DONE: READ OFFSET: {:?} / WRITE OFFSET: {:?}",
                                self.storage.current_read_offset(),
                                self.storage.current_append_offset()
                            );
                            self.op_index.increment();
                            self.run();
                        }
                        ReturnCode::EBUSY => {
                            debug!("Flash busy, waiting before reattempting read");
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
                let expect_write_fail = match self.storage.current_append_offset() {
                    StorageCookie::Cookie(cookie) => cookie + len > LINEAR_TEST_LOG.len(),
                    _ => false
                };

                // Set buffer value.
                for i in 0..buffer.len() {
                    buffer[i] = if i < len {
                        len as u8
                    } else {
                        0
                    };
                }

                if let Err((error, original_buffer)) = self.storage.append(buffer, len) {
                    self.buffer.replace(original_buffer.expect("No buffer returned in error!"));

                    match error {
                        ReturnCode::FAIL =>
                            if expect_write_fail {
                                debug!(
                                    "Write failed on {} byte write, as expected",
                                    len
                                );
                                self.op_index.increment();
                                self.run();
                            } else {
                                panic!(
                                    "Write failed unexpectedly on {} byte write (read cookie: {:?}, append cookie: {:?})",
                                    len,
                                    self.storage.current_read_offset(),
                                    self.storage.current_append_offset()
                                );
                            }
                        ReturnCode::EBUSY => self.wait(),
                        _ => panic!("WRITE FAILED: {:?}", error),
                    }
                } else if expect_write_fail {
                    panic!(
                        "Write succeeded unexpectedly on {} byte write (read cookie: {:?}, append cookie: {:?})",
                        len,
                        self.storage.current_read_offset(),
                        self.storage.current_append_offset()
                    );
                }
            })
            .unwrap();
    }

    fn sync(&self) {
        match self.storage.sync() {
            ReturnCode::SUCCESS => (),
            error => panic!("Sync failed: {:?}", error),
        }
    }

    fn wait(&self) {
        let interval = WAIT_MS * <A::Frequency>::frequency() / 1000;
        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);
    }
}

impl<A: Alarm<'static>> LogReadClient for LogStorageTest<A> {
    fn read_done(&self, buffer: &'static mut [u8], length: StorageLen, error: ReturnCode) {
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

                debug!("Successful read of size {}", length);
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

impl<A: Alarm<'static>> LogWriteClient for LogStorageTest<A> {
    fn append_done(
        &self,
        buffer: &'static mut [u8],
        length: StorageLen,
        records_lost: bool,
        error: ReturnCode,
    ) {
        assert!(!records_lost);
        match error {
            ReturnCode::SUCCESS => {
                debug!("Write succeeded on {} byte write, as expected", length);

                self.buffer.replace(buffer);
                self.op_index.increment();
                self.wait();
            }
            error => panic!("WRITE FAILED IN CALLBACK: {:?}", error),
        }
    }

    fn sync_done(&self, error: ReturnCode) {
        if error == ReturnCode::SUCCESS {
            debug!(
                "SYNC DONE: READ OFFSET: {:?} / WRITE OFFSET: {:?}",
                self.storage.current_read_offset(),
                self.storage.current_append_offset()
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

impl<A: Alarm<'static>> AlarmClient for LogStorageTest<A> {
    fn fired(&self) {
        self.run();
    }
}
