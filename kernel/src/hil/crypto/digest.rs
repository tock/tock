// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interfaces for message digest algorithms.
//!
//! This module separates the digest operation from the algorithm used to perform it. Algorithm
//! marker types provide the digest length and fixed-size output type, while [`Digest`] and
//! [`DigestClient`] define the asynchronous data flow shared by digest implementations.

use crate::ErrorCode;

/// Properties of a message digest algorithm.
pub trait DigestAlgorithm {
    /// Digest length, in bytes.
    const DIGEST_LEN: usize;

    /// Fixed-size byte array containing a digest produced by this algorithm.
    ///
    /// The byte length of this type must equal [`Self::DIGEST_LEN`].
    type Output: AsRef<[u8]> + AsMut<[u8]>;
}

macro_rules! digest_algorithm {
    ($name:ident, $digest_len:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name;

        impl DigestAlgorithm for $name {
            const DIGEST_LEN: usize = $digest_len;
            type Output = [u8; $digest_len];
        }
    };
}

digest_algorithm!(Md5, 16);
digest_algorithm!(Sha1, 20);
digest_algorithm!(Sha224, 28);
digest_algorithm!(Sha256, 32);
digest_algorithm!(Sha384, 48);
digest_algorithm!(Sha512, 64);

/// Computes a message digest using algorithm `A`.
pub trait Digest<A: DigestAlgorithm> {
    /// Initiate a digest operation over `len` bytes of input.
    ///
    /// Input is retrieved through [`DigestClient::read_input`]. The computed digest is returned
    /// through [`DigestClient::write_digest`], followed by exactly one
    /// [`DigestClient::digest_done`] callback. Callbacks must not occur before this method returns.
    ///
    /// Returns [`ErrorCode::BUSY`] if an operation is already in progress or [`ErrorCode::SIZE`]
    /// if `len` exceeds an implementation limit known when this method is called. On `Ok(())`, a
    /// completion callback will occur. On `Err`, no callbacks will occur for this request.
    fn digest(&self, len: usize) -> Result<(), ErrorCode>;

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

    /// Return the computed digest.
    ///
    /// The driver must call this exactly once with the digest of all input bytes before signaling
    /// successful completion.
    fn write_digest(&self, digest: &A::Output) -> Result<(), ErrorCode>;

    /// Signal completion of a digest operation.
    ///
    /// This callback occurs exactly once after a request accepted by [`Digest::digest`]. If a data
    /// callback returns an error, the driver must immediately abort the operation and report that
    /// error through `result`.
    fn digest_done(&self, result: Result<(), ErrorCode>);
}
