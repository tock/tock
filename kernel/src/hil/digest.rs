// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for computing digests (hashes, cryptographic hashes, and
//! HMACs) over data.

use crate::utilities::leasable_buffer::SubSlice;
use crate::utilities::leasable_buffer::SubSliceMut;
use crate::ErrorCode;

/// A specific digest algorithm and the buffer needed to hold the output of that
/// algorithm.
pub trait DigestAlgorithm {
    /// Buffer to store the output of the algorithm, i.e., the actual digest.
    type Digest: AsRef<[u8]> + AsMut<[u8]> + Default;
}

/// Helper object so we can construct a default `[u8; 48]`.
pub struct DigestBuffer48([u8; 48]);
impl Default for DigestBuffer48 {
    fn default() -> Self {
        Self([0; 48])
    }
}
impl AsRef<[u8]> for DigestBuffer48 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl AsMut<[u8]> for DigestBuffer48 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Helper object so we can construct a default `[u8; 64]`.
pub struct DigestBuffer64([u8; 64]);
impl Default for DigestBuffer64 {
    fn default() -> Self {
        Self([0; 64])
    }
}
impl AsRef<[u8]> for DigestBuffer64 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl AsMut<[u8]> for DigestBuffer64 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// SHA224 Hash
pub struct Sha224;
impl DigestAlgorithm for Sha224 {
    type Digest = [u8; 28];
}

/// SHA256 Hash
pub struct Sha256;
impl DigestAlgorithm for Sha256 {
    type Digest = [u8; 32];
}

/// SHA384 Hash
pub struct Sha384;
impl DigestAlgorithm for Sha384 {
    type Digest = DigestBuffer48;
}

/// SHA512 Hash
pub struct Sha512;
impl DigestAlgorithm for Sha512 {
    type Digest = DigestBuffer64;
}

/// SHA256 HMAC
pub struct HmacSha256Hmac;
impl DigestAlgorithm for HmacSha256Hmac {
    type Digest = [u8; 32];
}

/// SHA384 HMAC
pub struct HmacSha384Hmac;
impl DigestAlgorithm for HmacSha384Hmac {
    type Digest = DigestBuffer48;
}

/// SHA512 HMAC
pub struct HmacSha512Hmac;
impl DigestAlgorithm for HmacSha512Hmac {
    type Digest = DigestBuffer64;
}

/// Implement this trait and use `set_client()` in order to receive callbacks
/// when data has been added to a digest.
///
/// `DigestAlgorithm` is the type of digest the data will be used to compute.
pub trait ClientData<D: DigestAlgorithm> {
    /// Called when the data has been added to the digest. `data` is
    /// the `SubSlice` passed in the call to `add_data`, whose
    /// active slice contains the data that was not added. On `Ok`,
    /// `data` has an active slice of size zero (all data was added).
    /// Valid `ErrorCode` values are:
    ///  - OFF: the underlying digest engine is powered down and
    ///  cannot be used.
    ///  - BUSY: there is an outstanding `add_data`, `add_data_mut`,
    ///  `run`, or `verify` operation, so the digest engine is busy
    ///  and cannot accept more data.
    ///  - SIZE: the active slice of the SubSlice has zero size.
    ///  - CANCEL: the operation was cancelled by a call to `clear_data`.
    ///  - FAIL: an internal failure.
    fn add_data_done(&self, result: Result<(), ErrorCode>, data: SubSlice<'static, u8>);

    /// Called when the data has been added to the digest. `data` is
    /// the `SubSliceMut` passed in the call to
    /// `add_mut_data`, whose active slice contains the data that was
    /// not added. On `Ok`, `data` has an active slice of size zero
    /// (all data was added). Valid `ErrorCode` values are:
    ///  - OFF: the underlying digest engine is powered down and
    ///  cannot be used.
    ///  - BUSY: there is an outstanding `add_data`, `add_data_mut`,
    ///  `run`, or `verify` operation, so the digest engine is busy
    ///  and cannot accept more data.
    ///  - SIZE: the active slice of the SubSlice has zero size.
    ///  - CANCEL: the operation was cancelled by a call to `clear_data`.
    ///  - FAIL: an internal failure.
    fn add_mut_data_done(&self, result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>);
}

/// Implement this trait and use `set_client()` in order to receive callbacks
/// when a digest is completed.
///
/// `DigestAlgorithm` is the type of digest computed and the data structure to
/// store the computed digest.
pub trait ClientHash<D: DigestAlgorithm> {
    /// Called when a digest is computed. `digest` is the same
    /// reference passed to `run()` to store the hash value. If
    /// `result` is `Ok`, `digest` stores the computed hash. If
    /// `result` is `Err`, the data stored in `digest` is undefined
    /// and may have any value. Valid `ErrorCode` values are:
    ///  - OFF: the underlying digest engine is powered down and
    ///  cannot be used.
    ///  - BUSY: there is an outstanding `add_data`, `add_data_mut`,
    ///  `run`, or `verify` operation, so the digest engine is busy
    ///  and cannot perform a hash.
    ///  - CANCEL: the operation was cancelled by a call to `clear_data`.
    ///  - NOSUPPORT: the requested digest algorithm is not supported,
    ///  or one was not requested.
    ///  - FAIL: an internal failure.
    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut D::Digest);
}

/// Implement this trait and use `set_client()` in order to receive callbacks when
/// digest verification is complete.
///
/// `DigestAlgorithm` is the type of digest verified and the data structure to
/// store the verified digest.
pub trait ClientVerify<D: DigestAlgorithm> {
    /// Called when a verification is computed.  `compare` is the
    /// reference supplied to `verify()` and the data stored in
    /// `compare` is unchanged.  On `Ok` the `bool` indicates if the
    /// computed hash matches the value in `compare`. Valid
    /// `ErrorCode` values are:
    ///  - OFF: the underlying digest engine is powered down and
    ///  cannot be used.
    ///  - BUSY: there is an outstanding `add_data`, `add_data_mut`,
    ///  `run`, or `verify` operation, so the digest engine is busy
    ///  and cannot verify a hash.
    ///  - CANCEL: the operation was cancelled by a call to `clear_data`.
    ///  - NOSUPPORT: the requested digest algorithm is not supported,
    ///  or one was not requested.
    ///  - FAIL: an internal failure.
    fn verification_done(&self, result: Result<bool, ErrorCode>, compare: &'static mut D::Digest);
}

pub trait Client<D: DigestAlgorithm>: ClientData<D> + ClientHash<D> + ClientVerify<D> {}

impl<T: ClientData<D> + ClientHash<D> + ClientVerify<D>, D: DigestAlgorithm> Client<D> for T {}

pub trait ClientDataHash<D: DigestAlgorithm>: ClientData<D> + ClientHash<D> {}
impl<T: ClientData<D> + ClientHash<D>, D: DigestAlgorithm> ClientDataHash<D> for T {}

pub trait ClientDataVerify<D: DigestAlgorithm>: ClientData<D> + ClientVerify<D> {}
impl<T: ClientData<D> + ClientVerify<D>, D: DigestAlgorithm> ClientDataVerify<D> for T {}

/// Adding data (mutable or immutable) to a digest. There are two
/// separate methods, `add_data` for immutable data (e.g., flash) and
/// `add_mut_data` for mutable data (e.g., RAM). Each has its own
/// callback, but only one operation may be in flight at any time.
///
/// `DigestAlgorithm` is the type of digest the data will be used to compute.
pub trait DigestData<'a, D: DigestAlgorithm> {
    /// Set the client instance which will handle the `add_data_done`
    /// and `add_mut_data_done` callbacks.
    fn set_data_client(&'a self, client: &'a dyn ClientData<D>);

    /// Add data to the input of the hash function/digest. `Ok`
    /// indicates all of the active bytes in `data` will be added.
    /// There is no guarantee the data has been added to the digest
    /// until the `add_data_done()` callback is called.  On error the
    /// cause of the error is returned along with the SubSlice
    /// unchanged (it has the same range of active bytes as the call).
    /// Valid `ErrorCode` values are:
    ///  - OFF: the underlying digest engine is powered down and
    ///  cannot be used.
    ///  - BUSY: there is an outstanding `add_data`, `add_data_mut`,
    ///  `run`, or `verify` operation, so the digest engine is busy
    ///  and cannot accept more data.
    ///  - SIZE: the active slice of the SubSlice has zero size.
    fn add_data(
        &self,
        data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)>;

    /// Add data to the input of the hash function/digest. `Ok`
    /// indicates all of the active bytes in `data` will be added.
    /// There is no guarantee the data has been added to the digest
    /// until the `add_mut_data_done()` callback is called.  On error
    /// the cause of the error is returned along with the
    /// SubSlice unchanged (it has the same range of active
    /// bytes as the call).  Valid `ErrorCode` values are:
    ///  - OFF: the underlying digest engine is powered down and
    ///  cannot be used.
    ///  - BUSY: there is an outstanding `add_data`, `add_data_mut`,
    ///  `run`, or `verify` operation, so the digest engine is busy
    ///  and cannot accept more data.
    ///  - SIZE: the active slice of the SubSlice has zero size.
    fn add_mut_data(
        &self,
        data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)>;

    /// Clear the keys and any other internal state. Any pending
    /// operations terminate and issue a callback with an
    /// `ErrorCode::CANCEL`. This call does not clear buffers passed
    /// through `add_mut_data`, those are up to the client clear.
    fn clear_data(&self);
}

/// Computes a digest (cryptographic hash) over data provided through a
/// separate trait.
///
/// `DigestAlgorithm` is the type of digest computed and the data structure to
/// store the computed digest.
pub trait DigestHash<'a, D: DigestAlgorithm> {
    /// Set the client instance which will receive the `hash_done()`
    /// callback.
    fn set_hash_client(&'a self, client: &'a dyn ClientHash<D>);

    /// Compute a digest of all of the data added with `add_data` and
    /// `add_data_mut`, storing the computed value in `digest`.  The
    /// computed value is returned in a `hash_done` callback.  On
    /// error the return value will contain a return code and the
    /// slice passed in `digest`. Valid `ErrorCode` values are:
    ///  - OFF: the underlying digest engine is powered down and
    ///  cannot be used.
    ///  - BUSY: there is an outstanding `add_data`, `add_data_mut`,
    ///  `run`, or `verify` operation, so the digest engine is busy
    ///  and cannot accept more data.
    ///  - SIZE: the active slice of the SubSlice has zero size.
    ///  - NOSUPPORT: the currently selected digest algorithm is not
    ///  supported.
    ///
    /// If an appropriate `set_mode*()` wasn't called before this function the
    /// implementation should try to use a default option. In the case where
    /// there is only one digest supported this should be used. If there is no
    /// suitable or obvious default option, the implementation can return
    /// `ErrorCode::NOSUPPORT`.
    fn run(
        &'a self,
        digest: &'static mut D::Digest,
    ) -> Result<(), (ErrorCode, &'static mut D::Digest)>;
}

/// Verifies a digest (cryptographic hash) over data provided through a
/// separate trait
///
/// `DigestAlgorithm` is the type of digest verified and the data structure to
/// store the verified digest.
pub trait DigestVerify<'a, D: DigestAlgorithm> {
    /// Set the client instance which will receive the `verification_done()`
    /// callback.
    fn set_verify_client(&'a self, client: &'a dyn ClientVerify<D>);

    /// Compute a digest of all of the data added with `add_data` and
    /// `add_data_mut` then compare it with value in `compare`.  The
    /// compare value is returned in a `verification_done` callback, along with
    /// a boolean indicating whether it matches the computed value. On
    /// error the return value will contain a return code and the
    /// slice passed in `compare`. Valid `ErrorCode` values are:
    ///  - OFF: the underlying digest engine is powered down and
    ///  cannot be used.
    ///  - BUSY: there is an outstanding `add_data`, `add_data_mut`,
    ///  `run`, or `verify` operation, so the digest engine is busy
    ///  and cannot accept more data.
    ///  - SIZE: the active slice of the SubSlice has zero size.
    ///  - NOSUPPORT: the currently selected digest algorithm is not
    ///  supported.
    ///
    /// If an appropriate `set_mode*()` wasn't called before this function the
    /// implementation should try to use a default option. In the case where
    /// there is only one digest supported this should be used. If there is no
    /// suitable or obvious default option, the implementation can return
    /// `ErrorCode::NOSUPPORT`.
    fn verify(
        &'a self,
        compare: &'static mut D::Digest,
    ) -> Result<(), (ErrorCode, &'static mut D::Digest)>;
}

/// Computes a digest (cryptographic hash) over data or performs verification.
///
/// `DigestAlgorithm` is the type of digest computed and the data structure to
/// store the computed digest.
pub trait Digest<'a, D: DigestAlgorithm>:
    DigestData<'a, D> + DigestHash<'a, D> + DigestVerify<'a, D>
{
    /// Set the client instance which will receive `hash_done()`,
    /// `add_data_done()` and `verification_done()` callbacks.
    fn set_client(&'a self, client: &'a dyn Client<D>);
}

/// Computes a digest (cryptographic hash) over data.
///
/// `DigestAlgorithm` is the type of digest computed and the data structure to
/// store the computed digest.
pub trait DigestDataHash<'a, D: DigestAlgorithm>: DigestData<'a, D> + DigestHash<'a, D> {
    /// Set the client instance which will receive `hash_done()` and
    /// `add_data_done()` callbacks.
    fn set_client(&'a self, client: &'a dyn ClientDataHash<D>);
}

/// Verify a digest (cryptographic hash) over data.
///
/// `DigestAlgorithm` is the type of digest computed and the data structure to
/// store the computed digest.
pub trait DigestDataVerify<'a, D: DigestAlgorithm>:
    DigestData<'a, D> + DigestVerify<'a, D>
{
    /// Set the client instance which will receive `verify_done()` and
    /// `add_data_done()` callbacks.
    fn set_client(&'a self, client: &'a dyn ClientDataVerify<D>);
}

pub trait HmacSha256 {
    /// Call before adding data to perform HMACSha256
    ///
    /// The key used for the HMAC is passed to this function.
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode>;
}

pub trait HmacSha384 {
    /// Call before adding data to perform HMACSha384
    ///
    /// The key used for the HMAC is passed to this function.
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode>;
}

pub trait HmacSha512 {
    /// Call before adding data to perform HMACSha512
    ///
    /// The key used for the HMAC is passed to this function.
    fn set_mode_hmacsha512(&self, key: &[u8]) -> Result<(), ErrorCode>;
}
