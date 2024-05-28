// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Tock Key-Value store capsule with permissions.
//!
//! This capsule provides a KV interface with permissions and access control.
//!
//! ```rust,ignore
//! +-----------------------+
//! |  Capsule using K-V    |
//! +-----------------------+
//!
//!    hil::kv::KVPermissions
//!
//! +-----------------------+
//! | K-V store (this file) |
//! +-----------------------+
//!
//!    hil::kv::KV
//!
//! +-----------------------+
//! |  K-V library          |
//! +-----------------------+
//!
//!    hil::flash
//! ```

use core::mem;
use kernel::hil::kv;
use kernel::storage_permissions::StoragePermissions;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Operation {
    Get,
    Set,
    Add,
    Update,
    Delete,
}

/// Current version of the Tock K-V header.
const HEADER_VERSION: u8 = 0;
pub const HEADER_LENGTH: usize = mem::size_of::<KeyHeader>();

/// This is the header used for KV stores.
#[repr(packed)]
struct KeyHeader {
    version: u8,
    length: u32,
    write_id: u32,
}

impl KeyHeader {
    /// Create a new `KeyHeader` from a buffer
    fn new_from_buf(buf: &[u8]) -> Self {
        Self {
            version: buf[0],
            length: u32::from_le_bytes(buf[1..5].try_into().unwrap_or([0; 4])),
            write_id: u32::from_le_bytes(buf[5..9].try_into().unwrap_or([0; 4])),
        }
    }

    /// Copy the header to `buf`
    fn copy_to_buf(&self, buf: &mut [u8]) {
        buf[0] = self.version;
        buf[1..5].copy_from_slice(&self.length.to_le_bytes());
        buf[5..9].copy_from_slice(&self.write_id.to_le_bytes());
    }
}

/// Key-Value store with Tock-specific extensions for permissions and access
/// control.
///
/// Implements `KVPermissions` on top of `KV`.
pub struct KVStorePermissions<'a, K: kv::KV<'a>> {
    kv: &'a K,
    header_value: TakeCell<'static, [u8]>,

    client: OptionalCell<&'a dyn kv::KVClient>,
    operation: OptionalCell<Operation>,

    value: MapCell<SubSliceMut<'static, u8>>,
    valid_ids: OptionalCell<StoragePermissions>,
}

impl<'a, K: kv::KV<'a>> KVStorePermissions<'a, K> {
    pub fn new(
        kv: &'a K,
        header_value: &'static mut [u8; HEADER_LENGTH],
    ) -> KVStorePermissions<'a, K> {
        Self {
            kv,
            header_value: TakeCell::new(header_value),
            client: OptionalCell::empty(),
            operation: OptionalCell::empty(),
            value: MapCell::empty(),
            valid_ids: OptionalCell::empty(),
        }
    }

    fn insert(
        &self,
        key: SubSliceMut<'static, u8>,
        mut value: SubSliceMut<'static, u8>,
        permissions: StoragePermissions,
        operation: Operation,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            ErrorCode,
        ),
    > {
        let write_id = match permissions.get_write_id() {
            Some(write_id) => write_id,
            None => return Err((key, value, ErrorCode::INVAL)),
        };

        if self.operation.is_some() {
            return Err((key, value, ErrorCode::BUSY));
        }

        // The caller must ensure there is space for the header.
        if value.len() < HEADER_LENGTH {
            return Err((key, value, ErrorCode::SIZE));
        }

        // Create the Tock header.
        let header = KeyHeader {
            version: HEADER_VERSION,
            length: (value.len() - HEADER_LENGTH) as u32,
            write_id,
        };

        // Copy in the header to the buffer.
        header.copy_to_buf(value.as_slice());

        self.operation.set(operation);

        match operation {
            Operation::Set | Operation::Update => {
                self.valid_ids.set(permissions);

                // We first read the key to see if we are allowed to overwrite it.
                match self.header_value.take() {
                    Some(header_value) => match self.kv.get(key, SubSliceMut::new(header_value)) {
                        Ok(()) => {
                            self.value.replace(value);
                            Ok(())
                        }
                        Err((key, hvalue, e)) => {
                            self.header_value.replace(hvalue.take());
                            self.operation.clear();
                            Err((key, value, e))
                        }
                    },
                    None => Err((key, value, ErrorCode::FAIL)),
                }
            }

            Operation::Add => {
                // Since add will only succeed if the key is not already there,
                // we do not have to worry about overwriting and do not need to
                // check permissions.
                match self.kv.add(key, value) {
                    Ok(()) => Ok(()),
                    Err((key, val, e)) => {
                        self.operation.clear();
                        Err((key, val, e))
                    }
                }
            }

            _ => Err((key, value, ErrorCode::FAIL)),
        }
    }
}

impl<'a, K: kv::KV<'a>> kv::KVPermissions<'a> for KVStorePermissions<'a, K> {
    fn set_client(&self, client: &'a dyn kv::KVClient) {
        self.client.set(client);
    }

    fn get(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
        permissions: StoragePermissions,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            ErrorCode,
        ),
    > {
        if self.operation.is_some() {
            return Err((key, value, ErrorCode::BUSY));
        }

        self.operation.set(Operation::Get);
        self.valid_ids.set(permissions);

        match self.kv.get(key, value) {
            Ok(()) => Ok(()),
            Err((key, val, e)) => {
                self.operation.clear();
                Err((key, val, e))
            }
        }
    }

    fn set(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
        permissions: StoragePermissions,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            ErrorCode,
        ),
    > {
        self.insert(key, value, permissions, Operation::Set)
    }

    fn add(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
        permissions: StoragePermissions,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            ErrorCode,
        ),
    > {
        self.insert(key, value, permissions, Operation::Add)
    }

    fn update(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
        permissions: StoragePermissions,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            ErrorCode,
        ),
    > {
        self.insert(key, value, permissions, Operation::Update)
    }

    fn delete(
        &self,
        key: SubSliceMut<'static, u8>,
        permissions: StoragePermissions,
    ) -> Result<(), (SubSliceMut<'static, u8>, ErrorCode)> {
        if self.operation.is_some() {
            return Err((key, ErrorCode::BUSY));
        }

        self.operation.set(Operation::Delete);
        self.valid_ids.set(permissions);

        match self.header_value.take() {
            Some(header_value) => match self.kv.get(key, SubSliceMut::new(header_value)) {
                Ok(()) => Ok(()),
                Err((key, hvalue, e)) => {
                    self.header_value.replace(hvalue.take());
                    self.operation.clear();
                    Err((key, e))
                }
            },
            None => Err((key, ErrorCode::FAIL)),
        }
    }

    fn header_size(&self) -> usize {
        HEADER_LENGTH
    }
}

impl<'a, K: kv::KV<'a>> kv::KVClient for KVStorePermissions<'a, K> {
    fn get_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        mut value: SubSliceMut<'static, u8>,
    ) {
        self.operation.map(|op| {
            match op {
                Operation::Set => {
                    // Need to determine if we have permission to set this key.
                    let mut access_allowed = false;

                    if result.is_ok() || result.err() == Some(ErrorCode::SIZE) {
                        let header = KeyHeader::new_from_buf(value.as_slice());

                        if header.version == HEADER_VERSION {
                            self.valid_ids.map(|perms| {
                                access_allowed = perms.check_write_permission(header.write_id);
                            });
                        }
                    } else if result.err() == Some(ErrorCode::NOSUPPORT) {
                        // Key wasn't found, so we can create it fresh.
                        access_allowed = true;
                    }

                    self.header_value.replace(value.take());

                    if access_allowed {
                        self.value
                            .take()
                            .map(|set_value| match self.kv.set(key, set_value) {
                                Ok(()) => {}

                                Err((key, set_value, e)) => {
                                    self.operation.clear();
                                    self.client.map(move |cb| {
                                        cb.set_complete(Err(e), key, set_value);
                                    });
                                }
                            });
                    } else {
                        self.operation.clear();
                        self.value.take().map(|set_value| {
                            self.client.map(move |cb| {
                                cb.set_complete(Err(ErrorCode::NOSUPPORT), key, set_value);
                            });
                        });
                    }
                }
                Operation::Update => {
                    // Need to determine if we have permission to set this key.
                    let mut access_allowed = false;

                    if result.is_ok() || result.err() == Some(ErrorCode::SIZE) {
                        let header = KeyHeader::new_from_buf(value.as_slice());

                        if header.version == HEADER_VERSION {
                            self.valid_ids.map(|perms| {
                                access_allowed = perms.check_write_permission(header.write_id);
                            });
                        }
                    }

                    self.header_value.replace(value.take());

                    if access_allowed {
                        self.value
                            .take()
                            .map(|set_value| match self.kv.update(key, set_value) {
                                Ok(()) => {}

                                Err((key, set_value, e)) => {
                                    self.operation.clear();
                                    self.client.map(move |cb| {
                                        cb.update_complete(Err(e), key, set_value);
                                    });
                                }
                            });
                    } else {
                        self.operation.clear();
                        self.value.take().map(|set_value| {
                            self.client.map(move |cb| {
                                cb.update_complete(Err(ErrorCode::NOSUPPORT), key, set_value);
                            });
                        });
                    }
                }
                Operation::Delete => {
                    // Before we delete an object we retrieve the header to
                    // ensure that we have permissions to access it. In that
                    // case we don't need to supply a buffer long enough to
                    // store the full value, so a `SIZE` error code is ok and we
                    // can continue to remove the object.
                    let mut access_allowed = false;

                    if result.is_ok() || result.err() == Some(ErrorCode::SIZE) {
                        let header = KeyHeader::new_from_buf(value.as_slice());

                        if header.version == HEADER_VERSION {
                            self.valid_ids.map(|perms| {
                                access_allowed = perms.check_write_permission(header.write_id);
                            });
                        }
                    }

                    self.header_value.replace(value.take());

                    if access_allowed {
                        match self.kv.delete(key) {
                            Ok(()) => {}

                            Err((key, e)) => {
                                self.operation.clear();
                                self.client.map(move |cb| {
                                    cb.delete_complete(Err(e), key);
                                });
                            }
                        }
                    } else {
                        self.operation.clear();
                        self.client.map(move |cb| {
                            cb.delete_complete(Err(ErrorCode::NOSUPPORT), key);
                        });
                    }
                }
                Operation::Get => {
                    self.operation.clear();

                    let mut read_allowed = false;

                    if result.is_ok() || result.err() == Some(ErrorCode::SIZE) {
                        let header = KeyHeader::new_from_buf(value.as_slice());

                        if header.version == HEADER_VERSION {
                            self.valid_ids.map(|perms| {
                                read_allowed = perms.check_read_permission(header.write_id);
                            });

                            if read_allowed {
                                // Remove the header from the accessible portion
                                // of the buffer.
                                value.slice(HEADER_LENGTH..);
                            }
                        }
                    }

                    if !read_allowed {
                        // Access denied or the header is invalid, zero the buffer.
                        value.as_slice().iter_mut().for_each(|m| *m = 0)
                    }

                    self.client.map(move |cb| {
                        if read_allowed {
                            cb.get_complete(result, key, value);
                        } else {
                            // The operation failed or the caller doesn't
                            // have permission, just return the error for
                            // key not found (and an empty buffer).
                            cb.get_complete(Err(ErrorCode::NOSUPPORT), key, value);
                        }
                    });
                }
                _ => {}
            }
        });
    }

    fn set_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.operation.clear();
        self.client.map(move |cb| {
            cb.set_complete(result, key, value);
        });
    }

    fn add_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.operation.clear();
        self.client.map(move |cb| {
            cb.add_complete(result, key, value);
        });
    }

    fn update_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.operation.clear();
        self.client.map(move |cb| {
            cb.update_complete(result, key, value);
        });
    }

    fn delete_complete(&self, result: Result<(), ErrorCode>, key: SubSliceMut<'static, u8>) {
        self.operation.clear();
        self.client.map(move |cb| {
            cb.delete_complete(result, key);
        });
    }
}
