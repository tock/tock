//! Interface for symmetric-cipher encryption
//!
//! see boards/imix/src/aes_test.rs for example usage

use returncode::ReturnCode;

/// Implement this trait and use `set_client()` in order to receive callbacks from an `AES128`
/// instance.
pub trait Client {
    fn crypt_done(&self);
}

/// The number of bytes used for AES block operations.  Keys and IVs must have this length,
/// and encryption/decryption inputs must be have a multiple of this length.
pub const AES128_BLOCK_SIZE: usize = 16;

pub trait AES128<'a> {
    /// Enable the AES hardware; must be called before any other methods
    fn enable(&self);

    /// Disable the AES hardware
    fn disable(&self);

    /// Set the client instance which will receive `crypt_done()` callbacks
    fn set_client(&'a self, client: &'a Client);

    /// Set the encryption key; returns `EINVAL` if length is not `AES128_BLOCK_SIZE`
    fn set_key(&'a self, key: &'a [u8]) -> ReturnCode;

    /// Set the IV (or initial counter); returns `EINVAL` if length is not `AES128_BLOCK_SIZE`
    fn set_iv(&'a self, iv: &'a [u8]) -> ReturnCode;

    /// Set the source buffer.  If this is full, the encryption
    /// input will be this entire buffer, and its size must match
    /// `stop_index - start_index` when `crypt()` is called.
    /// If this is empty, the destination buffer will be read
    /// to provide the plaintext input.
    fn set_source(&'a self, buf: Option<&'a [u8]>) -> ReturnCode;

    /// Set the destination buffer.  If `set_source()` has not
    /// been used to pass a source buffer, this buffer will also
    /// provide the encryption input, which will be overwritten.
    /// The option should be full whenever `crypt()` is called.
    /// Returns SUCCESS if the buffer was installed, or EBUSY
    /// if the encryption unit is still busy.
    fn put_dest(&'a self, dest: Option<&'a mut [u8]>) -> ReturnCode;

    /// Return the destination buffer, if any.
    /// Returns EBUSY if the encryption unit is still busy.
    fn take_dest(&'a self) -> Result<Option<&'a mut [u8]>, ReturnCode>;

    /// Begin a new message (with the configured IV) when `crypt()` is next
    /// called.  Multiple calls to `crypt()` (accompanied by `set_source()` or
    /// `put_dest`) may be made between calls to `start_message()`, allowing the
    /// encryption context to extend over non-contiguous extents of data.
    fn start_message(&self);

    /// Request an encryption/decryption
    ///
    /// The indices `start_index` and `stop_index` must be valid offsets in
    /// a buffer previously passed in with `set_data`, and the length
    /// `stop_index - start_index` must be a multiple of
    /// `AES128_BLOCK_SIZE`.  Otherwise, INVAL will be returned.
    ///
    /// If SUCCESS is returned, the client's `crypt_done` method will eventually
    /// be called, and the portion of the data buffer between `start_index`
    /// and `stop_index` will hold the result of the encryption/decryption.
    ///
    /// For correct operation, the methods `set_key` and `set_iv` must have
    /// previously been called to set the buffers containing the
    /// key and the IV (or initial counter value), and a method `set_mode_*()`
    /// must have been called to set the desired mode.  These settings persist
    /// across calls to `crypt()`.
    fn crypt(&self, start_index: usize, stop_index: usize) -> ReturnCode;
}

pub trait AES128Ctr {
    /// Call before `AES128::crypt()` to perform AES128Ctr
    fn set_mode_aes128ctr(&self, encrypting: bool);
}

pub trait AES128CBC {
    /// Call before `AES128::crypt()` to perform AES128CBC
    fn set_mode_aes128cbc(&self, encrypting: bool);
}
