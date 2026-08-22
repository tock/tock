// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Capsule implementing the IPC Relay Request mechanism.
//!
//! This capsule enables client-to-server request-and-response messages. Sending
//! a request copies data from the ro_allow buffer of the client to the rw_allow
//! buffer of the server. The server sends a response which copies data from the
//! ro_allow buffer of the server to the rw_allow buffer of the client,
//! completing the transaction.
//!
//! Clients can only have one outstanding request, which may complete
//! successfully, error, or be canceled. Servers wait for requests, and must
//! respond before receiving the next request. Servers do not wait on clients;
//! if a response can not be handled immediately, it is instead dropped.
//!
//! Clients must be aware of the IPC ID of the server they wish to communicate
//! with. This could possibly come from an IPC Registry capsule, or another
//! mechanism. Servers receive the IPC ID of the client who sent them a request,
//! and may cache that IPC ID for later communication.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! let ipc_relay_request = components::ipc::ipc_relay_request::IpcRelayRequestComponent::new(
//!     board_kernel,
//!     capsules_core::ipc::ipc_relay_request::DRIVER_NUM,
//!     create_capability!(capabilities::MemoryAllocationCapability),
//! ).finalize(components::ipc_relay_request_component_static!());
//! ```

use crate::ipc::ipc_identifier::IpcIdentifier;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
pub const DRIVER_NUM: usize = crate::driver::NUM::IpcRelayRequest as usize;

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

/// Enum for tracking state of in-progress transactions
#[derive(Default)]
enum TransactionState {
    #[default]
    None,
    ClientTransaction(IpcIdentifier),
    ServerTransaction(IpcIdentifier),
}

/// Per-process metadata
#[derive(Default)]
pub struct App {
    transaction: TransactionState,
    requests_enabled: bool,
}

/// IPC Relay Request capsule
///
/// This capsule allows for single-copy allow-to-allow communication via
/// request-and-response transactions. Clients send requests to a server and
/// wait for a response. Servers handle one request at a time, sending responses
/// when complete.
pub struct IpcRelayRequest {
    /// Grant memory
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<{ ro_allow::COUNT }>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
}

impl IpcRelayRequest {
    /// Create a new IPC Relay Request capsule
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
        // Check if any transaction is active, and fail if so
        self.apps
            .enter(processid, |app, _| match app.transaction {
                TransactionState::None => Ok(()),
                _ => Err(ErrorCode::ALREADY), // transaction in progress
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
                self.apps
                    .enter(cntr.processid(), |server_app, server_kerneldata| {
                        // Check if the server is accepting requests
                        if server_app.requests_enabled {
                            let client_ipc_id = IpcIdentifier::new_from_processid(processid);
                            // upcall arguments-> ipc_id_lower: u32, ipc_id_upper: u32
                            let _ = server_kerneldata.schedule_upcall(
                                upcall::SERVER_REQUEST_WAITING,
                                (
                                    client_ipc_id.lower() as usize,
                                    client_ipc_id.upper() as usize,
                                    0,
                                ),
                            );
                            Ok(())
                        } else {
                            // Server isn't accepting requests. Return error to client
                            Err(ErrorCode::UNINSTALLED)
                        }
                    })??;

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
        //
        // Warning: there is no way for a client to totally rely on the server
        // responding. The server could ignore the request, could drop the
        // request by mistake, or could fault/terminate and never be able to
        // respond. Clients should have a timeout and cancel the request if
        // necessary.
        //
        // If there was a way for capsules to be aware of process state changes,
        // we could use that callback to check for any in-progress transactions
        // with that process and complete them with errors. That would not solve
        // the other failure cases though, so clients would still need a timeout.
        self.apps.enter(processid, |app, _| {
            app.transaction = TransactionState::ClientTransaction(server_ipc_id);
        })?;

        // Return status
        Ok(())
    }

    fn cancel_request(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps.enter(processid, |app, kerneldata| {
            // Check if there is a transaction to cancel
            match app.transaction {
                TransactionState::ClientTransaction(_) => {
                    // Clear the transaction
                    app.transaction = TransactionState::None;

                    // Also send a response received upcall with an error condition
                    // upcall arguments-> status: StatusCode
                    let _ = kerneldata.schedule_upcall(
                        upcall::CLIENT_RESPONSE_RECEIVED,
                        (ErrorCode::CANCEL.into(), 0, 0),
                    );

                    // No need to clear any transaction in the server
                    //
                    // If the server doesn't have a transaction in progress with
                    // this client, we're good anyways. When it tries to get the
                    // request, it won't find one. If the client does have a
                    // transaction in progress with this client, it'll receive
                    // an error that the client isn't available when it tries to
                    // send the response and it can move on.
                    Ok(())
                }
                _ => Err(ErrorCode::INVAL),
            }
        })??;

        // Return status
        Ok(())
    }

    // Enable requests to this server (disabled by default)
    fn enable_requests(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps.enter(processid, |app, _| {
            app.requests_enabled = true;
        })?;

        // Return status
        Ok(())
    }

    // Disable requests to this server. This clears any ongoing server
    // transaction and errors out any outstanding client requests.
    fn disable_requests(&self, processid: ProcessId) -> Result<(), ErrorCode> {
        self.apps.enter(processid, |app, _| {
            app.requests_enabled = false;
            app.transaction = TransactionState::None;
        })?;

        // Complete any outstanding client requests with errors
        for cntr in self.apps.iter() {
            // skip this process
            if cntr.processid() != processid {
                self.apps
                    .enter(cntr.processid(), |client_app, client_kerneldata| {
                        if let TransactionState::ClientTransaction(ipc_id) = client_app.transaction
                            && ipc_id == IpcIdentifier::new_from_processid(processid)
                        {
                            client_app.transaction = TransactionState::None;

                            // upcall arguments-> status: StatusCode
                            let _ = client_kerneldata.schedule_upcall(
                                upcall::CLIENT_RESPONSE_RECEIVED,
                                (ErrorCode::UNINSTALLED.into(), 0, 0),
                            );
                        }
                    })?;
            }
        }

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
    ) -> Result<Result<(u32, u64), (ErrorCode, u64)>, ErrorCode> {
        // Attempt the copy from client to server
        match self.copy_app_data(client_processid, server_processid) {
            // Copy succeeded
            Ok((data_len, client_len_longer)) => {
                // Get IpcIdentifier for client
                let client_ipc_id = IpcIdentifier::new_from_processid(client_processid);

                // Mark that this server transaction is in progress
                self.apps.enter(server_processid, |app, _| {
                    app.transaction = TransactionState::ServerTransaction(client_ipc_id);
                })?;

                // Get u64 encoding of client IPCIdentifier
                let ipc_id_value: u64 = client_ipc_id.into();

                // Return values to userspace
                // Success returns data length and client IpcIdentifier
                // If there was insufficient buffer space, size doesn't matter (it's full)
                if !client_len_longer {
                    // Successful copy of data
                    Ok(Ok((data_len as u32, ipc_id_value)))
                } else {
                    // Received data from client, but client data was larger than
                    // server buffer could hold. Return the client IpcIdentifier
                    Ok(Err((ErrorCode::SIZE, ipc_id_value)))
                }
            }
            // Copy failed, don't start a transaction in this case
            Err(errorcode) => Err(errorcode),
        }
    }

    fn get_any_next_request(
        &self,
        processid: ProcessId,
    ) -> Result<Result<(u32, u64), (ErrorCode, u64)>, ErrorCode> {
        // Check if any transaction is active, and fail if so
        self.apps
            .enter(processid, |app, _| match app.transaction {
                TransactionState::None => Ok(()),
                _ => Err(ErrorCode::ALREADY),
            })??;

        // Iterate client apps looking for a transaction in progress with this
        // app as a server destination
        // TODO: this should really be a round-robin iteration of clients for fairness... I have a design for that
        let mut client: Option<ProcessId> = None;
        for cntr in self.apps.iter() {
            // skip this process
            if cntr.processid() != processid {
                self.apps.enter(cntr.processid(), |client_app, _| {
                    // look for client with transaction active for this server
                    if let TransactionState::ClientTransaction(ipc_id) = &client_app.transaction
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
            Err(ErrorCode::NODEVICE)
        }
    }

    fn get_specific_next_request(
        &self,
        processid: ProcessId,
        client_ipc_id: IpcIdentifier,
    ) -> Result<Result<(u32, u64), (ErrorCode, u64)>, ErrorCode> {
        // Check if any transaction is active, and fail if so
        self.apps
            .enter(processid, |app, _| match app.transaction {
                TransactionState::None => Ok(()),
                _ => Err(ErrorCode::ALREADY),
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
                    if let TransactionState::ClientTransaction(ipc_id) = &client_app.transaction
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
            Err(ErrorCode::NODEVICE)
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
                        client_app.transaction = TransactionState::None;

                        if !server_len_longer {
                            // upcall arguments-> status: StatusCode, data_len: usize
                            let _ = client_kerneldata.schedule_upcall(
                                upcall::CLIENT_RESPONSE_RECEIVED,
                                (0, data_len, 0),
                            );
                        } else {
                            // upcall arguments-> status: StatusCode, data_len: usize
                            let _ = client_kerneldata.schedule_upcall(
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
                        client_app.transaction = TransactionState::None;

                        // upcall arguments-> status: StatusCode
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
        // or fail if no server transaction is in progress
        let client_ipc_id = self
            .apps
            .enter(processid, |app, _| match app.transaction {
                TransactionState::ServerTransaction(ipc_id) => {
                    // Clear server transaction since we're attempting a response
                    app.transaction = TransactionState::None;
                    Ok(ipc_id)
                }
                _ => Err(ErrorCode::INVAL), // Either ClientTransaction or None
            })??;

        // Check that client_ipc_id is a valid app
        let mut client: Option<ProcessId> = None;
        for cntr in self.apps.iter() {
            // skip this process, look for matching ipc_id
            if cntr.processid() != processid
                && IpcIdentifier::new_from_processid(cntr.processid()) == client_ipc_id
            {
                // Found the client, need to check if they have a transaction with us still
                self.apps.enter(cntr.processid(), |client_app, _| {
                    if let TransactionState::ClientTransaction(transaction_ipc_id) =
                        client_app.transaction
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
    }
}

impl SyscallDriver for IpcRelayRequest {
    /// IPC Relay Request mechanism
    ///
    /// Allows requests to be sent from clients to servers, with paired
    /// responses sent from server back to client.
    ///
    /// Commands are split into client-focused and server-focused. A single
    /// process can act as both a client and a server at different times.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Check driver presence
    /// - `0x10`: For clients, send request to process
    /// - `0x11`: For clients, cancel request
    /// - `0x20`: For servers, enable requests
    /// - `0x21`: For servers, disable requests
    /// - `0x22`: For servers, get any next request
    /// - `0x23`: For servers, get next request from process
    /// - `0x24`: For servers, send response
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        let ipc_id = IpcIdentifier::new_from_halves(data1 as u32, data2 as u32);

        match command_num {
            // Check existence
            0 => CommandReturn::success(),

            // --- Client Commands ---
            // Send request
            0x10 => self.send_request(processid, ipc_id).into(),

            // Cancel request
            0x11 => self.cancel_request(processid).into(),

            // --- Server Commands ---
            // Enable requests
            0x20 => self.enable_requests(processid).into(),

            // Disable requests
            0x21 => self.disable_requests(processid).into(),

            // Get next request
            0x22 => match self.get_any_next_request(processid) {
                // Success case
                Ok(Ok((data_len, ipc_id_val))) => {
                    CommandReturn::success_u32_u64(data_len, ipc_id_val)
                }

                // Partial failure. Truncated response
                Ok(Err((err, ipc_id_val))) => CommandReturn::failure_u64(err, ipc_id_val),

                // Failure case
                Err(err) => CommandReturn::failure_u64(err, 0),
            },

            // Get next request from IPC ID
            0x23 => match self.get_specific_next_request(processid, ipc_id) {
                // Success case
                Ok(Ok((data_len, ipc_id_val))) => {
                    CommandReturn::success_u32_u64(data_len, ipc_id_val)
                }

                // Partial failure. Truncated response
                Ok(Err((err, ipc_id_val))) => CommandReturn::failure_u64(err, ipc_id_val),

                // Failure case
                Err(err) => CommandReturn::failure_u64(err, 0),
            },

            // Send response
            0x24 => self.send_response(processid).into(),

            // Default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
