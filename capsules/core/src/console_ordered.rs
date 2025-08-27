// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides userspace with access to a serial interface whose output
//! is in-order with respect to kernel debug!() operations.
//!
//! Prints to the console are atomic up to particular constant length,
//! which can be set at capsule instantiation.
//!
//! Note that this capsule does *not* buffer writes in an additional
//! buffer; this is critical to ensure ordering. Instead, it pushes
//! writes into the kernel debug buffer. If there is insufficient space
//! in the buffer for the write (or an atomic block size chunk of a very
//! large write), the capsule waits and uses a retry timer. This means
//! that in-kernel debug statements can starve userspace prints, e.g.,
//! if they always keep the kernel debug buffer full.
//!
//! Setup
//! -----
//!
//! This capsule allows userspace programs to print to the kernel
//! debug log. This ensures that (as long as the writes are not
//! truncated) that kernel and userspace print operations are in
//! order. It requires a reference to an Alarm for timers to issue
//! callbacks and send more data. The three configuration constants are:
//!   - ATOMIC_SIZE: the minimum block of buffer that will be sent. If there is
//!     not enough space in the debug buffer to send ATOMIC_SIZE bytes, the
//!     console retries later.
//!   - RETRY_TIMER: if there is not enough space in the debug buffer to send
//!     the next chunk of a write, the console waits RETRY_TIMER ticks of the
//!     supplied alarm.
//!   - WRITE_TIMER: after completing a write, the console waits WRITE_TIMER
//!     ticks of the supplied alarm before issuing a callback or writing more.
//!
//! RETRY_TIMER and WRITE_TIMER should be set based on the speed of
//! the underlying UART and desired load. Generally speaking, setting
//! them around 50-100 byte times is good. For example, this means on
//! a 115200 UART, setting them to 5ms (576 bits, or 72 bytes) is
//! reasonable. ATOMIC_SIZE should be at least 80 (row width
//! of a standard console).
//!
//! ```rust,ignore
//! # use kernel::static_init;
//! # use capsules_core::console_ordered::ConsoleOrdered;
//! let console = static_init!(
//!     ConsoleOrdered,
//!     ConsoleOrdered::new(virtual_alarm,
//!                         board_kernel.create_grant(capsules_core::console_ordered::DRIVER_NUM,
//!                                                   &grant_cap),
//!                         ATOMIC_SIZE,
//!                         RETRY_TIMER,
//!                         WRITE_TIMER));
//!
//! ```
//!
//! Usage
//! -----
//!
//! The user must perform three steps in order to write a buffer:
//!
//! ```c
//! // (Optional) Set a callback to be invoked when the buffer has been written
//! subscribe(CONSOLE_DRIVER_NUM, 1, my_callback);
//! // Share the buffer from userspace with the driver
//! allow(CONSOLE_DRIVER_NUM, buffer, buffer_len_in_bytes);
//! // Initiate the transaction
//! command(CONSOLE_DRIVER_NUM, 1, len_to_write_in_bytes)
//! ```
//!

use core::cell::Cell;
use core::cmp;

use kernel::debug::debug_available_len;
use kernel::debug_process_slice;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, GrantKernelData, UpcallCount};
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::hil::uart;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Console as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    /// Before the allow syscall was handled by the kernel,
    /// console used allow number "1", so to preserve compatibility
    /// we still use allow number 1 now.
    pub const WRITE: usize = 1;
    /// The number of read-allow buffers (for putstr) the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Before the allow syscall was handled by the kernel,
    /// console used allow number "1", so to preserve compatibility
    /// we still use allow number 1 now.
    pub const READ: usize = 1;
    /// The number of read-write allow buffers (for getstr) the kernel stores for this grant
    pub const COUNT: u8 = 2;
}

#[derive(Default)]
pub struct App {
    write_position: usize, // Current write position
    write_len: usize,      // Length of total write
    writing: bool,         // Are we in the midst of a write
    pending_write: bool,   // Are we waiting to write
    tx_counter: usize,     // Used to keep order of writes
    read_len: usize,       // Read length
    rx_counter: usize,     // Used to order reads (no starvation)
}

pub struct ConsoleOrdered<'a, A: Alarm<'a>> {
    uart: &'a dyn uart::Receive<'a>,
    apps: Grant<
        App,
        UpcallCount<3>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    tx_in_progress: Cell<bool>, // If true there's an ongoing write so others must wait
    tx_counter: Cell<usize>,    // Sequence number for writes from different processes
    alarm: &'a A,               // Timer for trying to send  more

    rx_counter: Cell<usize>,
    rx_in_progress: OptionalCell<ProcessId>,
    rx_buffer: TakeCell<'static, [u8]>,

    atomic_size: Cell<usize>, // The maximum size write the capsule promises atomicity;
    // larger writes may be broken into atomic_size chunks.
    // This must be smaller than the debug buffer size or a long
    // write may never print.
    retry_timer: Cell<u32>, // How long the capsule will wait before retrying if there
    // is insufficient space in the debug buffer (alarm ticks)
    // when a write is first attempted.
    write_timer: Cell<u32>, // Time to wait after a successful write into the debug buffer,
                            // before checking whether write more or issue a callback that
                            // the current write has completed (alarm ticks).
}

impl<'a, A: Alarm<'a>> ConsoleOrdered<'a, A> {
    pub fn new(
        uart: &'a dyn uart::Receive<'a>,
        alarm: &'a A,
        rx_buffer: &'static mut [u8],
        grant: Grant<
            App,
            UpcallCount<3>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        atomic_size: usize,
        retry_timer: u32,
        write_timer: u32,
    ) -> ConsoleOrdered<'a, A> {
        ConsoleOrdered {
            uart,
            apps: grant,
            tx_in_progress: Cell::new(false),
            tx_counter: Cell::new(0),
            alarm,

            rx_counter: Cell::new(0),
            rx_in_progress: OptionalCell::empty(),
            rx_buffer: TakeCell::new(rx_buffer),

            atomic_size: Cell::new(atomic_size),
            retry_timer: Cell::new(retry_timer),
            write_timer: Cell::new(write_timer),
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
        app.write_position = 0;
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

        let debug_space_avail = debug_available_len();

        if self.tx_in_progress.get() {
            // A prior print is outstanding, enqueue
            app.pending_write = true;
        } else if app.write_len <= debug_space_avail {
            // Space for the full write, make it
            app.write_position = self.send(app, kernel_data).map_or(0, |len| len);
        } else if self.atomic_size.get() <= debug_space_avail {
            // Space for a partial write, make it
            app.write_position = self.send(app, kernel_data).map_or(0, |len| len);
        } else {
            // No space even for a partial, minimum size write: enqueue
            app.pending_write = true;
            self.alarm.set_alarm(
                self.alarm.now(),
                self.alarm.ticks_from_ms(self.retry_timer.get()),
            );
        }
        Ok(())
    }

    /// Internal helper function for sending data. Assumes that there is enough
    /// space in the debug buffer for the write. Writes longer than available
    /// debug buffer space will be truncated, so callers that wish to not lose
    /// data must check before calling.
    fn send(
        &self,
        app: &mut App,
        kernel_data: &GrantKernelData,
    ) -> Result<usize, kernel::process::Error> {
        // We can ignore the Result because if the call fails, it means
        // the process has terminated, so issuing a callback doesn't matter.
        // If the call fails, just use the alarm to try the next client.
        let res = kernel_data
            .get_readonly_processbuffer(ro_allow::WRITE)
            .and_then(|write| {
                write.enter(|data| {
                    // The slice might have become shorter than the requested
                    // write; if so, just write what there is.
                    let remaining_len = app.write_len - app.write_position;
                    let real_write_len = cmp::min(remaining_len, debug_available_len());
                    let this_write_end = app.write_position + real_write_len;
                    let remaining_data = match data.get(app.write_position..this_write_end) {
                        Some(remaining_data) => remaining_data,
                        None => data,
                    };

                    app.writing = true;
                    self.tx_in_progress.set(true);
                    if real_write_len > 0 {
                        let count = debug_process_slice!(remaining_data);
                        count
                    } else {
                        0
                    }
                })
            });
        // Start a timer to signal completion of this write
        // and potentially write more.
        self.alarm.set_alarm(
            self.alarm.now(),
            self.alarm.ticks_from_ms(self.write_timer.get()),
        );
        res
    }

    /// Internal helper function for starting a receive operation. Processes
    /// do not share reads, they take turns, with turn order monitored through
    /// a sequence number.
    fn receive_new(
        &self,
        processid: ProcessId,
        app: &mut App,
        kernel_data: &GrantKernelData,
        len: usize,
    ) -> Result<(), ErrorCode> {
        if app.read_len != 0 {
            // We are busy reading, don't try again
            Err(ErrorCode::BUSY)
        } else if len == 0 {
            //  Cannot read length 0
            Err(ErrorCode::INVAL)
        } else if self.rx_buffer.is_none() {
            // Console is busy receiving, so enqueue
            app.rx_counter = self.rx_counter.get();
            self.rx_counter.set(app.rx_counter + 1);
            app.read_len = len;
            Ok(())
        } else {
            // App can try to start a read
            let read_len = kernel_data
                .get_readwrite_processbuffer(rw_allow::READ)
                .map_or(0, |read| read.len())
                .min(len);
            if read_len > self.rx_buffer.map_or(0, |buf| buf.len()) {
                // For simplicity, impose a small maximum receive length
                // instead of doing incremental reads
                Err(ErrorCode::INVAL)
            } else {
                // Note: We have ensured above that rx_buffer is present
                app.read_len = read_len;
                self.rx_buffer.take().map(|buffer| {
                    self.rx_in_progress.set(processid);
                    let _ = self.uart.receive_buffer(buffer, app.read_len);
                });
                Ok(())
            }
        }
    }
}

impl<'a, A: Alarm<'a>> AlarmClient for ConsoleOrdered<'a, A> {
    fn alarm(&self) {
        if self.tx_in_progress.get() {
            // Clear here and set it later; if .enter fails (process
            // has died) it remains cleared.
            self.tx_in_progress.set(false);

            // Check if the current writer is finished; if so, issue an upcall, if not,
            // try to write more.
            for cntr in self.apps.iter() {
                cntr.enter(|app, kernel_data| {
                    // This is the in-progress write
                    if app.writing {
                        if app.write_position >= app.write_len {
                            let _res = kernel_data.schedule_upcall(1, (app.write_len, 0, 0));
                            app.writing = false;
                        } else {
                            // Still have more to write, don't allow others to jump in.
                            self.tx_in_progress.set(true);

                            // Promise to write to the end, or the atomic write unit, whichever is smaller
                            let remaining_len = app.write_len - app.write_position;
                            let debug_space_avail = debug_available_len();
                            let minimum_write = cmp::min(remaining_len, self.atomic_size.get());

                            // Write, or if there isn't space for a minimum write, retry later
                            if minimum_write <= debug_space_avail {
                                app.write_position +=
                                    self.send(app, kernel_data).map_or(0, |len| len);
                            } else {
                                self.alarm.set_alarm(
                                    self.alarm.now(),
                                    self.alarm.ticks_from_ms(self.retry_timer.get()),
                                );
                            }
                        }
                    }
                });
            }
        }

        // There's no ongoing send, try to send the next one (process with
        // lowest sequence number).
        if !self.tx_in_progress.get() {
            // Find if there's another writer and mark it busy.
            let mut next_writer: Option<ProcessId> = None;
            let mut seqno = self.tx_counter.get();

            // Find the process that has an outstanding write with the
            // earliest sequence number, handling wraparound.
            for cntr in self.apps.iter() {
                let appid = cntr.processid();
                cntr.enter(|app, _| {
                    if app.pending_write {
                        // Checks wither app.tx_counter is earlier than
                        // seqno, with the constrain that there are <
                        // usize/2 processes. wrapping_sub allows this to
                        // handle wraparound E.g., in 8-bit arithmetic
                        // 0x02 - 0xff = 0x03 and so 0xff is "earlier"
                        // than 0x02. -pal
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
                    let len = app.write_len;
                    let _ = self.send_new(app, kernel_data, len);
                })
            });
        }
    }
}

impl<'a, A: Alarm<'a>> SyscallDriver for ConsoleOrdered<'a, A> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Readonly buffer for write buffer
    ///
    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `1`: Write buffer completed callback
    ///
    /// Initiate serial transfers
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver existence check.
    /// - `1`: Transmits a buffer passed via `allow`, up to the length passed in
    ///   `arg1`
    fn command(&self, cmd_num: usize, arg1: usize, _: usize, appid: ProcessId) -> CommandReturn {
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
                    2 => {
                        // getnstr
                        let len = arg1;
                        self.receive_new(appid, app, kernel_data, len)
                    }
                    3 => {
                        // Abort RX
                        let _ = self.uart.receive_abort();
                        Ok(())
                    }
                    _ => Err(ErrorCode::NOSUPPORT),
                }
            })
            .map_err(ErrorCode::from);
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

impl<'a, A: Alarm<'a>> uart::ReceiveClient for ConsoleOrdered<'a, A> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        rcode: Result<(), ErrorCode>,
        error: uart::Error,
    ) {
        // First, handle this read, then see if there's another read to process.
        self.rx_in_progress
            .take()
            .map(|processid| {
                self.apps
                    .enter(processid, |app, kernel_data| {
                        // An iterator over the returned buffer yielding only the first `rx_len`
                        // bytes
                        let rx_buffer = buffer.iter().take(rx_len);
                        app.read_len = 0; // Mark that we are no longer reading.
                        match error {
                            uart::Error::None | uart::Error::Aborted => {
                                // Receive some bytes, signal error type and return bytes to process buffer
                                let count = kernel_data
                                    .get_readwrite_processbuffer(rw_allow::READ)
                                    .and_then(|read| {
                                        read.mut_enter(|data| {
                                            let mut c = 0;
                                            for (a, b) in data.iter().zip(rx_buffer) {
                                                c += 1;
                                                a.set(*b);
                                            }
                                            c
                                        })
                                    })
                                    .unwrap_or(-1);

                                // Make sure we report the same number
                                // of bytes that we actually copied into
                                // the app's buffer. This is defensive:
                                // we shouldn't ever receive more bytes
                                // than will fit in the app buffer since
                                // we use the app_buffer's length when
                                // calling `receive()`. However, a buggy
                                // lower layer could return more bytes
                                // than we asked for, and we don't want
                                // to propagate that length error to
                                // userspace. However, we do return an
                                // error code so that userspace knows
                                // something went wrong.
                                //
                                // If count < 0 this means the buffer
                                // disappeared: return NOMEM.
                                let read_buffer_len = kernel_data
                                    .get_readwrite_processbuffer(rw_allow::READ)
                                    .map_or(0, |read| read.len());
                                let (ret, received_length) = if count < 0 {
                                    (Err(ErrorCode::NOMEM), 0)
                                } else if rx_len > read_buffer_len {
                                    // Return `SIZE` indicating that
                                    // some received bytes were dropped.
                                    // We report the length that we
                                    // actually copied into the buffer,
                                    // but also indicate that there was
                                    // an issue in the kernel with the
                                    // receive.
                                    (Err(ErrorCode::SIZE), read_buffer_len)
                                } else {
                                    // This is the normal and expected
                                    // case.
                                    (rcode, rx_len)
                                };
                                let _ = kernel_data.schedule_upcall(
                                    2,
                                    (kernel::errorcode::into_statuscode(ret), received_length, 0),
                                );
                            }
                            _ => {
                                // Some UART error occurred
                                let _ = kernel_data.schedule_upcall(
                                    2,
                                    (
                                        kernel::errorcode::into_statuscode(Err(ErrorCode::FAIL)),
                                        0,
                                        0,
                                    ),
                                );
                            }
                        }
                    })
                    .unwrap_or_default();
            })
            .unwrap_or_default();

        // Whatever happens, we want to make sure to replace the rx_buffer for future transactions
        self.rx_buffer.replace(buffer);

        // Find if there's another reader and if so start reading
        let mut next_reader: Option<ProcessId> = None;
        let mut seqno = self.tx_counter.get();

        for cntr in self.apps.iter() {
            let appid = cntr.processid();
            cntr.enter(|app, _| {
                if app.read_len != 0 {
                    // Checks wither app.tx_counter is earlier than
                    // seqno, with the constrain that there are <
                    // usize/2 processes. wrapping_sub allows this to
                    // handle wraparound E.g., in 8-bit arithmetic
                    // 0x02 - 0xff = 0x03 and so 0xff is "earlier"
                    // than 0x02. -pal
                    if seqno.wrapping_sub(app.rx_counter) < usize::MAX / 2 {
                        seqno = app.rx_counter;
                        next_reader = Some(appid);
                    }
                }
            });
        }

        next_reader.map(|pid| {
            self.apps.enter(pid, |app, kernel_data| {
                let len = app.read_len;
                let _ = self.receive_new(pid, app, kernel_data, len);
            })
        });
    }
}
