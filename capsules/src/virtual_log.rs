use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::list::{List, ListLink};
use kernel::hil::log::{LogRead, LogReadClient, LogWrite, LogWriteClient};
use kernel::ReturnCode;

// The purpose of this type alias is to make it clear when a usize represents a log entry ID.
type EntryID = usize;

// This enum represents the current operation that a virtual log device is performing.
#[derive(Copy, PartialEq)]
enum Op {
    Idle,
    Read(&'static mut [u8], usize),
    Append(&'static mut [u8], usize),
    Sync,
    Erase,
}

pub struct VirtualLogDevice<'a, Log: LogRead<'a> + LogWrite<'a>> {
    // Every virtual log device has a reference to the mux.
    mux: &'a MuxLog<'a, Log>,
    // Every virtual log device has a pointer to the next virtual log device.
    next: ListLink<'a, VirtualLogDevice<'a, Log>>,
    // Every virtual log device has some local state.
    read_client: OptionalCell<&'a dyn LogReadClient>;
    append_client: OptionalCell<&'a dyn LogWriteClient>;
    operation: Cell<Op>,
    read_entry_id: Cell<EntryID>,
}

impl<'a, Log: LogRead<'a> + LogWrite<'a>> VirtualLogDevice<'a, Log> {
    pub const fn new(mux: &'a MuxLog<'a, Log>) -> VirtualLogDevice<'a, Log> {
        VirtualLogDevice {
            mux: mux,
            next: ListLink::empty(),
            read_client: OptionalCell::empty(),
            append_client: OptionalCell::empty(),
            operation: Cell::new(Op::Idle),
            read_entry_id: Cell::new(PAGE_HEADER_SIZE),
        }
    }
}

impl<'a, Log: LogRead<'a> + LogWrite<'a>> LogRead<'a> for VirtualLogDevice<'a, Log> {
    type EntryID = usize;

    // This method is used by a capsule to register itself as a read client of the virtual log device.
    fn set_read_client(&'a self, read_client: &'a dyn LogReadClient) {
        // TODO: Should we check if we're already part of the mux's devices list?
        self.mux.devices.push_head(self);
        self.read_client.set(client);
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)> {
        self.operation.set(Op::Read(buffer, length));
        self.mux.do_next_op();
        Ok(()))
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
    // TODO: check for errors
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
impl<'a, Log: LogRead<'a> + LogWrite<'a>> LogWrite<'a> for VirtualLogDevice<'a, Log> {
    // This method is used by a capsule to register itself as an append client of the virtual log device.
    fn set_append_client(&'a self, append_client: &'a dyn LogWriteClient) {
        // TODO: Should we check if we're already part of the mux's devices list?
        self.mux.devices.push_head(self);
        self.append_client.set(append_client);
    }

    fn append(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)> {
        self.operation.set(Op::Append(buffer, length));
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

/// The MuxLog struct manages multiple virtual log devices (i.e. VirtualLogDevice).
pub struct MuxLog<'a, Log: LogRead<'a> + LogWrite<'a>> {
    // The underlying log device being virtualized.
    log: &'a Log,
    // A list of virtual log devices that the mux manages.
    devices: List<'a, VirtualLogDevice<'a, Log>>,
    // Which virtual log device is currently being serviced.
    inflight: OptionalCell<&'a VirtualLogDevice<'a, Log>>,
}

impl<'a, Log: LogRead<'a> + LogWrite<'a>> MuxLog<'a, Log> {
    /// Creates a multiplexer around an underlying log device to virtualize it.
    pub const fn new(log: &'a Log) -> MuxLog<'a, Log> {
        MuxLog {
            log: log,
            devices: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    fn do_next_op(&self) {
        // If there's already a virtual log device being serviced, then return.
        if self.inflight.is_some() {
            return;
        }
        // Otherwise, we service the first log device that has something to do.
        // FIXME: Are there any fairness concerns here? What if we start searching where we left off?
        let mnode = self
            .devices
            .iter()
            .find(|node| node.operation.get() != Op::Idle);
        mnode.map(|node| {
            // Set the virtual log device's state to be idle after saving its operation locally.
            let op = node.operation.get();
            node.operation.set(Op::Idle);
            match op {
                Op::Read(buffer, length) => {
                    self.inflight.set(node);
                    self.log.read(buffer, length);
                }
                Op::Append(buffer, length) => {}
                Op::Sync => {}
                Op::Erase => {}
            }
        })
    }
}
