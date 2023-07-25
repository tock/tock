// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for Hasher

use crate::utilities::leasable_buffer::SubSlice;
use crate::utilities::leasable_buffer::SubSliceMut;
use crate::ErrorCode;

/// Implement this trait and use `set_client()` in order to receive callbacks.
///
/// 'L' is the length of the 'u8' array to store the hash output.
pub trait Client<const L: usize> {
    /// This callback is called when the data has been added to the hash
    /// engine.
    /// On error or success `data` will contain a reference to the original
    /// data supplied to `add_data()`.
    /// The possible ErrorCodes are:
    ///    - SIZE: The size of the `data` buffer is invalid
    fn add_data_done(&self, result: Result<(), ErrorCode>, data: SubSlice<'static, u8>);

    /// This callback is called when the data has been added to the hash
    /// engine.
    /// On error or success `data` will contain a reference to the original
    /// data supplied to `add_mut_data()`.
    /// The possible ErrorCodes are:
    ///    - SIZE: The size of the `data` buffer is invalid
    fn add_mut_data_done(&self, result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>);

    /// This callback is called when a hash is computed.
    /// On error or success `hash` will contain a reference to the original
    /// data supplied to `run()`.
    /// The possible ErrorCodes are:
    ///    - SIZE: The size of the `data` buffer is invalid
    fn hash_done(&self, result: Result<(), ErrorCode>, hash: &'static mut [u8; L]);
}

/// Computes a non-cryptographic hash over data
///
/// 'L' is the length of the 'u8' array to store the hash output.
pub trait Hasher<'a, const L: usize> {
    /// Set the client instance which will receive `hash_done()` and
    /// `add_data_done()` callbacks.
    /// This callback is called when the data has been added to the hash
    /// engine.
    /// The callback should follow the `Client` `add_data_done` callback.
    fn set_client(&'a self, client: &'a dyn Client<L>);

    /// Add data to the hash block. This is the data that will be used
    /// for the hash function.
    /// Returns the number of bytes parsed on success
    /// There is no guarantee the data has been written until the `add_data_done()`
    /// callback is fired.
    /// On error the return value will contain a return code and the original data
    /// The possible ErrorCodes are:
    ///    - BUSY: The system is busy performing an operation
    ///            The caller should expect a callback
    ///    - SIZE: The size of the `data` buffer is invalid
    fn add_data(
        &self,
        data: SubSlice<'static, u8>,
    ) -> Result<usize, (ErrorCode, SubSlice<'static, u8>)>;

    /// Add data to the hash block. This is the data that will be used
    /// for the hash function.
    /// Returns the number of bytes parsed on success
    /// There is no guarantee the data has been written until the `add_data_done()`
    /// callback is fired.
    /// On error the return value will contain a return code and the original data
    /// The possible ErrorCodes are:
    ///    - BUSY: The system is busy performing an operation
    ///            The caller should expect a callback
    ///    - SIZE: The size of the `data` buffer is invalid
    fn add_mut_data(
        &self,
        data: SubSliceMut<'static, u8>,
    ) -> Result<usize, (ErrorCode, SubSliceMut<'static, u8>)>;

    /// Request the implementation to generate a hash and stores the returned
    /// hash in the memory location specified.
    /// This doesn't return any data, instead the client needs to have
    /// set a `hash_done` handler to determine when this is complete.
    /// On error the return value will contain a return code and the original data
    /// If there is data from the `add_data()` command asyncrously waiting to
    /// be written it will be written before the operation starts.
    /// The possible ErrorCodes are:
    ///    - BUSY: The system is busy performing an operation
    ///            The caller should expect a callback
    ///    - SIZE: The size of the `data` buffer is invalid
    fn run(&'a self, hash: &'static mut [u8; L]) -> Result<(), (ErrorCode, &'static mut [u8; L])>;

    /// Clear the internal state of the engine.
    /// This won't clear the buffers provided to this API, that is up to the
    /// user to clear.
    fn clear_data(&self);
}

pub trait SipHash {
    /// Optionaly call before `Hasher::run()` to specify the keys used
    /// The possible ErrorCodes are:
    ///    - BUSY: The system is busy
    ///    - ALREADY: An operation is already on going
    ///    - INVAL: An invalid parameter was supplied
    ///    - NOSUPPORT: The operation is not supported
    fn set_keys(&self, k0: u64, k1: u64) -> Result<(), ErrorCode>;
}
