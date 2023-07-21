// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Tock Key-Value virtualizer.
//!
//! This capsule provides a virtualized Key-Value store interface.
//!
//! ```
//! +-------------------------+
//! |  Capsule using K-V      |
//! +-------------------------+
//!
//!    capsules::kv_store::KV
//!
//! +-------------------------+
//! | Virtualizer (this file) |
//! +-------------------------+
//!
//!    capsules::kv_store::KV
//!
//! +-------------------------+
//! |  K-V store              |
//! +-------------------------+
//!
//!    hil::kv_system
//!
//! +-------------------------+
//! |  K-V library            |
//! +-------------------------+
//!
//!    hil::flash
//! ```

use kernel::collections::list::{List, ListLink, ListNode};

use kernel::storage_permissions::StoragePermissions;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::LeasableMutableBuffer;
use kernel::ErrorCode;

use crate::kv_store;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Operation {
    Get,
    Set,
    Delete,
}

pub struct VirtualKV<'a, V: kv_store::KV<'a>> {
    mux_kv: &'a MuxKV<'a, V>,
    next: ListLink<'a, VirtualKV<'a, V>>,

    client: OptionalCell<&'a dyn kv_store::StoreClient>,
    operation: OptionalCell<Operation>,

    unhashed_key: MapCell<LeasableMutableBuffer<'static, u8>>,
    value: MapCell<LeasableMutableBuffer<'static, u8>>,
    valid_ids: OptionalCell<StoragePermissions>,
}

impl<'a, V: kv_store::KV<'a>> ListNode<'a, VirtualKV<'a, V>> for VirtualKV<'a, V> {
    fn next(&self) -> &'a ListLink<VirtualKV<'a, V>> {
        &self.next
    }
}

impl<'a, V: kv_store::KV<'a>> VirtualKV<'a, V> {
    pub fn new(mux_kv: &'a MuxKV<'a, V>) -> VirtualKV<'a, V> {
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
}

impl<'a, V: kv_store::KV<'a>> kv_store::KV<'a> for VirtualKV<'a, V> {
    fn set_client(&self, client: &'a dyn kv_store::StoreClient) {
        self.client.set(client);
    }

    fn get(
        &self,
        key: LeasableMutableBuffer<'static, u8>,
        value: LeasableMutableBuffer<'static, u8>,
        permissions: StoragePermissions,
    ) -> Result<
        (),
        (
            LeasableMutableBuffer<'static, u8>,
            LeasableMutableBuffer<'static, u8>,
            Result<(), ErrorCode>,
        ),
    > {
        if self.operation.is_some() {
            return Err((key, value, Err(ErrorCode::BUSY)));
        }

        self.operation.set(Operation::Get);
        self.valid_ids.set(permissions);
        self.unhashed_key.replace(key);
        self.value.replace(value);

        self.mux_kv.do_next_op(false).map_err(|e| {
            (
                self.unhashed_key.take().unwrap(),
                self.value.take().unwrap(),
                e,
            )
        })
    }

    fn set(
        &self,
        key: LeasableMutableBuffer<'static, u8>,
        value: LeasableMutableBuffer<'static, u8>,
        permissions: StoragePermissions,
    ) -> Result<
        (),
        (
            LeasableMutableBuffer<'static, u8>,
            LeasableMutableBuffer<'static, u8>,
            Result<(), ErrorCode>,
        ),
    > {
        match permissions.get_write_id() {
            Some(_write_id) => {}
            None => return Err((key, value, Err(ErrorCode::INVAL))),
        }

        if self.operation.is_some() {
            return Err((key, value, Err(ErrorCode::BUSY)));
        }

        // The caller must ensure there is space for the header.
        if value.len() < self.header_size() {
            return Err((key, value, Err(ErrorCode::SIZE)));
        }

        self.operation.set(Operation::Set);
        self.valid_ids.set(permissions);
        self.unhashed_key.replace(key);
        self.value.replace(value);

        self.mux_kv.do_next_op(false).map_err(|e| {
            (
                self.unhashed_key.take().unwrap(),
                self.value.take().unwrap(),
                e,
            )
        })
    }

    fn delete(
        &self,
        key: LeasableMutableBuffer<'static, u8>,
        permissions: StoragePermissions,
    ) -> Result<(), (LeasableMutableBuffer<'static, u8>, Result<(), ErrorCode>)> {
        if self.operation.is_some() {
            return Err((key, Err(ErrorCode::BUSY)));
        }

        self.operation.set(Operation::Delete);
        self.valid_ids.set(permissions);
        self.unhashed_key.replace(key);

        self.mux_kv
            .do_next_op(false)
            .map_err(|e| (self.unhashed_key.take().unwrap(), e))
    }

    fn header_size(&self) -> usize {
        self.mux_kv.kv.header_size()
    }
}

pub struct MuxKV<'a, V: kv_store::KV<'a>> {
    kv: &'a V,
    users: List<'a, VirtualKV<'a, V>>,
    inflight: OptionalCell<&'a VirtualKV<'a, V>>,
}

impl<'a, V: kv_store::KV<'a>> MuxKV<'a, V> {
    pub fn new(kv: &'a V) -> MuxKV<'a, V> {
        Self {
            kv,
            inflight: OptionalCell::empty(),
            users: List::new(),
        }
    }

    fn do_next_op(&self, async_op: bool) -> Result<(), Result<(), ErrorCode>> {
        // Find a virtual device which has pending work.
        let mnode = self.users.iter().find(|node| node.operation.is_some());

        mnode.map_or(Ok(()), |node| {
            node.operation.map_or(Ok(()), |op| {
                node.unhashed_key
                    .take()
                    .map_or(Ok(()), |unhashed_key| match op {
                        Operation::Get => node.value.take().map_or(Ok(()), |value| {
                            node.valid_ids.map_or(Ok(()), |perms| {
                                match self.kv.get(unhashed_key, value, perms) {
                                    Ok(()) => {
                                        self.inflight.set(node);
                                        Ok(())
                                    }
                                    Err((unhashed_key, value, e)) => {
                                        node.operation.clear();
                                        if async_op {
                                            node.client.map(move |cb| {
                                                cb.get_complete(e, unhashed_key, value);
                                            });
                                            Ok(())
                                        } else {
                                            node.unhashed_key.replace(unhashed_key);
                                            node.value.replace(value);
                                            Err(e)
                                        }
                                    }
                                }
                            })
                        }),

                        Operation::Set => node.value.take().map_or(Ok(()), |value| {
                            node.valid_ids.map_or(Ok(()), |perms| {
                                match self.kv.set(unhashed_key, value, perms) {
                                    Ok(()) => {
                                        self.inflight.set(node);
                                        Ok(())
                                    }
                                    Err((unhashed_key, value, e)) => {
                                        node.operation.clear();
                                        if async_op {
                                            node.client.map(move |cb| {
                                                cb.set_complete(e, unhashed_key, value);
                                            });
                                            Ok(())
                                        } else {
                                            node.unhashed_key.replace(unhashed_key);
                                            node.value.replace(value);
                                            Err(e)
                                        }
                                    }
                                }
                            })
                        }),
                        Operation::Delete => node.valid_ids.map_or(Ok(()), |perms| {
                            match self.kv.delete(unhashed_key, perms) {
                                Ok(()) => {
                                    self.inflight.set(node);
                                    Ok(())
                                }
                                Err((unhashed_key, e)) => {
                                    node.operation.clear();
                                    if async_op {
                                        node.client.map(move |cb| {
                                            cb.delete_complete(e, unhashed_key);
                                        });
                                        Ok(())
                                    } else {
                                        node.unhashed_key.replace(unhashed_key);
                                        Err(e)
                                    }
                                }
                            }
                        }),
                    })
            })
        })
    }
}

impl<'a, V: kv_store::KV<'a>> kv_store::StoreClient for MuxKV<'a, V> {
    fn get_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: LeasableMutableBuffer<'static, u8>,
        value: LeasableMutableBuffer<'static, u8>,
    ) {
        self.inflight.take().map(|node| {
            node.operation.clear();
            node.client.map(move |cb| {
                cb.get_complete(result, unhashed_key, value);
            });
        });

        let _ = self.do_next_op(true);
    }

    fn set_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: LeasableMutableBuffer<'static, u8>,
        value: LeasableMutableBuffer<'static, u8>,
    ) {
        self.inflight.take().map(|node| {
            node.operation.clear();
            node.client.map(move |cb| {
                cb.set_complete(result, unhashed_key, value);
            });
        });

        let _ = self.do_next_op(true);
    }

    fn delete_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: LeasableMutableBuffer<'static, u8>,
    ) {
        self.inflight.take().map(|node| {
            node.operation.clear();
            node.client.map(move |cb| {
                cb.delete_complete(result, unhashed_key);
            });
        });

        let _ = self.do_next_op(true);
    }
}
