// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for symmetric-cipher encryption
//!
//! (TODO) Update usage example.

use crate::{
    utilities::leasable_buffer::{SubSlice, SubSliceMut},
    ErrorCode,
};

/// Implement this trait and use `set_client()` in order to receive callbacks from an `AES128`
/// instance. This returns the provided source. The `dest` contains the result of the operation
/// and in both cases the `SubSliceMut` provided to `crypt()` will be returned.
pub trait Client<'a> {
    fn crypt_done(
        &'a self,
        source: Option<SubSlice<'static, u8>>,
        dest: Result<SubSliceMut<'static, u8>, (ErrorCode, SubSliceMut<'static, u8>)>,
    );
}

/// The number of bytes used for AES block operations.  Keys and IVs must have this length,
/// and encryption/decryption inputs must be have a multiple of this length.
pub const AES128_BLOCK_SIZE: usize = 16;
pub const AES128_KEY_SIZE: usize = 16;

/// Implement this trait for hardware supported AES128 operation.
pub trait AES128<'a> {
    /// Set the client instance which will receive `crypt_done()` callbacks
    fn set_client(&'a self, client: &'a dyn Client<'a>);

    /// Set the encryption key.
    /// Returns `INVAL` if length is not `AES128_KEY_SIZE`
    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode>;

    /// Set the IV (or initial counter).
    /// Returns `INVAL` if length is not `AES128_BLOCK_SIZE`
    fn set_iv(&self, iv: &[u8]) -> Result<(), ErrorCode>;

    /// Begin a new message (with the configured IV) when `crypt()` is
    /// next called.  Multiple calls to `crypt()` may be made between
    /// calls to `start_message()`, allowing the encryption context to
    /// extend over non-contiguous extents of data.
    ///
    /// If an encryption operation is in progress, this method instead
    /// has no effect.
    fn start_message(&self);

    /// Request an encryption/decryption
    ///
    /// If the source buffer is `Some`, the active region of the source
    /// SubSlice serves as the crypt input. Otherwise the destination buffer
    /// from the active subslice region will provide the input, which
    /// will be overwritten.
    ///
    /// If `Ok(())` is returned, the client's `crypt_done` method will eventually
    /// be called, and the portion of the active region of the destination buffer
    /// will hold the result of the encryption/decryption.
    ///
    /// If `Err(result, source, dest)` is returned, `result` is the
    /// error condition and `source` and `dest` are the buffers that
    /// were passed to `crypt`.
    ///
    /// The active regions of the `source` and `dest` subslice must be the same
    /// length (if a source buffer is provided) and a multiple of `AES128_BLOCK_SIZE`.
    /// Otherwise, `Err(INVAL, ...)` will be returned.
    ///
    /// If an encryption operation is already in progress,
    /// `Err(BUSY, ...)` will be returned.
    ///
    /// For correct operation, the methods `set_key` and `set_iv` must have
    /// previously been called to set the buffers containing the
    /// key and the IV (or initial counter value), and a method `set_mode_*()`
    /// must have been called to set the desired mode.  These settings persist
    /// across calls to `crypt()`.
    ///
    fn crypt(
        &self,
        source: Option<SubSlice<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSlice<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    >;
}

/// Implement this trait for AES128 hardware that supports Ctr mode.
pub trait AES128Ctr<'a>: AES128<'a> {
    /// Call before `AES128::crypt()` to perform AES128Ctr.
    fn set_mode_aes128ctr(&self, encrypting: bool) -> Result<(), ErrorCode>;
}

/// Implement this trait for AES128 hardware that supports CBC mode.
pub trait AES128CBC<'a>: AES128<'a> {
    /// Call before `AES128::crypt()` to perform AES128CBC.
    fn set_mode_aes128cbc(&self, encrypting: bool) -> Result<(), ErrorCode>;
}

/// Implement this trait for AES128 hardware that supports ECB mode.
pub trait AES128ECB<'a>: AES128<'a> {
    /// Call before `AES128::crypt()` to perform AES128ECB.
    fn set_mode_aes128ecb(&self, encrypting: bool) -> Result<(), ErrorCode>;
}

/// Implement this trait for AES128 hardware that supports CCM mode.
pub trait AES128CCM<'a>: AES128<'a> {
    /// Call before `AES128::crypt()` to perform AES128CCM.
    fn set_mode_aes128ccm(&self, encrypting: bool) -> Result<(), ErrorCode>;
}

/// Implement this trait for AES128 hardware that supports GCM mode.
pub trait AES128GCM<'a>: AES128<'a> {
    /// Call before `AES128::crypt()` to perform AES128GCM.
    fn set_mode_aes128gcm(&self, encrypting: bool) -> Result<(), ErrorCode>;
}
