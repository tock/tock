// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for RSA Public/Private key encryption math operations

use crate::ErrorCode;

/// Upcall from the `RsaCryptoBase` trait.
pub trait Client<'a> {
    /// This callback is called when the mod_exponent operation is complete.
    ///
    /// The possible ErrorCodes are:
    ///    - BUSY: An operation is already on going
    ///    - INVAL: An invalid parameter was supplied
    ///    - SIZE: The size of the `result` buffer is invalid
    ///    - NOSUPPORT: The operation is not supported
    fn mod_exponent_done(
        &'a self,
        status: Result<bool, ErrorCode>,
        message: &'static mut [u8],
        modulus: &'static [u8],
        exponent: &'static [u8],
        result: &'static mut [u8],
    );
}

pub trait RsaCryptoBase<'a> {
    /// Set the `Client` client to be called on completion.
    fn set_client(&'a self, client: &'a dyn Client<'a>);

    /// Clear any confidential data.
    fn clear_data(&self);

    /// Calculate (`message` ^ `exponent`) % `modulus` and store it in the
    /// `result` buffer.
    ///
    /// On completion the `mod_exponent_done()` upcall will be scheduled.
    ///
    /// The length of `modulus` must be a power of 2 and determines the length
    /// of the operation.
    ///
    /// The `message` and `exponent` buffers can be any length. All of the data
    /// in the buffer up to the length of the `modulus` will be used. This
    /// allows callers to allocate larger buffers to support multiple
    /// RSA lengths, but only the operation length (defined by the modulus)
    /// will be used.
    ///
    /// The `result` buffer must be at least as large as the `modulus` buffer,
    /// otherwise Err(SIZE) will be returned.
    /// If `result` is longer then `modulus` the data will be stored in the
    /// `result` buffer from 0 to `modulue.len()`.
    ///
    /// The possible ErrorCodes are:
    ///    - BUSY: An operation is already on going
    ///    - INVAL: An invalid parameter was supplied
    ///    - SIZE: The size of the `result` buffer is invalid
    ///    - NOSUPPORT: The operation is not supported
    fn mod_exponent(
        &self,
        message: &'static mut [u8],
        modulus: &'static [u8],
        exponent: &'static [u8],
        result: &'static mut [u8],
    ) -> Result<
        (),
        (
            ErrorCode,
            &'static mut [u8],
            &'static [u8],
            &'static [u8],
            &'static mut [u8],
        ),
    >;
}

/// Upcall from the `RsaCryptoBase` trait.
pub trait ClientMut<'a> {
    /// This callback is called when the mod_exponent operation is complete.
    ///
    /// The possible ErrorCodes are:
    ///    - BUSY: The system is busy
    ///    - ALREADY: An operation is already on going
    ///    - INVAL: An invalid parameter was supplied
    ///    - SIZE: The size of the `result` buffer is invalid
    ///    - NOSUPPORT: The operation is not supported
    fn mod_exponent_done(
        &'a self,
        status: Result<bool, ErrorCode>,
        message: &'static mut [u8],
        modulus: &'static mut [u8],
        exponent: &'static mut [u8],
        result: &'static mut [u8],
    );
}

pub trait RsaCryptoBaseMut<'a> {
    /// Set the `ClientMut` client to be called on completion.
    fn set_client(&'a self, client: &'a dyn ClientMut<'a>);

    /// Clear any confidential data.
    fn clear_data(&self);

    /// Calculate (`message` ^ `exponent`) % `modulus` and store it in the
    /// `result` buffer.
    ///
    /// On completion the `mod_exponent_done()` upcall will be scheduled.
    ///
    /// The length of `modulus` must be a power of 2 and determines the length
    /// of the operation.
    ///
    /// The `message` and `exponent` buffers can be any length. All of the data
    /// in the buffer up to the length of the `modulus` will be used. This
    /// allows callers to allocate larger buffers to support multiple
    /// RSA lengths, but only the operation length (defined by the modulus)
    /// will be used.
    ///
    /// The `result` buffer must be at least as large as the `modulus` buffer,
    /// otherwise Err(SIZE) will be returned.
    /// If `result` is longer then `modulus` the data will be stored in the
    /// `result` buffer from 0 to `modulue.len()`.
    ///
    /// The possible ErrorCodes are:
    ///    - BUSY: The system is busy
    ///    - ALREADY: An operation is already on going
    ///    - INVAL: An invalid parameter was supplied
    ///    - SIZE: The size of the `result` buffer is invalid
    ///    - NOSUPPORT: The operation is not supported
    fn mod_exponent(
        &self,
        message: &'static mut [u8],
        modulus: &'static mut [u8],
        exponent: &'static mut [u8],
        result: &'static mut [u8],
    ) -> Result<
        (),
        (
            ErrorCode,
            &'static mut [u8],
            &'static mut [u8],
            &'static mut [u8],
            &'static mut [u8],
        ),
    >;
}
