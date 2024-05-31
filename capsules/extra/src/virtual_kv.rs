// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Tock Key-Value virtualizer.
//!
//! This capsule provides a virtualized Key-Value store interface.
//!
//! ```text
//! +-------------------------+
//! |  Capsule using K-V      |
//! +-------------------------+
//!
//!    hil::kv::KVPermissions
//!
//! +-------------------------+
//! | Virtualizer (this file) |
//! +-------------------------+
//!
//!    hil::kv::KVPermissions
//!
//! +-------------------------+
//! |  K-V store Permissions  |
//! +-------------------------+
//!
//!    hil::kv::KV
//!
//! +-------------------------+
//! |  K-V library            |
//! +-------------------------+
//!
//!    hil::flash
//! ```

use kernel::collections::list::{List, ListLink, ListNode};

use kernel::hil::kv;
use kernel::hil::kv::KVPermissions;
use kernel::storage_permissions::StoragePermissions;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Operation {
    Get,
    Set,
    Delete,
    Add,
    Update,
}

pub struct VirtualKVPermissions<'a, V: kv::KVPermissions<'a>> {
    mux_kv: &'a MuxKVPermissions<'a, V>,
    next: ListLink<'a, VirtualKVPermissions<'a, V>>,

    client: OptionalCell<&'a dyn kv::KVClient>,
    operation: OptionalCell<Operation>,

    key: MapCell<SubSliceMut<'static, u8>>,
    value: MapCell<SubSliceMut<'static, u8>>,
    valid_ids: OptionalCell<StoragePermissions>,
}

impl<'a, V: kv::KVPermissions<'a>> ListNode<'a, VirtualKVPermissions<'a, V>>
    for VirtualKVPermissions<'a, V>
{
    fn next(&self) -> &'a ListLink<VirtualKVPermissions<'a, V>> {
        &self.next
    }
}

impl<'a, V: kv::KVPermissions<'a>> VirtualKVPermissions<'a, V> {
    pub fn new(mux_kv: &'a MuxKVPermissions<'a, V>) -> VirtualKVPermissions<'a, V> {
        Self {
            mux_kv,
            next: ListLink::empty(),
            client: OptionalCell::empty(),
            operation: OptionalCell::empty(),
            key: MapCell::empty(),
            value: MapCell::empty(),
            valid_ids: OptionalCell::empty(),
        }
    }

    pub fn setup(&'a self) {
        self.mux_kv.users.push_head(self);
    }

    fn insert(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
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
        match permissions.get_write_id() {
            Some(_write_id) => {}
            None => return Err((key, value, ErrorCode::INVAL)),
        }

        if self.operation.is_some() {
            return Err((key, value, ErrorCode::BUSY));
        }

        // The caller must ensure there is space for the header.
        if value.len() < self.header_size() {
            return Err((key, value, ErrorCode::SIZE));
        }

        self.operation.set(operation);
        self.valid_ids.set(permissions);
        self.key.replace(key);
        self.value.replace(value);

        self.mux_kv
            .do_next_op(false)
            .map_err(|e| (self.key.take().unwrap(), self.value.take().unwrap(), e))
    }
}

impl<'a, V: kv::KVPermissions<'a>> kv::KVPermissions<'a> for VirtualKVPermissions<'a, V> {
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
        self.key.replace(key);
        self.value.replace(value);

        self.mux_kv
            .do_next_op(false)
            .map_err(|e| (self.key.take().unwrap(), self.value.take().unwrap(), e))
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
        self.key.replace(key);

        self.mux_kv
            .do_next_op(false)
            .map_err(|e| (self.key.take().unwrap(), e))
    }

    fn header_size(&self) -> usize {
        self.mux_kv.kv.header_size()
    }
}

pub struct MuxKVPermissions<'a, V: kv::KVPermissions<'a>> {
    kv: &'a V,
    users: List<'a, VirtualKVPermissions<'a, V>>,
    inflight: OptionalCell<&'a VirtualKVPermissions<'a, V>>,
}

impl<'a, V: kv::KVPermissions<'a>> MuxKVPermissions<'a, V> {
    pub fn new(kv: &'a V) -> MuxKVPermissions<'a, V> {
        Self {
            kv,
            inflight: OptionalCell::empty(),
            users: List::new(),
        }
    }

    fn do_next_op(&self, async_op: bool) -> Result<(), ErrorCode> {
        // Find a virtual device which has pending work.
        let mnode = self.users.iter().find(|node| node.operation.is_some());

        mnode.map_or(Ok(()), |node| {
            node.operation.map_or(Ok(()), |op| {
                node.key.take().map_or(Ok(()), |key| match op {
                    Operation::Get => node.value.take().map_or(Ok(()), |value| {
                        node.valid_ids.map_or(Ok(()), |perms| {
                            match self.kv.get(key, value, perms) {
                                Ok(()) => {
                                    self.inflight.set(node);
                                    Ok(())
                                }
                                Err((key, value, e)) => {
                                    node.operation.clear();
                                    if async_op {
                                        node.client.map(move |cb| {
                                            cb.get_complete(Err(e), key, value);
                                        });
                                        Ok(())
                                    } else {
                                        node.key.replace(key);
                                        node.value.replace(value);
                                        Err(e)
                                    }
                                }
                            }
                        })
                    }),
                    Operation::Set => node.value.take().map_or(Ok(()), |value| {
                        node.valid_ids.map_or(Ok(()), |perms| {
                            match self.kv.set(key, value, perms) {
                                Ok(()) => {
                                    self.inflight.set(node);
                                    Ok(())
                                }
                                Err((key, value, e)) => {
                                    node.operation.clear();
                                    if async_op {
                                        node.client.map(move |cb| {
                                            cb.set_complete(Err(e), key, value);
                                        });
                                        Ok(())
                                    } else {
                                        node.key.replace(key);
                                        node.value.replace(value);
                                        Err(e)
                                    }
                                }
                            }
                        })
                    }),
                    Operation::Add => node.value.take().map_or(Ok(()), |value| {
                        node.valid_ids.map_or(Ok(()), |perms| {
                            match self.kv.add(key, value, perms) {
                                Ok(()) => {
                                    self.inflight.set(node);
                                    Ok(())
                                }
                                Err((key, value, e)) => {
                                    node.operation.clear();
                                    if async_op {
                                        node.client.map(move |cb| {
                                            cb.add_complete(Err(e), key, value);
                                        });
                                        Ok(())
                                    } else {
                                        node.key.replace(key);
                                        node.value.replace(value);
                                        Err(e)
                                    }
                                }
                            }
                        })
                    }),
                    Operation::Update => node.value.take().map_or(Ok(()), |value| {
                        node.valid_ids.map_or(Ok(()), |perms| {
                            match self.kv.update(key, value, perms) {
                                Ok(()) => {
                                    self.inflight.set(node);
                                    Ok(())
                                }
                                Err((key, value, e)) => {
                                    node.operation.clear();
                                    if async_op {
                                        node.client.map(move |cb| {
                                            cb.update_complete(Err(e), key, value);
                                        });
                                        Ok(())
                                    } else {
                                        node.key.replace(key);
                                        node.value.replace(value);
                                        Err(e)
                                    }
                                }
                            }
                        })
                    }),
                    Operation::Delete => {
                        node.valid_ids
                            .map_or(Ok(()), |perms| match self.kv.delete(key, perms) {
                                Ok(()) => {
                                    self.inflight.set(node);
                                    Ok(())
                                }
                                Err((key, e)) => {
                                    node.operation.clear();
                                    if async_op {
                                        node.client.map(move |cb| {
                                            cb.delete_complete(Err(e), key);
                                        });
                                        Ok(())
                                    } else {
                                        node.key.replace(key);
                                        Err(e)
                                    }
                                }
                            })
                    }
                })
            })
        })
    }
}

impl<'a, V: kv::KVPermissions<'a>> kv::KVClient for MuxKVPermissions<'a, V> {
    fn get_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.inflight.take().map(|node| {
            node.operation.clear();
            node.client.map(move |cb| {
                cb.get_complete(result, key, value);
            });
        });

        let _ = self.do_next_op(true);
    }

    fn set_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.inflight.take().map(|node| {
            node.operation.clear();
            node.client.map(move |cb| {
                cb.set_complete(result, key, value);
            });
        });

        let _ = self.do_next_op(true);
    }

    fn add_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.inflight.take().map(|node| {
            node.operation.clear();
            node.client.map(move |cb| {
                cb.add_complete(result, key, value);
            });
        });

        let _ = self.do_next_op(true);
    }

    fn update_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) {
        self.inflight.take().map(|node| {
            node.operation.clear();
            node.client.map(move |cb| {
                cb.update_complete(result, key, value);
            });
        });

        let _ = self.do_next_op(true);
    }

    fn delete_complete(&self, result: Result<(), ErrorCode>, key: SubSliceMut<'static, u8>) {
        self.inflight.take().map(|node| {
            node.operation.clear();
            node.client.map(move |cb| {
                cb.delete_complete(result, key);
            });
        });

        let _ = self.do_next_op(true);
    }
}
