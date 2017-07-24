//! Interfaces for reading, writing, and erasing flash storage pages as
//! well as querying the layout and locking/unlocking lock units. A
//! lock unit consists of one or more pages.
//!
//! Read, write and erase operate on single pages. The page size is
//! set by the associated type `page`. Here is an example of a page
//! type:
//!
//! ```rust
//! // Size in bytes
//! const PAGE_SIZE: u32 = 1024;
//!
//! pub struct NewChipPage(pub [u8; PAGE_SIZE as usize]);
//!
//! impl NewChipPage {
//!     pub const fn new() -> NewChipPage {
//!         NewChipPage([0; PAGE_SIZE as usize])
//!     }
//!
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
//! ```
//!
//! Then a basic implementation of this trait should look like:
//!
//! ```rust
//!
//! impl hil::flash::HasClient for NewChipStruct {
//!     fn set_client(&'a self, client: &'a C) { }
//! }
//!
//! impl hil::flash::Flash for NewChipStruct {
//!     type Page = NewChipPage;
//!
//!     fn read_page(&self, page_number: usize, buf: &'static mut Self::Page) -> ReturnCode { }
//!     fn write_page(&self, page_number: usize, buf: &'static mut Self::Page) -> ReturnCode { }
//!     fn erase_page(&self, page_number: usize) -> ReturnCode { }
//! }
//! ```
//!
//! A user of this flash interface might look like:
//!
//! ```rust
//! pub struct FlashUser<'a, F: hil::flash::Flash + 'static> {
//!     driver: &'a F,
//!     buffer: TakeCell<'static, F::Page>,
//! }
//!
//! impl<'a, F: hil::flash::Flash + 'a> FlashUser<'a, F> {
//!     pub fn new(driver: &'a F, buffer: &'static mut F::Page) -> FlashUser<'a, F> {
//!         FlashUser {
//!             driver: driver,
//!             buffer: TakeCell::new(buffer),
//!         }
//!     }
//! }
//!
//! impl<'a, F: hil::flash::Flash + 'a> hil::flash::Client<F> for FlashUser<'a, F> {
//!     fn read_complete(&self, buffer: &'static mut F::Page, error: hil::flash::Error) {}
//!     fn write_complete(&self, buffer: &'static mut F::Page, error: hil::flash::Error) { }
//!     fn erase_complete(&self, error: hil::flash::Error) {}
//! }
//! ```
//!
//! In addition to reading, writing, and erasing pages, most flash chips
//! support locking the chip against writes. This is important, for example,
//! if you want to make sure that a buggy capsule doesn't accidentally
//! overwrite the kernel.
//!
//! Locking is in terms of "units", which can be as small as a single
//! page but are typically much larger (e.g., 6-16 units for the entire
//! flash). If the kernel locks some pages (and does not unlock them), then
//! those pages cannot be written, even by a JTAG programmer; they must
//! be unlocked either by software or by special JTAG commands.

use returncode::ReturnCode;

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
pub trait Flash {
    /// Type of a single flash page for the given implementation.
    type Page: AsMut<[u8]>;

    /// Read a page of flash into the buffer.
    fn read_page(&self, page_number: usize, buf: &'static mut Self::Page) -> ReturnCode;

    /// Write a page of flash from the buffer.
    fn write_page(&self, page_number: usize, buf: &'static mut Self::Page) -> ReturnCode;

    /// Erase a page of flash.
    fn erase_page(&self, page_number: usize) -> ReturnCode;
}

/// Implement `Client` to receive callbacks from `Flash`.
pub trait Client<F: Flash> {
    /// Flash read complete.
    fn read_complete(&self, read_buffer: &'static mut F::Page, error: Error);

    /// Flash write complete.
    fn write_complete(&self, write_buffer: &'static mut F::Page, error: Error);

    /// Flash erase complete.
    fn erase_complete(&self, error: Error);
}

pub trait FlashInfo {
    /// Return the number of pages (the unit of read/write/erase)
    fn num_pages(&self) -> u32;
    /// Return the size of each page in bytes
    fn page_size(&self) -> u32;
    /// Return the size of the flash in bytes
    fn flash_size(&self) -> u32 {
        self.page_size() * self.num_pages()
    }


    /// Returns the number of lock units in the flash, 0
    /// if the flash does not support locking.
    fn num_lock_units(&self) -> u32;

    /// Returns the size of a lock unit in bytes, 0 if
    /// the flash does not support locking. Note that
    /// some flashes support only locking a subset of
    /// the flash, so num_lock_units * lock_unit_size
    /// may not be equal to flash_size.
    fn lock_unit_size(&self) -> u32;

    /// Returns the number of pages in a lock unit, 0
    /// 0 if flash does not support locking.
    fn pages_per_lock_unit(&self) -> u32;
    /// Simple helper function that returns the lock unit for
    /// a particular page number: page / lock_unit_size, returns
    /// 0xffffffff if locking is not supported for that page. Note that
    /// since some flashes support locking of only some of their
    /// pages, the return value of this function is not necessarily
    /// (page * page_size) / lock_unit_size
    fn page_to_lock_unit(&self, page: u32) -> u32;
}

pub trait FlashLocking {
    /// Lock a particular lock unit.
    fn lock_unit(&self, unit: u32);
    /// Unlock a particular lock unit.
    fn unlock_unit(&self, unit: u32);
    /// Locks [first,last] units (inclusive)
    fn lock_units(&self, first: u32, last: u32);
    /// Unlocks [first,last] units (inclusive)
    fn unlock_units(&self, first: u32, last: u32);

}

pub trait FlashLayout {
    /// Return the address at which the kernel image starts.
    fn kernel_start_address(&self) -> u32;
    /// Return the address at which the kernel image ends.
    fn kernel_end_address(&self) -> u32;
    /// Return the page number of the first page of the kernel image.
    fn kernel_first_page(&self) -> u32;
    /// Return the page number of the last page of the kernel image.
    fn kernel_last_page(&self) -> u32;

    /// Return the start address of the block of flash memory reserved for
    /// application images.
    fn apps_start_address(&self) -> u32;
    /// Return the last address of the block of flash memory reserved for
    /// application images.
    fn apps_end_address(&self) -> u32;
    /// Return the page number of the first page of the application region.
    fn apps_first_page(&self) -> u32;
    /// Return the page number of the last page of the application region.
    fn apps_last_page(&self) -> u32;
}
