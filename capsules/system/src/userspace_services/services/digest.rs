// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Hashing as a userspace service.

use kernel::errorcode::ErrorCode;
use kernel::hil::digest::{
    self, Client, ClientData, ClientHash, ClientVerify, Digest, DigestData, DigestHash,
    DigestVerify,
};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};

use crate::userspace_services::data::{Bytes, Serialize};
use crate::userspace_services::services::Role;
use crate::userspace_services::usercall::{
    Arguments, ReturnReader, UserspaceServiceAccess, UserspaceServiceClient,
};

mod ops {
    pub const RUN: usize = 0x01;
    pub const ADD_DATA: usize = 0x02;
    pub const VERIFY: usize = 0x03;
    pub const CLEAR_DATA: usize = 0x11;
}

const ROLE_ID: usize = Role::Digest as usize;

enum Operation<const L: usize> {
    AddData(SubSlice<'static, u8>),
    AddMutData(SubSliceMut<'static, u8>),
    Run(&'static mut [u8; L]),
    Verify(&'static mut [u8; L]),
}

const RETURN_HASHDONE_HASH_BUFFER_IDX: usize = 0;

/// Hashing userspace service interface.
pub struct ServiceInterface<const L: usize> {
    /// Self-reference to avoid needing &'static self in HILs.
    this: OptionalCell<&'static dyn UserspaceServiceClient>,
    /// Userspace service access interface.
    userv_access: &'static dyn UserspaceServiceAccess,

    // Clients.
    /// Client using the userspace service for input data addition.
    data_client: OptionalCell<&'static dyn ClientData<L>>,
    /// Client using the userspace service for hash calculation.
    hash_client: OptionalCell<&'static dyn ClientHash<L>>,
    /// Client using the userspace service for hash verification.
    verify_client: OptionalCell<&'static dyn ClientVerify<L>>,

    /// The userspace service's current operation.
    current_op: OptionalCell<Operation<L>>,
}

impl<const L: usize> ServiceInterface<L> {
    /// Create a new instance of the service interface.
    pub fn new(userv_access: &'static dyn UserspaceServiceAccess) -> ServiceInterface<L> {
        ServiceInterface {
            this: OptionalCell::empty(),
            userv_access,
            data_client: OptionalCell::empty(),
            hash_client: OptionalCell::empty(),
            verify_client: OptionalCell::empty(),
            current_op: OptionalCell::empty(),
        }
    }

    /// Initialize internal state necessary before use.
    pub fn init(&'static self) {
        self.this.set(self)
    }
}

impl<const L: usize> UserspaceServiceClient for ServiceInterface<L> {
    fn usercall_done(&self, return_data: Result<ReturnReader<'_>, ErrorCode>) {
        if let Some(op) = self.current_op.take() {
            match op {
                // Provide the client with its buffer back.
                Operation::AddData(data_slice) => {
                    self.data_client
                        .map(|c| c.add_data_done(return_data.map(|_reader| ()), data_slice));
                }

                Operation::AddMutData(data_slice) => {
                    self.data_client
                        .map(|c| c.add_mut_data_done(return_data.map(|_reader| ()), data_slice));
                }

                // Copy the resulting hash into the caller's provided buffer and return it.
                Operation::Run(hash) => {
                    match return_data {
                        Ok(reader) => {
                            let copy_res = reader.result_buffer_n(
                                RETURN_HASHDONE_HASH_BUFFER_IDX,
                                |hash_output_pslice| {
                                    // Copy bytes.
                                    // The run function esures that the caller's buffer is L bytes long.
                                    hash_output_pslice[0..L].copy_to_slice(hash);
                                },
                            );
                            self.hash_client
                                .map(|c| c.hash_done(copy_res.map_err(|kerr| kerr.into()), hash));
                        }

                        Err(_eval) => {
                            self.hash_client
                                .map(|c| c.hash_done(Err(ErrorCode::FAIL), hash));
                        }
                    }
                }

                // Provide the digest output buffer back to the client along with the comparison result.
                Operation::Verify(digest_buffer) => {
                    let verify_result = return_data.map(|reader| reader.direct_rvals().0 == 1);
                    self.verify_client
                        .map(|c| c.verification_done(verify_result, digest_buffer));
                }
            }
        } else {
            // This ServiceInterface called usercall()
            // but did not place an Operation variant into self.current_op.
        }
    }
}

impl<'a: 'static, const L: usize> Digest<'a, L> for ServiceInterface<L> {
    fn set_client(&'a self, client: &'a dyn Client<L>) {
        self.data_client.set(client);
        self.hash_client.set(client);
        self.verify_client.set(client);
    }
}

impl<'a: 'static, const L: usize> DigestData<'a, L> for ServiceInterface<L> {
    fn set_data_client(&'a self, client: &'a dyn ClientData<L>) {
        self.data_client.set(client)
    }

    fn add_data(
        &self,
        data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)> {
        if self.current_op.is_some() {
            Err((ErrorCode::BUSY, data))
        } else {
            if let Some(this) = self.this.get() {
                let usercall_args: [&dyn Serialize; _] = [&Bytes(data.as_slice())];

                let usercall_result = self.userv_access.usercall(
                    this,
                    ROLE_ID,
                    ops::ADD_DATA,
                    Arguments::Extended(0, 0, &usercall_args),
                );
                if let Err(ec) = usercall_result {
                    Err((ec, data))
                } else {
                    self.current_op.set(Operation::AddData(data));
                    Ok(())
                }
            } else {
                Err((ErrorCode::NODEVICE, data))
            }
        }
    }

    fn add_mut_data(
        &self,
        data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        if self.current_op.is_some() {
            Err((ErrorCode::BUSY, data))
        } else {
            if let Some(this) = self.this.get() {
                let usercall_args: [&dyn Serialize; _] = [&Bytes(data.as_slice())];

                let usercall_result = self.userv_access.usercall(
                    this,
                    ROLE_ID,
                    ops::ADD_DATA,
                    Arguments::Extended(data.len(), 0, &usercall_args),
                );
                if let Err(ec) = usercall_result {
                    Err((ec, data))
                } else {
                    self.current_op.set(Operation::AddMutData(data));
                    Ok(())
                }
            } else {
                Err((ErrorCode::NODEVICE, data))
            }
        }
    }

    fn clear_data(&self) {
        if let Some(this) = self.this.get() {
            // No return type means no error-handling for the operation or the usercall.
            let _usercall_result =
                self.userv_access
                    .usercall(this, ROLE_ID, ops::CLEAR_DATA, Arguments::Short(0, 0));
        }
    }
}

impl<'a: 'static, const L: usize> DigestHash<'a, L> for ServiceInterface<L> {
    fn set_hash_client(&'a self, client: &'a dyn ClientHash<L>) {
        self.hash_client.set(client)
    }

    fn run(&'a self, hash: &'static mut [u8; L]) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        if self.current_op.is_some() {
            Err((ErrorCode::BUSY, hash))
        } else {
            if let Some(this) = self.this.get() {
                let usercall_result =
                    self.userv_access
                        .usercall(this, ROLE_ID, ops::RUN, Arguments::Short(0, 0));
                if let Err(ec) = usercall_result {
                    Err((ec, hash))
                } else {
                    self.current_op.set(Operation::Run(hash));
                    Ok(())
                }
            } else {
                Err((ErrorCode::NODEVICE, hash))
            }
        }
    }
}

impl<'a: 'static, const L: usize> DigestVerify<'a, L> for ServiceInterface<L> {
    fn set_verify_client(&'a self, client: &'a dyn ClientVerify<L>) {
        self.verify_client.set(client)
    }

    fn verify(
        &'a self,
        expected_digest_buffer: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        if self.current_op.is_some() {
            Err((ErrorCode::BUSY, expected_digest_buffer))
        } else {
            if let Some(this) = self.this.get() {
                let usercall_res = self.userv_access.usercall(
                    this,
                    ROLE_ID,
                    ops::VERIFY,
                    Arguments::Extended(0, 0, &[&Bytes(expected_digest_buffer)]),
                );

                if let Err(ec) = usercall_res {
                    Err((ec, expected_digest_buffer))
                } else {
                    self.current_op
                        .set(Operation::Verify(expected_digest_buffer));
                    Ok(())
                }
            } else {
                Err((ErrorCode::NODEVICE, expected_digest_buffer))
            }
        }
    }
}

impl<const L: usize> digest::Sha256 for ServiceInterface<L> {
    fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
        Ok(())
    }
}
