//! The Flash Controller interface with hardware

use crate::error_codes::ErrorCode;

/// Implementation required for the flash controller hardware. This
/// should read, write and erase flash from the hardware using the
/// flash controller.
///
/// This is the public trait for the Flash controller implementation.
///
/// The size of the regions (pages) must be the smallest size that can be
/// erased in a single operation. This is specified as the constant `S`
/// when implementing `FlashController` and `TicKV` and it must match
/// the length of the `read_buffer`.
///
/// The start and end address of the FlashController must be aligned
/// to the size of regions.
/// All `region_number`s and `address`es are offset from zero. If you
/// want to use flash that doesn't start at zero, or is a partition
/// offset from the start of flash you will need to add that offset
/// to the values in your implementation.
///
/// The boiler plate for an implementation will look something like this
///
/// ```rust
/// use tickv::error_codes::ErrorCode;
/// use tickv::flash_controller::FlashController;
///
/// #[derive(Default)]
/// struct FlashCtrl {}
///
/// impl FlashCtrl {
///     fn new() -> Self {
///         Self { /* fields */ }
///     }
/// }
///
/// impl FlashController<1024> for FlashCtrl {
///     fn read_region(&self, region_number: usize, offset: usize, buf: &mut [u8; 1024]) -> Result<(), ErrorCode> {
///         unimplemented!()
///     }
///
///     fn write(&self, address: usize, buf: &[u8]) -> Result<(), ErrorCode> {
///         unimplemented!()
///     }
///
///     fn erase_region(&self, region_number: usize) -> Result<(), ErrorCode> {
///         unimplemented!()
///     }
/// }
/// ```
pub trait FlashController<const S: usize> {
    /// This function must read the data from the flash region specified by
    /// `region_number` into `buf`. The length of the data read should be the
    /// same length as buf. `offset` indicates an offset into the region that
    /// should be read.
    ///
    /// On success it should return nothing, on failure it
    /// should return ErrorCode::ReadFail.
    ///
    /// If the read operation is to be complete asynchronously then
    /// `read_region()` can return `ErrorCode::ReadNotReady(region_number)`.
    /// By returning `ErrorCode::ReadNotReady(region_number)`
    /// `read_region()` can indicate that the operation should be retried in
    /// the future.
    /// After running the `continue_()` functions after a async
    /// `read_region()` has returned `ErrorCode::ReadNotReady(region_number)`
    /// the `read_region()` function will be called again and this time should
    /// return the data.
    fn read_region(
        &self,
        region_number: usize,
        offset: usize,
        buf: &mut [u8; S],
    ) -> Result<(), ErrorCode>;

    /// This function must write the length of `buf` to the specified address
    /// in flash.
    /// If the length of `buf` is smaller then the minimum supported write size
    /// the implementation can write a larger value. This should be done by first
    /// reading the value, making the changed from `buf` and then writing it back.
    ///
    /// On success it should return nothing, on failure it
    /// should return ErrorCode::WriteFail.
    ///
    /// If the write operation is to be complete asynchronously then
    /// `write()` can return `ErrorCode::WriteNotReady(region_number)`.
    /// By returning `ErrorCode::WriteNotReady(region_number)`
    /// `read_region()` can indicate that the operation should be retried in
    /// the future. Note that that region will not be written
    /// again so the write must occur otherwise the operation fails.
    fn write(&self, address: usize, buf: &[u8]) -> Result<(), ErrorCode>;

    /// This function must erase the region specified by `region_number`.
    ///
    /// On success it should return nothing, on failure it
    /// should return ErrorCode::WriteFail.
    ///
    /// If the erase is going to happen asynchronously then this should return
    /// `EraseNotReady(region_number)`. Note that that region will not be erased
    /// again so the erasure must occur otherwise the operation fails.
    fn erase_region(&self, region_number: usize) -> Result<(), ErrorCode>;
}
