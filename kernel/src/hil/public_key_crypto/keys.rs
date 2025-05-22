// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Key interface for Public/Private key encryption

use crate::hil::entropy;
use crate::ErrorCode;

/// Upcall from the `PubPrivKeyGenerate` trait.
pub trait PubPrivKeyGenerateClient<'a> {
    /// The `generate()` command has been completed.
    fn generation_complete(
        &'a self,
        result: Result<(), (ErrorCode, &'static mut [u8], &'static mut [u8])>,
    );
}

/// An internal representation of an asymmetric Public key.
///
/// This trait is useful for managing keys internally in Tock.
///
/// PubKey is designed for fixed length keys. That is an implementation should
/// support only a single key length, for example RSA 2048.
/// Note that we don't use const generics here though. That is because even
/// within a single key length implementation, there can be different length
/// inputs, for examples compressed or uncompressed keys.
pub trait PubKey {
    /// Import an existing public key.
    ///
    /// The reference to the `public_key` is stored internally and can be
    /// retrieved with the `pub_key()` function.
    /// The `public_key` can be either a mutable static or an immutable static,
    /// depending on where the key is stored (flash or memory).
    ///
    /// The possible ErrorCodes are:
    ///     - `BUSY`: A key is already imported or in the process of being
    ///               generated.
    ///     - `INVAL`: An invalid key was supplied.
    ///     - `SIZE`: An invalid key size was supplied.
    fn import_public_key(
        &self,
        public_key: &'static [u8],
    ) -> Result<(), (ErrorCode, &'static [u8])>;

    /// Return the public key supplied by `import_public_key()` or
    /// `generate()`.
    ///
    /// On success the return value is `Ok(())` with the buffer that was
    /// originally passed in to hold the key.
    ///
    /// On failure the possible ErrorCodes are:
    ///     - `NODEVICE`: The key does not exist
    fn pub_key(&self) -> Result<&'static [u8], ErrorCode>;

    /// Report the length of the public key in bytes, as returned from `pub_key()`.
    /// A value of 0 indicates that the key does not exist.
    fn len(&self) -> usize;
}

/// An internal representation of an asymmetric Public key.
///
/// This trait is useful for managing keys internally in Tock.
///
/// PubKey is designed for fixed length keys. That is an implementation should
/// support only a single key length, for example RSA 2048.
/// Note that we don't use const generics here though. That is because even
/// within a single key length implementation, there can be different length
/// inputs, for examples compressed or uncompressed keys.
pub trait PubKeyMut {
    /// Import an existing public key.
    ///
    /// The reference to the `public_key` is stored internally and can be
    /// retrieved with the `pub_key()` function.
    /// The `public_key` can be either a mutable static or an immutable static,
    /// depending on where the key is stored (flash or memory).
    ///
    /// The possible ErrorCodes are:
    ///     - `BUSY`: A key is already imported or in the process of being
    ///                  generated.
    ///     - `INVAL`: An invalid key was supplied.
    ///     - `SIZE`: An invalid key size was supplied.
    fn import_public_key(
        &self,
        public_key: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Return the public key supplied by `import_public_key()` or
    /// `generate()`.
    ///
    /// On success the return value is `Ok(())` with the buffer that was
    /// originally passed in to hold the key.
    ///
    /// On failure the possible ErrorCodes are:
    ///     - `NODEVICE`: The key does not exist
    fn pub_key(&self) -> Result<&'static mut [u8], ErrorCode>;

    /// Report the length of the public key in bytes, as returned from `pub_key()`.
    /// A value of 0 indicates that the key does not exist.
    fn len(&self) -> usize;
}

/// An internal representation of an asymmetric Public and Private key.
///
/// This trait is useful for managing keys internally in Tock.
///
/// PubPrivKey is designed for fixed length keys. That is an implementation
/// should support only a single key length, for example RSA 2048.
/// Note that we don't use const generics here though. That is because even
/// within a single key length implementation, there can be different length
/// inputs, for examples compressed or uncompressed keys.
pub trait PubPrivKey: PubKey {
    /// Import an existing private key.
    ///
    /// The reference to the `private_key` is stored internally and can be
    /// retrieved with the `priv_key()` function.
    /// The `private_key` can be either a mutable static or an immutable static,
    /// depending on where the key is stored (flash or memory).
    ///
    /// The possible ErrorCodes are:
    ///     - `BUSY`: A key is already imported or in the process of being
    ///               generated.
    ///     - `INVAL`: An invalid key was supplied.
    ///     - `SIZE`: An invalid key size was supplied.
    fn import_private_key(
        &self,
        private_key: &'static [u8],
    ) -> Result<(), (ErrorCode, &'static [u8])>;

    /// Return the private key supplied by `import_private_key()` or
    /// `generate()`.
    ///
    /// On success the return value is `Ok(())` with the buffer that was
    /// originally passed in to hold the key.
    ///
    /// On failure the possible ErrorCodes are:
    ///     - `NODEVICE`: The key does not exist
    fn priv_key(&self) -> Result<&'static [u8], ErrorCode>;

    /// Report the length of the private key in bytes, as returned from `priv_key()`.
    /// A value of 0 indicates that the key does not exist.
    fn len(&self) -> usize;
}

/// An internal representation of an asymmetric Public and Private key.
///
/// This trait is useful for managing keys internally in Tock.
///
/// PubPrivKey is designed for fixed length keys. That is an implementation
/// should support only a single key length, for example RSA 2048.
/// Note that we don't use const generics here though. That is because even
/// within a single key length implementation, there can be different length
/// inputs, for examples compressed or uncompressed keys.
pub trait PubPrivKeyMut: PubKeyMut {
    /// Import an existing private key.
    ///
    /// The reference to the `private_key` is stored internally and can be
    /// retrieved with the `priv_key()` function.
    /// The `private_key` can be either a mutable static or an immutable static,
    /// depending on where the key is stored (flash or memory).
    ///
    /// The possible ErrorCodes are:
    ///     - `BUSY`: A key is already imported or in the process of being
    ///                  generated.
    ///     - `INVAL`: An invalid key was supplied.
    ///     - `SIZE`: An invalid key size was supplied.
    fn import_private_key(
        &self,
        private_key: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Return the private key supplied by `import_private_key()` or
    /// `generate()`.
    ///
    /// On success the return value is `Ok(())` with the buffer that was
    /// originally passed in to hold the key.
    ///
    /// On failure the possible ErrorCodes are:
    ///     - `NODEVICE`: The key does not exist
    fn priv_key(&self) -> Result<&'static mut [u8], ErrorCode>;

    /// Report the length of the private key in bytes, as returned from `priv_key()`.
    /// A value of 0 indicates that the key does not exist.
    fn len(&self) -> usize;
}

/// An internal representation of generating asymmetric Public/Private key
/// pairs.
///
/// This trait is useful for managing keys internally in Tock.
pub trait PubPrivKeyGenerate<'a>: PubPrivKey {
    /// Set the client. This client will be called when the `generate()`
    /// function is complete. If using an existing key this doesn't need to be
    /// used.
    fn set_client(&'a self, client: &'a dyn PubPrivKeyGenerateClient<'a>);

    /// This generates a new private/public key pair. The length will be
    /// hard coded by the implementation, for example RSA 2048 will create a
    /// 2048 bit key.
    /// This will call the `generation_complete()` on completion. They keys
    /// cannot be used and will return `None` until the upcall has been called.
    ///
    /// The keys generated by `generate()` will depend on the implementation.
    ///
    /// The original key buffers can be retrieve usind the `pub_key()` and
    /// `priv_key()` functions.
    ///
    /// The possible ErrorCodes are:
    ///     - `BUSY`: A key is already imported or in the process of being
    ///               generated.
    ///     - `OFF`: The underlying `trng` is powered down.
    ///     - `SIZE`: An invalid buffer size was supplied.
    fn generate(
        &'a self,
        trng: &'a dyn entropy::Entropy32,
        public_key_buffer: &'static mut [u8],
        private_key_buffer: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8], &'static mut [u8])>;
}

pub trait RsaKey: PubKey {
    /// Run the specified closure over the modulus, if it exists
    /// The modulus is returned MSB (big endian)
    /// Returns `Some()` if the key exists and the closure was called,
    /// otherwise returns `None`.
    fn map_modulus(&self, closure: &dyn Fn(&[u8])) -> Option<()>;

    /// The the modulus if it exists.
    /// The modulus is returned MSB (big endian)
    /// Returns `Some()` if the key exists otherwise returns `None`.
    /// The modulus can be returned by calling `import_public_key()` with
    /// the output of this function.
    fn take_modulus(&self) -> Option<&'static [u8]>;

    /// Returns the public exponent of the key pair if it exists
    fn public_exponent(&self) -> Option<u32>;
}

pub trait RsaPrivKey: PubPrivKey + RsaKey {
    /// Returns the specified closure over the private exponent, if it exists
    /// The exponent is returned MSB (big endian)
    /// Returns `Some()` if the key exists and the closure was called,
    /// otherwise returns `None`.
    fn map_exponent(&self, closure: &dyn Fn(&[u8])) -> Option<()>;

    /// The the private exponent if it exists.
    /// The exponent is returned MSB (big endian)
    /// Returns `Some()` if the key exists otherwise returns `None`.
    /// The exponent can be returned by calling `import_private_key()` with
    /// the output of this function.
    fn take_exponent(&self) -> Option<&'static [u8]>;
}

pub trait RsaKeyMut: PubKeyMut {
    /// Run the specified closure over the modulus, if it exists
    /// The modulus is returned MSB (big endian)
    /// Returns `Some()` if the key exists and the closure was called,
    /// otherwise returns `None`.
    fn map_modulus(&self, closure: &dyn Fn(&mut [u8])) -> Option<()>;

    /// The the modulus if it exists.
    /// The modulus is returned MSB (big endian)
    /// Returns `Some()` if the key exists otherwise returns `None`.
    /// The modulus can be returned by calling `import_public_key()` with
    /// the output of this function.
    fn take_modulus(&self) -> Option<&'static mut [u8]>;

    /// Returns the public exponent of the key pair if it exists
    fn public_exponent(&self) -> Option<u32>;
}

pub trait RsaPrivKeyMut: PubPrivKeyMut + RsaKeyMut {
    /// Returns the specified closure over the private exponent, if it exists
    /// The exponent is returned MSB (big endian)
    /// Returns `Some()` if the key exists and the closure was called,
    /// otherwise returns `None`.
    fn map_exponent(&self, closure: &dyn Fn(&mut [u8])) -> Option<()>;

    /// The the private exponent if it exists.
    /// The exponent is returned MSB (big endian)
    /// Returns `Some()` if the key exists otherwise returns `None`.
    /// The exponent can be returned by calling `import_private_key()` with
    /// the output of this function.
    fn take_exponent(&self) -> Option<&'static mut [u8]>;
}

/// Client for selecting keys.
pub trait SelectKeyClient {
    /// Called when the number of keys available is known.
    fn get_key_count_done(&self, count: usize);

    /// Called when the specified key is active and ready to use for the next
    /// cryptographic operation.
    ///
    /// ### `error`:
    ///
    /// - `Ok(())`: The key was selected successfully.
    /// - `Err(())`: The key was selected set successfully.
    ///   - `ErrorCode::INVAL`: The index was not valid.
    ///   - `ErrorCode::FAIL`: The key could not be set.
    fn select_key_done(&self, index: usize, error: Result<(), ErrorCode>);
}

/// Interface for selecting an active key among the number of available keys.
///
/// This interface allows for the implementer to maintain an opaque internal
/// representation of keys. They may be stored in memory, flash, or in secure
/// element (where the actual key may not be accessible). Users of this
/// interface can select which key is active for cryptographic operations. There
/// is no assumption for implementers of this interface that keys can be added
/// or changed or that keys can be specified by their actual bytes in a slice.
///
/// Selecting a key is asynchronous as it may require communication over a bus
/// or waiting for an interrupt.
///
/// Keys are specified by an index starting at zero and going to
/// `get_key_count()-1`, without gaps. Selecting a key between `0` and
/// `get_key_count()-1` must not fail with `ErrorCode::INVAL`.
///
/// The stored keys can be public or private keys.
pub trait SelectKey<'a> {
    /// Return the number of keys that the device can switch among.
    ///
    /// Each key must be identifiable by a consistent index.
    ///
    /// This operation is asynchronous and its completion is signaled by
    /// `get_key_count_done()`.
    ///
    /// ## Return
    ///
    /// `Ok()` if getting the count has started. Otherwise:
    /// - `Err(ErrorCode::FAIL)` if the key count could not be started and there
    ///   will be no callback.
    fn get_key_count(&self) -> Result<(), ErrorCode>;

    /// Set the key identified by its index as the active key.
    ///
    /// Indices start at 0 and go to `get_key_count() - 1`.
    ///
    /// This operation is asynchronous and its completion is signaled by
    /// `select_key_done()`.
    ///
    /// ## Return
    ///
    /// `Ok()` if the key select operation was accepted. Otherwise:
    /// - `Err(ErrorCode::INVAL)` if the index is not valid.
    fn select_key(&self, index: usize) -> Result<(), ErrorCode>;

    fn set_client(&self, client: &'a dyn SelectKeyClient);
}

/// Client for setting keys.
pub trait SetKeyBySliceClient<const KL: usize> {
    /// Called when the key has been set.
    ///
    /// Returns the key that was set.
    ///
    /// ### `error`:
    ///
    /// - `Ok(())`: The key was set successfully.
    /// - `Err(())`: The key was not set successfully.
    ///   - `ErrorCode::FAIL`: The key could not be set.
    fn set_key_done(&self, previous_key: &'static mut [u8; KL], error: Result<(), ErrorCode>);
}

/// Interface for setting keys by a slice.
///
/// `KL` is the length of the keys.
///
/// Implementers must be able to store keys from a slice. This is most commonly
/// used for implementations that hold keys in memory. However, this interface
/// is asynchronous as keys may be stored in external storage or an external
/// chip and require an asynchronous operations.
///
/// Implementors cannot hold the slice of the key being set. Instead, they must
/// make an internal copy of the key and return the slice in
/// [`SetKeyBySliceClient::set_key_done()`].
pub trait SetKeyBySlice<'a, const KL: usize> {
    /// Set the current key.
    ///
    /// This is asynchronous. The key slice will be returned in
    /// [`SetKeyBySliceClient::set_key_done()`], or immediately if there is an
    /// error.
    ///
    /// ### Return
    ///
    /// `Ok()` if the key setting operation was accepted. Otherwise:
    /// - `Err(ErrorCode::FAIL)` if the key cannot be set.
    fn set_key(&self, key: &'static mut [u8; KL])
        -> Result<(), (ErrorCode, &'static mut [u8; KL])>;

    fn set_client(&self, client: &'a dyn SetKeyBySliceClient<KL>);
}
