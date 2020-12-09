//! Interface for reading, writing, and erasing flash storage pages.
//!
//! Operates on single pages. The page size is set by the associated type
//! `page`. Here is an example of a page type and implementation of this trait:
//!
//! ```rust
//! #![feature(min_const_generics)]
//! use core::ops::{Index, IndexMut};
//! use kernel::hil::flash;
//! use kernel::ReturnCode;
//! use kernel::common::leasable_buffer::LeasableBuffer;
//!
//! // Size in bytes
//! const PAGE_SIZE: u32 = 1024;
//!
//! struct NewChipStruct;
//!
//! impl<'a, C> flash::HasClient<'a, C> for NewChipStruct {
//!     fn set_client(&'a self, client: &'a C) { }
//! }
//!
//! impl<const S: usize> flash::Flash<S> for NewChipStruct {
//!     fn set_bank(&self, bank_number: usize, options: flash::BankOptions) -> Result<(), ReturnCode> {
//!        unimplemented!()
//!     }
//!
//!     fn read_page(
//!         &self,
//!         page_number: usize,
//!         buf: &'static mut [u8; S],
//!     ) -> Result<(), (ReturnCode, &'static mut [u8; S])> {
//!        unimplemented!()
//!     }
//!
//!     fn write(
//!         &self,
//!         address: usize,
//!         buf: LeasableBuffer<'static, u8>,
//!     ) -> Result<(), (ReturnCode, &'static mut [u8])> {
//!         unimplemented!()
//!     }
//!
//!     fn erase_page(&self, page_number: usize) -> Result<(), ReturnCode> {
//!         unimplemented!()
//!     }
//!
//!     fn blocking_read_page(&self, page_number: usize, buf: &mut [u8; S]) -> Result<(), ReturnCode> {
//!         unimplemented!()
//!     }
//!
//!     fn blocking_write(&self, address: usize, buf: &u8) -> Result<(), ReturnCode> {
//!         unimplemented!()
//!     }
//!
//!     fn blocking_erase_page(&self, page_number: usize) -> Result<(), ReturnCode> {
//!         unimplemented!()
//!     }
//! }
//! ```
//!
//! A user of this flash interface might look like:
//!
//! ```rust
//! #![feature(min_const_generics)]
//! use kernel::common::cells::TakeCell;
//! use kernel::hil::flash;
//! use kernel::ReturnCode;
//!
//! pub struct FlashUser<'a, F: flash::Flash<S> + 'static, const S: usize> {
//!     driver: &'a F,
//!     buffer: TakeCell<'static, [u8; S]>,
//! }
//!
//! impl<'a, F: flash::Flash<S>, const S: usize> FlashUser<'a, F, S> {
//!     pub fn new(driver: &'a F, buffer: &'static mut [u8; S]) -> FlashUser<'a, F, S> {
//!         FlashUser {
//!             driver: driver,
//!             buffer: TakeCell::new(buffer),
//!         }
//!     }
//! }
//!
//! impl<'a, F: flash::Flash<S>, const S: usize> flash::Client<S> for FlashUser<'a, F, S> {
//!     fn read_complete(&self, read_buffer: &'static mut [u8], ret: Result<(), ReturnCode>) {}
//!     fn write_complete(&self, write_buffer: &'static mut [u8], ret: Result<(), ReturnCode>) { }
//!     fn erase_complete(&self, ret: Result<(), ReturnCode>) {}
//! }
//! ```

use crate::common::leasable_buffer::LeasableBuffer;
use crate::returncode::ReturnCode;

/// A list of possible rules to access banks
/// This is used to specify if async accesses are allowed on flash banks
pub enum BankOptions {
    AsyncAllowed,
    AsyncReadOnly,
    AsyncDisabled,
}

pub trait HasClient<'a, C> {
    /// Set the client for this flash peripheral. The client will be called
    /// when operations complete.
    fn set_client(&'a self, client: &'a C);
}

/// A page of writeable persistent flash memory.
pub trait Flash<const S: usize> {
    /// Specify the flash bank that the implementation will use.
    ///
    /// This should be called by a board setup to specify the flash bank
    /// to use if there are multiple flash banks.
    /// As part of this call the caller should also specify if async
    /// operations are allowed. This will depend on the hardware, the
    /// bank specified and the bank currently being used.
    ///
    /// On success returns nothing
    /// On failure returns a `ReturnCode`
    fn set_bank(&self, bank_number: usize, options: BankOptions) -> Result<(), ReturnCode>;

    /// Read a page of flash into the buffer.
    ///
    /// This function will read the flash page specified by `page_number`
    /// and store it in the buffer `buf`.
    ///
    /// On success returns nothing
    /// On failure returns a `ReturnCode` and the buffer passed in.
    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut [u8; S],
    ) -> Result<(), (ReturnCode, &'static mut [u8; S])>;

    /// Read an address of flash into the buffer.
    ///
    /// This function will read data stored in flash at `address` and
    /// `length` into the buffer `buf`.
    ///
    /// Not all implementations support this, in that case `read_page`
    /// should be used instead.
    ///
    /// On success returns nothing
    /// On failure returns a `ReturnCode` and the buffer passed in.
    #[allow(unused_variables)]
    fn read_slice(
        &self,
        address: usize,
        length: usize,
        buf: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        /* Default to not supported as not all flash controller allow this */
        Err((ReturnCode::ENOSUPPORT, buf.take()))
    }

    /// Write a page of flash from the buffer.
    ///
    /// This function will write the buffer `buf` to the `address` specified
    /// in flash.
    ///
    /// This function will not erase the page first. The user of this function
    /// must ensure that a page is erased before writing.
    ///
    /// On success returns nothing
    /// On failure returns a `ReturnCode` and the buffer passed in.
    fn write(
        &self,
        address: usize,
        buf: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ReturnCode, &'static mut [u8])>;

    /// Erase a page of flash by setting every byte to 0xFF.
    ///
    /// On success returns nothing
    /// On failure returns a `ReturnCode`
    fn erase_page(&self, page_number: usize) -> Result<(), ReturnCode>;

    /// Blocking page read
    ///
    /// This function will read the flash page specified by `page_number`
    /// and store it in the buffer `buf`.
    /// This is a blocking call and should be used when async operations
    /// aren't allowed by the hardware. This will not call a callback.
    ///
    /// On success returns nothing
    /// On failure returns a `ReturnCode`.
    fn blocking_read_page(&self, page_number: usize, buf: &mut [u8; S]) -> Result<(), ReturnCode>;

    /// Blocking write
    ///
    /// This function will write the buffer `buf` to the `address` specified
    /// in flash.
    ///
    /// This function will not erase the page first. The user of this function
    /// must ensure that a page is erased before writing.
    ///
    /// This is a blocking call and should be used when async operations
    /// aren't allowed by the hardware. This will not call a callback.
    ///
    /// On success returns nothing
    /// On failure returns a `ReturnCode`.
    fn blocking_write(&self, address: usize, buf: &u8) -> Result<(), ReturnCode>;

    /// Blocking page erase
    ///
    /// This is a blocking call and should be used when async operations
    /// aren't allowed by the hardware. This will not call a callback.
    ///
    /// On success returns nothing
    /// On failure returns a `ReturnCode`.
    fn blocking_erase_page(&self, page_number: usize) -> Result<(), ReturnCode>;
}

/// Implement `Client` to receive callbacks from `Flash`.
pub trait Client<const S: usize> {
    /// Flash read complete.
    ///
    /// This will be called when the read operation is complete.
    /// On success `ret` will be nothing.
    /// On error `ret` will contain a `ReturnCode`
    fn read_complete(&self, read_buffer: &'static mut [u8], ret: Result<(), ReturnCode>);

    /// Flash write complete.
    ///
    /// This will be called when the write operation is complete.
    /// On success `ret` will be nothing.
    /// On error `ret` will contain a `ReturnCode`
    fn write_complete(&self, write_buffer: &'static mut [u8], ret: Result<(), ReturnCode>);

    /// Flash erase complete.
    ///
    /// This will be called when the erase operation is complete.
    /// On success `ret` will be nothing.
    /// On error `ret` will contain a `ReturnCode`
    fn erase_complete(&self, ret: Result<(), ReturnCode>);
}
