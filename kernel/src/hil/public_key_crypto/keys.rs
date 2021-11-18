//! Key interface for Public/Private key encryption

use crate::hil::entropy;
use crate::ErrorCode;

/// Upcall from the `KeyGenerate` trait.
pub trait KeyGenerateClient<'a> {
    /// The `generate()` command has been completed.
    fn generation_complete(
        &'a self,
        result: Result<(), (ErrorCode, &'static mut [u8], &'static mut [u8])>,
    );
}

/// An internal representation of a asymetric Public key.
///
/// This trait is useful for managing keys internally in Tock.
///
/// PubKey is designed for fixed length keys. That is an implementation should
/// support only a single key length, for example RSA 2048.
/// Note that we don't use const generics here though. That is because even
/// within a single key length implementation, there can be different length
/// inputs, for examples compressed or uncompressed keys.
pub trait PublicKey {
    /// Import an existing public key.
    ///
    /// The reference to the `public_key` is stored internally and can be
    /// retrieved with the `pub_key()` function.
    /// The `public_key` can be either a mutable static or an immutable static,
    /// depending on where the key is stored (flash or memory).
    ///
    /// The possible ErrorCodes are:
    ///     - `ALREADY`: A key is already imported or in the process of being
    ///                  generated.
    ///     - `INVAL`: An invalid key was supplied.
    ///     - `SIZE`: An invalid key size was supplied.
    fn import_public_key(
        &self,
        public_key: &'static [u8]
    ) -> Result<(), (ErrorCode, &'static [u8])>;

    /// Return the public key supplied by `import_public_key()` or
    /// `generate()`.
    ///
    /// On success the return value is `Ok(())` with the buffer that was
    /// originally passed in to hold the key.
    ///
    /// On failure the possible ErrorCodes are:
    ///     - `NODEVICE`: The key does not exist
    fn public_key(&self) -> Result<&'static [u8], ErrorCode>;

    /// Report the length of the public key in bytes, as returned from `pub_key()`.
    /// A value of 0 indicates that the key does not exist.
    fn len(&self) -> usize;
}

pub trait MutablePublicKey {
    /// Import an existing public key.
    ///
    /// The reference to the `public_key` is stored internally and can be
    /// retrieved with the `pub_key()` function.
    /// The `public_key` can be either a mutable static or an immutable static,
    /// depending on where the key is stored (flash or memory).
    ///
    /// The possible ErrorCodes are:
    ///     - `ALREADY`: A key is already imported or in the process of being
    ///                  generated.
    ///     - `INVAL`: An invalid key was supplied.
    ///     - `SIZE`: An invalid key size was supplied.
    fn import_public_key(
        &self,
        public_key: &'static mut [u8]
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Return the public key supplied by `import_public_key()` or
    /// `generate()`.
    ///
    /// On success the return value is `Ok(())` with the buffer that was
    /// originally passed in to hold the key.
    ///
    /// On failure the possible ErrorCodes are:
    ///     - `NODEVICE`: The key does not exist
    fn public_key(&self) -> Result<&'static mut [u8], ErrorCode>;

    /// Report the length of the public key in bytes, as returned from `pub_key()`.
    /// A value of 0 indicates that the key does not exist.
    fn len(&self) -> usize;
}


/// An internal representation of a asymetric Public and Private key.
///
/// This trait is useful for managing keys internally in Tock.
///
/// PubPrivKey is designed for fixed length keys. That is an implementation
/// should support only a single key length, for example RSA 2048.
/// Note that we don't use const generics here though. That is because even
/// within a single key length implementation, there can be different length
/// inputs, for examples compressed or uncompressed keys.
pub trait PublicPrivateKey: PublicKey {
    /// Import an existing private key.
    ///
    /// The reference to the `private_key` is stored internally and can be
    /// retrieved with the `priv_key()` function.
    /// The `private_key` can be either a mutable static or an immutable static,
    /// depending on where the key is stored (flash or memory).
    ///
    /// The possible ErrorCodes are:
    ///     - `ALREADY`: A key is already imported or in the process of being
    ///                  generated.
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
    fn private_key(&self) -> Result<&'static [u8], ErrorCode>;

    /// Report the length of the private key in bytes, as returned from `priv_key()`.
    /// A value of 0 indicates that the key does not exist.
    fn len(&self) -> usize;
}

pub trait MutablePublicPrivateKey: MutablePublicKey {
    /// Import an existing private key.
    ///
    /// The reference to the `private_key` is stored internally and can be
    /// retrieved with the `priv_key()` function.
    /// The `private_key` can be either a mutable static or an immutable static,
    /// depending on where the key is stored (flash or memory).
    ///
    /// The possible ErrorCodes are:
    ///     - `ALREADY`: A key is already imported or in the process of being
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
    fn private_key(&self) -> Result<&'static mut [u8], ErrorCode>;

    /// Report the length of the private key in bytes, as returned from `priv_key()`.
    /// A value of 0 indicates that the key does not exist.
    fn len(&self) -> usize;
}

/// An internal representation of generating asymetric Public/Private key
/// pairs.
///
/// This trait is useful for managing keys internally in Tock.
pub trait KeyGenerate<'a>: MutablePublicPrivateKey {
    /// Set the client. This client will be called when the `generate()`
    /// function is complete. If using an existing key this doesn't need to be
    /// used.
    fn set_client(&'a self, client: &'a dyn KeyGenerateClient<'a>);

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
    ///     - `BUSY`: The underlying `trng` is busy.
    ///     - `ALREADY`: A key is already imported or in the process of being
    ///                  generated.
    ///     - `OFF`: The underlying `trng` is powered down.
    ///     - `SIZE`: An invalid buffer size was supplied.
    fn generate(
        &'a self,
        trng: &'a dyn entropy::Entropy32,
        public_key_buffer: &'static mut [u8],
        private_key_buffer: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8], &'static mut [u8])>;
}

pub trait RsaKey: PublicKey {
    /// Run the specified closure over the modulus, if it exists
    /// The modulus is returned MSB (big endian)
    /// Returns `Some()` if the key exists and the closure was called,
    /// otherwise returns `None`.
    fn map_modulus(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()>;

    /// The the modulus if it exists.
    /// The modulus is returned MSB (big endian)
    /// Returns `Some()` if the key exists otherwise returns `None`.
    /// The modulus can be returned by calling `import_public_key()` with
    /// the output of this function.
    fn take_modulus(&self) -> Option<&'static [u8]>;

    /// Returns the public exponent of the key pair if it exists
    fn public_exponent(&self) -> Option<u32>;
}

pub trait RsaPrivateKey: PublicPrivateKey + RsaKey {
    /// Returns the specified closure over the private exponent, if it exists
    /// The exponent is returned MSB (big endian)
    /// Returns `Some()` if the key exists and the closure was called,
    /// otherwise returns `None`.
    fn map_exponent(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()>;

    /// The the private exponent if it exists.
    /// The exponent is returned MSB (big endian)
    /// Returns `Some()` if the key exists otherwise returns `None`.
    /// The exponent can be returned by calling `import_private_key()` with
    /// the output of this function.
    fn take_exponent(&self) -> Option<&'static [u8]>;
}

pub trait MutableRsaKey: MutablePublicKey {
    /// Run the specified closure over the modulus, if it exists
    /// The modulus is returned MSB (big endian)
    /// Returns `Some()` if the key exists and the closure was called,
    /// otherwise returns `None`.
    fn map_modulus(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()>;

    /// The the modulus if it exists.
    /// The modulus is returned MSB (big endian)
    /// Returns `Some()` if the key exists otherwise returns `None`.
    /// The modulus can be returned by calling `import_public_key()` with
    /// the output of this function.
    fn take_modulus(&self) -> Option<&'static [u8]>;

    /// Returns the public exponent of the key pair if it exists
    fn public_exponent(&self) -> Option<u32>;
}

pub trait MutableRsaPrivateKey: MutablePublicPrivateKey + MutableRsaKey {
    /// Returns the specified closure over the private exponent, if it exists
    /// The exponent is returned MSB (big endian)
    /// Returns `Some()` if the key exists and the closure was called,
    /// otherwise returns `None`.
    fn map_exponent(&self, closure: &dyn Fn(&[u8]) -> ()) -> Option<()>;

    /// The the private exponent if it exists.
    /// The exponent is returned MSB (big endian)
    /// Returns `Some()` if the key exists otherwise returns `None`.
    /// The exponent can be returned by calling `import_private_key()` with
    /// the output of this function.
    fn take_exponent(&self) -> Option<&'static mut [u8]>;
}
