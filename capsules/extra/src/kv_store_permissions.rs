// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock Key-Value store capsule.
//!
//! This capsule provides a higher level Key-Value store interface based on an
//! underlying `hil::kv_system` storage layer.
//!
//! ```
//! +-----------------------+
//! |                       |
//! |  Capsule using K-V    |
//! |                       |
//! +-----------------------+
//!
//!    capsules::kv_store
//!
//! +-----------------------+
//! |                       |
//! | K-V store (this file) |
//! |                       |
//! +-----------------------+
//!
//!    hil::kv_system
//!
//! +-----------------------+
//! |                       |
//! |  K-V library          |
//! |                       |
//! +-----------------------+
//!
//!    hil::flash
//! ```

use core::mem;
use kernel::hil::kv;
use kernel::hil::kv_system::{self, KVSystem};
use kernel::storage_permissions::StoragePermissions;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Operation {
    Get,
    Set,
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

/// KVStore implements the Tock-specific extension to KVSystem that includes
/// permissions and access control.
pub struct KVStorePermissions<'a, K: kv::KV> {
    kv: &'a K,
    header_value: TakeCell<'static, [u8]>,

    client: OptionalCell<&'a dyn kv::KVClient>,
    operation: OptionalCell<Operation>,

    key: MapCell<SubSliceMut<'static, u8>>,
    value: MapCell<SubSliceMut<'static, u8>>,
    valid_ids: OptionalCell<StoragePermissions>,
}

impl<'a, K: kv::KV> KVStore<'a, K> {
    pub fn new(kv: &'a K, header_value: &'static mut [u8; HEADER_LENGTH]) -> KVStore<'a, K> {
        Self {
            kv,
            header_value: TakeCell::new(header_value),
            client: OptionalCell::empty(),
            operation: OptionalCell::empty(),
            key: MapCell::empty(),
            value: MapCell::empty(),
            valid_ids: OptionalCell::empty(),
        }
    }
}

impl<'a, K: KVSystem<'a, K = T>, T: kv_system::KeyType> kv::KVPermissions<'a>
    for KVStore<'a, K, T>
{
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
        // self.value.replace(value);

        // self.key
        //     .take()
        //     .map_or(ErrorCode::FAIL, |key| {
        match self.kv.get(key, value) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.operation.clear();
                // self.hashed_key.replace(hashed_key);
                // self.unhashed_key.replace(unhashed_key);
                e
            }
        }
        // })
        // .map_err(|e| {
        //     (
        //         self.unhashed_key.take().unwrap(),
        //         self.value.take().unwrap(),
        //         Err(e),
        //     )
        // })
    }

    fn set(
        &self,
        key: SubSliceMut<'static, u8>,
        mut value: SubSliceMut<'static, u8>,
        permissions: StoragePermissions,
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

        self.operation.set(Operation::Set);
        self.valid_ids.set(permissions);

        // We first read what was there to see if we are allowed to overwrite
        // it.
        self.header_value.take().map(|header_value| {
            match self.kv.get(key, SubSliceMut::new(header_value)) {
                Ok(()) => {
                    self.value.replace(value);
                }
                Err((key, hvalue, e)) => {
                    self.header_value.replace(hvalue.take());
                    self.operation.clear();
                    (key, value, e)
                }
            }
        });

        // self.hashed_key
        //     .take()
        //     .map_or(Err(ErrorCode::FAIL), |hashed_key| {
        // match self.kv.set(key, value) {
        //     Ok(()) => Ok(()),
        //     Err((unhashed_key, hashed_key, e)) => {
        //         self.operation.clear();
        //         self.hashed_key.replace(hashed_key);
        //         self.unhashed_key.replace(unhashed_key);
        //         e
        //     }
        // }
        // })
        // .map_err(|e| {
        //     (
        //         self.unhashed_key.take().unwrap(),
        //         self.value.take().unwrap(),
        //         Err(e),
        //     )
        // })
    }

    fn delete(
        &self,
        key: SubSliceMut<'static, u8>,
        permissions: StoragePermissions,
    ) -> Result<(), (SubSliceMut<'static, u8>, Result<(), ErrorCode>)> {
        if self.operation.is_some() {
            return Err((key, Err(ErrorCode::BUSY)));
        }

        self.operation.set(Operation::Delete);
        self.valid_ids.set(permissions);

        // // self.hashed_key
        // //     .take()
        // //     .map_or(Err(ErrorCode::FAIL), |hashed_key| {
        //         match self.kv.delete(key) {
        //             Ok(()) => Ok(()),
        //             Err((unhashed_key, hashed_key, e)) => {
        //                 self.hashed_key.replace(hashed_key);
        //                 self.operation.clear();
        //                 self.unhashed_key.replace(unhashed_key);
        //                 e
        //             }
        //         }
        //     // })
        //     // .map_err(|e| (self.unhashed_key.take().unwrap(), Err(e)))

        self.header_value.take().map(|header_value| {
            match self.kv.get(key, SubSliceMut::new(header_value)) {
                Ok(()) => {
                    self.value.replace(value);
                }
                Err((key, hvalue, e)) => {
                    self.header_value.replace(hvalue.take());
                    self.operation.clear();
                    (key, value, e)
                }
            }
        });
    }

    fn header_size(&self) -> usize {
        HEADER_LENGTH
    }
}

impl<'a, K: kv::KV> kv::KVClient for KVStorePermissions<'a, K> {
    fn generate_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: SubSliceMut<'static, u8>,
        hashed_key: &'static mut T,
    ) {
        self.operation.map(|op| {
            if result.is_err() {
                // On error, we re-store our state, run the next pending
                // operation, and notify the original user that their operation
                // failed using a callback.
                self.hashed_key.replace(hashed_key);
                self.operation.clear();

                match op {
                    Operation::Get => {
                        self.value.take().map(|value| {
                            self.client.map(move |cb| {
                                cb.get_complete(result, unhashed_key, value);
                            });
                        });
                    }
                    Operation::Set => {
                        self.value.take().map(|value| {
                            self.client.map(move |cb| {
                                cb.set_complete(result, unhashed_key, value);
                            });
                        });
                    }
                    Operation::Delete => {
                        self.client.map(move |cb| {
                            cb.delete_complete(result, unhashed_key);
                        });
                    }
                }
            } else {
                match op {
                    Operation::Get => {
                        self.value
                            .take()
                            .map(|value| match self.kv.get_value(hashed_key, value) {
                                Ok(()) => {
                                    self.unhashed_key.replace(unhashed_key);
                                }
                                Err((key, value, e)) => {
                                    self.hashed_key.replace(key);
                                    self.operation.clear();
                                    self.client.map(move |cb| {
                                        cb.get_complete(e, unhashed_key, value);
                                    });
                                }
                            });
                    }
                    Operation::Set => {
                        self.value.take().map(|value| {
                            match self.kv.append_key(hashed_key, value) {
                                Ok(()) => {
                                    self.unhashed_key.replace(unhashed_key);
                                }
                                Err((key, value, e)) => {
                                    self.hashed_key.replace(key);
                                    self.operation.clear();
                                    self.client.map(move |cb| {
                                        cb.set_complete(e, unhashed_key, value);
                                    });
                                }
                            }
                        });
                    }
                    Operation::Delete => {
                        self.header_value.take().map(|value| {
                            match self
                                .kv
                                .get_value(hashed_key, LeasableMutableBuffer::new(value))
                            {
                                Ok(()) => {
                                    self.unhashed_key.replace(unhashed_key);
                                }
                                Err((key, value, e)) => {
                                    self.hashed_key.replace(key);
                                    self.header_value.replace(value.take());
                                    self.operation.clear();
                                    self.client.map(move |cb| {
                                        cb.delete_complete(e, unhashed_key);
                                    });
                                }
                            }
                        });
                    }
                }
            }
        });
    }

    fn append_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut T,
        value: SubSliceMut<'static, u8>,
    ) {
        self.hashed_key.replace(key);

        self.operation.map(|op| match op {
            Operation::Get | Operation::Delete => {}
            Operation::Set => {
                match result {
                    Err(ErrorCode::NOSUPPORT) => {
                        // We could not append because of a collision. So now we
                        // must figure out if we are allowed to overwrite this
                        // key. That starts by reading the key.
                        self.hashed_key.take().map(|hashed_key| {
                            self.header_value.take().map(|header_value| {
                                match self
                                    .kv
                                    .get_value(hashed_key, LeasableMutableBuffer::new(header_value))
                                {
                                    Ok(()) => {
                                        self.value.replace(value);
                                    }
                                    Err((key, hvalue, e)) => {
                                        self.hashed_key.replace(key);
                                        self.header_value.replace(hvalue.take());
                                        self.operation.clear();
                                        self.unhashed_key.take().map(|unhashed_key| {
                                            self.client.map(move |cb| {
                                                cb.set_complete(e, unhashed_key, value);
                                            });
                                        });
                                    }
                                }
                            });
                        });
                    }
                    _ => {
                        // On success or any other error we just return the
                        // result back to the caller via a callback.
                        self.operation.clear();
                        self.unhashed_key.take().map(|unhashed_key| {
                            self.client.map(move |cb| {
                                cb.set_complete(result, unhashed_key, value);
                            });
                        });
                    }
                }
            }
        });
    }

    fn get_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        mut value: SubSliceMut<'static, u8>,
    ) {
        fn access_allowed(result: Result<(), ErrorCode>, value: SubSliceMut<'static, u8>) -> bool {
            let mut access_allowed = false;

            if result.is_ok() || result.err() == Some(ErrorCode::SIZE) {
                let header = KeyHeader::new_from_buf(value.as_slice());

                if header.version == HEADER_VERSION {
                    self.valid_ids.map(|perms| {
                        access_allowed = perms.check_write_permission(header.write_id);
                    });
                }
            }

            access_allowed
        }

        self.operation.map(|op| {
            match op {
                Operation::Set => {
                    // Need to determine if we have permission to set this key.
                    let access_allowed = self.access_allowed(result, value);

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
                                cb.set_complete(Err(ErrorCode::FAIL), key, set_value);
                            });
                        });
                        // });
                    }
                }
                Operation::Delete => {
                    let mut access_allowed = false;

                    // Before we delete an object we retrieve the header to
                    // ensure that we have permissions to access it. In that
                    // case we don't need to supply a buffer long enough to
                    // store the full value, so a `SIZE` error code is ok and we
                    // can continue to remove the object.
                    if result.is_ok() || result.err() == Some(ErrorCode::SIZE) {
                        let header = KeyHeader::new_from_buf(ret_buf.as_slice());

                        if header.version == HEADER_VERSION {
                            self.valid_ids.map(|perms| {
                                access_allowed = perms.check_write_permission(header.write_id);
                            });
                        }
                    }

                    self.header_value.replace(ret_buf.take());

                    if access_allowed {
                        match self.kv.delete(key) {
                            Ok(()) => {}

                            Err((key, e)) => {
                                self.operation.clear();
                                // self.hashed_key.replace(key);
                                // self.unhashed_key.take().map(|unhashed_key| {
                                self.client.map(move |cb| {
                                    cb.delete_complete(Err(e), key);
                                });
                                // });
                            }
                        }
                    } else {
                        self.operation.clear();
                        // self.hashed_key.replace(key);
                        // self.unhashed_key.take().map(|unhashed_key| {
                        self.client.map(move |cb| {
                            cb.delete_complete(Err(ErrorCode::FAIL), key);
                        });
                        // });
                    }
                }
                Operation::Get => {
                    // self.hashed_key.replace(key);
                    self.operation.clear();

                    let mut read_allowed = false;

                    if result.is_ok() || result.err() == Some(ErrorCode::SIZE) {
                        let header = KeyHeader::new_from_buf(ret_buf.as_slice());

                        if header.version == HEADER_VERSION {
                            self.valid_ids.map(|perms| {
                                read_allowed = perms.check_read_permission(header.write_id);
                            });

                            if read_allowed {
                                // Remove the header from the accessible portion
                                // of the buffer.
                                ret_buf.slice(HEADER_LENGTH..);
                            }
                        }
                    }

                    if !read_allowed {
                        // Access denied or the header is invalid, zero the buffer.
                        ret_buf.as_slice().iter_mut().for_each(|m| *m = 0)
                    }

                    // self.unhashed_key.take().map(|unhashed_key| {
                    self.client.map(move |cb| {
                        if read_allowed {
                            cb.get_complete(result, key, ret_buf);
                        } else {
                            // The operation failed or the caller doesn't
                            // have permission, just return the error for
                            // key not found (and an empty buffer).
                            cb.get_complete(Err(ErrorCode::NOSUPPORT), key, ret_buf);
                        }
                    });
                    // });
                }
            }
        });
    }

    fn invalidate_key_complete(&self, result: Result<(), ErrorCode>, key: &'static mut T) {
        self.hashed_key.replace(key);

        self.operation.map(|op| match op {
            Operation::Get => {}
            Operation::Set => {
                // Now that we have deleted the existing key-value we can store
                // our new key and value.
                match result {
                    Ok(()) => {
                        self.hashed_key.take().map(|hashed_key| {
                            self.value.take().map(|value| {
                                match self.kv.append_key(hashed_key, value) {
                                    Ok(()) => {}
                                    Err((key, value, e)) => {
                                        self.hashed_key.replace(key);
                                        self.operation.clear();
                                        self.unhashed_key.take().map(|unhashed_key| {
                                            self.client.map(move |cb| {
                                                cb.set_complete(e, unhashed_key, value);
                                            });
                                        });
                                    }
                                }
                            });
                        });
                    }
                    _ => {
                        // Some error with delete, signal error.
                        self.operation.clear();
                        self.unhashed_key.take().map(|unhashed_key| {
                            self.value.take().map(|value| {
                                self.client.map(move |cb| {
                                    cb.set_complete(Err(ErrorCode::NOSUPPORT), unhashed_key, value);
                                });
                            });
                        });
                    }
                }
            }
            Operation::Delete => {
                self.operation.clear();
                self.unhashed_key.take().map(|unhashed_key| {
                    self.client.map(move |cb| {
                        cb.delete_complete(result, unhashed_key);
                    });
                });
            }
        });
    }

    fn garbage_collect_complete(&self, _result: Result<(), ErrorCode>) {}
}
