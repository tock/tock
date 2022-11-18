//! Provides userspace with access to a serial interface whose output
//! is in-order with respect to kernel debug!() operations.
//!
//! Setup
//! -----
//!
//! This capsule allows userspace programs to print to the kernel debug
//! log. This ensures that (as long as the writes are not truncated) that
//! kernel and userspace print operations are in order.
//!
//! ```rust
//! # use kernel::static_init;
//! # use capsules::console::Console;
//!
//! let print_log = static_init!(
//!     PrintLog,
//!     PrintLog::new(&usart::USART0,
//!                  115200,
//!                  board_kernel.create_grant(&grant_cap)));
//! ```
//!
//! Usage
//! -----
//!
//! The user must perform three steps in order to write a buffer:
//!
//! ```c
//! // (Optional) Set a callback to be invoked when the buffer has been written
//! subscribe(PRINTLOG_DRIVER_NUM, 1, my_callback);
//! // Share the buffer from userspace with the driver
//! allow(PRINTLOG_DRIVER_NUM, buffer, buffer_len_in_bytes);
//! // Initiate the transaction
//! command(PRINTLOG_DRIVER_NUM, 1, len_to_write_in_bytes)
//! ```
//!

use core::cell::Cell;

use kernel::debug_process_slice;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, GrantKernelData, UpcallCount};
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::processbuffer::ReadableProcessBuffer;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::PrintLog as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    /// Before the allow syscall was handled by the kernel,
    /// console used allow number "1", so to preserve compatibility
    /// we still use allow number 1 now.
    pub const WRITE: usize = 1;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Before the allow syscall was handled by the kernel,
    /// console used allow number "1", so to preserve compatibility
    /// we still use allow number 1 now.
    pub const _READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 0;
}

#[derive(Default)]
pub struct App {
    write_len: usize,    // Length of write
    writing: bool,       // Are we in the midst of a write
    pending_write: bool, // Are we waiting to write
    tx_counter: usize,   // Used to keep order of writes
}

pub struct PrintLog<'a, A: Alarm<'a>> {
    apps: Grant<
        App,
        UpcallCount<3>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    tx_in_progress: Cell<bool>, // If true there's an ongoing write so others must wait
    tx_counter: Cell<usize>,    // Sequence number for writes from different processes
    alarm: &'a A,               // Timer for trying to send  more
}

impl<'a, A: Alarm<'a>> PrintLog<'a, A> {
    pub fn new(
        alarm: &'a A,
        grant: Grant<
            App,
            UpcallCount<3>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> PrintLog<'a, A> {
        PrintLog {
            apps: grant,
            tx_in_progress: Cell::new(false),
            tx_counter: Cell::new(0),
            alarm: alarm,
        }
    }

    /// Internal helper function for starting up a new print; allocate a sequence number and
    /// start the send state machine.
    fn send_new(
        &self,
        app: &mut App,
        kernel_data: &GrantKernelData,
        len: usize,
    ) -> Result<(), ErrorCode> {
        // We are already writing
        if app.writing || app.pending_write {
            return Err(ErrorCode::BUSY);
        }
        app.write_len = kernel_data
            .get_readonly_processbuffer(ro_allow::WRITE)
            .map_or(0, |write| write.len())
            .min(len);
        // We have nothing to write
        if app.write_len == 0 {
            return Err(ErrorCode::NOMEM);
        }
        // Order the prints through a global counter.
        app.tx_counter = self.tx_counter.get();
        self.tx_counter.set(app.tx_counter.wrapping_add(1));
        if self.tx_in_progress.get() {
            // A prior print is outstanding, enqueue
            app.pending_write = true;
        } else {
            // No prior print, send some text.
            self.send(app, kernel_data);
        }
        Ok(())
    }

    /// Internal helper function for sending data.
    fn send(&self, app: &mut App, kernel_data: &GrantKernelData) {
        // We can ignore the Result because if the call fails, it means
        // the process has terminated, so issuing a callback doesn't matter.
        // If the call fails, just use the alarm to try the next client.
        let _res = kernel_data
            .get_readonly_processbuffer(ro_allow::WRITE)
            .and_then(|write| {
                write.enter(|data| {
                    // The slice might have become shorter than the requested
                    // write; if so, just write what there is.
                    let remaining_data = match data.get(0..app.write_len) {
                        Some(remaining_data) => remaining_data,
                        None => data,
                    };
                    app.writing = true;
                    self.tx_in_progress.set(true);
                    if remaining_data.len() > 0 {
                        debug_process_slice!(remaining_data);
                    } else {
                        // We have a zero-length slice: send something else
                    }
                })
            });

        // We're writing, so start a timer to send more and
        // block subsequent writes. Detect there are other
        // processes with things to write by looking at the
        // sequence number; if the next-to-be-given sequence
        // number != this process+1, then a process has
        // grabbed this process+1 and has something to send.
        self.alarm
            .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(10));
    }
}

impl<'a, A: Alarm<'a>> AlarmClient for PrintLog<'a, A> {
    fn alarm(&self) {
        //debug!("Alarm fired.");
        if self.tx_in_progress.get() {
            self.tx_in_progress.set(false);

            // Issue an upcall for the in-progress write
            for cntr in self.apps.iter() {
                cntr.enter(|app, kernel_data| {
                    // This is the in-progress write
                    if app.writing {
                        let _res = kernel_data.schedule_upcall(1, (app.write_len, 0, 0));
                        app.writing = false;
                    }
                });
            }
        }

        // Find if there's another writer and mark it busy.
        let mut next_writer: Option<ProcessId> = None;
        let mut seqno = self.tx_counter.get();

        // Find the process that has an outstanding write with the
        // lowest sequence number.
        for cntr in self.apps.iter() {
            let appid = cntr.processid();
            cntr.enter(|app, _| {
                if app.pending_write {
                    // This means app.tx_counter is smaller than seqno, as long as
                    // there aren't usize/2 processes.
                    if seqno.wrapping_sub(app.tx_counter) < usize::MAX / 2 {
                        seqno = app.tx_counter;
                        next_writer = Some(appid);
                    }
                }
            });
        }

        next_writer.map(|pid| {
            self.apps.enter(pid, |app, kernel_data| {
                app.pending_write = false;
                self.send(app, kernel_data);
            })
        });
    }
}

impl<'a, A: Alarm<'a>> SyscallDriver for PrintLog<'a, A> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///

    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Readonly buffer for write buffer

    // Setup callbacks.
    //
    // ### `subscribe_num`
    //
    // - `1`: Write buffer completed callback

    /// Initiate serial transfers
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Transmits a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    fn command(&self, cmd_num: usize, arg1: usize, _: usize, appid: ProcessId) -> CommandReturn {
        //debug!("PrintLog receiving command: {} {} from {:?}", cmd_num, arg1, appid);
        let res = self
            .apps
            .enter(appid, |app, kernel_data| {
                match cmd_num {
                    0 => Ok(()),
                    1 => {
                        // putstr
                        let len = arg1;
                        self.send_new(app, kernel_data, len)
                    }
                    _ => Err(ErrorCode::NOSUPPORT),
                }
            })
            .map_err(ErrorCode::from);
        //debug!("Result: {:?}", res);
        match res {
            Ok(Ok(())) => CommandReturn::success(),
            Ok(Err(e)) => CommandReturn::failure(e),
            Err(e) => CommandReturn::failure(e),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
