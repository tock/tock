// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Interface for verifying signatures.

use crate::ErrorCode;

/// Type of a signature algorithm and the data structure needed to store the
/// signature.
pub trait SignatureAlgorithm: Default {
    /// Get a slice to the underlying data.
    fn as_slice(&self) -> &[u8];
    /// Get a mutable slice to the underlying data.
    fn as_mut_slice(&mut self) -> &mut [u8];
}

/// RSA 2048 Signature
struct Rsa2048Signature([u8; 256]);
impl SignatureAlgorithm for Rsa2048Signature {
    fn as_slice(&self) -> &[u8] {
        &self.0
    }
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
impl Default for Rsa2048Signature {
    fn default() -> Self {
        Rsa2048Signature([0; 256])
    }
}

/// This trait provides callbacks for when the verification has completed.
pub trait ClientVerify<D: crate::hil::digest::DigestAlgorithm, S: SignatureAlgorithm> {
    /// Called when the verification is complete.
    ///
    /// If the verification operation did not encounter any errors, `result`
    /// will be set to `Ok()`. If the signature was correctly verified `result`
    /// will be `Ok(true)`. If the signature did not match the hash `result`
    /// will be `Ok(false)`.
    ///
    /// If verification operation did encounter errors `result` will be `Err()`
    /// with an appropriate `ErrorCode`. Valid `ErrorCode`s include:
    ///
    /// - `CANCEL`: the operation was cancelled.
    /// - `FAIL`: an internal failure.
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        hash: &'static mut D,
        signature: &'static mut S,
    );
}

/// Verify a signature.
///
/// This is a generic interface, and it is up to the implementation as to the
/// signature verification algorithm being used.
///
/// - `D`: The digest used for the signature.
/// - `S`: The signature calculation algorithm.
pub trait SignatureVerify<'a, D: crate::hil::digest::DigestAlgorithm, S: SignatureAlgorithm> {
    /// Set the client instance which will receive the `verification_done()`
    /// callback.
    fn set_verify_client(&self, client: &'a dyn ClientVerify<D, S>);

    /// Verify the signature matches the given hash.
    ///
    /// If this returns `Ok(())`, then the `verification_done()` callback will
    /// be called. If this returns `Err()`, no callback will be called.
    ///
    /// The valid `ErrorCode`s that can occur are:
    ///
    /// - `OFF`: the underlying digest engine is powered down and cannot be
    ///   used.
    /// - `BUSY`: there is an outstanding operation already in process, and the
    ///   verification engine cannot accept another request.
    fn verify(
        &self,
        hash: &'static mut D,
        signature: &'static mut S,
    ) -> Result<(), (ErrorCode, &'static mut D, &'static mut S)>;
}
