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
//! This is a user friendly high level HIL. This HIL is used inside the kernel
//! and exposed to applications to allow KV operations. The API from this level
//! should be high level, for example set/get/delete on unhashed keys.
//! This level is in charge of enforcing permissions. It is expected that
//! this level will combine the user data with a header and pass that to the
//! level 2 system HIL.
//!
//! This level is also in charge of generating the key hash by calling into
//! level 2.
//!
//! There is currently no implementation of this HIL in Tock.
//!
//! The expected setup inside Tock will look like this:
//! +-----------------------+
//! |                       |
//! |  Capsule using K-V    |
//! |                       |
//! +-----------------------+
//!
//!    hil::kv_store (not written yet)
//!
//! +-----------------------+
//! |                       |
//! |  K-V in Tock          |
//! |                       |
//! +-----------------------+
//!
//!    hil::kv_system (this PR)
//!
//! +-----------------------+
//! |                       |
//! |  K-V library          |
//! |                       |
//! +-----------------------+
//!
//!    hil::flash

use crate::returncode::ReturnCode;

/// The type of keys, this should define the output size of the digest
/// operations.
pub trait KeyType: Eq + Copy + Clone + Sized + AsRef<[u8]> + AsMut<[u8]> {}

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait Client<'a, K: KeyType> {
    /// This callback is called when the append_key operation completes
    ///
    /// `result`: Nothing on success, 'ReturnCode' on error
    /// `unhashed_key`: The unhashed_key buffer
    /// `key_buf`: The key_buf buffer
    fn generate_key_complete(
        &'a self,
        result: Result<(), ReturnCode>,
        unhashed_key: &'static [u8],
        key_buf: &'static K,
    );

    /// This callback is called when the append_key operation completes
    ///
    /// `result`: Nothing on success, 'ReturnCode' on error
    /// `key`: The key buffer
    /// `value`: The value buffer
    fn append_key_complete(
        &'a self,
        result: Result<(), ReturnCode>,
        key: &'static mut K,
        value: &'static mut [u8],
    );

    /// This callback is called when the get_value operation completes
    ///
    /// `result`: Nothing on success, 'ReturnCode' on error
    /// `key`: The key buffer
    /// `ret_buf`: The ret_buf buffer
    fn get_value_complete(
        &'a self,
        result: Result<(), ReturnCode>,
        key: &'static mut K,
        ret_buf: &'static mut [u8],
    );

    /// This callback is called when the invalidate_key operation completes
    ///
    /// `result`: Nothing on success, 'ReturnCode' on error
    /// `key`: The key buffer
    fn invalidate_key_complete(&'a self, result: Result<(), ReturnCode>, key: &'static mut K);

    /// This callback is called when the garbage_collect operation completes
    ///
    /// `result`: Nothing on success, 'ReturnCode' on error
    fn garbage_collect_complete(&'a self, result: Result<(), ReturnCode>, key: &'static mut K);
}

pub trait KVSystem {
    /// The type of the hashed key. For example '[u8; 64]'.
    type K: KeyType;

    /// Set the client
    fn set_client(&self, client: &dyn Client<Self::K>);

    /// Generate key
    ///
    /// `unhashed_key`: A unhashed key that should be hashed.
    /// `key_buf`: A buffer to store the hashed key output.
    ///
    /// On success returns nothing.
    /// On error the unhashed_key, key_buf and `ReturnCode` will be returned.
    fn generate_key(
        &self,
        unhashed_key: &'static [u8],
        key_buf: &'static Self::K,
    ) -> Result<(), (&'static [u8], &'static Self::K, ReturnCode)>;

    /// Appends the key/value pair.
    ///
    /// `key`: A hashed key. This key will be used in future to retrieve
    ///        or remove the `value`.
    /// `value`: A buffer containing the data to be stored to flash.
    ///
    /// On success nothing will be returned.
    /// On error the key, value and a `ReturnCode` will be returned.
    ///
    /// The possible `ReturnCode`s are:
    ///    `EBUSY`: An operation is already in progress
    ///    `EINVAL`: An invalid parameter was passed
    ///    `ENODEVICE`: No KV store was setup
    ///    `ENOSUPPORT`: The key could not be added due to a collision.
    ///    `ENOMEM`: The key could not be added due to no more space.
    fn append_key(
        &self,
        key: &'static Self::K,
        value: &'static [u8],
    ) -> Result<(), (&'static Self::K, &'static [u8], ReturnCode)>;

    /// Retrieves the value from a specified key.
    ///
    /// `key`: A hashed key. This key will be used to retrieve the `value`.
    /// `ret_buf`: A buffer to store the value to.
    ///
    /// On success nothing will be returned.
    /// On error the key, ret_buf and a `ReturnCode` will be returned.
    ///
    /// The possible `ReturnCode`s are:
    ///    `EBUSY`: An operation is already in progress
    ///    `EINVAL`: An invalid parameter was passed
    ///    `ENODEVICE`: No KV store was setup
    ///    `ENOSUPPORT`: The key could not be found.
    fn get_value(
        &self,
        key: &'static Self::K,
        ret_buf: &'static mut [u8],
    ) -> Result<(), (&'static Self::K, &'static [u8], ReturnCode)>;

    /// Invalidates the key in flash storage
    ///
    /// `key`: A hashed key. This key will be used to remove the `value`.
    ///
    /// On success nothing will be returned.
    /// On error the key and a `ReturnCode` will be returned.
    ///
    /// The possible `ReturnCode`s are:
    ///    `EBUSY`: An operation is already in progress
    ///    `EINVAL`: An invalid parameter was passed
    ///    `ENODEVICE`: No KV store was setup
    ///    `ENOSUPPORT`: The key could not be found.
    fn invalidate_key(&self, key: &'static Self::K) -> Result<(), (&'static Self::K, ReturnCode)>;

    /// Perform a garbage collection on the KV Store
    ///
    /// For implementations that don't require garbage collecting
    /// this can just be a NOP that returns 'Ok(0)'.
    ///
    /// On success the number of bytes freed will be returned.
    /// On error a `ReturnCode` will be returned.
    ///
    /// The possible `ReturnCode`s are:
    ///    `EBUSY`: An operation is already in progress
    ///    `EINVAL`: An invalid parameter was passed
    ///    `ENODEVICE`: No KV store was setup
    fn garbage_collect(&self) -> Result<usize, ReturnCode>;
}
