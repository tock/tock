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
use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil::kv_system::{self, KVSystem, KeyType};
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

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait StoreClient<K: KeyType> {
    /// This callback is called when the get operation completes.
    ///
    /// - `result`: Nothing on success, 'ErrorCode' on error
    /// - `key`: The key buffer
    /// - `ret_buf`: The ret_buf buffer
    fn get_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    );

    /// This callback is called when the set operation completes.
    ///
    /// - `result`: Nothing on success, 'ErrorCode' on error
    /// - `key`: The key buffer
    /// - `value`: The value buffer
    fn set_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    );

    /// This callback is called when the delete operation completes.
    ///
    /// - `result`: Nothing on success, 'ErrorCode' on error
    /// - `key`: The key buffer
    fn delete_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: SubSliceMut<'static, u8>,
    );
}

pub struct KVStore<'a, K: KVSystem<'a> + KVSystem<'a, K = T>, T: 'static + kv_system::KeyType> {
    mux_kv: &'a MuxKVStore<'a, K, T>,
    next: ListLink<'a, KVStore<'a, K, T>>,

    client: OptionalCell<&'a dyn StoreClient<T>>,
    operation: OptionalCell<Operation>,

    unhashed_key: MapCell<SubSliceMut<'static, u8>>,
    value: MapCell<SubSliceMut<'static, u8>>,
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
            unhashed_key: MapCell::empty(),
            value: MapCell::empty(),
            valid_ids: OptionalCell::empty(),
        }
    }

    pub fn setup(&'a self) {
        self.mux_kv.users.push_head(self);
    }

    pub fn set_client(&self, client: &'a dyn StoreClient<T>) {
        self.client.set(client);
    }

    pub fn get(
        &self,
        unhashed_key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
        perms: StoragePermissions,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            Result<(), ErrorCode>,
        ),
    > {
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
        unhashed_key: SubSliceMut<'static, u8>,
        mut value: SubSliceMut<'static, u8>,
        perms: StoragePermissions,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            Result<(), ErrorCode>,
        ),
    > {
        let write_id = match perms.get_write_id() {
            Some(write_id) => write_id,
            None => return Err((unhashed_key, value, Err(ErrorCode::INVAL))),
        };

        if self.operation.is_some() {
            return Err((unhashed_key, value, Err(ErrorCode::BUSY)));
        }

        // The caller must ensure there is space for the header.
        if value.len() < HEADER_LENGTH {
            return Err((unhashed_key, value, Err(ErrorCode::SIZE)));
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
        self.unhashed_key.replace(unhashed_key);
        self.value.replace(value);
        self.mux_kv.do_next_op();
        Ok(())
    }

    pub fn delete(
        &self,
        unhashed_key: SubSliceMut<'static, u8>,
        perms: StoragePermissions,
    ) -> Result<(), (SubSliceMut<'static, u8>, Result<(), ErrorCode>)> {
        if self.operation.is_some() {
            return Err((unhashed_key, Err(ErrorCode::BUSY)));
        }

        self.operation.set(Operation::Delete);
        self.valid_ids.set(perms);
        self.unhashed_key.replace(unhashed_key);
        self.mux_kv.do_next_op();
        Ok(())
    }

    pub fn header_size(&self) -> usize {
        HEADER_LENGTH
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
        unhashed_key: SubSliceMut<'static, u8>,
        hashed_key: &'static mut T,
    ) {
        self.inflight.take().map(|node| {
            node.operation.map(|op| {
                if result.is_err() {
                    // On error, we re-store our state, run the next pending
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
                                match self.kv.get_value(hashed_key, SubSliceMut::new(value)) {
                                    Ok(()) => {
                                        node.unhashed_key.replace(unhashed_key);
                                        self.inflight.set(node);
                                    }
                                    Err((key, value, e)) => {
                                        self.hashed_key.replace(key);
                                        self.header_value.replace(value.take());
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
        value: SubSliceMut<'static, u8>,
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
        mut ret_buf: SubSliceMut<'static, u8>,
    ) {
        self.inflight.take().map(|node| {
            node.operation.map(|op| {
                match op {
                    Operation::Set => {}
                    Operation::Delete => {
                        let mut access_allowed = false;

                        // Before we delete an object we retrieve the header to
                        // ensure that we have permissions to access it. In that
                        // case we don't need to supply a buffer long enough to
                        // store the full value, so a `SIZE` error code is ok
                        // and we can continue to remove the object.
                        if result.is_ok() || result.err() == Some(ErrorCode::SIZE) {
                            let header = KeyHeader::new_from_buf(ret_buf.as_slice());

                            if header.version == HEADER_VERSION {
                                node.valid_ids.map(|perms| {
                                    access_allowed = perms.check_write_permission(header.write_id);
                                });
                            }
                        }

                        self.header_value.replace(ret_buf.take());

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

                        if result.is_ok() || result.err() == Some(ErrorCode::SIZE) {
                            let header = KeyHeader::new_from_buf(ret_buf.as_slice());

                            if header.version == HEADER_VERSION {
                                node.valid_ids.map(|perms| {
                                    read_allowed = perms.check_read_permission(header.write_id);
                                });

                                if read_allowed {
                                    // Remove the header from the accessible
                                    // portion of the buffer.
                                    ret_buf.slice(HEADER_LENGTH..);
                                }
                            }
                        }

                        if !read_allowed {
                            // Access denied or the header is invalid, zero the buffer.
                            ret_buf.as_slice().iter_mut().for_each(|m| *m = 0)
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
