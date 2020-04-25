//! In kernel abstraction over hash functions
//!
//! This is an abstraction over hash functions and implementations of
//! these hash functions. It supports exotic hash functions and
//! checksums with non-byte outputs and inputs.
//!
//! The constraints over the Hasher are enforced by the HashType type
//! parameter.
//!
//! Authors
//! -------------------
//! * Leon Schuermann <leon.git@is.currently.online>
//! * Daniel Rutz <info@danielrutz.com>
//! * April 01, 2020

use kernel::ReturnCode;

/// A trait to be implemented on zero-sized types indicating
/// properties and types of hash functions
///
/// Implementors should be zero sized
pub trait HashType {
    /// The input type of the hash function
    ///
    /// Typically a byte
    type Input = u8;

    /// The output type of a hash function
    ///
    /// Typically a byte array with a fixed length
    type Output;

    /// Gives a unique identifier of this hash function, like
    /// `sha2_384` or `md5`
    fn identifier() -> &'static str;

    /// Returns the hash output length in bits
    fn output_bits() -> usize;
}

/// A hasher instance, taking an iterator over bytes and returning a type
pub trait Hasher<'a, T: HashType> {
    /// Set the client to receive callbacks
    ///
    /// Must be done prior to any operation
    fn set_client(&'a self, client: &'a dyn HasherClient<T>);

    /// Reset the state of the hasher. The next call to `input_data`
    /// will start a new hash.
    ///
    /// A client must wait for the `reset_done` callback prior to
    /// calling any other method.
    fn reset(&self);

    /// Input data into the hash function
    ///
    /// The function will return how many items have been consumed
    /// from the Iterator. In addition to that, a boolean flag is
    /// returned indicating whether the caller has to wait for a
    /// `data_processed` callback, whereby `true` means a callback is
    /// required.
    ///
    /// For synchronous or software implementations, this reduces
    /// scheduler overhead and may improve throughput.
    fn input_data(
        &self,
        iter: &mut dyn Iterator<Item = T::Input>,
    ) -> Result<(usize, bool), ReturnCode>;

    /// Request hash calculation
    ///
    /// The hash will be returned in the `hash_ready` callback on the
    /// registered client
    fn get_hash(&self) -> Result<(), ReturnCode>;
}

pub trait HasherClient<T: HashType> {
    /// The Hasher state has been reset
    ///
    /// It is now safe to call `input_data` again to start a new hash.
    fn reset_done(&self);

    /// The data input using `input_data` was processed
    ///
    /// It is now safe to call `input_data` again or request the hash
    /// using `hash_ready`
    fn data_processed(&self, err: Option<ReturnCode>);

    /// The requested hash has been calculated
    fn hash_ready(&self, hash: Result<&T::Output, ReturnCode>);
}

// ----- HASH FUNCTION DEFINITIONS -----
pub mod hash_functions {
    //! Definitions of common hash functions as HashTypes
    use crate::hash::HashType;

    pub enum MD5 {}
    impl HashType for MD5 {
        type Output = [u8; 16];

        fn identifier() -> &'static str {
            "md5"
        }

        fn output_bits() -> usize {
            128
        }
    }

    pub enum SHA1 {}
    impl HashType for SHA1 {
        type Output = [u8; 20];

        fn identifier() -> &'static str {
            "sha1"
        }

        fn output_bits() -> usize {
            160
        }
    }

    pub enum SHA2_224 {}
    impl HashType for SHA2_224 {
        type Output = [u8; 28];

        fn identifier() -> &'static str {
            "sha2_224"
        }

        fn output_bits() -> usize {
            224
        }
    }

    pub enum SHA2_256 {}
    impl HashType for SHA2_256 {
        type Output = [u8; 32];

        fn identifier() -> &'static str {
            "sha2_256"
        }

        fn output_bits() -> usize {
            256
        }
    }

    pub enum SHA2_384 {}
    impl HashType for SHA2_384 {
        type Output = [u8; 48];

        fn identifier() -> &'static str {
            "sha2_384"
        }

        fn output_bits() -> usize {
            384
        }
    }

    pub enum SHA2_512 {}
    impl HashType for SHA2_512 {
        type Output = [u8; 64];

        fn identifier() -> &'static str {
            "sha2_512"
        }

        fn output_bits() -> usize {
            512
        }
    }

    pub enum SHA3_224 {}
    impl HashType for SHA3_224 {
        type Output = [u8; 28];

        fn identifier() -> &'static str {
            "sha3_224"
        }

        fn output_bits() -> usize {
            224
        }
    }

    pub enum SHA3_256 {}
    impl HashType for SHA3_256 {
        type Output = [u8; 32];

        fn identifier() -> &'static str {
            "sha3_256"
        }

        fn output_bits() -> usize {
            256
        }
    }

    pub enum SHA3_384 {}
    impl HashType for SHA3_384 {
        type Output = [u8; 48];

        fn identifier() -> &'static str {
            "sha3_384"
        }

        fn output_bits() -> usize {
            384
        }
    }

    pub enum SHA3_512 {}
    impl HashType for SHA3_512 {
        type Output = [u8; 64];

        fn identifier() -> &'static str {
            "sha3_512"
        }

        fn output_bits() -> usize {
            512
        }
    }
}
