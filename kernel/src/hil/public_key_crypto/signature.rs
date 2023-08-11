// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for verifying signatures.

use crate::ErrorCode;

/// This trait provides callbacks for when the verification has completed.
pub trait ClientVerify<const HL: usize, const SL: usize> {
    /// Called when the verification is complete.
    ///
    /// If the verification operation did not encounter any errors, `result`
    /// will be set to `Ok()`. If the signature was correctly verified `result`
    /// will be `Ok(true)`. If the signature did not match the hash `result`
    /// will be `Ok(false)`.
    ///
    /// If verification operation did encounter errors `result` will be `Err()`
    /// with an appropriate `ErrorCode`.
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        hash: &'static mut [u8; HL],
        signature: &'static mut [u8; SL],
    );
}

/// Verify a signature.
///
/// - `HL`: The length in bytes of the hash.
/// - `SL`: The length in bytes of the signature.
pub trait SignatureVerify<'a, const HL: usize, const SL: usize> {
    /// Set the client instance which will receive the `verification_done()`
    /// callback.
    fn set_verify_client(&self, client: &'a dyn ClientVerify<HL, SL>);

    /// Verify the signature.
    fn verify(
        &self,
        hash: &'static mut [u8; HL],
        signature: &'static mut [u8; SL],
    ) -> Result<(), (ErrorCode, &'static mut [u8; HL], &'static mut [u8; SL])>;
}
