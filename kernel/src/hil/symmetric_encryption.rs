//! Interface for symmetric-cipher encryption
//!
//! see boards/imix/src/aes_test.rs for example usage

use returncode::ReturnCode;

/// Implement this trait and use `set_client()` in order to receive callbacks from an `AES128`
/// instance.
pub trait Client<'a> {
    fn crypt_done(&'a self, source: Option<&'a mut [u8]>, dest: &'a mut [u8]);
}

/// The number of bytes used for AES block operations.  Keys and IVs must have this length,
/// and encryption/decryption inputs must be have a multiple of this length.
pub const AES128_BLOCK_SIZE: usize = 16;
pub const AES128_KEY_SIZE: usize = 16;

pub trait AES128<'a> {
    /// Enable the AES hardware.
    /// Must be called before any other methods
    fn enable(&self);

    /// Disable the AES hardware
    fn disable(&self);

    /// Set the client instance which will receive `crypt_done()` callbacks
    fn set_client(&'a self, client: &'a Client<'a>);

    /// Set the encryption key.
    /// Returns `EINVAL` if length is not `AES128_KEY_SIZE`
    fn set_key(&self, key: &[u8]) -> ReturnCode;

    /// Set the IV (or initial counter).
    /// Returns `EINVAL` if length is not `AES128_BLOCK_SIZE`
    fn set_iv(&self, iv: &[u8]) -> ReturnCode;

    /// Begin a new message (with the configured IV) when `crypt()` is
    /// next called.  Multiple calls to `crypt()` may be made between
    /// calls to `start_message()`, allowing the encryption context to
    /// extend over non-contiguous extents of data.
    ///
    /// If an encryption operation is in progress, this method instead
    /// has no effect.
    fn start_message(&self);

    /// Request an encryption/decryption
    ///
    /// If the source buffer is not `None`, the encryption input
    /// will be that entire buffer.  Otherwise the destination buffer
    /// at indices between `start_index` and `stop_index` will
    /// provide the input, which will be overwritten.
    ///
    /// If `None` is returned, the client's `crypt_done` method will eventually
    /// be called, and the portion of the data buffer between `start_index`
    /// and `stop_index` will hold the result of the encryption/decryption.
    ///
    /// If `Some(result, source, dest)` is returned, `result` is the
    /// error condition and `source` and `dest` are the buffers that
    /// were passed to `crypt`.
    ///
    /// The indices `start_index` and `stop_index` must be valid
    /// offsets in the destination buffer, and the length
    /// `stop_index - start_index` must be a multiple of
    /// `AES128_BLOCK_SIZE`.  Otherwise, `Some(EINVAL, ...)` will be
    /// returned.
    ///
    /// If the source buffer is not `None`, its length must be
    /// `stop_index - start_index`.  Otherwise, `Some(EINVAL, ...)`
    /// will be returned.
    ///
    /// If an encryption operation is already in progress,
    /// `Some(EBUSY, ...)` will be returned.
    ///
    /// For correct operation, the methods `set_key` and `set_iv` must have
    /// previously been called to set the buffers containing the
    /// key and the IV (or initial counter value), and a method `set_mode_*()`
    /// must have been called to set the desired mode.  These settings persist
    /// across calls to `crypt()`.
    ///
    fn crypt(
        &'a self,
        source: Option<&'a mut [u8]>,
        dest: &'a mut [u8],
        start_index: usize,
        stop_index: usize,
    ) -> Option<(ReturnCode, Option<&'a mut [u8]>, &'a mut [u8])>;
}

pub trait AES128Ctr {
    /// Call before `AES128::crypt()` to perform AES128Ctr
    fn set_mode_aes128ctr(&self, encrypting: bool);
}

pub trait AES128CBC {
    /// Call before `AES128::crypt()` to perform AES128CBC
    fn set_mode_aes128cbc(&self, encrypting: bool);
}

pub trait CCMClient {
    /// `res` is SUCCESS if the encryption/decryption process succeeded. This
    /// does not mean that the message has been verified in the case of
    /// decryption.
    /// If we are encrypting: `tag_is_valid` is `true` iff `res` is SUCCESS.
    /// If we are decrypting: `tag_is_valid` is `true` iff `res` is SUCCESS and the
    /// message authentication tag is valid.
    fn crypt_done(&self, buf: &'static mut [u8], res: ReturnCode, tag_is_valid: bool);
}

pub const CCM_NONCE_LENGTH: usize = 13;

pub trait AES128CCM<'a> {
    /// Set the client instance which will receive `crypt_done()` callbacks
    fn set_client(&'a self, client: &'a CCMClient);

    /// Set the key to be used for CCM encryption
    fn set_key(&self, key: &[u8]) -> ReturnCode;

    /// Set the nonce (length NONCE_LENGTH) to be used for CCM encryption
    fn set_nonce(&self, nonce: &[u8]) -> ReturnCode;

    /// Try to begin the encryption/decryption process
    fn crypt(
        &self,
        buf: &'static mut [u8],
        a_off: usize,
        m_off: usize,
        m_len: usize,
        mic_len: usize,
        confidential: bool,
        encrypting: bool,
    ) -> (ReturnCode, Option<&'static mut [u8]>);
}
