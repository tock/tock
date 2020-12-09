//! Interface for reading, writing, and erasing flash storage pages.
//!
//! Operates on single pages. The page size is set by the associated type
//! `page`. Here is an example of a page type and implementation of this trait:
//!
//! ```rust
//! use core::ops::{Index, IndexMut};
//!
//! use kernel::hil;
//! use kernel::hil::flash::Page;
//! use kernel::ErrorCode;
//!
//! // Size in bytes
//! const PAGE_SIZE: u32 = 1024;
//!
//! struct NewChipPage(pub [u8; PAGE_SIZE as usize]);
//!
//! impl Default for NewChipPage {
//!     fn default() -> Self {
//!         Self {
//!             0: [0; PAGE_SIZE as usize],
//!         }
//!     }
//! }
//!
//! impl NewChipPage {
//!     fn len(&self) -> usize {
//!         self.0.len()
//!     }
//! }
//!
//! impl AsMut<[u8]> for NewChipPage {
//!     fn as_mut(&mut self) -> &mut [u8] {
//!         &mut self.0
//!     }
//! }
//!
//! impl<const W: usize, const E: usize> hil::flash::Flash<W, E> for NewChipStruct {
//!     fn read(
//!         &self,
//!         region: &Page,
//!         buf: &'static mut [u8],
//!     ) -> Result<(), (ErrorCode, &'static mut [u8])> {
//!        unimplemented!()
//!     }
//!
//!     fn get_read_region(&self, address: u64, length: u32) -> Result<Page, ErrorCode> {
//!        unimplemented!()
//!     }
//!
//!     fn write(
//!         &self,
//!         region: &Page,
//!         buf: &'static mut [u8],
//!     ) -> Result<(), (ErrorCode, &'static mut [u8])> {
//!         unimplemented!()
//!     }
//!
//!     fn get_write_region(&self, address: u64, length: u32) -> Result<Page, ErrorCode> {
//!        unimplemented!()
//!     }
//!
//!     fn erase(&self, region: &Page) -> Result<(), ErrorCode> {
//!         unimplemented!()
//!     }
//!
//!     fn get_erase_region(&self, address: u64, length: u32) -> Result<Page, ErrorCode> {
//!        unimplemented!()
//!     }
//! }
//!
//! struct NewChipStruct {};
//!
//! impl<'a, C> hil::flash::HasClient<'a, C> for NewChipStruct {
//!     fn set_client(&'a self, client: &'a C) { }
//! }
//!
//! impl hil::flash::LegacyFlash for NewChipStruct {
//!     type Page = NewChipPage;
//!
//!     fn read_page(&self, page_number: usize, buf: &'static mut Self::Page) -> Result<(), (ErrorCode, &'static mut Self::Page)> { Err((ErrorCode::FAIL, buf)) }
//!     fn write_page(&self, page_number: usize, buf: &'static mut Self::Page) -> Result<(), (ErrorCode, &'static mut Self::Page)> { Err((ErrorCode::FAIL, buf)) }
//!     fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> { Err(ErrorCode::FAIL) }
//! }
//! ```
//!
//! A user of this flash interface might look like:
//!
//! ```rust
//! use kernel::utilities::cells::TakeCell;
//! use kernel::hil;
//! use kernel::ErrorCode;
//!
//! pub struct FlashUser<'a, F: hil::flash::Flash<W, E> + 'static, const W: usize, const E: usize> {
//!     driver: &'a F,
//!     buffer: TakeCell<'static, [u8; W]>,
//! }
//!
//! impl<'a, F: hil::flash::Flash<W, E>, const W: usize, const E: usize> FlashUser<'a, F, W, E> {
//!     pub fn new(driver: &'a F, buffer: &'static mut [u8; W]) -> FlashUser<'a, F, W, E> {
//!         FlashUser {
//!             driver: driver,
//!             buffer: TakeCell::new(buffer),
//!         }
//!     }
//! }
//!
//! impl<'a, F: hil::flash::Flash<W, E>, const W: usize, const E: usize> hil::flash::Client<W, E> for FlashUser<'a, F, W, E> {
//!     fn read_complete(&self, read_buffer: &'static mut [u8], ret: Result<(), ErrorCode>) {}
//!     fn write_complete(&self, write_buffer: &'static mut [u8], ret: Result<(), ErrorCode>) {}
//!     fn erase_complete(&self, ret: Result<(), ErrorCode>) {}
//! }
//! ```

use crate::ErrorCode;

/// Flash errors returned in the callbacks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Success.
    CommandComplete,

    /// An error occurred during the flash operation.
    FlashError,
}

pub trait HasClient<'a, C> {
    /// Set the client for this flash peripheral. The client will be called
    /// when operations complete.
    fn set_client(&'a self, client: &'a C);
}

/// A page of writable persistent flash memory.
pub trait LegacyFlash {
    /// Type of a single flash page for the given implementation.
    type Page: AsMut<[u8]> + Default;

    /// Read a page of flash into the buffer.
    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)>;

    /// Write a page of flash from the buffer.
    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)>;

    /// Erase a page of flash by setting every byte to 0xFF.
    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode>;
}

/// Implement `Client` to receive callbacks from `LegacyFlash`.
pub trait LegacyClient<F: LegacyFlash> {
    /// Flash read complete.
    fn read_complete(&self, read_buffer: &'static mut F::Page, error: Error);

    /// Flash write complete.
    fn write_complete(&self, write_buffer: &'static mut F::Page, error: Error);

    /// Flash erase complete.
    fn erase_complete(&self, error: Error);
}

/// A checked flash region
pub struct Page {
    pub address: u64,
    pub length: u32,
}

/// A page of writeable persistent flash memory.
///
/// `W`: Should be the minimum number of bytes that can be written
///      in an operation.
/// `E`: Should be the minimum number of bytes that can be erased
///      in an operation.
pub trait Flash<const W: usize, const E: usize> {
    /// Read data from flash into a buffer.
    ///
    /// This function will read data stored in flash at `address` and
    /// `length` into the buffer `buf`.
    /// `address` is calculated as an offset from the start of the flash
    /// region.
    ///
    /// On success returns nothing
    /// On failure returns a `ErrorCode` and the buffer passed in.
    /// If `ErrorCode::NOSUPPORT` is returned then `read_page()`
    // should be used instead.
    fn read(
        &self,
        region: &Page,
        buf: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Create a `Page` from the provided `address` and `length`.
    ///
    /// The `Page->address` and Page->length` will be modified to match
    /// the flash hardware requirements.
    fn get_read_region(&self, address: u64, length: u32) -> Result<Page, ErrorCode>;

    /// Write data from a buffer to flash.
    ///
    /// This function will write the buffer `buf` to the `address` specified
    /// in flash.
    ///
    /// `address` must be aligned to `W`.
    /// The length of `buf` must be aligned to `W`.
    ///
    /// This function will not erase the page first. The user of this function
    /// must ensure that a page is erased before writing.
    /// Writes to flash can only turn a `1` to a `0`. To change a `0` to a `1`
    /// the region must be erased.
    ///
    /// Note that some hardware only allows a limited number of writes before
    /// an erase. If that is the case the implementation MUST return an error
    /// `ErrorCode::NOMEM` when this happens, even if the hardware silently
    /// ignores the write.
    ///
    /// On success returns nothing
    /// On failure returns a `ErrorCode` and the buffer passed in.
    fn write(
        &self,
        region: &Page,
        buf: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Create a `Page` from the provided `address` and `length`.
    ///
    /// The `Page->address` and Page->length` will be modified to match
    /// the flash hardware requirements.
    fn get_write_region(&self, address: u64, length: u32) -> Result<Page, ErrorCode>;

    /// Erase a page/pages of flash, setting every byte to 0xFF.
    ///
    /// This will erase all pages starting from `address` for the `length`.
    /// `address` and `length must allign with `E`.
    ///
    /// On success returns nothing
    /// On failure returns a `ErrorCode`.
    fn erase(&self, region: &Page) -> Result<(), ErrorCode>;

    /// Create a `Page` from the provided `address` and `length`.
    ///
    /// The `Page->address` and Page->length` will be modified to match
    /// the flash hardware requirements.
    fn get_erase_region(&self, address: u64, length: u32) -> Result<Page, ErrorCode>;
}

/// Implement `Client` to receive callbacks from `Flash`.
pub trait Client<const W: usize, const E: usize> {
    /// Flash read complete.
    ///
    /// This will be called when the read operation is complete.
    /// On success `ret` will be nothing.
    /// On error `ret` will contain a `ErrorCode`
    fn read_complete(&self, read_buffer: &'static mut [u8], ret: Result<(), ErrorCode>);

    /// Flash write complete.
    ///
    /// This will be called when the write operation is complete.
    /// On success `ret` will be nothing.
    /// On error `ret` will contain a `ErrorCode`
    fn write_complete(&self, write_buffer: &'static mut [u8], ret: Result<(), ErrorCode>);

    /// Flash erase complete.
    ///
    /// This will be called when the erase operation is complete.
    /// On success `ret` will be nothing.
    /// On error `ret` will contain a `ErrorCode`
    fn erase_complete(&self, ret: Result<(), ErrorCode>);
}
