// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for verifying signatures.

use crate::ErrorCode;

/// This trait provides callbacks for when the verification has completed.
pub trait ClientVerify<const H: usize, const S: usize> {
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        hash: &'static mut [u8; H],
        signature: &'static mut [u8; S],
    );
}

/// Verify a signature.
///
/// - `H`: The length in bytes of the hash.
/// - `S`: The length in bytes of the signature.
pub trait SignatureVerify<'a, const H: usize, const S: usize> {
    /// Set the client instance which will receive the `verification_done()`
    /// callback.
    #[allow(unused_variables)]
    fn set_verify_client(&'a self, client: &'a dyn ClientVerify<H, S>) {}

    // Verify the signature. Returns `Ok(())` if the signature matches.
    fn verify(
        &'a self,
        hash: &'static mut [u8; H],
        signature: &'static mut [u8; S],
    ) -> Result<(), (ErrorCode, &'static mut [u8; H], &'static mut [u8; S])>;
}
