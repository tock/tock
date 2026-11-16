// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interfaces for message digest algorithms.
//!
//! This module separates the digest operation from the algorithm used to perform it. Algorithm
//! marker types provide the digest length and fixed-size output type, while [`Digest`] / [StreamingDigest] and
//! [`DigestClient`] / [`StreamingDigestClient`] define the asynchronous data flow shared by digest implementations.

/// Properties of a message digest algorithm.
pub trait DigestAlgorithm {
    /// Digest length, in bytes.
    const DIGEST_LEN: usize;

    /// Block length of the hash function, in bytes.
    ///
    /// The number of input bytes processed by each invocation of the hash function. For
    /// SHA-3, this is the rate of the underlying Keccak sponge, as defined in FIPS 202.
    const BLOCK_LEN: usize;

    /// Zeroed fixed-size byte array for storing a digest produced by this algorithm.
    const ZEROED: Self::Output;

    /// Fixed-size byte array containing a digest produced by this algorithm.
    ///
    /// The byte length of this type must equal [`Self::DIGEST_LEN`].
    type Output: AsRef<[u8]> + AsMut<[u8]> + Sized;
}

macro_rules! digest_algorithm {
    ($name:ident, $digest_len:expr, $block_len:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name;

        impl DigestAlgorithm for $name {
            const DIGEST_LEN: usize = $digest_len;
            const BLOCK_LEN: usize = $block_len;
            const ZEROED: Self::Output = [0u8; $digest_len];
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

mod one_shot;
mod streaming;

pub use one_shot::{Digest, DigestClient};
pub use streaming::{Context, StreamingDigest, StreamingDigestClient};
