// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for reading, writing, and erasing flash storage pages.
//!
//! Operates on single pages. The page size is set by the associated type
//! [`Flash::Page`]. For flash modules where the write and erase page size
//! differs, this type models an erase page. An erase page is always assumed to
//! be larger or equal to a write-page size.
//!
//! Generally, flash must be erased before it is written. Users of the `Flash`
//! trait may write a given page at most once between two erases of its page.
//! Writing a page does it erase it implicitly, and users must instead call
//! [`Flash::erase_page`] before rewriting a page that is not in an erased
//! state. An exception to this are `Flash` implementations that implement
//! certain extension traits, like [`BitClearable`], which can---under certain
//! circumstances---allow a flash page to be written multiple times between two
//! erase cycles..
//!
//! Users can use [`Flash::is_page_erased`] to determine whether a given flash
//! page is in a state that permits writing without a prior erase operation.
//! This implementation is conservative: when `is_page_erased` returns `true`, a
//! page is guaranteed to be in its erased state and write to that flash page
//! can be attempted. However, if `is_page_erased` returns `false`, a page may
//! or may not be in an erased state (meaning that a legal implementation of
//! this method is allowed to always return `false`, and `false` does not mean
//! that a write to this page is necessarily forbidden). In this case, users of
//! the Flash HIL must track whether they've erased a given page before they
//! attempt a write to it themselves. If the method returns `false`, users must
//! perform an erase before the next write, and can then perform a write
//! irrespective of this function's return value.
//!
//! Here is an example of a page type and implementation of this trait:
//!
//! ```rust
//! use core::ops::{Index, IndexMut};
//!
//! use kernel::hil;
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
//! impl Index<usize> for NewChipPage {
//!     type Output = u8;
//!
//!     fn index(&self, idx: usize) -> &u8 {
//!         &self.0[idx]
//!     }
//! }
//!
//! impl IndexMut<usize> for NewChipPage {
//!     fn index_mut(&mut self, idx: usize) -> &mut u8 {
//!         &mut self.0[idx]
//!     }
//! }
//!
//! impl AsMut<[u8]> for NewChipPage {
//!     fn as_mut(&mut self) -> &mut [u8] {
//!         &mut self.0
//!     }
//! }
//!
//! struct NewChipStruct {};
//!
//! impl<'a, C> hil::flash::HasClient<'a, C> for NewChipStruct {
//!     fn set_client(&'a self, client: &'a C) { }
//! }
//!
//! impl hil::flash::Flash for NewChipStruct {
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
//!
//! pub struct FlashUser<'a, F: hil::flash::Flash + 'static> {
//!     driver: &'a F,
//!     buffer: TakeCell<'static, F::Page>,
//! }
//!
//! impl<'a, F: hil::flash::Flash> FlashUser<'a, F> {
//!     pub fn new(driver: &'a F, buffer: &'static mut F::Page) -> FlashUser<'a, F> {
//!         FlashUser {
//!             driver: driver,
//!             buffer: TakeCell::new(buffer),
//!         }
//!     }
//! }
//!
//! impl<'a, F: hil::flash::Flash> hil::flash::Client<F> for FlashUser<'a, F> {
//!     fn read_complete(&self, buffer: &'static mut F::Page, result: Result<(), hil::flash::Error>) {}
//!     fn write_complete(&self, buffer: &'static mut F::Page, result: Result<(), hil::flash::Error>) { }
//!     fn erase_complete(&self, result: Result<(), hil::flash::Error>) {}
//! }
//! ```

use crate::ErrorCode;

/// Flash errors returned in the callbacks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// An error occurred during the flash operation.
    FlashError,

    /// A flash memory protection violation was detected.
    FlashMemoryProtectionError,
}

pub trait HasClient<'a, C> {
    /// Set the client for this flash peripheral. The client will be called
    /// when operations complete.
    fn set_client(&'a self, client: &'a C);
}

/// A page of writable persistent flash memory.
pub trait Flash {
    /// Type of a single flash page for the given implementation.
    type Page: AsMut<[u8]> + Default;

    /// Read a page of flash into the buffer.
    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)>;

    /// Write a page of flash from the buffer.
    ///
    /// The page must be erased before writing to it with this function.
    /// Implementations are not required to check that the page is erased. If
    /// the hardware detects a write to a location that is not erased, the
    /// implementation reports [`Error::FlashError`] in
    /// [`Client::write_complete`]. However, the resulting flash state after a
    /// write on a non-erased flash page is undefined.
    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)>;

    /// Erase a page of flash by setting every byte to 0xFF.
    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode>;

    /// Check if page `page_number` is guaranteed to be erased.
    ///
    /// A page for which this returns `true` can be written without erasing it
    /// first. Implementations return `false` when they cannot guarantee this.
    /// When this implementation returns `false`, users of the HIL themselves
    /// are responsible that they've performed at least one prior erase of this
    /// page, and that no write to the page has been performed since the erase.
    // TODO: remove this default implementation once all implementations
    // provide this method.
    fn is_page_erased(&self, _page_number: usize) -> bool {
        false
    }
}

/// Extension trait marker for flash implementations that can clear bits of a
/// page without erasing it.
///
/// A page of such flash may be written several times between two erases, as
/// long as each write following the last erase never attempts to reset a `0`
/// bit to a `1` bit. After the write completes, the page holds the contents of
/// the buffer.
///
/// Implementations must take care of any device-specific restrictions on these
/// writes, such as not reprogramming bits that are already zero (e.g., as
/// required on the Apollo3 chip). Flash that limits how often a location may be
/// written between erases (e.g., the nRF52 series) cannot implement this trait.
pub trait BitClearable: Flash {}

/// Implement `Client` to receive callbacks from `Flash`.
pub trait Client<F: Flash> {
    /// Flash read complete.
    fn read_complete(&self, read_buffer: &'static mut F::Page, result: Result<(), Error>);

    /// Flash write complete.
    fn write_complete(&self, write_buffer: &'static mut F::Page, result: Result<(), Error>);

    /// Flash erase complete.
    fn erase_complete(&self, result: Result<(), Error>);
}
