use kernel::{hil::symmetric_encryption, utilities::leasable_buffer::SubSliceMut, ErrorCode};
pub trait Aes128Ctr<'a> {
    /// Set the client instance which will receive `crypt_done()` callbacks.
    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>);

    /// Set the key and IV for AES128CTR operation.
    ///
    /// The IV can be of variable length, and must be
    /// checked by the implementor to ensure it is
    /// valid.
    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        iv: &[u8],
    ) -> Result<(), ErrorCode>;

    /// Perform AES128CTR crypt operation. This must be used after `setup_cipher()`
    /// and will use whatever the key and IV have been set to.
    ///
    /// CAUTION: The IV must be set to a value that is unique for each
    /// crypt operation. If the same IV is used for multiple operations,
    /// it can lead to security vulnerabilities.
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    >;
}

pub trait Aes128Cbc<'a> {
    /// Set the client instance which will receive `crypt_done()` callbacks.
    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>);

    /// Set the key and IV for AES128CBC operation.
    ///
    /// The IV must be of length `AES128_BLOCK_SIZE`.
    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        iv: &[u8; symmetric_encryption::AES128_BLOCK_SIZE],
    ) -> Result<(), ErrorCode>;

    /// Perform AES128CBC crypt operation. This must be used after `setup_cipher()`
    /// and will use whatever key and IV have been set to.
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    >;
}

pub trait Aes128Ecb<'a> {
    /// Set the client instance which will receive `crypt_done()` callbacks.
    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>);

    /// Set the key for AES128ECB operation.
    ///
    /// The key must be of length `AES128_KEY_SIZE`.
    /// Note: ECB does not use an IV.
    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
    ) -> Result<(), ErrorCode>;

    /// Perform AES128ECB crypt operation.
    ///
    /// This must be used after `setup_cipher()`
    /// and will use whatever key has been set to.
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    >;
}

pub const CCM_NONCE_LENGTH: usize = 13;

pub trait Aes128Ccm<'a> {
    /// Set the client instance which will receive `crypt_done()` callbacks.
    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>);

    /// Set the key and nonce for AES128CCM operation.
    ///
    /// CCM allows a nonce of variable length, but it
    /// must be at least 7 bytes and at most 13 bytes.
    /// This invariant must be checked and enforced by
    /// the implementor.
    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        nonce: &[u8],
    ) -> Result<(), ErrorCode>;

    /// Perform AES128CCM crypt operation.
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    >;
}

pub trait Aes128Gcm<'a> {
    /// Set the client instance which will receive `crypt_done()` callbacks.
    fn set_client(&self, client: &'a dyn symmetric_encryption::Client<'a>);

    /// Set the key and nonce for AES128GCM operation.
    ///
    /// CCM allows a nonce of variable length,
    /// but it must be at least 7 bytes and at most 13 bytes.
    /// This invariant must be checked and enforced by
    /// the implementor.
    fn setup_cipher(
        &self,
        key: &[u8; symmetric_encryption::AES128_KEY_SIZE],
        nonce: &[u8; 12],
    ) -> Result<(), ErrorCode>;

    /// Perform AES128GCM crypt operation. Although technically flexible in
    /// the length of the nonce, it is recommended to use a 12-byte nonce.
    /// For simplicity and consistency, a 12-byte nonce is enforced.
    fn crypt(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        dest: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            Option<SubSliceMut<'static, u8>>,
            SubSliceMut<'static, u8>,
        ),
    >;
}
