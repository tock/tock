// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interfaces for message digest algorithms.
//!
//! This module separates the digest operation from the algorithm used to perform it. Algorithm
//! marker types provide the digest length and fixed-size output type, while [`Digest`] and
//! [`DigestClient`] define the asynchronous data flow shared by digest implementations.
//!
//! # Operation flow
//!
//! A digest operation proceeds as follows:
//!
//! 1. [`Digest::begin`] starts a new operation.
//! 2. [`Digest::update`] absorbs input, which the driver pulls through
//!    [`DigestClient::read_input`]. Each accepted update completes with
//!    [`DigestClient::update_done`]. Updates may be repeated any number of times, including zero.
//! 3. [`Digest::finalize`] completes the computation. The driver delivers the result through
//!    [`DigestClient::write_digest`] and then signals completion through
//!    [`DigestClient::digest_done`].
//!
//! [`Digest::cancel`] may be called at any point after [`Digest::begin`] to abandon the operation.
//! At most one asynchronous request ([`Digest::update`] or [`Digest::finalize`]) may be
//! outstanding at a time.

use crate::ErrorCode;

/// Properties of a message digest algorithm.
pub trait DigestAlgorithm {
    /// Digest length, in bytes.
    const DIGEST_LEN: usize;

    /// Block length of the hash function, in bytes.
    ///
    /// The number of input bytes processed by each invocation of the compression function. For
    /// SHA-3, this is the rate of the underlying Keccak sponge, as defined in FIPS 202.
    const BLOCK_LEN: usize;

    /// Fixed-size byte array containing a digest produced by this algorithm.
    ///
    /// The byte length of this type must equal [`Self::DIGEST_LEN`].
    type Output: AsRef<[u8]> + AsMut<[u8]>;
}

macro_rules! digest_algorithm {
    ($name:ident, $digest_len:expr, $block_len:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name;

        impl DigestAlgorithm for $name {
            const DIGEST_LEN: usize = $digest_len;
            const BLOCK_LEN: usize = $block_len;
            type Output = [u8; $digest_len];
        }
    };
}

digest_algorithm!(Md5, 16, 512 >> 3);
digest_algorithm!(Sha1, 20, 512 >> 3);
digest_algorithm!(Sha224, 28, 512 >> 3);
digest_algorithm!(Sha256, 32, 512 >> 3);
digest_algorithm!(Sha384, 48, 1024 >> 3);
digest_algorithm!(Sha512_224, 28, 1024 >> 3);
digest_algorithm!(Sha512_256, 32, 1024 >> 3);
digest_algorithm!(Sha512, 64, 1024 >> 3);
digest_algorithm!(Sha3_224, 28, 1152 >> 3);
digest_algorithm!(Sha3_256, 32, 1088 >> 3);
digest_algorithm!(Sha3_384, 48, 832 >> 3);
digest_algorithm!(Sha3_512, 64, 576 >> 3);
digest_algorithm!(Sm3, 32, 512 >> 3);

/// Computes a message digest using algorithm `A`.
pub trait Digest<A: DigestAlgorithm> {
    /// Start a new digest operation.
    ///
    /// Initializes the digest state for algorithm `A`. This call completes synchronously and
    /// does not cause a callback. On success, the driver accepts [`Self::update`],
    /// [`Self::finalize`], and [`Self::cancel`] for this operation.
    ///
    /// # Errors
    ///
    /// - [`ErrorCode::BUSY`]: an operation is already in progress.
    /// - [`ErrorCode::OFF`]: the underlying hardware is unavailable.
    ///
    fn begin(&self) -> Result<(), ErrorCode>;

    /// Absorb `len` bytes of input into the digest.
    ///
    /// The driver obtains the input through one or more [`DigestClient::read_input`] callbacks
    /// totaling exactly `len` bytes, and then signals completion through
    /// [`DigestClient::update_done`].
    ///
    /// Multiple updates are equivalent to a single update over the concatenated input.
    ///
    /// # Errors
    ///
    /// If this call returns an error, no callback occurs and the operation state is unchanged.
    /// - [`ErrorCode::INVAL`]: no operation is in progress, or the operation has already been
    ///   finalized, or `len` is zero and no absorption can happen.
    /// - [`ErrorCode::BUSY`]: a previous [`Self::update`] has not yet
    ///   completed.
    /// - [`ErrorCode::SIZE`]: absorbing `len` more bytes would exceed the maximum message length
    ///   supported by the implementation.
    fn update(&self, len: usize) -> Result<(), ErrorCode>;

    /// Complete the digest computation.
    ///
    /// The driver delivers the digest of all absorbed input through
    /// [`DigestClient::write_output`] and then signals completion through
    /// [`DigestClient::digest_done`]. Finalizing without any prior [`Self::update`] produces the
    /// digest of the empty message.
    ///
    /// After [`DigestClient::digest_done`], the operation is over regardless of the result, and a
    /// new operation must be started with [`Self::begin`].
    ///
    /// # Errors
    ///
    /// If this call returns an error, no callback occurs and the operation state is unchanged.
    ///
    /// - [`ErrorCode::INVAL`]: no operation is in progress, or the operation has already been
    ///   finalized.
    /// - [`ErrorCode::BUSY`]: a previous [`Self::update`] has not yet completed.
    /// - [`ErrorCode::NOSUPPORT`]: the implementation does not support zero-sized messages.
    fn finalize(&self) -> Result<(), ErrorCode>;

    /// Abandon the operation in progress.
    ///
    /// Discards all absorbed chunks of data and returns the driver to idle, so a new operation can be
    /// started with [`Self::begin`].
    ///
    /// If an [`Self::update`] or [`Self::finalize`] request is outstanding, the driver completes
    /// it with [`ErrorCode::CANCEL`] through callback [`DigestClient::digest_done`].
    /// The driver issues no further data callbacks for the operation;
    /// in particular, [`DigestClient::write_digest`] is not called.
    /// If no request is outstanding, no callback occurs.
    ///
    /// # Errors
    ///
    /// - [`ErrorCode::INVAL`]: no operation is in progress.
    fn cancel(&self) -> Result<(), ErrorCode>;

    /// Set the client that will receive callbacks for digest operations.
    fn set_client(&self, client: &'static dyn DigestClient<A>);
}

/// Client callbacks for [`Digest`] operations.
pub trait DigestClient<A: DigestAlgorithm> {
    /// Retrieve input to hash.
    ///
    /// The client must write input into `input` and return the number of bytes written. A driver
    /// may issue this callback multiple times, but must not request more than the `len` passed to
    /// [`Digest::digest`] in total.
    fn read_input(&self, input: &mut [u8]) -> Result<usize, ErrorCode>;

    /// Signal completion of a [`Digest::update`] request.
    ///
    /// This callback occurs exactly once after a request accepted by [`Digest::update`].
    ///
    /// On success, all `len` bytes have been absorbed and the driver accepts another
    /// [`Digest::update`] or [`Digest::finalize`].
    ///
    /// On error, the operation has been aborted and its state discarded; the client must start
    /// a new operation with [`Digest::begin`]. If [`Self::read_input`] returned an error, that
    /// error is reported here.
    fn update_done(&self, result: Result<(), ErrorCode>);

    /// Return the computed digest.
    ///
    /// The driver must call this exactly once with the digest of all input bytes before signaling
    /// successful completion. If this callback returns an
    /// error, the driver reports that error through [`Self::digest_done`].
    fn write_digest(&self, digest: &A::Output) -> Result<(), ErrorCode>;

    /// Signal completion of a digest operation.
    ///
    /// This callback occurs exactly once after a request accepted by [`Digest::finalize`]. If a
    /// data callback returns an error, the driver must immediately abort the operation and report
    /// that error through `result`. [`ErrorCode::CANCEL`] indicates the request was abandoned by
    /// [`Digest::cancel`]. After this callback, the driver is idle and accepts a new
    /// [`Digest::begin`].
    fn digest_done(&self, result: Result<(), ErrorCode>);
}
