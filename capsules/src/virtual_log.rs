//! Virtualize the log storage abstraction.
use core::cell::Cell;
use core::marker::Copy;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::list::{List, ListLink, ListNode};
use kernel::debug;
use kernel::hil::log::{LogRead, LogReadClient, LogWrite, LogWriteClient};
use kernel::ReturnCode;

// Represents the current operation that a virtual log device is performing.
#[derive(Copy, Clone, PartialEq)]
enum Op {
    Idle,
    Read(usize),
    Append(usize),
    Sync,
    Erase,
}

pub struct VirtualLogDevice<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
{
    // A reference to the mux
    mux: &'a MuxLog<'a, Log>,
    // A pointer to the next virtual log device
    next: ListLink<'a, VirtualLogDevice<'a, Log>>,
    // Local state for the virtual log device
    read_client: OptionalCell<&'a dyn LogReadClient>,
    append_client: OptionalCell<&'a dyn LogWriteClient>,
    // The operation currently being performed.
    operation: Cell<Op>,
    // Virtual log device have their own local `read_entry_id` but
    // share a global `oldest_entry_id` and `append_entry_id`.
    read_entry_id: Cell<Log::EntryID>,
    // Although log devices can both read and write/append, only one buffer
    // is necessary because there can only be one outstanding operation.
    buffer: TakeCell<'static, [u8]>,
}

impl<'a, Log> ListNode<'a, VirtualLogDevice<'a, Log>> for VirtualLogDevice<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
{
    fn next(&'a self) -> &'a ListLink<'a, VirtualLogDevice<'a, Log>> {
        &self.next
    }
}

impl<'a, Log> VirtualLogDevice<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
{
    pub fn new(mux: &'a MuxLog<'a, Log>) -> VirtualLogDevice<'a, Log> {
        VirtualLogDevice {
            mux: mux,
            next: ListLink::empty(),
            read_client: OptionalCell::empty(),
            append_client: OptionalCell::empty(),
            operation: Cell::new(Op::Idle),
            read_entry_id: Cell::new(mux.log.log_start()),
            buffer: TakeCell::empty(),
        }
    }
}

impl<'a, Log> LogRead<'a> for VirtualLogDevice<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
    Log::EntryID: Copy,
{
    type EntryID = <Log as LogRead<'a>>::EntryID;

    // This method is used by a capsule to register itself as a read client of the virtual log device.
    fn set_read_client(&'a self, read_client: &'a dyn LogReadClient) {
        // TODO: Should we check if we're already part of the mux's devices list?
        self.mux.virtual_log_devices.push_head(self);
        self.read_client.set(read_client);
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)> {
        self.operation.set(Op::Read(length));
        match self.buffer.replace(buffer) {
            Some(_) => {
                debug!("Unexpected replacement of a read buffer that should've been returned via read_done.")
            }
            None => (),
        }
        self.mux.do_next_op();
        Ok(())
    }

    fn log_start(&self) -> Self::EntryID {
        self.mux.log.log_start()
    }

    fn log_end(&self) -> Self::EntryID {
        self.mux.log.log_end()
    }

    // TODO: this needs to be virtualized
    fn next_read_entry_id(&self) -> Self::EntryID {
        self.mux.log.next_read_entry_id()
    }

    // The seek function on the virtual log device doesn't actually cause a seek to occur on the
    // underlying persistent storage device. All it does is update a state variable representing
    // the location of its position in the log file.
    fn seek(&self, entry: Self::EntryID) -> ReturnCode {
        self.read_entry_id.set(entry);
        ReturnCode::SUCCESS
    }

    fn get_size(&self) -> usize {
        self.mux.log.get_size()
    }
}

// TODO: Should the append, sync, and erase functions check to make sure the virtual log device is idle?
// TODO: Should the virtual log device do some queuing of operations on its own?
impl<'a, Log> LogWrite<'a> for VirtualLogDevice<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
    Log::EntryID: Copy,
{
    // This method is used by a capsule to register itself as an append client of the virtual log device.
    fn set_append_client(&'a self, append_client: &'a dyn LogWriteClient) {
        // TODO: Should we check if we're already part of the mux's devices list?
        self.mux.virtual_log_devices.push_head(self);
        self.append_client.set(append_client);
    }

    fn append(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)> {
        self.operation.set(Op::Append(length));
        match self.buffer.replace(buffer) {
            Some(_) => {
                debug!("Unexpected replacement of an append buffer that should've been returned via append_done.")
            }
            None => (),
        }
        self.mux.do_next_op();
        Ok(())
    }

    fn sync(&self) -> ReturnCode {
        self.operation.set(Op::Sync);
        self.mux.do_next_op();
        ReturnCode::SUCCESS
    }

    fn erase(&self) -> ReturnCode {
        self.operation.set(Op::Erase);
        self.mux.do_next_op();
        ReturnCode::SUCCESS
    }
}

impl<'a, Log> LogReadClient for VirtualLogDevice<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
{
    /// Propagates the `read_done` callback up to the end user.
    fn read_done(&self, buffer: &'static mut [u8], result: Result<usize, ReturnCode>) {
        self.read_client.map_or_else(
            || debug!("Log read complete but log client is gone."),
            move |client| client.read_done(buffer, result),
        )
    }

    /// Propagates the `seek_done` callback up to the end user.
    fn seek_done(&self, error: ReturnCode) {
        self.read_client.map_or_else(
            || debug!("Log seek complete but log client is gone."),
            |client| client.seek_done(error),
        )
    }
}

impl<'a, Log> LogWriteClient for VirtualLogDevice<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
{
    /// Propagates the `append_done` callback up to the end user.
    fn append_done(&self, buffer: &'static mut [u8], result: Result<(usize, bool), ReturnCode>) {
        self.append_client.map_or_else(
            || debug!("Log append complete but log client is gone."),
            move |client| client.append_done(buffer, result),
        )
    }

    /// Propagates the `sync_done` callback up to the end user.
    fn sync_done(&self, error: ReturnCode) {
        self.append_client.map_or_else(
            || debug!("Log sync complete but log client is gone."),
            |client| client.sync_done(error),
        )
    }

    /// Propagates the `erase_done` callback up to the end user.
    fn erase_done(&self, error: ReturnCode) {
        self.append_client.map_or_else(
            || debug!("Log append complete but log client is gone."),
            |client| client.erase_done(error),
        )
    }
}

/// The MuxLog struct manages multiple virtual log devices (i.e. VirtualLogDevice) and is the lone
/// client of the underlying log device. Each of the virtual log devices can have at most one
/// outstanding log request.
pub struct MuxLog<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
{
    // The underlying log device being virtualized.
    log: &'a Log,
    // A list of virtual log devices that the mux manages.
    virtual_log_devices: List<'a, VirtualLogDevice<'a, Log>>,
    // Which virtual log device is currently being serviced.
    inflight: OptionalCell<&'a VirtualLogDevice<'a, Log>>,
}

impl<'a, Log> LogReadClient for MuxLog<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
    Log::EntryID: Copy,
{
    fn read_done(&self, buffer: &'static mut [u8], result: Result<usize, ReturnCode>) {
        self.inflight.take().map(move |device| {
            self.do_next_op();
            device.read_done(buffer, result);
        });
    }

    fn seek_done(&self, error: ReturnCode) {
        self.inflight.take().map(|device| {
            self.do_next_op();
            device.seek_done(error);
        });
    }
}

impl<'a, Log> LogWriteClient for MuxLog<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
    Log::EntryID: Copy,
{
    fn append_done(&self, buffer: &'static mut [u8], result: Result<(usize, bool), ReturnCode>) {
        self.inflight.take().map(move |device| {
            self.do_next_op();
            device.append_done(buffer, result);
        });
    }

    fn sync_done(&self, error: ReturnCode) {
        self.inflight.take().map(move |device| {
            self.do_next_op();
            device.sync_done(error);
        });
    }

    fn erase_done(&self, error: ReturnCode) {
        self.inflight.take().map(|device| {
            self.do_next_op();
            device.erase_done(error);
        });
    }
}

impl<'a, Log> MuxLog<'a, Log>
where
    Log: LogRead<'a> + LogWrite<'a>,
    Log::EntryID: Copy,
{
    /// Creates a multiplexer around an underlying log device to virtualize it.
    pub const fn new(log: &'a Log) -> MuxLog<'a, Log> {
        MuxLog {
            log: log,
            virtual_log_devices: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    fn do_next_op(&self) {
        // FIXME: Are there any fairness concerns here? What if we start searching where we left off?
        if self.inflight.is_none() {
            self.virtual_log_devices
                .iter()
                .find(|virtual_log_device| virtual_log_device.operation.get() != Op::Idle)
                .map(|virtual_log_device| {
                    self.inflight.set(virtual_log_device);
                    self.log.seek(virtual_log_device.read_entry_id.get());
                    let op = virtual_log_device.operation.get();
                    virtual_log_device.operation.set(Op::Idle);
                    match op {
                        Op::Read(length) => match virtual_log_device.buffer.take() {
                            Some(read_buffer) => match self.log.read(read_buffer, length) {
                                Ok(()) => (),
                                Err((error_code, Some(read_buffer))) => {
                                    self.read_done(read_buffer, Err(error_code))
                                }
                                Err((_, None)) => unreachable!(), // FIXME: change the return type of read() to get rid of this case
                            },
                            None => debug!("Error: read buffer is missing when issuing log read."),
                        },
                        Op::Append(length) => match virtual_log_device.buffer.take() {
                            Some(append_buffer) => match self.log.append(append_buffer, length) {
                                Ok(()) => (),
                                Err((error_code, Some(append_buffer))) => {
                                    self.append_done(append_buffer, Err(error_code))
                                }
                                Err((_, None)) => unreachable!(), // FIXME: change the return type of append() to get rid of this case
                            },
                            None => {
                                debug!("Error: append buffer is missing when issuing log append.")
                            }
                        },
                        Op::Sync => {
                            self.log.sync();
                        }
                        Op::Erase => {
                            self.log.erase();
                        }
                        Op::Idle => unreachable!(),
                    }
                });
        }
    }
}
