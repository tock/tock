//! Provides low-level debugging functionality to userspace. The system call
//! interface is documented in doc/syscalls/00008_low_level_debug.md.

mod fmt;

use core::cell::Cell;
use kernel::hil::uart::{Transmit, TransmitClient};
use kernel::{AppId, Grant, ReturnCode};

// LowLevelDebug requires a &mut [u8] buffer of length at least BUF_LEN.
pub use fmt::BUF_LEN;

pub const DRIVER_NUM: usize = crate::driver::NUM::LowLevelDebug as usize;

pub struct LowLevelDebug<'u, U: Transmit<'u>> {
    buffer: Cell<Option<&'static mut [u8]>>,
    grant: Grant<AppData>,
    // grant_failed is set to true when LowLevelDebug fails to allocate an app's
    // grant region. When it has a chance, LowLevelDebug will print a message
    // indicating a grant initialization has failed, then set this back to
    // false. Although LowLevelDebug cannot print an application ID without
    // using grant storage, it will at least output an error indicating some
    // application's message was dropped.
    grant_failed: Cell<bool>,
    uart: &'u U,
}

impl<'u, U: Transmit<'u>> LowLevelDebug<'u, U> {
    pub fn new(
        buffer: &'static mut [u8],
        uart: &'u U,
        grant: Grant<AppData>,
    ) -> LowLevelDebug<'u, U> {
        LowLevelDebug {
            buffer: Cell::new(Some(buffer)),
            grant,
            grant_failed: Cell::new(false),
            uart,
        }
    }
}

impl<'u, U: Transmit<'u>> kernel::Driver for LowLevelDebug<'u, U> {
    fn command(&self, minor_num: usize, r2: usize, r3: usize, caller_id: AppId) -> ReturnCode {
        match minor_num {
            0 => return ReturnCode::SUCCESS,
            1 => self.push_entry(DebugEntry::AlertCode(r2), caller_id),
            2 => self.push_entry(DebugEntry::Print1(r2), caller_id),
            3 => self.push_entry(DebugEntry::Print2(r2, r3), caller_id),
            _ => return ReturnCode::ENOSUPPORT,
        }
        ReturnCode::SUCCESS
    }
}

impl<'u, U: Transmit<'u>> TransmitClient for LowLevelDebug<'u, U> {
    fn transmitted_buffer(&self, tx_buffer: &'static mut [u8], _tx_len: usize, _rval: ReturnCode) {
        // Identify and transmit the next queued entry. If there are no queued
        // entries remaining, store buffer.

        // Prioritize printing the "grant init failed" message over per-app
        // debug entries.
        if self.grant_failed.take() {
            const MESSAGE: &[u8] = b"LowLevelDebug: grant init failed\n";
            tx_buffer.copy_from_slice(MESSAGE);
            let (_, returned_buffer) = self.uart.transmit_buffer(tx_buffer, MESSAGE.len());
            self.buffer.set(returned_buffer);
            return;
        }

        for applied_grant in self.grant.iter() {
            let (app_num, first_entry) = applied_grant.enter(|owned_app_data, _| {
                owned_app_data.queue.rotate_left(1);
                (
                    owned_app_data.appid().id(),
                    owned_app_data.queue[QUEUE_SIZE - 1].take(),
                )
            });
            let to_print = match first_entry {
                None => continue,
                Some(to_print) => to_print,
            };
            self.transmit_entry(tx_buffer, app_num, to_print);
            return;
        }
        self.buffer.set(Some(tx_buffer));
    }
}

// -----------------------------------------------------------------------------
// Implementation details below
// -----------------------------------------------------------------------------

impl<'u, U: Transmit<'u>> LowLevelDebug<'u, U> {
    // If the UART is not busy (the buffer is available), transmits the entry.
    // Otherwise, adds it to the app's queue.
    fn push_entry(&self, entry: DebugEntry, appid: AppId) {
        use DebugEntry::Dropped;

        if let Some(buffer) = self.buffer.take() {
            self.transmit_entry(buffer, appid.id(), entry);
            return;
        }

        let result = self.grant.enter(appid, |borrow, _| {
            for queue_entry in &mut borrow.queue {
                if queue_entry.is_none() {
                    *queue_entry = Some(entry);
                    return;
                }
            }
            // The queue is full, so indicate some entries were dropped. If
            // there is not a drop entry, replace the last entry with a drop
            // counter. A new drop counter is initialized to two, as the
            // overwrite drops an entry plus we're dropping this entry.
            borrow.queue[QUEUE_SIZE - 1] = match borrow.queue[QUEUE_SIZE - 1] {
                Some(Dropped(count)) => Some(Dropped(count + 1)),
                _ => Some(Dropped(2)),
            };
        });

        // If we were unable to enter the grant region, schedule a diagnostic
        // message. This gives the user a chance of figuring out what happened
        // when LowLevelDebug fails.
        if result.is_err() {
            self.grant_failed.set(true);
        }
    }

    // Immediately prints the provided entry to the UART.
    fn transmit_entry(&self, buffer: &'static mut [u8], app_num: usize, entry: DebugEntry) {
        let msg_len = fmt::format_entry(app_num, entry, buffer);
        // The uart's error message is ignored because we cannot do anything if
        // it fails anyway.
        let (_, returned_buffer) = self.uart.transmit_buffer(buffer, msg_len);
        self.buffer.set(returned_buffer);
    }
}

// Length of the debug queue for each app. Each queue entry takes 3 words (tag
// and 2 usizes to print). The queue will be allocated in an app's grant region
// when that app first uses the debug driver.
const QUEUE_SIZE: usize = 4;

#[derive(Default)]
pub struct AppData {
    queue: [Option<DebugEntry>; QUEUE_SIZE],
}

#[derive(Clone, Copy)]
pub(crate) enum DebugEntry {
    Dropped(usize),       // Some debug messages were dropped
    AlertCode(usize),     // Display a predefined alert code
    Print1(usize),        // Print a single number
    Print2(usize, usize), // Print two numbers
}

impl<'u, U: Transmit<'u>> kernel::driver_registry::DriverInfo<crate::driver::NUM>
    for LowLevelDebug<'u, U>
{
    fn driver(&self) -> &dyn kernel::Driver {
        self
    }

    fn driver_type(&self) -> crate::driver::NUM {
        crate::driver::NUM::LowLevelDebug
    }

    fn driver_name(&self) -> &'static str {
        "low_level_debug"
    }

    fn instance_identifier<'b>(&'b self) -> &'b str {
        // TODO: This should be changed to something more meaningful
        "lld0"
    }
}
