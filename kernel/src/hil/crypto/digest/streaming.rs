// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interfaces for message digests computed incrementally, with
//! the state of computation saved to and restored from a client-held context
//! between requests.

use crate::{
    ErrorCode,
    hil::crypto::digest::{
        DigestAlgorithm,
        one_shot::{Digest, DigestClient},
    },
    processbuffer::ReadableProcessSlice,
};

/// Computes a message digest using algorithm `A` over input supplied in pieces.
pub trait StreamingDigest<A: DigestAlgorithm>: Digest<A> {
    /// The resumable state of a digest computation.
    ///
    /// The default state is the state with no input absorbed.
    type DigestContext: Context;

    /// Start a new digest operation.
    ///
    /// Initializes the digest state for algorithm `A`. To hash a new message,
    /// the client supplies `DigestContext::default()`, which is equal to the initial state,
    /// and the driver saves all the necessary information for the upcoming message computation
    /// calling [`StreamingDigestClient::save_context`].
    /// This call completes synchronously and does not cause a callback. On success, the driver
    /// accepts [`Self::update`], [`Self::finalize`], and [`Self::cancel`] for this operation.
    ///
    /// The driver does not keep its state across requests.
    ///
    /// Returns [`ErrorCode::BUSY`] if an operation is already in progress.
    fn begin(&self) -> Result<(), ErrorCode>;

    /// Absorb `len` bytes of input into the digest.
    ///
    /// The driver resumes from the state supplied through [`StreamingDigestClient::restore_context`],
    /// obtains the input through one or more [`StreamingDigestClient::update_input`] callbacks
    /// totaling exactly `len` bytes, and then signals completion through
    /// [`StreamingDigestClient::update_done`].
    ///
    /// Multiple updates are equivalent to a single update over the concatenated input.
    ///
    /// Returns an error, no callback occurs and the operation state is unchanged.
    /// - [`ErrorCode::INVAL`]: no operation is in progress, or the operation has already been
    ///   finalized, or `len` is zero and no absorption can happen.
    /// - [`ErrorCode::BUSY`]: a previous [`Self::update`] has not yet
    ///   completed.
    /// - [`ErrorCode::SIZE`]: absorbing `len` more bytes would exceed the maximum message length
    ///   supported by the implementation.
    fn update(&self, len: usize) -> Result<(), ErrorCode>;

    /// Complete the digest computation.
    ///
    /// The driver resumes from the state supplied through [`StreamingDigestClient::restore_context`],
    /// delivers the digest of all absorbed input through [`DigestClient::write_output`] and
    /// then signals completion through [`DigestClient::digest_done`].
    ///
    /// Finalizing without any prior [`Self::update`] produces the
    /// digest of the empty message.
    ///
    /// After [`DigestClient::digest_done`], the operation is over regardless of the result, and a
    /// new operation must be started with [`Self::begin`].
    ///
    /// Returns an error, no callback occurs and the operation state is unchanged.
    /// - [`ErrorCode::INVAL`]: no operation is in progress, or the operation has already been
    ///   finalized.
    /// - [`ErrorCode::BUSY`]: a previous [`Self::update`] has not yet completed.
    /// - [`ErrorCode::NOSUPPORT`]: the implementation does not support zero-sized messages.
    fn finalize(&self) -> Result<(), ErrorCode>;

    /// Set the client that will receive callbacks for streaming digest operations.
    fn set_streaming_client(
        &self,
        client: &'static dyn StreamingDigestClient<A, Self::DigestContext>,
    );
}

/// Client callbacks for [`StreamingDigest`] operations.
///
/// The data callbacks of [`DigestClient`] are used for streaming digest operations.
pub trait StreamingDigestClient<A: DigestAlgorithm, C: Context>: DigestClient<A> {
    /// Supply input to update the state, together with the context of the message.
    ///
    /// The driver issues this callback one or more times during a
    /// [`StreamingDigest::update`] request. The client passes `advance` the next unconsumed
    /// bytes of the input and the storage holding the context of the message. `advance`
    /// takes a portion of the bytes, advances the context to include them, and returns the
    /// length of that portion.
    ///
    /// `advance` may take fewer bytes than offered; the client should then offer the rest again
    /// in a later callback. `advance` never takes more than the input still owed for the request.
    ///
    /// Returns [`ErrorCode::NOMEM`] if the client cannot access the input
    /// or the context, in which case it must not call `advance`. The driver ends the request
    /// with that error.
    fn update_input(
        &self,
        advance: &dyn Fn(&ReadableProcessSlice, &mut C) -> Result<usize, ErrorCode>,
    ) -> Result<(), ErrorCode>;

    /// Signal completion of a [`StreamingDigest::update`] request.
    ///
    /// This callback occurs exactly once after a request accepted by [`StreamingDigest::update`],
    ///
    /// On success, all `len` bytes have been absorbed, the new state has been saved
    /// through [`Self::save_context`], and the driver accepts another
    /// [`StreamingDigest::update`] or [`StreamingDigest::finalize`].
    ///
    /// On error, the operation has been aborted and nothing was saved; the client must restart
    /// the operation. If a data or context callback failed, that
    /// error is reported here.
    fn update_done(&self, result: Result<(), ErrorCode>);

    /// Supply the state to resume from.
    ///
    /// The driver issues this callback at the start of every [`StreamingDigest::update`] and
    /// [`StreamingDigest::finalize`] request, before any data callback. The client calls
    /// `restore` exactly once with the context of the message the request belongs to:
    /// `C::default()` if nothing has been absorbed yet, otherwise the context most recently
    /// written by [`Self::save_context`]. `restore` reads the context and does not keep a
    /// reference to it.
    ///
    /// Returns [`ErrorCode::NOMEM`] if the client has no context to supply or it failed to access it.
    fn restore_context(&self, restore: &dyn Fn(&C)) -> Result<(), ErrorCode>;

    /// Store the state to resume from later.
    ///
    /// The driver issues this callback after it has absorbed all input of a
    /// [`StreamingDigest::update`] request and if there is context left to be saved,
    /// before a successful [`Self::update_done`].
    ///
    /// The client calls `save` exactly once with the storage for the context of the message;
    /// `save` overwrites all of it. This callback does occur for requests that fail only in case
    /// the input bytes are used as a context and the rollback has to be performed,
    /// so the previously saved context stays intact.
    ///
    /// Returns [`ErrorCode::NOMEM`] if the client has no context to supply or it failed to access it.
    fn save_context(&self, save: &mut dyn FnMut(&mut C)) -> Result<(), ErrorCode>;
}

/// Marker for types that hold the resumable state of digest computation.
pub trait Context: Default {}
