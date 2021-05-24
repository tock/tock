//! Interface for Hardware accelerators

use crate::common::leasable_buffer::LeasableBuffer;
use crate::ErrorCode;

/// Implement this trait and use `set_client()` in order to receive callbacks.
pub trait Client<'a, const T: usize> {
    /// This callback is called when the binary data has been loaded
    /// On error or success `input` will contain a reference to the original
    /// data supplied to `load_binary()`.
    fn binary_load_done(&'a self, result: Result<(), ErrorCode>, input: &'static mut [u8]);

    /// This callback is called when a operation is computed.
    /// On error or success `output` will contain a reference to the original
    /// data supplied to `run()`.
    fn op_done(&'a self, result: Result<(), ErrorCode>, output: &'static mut [u8; T]);
}

/// A generic accelerator. This can be used to accelerate any type of
/// operation.
pub trait Accel<'a, const T: usize> {
    /// Set the client instance which will receive
    fn set_client(&'a self, client: &'a dyn Client<'a, T>);

    /// Load the acceleration binary data into the accelerator.
    /// This data will be accelerator specific and could be an
    /// elf file which will be run or could be binary settings used to
    /// configure the accelerator.
    /// This function can be called multiple times if multiple binary blobs
    /// are required.
    /// There is no guarantee the data has been written until the `binary_load_done()`
    /// callback is fired.
    /// On error the return value will contain a return code and the original data
    fn load_binary(
        &self,
        input: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Set implementation specific properties.
    /// This function is used to set hardware specific properties.
    /// The properties are set using a key/value system. The key will
    /// indicate what property is being set and the value is the value to be
    /// set. For a list of possible keys check the hardware implementation
    /// documentation.
    /// This function can be used to set start addresses, input seeds or other
    /// properties.
    fn set_property(&self, key: usize, value: usize) -> Result<(), ErrorCode>;

    /// Run the acceleration operation.
    /// This doesn't return any data, instead the client needs to have
    /// set a `op_done` handler to determine when this is complete.
    /// On error the return value will contain a return code and the original data
    /// If there is data from the `load_binary()` command asyncrously waiting to
    /// be written it will be written before the operation starts.
    fn run(&'a self, output: &'static mut [u8; T])
        -> Result<(), (ErrorCode, &'static mut [u8; T])>;

    /// Clear the keys and any other sensitive data.
    /// This won't clear the buffers provided to this API, that is up to the
    /// user to clear those.
    fn clear_data(&self);
}
