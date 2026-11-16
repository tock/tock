// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interfaces for message digest algorithms performing calculations in one run.

use crate::{ErrorCode, hil::crypto::digest::DigestAlgorithm, processbuffer::ReadableProcessSlice};

/// Computes a message digest using algorithm `A`.
pub trait Digest<A: DigestAlgorithm> {
    /// Initiate a digest operation over `len` bytes of input.
    ///
    /// Input is retrieved through [`DigestClient::read_input`]. The computed digest is returned
    /// through [`DigestClient::write_digest`], followed by exactly one
    /// [`DigestClient::digest_done`] callback. Callbacks must not occur before this method returns.
    ///
    /// Returns [`ErrorCode::BUSY`] if an operation is already in progress, [`ErrorCode::RESERVE`]
    /// if no client is registered, or [`ErrorCode::SIZE`] if `len` exceeds an implementation
    /// limit known when this method is called. On `Ok(())`, a
    /// completion callback will occur. On `Err`, no callbacks will occur for this request.
    fn digest(&self, len: usize) -> Result<(), ErrorCode>;

    /// Set the client that will receive callbacks for digest operations.
    fn set_client(&self, client: &'static dyn DigestClient<A>);
}

/// Client callbacks for [`Digest`] operations.
pub trait DigestClient<A: DigestAlgorithm> {
    /// Supply input to hash.
    ///
    /// The client passes the unconsumed bytes of the message to `sink`, which absorbs a portion of the slice
    /// and return the number of bytes consumed. A driver may issue this callback multiple times,
    /// but each destination must fit within the remaining input. The total number of bytes consumed
    /// must equal the `len` passed to [`Digest::digest`].
    /// Follow the [crypto data movement contract](super#callback-oriented-data-movement):
    /// return between one byte and `input.len()` bytes for each nonempty request. A zero or
    /// oversized count terminates the operation with [`ErrorCode::SIZE`]. Empty messages do
    /// not cause input callbacks.
    fn read_input(
        &self,
        sink: &mut dyn FnMut(&ReadableProcessSlice) -> Result<usize, ErrorCode>,
    ) -> Result<(), ErrorCode>;

    /// Return the computed digest.
    ///
    /// The driver must call this exactly once, after it has taken all input and before signaling
    /// successful completion. The client calls `fill` exactly once with the storage that receives
    /// the digest of all input bytes. `fill` overwrites all of it.
    fn write_digest(&self, fill: &mut dyn FnMut(&mut A::Output)) -> Result<(), ErrorCode>;

    /// Signal completion of a digest operation.
    ///
    /// This callback occurs exactly once after a request accepted by [`Digest::digest`]. If a data
    /// callback returns an error, the driver must immediately abort the operation and report that
    /// error through `result`.
    fn digest_done(&self, result: Result<(), ErrorCode>);
}
