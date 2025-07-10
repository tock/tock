// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Interface for verifying signatures.

use crate::ErrorCode;

/// This trait provides callbacks for when the verification has completed.
pub trait ClientVerify<const HASH_LEN: usize, const SIGNATURE_LEN: usize> {
    /// Called when the verification is complete.
    ///
    /// If the verification operation encounters an error, result will be a
    /// `Result::Err()` specifying the ErrorCode. Otherwise, result will be a
    /// `Result::Ok` set to `Ok(true)` if the signature was correctly verified
    /// and `Ok(false)` otherwise.
    ///
    /// If verification operation did encounter errors `result` will be `Err()`
    /// with an appropriate `ErrorCode`. Valid `ErrorCode`s include:
    ///
    /// - `CANCEL`: the operation was cancelled.
    /// - `FAIL`: an internal failure.
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        hash: &'static mut [u8; HASH_LEN],
        signature: &'static mut [u8; SIGNATURE_LEN],
    );
}

/// Verify a signature.
///
/// This is a generic interface, and it is up to the implementation as to the
/// signature verification algorithm being used.
///
/// - `HASH_LEN`: The length in bytes of the hash.
/// - `SIGNATURE_LEN`: The length in bytes of the signature.
pub trait SignatureVerify<'a, const HASH_LEN: usize, const SIGNATURE_LEN: usize> {
    /// Set the client instance which will receive the `verification_done()`
    /// callback.
    fn set_verify_client(&self, client: &'a dyn ClientVerify<HASH_LEN, SIGNATURE_LEN>);

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
        hash: &'static mut [u8; HASH_LEN],
        signature: &'static mut [u8; SIGNATURE_LEN],
    ) -> Result<
        (),
        (
            ErrorCode,
            &'static mut [u8; HASH_LEN],
            &'static mut [u8; SIGNATURE_LEN],
        ),
    >;
}

/// This trait provides callbacks for when the signing has completed.
pub trait ClientSign<const HL: usize, const SL: usize> {
    /// Called when the signing is complete.
    ///
    /// If the signing operation encounters an error, result will be a
    /// `Result::Err()` specifying the ErrorCode. Otherwise, result will be
    /// a `Result::Ok(())`.
    ///
    /// If signing operation did encounter errors `result` will be `Err()`
    /// with an appropriate `ErrorCode`. Valid `ErrorCode`s include:
    ///
    /// - `CANCEL`: the operation was cancelled.
    /// - `FAIL`: an internal failure.
    fn signing_done(
        &self,
        result: Result<(), ErrorCode>,
        hash: &'static mut [u8; HL],
        signature: &'static mut [u8; SL],
    );
}

/// Sign a message.
///
/// This is a generic interface, and it is up to the implementation as to the
/// signing algorithm being used.
///
/// - `HL`: The length in bytes of the hash.
/// - `SL`: The length in bytes of the signature.
pub trait SignatureSign<'a, const HL: usize, const SL: usize> {
    /// Set the client instance which will receive the `signing_done()`
    /// callback.
    fn set_sign_client(&self, client: &'a dyn ClientSign<HL, SL>);

    /// Sign the given hash.
    ///
    /// If this returns `Ok(())`, then the `signing_done()` callback will
    /// be called. If this returns `Err()`, no callback will be called.
    ///
    /// The valid `ErrorCode`s that can occur are:
    ///
    /// - `OFF`: the underlying digest engine is powered down and cannot be
    ///   used.
    /// - `BUSY`: there is an outstanding operation already in process, and the
    ///   signing engine cannot accept another request.
    fn sign(
        &self,
        hash: &'static mut [u8; HL],
        signature: &'static mut [u8; SL],
    ) -> Result<(), (ErrorCode, &'static mut [u8; HL], &'static mut [u8; SL])>;
}
