// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! TicKV to Tock key-value store capsule.
//!
//! This capsule provides a higher level Key-Value store interface based on an
//! underlying `tickv::kv_system` storage layer.
//!
//! ```text
//! +-----------------------+
//! |  Capsule using K-V    |
//! +-----------------------+
//!
//!    hil::kv::KV
//!
//! +-----------------------+
//! | K-V store (this file) |
//! +-----------------------+
//!
//!    capsules::tickv::kv_system
//!
//! +-----------------------+
//! |  K-V library          |
//! +-----------------------+
//!
//!    hil::flash
//! ```

use crate::tickv::{KVSystem, KVSystemClient, KeyType};
use kernel::hil::kv;
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

/// `TicKVKVStore` implements the KV interface using the TicKV KVSystem
/// interface.
pub struct TicKVKVStore<'a, K: KVSystem<'a> + KVSystem<'a, K = T>, T: 'static + KeyType> {
    kv: &'a K,
    hashed_key: TakeCell<'static, T>,

    client: OptionalCell<&'a dyn kv::KVClient>,
    operation: OptionalCell<Operation>,

    unhashed_key: MapCell<SubSliceMut<'static, u8>>,
    value: MapCell<SubSliceMut<'static, u8>>,
}

impl<'a, K: KVSystem<'a, K = T>, T: KeyType> TicKVKVStore<'a, K, T> {
    pub fn new(kv: &'a K, key: &'static mut T) -> TicKVKVStore<'a, K, T> {
        Self {
            kv,
            hashed_key: TakeCell::new(key),
            client: OptionalCell::empty(),
            operation: OptionalCell::empty(),
            unhashed_key: MapCell::empty(),
            value: MapCell::empty(),
        }
    }

    fn insert(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
        operation: Operation,
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

        self.operation.set(operation);

        match self.hashed_key.take() {
            Some(hashed_key) => match self.kv.generate_key(key, hashed_key) {
                Ok(()) => {
                    self.value.replace(value);
                    Ok(())
                }
                Err((unhashed_key, hashed_key, _e)) => {
                    self.operation.clear();
                    self.hashed_key.replace(hashed_key);
                    Err((unhashed_key, value, ErrorCode::FAIL))
                }
            },
            None => Err((key, value, ErrorCode::FAIL)),
        }
    }
}

impl<'a, K: KVSystem<'a, K = T>, T: KeyType> kv::KV<'a> for TicKVKVStore<'a, K, T> {
    fn set_client(&self, client: &'a dyn kv::KVClient) {
        self.client.set(client);
    }

    fn get(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
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

        match self.hashed_key.take() {
            Some(hashed_key) => match self.kv.generate_key(key, hashed_key) {
                Ok(()) => {
                    self.value.replace(value);
                    Ok(())
                }
                Err((unhashed_key, hashed_key, _e)) => {
                    self.operation.clear();
                    self.hashed_key.replace(hashed_key);
                    Err((unhashed_key, value, ErrorCode::FAIL))
                }
            },
            None => Err((key, value, ErrorCode::FAIL)),
        }
    }

    fn set(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            ErrorCode,
        ),
    > {
        self.insert(key, value, Operation::Set)
    }

    fn add(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            ErrorCode,
        ),
    > {
        self.insert(key, value, Operation::Add)
    }

    fn update(
        &self,
        key: SubSliceMut<'static, u8>,
        value: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
            ErrorCode,
        ),
    > {
        self.insert(key, value, Operation::Update)
    }

    fn delete(
        &self,
        key: SubSliceMut<'static, u8>,
    ) -> Result<(), (SubSliceMut<'static, u8>, ErrorCode)> {
        if self.operation.is_some() {
            return Err((key, ErrorCode::BUSY));
        }

        self.operation.set(Operation::Delete);

        match self.hashed_key.take() {
            Some(hashed_key) => match self.kv.generate_key(key, hashed_key) {
                Ok(()) => Ok(()),
                Err((unhashed_key, hashed_key, _e)) => {
                    self.hashed_key.replace(hashed_key);
                    self.operation.clear();
                    Err((unhashed_key, ErrorCode::FAIL))
                }
            },
            None => Err((key, ErrorCode::FAIL)),
        }
    }
}

impl<'a, K: KVSystem<'a, K = T>, T: KeyType> KVSystemClient<T> for TicKVKVStore<'a, K, T> {
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
                                cb.get_complete(Err(ErrorCode::FAIL), unhashed_key, value);
                            });
                        });
                    }
                    Operation::Set => {
                        self.value.take().map(|value| {
                            self.client.map(move |cb| {
                                cb.set_complete(Err(ErrorCode::FAIL), unhashed_key, value);
                            });
                        });
                    }
                    Operation::Add => {
                        self.value.take().map(|value| {
                            self.client.map(move |cb| {
                                cb.add_complete(Err(ErrorCode::FAIL), unhashed_key, value);
                            });
                        });
                    }
                    Operation::Update => {
                        self.value.take().map(|value| {
                            self.client.map(move |cb| {
                                cb.update_complete(Err(ErrorCode::FAIL), unhashed_key, value);
                            });
                        });
                    }
                    Operation::Delete => {
                        self.client.map(move |cb| {
                            cb.delete_complete(Err(ErrorCode::FAIL), unhashed_key);
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
                                Err((key, value, _e)) => {
                                    self.hashed_key.replace(key);
                                    self.operation.clear();
                                    self.client.map(move |cb| {
                                        cb.get_complete(Err(ErrorCode::FAIL), unhashed_key, value);
                                    });
                                }
                            });
                    }
                    Operation::Set => {
                        self.value.take().map(|value| {
                            // Try to append which will work if the key is new.
                            match self.kv.append_key(hashed_key, value) {
                                Ok(()) => {
                                    self.unhashed_key.replace(unhashed_key);
                                }
                                Err((key, value, e)) => {
                                    self.hashed_key.replace(key);
                                    self.operation.clear();
                                    self.client.map(move |cb| {
                                        cb.set_complete(Err(e), unhashed_key, value);
                                    });
                                }
                            }
                        });
                    }
                    Operation::Add => {
                        self.value.take().map(|value| {
                            // Add only works if the key does not exist, so we
                            // can go right to append.
                            match self.kv.append_key(hashed_key, value) {
                                Ok(()) => {
                                    self.unhashed_key.replace(unhashed_key);
                                }
                                Err((key, value, e)) => {
                                    self.hashed_key.replace(key);
                                    self.operation.clear();
                                    self.client.map(move |cb| {
                                        cb.add_complete(Err(e), unhashed_key, value);
                                    });
                                }
                            }
                        });
                    }
                    Operation::Update => {
                        // Update requires the key to exist, so we start by
                        // trying to delete it.
                        match self.kv.invalidate_key(hashed_key) {
                            Ok(()) => {
                                self.unhashed_key.replace(unhashed_key);
                            }
                            Err((key, _e)) => {
                                self.hashed_key.replace(key);
                                self.operation.clear();
                                self.value.take().map(|value| {
                                    self.client.map(move |cb| {
                                        cb.update_complete(
                                            Err(ErrorCode::FAIL),
                                            unhashed_key,
                                            value,
                                        );
                                    });
                                });
                            }
                        }
                    }
                    Operation::Delete => {
                        match self.kv.invalidate_key(hashed_key) {
                            Ok(()) => {
                                self.unhashed_key.replace(unhashed_key);
                            }
                            Err((key, _e)) => {
                                self.hashed_key.replace(key);
                                self.operation.clear();
                                self.client.map(move |cb| {
                                    cb.delete_complete(Err(ErrorCode::FAIL), unhashed_key);
                                });
                            }
                        };
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
                        // need to delete the existing key.
                        self.hashed_key.take().map(|hashed_key| {
                            match self.kv.invalidate_key(hashed_key) {
                                Ok(()) => {
                                    self.value.replace(value);
                                }
                                Err((key, _e)) => {
                                    self.hashed_key.replace(key);
                                    self.operation.clear();
                                    self.unhashed_key.take().map(|unhashed_key| {
                                        self.client.map(move |cb| {
                                            cb.set_complete(
                                                Err(ErrorCode::FAIL),
                                                unhashed_key,
                                                value,
                                            );
                                        });
                                    });
                                }
                            }
                        });
                    }
                    _ => {
                        // On success or any other error we just return the
                        // result back to the caller via a callback.
                        self.operation.clear();
                        self.unhashed_key.take().map(|unhashed_key| {
                            self.client.map(move |cb| {
                                cb.set_complete(
                                    result.map_err(|e| match e {
                                        ErrorCode::NOMEM => ErrorCode::NOMEM,
                                        _ => ErrorCode::FAIL,
                                    }),
                                    unhashed_key,
                                    value,
                                );
                            });
                        });
                    }
                }
            }
            Operation::Add => {
                self.operation.clear();
                self.unhashed_key.take().map(|unhashed_key| {
                    self.client.map(move |cb| {
                        cb.add_complete(
                            result.map_err(|e| match e {
                                ErrorCode::NOSUPPORT => ErrorCode::NOSUPPORT,
                                ErrorCode::NOMEM => ErrorCode::NOMEM,
                                _ => ErrorCode::FAIL,
                            }),
                            unhashed_key,
                            value,
                        );
                    });
                });
            }
            Operation::Update => {
                self.operation.clear();
                self.unhashed_key.take().map(|unhashed_key| {
                    self.client.map(move |cb| {
                        cb.update_complete(
                            result.map_err(|e| match e {
                                ErrorCode::NOMEM => ErrorCode::NOMEM,
                                _ => ErrorCode::FAIL,
                            }),
                            unhashed_key,
                            value,
                        );
                    });
                });
            }
        });
    }

    fn get_value_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut T,
        ret_buf: SubSliceMut<'static, u8>,
    ) {
        self.operation.map(|op| match op {
            Operation::Get => {
                self.hashed_key.replace(key);
                self.operation.clear();

                self.unhashed_key.take().map(|unhashed_key| {
                    self.client.map(move |cb| {
                        cb.get_complete(
                            result.map_err(|e| match e {
                                ErrorCode::SIZE => ErrorCode::SIZE,
                                ErrorCode::NOSUPPORT => ErrorCode::NOSUPPORT,
                                _ => ErrorCode::FAIL,
                            }),
                            unhashed_key,
                            ret_buf,
                        );
                    });
                });
            }
            _ => {}
        });
    }

    fn invalidate_key_complete(&self, result: Result<(), ErrorCode>, key: &'static mut T) {
        self.hashed_key.replace(key);

        self.operation.map(|op| match op {
            Operation::Get | Operation::Add => {}
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
                                                cb.set_complete(Err(e), unhashed_key, value);
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
                                    cb.set_complete(Err(ErrorCode::FAIL), unhashed_key, value);
                                });
                            });
                        });
                    }
                }
            }
            Operation::Update => {
                // Now that we have deleted the existing key-value we can store
                // our new key and value.
                match result {
                    Ok(()) => {
                        self.hashed_key.take().map(|hashed_key| {
                            self.value.take().map(|value| {
                                match self.kv.append_key(hashed_key, value) {
                                    Ok(()) => {}
                                    Err((key, value, _e)) => {
                                        self.hashed_key.replace(key);
                                        self.operation.clear();
                                        self.unhashed_key.take().map(|unhashed_key| {
                                            self.client.map(move |cb| {
                                                cb.update_complete(
                                                    Err(ErrorCode::FAIL),
                                                    unhashed_key,
                                                    value,
                                                );
                                            });
                                        });
                                    }
                                }
                            });
                        });
                    }
                    _ => {
                        // Could not remove which means we can not update.
                        self.operation.clear();
                        self.unhashed_key.take().map(|unhashed_key| {
                            self.value.take().map(|value| {
                                self.client.map(move |cb| {
                                    cb.update_complete(
                                        Err(ErrorCode::NOSUPPORT),
                                        unhashed_key,
                                        value,
                                    );
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
