//! Provides userspace with access to a serial interface.
//!
//! Setup
//! -----
//!
//! You need a device that provides the `hil::uart::UART` trait.
//!
//! ```rust
//! let console = static_init!(
//!     Console<usart::USART>,
//!     Console::new(&usart::USART0,
//!                  115200,
//!                  &mut console::WRITE_BUF,
//!                  &mut console::READ_BUF,
//!                  kernel::Grant::create()));
//! hil::uart::UART::set_client(&usart::USART0, console);
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
//! When the buffer has been written successfully, the buffer is released from
//! the driver. Successive writes must call `allow` each time a buffer is to be
//! written.

use core::cmp;
use kernel::common::cells::{MapCell, OptionalCell, TakeCell};
use kernel::common::chunked_process::{ChunkedProcess, ChunkedProcessClient, ChunkedProcessMode};
use kernel::hil::uart;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::CONSOLE as usize;

// ============================================================================
// I'm guessing this could be helpful for all kinds of apps and should be moved
// somewhere else
use core::mem;
enum AppOperationState {
    None,
    BufferSet(AppSlice<Shared, u8>),
    Pending(usize, usize, AppSlice<Shared, u8>),
    InProgress,
}
impl AppOperationState {
    fn new() -> AppOperationState {
        AppOperationState::None
    }

    fn set_buffer(&mut self, buf: Option<AppSlice<Shared, u8>>) -> bool {
        let prev = mem::replace(self, AppOperationState::None);

        if let AppOperationState::None = prev {
            if let Some(b) = buf {
                mem::replace(self, AppOperationState::BufferSet(b));
            } else {
                mem::replace(self, AppOperationState::None);
            }
            true
        } else if let AppOperationState::BufferSet(_) = prev {
            if let Some(b) = buf {
                mem::replace(self, AppOperationState::BufferSet(b));
            } else {
                mem::replace(self, AppOperationState::None);
            }
            true
        } else {
            // Either pending or in progress, don't remove buffer
            mem::replace(self, prev);
            false
        }
    }

    fn buffer_len(&self) -> Option<usize> {
        match self {
            &AppOperationState::BufferSet(ref buf) => Some(buf.len()),
            &AppOperationState::Pending(_, _, ref buf) => Some(buf.len()),
            _ => None,
        }
    }

    fn process_pending(&mut self) -> Option<(usize, usize, AppSlice<Shared, u8>)> {
        let prev = mem::replace(self, AppOperationState::InProgress);

        if let AppOperationState::Pending(start, len, buf) = prev {
            Some((start, len, buf))
        } else {
            // No operation pending, either none set or in progress
            // Revert back to old state
            mem::replace(self, prev);
            None
        }
    }

    fn has_pending(&self) -> bool {
        if let &AppOperationState::Pending(_, _, _) = &self {
            true
        } else {
            false
        }
    }

    fn set_pending(&mut self, start: usize, len: usize) -> bool {
        let prev = mem::replace(self, AppOperationState::None);

        if let AppOperationState::BufferSet(buf) = prev {
            mem::replace(self, AppOperationState::Pending(start, len, buf));
            true
        } else {
            // Either no buffer set, already pending or in progress
            mem::replace(self, prev);
            false
        }
    }

    fn operation_finished(&mut self, buf: AppSlice<Shared, u8>) {
        let prev = mem::replace(self, AppOperationState::BufferSet(buf));

        match prev {
            AppOperationState::InProgress => (),
            _ => panic!("Operation finished called with no operation in progress"),
        }
    }
}
impl Default for AppOperationState {
    fn default() -> AppOperationState {
        AppOperationState::new()
    }
}
// ============================================================================

#[derive(Default)]
pub struct App {
    write_callback: Option<Callback>,
    write: AppOperationState,

    read_callback: Option<Callback>,
    read_buffer: Option<AppSlice<Shared, u8>>,
    read_len: usize,
}

pub static mut WRITE_BUF: [u8; 64] = [0; 64];
pub static mut READ_BUF: [u8; 64] = [0; 64];

// TODO: Remove static lifetime on ChunkedProcess
pub struct Console<'a> {
    uart: &'a uart::UartData<'a>,
    apps: Grant<App>,
    process: MapCell<
        Result<
            &'static mut [u8],
            ChunkedProcess<'static, 'static, AppSlice<Shared, u8>, u8, AppId, (AppId, ReturnCode)>,
        >,
    >,
    chunked_process_client: OptionalCell<&'static Console<'static>>,
    current_write_operation: OptionalCell<AppId>,
    rx_in_progress: OptionalCell<AppId>,
    rx_buffer: TakeCell<'static, [u8]>,
}

impl Console<'a> {
    pub fn new(
        uart: &'a uart::UartData<'a>,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        grant: Grant<App>,
    ) -> Console<'a> {
        Console {
            uart: uart,
            apps: grant,

            process: MapCell::new(Ok(tx_buffer)),
            chunked_process_client: OptionalCell::empty(),
            current_write_operation: OptionalCell::empty(),

            rx_in_progress: OptionalCell::empty(),
            rx_buffer: TakeCell::new(rx_buffer),
        }
    }

    pub fn set_self_reference(&self, self_ref: &'static Console<'static>) {
        self.chunked_process_client.set(self_ref);
    }

    /// If there is some send-operation of any app, this will set the
    /// app's operation state to pending and return false.
    /// Otherwise, this tries to start the write operation immediately.
    fn try_send(&self, app: &mut App, app_id: AppId, len: usize) -> (bool, ReturnCode) {
        // Do some basic checks. Are the buffers set? Is the length okay?
        let checks = app
            .write
            .buffer_len()
            .map_or(Err(ReturnCode::ERESERVE), |blen| {
                if blen < len {
                    Err(ReturnCode::EINVAL)
                } else {
                    Ok(())
                }
            });

        if let Err(e) = checks {
            return (false, e);
        }

        // First, try to set this operation as pending
        if app.write.set_pending(0, len) {
            self.try_pending_send(app, app_id)
        } else {
            // There'a already a pending write operation
            (false, ReturnCode::EBUSY)
        }
    }

    fn try_pending_send(&self, app: &mut App, app_id: AppId) -> (bool, ReturnCode) {
        if app.write.has_pending() {
            // App has pending operation
            // Try to take ownership over the global operation buffer

            // The process MapCell is only there for swapping the Result-variants
            let process = self.process.take().expect("process mapcell taken");

            let (new_process, ret) = process
                .map(|buf: &'static mut [u8]| {
                    // We've got ownership over the buffer (no operation was currently
                    // going on)
                    let (_start, len, appbuf) = app
                        .write
                        .process_pending()
                        .expect("App not in pending state");
                    (
                        Err(self.start_operation(
                            ChunkedProcessMode::Read,
                            buf,
                            app_id,
                            appbuf,
                            len,
                        )),
                        (true, ReturnCode::SUCCESS),
                    )
                })
                .unwrap_or_else(|operation| {
                    // There's already an operation going on
                    // Put the value back
                    // Return false as the operation wasn't immediately started,
                    // but SUCCESS as it is pending
                    (Err(operation), (false, ReturnCode::SUCCESS))
                });

            self.process.put(new_process);

            ret
        } else {
            // No pending write operation for this app
            (false, ReturnCode::ERESERVE)
        }
    }

    // This method could later be reused for RX as well
    fn start_operation(
        &self,
        mode: ChunkedProcessMode,
        buf: &'static mut [u8],
        app_id: AppId,
        app_buffer: AppSlice<Shared, u8>,
        len: usize,
    ) -> ChunkedProcess<'static, 'static, AppSlice<Shared, u8>, u8, AppId, (AppId, ReturnCode)>
    {
        let chunked_process = ChunkedProcess::new(app_buffer, buf);
        self.chunked_process_client
            .map(|client| chunked_process.set_client(*client))
            .expect("can't set process client");

        chunked_process
            .run(mode, 0, len, app_id)
            .expect("chunked process run error");
        chunked_process
    }

    fn finished_send(&self, _mode: ChunkedProcessMode, res: Result<AppId, (AppId, ReturnCode)>) {
        // Destroy the chunked process instance and place back the raw buffer
        // TODO: Sane error handling here please
        let (slice, buf) = self
            .process
            .take()
            .expect("process is taken")
            .err()
            .expect("chunked process is not set")
            .destroy()
            .expect("chunked process error");
        self.process.put(Ok(buf));

        match res {
            Err((appid, retcode)) => {
                self.apps
                    .enter(appid, move |app, _| {
                        app.write.operation_finished(slice);
                        app.write_callback
                            .map(|mut cb| cb.schedule(From::from(retcode), 0, 0));
                    })
                    .unwrap_or_else(|_e| ());
            }
            Ok(appid) => {
                self.apps
                    .enter(appid, move |app, _| {
                        app.write.operation_finished(slice);
                        app.write_callback
                            .map(|mut cb| cb.schedule(From::from(ReturnCode::SUCCESS), 0, 0));
                    })
                    .unwrap_or_else(|_e| ());
            }
        }

        for cntr in self.apps.iter() {
            let app_id = cntr.appid();

            let operation_started = cntr.enter(|app, _| self.try_pending_send(app, app_id).0);

            if operation_started {
                break;
            };
        }
    }

    /// Internal helper function for starting a receive operation
    fn receive_new(&self, app_id: AppId, app: &mut App, len: usize) -> ReturnCode {
        if self.rx_buffer.is_none() {
            // For now, we tolerate only one concurrent receive operation on this console.
            // Competing apps will have to retry until success.
            return ReturnCode::EBUSY;
        }

        match app.read_buffer {
            Some(ref slice) => {
                let read_len = cmp::min(len, slice.len());
                if read_len > self.rx_buffer.map_or(0, |buf| buf.len()) {
                    // For simplicity, impose a small maximum receive length
                    // instead of doing incremental reads
                    ReturnCode::EINVAL
                } else {
                    // Note: We have ensured above that rx_buffer is present
                    app.read_len = read_len;
                    self.rx_buffer.take().map(|buffer| {
                        self.rx_in_progress.set(app_id);
                        let (_err, _opt) = self.uart.receive_buffer(buffer, app.read_len);
                    });
                    ReturnCode::SUCCESS
                }
            }
            None => {
                // Must supply read buffer before performing receive operation
                ReturnCode::EINVAL
            }
        }
    }
}

impl Driver for Console<'a> {
    /// Setup shared buffers.
    ///
    /// ### `allow_num`
    ///
    /// - `1`: Writeable buffer for write buffer
    /// - `2`: Writeable buffer for read buffer
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            1 => self
                .apps
                .enter(appid, |app, _| {
                    if app.write.set_buffer(slice) {
                        ReturnCode::SUCCESS
                    } else {
                        // Either in progress or pending
                        ReturnCode::EBUSY
                    }
                })
                .unwrap_or_else(|err| err.into()),
            2 => self
                .apps
                .enter(appid, |app, _| {
                    app.read_buffer = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `1`: Write buffer completed callback
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            1 /* putstr/write_done */ => {
                self.apps.enter(app_id, |app, _| {
                    app.write_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            2 /* getnstr done */ => {
                self.apps.enter(app_id, |app, _| {
                    app.read_callback = callback;
                    ReturnCode::SUCCESS
                }).unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    /// Initiate serial transfers
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Transmits a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `2`: Receives into a buffer passed via `allow`, up to the length
    ///        passed in `arg1`
    /// - `3`: Cancel any in progress receives and return (via callback)
    ///        what has been received so far.
    fn command(&self, cmd_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* putstr */ => {
                let len = arg1;
                self.apps.enter(appid, |app, _| {
                    self.try_send(app, appid, len).1
                }).unwrap_or_else(|err| err.into())
            },
            2 /* getnstr */ => {
                let len = arg1;
                self.apps.enter(appid, |app, _| {
                    self.receive_new(appid, app, len)
                }).unwrap_or_else(|err| err.into())
            },
            3 /* abort rx */ => {
                self.uart.receive_abort();
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT
        }
    }
}

impl uart::TransmitClient for Console<'a> {
    fn transmitted_buffer(&self, buffer: &'static mut [u8], _tx_len: usize, _rcode: ReturnCode) {
        // Clear the temporary current operation values
        let app_id = self
            .current_write_operation
            .take()
            .expect("current operation app id not saved before send");

        let finished = self
            .process
            .map(move |process| {
                process
                    .as_ref()
                    .err()
                    .expect("no chunked process in callback")
                    .chunk_done(buffer, Ok(app_id))
            })
            .expect("process is None");

        if let Some((mode, res)) = finished {
            self.finished_send(mode, res);
        }
    }
}

impl uart::ReceiveClient for Console<'a> {
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        rcode: ReturnCode,
        error: uart::Error,
    ) {
        self.rx_in_progress
            .take()
            .map(|appid| {
                self.apps
                    .enter(appid, |app, _| {
                        app.read_callback.map(|mut cb| {
                            // An iterator over the returned buffer yielding only the first `rx_len`
                            // bytes
                            let rx_buffer = buffer.iter().take(rx_len);
                            match error {
                                uart::Error::None | uart::Error::Aborted => {
                                    // Receive some bytes, signal error type and return bytes to process buffer
                                    if let Some(mut app_buffer) = app.read_buffer.take() {
                                        for (a, b) in app_buffer.iter_mut().zip(rx_buffer) {
                                            *a = *b;
                                        }
                                        cb.schedule(From::from(rcode), rx_len, 0);
                                    } else {
                                        // Oops, no app buffer
                                        cb.schedule(From::from(ReturnCode::EINVAL), 0, 0);
                                    }
                                }
                                _ => {
                                    // Some UART error occurred
                                    cb.schedule(From::from(ReturnCode::FAIL), 0, 0);
                                }
                            }
                        });
                    })
                    .unwrap_or_default();
            })
            .unwrap_or_default();

        // Whatever happens, we want to make sure to replace the rx_buffer for future transactions
        self.rx_buffer.replace(buffer);
    }
}

impl ChunkedProcessClient<'static, u8, AppId, (AppId, ReturnCode)> for Console<'a> {
    fn read_chunk(
        &self,
        _current_pos: usize,
        appid: AppId,
        chunk: &'static mut [u8],
        len: usize,
    ) -> Result<(), (&'static mut [u8], (AppId, ReturnCode))> {
        // We're reading from the chunk, so this is actually a write operation

        // We need to store the current AppId temporarily because this isn't returned
        // from the underlying Uart implementation. This is required for the accumulator value.
        // If there was a value here before replace, multiple write operations were
        // running simultaneously (this should NEVER happen)
        assert!(self.current_write_operation.replace(appid).is_none());

        let (err, opt) = self.uart.transmit_buffer(chunk, len);

        if let Some(buf) = opt {
            Err((buf, (appid, err)))
        } else {
            Ok(())
        }
    }

    fn write_chunk(
        &self,
        _current_pos: usize,
        _appid: AppId,
        _chunk: &'static mut [u8],
        _len: usize,
    ) -> Result<(), (&'static mut [u8], (AppId, ReturnCode))> {
        unimplemented!();
    }

    fn read_write_chunk(
        &self,
        _current_pos: usize,
        _appid: AppId,
        _chunk: &'static mut [u8],
        _len: usize,
    ) -> Result<(), (&'static mut [u8], (AppId, ReturnCode))> {
        unimplemented!();
    }
}
