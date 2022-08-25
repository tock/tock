//! This provides userspace access to block storage.
//!
//! This is a basic implementation that gives total control of the storage
//! to the userspace.
//!
//! Example instantiation:
//!
//! ```rust
//! # use kernel::static_init;
//!
//! ```

use core::cell::Cell;
use core::cmp;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::hil::block_storage::BlockIndex;
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::TakeCell;
use kernel::{ErrorCode, ProcessId};

use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::BlockStorage as usize;

#[allow(non_camel_case_types)]
enum Command {
    CHECK = 0,
    /// device size in bytes
    SIZE = 1,
    /// Size of write, discard blocks.
    /// Separate from size because doesn't fit in one return call.
    GEOMETRY = 2,
    /// Read an arbitrary range from an arbitrary address.
    READ_RANGE = 3,
    /// Read a single write block at given block index.
    READ = 4,
    /// Discard a single discard block at given block index.
    DISCARD = 5,
    /// Write a single write block at given block index.
    WRITE = 6,
}

impl TryFrom<usize> for Command {
    type Error = ();
    fn try_from(v: usize) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Command::CHECK),
            1 => Ok(Command::SIZE),
            2 => Ok(Command::GEOMETRY),
            3 => Ok(Command::READ_RANGE),
            4 => Ok(Command::READ),
            5 => Ok(Command::DISCARD),
            6 => Ok(Command::WRITE),
            _ => Err(()),
        }
    }
}

/// Ids for read-only allow buffers
mod ro_allow {
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: usize = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: usize = 1;
}

enum Upcall {
    READ = 0,
    DISCARD = 1,
    WRITE = 2,
}

const UPCALL_COUNT: usize = 3;

/// Stores the state of an operation in flight
#[derive(Clone, Copy)]
enum Operation {
    None,
    Requested(ProcessId),
}

#[derive(Clone, Copy)]
struct State {
    read: Operation,
    discard: Operation,
    write: Operation,
}

/// Userspace interface for `hil::block_storage::Storage`.
///
/// Supports only one command of a given type at a time
/// (but may support only one command in flight at all,
/// if that's what the underlying device does).
///
/// Requires a bounce buffer of size at least `W` bytes.
///
/// `W` is the size of a write block, `E` is the discard block size.
pub struct BlockStorage<'a, T, const W: usize, const E: usize>
where
    T: hil::block_storage::Storage<W, E>,
{
    /// The underlying physical storage device.
    device: &'a T,
    /// Per-app state.
    apps: Grant<
        (),
        UpcallCount<UPCALL_COUNT>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    state: Cell<State>,
    /// Bounce buffer because using userspace buffers for DMA is impossible
    buffer: TakeCell<'static, [u8]>,
}

impl<T, const W: usize, const E: usize> BlockStorage<'static, T, W, E>
where
    T: hil::block_storage::Storage<W, E>,
{
    pub fn new(
        device: &'static T,
        grant: Grant<
            (),
            UpcallCount<UPCALL_COUNT>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        buffer: &'static mut [u8],
    ) -> Self {
        Self {
            device,
            apps: grant,
            state: Cell::new(State {
                read: Operation::None,
                discard: Operation::None,
                write: Operation::None,
            }),
            buffer: TakeCell::new(buffer),
        }
    }

    fn start_read(&self, region: &BlockIndex<W>, appid: ProcessId) -> Result<(), ErrorCode> {
        let state = self.state.get();
        match state.read {
            Operation::Requested(..) => Err(ErrorCode::BUSY),
            Operation::None => self.buffer.take().map_or_else(
                || Err(ErrorCode::NOMEM),
                |buffer| match self.device.read(region, buffer) {
                    Ok(()) => {
                        self.state.set(State {
                            read: Operation::Requested(appid),
                            ..state
                        });
                        Ok(())
                    }
                    Err((e, buf)) => {
                        self.buffer.replace(buf);
                        Err(e)
                    }
                },
            ),
        }
    }

    fn start_discard(&self, region: &BlockIndex<E>, appid: ProcessId) -> Result<(), ErrorCode> {
        let state = self.state.get();
        match state.discard {
            Operation::Requested(..) => Err(ErrorCode::BUSY),
            Operation::None => self.device.discard(region).map(|()| {
                self.state.set(State {
                    discard: Operation::Requested(appid),
                    ..state
                });
            }),
        }
    }

    fn start_write(&self, region: &BlockIndex<W>, app_id: ProcessId) -> Result<(), ErrorCode> {
        let state = self.state.get();
        match state.write {
            Operation::Requested(..) => Err(ErrorCode::BUSY),
            Operation::None => self.buffer.take().map_or_else(
                || Err(ErrorCode::NOMEM),
                |buffer| {
                    let ret = self.apps.enter(app_id, |_, kernel_data| {
                        kernel_data
                            .get_readonly_processbuffer(ro_allow::WRITE)
                            .and_then(|write| {
                                write.enter(|app_buffer| {
                                    let write_len = cmp::min(app_buffer.len(), W as usize);

                                    let app_buffer = &app_buffer[0..(write_len)];
                                    app_buffer.copy_to_slice(&mut buffer[0..write_len]);
                                })
                            })
                    });
                    let ret = match ret {
                        // Failed to enter
                        Err(e) => Err((ErrorCode::from(e), buffer)),
                        // Failed to get buffer
                        Ok(Err(e)) => Err((e.into(), buffer)),
                        Ok(Ok(())) => match self.device.write(region, buffer) {
                            Ok(()) => {
                                self.state.set(State {
                                    write: Operation::Requested(app_id),
                                    ..state
                                });
                                Ok(())
                            }
                            e => e,
                        },
                    };
                    ret.map_err(|(e, buf)| {
                        self.buffer.replace(buf);
                        e
                    })
                },
            ),
        }
    }
}

impl<T, const W: usize, const E: usize> hil::block_storage::ReadableClient
    for BlockStorage<'_, T, W, E>
where
    T: hil::block_storage::Storage<W, E>,
{
    fn read_complete(&self, read_buffer: &'static mut [u8], ret: Result<(), ErrorCode>) {
        let state = self.state.get();
        match state.read {
            Operation::Requested(app_id) => {
                self.apps
                    .enter(app_id, move |_, kernel_data| {
                        let ret = match ret {
                            Ok(()) => {
                                // Need to copy in the contents of the buffer
                                kernel_data
                                    .get_readwrite_processbuffer(rw_allow::READ)
                                    .and_then(|read| {
                                        read.mut_enter(|app_buffer| {
                                            let read_len = cmp::min(app_buffer.len(), W as usize);

                                            let d = &app_buffer[0..(read_len)];
                                            d.copy_from_slice(&read_buffer[0..read_len]);
                                        })
                                    })
                                    .map_err(ErrorCode::from)
                            }
                            Err(e) => Err(e),
                        };

                        // Replace the buffer we used to do this read.
                        self.buffer.replace(read_buffer);
                        self.state.set(State {
                            read: Operation::None,
                            ..state
                        });

                        // And then signal the app.
                        let upcall_data = ret.map_or_else(|e| (1, e.into(), 0), |()| (0, 0, 0));
                        kernel_data
                            .schedule_upcall(Upcall::READ as usize, upcall_data)
                            .unwrap_or_else(|e| kernel::debug!("Can't upcall: {:?}", e))
                    })
                    .unwrap_or_else(|e| kernel::debug!("Can't get grant: {:?}", e))
            }
            _ => kernel::debug!("Unexpected read reply"),
        }
    }
}

impl<T, const W: usize, const E: usize> hil::block_storage::WriteableClient
    for BlockStorage<'_, T, W, E>
where
    T: hil::block_storage::Storage<W, E>,
{
    /// Block write complete.
    ///
    /// This will be called when the write operation is complete.
    fn write_complete(&self, write_buffer: &'static mut [u8], ret: Result<(), ErrorCode>) {
        let state = self.state.get();
        match state.write {
            Operation::Requested(app_id) => {
                self.apps
                    .enter(app_id, move |_, kernel_data| {
                        self.state.set(State {
                            write: Operation::None,
                            ..state
                        });
                        self.buffer.replace(write_buffer);

                        // And then signal the app.
                        let upcall_data = ret.map_or_else(|e| (1, e.into(), 0), |()| (0, 0, 0));
                        kernel_data
                            .schedule_upcall(Upcall::WRITE as usize, upcall_data)
                            .unwrap_or_else(|e| kernel::debug!("Can't upcall: {:?}", e))
                    })
                    .unwrap_or_else(|e| kernel::debug!("Can't get grant: {:?}", e))
            }
            _ => kernel::debug!("Unexpected read reply"),
        }
    }

    /// Block discard complete.
    ///
    /// This will be called when the discard operation is complete.
    fn discard_complete(&self, ret: Result<(), ErrorCode>) {
        let state = self.state.get();
        match state.discard {
            Operation::Requested(app_id) => {
                self.apps
                    .enter(app_id, move |_, kernel_data| {
                        self.state.set(State {
                            discard: Operation::None,
                            ..state
                        });

                        // And then signal the app.
                        let upcall_data = ret.map_or_else(|e| (1, e.into(), 0), |()| (0, 0, 0));
                        kernel_data
                            .schedule_upcall(Upcall::DISCARD as usize, upcall_data)
                            .unwrap_or_else(|e| kernel::debug!("Can't upcall: {:?}", e))
                    })
                    .unwrap_or_else(|e| kernel::debug!("Can't get grant: {:?}", e))
            }
            _ => kernel::debug!("Unexpected read reply"),
        }
    }
}

impl<T, const W: usize, const E: usize> SyscallDriver for BlockStorage<'static, T, W, E>
where
    T: hil::block_storage::Storage<W, E>,
{
    fn command(
        &self,
        command_num: usize,
        offset: usize,
        _length: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        kernel::debug!("cmd {} of {} len {}", command_num, offset, _length);
        match Command::try_from(command_num) {
            Ok(Command::CHECK) => CommandReturn::success(),
            Ok(Command::SIZE) => CommandReturn::success_u64(self.device.get_size()),
            Ok(Command::GEOMETRY) => CommandReturn::success_u32_u32(W as u32, E as u32),
            Ok(Command::READ_RANGE) => CommandReturn::failure(ErrorCode::NOSUPPORT),
            Ok(Command::READ) => self.start_read(&BlockIndex(offset as u32), appid).into(),
            Ok(Command::DISCARD) => self.start_discard(&BlockIndex(offset as u32), appid).into(),
            Ok(Command::WRITE) => self.start_write(&BlockIndex(offset as u32), appid).into(),
            Err(()) => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
