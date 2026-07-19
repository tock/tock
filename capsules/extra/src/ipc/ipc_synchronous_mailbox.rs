// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Capsule implementing the IPC synchronous mailbox mechanism.
//!
//! This capsule enables client-to-server request-and-response messages. Sending
//! a request copies data from the ro_allow buffer of the client to the rw_allow
//! buffer of the server. The server sends a response which copies data from the
//! ro_allow buffer of the server to the rw_allow buffer of the client,
//! completing the transaction.
//!
//! Clients can only have one outstanding request, which may be complete,
//! error, or be canceled. Servers wait for requests, and must respond
//! before receiving the next request. Servers do not wait on clients; if a
//! response can not be handled immediately, it is instead dropped.
//!
//! Clients must be aware of the ID of the server they wish to communicate
//! with. This could possibly come from an IPC Registry capsule, or another
//! mechanism. Servers receive the ID of the client who sent them a request,
//! and may cache that ID for later communication.
//!
//! TODO add example of how to instantiate

use kernel::debug;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use capsules_core::driver;

use crate::ipc::ipc_identifier::IpcIdentifier;
pub const DRIVER_NUM: usize = driver::NUM::IpcSynchronousMailbox as usize;

/// Ids for read-only allow buffers
mod ro_allow {
    pub const READ_BUFFER: usize = 0;
    /// The number of read-only allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// Ids for read-write allow buffers
mod rw_allow {
    pub const WRITE_BUFFER: usize = 0;
    /// The number of read-write allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

/// IDs for subscribed upcalls.
mod upcall {
    /// For clients, subscribe to response received callback.
    pub const CLIENT_RESPONSE_RECEIVED: usize = 0;
    /// For servers, subscribe to request waiting callback.
    pub const SERVER_REQUEST_WAITING: usize = 1;
    /// Number of upcalls.
    pub const COUNT: u8 = 2;
}

/// Per-process metadata
//TODO: remove debug here
#[derive(Default, Debug)]
pub struct App {
    client_transaction: Option<IpcIdentifier>,
    server_transaction: Option<IpcIdentifier>,
}

// TODO: document all of this with doc comments

pub struct IpcSynchronousMailbox {
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
}

impl IpcSynchronousMailbox {
    pub fn new(
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<{ ro_allow::COUNT }>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
    ) -> Self {
        Self { apps: grant }
    }

    fn send_request(
        &self,
        processid: ProcessId,
        server_ipc_id: IpcIdentifier,
    ) -> Result<(), ErrorCode> {
        // Check if a client transaction is active, and fail if so
        self.apps
            .enter(processid, |app, _| match app.client_transaction {
                Some(_) => Err(ErrorCode::ALREADY),
                None => Ok(()),
            })??;

        // Search apps for server_id_num
        let mut found = false;
        for cntr in self.apps.iter() {
            // skip this process, look for matching ipc_id
            if cntr.processid() != processid
                && IpcIdentifier::new_from_processid(cntr.processid()) == server_ipc_id
            {
                // Found the server
                found = true;

                // Send request-waiting upcall to destination with client ID
                self.apps.enter(cntr.processid(), |_, server_kerneldata| {
                    let client_ipc_id = IpcIdentifier::new_from_processid(processid);
                    debug!("KERNEL: sending upcall to {}", cntr.processid().id());
                    let _ = server_kerneldata.schedule_upcall(
                        upcall::SERVER_REQUEST_WAITING,
                        (
                            client_ipc_id.lower() as usize,
                            client_ipc_id.upper() as usize,
                            0,
                        ),
                    );
                })?;

                // There won't be another match, so exit early
                break;
            }
        }

        // If we didn't find it, either the ID was invalid or the server hasn't
        // registered with this capsule yet. Either way, we can't communicate
        // with it
        if !found {
            return Err(ErrorCode::NODEVICE);
        }

        // Mark that a client transaction is in progress, do this only if prior
        // work succeeds
        self.apps.enter(processid, |app, _| {
            app.client_transaction = Some(server_ipc_id);
        })?;

        // Return status
        Ok(())
    }

    fn cancel_request(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps.enter(processid, |app, _| {
            // Check if a client transaction is active, and fail if not
            match app.client_transaction {
                Some(_) => Ok(()),
                None => Err(ErrorCode::ALREADY),
            }
        })??;

        // Remove any in-progress server transaction with this client
        //
        // Note: a server may end up with a spurious request-waiting upcall in
        // this case, but that's not any different from getting an upcall for a
        // process that then dies. In all cases there may be no request to get.
        // Similarly, a server may end up sending a response that is no longer
        // being waited for, in which case it'll just be dropped.
        for cntr in self.apps.iter() {
            // skip this process
            if cntr.processid() != processid {
                let mut found = false;
                self.apps.enter(cntr.processid(), |app, _| {
                    if let Some(ipc_id) = &app.server_transaction
                        && *ipc_id == IpcIdentifier::new_from_processid(processid)
                    {
                        app.server_transaction = None;
                        found = true;
                    }
                })?;

                // There won't be another match, so exit early
                if found {
                    break;
                }
            }
        }

        // Mark that no client transaction is in progress, do this only if prior
        // work succeeds
        self.apps.enter(processid, |app, _| {
            app.client_transaction = None;
        })?;

        // Return status
        Ok(())
    }

    // Copy bytes up to size of buffers from source to destination.
    //
    // Having insufficient buffer space for all the source data will copy up to
    // destination buffer size, but will track the error to inform the
    // destination later.
    //
    // It is acceptable for buffers of length zero to be used.
    fn copy_app_data(
        &self,
        src_processid: ProcessId,
        dst_processid: ProcessId,
    ) -> Result<(usize, bool), ErrorCode> {
        // Error if src and dst are identical
        if src_processid == dst_processid {
            return Err(ErrorCode::INVAL);
        }

        // Track status
        let mut data_len = 0;
        let mut src_len_longer = false;

        // Get src buffer
        self.apps.enter(src_processid, |_, src_kerneldata| {
            src_kerneldata
                .get_readonly_processbuffer(ro_allow::READ_BUFFER)
                .and_then(|src_allow| {
                    src_allow.enter(|src_buf| {
                        // Get dst buffer
                        self.apps.enter(dst_processid, |_, dst_kerneldata| {
                            dst_kerneldata
                                .get_readwrite_processbuffer(rw_allow::WRITE_BUFFER)
                                .and_then(|dst_allow| {
                                    dst_allow.mut_enter(|dst_buf| {
                                        // Get minimum length
                                        data_len = core::cmp::min(src_buf.len(), dst_buf.len());

                                        // Track if src length was longer than dst length
                                        if src_buf.len() > dst_buf.len() {
                                            src_len_longer = true;
                                        }

                                        // Iterate and copy byte-by-byte up to length
                                        src_buf[0..data_len]
                                            .iter()
                                            .zip(dst_buf[0..data_len].iter())
                                            .for_each(|(src_byte, dst_byte)| {
                                                dst_byte.set(src_byte.get())
                                            });
                                    })
                                })
                        })
                    })
                })
        })????;

        // Return status
        Ok((data_len, src_len_longer))
    }

    fn handle_request_copy(
        &self,
        server_processid: ProcessId,
        client_processid: ProcessId,
    ) -> Result<CommandReturn, ErrorCode> {
        // Attempt the copy from client to server
        match self.copy_app_data(client_processid, server_processid) {
            // Copy succeeded
            Ok((data_len, client_len_longer)) => {
                // Get IpcIdentifier for client
                let client_ipc_id = IpcIdentifier::new_from_processid(client_processid);

                // Mark that this server transaction is in progress
                self.apps.enter(server_processid, |app, _| {
                    app.server_transaction = Some(client_ipc_id);
                })?;

                // Get u64 encoding of client IPCIdentifier
                let ipc_id_value: u64 = client_ipc_id.into();

                // Return values to userspace
                // Success returns data length and client IpcIdentifier
                // If there was insufficient buffer space, size doesn't matter (it's full)
                if !client_len_longer {
                    // Successful copy of data
                    Ok(CommandReturn::success_u32_u64(
                        data_len as u32,
                        ipc_id_value,
                    ))
                } else {
                    // Received data from client, but client data was larger than
                    // server buffer could hold. Return the client IpcIdentifier
                    Ok(CommandReturn::failure_u64(ErrorCode::SIZE, ipc_id_value))
                }
            }
            // Copy failed, don't start a transaction in this case
            Err(errorcode) => Err(errorcode),
        }
    }

    fn get_any_next_request(&self, processid: ProcessId) -> Result<CommandReturn, ErrorCode> {
        // Check if a server transaction is active, and fail if so
        self.apps
            .enter(processid, |app, _| match app.server_transaction {
                Some(_) => Err(ErrorCode::ALREADY),
                None => Ok(()),
            })??;

        // Iterate client apps looking for a transaction in progress with this
        // app as a server destination
        // TODO: this should really be a round-robin iteration... I have a
        // design for that
        let mut client: Option<ProcessId> = None;
        for cntr in self.apps.iter() {
            // skip this process
            if cntr.processid() != processid {
                self.apps.enter(cntr.processid(), |client_app, _| {
                    // look for client with transaction active for this server
                    if let Some(ipc_id) = &client_app.client_transaction
                        && *ipc_id == IpcIdentifier::new_from_processid(processid)
                    {
                        // Found it!
                        client = Some(cntr.processid());
                    }
                })?;
            }
        }

        if let Some(client_processid) = client {
            // Found request. Attempt the data copy
            self.handle_request_copy(processid, client_processid)
        } else {
            // No app had a request.
            // This isn't really a failure at all. Userspace can ignore it.
            // But importantly, there is no data in the buffer to read.
            Ok(CommandReturn::failure_u64(ErrorCode::NODEVICE, 0))
        }
    }

    fn get_specific_next_request(
        &self,
        processid: ProcessId,
        client_ipc_id: IpcIdentifier,
    ) -> Result<CommandReturn, ErrorCode> {
        // Check if a server transaction is active, and fail if so
        self.apps
            .enter(processid, |app, _| match app.server_transaction {
                Some(_) => Err(ErrorCode::ALREADY),
                None => Ok(()),
            })??;

        // Check specific app for a client transaction active with this app as a
        // server
        let mut client: Option<ProcessId> = None;
        for cntr in self.apps.iter() {
            // skip this process and any process except for the specified client
            if cntr.processid() != processid
                && IpcIdentifier::new_from_processid(cntr.processid()) == client_ipc_id
            {
                self.apps.enter(cntr.processid(), |client_app, _| {
                    // look for client with transaction active for this server
                    if let Some(ipc_id) = &client_app.client_transaction
                        && *ipc_id == IpcIdentifier::new_from_processid(processid)
                    {
                        // Found it!
                        client = Some(cntr.processid());
                    }
                })?;

                // This was the specified client, so no need to search further
                break;
            }
        }

        if let Some(client_processid) = client {
            // Found request. Attempt the data copy
            self.handle_request_copy(processid, client_processid)
        } else {
            // No app had a request.
            // This isn't really a failure at all. Userspace can ignore it.
            // But importantly, there is no data in the buffer to read.
            Ok(CommandReturn::failure_u64(ErrorCode::NODEVICE, 0))
        }
    }

    fn handle_response_copy(
        &self,
        server_processid: ProcessId,
        client_processid: ProcessId,
    ) -> Result<(), ErrorCode> {
        // Attempt the copy from server to client
        match self.copy_app_data(server_processid, client_processid) {
            // Copy succeeded
            Ok((data_len, server_len_longer)) => {
                // Mark that this client transaction is completed and send upcall
                self.apps
                    .enter(client_processid, |client_app, client_kerneldata| {
                        client_app.client_transaction = None;

                        if !server_len_longer {
                            let _ = client_kerneldata.schedule_upcall(
                                // status, data length
                                upcall::CLIENT_RESPONSE_RECEIVED,
                                (0, data_len, 0),
                            );
                        } else {
                            let _ = client_kerneldata.schedule_upcall(
                                // status, data length
                                upcall::CLIENT_RESPONSE_RECEIVED,
                                (ErrorCode::SIZE.into(), data_len, 0),
                            );
                        }
                    })?;

                Ok(())
            }
            // Copy failed, we gave it an honest effort so clear the client transaction
            Err(errorcode) => {
                // Mark the client transaction as completed and send error upcall
                self.apps
                    .enter(client_processid, |client_app, client_kerneldata| {
                        client_app.client_transaction = None;

                        // status, data length
                        let _ = client_kerneldata.schedule_upcall(
                            upcall::CLIENT_RESPONSE_RECEIVED,
                            (ErrorCode::FAIL.into(), 0, 0),
                        );
                    })?;

                Err(errorcode)
            }
        }
    }

    fn send_response(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        // Note: a server may only respond to the currently active transaction
        // and may not activate a new transaction until the current one has been
        // responded to

        // Check if a server transaction is in progress and get client,
        // and fail if not
        let mut transaction: Option<IpcIdentifier> = None;
        self.apps.enter(processid, |app, _| {
            // Save transaction and clear it
            transaction = app.server_transaction;
            app.server_transaction = None;
        })?;

        if let Some(client_ipc_id) = transaction {
            debug!(
                "KERNEL: sending response to client {}",
                Into::<u64>::into(client_ipc_id)
            );
            // Check that client_ipc_id is a valid app
            let mut client: Option<ProcessId> = None;
            for cntr in self.apps.iter() {
                // skip this process, look for matching ipc_id
                if cntr.processid() != processid
                    && IpcIdentifier::new_from_processid(cntr.processid()) == client_ipc_id
                {
                    // Found the client, need to check if they have a transaction
                    // with us still
                    self.apps.enter(cntr.processid(), |client_app, _| {
                        debug!(
                            "KERNEL: found client, they have transaction: {:?}",
                            client_app.client_transaction
                        );
                        if let Some(transaction_ipc_id) = client_app.client_transaction
                            && transaction_ipc_id == IpcIdentifier::new_from_processid(processid)
                        {
                            // Target has a transaction with us! Ready to do the copy
                            client = Some(cntr.processid());
                        }
                    })?;

                    // We found the client, so no need to search further
                    break;
                }
            }

            if let Some(client_processid) = client {
                // Found response target. Attempt the data copy
                // Also clears client transaction and sends upcall to client if successful
                self.handle_response_copy(processid, client_processid)
            } else {
                // Client transaction was gone? Maybe it was canceled.
                // This isn't really a failure at all. Userspace can ignore it.
                Err(ErrorCode::NODEVICE)
            }
        } else {
            // No transaction active
            Err(ErrorCode::INVAL)
        }
    }

    // TODO: need a way to know if any process state changes, if so we should
    // check the apps list for any transaction in progress with the former
    // process as a destination. The client for that transaction should receive
    // an response-received upcall with an errorcode and should be marked as
    // transaction completed
}

impl SyscallDriver for IpcSynchronousMailbox {
    /// Synchronous mailbox IPC mechanism
    ///
    /// Commands are split into client-focused and server-focused. A single
    /// process can act as both a client and a server at different times.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Check driver presence
    /// - `1`: For clients, send request to process
    /// - `2`: For clients, cancel request
    /// - `3`: For servers, get any next request
    /// - `4`: For servers, get next request from process
    /// - `5`: For servers, send response
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        debug!(
            "KERNEL: Got request {} from {}",
            command_num,
            processid.id()
        );

        let ipc_id = IpcIdentifier::new_from_halves(data1 as u32, data2 as u32);

        match command_num {
            0 => CommandReturn::success(),

            // Client
            1 => self.send_request(processid, ipc_id).into(),

            // Client
            2 => self.cancel_request(processid).into(),

            // Server
            // TODO: change these to just use results and do all CommandReturn
            // work out here. That'll ensure I don't use multiple types accidentally
            3 => match self.get_any_next_request(processid) {
                Ok(cmd) => cmd,
                Err(err) => CommandReturn::failure_u64(err, 0),
            },

            // Server
            4 => match self.get_specific_next_request(processid, ipc_id) {
                Ok(cmd) => cmd,
                Err(err) => CommandReturn::failure_u64(err, 0),
            },

            // Server
            5 => self.send_response(processid).into(),

            // Default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
