//! Interface for Digest

use crate::utilities::leasable_buffer::LeasableBuffer;
use crate::utilities::leasable_buffer::LeasableMutableBuffer;
use crate::ErrorCode;

/// Implement this trait and use `set_client()` in order to receive callbacks
/// for adding imutable data.
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait ClientData<'a, const L: usize> {
    /// This callback is called when the data has been added to the digest
    /// engine.
    /// On error or success `data` will contain a reference to the original
    /// data supplied to `add_data()`.
    fn add_data_done(&'a self, result: Result<(), ErrorCode>, data: &'static [u8]);
}

/// Implement this trait and use `set_client()` in order to receive callbacks
/// for adding mutable data.
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait ClientDataMut<'a, const L: usize> {
    /// This callback is called when the data has been added to the digest
    /// engine.
    /// On error or success `data` will contain a reference to the original
    /// data supplied to `add_data()`.
    fn add_data_done(&'a self, result: Result<(), ErrorCode>, data: &'static mut [u8]);
}

/// Implement this trait and use `set_client()` in order to receive callbacks when
/// a digest is completed.
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait ClientHash<'a, const L: usize> {
    /// This callback is called when a digest is computed.
    /// On error or success `digest` will contain a reference to the original
    /// data supplied to `run()`.
    fn hash_done(&'a self, result: Result<(), ErrorCode>, digest: &'static mut [u8; L]);
}

/// Implement this trait and use `set_client()` in order to receive callbacks when
/// a digest is completed and whether it matches the value to compare with.
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait ClientVerify<'a, const L: usize> {
    /// This callback is called when a verification is computed.
    /// On error or success `digest` will contain a reference to the original
    /// data supplied to `verify()`.
    /// On success the result indicate if the hashes match or don't.
    /// On failure the result will indicate an `ErrorCode`.
    fn verification_done(&'a self, result: Result<bool, ErrorCode>, compare: &'static mut [u8; L]);
}

pub trait Client<'a, const L: usize>:
    ClientData<'a, L> + ClientHash<'a, L> + ClientVerify<'a, L>
{
}

pub trait ClientMut<'a, const L: usize>:
    ClientDataMut<'a, L> + ClientHash<'a, L> + ClientVerify<'a, L>
{
}

impl<'a, T: ClientData<'a, L> + ClientHash<'a, L> + ClientVerify<'a, L>, const L: usize>
    Client<'a, L> for T
{
}

impl<'a, T: ClientDataMut<'a, L> + ClientHash<'a, L> + ClientVerify<'a, L>, const L: usize>
    ClientMut<'a, L> for T
{
}

pub trait ClientDataHash<'a, const L: usize>: ClientData<'a, L> + ClientHash<'a, L> {}

impl<'a, T: ClientData<'a, L> + ClientHash<'a, L>, const L: usize> ClientDataHash<'a, L> for T {}

pub trait ClientDataVerify<'a, const L: usize>: ClientData<'a, L> + ClientVerify<'a, L> {}

impl<'a, T: ClientData<'a, L> + ClientVerify<'a, L>, const L: usize> ClientDataVerify<'a, L> for T {}

pub trait ClientDataMutHash<'a, const L: usize>: ClientDataMut<'a, L> + ClientHash<'a, L> {}

impl<'a, T: ClientDataMut<'a, L> + ClientHash<'a, L>, const L: usize> ClientDataMutHash<'a, L>
    for T
{
}

pub trait ClientDataMutVerify<'a, const L: usize>:
    ClientDataMut<'a, L> + ClientVerify<'a, L>
{
}

impl<'a, T: ClientDataMut<'a, L> + ClientVerify<'a, L>, const L: usize> ClientDataMutVerify<'a, L>
    for T
{
}

/// Computes a digest (cryptographic hash) over mutable data.
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait DigestDataMut<'a, const L: usize> {
    /// Set the client instance which will receive the `add_data_done()`
    /// callback.
    /// This is not required if using the `set_client()` fuction from the
    /// `Digest` trait.
    #[allow(unused_variables)]
    fn set_data_client(&'a self, client: &'a dyn ClientDataMut<'a, L>) {}

    /// Add data to the digest block. This is the data that will be used
    /// for the hash function.
    /// Returns the number of bytes parsed on success
    /// There is no guarantee the data has been written until the `add_data_done()`
    /// callback is fired.
    /// On error the return value will contain a return code and the original data
    fn add_data(
        &self,
        data: LeasableMutableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static mut [u8])>;

    /// Clear the keys and any other sensitive data.
    /// This won't clear the buffers provided to this API, that is up to the
    /// user to clear.
    fn clear_data(&self);
}

/// Computes a digest (cryptographic hash) over data
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait DigestData<'a, const L: usize> {
    /// Set the client instance which will receive the `add_data_done()`
    /// callback.
    /// This is not required if using the `set_client()` fuction from the
    /// `Digest` trait.
    #[allow(unused_variables)]
    fn set_data_client(&'a self, client: &'a dyn ClientData<'a, L>) {}

    /// Add data to the digest block. This is the data that will be used
    /// for the hash function.
    /// Returns the number of bytes parsed on success
    /// There is no guarantee the data has been written until the `add_data_done()`
    /// callback is fired.
    /// On error the return value will contain a return code and the original data
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static [u8])>;

    /// Clear the keys and any other sensitive data.
    /// This won't clear the buffers provided to this API, that is up to the
    /// user to clear.
    fn clear_data(&self);
}

/// Computes a digest (cryptographic hash) over data provided through a
/// separate trait
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait DigestHash<'a, const L: usize> {
    /// Set the client instance which will receive the `hash_done()`
    /// callback.
    /// This is not required if using the `set_client()` fuction from the
    /// `Digest` trait.
    #[allow(unused_variables)]
    fn set_hash_client(&'a self, client: &'a dyn ClientHash<'a, L>) {}

    /// Request the hardware block to generate a Digest and stores the returned
    /// digest in the memory location specified.
    /// This doesn't return any data, instead the client needs to have
    /// set a `hash_done` handler to determine when this is complete.
    /// On error the return value will contain a return code and the original data
    /// If there is data from the `add_data()` command asyncrously waiting to
    /// be written it will be written before the operation starts.
    ///
    /// If an appropriate `set_mode*()` wasn't called before this function the
    /// implementation should try to use a default option. In the case where
    /// there is only one digest supported this should be used. If there is no
    /// suitable or obvious default option, the implementation can return an
    /// error with error code ENOSUPPORT.
    fn run(&'a self, digest: &'static mut [u8; L])
        -> Result<(), (ErrorCode, &'static mut [u8; L])>;
}

/// Computes a digest (cryptographic hash) over data provided through a
/// separate trait
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait DigestVerify<'a, const L: usize> {
    /// Set the client instance which will receive the `verification_done()`
    /// callback.
    /// This is not required if using the `set_client()` fuction from the
    /// `Digest` trait.
    #[allow(unused_variables)]
    fn set_verify_client(&'a self, client: &'a dyn ClientVerify<'a, L>) {}

    /// Compare the specified digest in the `compare` buffer to the calculated
    /// digest. This function is similar to `run()` and should be used instead
    /// of `run()` if the caller doesn't need to know the output, just if it
    /// matches a known value.
    ///
    /// For example:
    /// ```ignore
    ///     // Compute a digest on data
    ///     add_data(...);
    ///     add_data(...);
    ///
    ///     // Compare the computed digest generated to an existing digest
    ///     verify(...);
    /// ```
    /// NOTE: The above is just pseudo code. The user is expected to check for
    /// errors and wait for the asyncronous calls to complete.
    ///
    /// The verify function is useful to compare input with a known digest
    /// value. The verify function also saves callers from allocating a buffer
    /// for the digest when they just need verification.
    fn verify(
        &'a self,
        compare: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])>;
}

/// Computes a digest (cryptographic hash) over data or performs verification.
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait Digest<'a, const L: usize>:
    DigestData<'a, L> + DigestHash<'a, L> + DigestVerify<'a, L>
{
    /// Set the client instance which will receive `hash_done()`,
    /// `add_data_done()` and `verification_done()` callbacks.
    fn set_client(&'a self, client: &'a dyn Client<'a, L>);
}

/// Computes a digest (cryptographic hash) or performs verification
/// over mutable data
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait DigestMut<'a, const L: usize>:
    DigestDataMut<'a, L> + DigestHash<'a, L> + DigestVerify<'a, L>
{
    /// Set the client instance which will receive `hash_done()`,
    /// `add_data_done()` and `verification_done()` callbacks.
    fn set_client(&'a self, client: &'a dyn ClientMut<'a, L>);
}

/// Computes a digest (cryptographic hash) over data
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait DigestDataHash<'a, const L: usize>: DigestData<'a, L> + DigestHash<'a, L> {}

impl<'a, T: DigestData<'a, L> + DigestHash<'a, L>, const L: usize> DigestDataHash<'a, L> for T {}

/// Computes a digest (cryptographic hash) over mutable data
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait DigestDataHashMut<'a, const L: usize>: DigestDataMut<'a, L> + DigestHash<'a, L> {}

impl<'a, T: DigestDataMut<'a, L> + DigestHash<'a, L>, const L: usize> DigestDataHashMut<'a, L>
    for T
{
}

/// Performs a verification on data
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait DigestDataVerify<'a, const L: usize>: DigestData<'a, L> + DigestVerify<'a, L> {}

impl<'a, T: DigestData<'a, L> + DigestVerify<'a, L>, const L: usize> DigestDataVerify<'a, L> for T {}

/// Performs a verification on data
///
/// 'L' is the length of the 'u8' array to store the digest output.
pub trait DigestDataVerifyMut<'a, const L: usize>:
    DigestDataMut<'a, L> + DigestVerify<'a, L>
{
}

impl<'a, T: DigestDataMut<'a, L> + DigestVerify<'a, L>, const L: usize> DigestDataVerifyMut<'a, L>
    for T
{
}

pub trait Sha224 {
    /// Call before `Digest::run()` to perform Sha224
    fn set_mode_sha224(&self) -> Result<(), ErrorCode>;
}

pub trait Sha256 {
    /// Call before `Digest::run()` to perform Sha256
    fn set_mode_sha256(&self) -> Result<(), ErrorCode>;
}

pub trait Sha384 {
    /// Call before `Digest::run()` to perform Sha384
    fn set_mode_sha384(&self) -> Result<(), ErrorCode>;
}

pub trait Sha512 {
    /// Call before `Digest::run()` to perform Sha512
    fn set_mode_sha512(&self) -> Result<(), ErrorCode>;
}

pub trait HMACSha256 {
    /// Call before `Digest::run()` to perform HMACSha256
    ///
    /// The key used for the HMAC is passed to this function.
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode>;
}

pub trait HMACSha384 {
    /// Call before `Digest::run()` to perform HMACSha384
    ///
    /// The key used for the HMAC is passed to this function.
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode>;
}

pub trait HMACSha512 {
    /// Call before `Digest::run()` to perform HMACSha512
    ///
    /// The key used for the HMAC is passed to this function.
    fn set_mode_hmacsha512(&self, key: &[u8]) -> Result<(), ErrorCode>;
}
