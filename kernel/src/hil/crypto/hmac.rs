// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interfaces for HMAC (Hash-based Message Authentication Code) algorithms.
//!
//! This module separates the HMAC operation from the hash algorithm it is built on, as specified
//! in RFC 2104 and FIPS 198-1. Algorithm marker types provide the MAC length, the block length of
//! the underlying hash, and the fixed-size output type, while [`Hmac`] and [`HmacClient`] define
//! the asynchronous data flow shared by HMAC implementations.
//!
//! # Operation flow
//!
//! An HMAC operation proceeds as follows:
//!
//! 1. [`Hmac::begin`] starts a new operation and records the key length. No key material is
//!    read at this point.
//! 2. [`Hmac::update`] absorbs message input. The first update of an operation first pulls the
//!    key through [`HmacClient::read_key`]; every update then pulls message input through
//!    [`HmacClient::read_input`]. Each accepted update completes with
//!    [`HmacClient::update_done`]. Updates may be repeated any number of times, including zero.
//! 3. [`Hmac::finalize`] completes the computation. The driver delivers the HMAC through [`HmacClient::write_output`] and
//!    then signals completion through [`HmacClient::authentication_done`].
//!
//! [`Hmac::cancel`] may be called at any point after [`Hmac::begin`] is accepted to abandon the
//! operation. At most one asynchronous request ([`Hmac::begin`], [`Hmac::update`], or
//! [`Hmac::finalize`]) may be outstanding at a time.
//!
//! # Key handling
//!
//! The key is supplied anew for every operation and is read exactly once
//! [`Hmac::update`] is called at the first time or in case of zero-sized messages
//! once [`Hmac::finalize`] is called. The client must therefore keep the
//! key available from [`Hmac::begin`] until that request has pulled it.

use crate::{ErrorCode, hil::crypto::digest::DigestAlgorithm};

/// Computes an HMAC using algorithm `A`.
pub trait Hmac<A: DigestAlgorithm> {
    /// Start a new HMAC operation with a key of `key_len` bytes.
    ///
    /// The driver obtains the key through one or more [`HmacClient::read_key`] callbacks
    /// totaling exactly `key_len` bytes, derives the inner and outer padded keys, and signals
    /// completion through [`HmacClient::begin_done`]. If `key_len` is zero, the driver issues no
    /// key callbacks and uses the empty key, as permitted by RFC 2104.
    ///
    /// Keys longer than [`HmacAlgorithm::BLOCK_LEN`] are not permitted to use,
    /// user has to hash the key using the underlying hash function `A`,
    /// as required by RFC 2104.
    ///
    /// # Errors
    ///
    /// If this call returns an error, no callback occurs and no operation is started.
    ///
    /// - [`ErrorCode::BUSY`]: an operation is already in progress.
    /// - [`ErrorCode::SIZE`]: `key_len` exceeds the maximum key length supported by the
    ///   algorithm.
    fn begin(&self, key_len: usize) -> Result<(), ErrorCode>;

    /// Absorb `len` bytes of message input.
    ///
    /// If this is the first update of the operation, the driver first obtains the key through
    /// one or more [`HmacClient::read_key`] callbacks totaling exactly the `key_len` passed to
    /// [`Self::begin`].
    ///
    /// The driver then obtains the input through one or more [`HmacClient::read_input`]
    /// callbacks totaling exactly `len` bytes, and signals completion through
    /// [`HmacClient::update_done`].
    ///
    /// Multiple updates are equivalent to a single update over the concatenated input.
    ///
    /// # Errors
    ///
    /// If this call returns an error, no callback occurs and the operation state is unchanged.
    /// In particular, a failed first update does not consume the key.
    ///
    /// - [`ErrorCode::INVAL`]: no operation is in progress, or the operation has already been
    ///   finalized.
    /// - [`ErrorCode::BUSY`]: a previous [`Self::update`] has not yet completed.
    /// - [`ErrorCode::SIZE`]: absorbing `len` more bytes would exceed the maximum message length
    ///   supported by the implementation.
    fn update(&self, len: usize) -> Result<(), ErrorCode>;

    /// Complete the HMAC computation.
    ///
    /// The driver delivers the HMAC over all absorbed input through
    /// [`HmacClient::write_output`] and then signals completion through
    /// [`HmacClient::hmac_done`]. Finalizing without any prior [`Self::update`] produces the tag
    /// of the empty message, and the driver first obtains the key through
    /// one or more [`HmacClient::read_key`] callbacks totaling exactly the `key_len`
    /// passed to [`Self::begin`].
    ///
    /// After [`HmacClient::hmac_done`], the operation is over regardless of the result, all key
    /// material has been cleared, and a new operation must be started with [`Self::begin`].
    ///
    /// # Errors
    ///
    /// If this call returns an error, no callback occurs and the operation state is unchanged.
    ///
    /// - [`ErrorCode::INVAL`]: no operation is in progress, or the operation has already been
    ///   finalized.
    /// - [`ErrorCode::BUSY`]: a previous [`Self::begin`] or [`Self::update`] has not yet
    ///   completed.
    fn finalize(&self) -> Result<(), ErrorCode>;

    /// Abandon the operation in progress.
    ///
    /// Discards all absorbed state and key material and returns the driver to idle, so a new
    /// operation can be started with [`Self::begin`].
    ///
    /// If a [`Self::begin`], [`Self::update`], or [`Self::finalize`] request is outstanding, the
    /// driver completes it with [`ErrorCode::CANCEL`] through callback [`HmacClient::authentication_done`].
    /// The driver issues no further data callbacks for the operation; in particular,
    /// [`HmacClient::read_key`] and [`HmacClient::write_output`] are not called.
    /// If no request is outstanding, no callback occurs.
    ///
    /// # Errors
    ///
    /// - [`ErrorCode::INVAL`]: no operation is in progress.
    fn cancel(&self) -> Result<(), ErrorCode>;

    /// Set the client that will receive callbacks for HMAC operations.
    fn set_client(&self, client: &'static dyn HmacClient<A>);
}

/// Client callbacks for [`Hmac`] operations.
pub trait HmacClient<A: DigestAlgorithm> {
    /// Retrieve the key.
    ///
    /// The client must write key bytes into `key` and return the number of bytes written. A
    /// driver may issue this callback multiple times, but each destination must fit within the
    /// remaining key. The total number of bytes consumed must equal the `key_len` passed to
    /// [`Hmac::begin`].
    /// Follow the [crypto data movement contract](super#callback-oriented-data-movement):
    /// return between one byte and `key.len()` bytes for each nonempty request.
    /// A zero or oversized count terminates the operation with [`ErrorCode::SIZE`].
    /// Empty keys do not cause key callbacks.
    fn read_key(&self, key: &mut [u8]) -> Result<usize, ErrorCode>;

    /// Retrieve input to hash.
    ///
    /// The client must write input into `input` and return the number of bytes written. A driver
    /// may issue this callback multiple times, but each destination must fit within the remaining
    /// input. The total number of bytes consumed must equal the `len` passed to [`Digest::digest`].
    /// Follow the [crypto data movement contract](super#callback-oriented-data-movement):
    /// return between one byte and `input.len()` bytes for each nonempty request. A zero or
    /// oversized count terminates the operation with [`ErrorCode::SIZE`]. Empty messages do
    /// not cause input callbacks.
    fn read_input(&self, input: &mut [u8]) -> Result<usize, ErrorCode>;

    /// Signal completion of a [`Hmac::update`] request.
    ///
    /// This callback occurs exactly once after a request accepted by [`Hmac::update`].
    ///
    /// On success, all `len` bytes have been absorbed (preceded by the key, for the first
    /// update) and the driver accepts another [`Hmac::update`] or [`Hmac::finalize`].
    ///
    /// On error, the operation has been aborted and its state and key material discarded; the
    /// client must start a new operation with [`Hmac::begin`]. If [`Self::read_key`] or
    /// [`Self::read_input`] returned an error, that error is reported here.
    /// [`ErrorCode::CANCEL`] indicates the request was abandoned by [`Hmac::cancel`].
    fn update_done(&self, result: Result<(), ErrorCode>);

    /// Return the computed HMAC.
    ///
    /// The driver must call this exactly once with the HMAC bytes over all input bytes before signaling
    /// successful completion. If this callback returns an error, the driver reports that error through
    /// [`Self::authentication_done`].
    fn write_output(&self, digest: &A::Output) -> Result<(), ErrorCode>;

    /// Signal completion of an HMAC operation.
    ///
    /// This callback occurs exactly once after a request accepted by [`Hmac::finalize`]. If a
    /// [`Self::write_output`] callback returns an error, the driver must immediately
    /// abort the operation and report that error through `result`. [`ErrorCode::CANCEL`] indicates
    /// the request was abandoned by [`Hmac::cancel`]. After this callback, all key material has been cleared,
    /// the driver is idle, and it accepts a new [`Hmac::begin`].
    fn authentication_done(&self, result: Result<(), ErrorCode>);
}
