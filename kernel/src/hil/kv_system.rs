// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Low level interface for Key-Value (KV) Stores
//!
//! The KV store implementation in Tock has three levels, described below.
//!
//! 1 - Hardware Level:
//! This level is the interface that writes a buffer to the hardware. This will
//! generally be writing to flash, although in theory it would be possible to
//! write to other mediums.
//!
//! An example of the HIL used here is the Tock Flash HIL.
//!
//! 2 - KV System Level:
//! This level can be thought of like a file system. It is responsible for
//! taking save/load operations and generating a buffer to pass to level 1
//! This level is also in charge of generating hashes and checksums.
//!
//! This level allows generating a key hash but otherwise operates on
//! hashed keys. This level is not responsible for permission checks.
//!
//! This file describes the HIL for this level.
//!
//! 3 - KV Store:
//! This is a user friendly high level API. This API is used inside the kernel
//! and exposed to applications to allow KV operations. The API from this level
//! should be high level, for example set/get/delete on unhashed keys.
//! This level is in charge of enforcing permissions.
//!
//! This level is also in charge of generating the key hash by calling into
//! level 2.
//!
//! The expected setup inside Tock will look like this:
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
//! |  K-V in Tock          |
//! |                       |
//! +-----------------------+
//!
//!    hil::kv_system (this file)
//!
//! +-----------------------+
//! |                       |
//! |  K-V library          |
//! |                       |
//! +-----------------------+
//!
//!    hil::flash

use crate::ErrorCode;

/// The type of keys, this should define the output size of the digest
/// operations.
pub trait KeyType: Eq + Copy + Clone + Sized + AsRef<[u8]> + AsMut<[u8]> {}

impl KeyType for [u8; 8] {}

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait StoreClient<K: KeyType> {
    /// This callback is called when the get operation completes
    ///
    /// `result`: Nothing on success, 'ErrorCode' on error
    /// `key`: The key buffer
    /// `ret_buf`: The ret_buf buffer
    fn get_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut [u8],
        ret_buf: &'static mut [u8],
    );

    /// This callback is called when the set operation completes
    ///
    /// `result`: Nothing on success, 'ErrorCode' on error
    /// `key`: The key buffer
    /// `value`: The value buffer
    fn set_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut [u8],
        value: &'static mut [u8],
    );

    /// This callback is called when the delete operation completes
    ///
    /// `result`: Nothing on success, 'ErrorCode' on error
    /// `key`: The key buffer
    fn delete_complete(&self, result: Result<(), ErrorCode>, key: &'static mut [u8]);
}

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait Client<K: KeyType> {
    /// This callback is called when the append_key operation completes
    ///
    /// `result`: Nothing on success, 'ErrorCode' on error
    /// `unhashed_key`: The unhashed_key buffer
    /// `key_buf`: The key_buf buffer
    fn generate_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        unhashed_key: &'static mut [u8],
        key_buf: &'static mut K,
    );

    /// This callback is called when the append_key operation completes
    ///
    /// `result`: Nothing on success, 'ErrorCode' on error
    /// `key`: The key buffer
    /// `value`: The value buffer
    fn append_key_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut K,
        value: &'static mut [u8],
    );

    /// This callback is called when the get_value operation completes
    ///
    /// `result`: Nothing on success, 'ErrorCode' on error
    /// `key`: The key buffer
    /// `ret_buf`: The ret_buf buffer
    fn get_value_complete(
        &self,
        result: Result<(), ErrorCode>,
        key: &'static mut K,
        ret_buf: &'static mut [u8],
    );

    /// This callback is called when the invalidate_key operation completes
    ///
    /// `result`: Nothing on success, 'ErrorCode' on error
    /// `key`: The key buffer
    fn invalidate_key_complete(&self, result: Result<(), ErrorCode>, key: &'static mut K);

    /// This callback is called when the garbage_collect operation completes
    ///
    /// `result`: Nothing on success, 'ErrorCode' on error
    fn garbage_collect_complete(&self, result: Result<(), ErrorCode>);
}

pub trait KVSystem<'a> {
    /// The type of the hashed key. For example '[u8; 8]'.
    type K: KeyType;

    /// Set the client
    fn set_client(&self, client: &'a dyn Client<Self::K>);

    /// Generate key
    ///
    /// `unhashed_key`: A unhashed key that should be hashed.
    /// `key_buf`: A buffer to store the hashed key output.
    ///
    /// On success returns nothing.
    /// On error the unhashed_key, key_buf and `Result<(), ErrorCode>` will be returned.
    fn generate_key(
        &self,
        unhashed_key: &'static mut [u8],
        key_buf: &'static mut Self::K,
    ) -> Result<
        (),
        (
            &'static mut [u8],
            &'static mut Self::K,
            Result<(), ErrorCode>,
        ),
    >;

    /// Appends the key/value pair.
    ///
    /// If the key already exists in the store and has not been invalidated then
    /// the append operation will fail. To update an existing key to a new value
    /// the key must first be invalidated.
    ///
    /// `key`: A hashed key. This key will be used in future to retrieve
    ///        or remove the `value`.
    /// `value`: A buffer containing the data to be stored to flash.
    ///
    /// On success nothing will be returned.
    /// On error the key, value and a `Result<(), ErrorCode>` will be returned.
    ///
    /// The possible `Result<(), ErrorCode>`s are:
    ///    `BUSY`: An operation is already in progress
    ///    `INVAL`: An invalid parameter was passed
    ///    `NODEVICE`: No KV store was setup
    ///    `NOSUPPORT`: The key could not be added due to a collision.
    ///    `NOMEM`: The key could not be added due to no more space.
    fn append_key(
        &self,
        key: &'static mut Self::K,
        value: &'static mut [u8],
    ) -> Result<
        (),
        (
            &'static mut Self::K,
            &'static mut [u8],
            Result<(), ErrorCode>,
        ),
    >;

    /// Retrieves the value from a specified key.
    ///
    /// `key`: A hashed key. This key will be used to retrieve the `value`.
    /// `ret_buf`: A buffer to store the value to.
    ///
    /// On success nothing will be returned.
    /// On error the key, ret_buf and a `Result<(), ErrorCode>` will be returned.
    ///
    /// The possible `Result<(), ErrorCode>`s are:
    ///    `BUSY`: An operation is already in progress
    ///    `INVAL`: An invalid parameter was passed
    ///    `NODEVICE`: No KV store was setup
    ///    `ENOSUPPORT`: The key could not be found.
    fn get_value(
        &self,
        key: &'static mut Self::K,
        ret_buf: &'static mut [u8],
    ) -> Result<
        (),
        (
            &'static mut Self::K,
            &'static mut [u8],
            Result<(), ErrorCode>,
        ),
    >;

    /// Invalidates the key in flash storage
    ///
    /// `key`: A hashed key. This key will be used to remove the `value`.
    ///
    /// On success nothing will be returned.
    /// On error the key and a `Result<(), ErrorCode>` will be returned.
    ///
    /// The possible `Result<(), ErrorCode>`s are:
    ///    `BUSY`: An operation is already in progress
    ///    `INVAL`: An invalid parameter was passed
    ///    `NODEVICE`: No KV store was setup
    ///    `ENOSUPPORT`: The key could not be found.
    fn invalidate_key(
        &self,
        key: &'static mut Self::K,
    ) -> Result<(), (&'static mut Self::K, Result<(), ErrorCode>)>;

    /// Perform a garbage collection on the KV Store
    ///
    /// For implementations that don't require garbage collecting
    /// this can just be a NOP that returns 'Ok(0)'.
    ///
    /// On success the number of bytes freed will be returned.
    /// On error a `Result<(), ErrorCode>` will be returned.
    ///
    /// The possible `Result<(), ErrorCode>`s are:
    ///    `BUSY`: An operation is already in progress
    ///    `INVAL`: An invalid parameter was passed
    ///    `NODEVICE`: No KV store was setup
    fn garbage_collect(&self) -> Result<usize, Result<(), ErrorCode>>;
}
