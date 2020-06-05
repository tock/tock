//! Interface for Digest

use crate::common::leasable_buffer::LeasableBuffer;
use crate::returncode::ReturnCode;

/// The 'types' of digests, this should define the output size of the digest
/// operations.
pub trait DigestType: Eq + Copy + Clone + Sized + AsRef<[u8]> + AsMut<[u8]> {}

impl DigestType for [u8; 32] {}

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait Client<'a, T: DigestType> {
    /// This callback is called when the data has been added to the digest
    /// engine.
    /// On error or success `data` will contain a reference to the original
    /// data supplied to `add_data()`.
    fn add_data_done(&'a self, result: Result<(), ReturnCode>, data: &'static mut [u8]);

    /// This callback is called when a digest is computed.
    /// On error or success `digest` will contain a reference to the original
    /// data supplied to `run()`.
    fn hash_done(&'a self, result: Result<(), ReturnCode>, digest: &'static mut T);
}

/// Computes a digest (cryptographic hash) over data
pub trait Digest<'a, T: DigestType> {
    /// Set the client instance which will receive `hash_done()` and
    /// `add_data_done()` callbacks.
    /// This callback is called when the data has been added to the digest
    /// engine.
    /// The callback should follow the `Client` `add_data_done` callback.
    fn set_client(&'a self, client: &'a dyn Client<'a, T>);

    /// Add data to the digest block. This is the data that will be used
    /// for the hash function.
    /// Returns the number of bytes parsed on success
    /// There is no guarantee the data has been written until the `add_data_done()`
    /// callback is fired.
    /// On error the return value will contain a return code and the original data
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ReturnCode, &'static mut [u8])>;

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
    fn run(&'a self, digest: &'static mut T) -> Result<(), (ReturnCode, &'static mut T)>;

    /// Clear the keys and any other sensitive data.
    /// This won't clear the buffers provided to this API, that is up to the
    /// user to clear.
    fn clear_data(&self);
}

pub trait HMACSha256 {
    /// Call before `Digest::run()` to perform HMACSha256
    ///
    /// The key used for the HMAC is passed to this function.
    fn set_mode_hmacsha256(&self, key: &[u8; 32]) -> Result<(), ReturnCode>;
}
