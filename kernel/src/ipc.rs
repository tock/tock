//! Inter-process communication mechanism for Tock.
//!
//! This is a special syscall driver that allows userspace applications to
//! share memory.

use crate::capabilities::MemoryAllocationCapability;
use crate::grant::Grant;
use crate::process;
use crate::process::ProcessId;
use crate::sched::Kernel;
use crate::{
    CommandReturn, Driver, ErrorCode, ReadOnlyProcessBuffer, ReadWriteProcessBuffer,
    ReadableProcessBuffer,
};

/// Syscall number
pub const DRIVER_NUM: usize = 0x10000;

/// Enum to mark which type of upcall is scheduled for the IPC mechanism.
#[derive(Copy, Clone, Debug)]
pub enum IPCUpcallType {
    /// Indicates that the upcall is for the service upcall handler this
    /// process has setup.
    Service,
    /// Indicates that the upcall is from a different service app and will
    /// call one of the client upcalls setup by this process.
    Client,
}

/// State that is stored in each process's grant region to support IPC.
struct IPCData<const NUM_PROCS: usize> {
    /// An array of process buffers that this application has shared
    /// with other applications.
    shared_memory: [ReadWriteProcessBuffer; NUM_PROCS],
    search_buf: ReadOnlyProcessBuffer,
}

impl<const NUM_PROCS: usize> Default for IPCData<NUM_PROCS> {
    fn default() -> IPCData<NUM_PROCS> {
        const DEFAULT_RW_PROC_BUF: ReadWriteProcessBuffer = ReadWriteProcessBuffer::const_default();
        IPCData {
            shared_memory: [DEFAULT_RW_PROC_BUF; NUM_PROCS],
            search_buf: ReadOnlyProcessBuffer::default(),
        }
    }
}

/// The upcall setup by a service. Each process can only be one service.
/// Subscribe with subscribe_num == 0 is how a process registers
/// itself as an IPC service. Each process can only register as a
/// single IPC service. The identifier for the IPC service is the
/// application name stored in the TBF header of the application.
/// The upcall that is passed to subscribe is called when another
/// process notifies the server process.
const SERVICE_UPCALL_NUM: usize = 0;

/// This const specifies the subscribe_num of the first upcall
/// in the array of client upcalls.
/// Subscribe with subscribe_num >= 1 is how a client registers
/// a upcall for a given service. The service number (passed
/// as subscribe_num) is returned from the allow() call.
/// Once subscribed, the client will receive upcalls when the
/// service process calls notify_client().

const CLIENT_UPCALL_NUM_BASE: usize = 1;

/// The IPC mechanism struct.
/// NUM_UPCALLS should always equal NUM_PROCS + 1. The extra upcall
/// is so processes can register as a service. Once const_evaluatable_checked
/// is stable we will not need two separate const generic parameters.
pub struct IPC<const NUM_PROCS: usize, const NUM_UPCALLS: usize> {
    /// The grant regions for each process that holds the per-process IPC data.
    data: Grant<IPCData<NUM_PROCS>, NUM_UPCALLS>,
}

impl<const NUM_PROCS: usize, const NUM_UPCALLS: usize> IPC<NUM_PROCS, NUM_UPCALLS> {
    pub fn new(
        kernel: &'static Kernel,
        driver_num: usize,
        capability: &dyn MemoryAllocationCapability,
    ) -> Self {
        Self {
            data: kernel.create_grant(driver_num, capability),
        }
    }

    /// Schedule an IPC upcall for a process. This is called by the main
    /// scheduler loop if an IPC task was queued for the process.
    pub(crate) unsafe fn schedule_upcall(
        &self,
        schedule_on: ProcessId,
        called_from: ProcessId,
        cb_type: IPCUpcallType,
    ) -> Result<(), process::Error> {
        self.data
            .enter(schedule_on, |_mydata, my_upcalls| {
                let to_schedule: usize = match cb_type {
                    IPCUpcallType::Service => SERVICE_UPCALL_NUM,
                    IPCUpcallType::Client => match called_from.index() {
                        Some(i) => i + CLIENT_UPCALL_NUM_BASE,
                        None => panic!("Invalid app issued IPC request"), //TODO: return Error instead
                    },
                };
                self.data
                    .enter(called_from, |called_from_data, _called_from_upcalls| {
                        // If the other app shared a buffer with us, make
                        // sure we have access to that slice and then call
                        // the upcall. If no slice was shared then just
                        // call the upcall.
                        match schedule_on.index() {
                            Some(i) => {
                                if i >= called_from_data.shared_memory.len() {
                                    return;
                                }

                                match called_from_data.shared_memory.get(i) {
                                    Some(slice) => {
                                        self.data.kernel.process_map_or(
                                            None,
                                            schedule_on,
                                            |process| {
                                                process.add_mpu_region(
                                                    slice.ptr(),
                                                    slice.len(),
                                                    slice.len(),
                                                )
                                            },
                                        );
                                        my_upcalls.schedule_upcall(
                                            to_schedule,
                                            called_from.id() + 1,
                                            crate::mem::ReadableProcessBuffer::len(slice),
                                            crate::mem::ReadableProcessBuffer::ptr(slice) as usize,
                                        );
                                    }
                                    None => {
                                        my_upcalls.schedule_upcall(
                                            to_schedule,
                                            called_from.id() + 1,
                                            0,
                                            0,
                                        );
                                    }
                                }
                            }
                            None => {}
                        }
                    })
            })
            .and_then(|x| x)
    }
}

impl<const NUM_PROCS: usize, const NUM_UPCALLS: usize> Driver for IPC<NUM_PROCS, NUM_UPCALLS> {
    /// command is how notify() is implemented.
    /// Notifying an IPC service is done by setting client_or_svc to 0,
    /// and notifying an IPC client is done by setting client_or_svc to 1.
    /// In either case, the target_id is the same number as provided in a notify
    /// upcall or as returned by allow.
    ///
    /// Returns INVAL if the other process doesn't exist.

    /// Initiates a service discovery or notifies a client or service.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check, always returns Ok(())
    /// - `1`: Perform discovery on the package name passed to `allow_readonly`. Returns the
    ///        service descriptor if the service is found, otherwise returns an error.
    /// - `2`: Notify a service previously discovered to have the service descriptor in
    ///        `target_id`. Returns an error if `target_id` refers to an invalid service or the
    ///        notify fails to enqueue.
    /// - `3`: Notify a client with descriptor `target_id`, typically in response to a previous
    ///        notify from the client. Returns an error if `target_id` refers to an invalid client
    ///        or the notify fails to enqueue.
    fn command(
        &self,
        command_number: usize,
        target_id: usize,
        _: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            1 =>
            /* Discover */
            {
                self.data
                    .enter(appid, |data, _upcalls| {
                        data.search_buf
                            .enter(|slice| {
                                self.data
                                    .kernel
                                    .process_until(|p| {
                                        let s = p.get_process_name().as_bytes();
                                        // are slices equal?
                                        if s.len() == slice.len()
                                            && s.iter()
                                                .zip(slice.iter())
                                                .all(|(c1, c2)| *c1 == c2.get())
                                        {
                                            Some(CommandReturn::success_u32(
                                                p.processid().id() as u32 + 1,
                                            ))
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(CommandReturn::failure(ErrorCode::NODEVICE))
                            })
                            .unwrap_or(CommandReturn::failure(ErrorCode::INVAL))
                    })
                    .unwrap_or(CommandReturn::failure(ErrorCode::NOMEM))
            }
            2 =>
            /* Service notify */
            {
                let cb_type = IPCUpcallType::Service;
                let app_identifier = target_id - 1;

                self.data
                    .kernel
                    .lookup_app_by_identifier(app_identifier)
                    .map_or(CommandReturn::failure(ErrorCode::INVAL), |otherapp| {
                        self.data.kernel.process_map_or(
                            CommandReturn::failure(ErrorCode::INVAL),
                            otherapp,
                            |target| {
                                let ret = target.enqueue_task(process::Task::IPC((appid, cb_type)));
                                match ret {
                                    true => CommandReturn::success(),
                                    false => CommandReturn::failure(ErrorCode::FAIL),
                                }
                            },
                        )
                    })
            }
            3 =>
            /* Client notify */
            {
                let cb_type = IPCUpcallType::Client;
                let app_identifier = target_id - 1;

                self.data
                    .kernel
                    .lookup_app_by_identifier(app_identifier)
                    .map_or(CommandReturn::failure(ErrorCode::INVAL), |otherapp| {
                        self.data.kernel.process_map_or(
                            CommandReturn::failure(ErrorCode::INVAL),
                            otherapp,
                            |target| {
                                let ret = target.enqueue_task(process::Task::IPC((appid, cb_type)));
                                match ret {
                                    true => CommandReturn::success(),
                                    false => CommandReturn::failure(ErrorCode::FAIL),
                                }
                            },
                        )
                    })
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    /// allow_readonly with subdriver number `0` stores the provided buffer for service discovery.
    /// The buffer should contain the package name of a process that exports an IPC service.
    fn allow_readonly(
        &self,
        appid: ProcessId,
        subdriver: usize,
        mut buffer: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        if subdriver == 0 {
            // Package name for discovery
            let res = self.data.enter(appid, |data, _upcalls| {
                core::mem::swap(&mut data.search_buf, &mut buffer);
            });
            match res {
                Ok(_) => Ok(buffer),
                Err(e) => Err((buffer, e.into())),
            }
        } else {
            Err((buffer, ErrorCode::NOSUPPORT))
        }
    }

    /// allow_readwrite enables processes to discover IPC services on the platform or
    /// share buffers with existing services.
    ///
    /// If allow is called with target_id >= 1, it is a share command where the
    /// application is explicitly sharing a slice with an IPC service (as
    /// specified by the target_id). allow() simply allows both processes to
    /// access the buffer, it does not signal the service.
    ///
    /// target_id == 0 is currently unsupported and reserved for future use.
    fn allow_readwrite(
        &self,
        appid: ProcessId,
        target_id: usize,
        mut buffer: ReadWriteProcessBuffer,
    ) -> Result<ReadWriteProcessBuffer, (ReadWriteProcessBuffer, ErrorCode)> {
        if target_id == 0 {
            Err((buffer, ErrorCode::NOSUPPORT))
        } else {
            match self.data.enter(appid, |data, _upcalls| {
                // Lookup the index of the app based on the passed in
                // identifier. This also let's us check that the other app is
                // actually valid.
                let app_identifier = target_id - 1;
                let otherapp = self.data.kernel.lookup_app_by_identifier(app_identifier);
                if let Some(oa) = otherapp {
                    if let Some(i) = oa.index() {
                        if let Some(smem) = data.shared_memory.get_mut(i) {
                            core::mem::swap(smem, &mut buffer);
                            Ok(())
                        } else {
                            Err(ErrorCode::INVAL)
                        }
                    } else {
                        Err(ErrorCode::INVAL)
                    }
                } else {
                    Err(ErrorCode::BUSY)
                }
            }) {
                Ok(_) => Ok(buffer),
                Err(e) => Err((buffer, e.into())),
            }
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), crate::process::Error> {
        self.data.enter(processid, |_, _| {})
    }
}
