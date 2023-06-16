// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock Key-Value store capsule.
//!
//! This capsule provides a virtualized Key-Value store interface based on an
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
//! | K-V store (this file)|
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

use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil::kv_system::{self, KVSystem};
use kernel::storage_permissions::StoragePermissions;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Operation {
    Get,
    Set,
    Delete,
}

const HEADER_VERSION: u8 = 0;
const HEADER_LENGTH: usize = 9;

/// This is the header used for KV stores
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

    /// Get the length of `KeyHeader`
    fn len(&self) -> usize {
        HEADER_LENGTH
    }

    /// Copy the header to `buf`
    fn copy_to_buf(&self, buf: &mut [u8]) {
        buf[0] = self.version;
        buf[1..5].copy_from_slice(&self.length.to_le_bytes());
        buf[5..9].copy_from_slice(&self.write_id.to_le_bytes());
    }
}

pub struct KVStore<'a, K: KVSystem<'a> + KVSystem<'a, K = T>, T: 'static + kv_system::KeyType> {
    mux_kv: &'a MuxKVStore<'a, K, T>,
    next: ListLink<'a, KVStore<'a, K, T>>,

    client: OptionalCell<&'a dyn kv_system::StoreClient<T>>,
    operation: OptionalCell<Operation>,

    unhashed_key: TakeCell<'static, [u8]>,
    value: TakeCell<'static, [u8]>,
    valid_ids: OptionalCell<StoragePermissions>,
}

impl<'a, K: KVSystem<'a, K = T>, T: kv_system::KeyType> ListNode<'a, KVStore<'a, K, T>>
    for KVStore<'a, K, T>
{
    fn next(&self) -> &'a ListLink<KVStore<'a, K, T>> {
        &self.next
    }
}

impl<'a, K: KVSystem<'a, K = T>, T: kv_system::KeyType> KVStore<'a, K, T> {
    pub fn new(mux_kv: &'a MuxKVStore<'a, K, T>) -> KVStore<'a, K, T> {
        Self {
            mux_kv,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            operation: OptionalCell::empty(),
            unhashed_key: TakeCell::empty(),
            value: TakeCell::empty(),
            valid_ids: OptionalCell::empty(),
        }
    }

    pub fn setup(&'a self) {
        self.mux_kv.users.push_head(self);
    }

    pub fn set_client(&self, client: &'a dyn kv_system::StoreClient<T>) {
        self.client.set(client);
    }

    pub fn get(
        &self,
        unhashed_key: &'static mut [u8],
        value: &'static mut [u8],
        perms: StoragePermissions,
    ) -> Result<(), (&'static mut [u8], &'static mut [u8], Result<(), ErrorCode>)> {
        if self.operation.is_some() {
            return Err((unhashed_key, value, Err(ErrorCode::BUSY)));
        }

        self.operation.set(Operation::Get);
        self.valid_ids.set(perms);
        self.unhashed_key.replace(unhashed_key);

        self.value.replace(value);
        self.mux_kv.do_next_op();
        Ok(())
    }

    pub fn set(
        &self,
        unhashed_key: &'static mut [u8],
        value: &'static mut [u8],
        length: usize,
        perms: StoragePermissions,
    ) -> Result<(), (&'static mut [u8], &'static mut [u8], Result<(), ErrorCode>)> {
        let write_id = match perms.get_write_id() {
            Some(write_id) => write_id,
            None => return Err((unhashed_key, value, Err(ErrorCode::INVAL))),
        };

        if self.operation.is_some() {
            return Err((unhashed_key, value, Err(ErrorCode::BUSY)));
        }

        // Create the Tock header and ensure we have space to fit it
        let header = KeyHeader {
            version: HEADER_VERSION,
            length: length as u32,
            write_id,
        };
        if length + header.len() > value.len() {
            return Err((unhashed_key, value, Err(ErrorCode::SIZE)));
        }

        // Move the value to make space for the header
        value.copy_within(0..length, header.len());
        header.copy_to_buf(value);

        self.operation.set(Operation::Set);
        self.unhashed_key.replace(unhashed_key);
        self.value.replace(value);
        self.mux_kv.do_next_op();
        Ok(())
    }

    pub fn delete(
        &self,
        unhashed_key: &'static mut [u8],
        perms: StoragePermissions,
    ) -> Result<(), (&'static mut [u8], Result<(), ErrorCode>)> {
        if self.operation.is_some() {
            return Err((unhashed_key, Err(ErrorCode::BUSY)));
        }

        self.operation.set(Operation::Delete);
        self.valid_ids.set(perms);
        self.unhashed_key.replace(unhashed_key);
        self.mux_kv.do_next_op();
        Ok(())
    }
}

/// Keep track of whether the kv is busy with doing a cleanup.
#[derive(PartialEq)]
enum StateCleanup {
    CleanupRequested,
    CleanupInProgress,
}

pub struct MuxKVStore<'a, K: KVSystem<'a> + KVSystem<'a, K = T>, T: 'static + kv_system::KeyType> {
    kv: &'a K,
    hashed_key: TakeCell<'static, T>,
    header_value: TakeCell<'static, [u8]>,
    cleanup: OptionalCell<StateCleanup>,
    users: List<'a, KVStore<'a, K, T>>,
    inflight: OptionalCell<&'a KVStore<'a, K, T>>,
}

impl<'a, K: KVSystem<'a> + KVSystem<'a, K = T>, T: 'static + kv_system::KeyType>
    MuxKVStore<'a, K, T>
{
    pub fn new(
        kv: &'a K,
        key: &'static mut T,
        header_value: &'static mut [u8; HEADER_LENGTH],
    ) -> MuxKVStore<'a, K, T> {
        Self {
            kv,
            hashed_key: TakeCell::new(key),
            header_value: TakeCell::new(header_value),
            inflight: OptionalCell::empty(),
            cleanup: OptionalCell::empty(),
            users: List::new(),
        }
    }

    fn do_next_op(&self) {
        if self.inflight.is_some() || self.cleanup.contains(&StateCleanup::CleanupInProgress) {
            return;
        }

        // Find a virtual device which has pending work.
        let mnode = self.users.iter().find(|node| node.operation.is_some());

        let ret = mnode.map_or(Err(ErrorCode::NODEVICE), |node| {
            node.operation.map(|op| {
                node.unhashed_key.take().map(|unhashed_key| {
                    self.hashed_key.take().map(|hashed_key| {
                        match op {
                            Operation::Get | Operation::Set => {
                                match self.kv.generate_key(unhashed_key, hashed_key) {
                                    Ok(()) => {
                                        self.inflight.set(node);
                                    }
                                    Err((unhashed_key, hashed_key, e)) => {
                                        // Issue callback with error.
                                        self.hashed_key.replace(hashed_key);
                                        node.operation.clear();
                                        node.value.take().map(|value| {
                                            node.client.map(move |cb| {
                                                cb.get_complete(e, unhashed_key, value);
                                            });
                                        });
                                    }
                                }
                            }
                            Operation::Delete => {
                                match self.kv.generate_key(unhashed_key, hashed_key) {
                                    Ok(()) => {
                                        self.inflight.set(node);
                                    }
                                    Err((unhashed_key, hashed_key, e)) => {
                                        self.hashed_key.replace(hashed_key);
                                        node.operation.clear();
                                        node.client.map(move |cb| {
                                            cb.delete_complete(e, unhashed_key);
                                        });
                                    }
                                }
                            }
                        };
                    });
                });
            });
            Ok(())
        });

        // If we have nothing scheduled, and we have recently done a delete, run
        // a garbage collect.
        if ret == Err(ErrorCode::NODEVICE) && self.cleanup.contains(&StateCleanup::CleanupRequested)
        {
            self.cleanup.set(StateCleanup::CleanupInProgress);
            // We have no way to report this error, and even if we could, what
            // would a user do?
            let _ = self.kv.garbage_collect();
        }
    }
}

impl<'a, K: KVSystem<'a, K = T>, T: kv_system::KeyType> kv_system::Client<T>
    for MuxKVStore<'a, K, T>
{
    fn generate_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: &'static mut [u8],
        hashed_key: &'static mut T,
    ) {
        self.inflight.take().map(|node| {
            node.operation.map(|op| {
                if result.is_err() {
                    // One error, we re-store our state, run the next pending
                    // operation, and notify the original user that their
                    // operation failed using a callback.
                    self.hashed_key.replace(hashed_key);
                    node.operation.clear();

                    match op {
                        Operation::Get => {
                            node.value.take().map(|value| {
                                node.client.map(move |cb| {
                                    cb.get_complete(result, unhashed_key, value);
                                });
                            });
                        }
                        Operation::Set => {
                            node.value.take().map(|value| {
                                node.client.map(move |cb| {
                                    cb.set_complete(result, unhashed_key, value);
                                });
                            });
                        }
                        Operation::Delete => {
                            node.client.map(move |cb| {
                                cb.delete_complete(result, unhashed_key);
                            });
                        }
                    }
                    // });
                } else {
                    match op {
                        Operation::Get => {
                            node.value.take().map(|value| {
                                match self.kv.get_value(hashed_key, value) {
                                    Ok(()) => {
                                        node.unhashed_key.replace(unhashed_key);
                                        self.inflight.set(node);
                                    }
                                    Err((key, value, e)) => {
                                        self.hashed_key.replace(key);
                                        node.operation.clear();
                                        node.client.map(move |cb| {
                                            cb.get_complete(e, unhashed_key, value);
                                        });
                                    }
                                }
                            });
                        }
                        Operation::Set => {
                            node.value.take().map(|value| {
                                match self.kv.append_key(hashed_key, value) {
                                    Ok(()) => {
                                        node.unhashed_key.replace(unhashed_key);
                                        self.inflight.set(node);
                                    }
                                    Err((key, value, e)) => {
                                        self.hashed_key.replace(key);
                                        node.operation.clear();
                                        node.client.map(move |cb| {
                                            cb.set_complete(e, unhashed_key, value);
                                        });
                                    }
                                }
                            });
                        }
                        Operation::Delete => {
                            self.header_value.take().map(|value| {
                                match self.kv.get_value(hashed_key, value) {
                                    Ok(()) => {
                                        node.unhashed_key.replace(unhashed_key);
                                        self.inflight.set(node);
                                    }
                                    Err((key, value, e)) => {
                                        self.hashed_key.replace(key);
                                        self.header_value.replace(value);
                                        node.operation.clear();
                                        node.client.map(move |cb| {
                                            cb.delete_complete(e, unhashed_key);
                                        });
                                    }
                                }
                            });
                        }
                    }
                }
            });
        });

        if self.inflight.is_none() {
            self.do_next_op();
        }
    }

    fn append_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut T,
        value: &'static mut [u8],
    ) {
        self.hashed_key.replace(key);

        self.inflight.take().map(|node| {
            node.operation.map(|op| match op {
                Operation::Get | Operation::Delete => {}
                Operation::Set => {
                    node.operation.clear();
                    node.unhashed_key.take().map(|unhashed_key| {
                        node.client.map(move |cb| {
                            cb.set_complete(result, unhashed_key, value);
                        });
                    });
                }
            });
        });

        self.do_next_op();
    }

    fn get_value_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut T,
        ret_buf: &'static mut [u8],
    ) {
        self.inflight.take().map(|node| {
            node.operation.map(|op| {
                match op {
                    Operation::Set => {}
                    Operation::Delete => {
                        let mut access_allowed = false;

                        if result.is_ok() {
                            let header = KeyHeader::new_from_buf(ret_buf);

                            if header.version == HEADER_VERSION {
                                node.valid_ids.map(|perms| {
                                    access_allowed = perms.check_write_permission(header.write_id);
                                });
                            }
                        }

                        self.header_value.replace(ret_buf);

                        if access_allowed {
                            match self.kv.invalidate_key(key) {
                                Ok(()) => {
                                    self.inflight.set(node);
                                }

                                Err((key, e)) => {
                                    node.operation.clear();
                                    self.hashed_key.replace(key);
                                    node.unhashed_key.take().map(|unhashed_key| {
                                        node.client.map(move |cb| {
                                            cb.delete_complete(e, unhashed_key);
                                        });
                                    });
                                }
                            }
                        } else {
                            node.operation.clear();
                            self.hashed_key.replace(key);
                            node.unhashed_key.take().map(|unhashed_key| {
                                node.client.map(move |cb| {
                                    cb.delete_complete(Err(ErrorCode::FAIL), unhashed_key);
                                });
                            });
                        }
                    }
                    Operation::Get => {
                        self.hashed_key.replace(key);
                        node.operation.clear();

                        let mut read_allowed = false;

                        if result.is_ok() {
                            let header = KeyHeader::new_from_buf(ret_buf);

                            if header.version == HEADER_VERSION {
                                node.valid_ids.map(|perms| {
                                    read_allowed = perms.check_read_permission(header.write_id);
                                });

                                if read_allowed {
                                    ret_buf.copy_within(
                                        HEADER_LENGTH..(HEADER_LENGTH + header.length as usize),
                                        0,
                                    );
                                }
                            }
                        }

                        if !read_allowed {
                            // Access denied or the header is invalid, zero the buffer.
                            ret_buf.iter_mut().for_each(|m| *m = 0)
                        }

                        node.unhashed_key.take().map(|unhashed_key| {
                            node.client.map(move |cb| {
                                if read_allowed {
                                    cb.get_complete(result, unhashed_key, ret_buf);
                                } else {
                                    // The operation failed or the caller
                                    // doesn't have permission, just return the
                                    // error for key not found (and an empty
                                    // buffer).
                                    cb.get_complete(
                                        Err(ErrorCode::NOSUPPORT),
                                        unhashed_key,
                                        ret_buf,
                                    );
                                }
                            });
                        });
                    }
                }
            });
        });

        if self.inflight.is_none() {
            self.do_next_op();
        }
    }

    fn invalidate_key_complete(&self, result: Result<(), ErrorCode>, key: &'static mut T) {
        self.hashed_key.replace(key);

        self.inflight.take().map(|node| {
            node.operation.map(|op| match op {
                Operation::Set | Operation::Get => {}
                Operation::Delete => {
                    node.operation.clear();
                    node.unhashed_key.take().map(|unhashed_key| {
                        node.client.map(move |cb| {
                            cb.delete_complete(result, unhashed_key);
                        });
                    });
                }
            });
        });

        self.cleanup.set(StateCleanup::CleanupRequested);
        self.do_next_op();
    }

    fn garbage_collect_complete(&self, _result: Result<(), ErrorCode>) {
        self.cleanup.clear();
        self.do_next_op();
    }
}
