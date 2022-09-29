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

use core::cell::Cell;
use kernel::collections::list::{List, ListLink, ListNode};
use kernel::hil::kv_system::{self, KVSystem};
use kernel::storage_permissions::StoragePermissions;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq)]
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

    next_operation: OptionalCell<Operation>,

    hashed_key: TakeCell<'static, T>,
    unhashed_key: TakeCell<'static, [u8]>,
    value: TakeCell<'static, [u8]>,
    header_value: TakeCell<'static, [u8]>,

    valid_ids: OptionalCell<StoragePermissions>,
    next_valid_ids: OptionalCell<StoragePermissions>,
}

impl<'a, K: KVSystem<'a, K = T>, T: kv_system::KeyType> ListNode<'a, KVStore<'a, K, T>>
    for KVStore<'a, K, T>
{
    fn next(&self) -> &'a ListLink<KVStore<'a, K, T>> {
        &self.next
    }
}

impl<'a, K: KVSystem<'a, K = T>, T: kv_system::KeyType> KVStore<'a, K, T> {
    pub fn new(
        mux_kv: &'a MuxKVStore<'a, K, T>,
        key: &'static mut T,
        header_value: &'static mut [u8; HEADER_LENGTH],
    ) -> KVStore<'a, K, T> {
        Self {
            mux_kv,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            next_operation: OptionalCell::empty(),
            hashed_key: TakeCell::new(key),
            unhashed_key: TakeCell::empty(),
            value: TakeCell::empty(),
            header_value: TakeCell::new(header_value),
            valid_ids: OptionalCell::empty(),
            next_valid_ids: OptionalCell::empty(),
        }
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
        if self.mux_kv.operation.is_none() {
            if self.hashed_key.is_none() {
                return Err((unhashed_key, value, Err(ErrorCode::NOMEM)));
            }

            self.mux_kv.operation.set(Operation::Get);
            self.valid_ids.set(perms);

            if let Some(Err((unhashed_key, e))) = self.hashed_key.take().map(|buf| {
                if let Err((unhashed_key, hashed_key, e)) =
                    self.mux_kv.kv.generate_key(unhashed_key, buf)
                {
                    self.hashed_key.replace(hashed_key);
                    self.mux_kv.operation.clear();
                    return Err((unhashed_key, e));
                }

                Ok(())
            }) {
                return Err((unhashed_key, value, e));
            }

            self.value.replace(value);
            Ok(())
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.next_operation.is_none() {
                self.next_operation.set(Operation::Get);
                self.unhashed_key.replace(unhashed_key);
                self.value.replace(value);
                self.next_valid_ids.set(perms);

                Ok(())
            } else {
                Err((unhashed_key, value, Err(ErrorCode::BUSY)))
            }
        }
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

        if self.mux_kv.operation.is_none() {
            // Make sure we have the hashed_key buffer
            if self.hashed_key.is_none() {
                return Err((unhashed_key, value, Err(ErrorCode::NOMEM)));
            }

            self.mux_kv.operation.set(Operation::Set);

            if let Some(Err((unhashed_key, e))) = self.hashed_key.take().map(|buf| {
                if let Err((unhashed_key, hashed_key, e)) =
                    self.mux_kv.kv.generate_key(unhashed_key, buf)
                {
                    self.hashed_key.replace(hashed_key);
                    self.mux_kv.operation.clear();
                    return Err((unhashed_key, e));
                }

                Ok(())
            }) {
                return Err((unhashed_key, value, e));
            }
            self.value.replace(value);
            Ok(())
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.next_operation.is_none() {
                self.next_operation.set(Operation::Set);
                self.unhashed_key.replace(unhashed_key);
                self.value.replace(value);
                Ok(())
            } else {
                Err((unhashed_key, value, Err(ErrorCode::BUSY)))
            }
        }
    }

    pub fn delete(
        &self,
        unhashed_key: &'static mut [u8],
        perms: StoragePermissions,
    ) -> Result<(), (&'static mut [u8], Result<(), ErrorCode>)> {
        if self.mux_kv.operation.is_none() {
            if self.hashed_key.is_none() {
                return Err((unhashed_key, Err(ErrorCode::NOMEM)));
            }

            self.valid_ids.set(perms);

            self.mux_kv.operation.set(Operation::Delete);

            if let Some(Err((unhashed_key, e))) = self.hashed_key.take().map(|buf| {
                if let Err((unhashed_key, hashed_key, e)) =
                    self.mux_kv.kv.generate_key(unhashed_key, buf)
                {
                    self.hashed_key.replace(hashed_key);
                    self.mux_kv.operation.clear();
                    return Err((unhashed_key, e));
                }

                Ok(())
            }) {
                return Err((unhashed_key, e));
            }
            Ok(())
        } else {
            // Another app is already running, queue this app as long as we
            // don't already have data queued.
            if self.next_operation.is_none() {
                self.next_operation.set(Operation::Delete);
                self.unhashed_key.replace(unhashed_key);
                self.next_valid_ids.set(perms);

                Ok(())
            } else {
                Err((unhashed_key, Err(ErrorCode::BUSY)))
            }
        }
    }
}

impl<'a, K: KVSystem<'a, K = T>, T: kv_system::KeyType + core::fmt::Debug> kv_system::Client<T>
    for KVStore<'a, K, T>
{
    fn generate_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: &'static mut [u8],
        hashed_key: &'static mut T,
    ) {
        self.unhashed_key.replace(unhashed_key);

        self.mux_kv.operation.map(|op| {
            if result.is_err() {
                self.hashed_key.replace(hashed_key);

                self.unhashed_key.take().map(|unhashed_key| match op {
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
                });
            } else {
                match op {
                    Operation::Get => {
                        self.value.take().map(|value| {
                            if let Err((key, value, e)) =
                                self.mux_kv.kv.get_value(hashed_key, value)
                            {
                                self.unhashed_key.take().map(|unhashed_key| {
                                    self.hashed_key.replace(key);
                                    self.client.map(move |cb| {
                                        cb.get_complete(e, unhashed_key, value);
                                    });
                                });
                            }
                        });
                    }
                    Operation::Set => {
                        self.value.take().map(|value| {
                            if let Err((key, value, e)) =
                                self.mux_kv.kv.append_key(hashed_key, value)
                            {
                                self.hashed_key.replace(key);
                                self.unhashed_key.take().map(|unhashed_key| {
                                    self.client.map(move |cb| {
                                        cb.set_complete(e, unhashed_key, value);
                                    });
                                });
                            }
                        });
                    }
                    Operation::Delete => {
                        self.header_value.take().map(|value| {
                            if let Err((key, value, e)) =
                                self.mux_kv.kv.get_value(hashed_key, value)
                            {
                                self.unhashed_key.take().map(|unhashed_key| {
                                    self.hashed_key.replace(key);
                                    self.client.map(move |cb| {
                                        cb.get_complete(e, unhashed_key, value);
                                    });
                                });
                            }
                        });
                    }
                }
            }
        });

        self.mux_kv.do_next_op();
    }

    fn append_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut T,
        value: &'static mut [u8],
    ) {
        self.hashed_key.replace(key);
        self.value.replace(value);

        self.mux_kv.operation.map(|op| match op {
            Operation::Get | Operation::Delete => {}
            Operation::Set => {
                self.unhashed_key.take().map(|unhashed_key| {
                    self.value.take().map(|value| {
                        self.client.map(move |cb| {
                            cb.set_complete(result, unhashed_key, value);
                        });
                    });
                });
                self.mux_kv.operation.clear();
            }
        });

        self.mux_kv.do_next_op();
    }

    fn get_value_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut T,
        ret_buf: &'static mut [u8],
    ) {
        self.hashed_key.replace(key);

        self.mux_kv.operation.map(|op| match op {
            Operation::Set => {}
            Operation::Delete => {
                let mut access_allowed = false;

                let header = KeyHeader::new_from_buf(ret_buf);

                if header.version == HEADER_VERSION {
                    self.valid_ids.map(|perms| {
                        access_allowed = perms.check_write_permission(header.write_id);
                    });
                }

                self.header_value.replace(ret_buf);

                if access_allowed {
                    self.hashed_key.take().map(|hashed_key| {
                        if let Err((key, e)) = self.mux_kv.kv.invalidate_key(hashed_key) {
                            self.hashed_key.replace(key);
                            self.unhashed_key.take().map(|unhashed_key| {
                                self.client.map(move |cb| {
                                    cb.delete_complete(e, unhashed_key);
                                });
                            });
                        }
                    });
                } else {
                    self.unhashed_key.take().map(|unhashed_key| {
                        self.client.map(move |cb| {
                            cb.delete_complete(Err(ErrorCode::FAIL), unhashed_key);
                        });
                    });
                }
            }
            Operation::Get => {
                let mut read_allowed = false;

                if result.is_ok() {
                    let header = KeyHeader::new_from_buf(ret_buf);

                    if header.version == HEADER_VERSION {
                        self.valid_ids.map(|perms| {
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
                    // Access denied or the header is invalid, zero the buffer
                    ret_buf.iter_mut().for_each(|m| *m = 0)
                }

                self.unhashed_key.take().map(|unhashed_key| {
                    self.client.map(move |cb| {
                        if read_allowed {
                            cb.get_complete(result, unhashed_key, ret_buf);
                        } else {
                            // The operation failed or the caller doesn't have permission,
                            // just return an error (and an empty buffer)
                            cb.get_complete(Err(ErrorCode::FAIL), unhashed_key, ret_buf);
                        }
                    });
                });
                self.mux_kv.operation.clear();
            }
        });

        self.mux_kv.do_next_op();
    }

    fn invalidate_key_complete(&self, result: Result<(), ErrorCode>, key: &'static mut T) {
        self.hashed_key.replace(key);

        self.mux_kv.operation.map(|op| match op {
            Operation::Set | Operation::Get => {}
            Operation::Delete => {
                self.unhashed_key.take().map(|unhashed_key| {
                    self.client.map(move |cb| {
                        cb.delete_complete(result, unhashed_key);
                    });
                });
                self.mux_kv.operation.clear();
            }
        });

        self.mux_kv.perform_cleanup.set(true);
        self.mux_kv.do_next_op();
    }

    fn garbage_collect_complete(&self, _result: Result<(), ErrorCode>) {
        self.mux_kv.perform_cleanup.set(false);
        self.mux_kv.do_next_op();
    }
}

pub struct MuxKVStore<'a, K: KVSystem<'a> + KVSystem<'a, K = T>, T: 'static + kv_system::KeyType> {
    kv: &'a K,
    operation: OptionalCell<Operation>,
    perform_cleanup: Cell<bool>,
    users: List<'a, KVStore<'a, K, T>>,
}

impl<'a, K: KVSystem<'a> + KVSystem<'a, K = T>, T: 'static + kv_system::KeyType>
    MuxKVStore<'a, K, T>
{
    pub const fn new(kv: &'a K) -> MuxKVStore<'a, K, T> {
        Self {
            kv,
            operation: OptionalCell::empty(),
            perform_cleanup: Cell::new(false),
            users: List::new(),
        }
    }

    fn do_next_op(&self) {
        if self.operation.is_some() {
            return;
        }

        let mnode = self.users.iter().find(|node| node.next_operation.is_some());

        let ret = mnode.map_or(Err(ErrorCode::NODEVICE), |node| {
            node.next_operation.map(|op| {
                self.operation.set(op.clone());

                node.unhashed_key.take().map(|unhashed_key| {
                    node.hashed_key.take().map(|hashed_key| {
                        match op {
                            Operation::Get => {
                                node.valid_ids.insert(node.next_valid_ids.take());
                                node.next_valid_ids.clear();

                                if let Err((unhashed_key, hashed_key, e)) =
                                    self.kv.generate_key(unhashed_key, hashed_key)
                                {
                                    node.hashed_key.replace(hashed_key);
                                    node.value.take().map(|value| {
                                        node.client.map(move |cb| {
                                            cb.get_complete(e, unhashed_key, value);
                                        });
                                    });
                                }
                            }
                            Operation::Set => {
                                if let Err((unhashed_key, hashed_key, e)) =
                                    self.kv.generate_key(unhashed_key, hashed_key)
                                {
                                    node.hashed_key.replace(hashed_key);
                                    node.value.take().map(|value| {
                                        node.client.map(move |cb| {
                                            cb.set_complete(e, unhashed_key, value);
                                        });
                                    });
                                }
                            }
                            Operation::Delete => {
                                if let Err((unhashed_key, hashed_key, e)) =
                                    self.kv.generate_key(unhashed_key, hashed_key)
                                {
                                    node.hashed_key.replace(hashed_key);
                                    node.client.map(move |cb| {
                                        cb.delete_complete(e, unhashed_key);
                                    });
                                }
                            }
                        };
                    });
                });
            });
            Ok(())
        });

        // If we have nothing scheduled, run a garbage collect
        if ret == Err(ErrorCode::NODEVICE) && self.perform_cleanup.get() {
            // We have no way to report this error, and even if we could, what
            // would a user do?
            let _ = self.kv.garbage_collect();
        }
    }
}
