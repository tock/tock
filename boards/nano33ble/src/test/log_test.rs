//! Tests the log storage interface in circular mode. For testing in linear mode, see
//! linear_log_test.rs.
//!
//! This testing framework creates a circular log storage interface in flash and runs a series of
//! operations upon it. The tests check to make sure that the correct values are read and written
//! after each operation, that errors are properly detected and handled, and that the log generally
//! behaves as expected. The tests perform both valid and invalid operations to fully test the log's
//! behavior.
//!
//! Pressing the `USER` button on the imix at any time during the test will erase the log and reset
//! the test state. Pressing the `RESET` button will reboot the imix without erasing the log,
//! allowing for testing logs across reboots.
//!
//! In order to fully test the log, the tester should try a variety of erases and reboots to ensure
//! that the log works correctly across these operations. The tester can also modify the testing
//! operations and parameters defined below to test logs in different configurations. Different
//! configurations should be tested in order to exercise the log under a greater number of
//! scenarios (e.g. saturating/not saturating log pages with data, always/not always ending
//! operations at page boundaries, etc.).
//!
//! To run the test, add the following line to the imix boot sequence:
//! ```
//!     test::log_test::run(mux_alarm, dynamic_deferred_caller, &peripherals.flash_controller);
//! ```
//! and use the `USER` and `RESET` buttons to manually erase the log and reboot the imix,
//! respectively.

use capsules::log;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::cell::Cell;
use kernel::common::cells::{NumericCellExt, TakeCell};
use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
use kernel::debug;
use kernel::hil::flash;
use kernel::hil::gpio::{self};
use kernel::hil::log::{LogRead, LogReadClient, LogWrite, LogWriteClient};
use kernel::hil::time::{Alarm, AlarmClient};
use kernel::static_init;
use kernel::storage_volume;
use kernel::ReturnCode;
use nrf52840::{nvmc, rtc::Rtc};

// Allocate 2kiB volume for log storage.
storage_volume!(TEST_LOG, 2);

pub unsafe fn run(
    mux_alarm: &'static MuxAlarm<'static, Rtc>,
    deferred_caller: &'static DynamicDeferredCall,
    flash_controller: &'static nvmc::Nvmc,
) {
    // Set up flash controller.
    flash_controller.configure_writeable();
    flash_controller.configure_eraseable();
    let pagebuffer = static_init!(nvmc::NrfPage, nvmc::NrfPage::default());

    // Create actual log storage abstraction on top of flash.
    let log = static_init!(
        Log,
        log::Log::new(
            &TEST_LOG,
            &flash_controller,
            pagebuffer,
            deferred_caller,
            true
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

static TEST_OPS: [TestOp; 24] = [
    // Read back any existing entries.
    TestOp::BadRead,
    TestOp::Read,
    // Write multiple pages, but don't fill log.
    TestOp::BadWrite,
    TestOp::Write,
    TestOp::Read,
    TestOp::BadWrite,
    TestOp::Write,
    TestOp::Read,
    // Seek to beginning and re-verify entire log.
    TestOp::SeekBeginning,
    TestOp::Read,
    // Write multiple pages, over-filling log and overwriting oldest entries.
    TestOp::SeekBeginning,
    TestOp::Write,
    // Read offset should be incremented since it was invalidated by previous write.
    TestOp::BadRead,
    TestOp::Read,
    // Write multiple pages and sync. Read offset should be invalidated due to sync clobbering
    // previous read offset.
    TestOp::Write,
    TestOp::Sync,
    TestOp::Read,
    // Try bad seeks, should fail and not change read entry ID.
    TestOp::Write,
    TestOp::BadSeek(0),
    TestOp::BadSeek(core::usize::MAX),
    TestOp::Read,
    // Try bad write, nothing should change.
    TestOp::BadWrite,
    TestOp::Read,
    // Sync log before finishing test so that all changes persist for next test iteration.
    TestOp::Sync,
];

// Buffer for reading from and writing to in the log tests.
static mut BUFFER: [u8; 8] = [0; 8];
// Length of buffer to actually use.
const BUFFER_LEN: usize = 8;
// Amount to shift value before adding to magic in order to fit in buffer.
const VALUE_SHIFT: usize = 8 * (8 - BUFFER_LEN);
// Dummy buffer for testing bad writes.
static mut DUMMY_BUFFER: [u8; 520] = [0; 520];
// Time to wait in between log operations.
const WAIT_MS: u32 = 2;
// Magic number to write to log storage (+ offset).
const MAGIC: u64 = 0x0102030405060708;
// Number of entries to write per write operation.
const ENTRIES_PER_WRITE: u64 = 120;

// Test's current state.
#[derive(Clone, Copy, PartialEq)]
enum TestState {
    Operate, // Running through test operations.
    Erase,   // Erasing log and restarting test.
    CleanUp, // Cleaning up test after all operations complete.
}

// A single operation within the test.
#[derive(Clone, Copy, PartialEq)]
enum TestOp {
    Read,
    BadRead,
    Write,
    BadWrite,
    Sync,
    SeekBeginning,
    BadSeek(usize),
}

type Log = log::Log<'static, nvmc::Nvmc>;
struct LogTest<A: Alarm<'static>> {
    log: &'static Log,
    buffer: TakeCell<'static, [u8]>,
    alarm: A,
    state: Cell<TestState>,
    ops: &'static [TestOp],
    op_index: Cell<usize>,
    op_start: Cell<bool>,
    read_val: Cell<u64>,
    write_val: Cell<u64>,
}

impl<A: Alarm<'static>> LogTest<A> {
    fn new(
        log: &'static Log,
        buffer: &'static mut [u8],
        alarm: A,
        ops: &'static [TestOp],
    ) -> LogTest<A> {
        // Recover test state.
        let read_val = entry_id_to_test_value(log.next_read_entry_id());
        let write_val = entry_id_to_test_value(log.log_end());

        debug!(
            "Log recovered from flash (Start and end entry IDs: {:?} to {:?}; read and write values: {} and {})",
            log.next_read_entry_id(),
            log.log_end(),
            read_val,
            write_val
        );

        LogTest {
            log,
            buffer: TakeCell::new(buffer),
            alarm,
            state: Cell::new(TestState::Operate),
            ops,
            op_index: Cell::new(0),
            op_start: Cell::new(true),
            read_val: Cell::new(read_val),
            write_val: Cell::new(write_val),
        }
    }

    fn run(&self) {
        match self.state.get() {
            TestState::Operate => {
                let op_index = self.op_index.get();
                if op_index == self.ops.len() {
                    self.state.set(TestState::CleanUp);
                    self.log.seek(self.log.log_start());
                    return;
                }

                match self.ops[op_index] {
                    TestOp::Read => self.read(),
                    TestOp::BadRead => self.bad_read(),
                    TestOp::Write => self.write(),
                    TestOp::BadWrite => self.bad_write(),
                    TestOp::Sync => self.sync(),
                    TestOp::SeekBeginning => self.seek_beginning(),
                    TestOp::BadSeek(entry_id) => self.bad_seek(entry_id),
                }
            }
            TestState::Erase => self.erase(),
            TestState::CleanUp => {
                debug!(
                    "Log Storage test succeeded! (Final log start and end entry IDs: {:?} to {:?})",
                    self.log.next_read_entry_id(),
                    self.log.log_end()
                );
            }
        }
    }

    fn next_op(&self) {
        self.op_index.increment();
        self.op_start.set(true);
    }

    fn erase(&self) {
        match self.log.erase() {
            ReturnCode::SUCCESS => (),
            ReturnCode::EBUSY => {
                self.wait();
            }
            _ => panic!("Could not erase log storage!"),
        }
    }

    fn read(&self) {
        // Update read value if clobbered by previous operation.
        if self.op_start.get() {
            let next_read_val = entry_id_to_test_value(self.log.next_read_entry_id());
            if self.read_val.get() < next_read_val {
                debug!(
                    "Increasing read value from {} to {} due to clobbering (read entry ID is {:?})!",
                    self.read_val.get(),
                    next_read_val,
                    self.log.next_read_entry_id()
                );
                self.read_val.set(next_read_val);
            }
        }

        self.buffer.take().map_or_else(
            || panic!("NO BUFFER"),
            move |buffer| {
                // Clear buffer first to make debugging more sane.
                buffer.clone_from_slice(&0u64.to_be_bytes());

                if let Err((error, _original_buffer)) = self.log.read(buffer, BUFFER_LEN) {
                    match error {
                        ReturnCode::FAIL => {
                            // No more entries, start writing again.
                            debug!(
                                "READ DONE: READ OFFSET: {:?} / WRITE OFFSET: {:?}",
                                self.log.next_read_entry_id(),
                                self.log.log_end()
                            );
                            self.next_op();
                            self.run();
                        }
                        ReturnCode::EBUSY => {
                            debug!("Flash busy, waiting before reattempting read");
                            self.wait();
                        }
                        _ => panic!("READ #{} FAILED: {:?}", self.read_val.get(), error),
                    }
                }
            },
        );
    }

    fn bad_read(&self) {
        // Ensure failure if buffer is smaller than provided max read length.
        self.buffer
            .take()
            .map(
                move |buffer| match self.log.read(buffer, buffer.len() + 1) {
                    Ok(_) => panic!("Read with too-large max read length succeeded unexpectedly!"),
                    Err((error, _original_buffer)) => {
                        assert_eq!(error, ReturnCode::EINVAL);
                    }
                },
            )
            .unwrap();

        // Ensure failure if buffer is too small to hold entry.
        self.buffer
            .take()
            .map(move |buffer| match self.log.read(buffer, BUFFER_LEN - 1) {
                Ok(_) => panic!("Read with too-small buffer succeeded unexpectedly!"),
                Err((error, _original_buffer)) => {
                    if self.read_val.get() == self.write_val.get() {
                        assert_eq!(error, ReturnCode::FAIL);
                    } else {
                        assert_eq!(error, ReturnCode::ESIZE);
                    }
                }
            })
            .unwrap();

        self.next_op();
        self.run();
    }

    fn write(&self) {
        self.buffer
            .take()
            .map(move |buffer| {
                buffer.clone_from_slice(
                    &(MAGIC + (self.write_val.get() << VALUE_SHIFT)).to_be_bytes(),
                );
                if let Err((error, _original_buffer)) = self.log.append(buffer, BUFFER_LEN) {
                    match error {
                        ReturnCode::EBUSY => self.wait(),
                        _ => panic!("WRITE FAILED: {:?}", error),
                    }
                }
            })
            .unwrap();
    }

    fn bad_write(&self) {
        let original_offset = self.log.log_end();

        // Ensure failure if entry length is 0.
        self.buffer
            .take()
            .map(move |buffer| match self.log.append(buffer, 0) {
                Ok(_) => panic!("Appending entry of size 0 succeeded unexpectedly!"),
                Err((error, _original_buffer)) => {
                    assert_eq!(error, ReturnCode::EINVAL);
                }
            })
            .unwrap();

        // Ensure failure if proposed entry length is greater than buffer length.
        self.buffer
            .take()
            .map(
                move |buffer| match self.log.append(buffer, buffer.len() + 1) {
                    Ok(_) => panic!("Appending with too-small buffer succeeded unexpectedly!"),
                    Err((error, _original_buffer)) => {
                        assert_eq!(error, ReturnCode::EINVAL);
                    }
                },
            )
            .unwrap();

        // Ensure failure if entry is too large to fit within a single flash page.
        unsafe {
            match self.log.append(&mut DUMMY_BUFFER, DUMMY_BUFFER.len()) {
                Ok(_) => panic!("Appending with too-small buffer succeeded unexpectedly!"),
                Err((error, _original_buffer)) => assert_eq!(error, ReturnCode::ESIZE),
            }
        }

        // Make sure that append offset was not changed by failed writes.
        assert_eq!(original_offset, self.log.log_end());
        self.next_op();
        self.run();
    }

    fn sync(&self) {
        match self.log.sync() {
            ReturnCode::SUCCESS => (),
            error => panic!("Sync failed: {:?}", error),
        }
    }

    fn seek_beginning(&self) {
        let entry_id = self.log.log_start();
        match self.log.seek(entry_id) {
            ReturnCode::SUCCESS => debug!("Seeking to {:?}...", entry_id),
            error => panic!("Seek failed: {:?}", error),
        }
    }

    fn bad_seek(&self, entry_id: usize) {
        // Make sure seek fails with EINVAL.
        let original_offset = self.log.next_read_entry_id();
        match self.log.seek(entry_id) {
            ReturnCode::EINVAL => (),
            ReturnCode::SUCCESS => panic!(
                "Seek to invalid entry ID {:?} succeeded unexpectedly!",
                entry_id
            ),
            error => panic!(
                "Seek to invalid entry ID {:?} failed with unexpected error {:?}!",
                entry_id, error
            ),
        }

        // Make sure that read offset was not changed by failed seek.
        assert_eq!(original_offset, self.log.next_read_entry_id());
        self.next_op();
        self.run();
    }

    fn wait(&self) {
        let delay = A::ticks_from_ms(WAIT_MS);
        let now = self.alarm.now();
        self.alarm.set_alarm(now, delay);
    }
}

impl<A: Alarm<'static>> LogReadClient for LogTest<A> {
    fn read_done(&self, buffer: &'static mut [u8], result: Result<usize, ReturnCode>) {
        match result {
            Ok(length) => {
                // Verify correct number of bytes were read.
                if length != BUFFER_LEN {
                    panic!(
                        "{} bytes read, expected {} on read number {} (offset {:?}). Value read was {:?}",
                        length,
                        BUFFER_LEN,
                        self.read_val.get(),
                        self.log.next_read_entry_id(),
                        &buffer[0..length],
                    );
                }

                // Verify correct value was read.
                let expected = (MAGIC + (self.read_val.get() << VALUE_SHIFT)).to_be_bytes();
                for i in 0..BUFFER_LEN {
                    if buffer[i] != expected[i] {
                        panic!(
                            "Expected {:?}, read {:?} on read number {} (offset {:?})",
                            &expected[0..BUFFER_LEN],
                            &buffer[0..BUFFER_LEN],
                            self.read_val.get(),
                            self.log.next_read_entry_id(),
                        );
                    }
                }

                self.buffer.replace(buffer);
                self.read_val.set(self.read_val.get() + 1);
                self.op_start.set(false);
                self.wait();
            }
            Err(error_code) => panic!("Read failed with error code {:?}", error_code),
        }
    }

    fn seek_done(&self, error: ReturnCode) {
        if error == ReturnCode::SUCCESS {
            debug!("Seeked");
            self.read_val
                .set(entry_id_to_test_value(self.log.next_read_entry_id()));
        } else {
            panic!("Seek failed: {:?}", error);
        }

        if self.state.get() == TestState::Operate {
            self.next_op();
        }
        self.run();
    }
}

impl<A: Alarm<'static>> LogWriteClient for LogTest<A> {
    fn append_done(&self, buffer: &'static mut [u8], result: Result<(usize, bool), ReturnCode>) {
        match result {
            Ok((length, records_lost)) => {
                self.buffer.replace(buffer);
                self.op_start.set(false);
                if length != BUFFER_LEN {
                    panic!(
                        "Appended {} bytes, expected {} (write #{}, offset {:?})!",
                        length,
                        BUFFER_LEN,
                        self.write_val.get(),
                        self.log.log_end()
                    );
                }
                let expected_records_lost =
                    self.write_val.get() > entry_id_to_test_value(TEST_LOG.len());
                if records_lost && records_lost != expected_records_lost {
                    panic!("Append callback states records_lost = {}, expected {} (write #{}, offset {:?})!",
                           records_lost,
                           expected_records_lost,
                           self.write_val.get(),
                           self.log.log_end()
                    );
                }

                // Stop writing after `ENTRIES_PER_WRITE` entries have been written.
                if (self.write_val.get() + 1) % ENTRIES_PER_WRITE == 0 {
                    debug!(
                        "WRITE DONE: READ OFFSET: {:?} / WRITE OFFSET: {:?}",
                        self.log.next_read_entry_id(),
                        self.log.log_end()
                    );
                    self.next_op();
                }
                self.write_val.set(self.write_val.get() + 1);
                self.wait();
            }
            Err(error_code) => panic!("Write failed with error code {:?}", error_code),
        }
    }

    fn sync_done(&self, error: ReturnCode) {
        if error == ReturnCode::SUCCESS {
            debug!(
                "SYNC DONE: READ OFFSET: {:?} / WRITE OFFSET: {:?}",
                self.log.next_read_entry_id(),
                self.log.log_end()
            );
        } else {
            panic!("Sync failed: {:?}", error);
        }

        self.next_op();
        self.run();
    }

    fn erase_done(&self, error: ReturnCode) {
        match error {
            ReturnCode::SUCCESS => {
                // Reset test state.
                self.op_index.set(0);
                self.op_start.set(true);
                self.read_val.set(0);
                self.write_val.set(0);

                // Make sure that flash has been erased.
                for i in 0..TEST_LOG.len() {
                    if TEST_LOG[i] != 0xFF {
                        panic!(
                            "Log not properly erased, read {} at byte {}. SUMMARY: {:?}",
                            TEST_LOG[i],
                            i,
                            &TEST_LOG[i..i + 8]
                        );
                    }
                }

                // Make sure that a read on an empty log fails normally.
                self.buffer.take().map(move |buffer| {
                    if let Err((error, _original_buffer)) = self.log.read(buffer, BUFFER_LEN) {
                        match error {
                            ReturnCode::FAIL => (),
                            ReturnCode::EBUSY => {
                                self.wait();
                            }
                            _ => panic!("Read on empty log did not fail as expected: {:?}", error),
                        }
                    } else {
                        panic!("Read on empty log succeeded! (it shouldn't)");
                    }
                });

                // Move to next operation.
                debug!("Log Storage erased");
                self.state.set(TestState::Operate);
                self.run();
            }
            ReturnCode::EBUSY => {
                // Flash busy, try again.
                self.wait();
            }
            _ => {
                panic!("Erase failed: {:?}", error);
            }
        }
    }
}

impl<A: Alarm<'static>> AlarmClient for LogTest<A> {
    fn alarm(&self) {
        self.run();
    }
}

impl<A: Alarm<'static>> gpio::Client for LogTest<A> {
    fn fired(&self) {
        // Erase log.
        self.state.set(TestState::Erase);
        self.erase();
    }
}

fn entry_id_to_test_value(entry_id: usize) -> u64 {
    // Page and entry header sizes for log storage.
    const PAGE_SIZE: usize = 512;

    let pages_written = entry_id / PAGE_SIZE;
    let entry_size = log::ENTRY_HEADER_SIZE + BUFFER_LEN;
    let entries_per_page = (PAGE_SIZE - log::PAGE_HEADER_SIZE) / entry_size;
    let entries_last_page = if entry_id % PAGE_SIZE >= log::PAGE_HEADER_SIZE {
        (entry_id % PAGE_SIZE - log::PAGE_HEADER_SIZE) / entry_size
    } else {
        0
    };
    (pages_written * entries_per_page + entries_last_page) as u64
}
