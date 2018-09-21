//! Map arbitrary nonvolatile reads and writes to page operations.
//!
//! This splits non-page-aligned reads and writes into a series of page level
//! reads and writes. While it is handling a read or write it returns `EBUSY` to
//! all additional requests.
//!
//! This module is designed to be used on top of any flash storage and below any
//! user of `NonvolatileStorage`. This module handles different sized pages.
//!
//! ```plain
//! hil::nonvolatile_storage::NonvolatileStorage
//!                ┌─────────────┐
//!                │             │
//!                │ This module │
//!                │             │
//!                └─────────────┘
//!               hil::flash::Flash
//! ```
//!
//! Usage
//! -----
//!
//! ```
//! sam4l::flashcalw::FLASH_CONTROLLER.configure();
//! pub static mut PAGEBUFFER: sam4l::flashcalw::Sam4lPage = sam4l::flashcalw::Sam4lPage::new();
//! let nv_to_page = static_init!(
//!     capsules::nonvolatile_to_pages::NonvolatileToPages<'static, sam4l::flashcalw::FLASHCALW>,
//!     capsules::nonvolatile_to_pages::NonvolatileToPages::new(
//!         &mut sam4l::flashcalw::FLASH_CONTROLLER,
//!         &mut PAGEBUFFER));
//! hil::flash::HasClient::set_client(&sam4l::flashcalw::FLASH_CONTROLLER, nv_to_page);
//! ```

use core::cell::Cell;
use core::cmp;
use kernel::common::cells::NumericCellExt;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::ReturnCode;

/// This module is either waiting to do something, or handling a read/write.
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Idle,
    Read,
    Write,
}

pub struct NonvolatileToPages<'a, F: hil::flash::Flash + 'static> {
    /// The module providing a `Flash` interface.
    driver: &'a F,
    /// Callback to the user of this capsule.
    client: OptionalCell<&'static hil::nonvolatile_storage::NonvolatileStorageClient>,
    /// Buffer correctly sized for the underlying flash page size.
    pagebuffer: TakeCell<'static, F::Page>,
    /// Current state of this capsule.
    state: Cell<State>,
    /// Temporary holding place for the user's buffer.
    buffer: TakeCell<'static, [u8]>,
    /// Absolute address of where we are reading or writing. This gets updated
    /// as the operation proceeds across pages.
    address: Cell<usize>,
    /// Total length to read or write. We need to store this to return it to the
    /// client.
    length: Cell<usize>,
    /// How many bytes are left to read or write.
    remaining_length: Cell<usize>,
    /// Where we are in the user buffer.
    buffer_index: Cell<usize>,
}

impl<F: hil::flash::Flash> NonvolatileToPages<'a, F> {
    pub fn new(driver: &'a F, buffer: &'static mut F::Page) -> NonvolatileToPages<'a, F> {
        NonvolatileToPages {
            driver: driver,
            client: OptionalCell::empty(),
            pagebuffer: TakeCell::new(buffer),
            state: Cell::new(State::Idle),
            buffer: TakeCell::empty(),
            address: Cell::new(0),
            length: Cell::new(0),
            remaining_length: Cell::new(0),
            buffer_index: Cell::new(0),
        }
    }
}

impl<F: hil::flash::Flash> hil::nonvolatile_storage::NonvolatileStorage
    for NonvolatileToPages<'a, F> {
    fn set_client(&self, client: &'static hil::nonvolatile_storage::NonvolatileStorageClient) {
        self.client.set(client);
    }

    fn read(&self, buffer: &'static mut [u8], address: usize, length: usize) -> ReturnCode {
        if self.state.get() != State::Idle {
            return ReturnCode::EBUSY;
        }

        self.pagebuffer.take().map_or(
            ReturnCode::ERESERVE,
            move |pagebuffer| {
                let page_size = pagebuffer.as_mut().len();

                // Just start reading. We'll worry about how much of the page we
                // want later.
                self.state.set(State::Read);
                self.buffer.replace(buffer);
                self.address.set(address);
                self.length.set(length);
                self.remaining_length.set(length);
                self.buffer_index.set(0);
                self.driver.read_page(address / page_size, pagebuffer)
            },
        )
    }

    fn write(&self, buffer: &'static mut [u8], address: usize, length: usize) -> ReturnCode {
        if self.state.get() != State::Idle {
            return ReturnCode::EBUSY;
        }

        self.pagebuffer.take().map_or(
            ReturnCode::ERESERVE,
            move |pagebuffer| {
                let page_size = pagebuffer.as_mut().len();

                self.state.set(State::Write);
                self.length.set(length);

                if address % page_size == 0 && length >= page_size {
                    // This write is aligned to a page and we are writing an entire
                    // page or more.

                    // Copy data into page buffer.
                    for i in 0..page_size {
                        pagebuffer.as_mut()[i] = buffer[i];
                    }

                    self.buffer.replace(buffer);
                    self.address.set(address + page_size);
                    self.remaining_length.set(length - page_size);
                    self.buffer_index.set(page_size);
                    self.driver.write_page(address / page_size, pagebuffer)
                } else {
                    // Need to do a read first.
                    self.buffer.replace(buffer);
                    self.address.set(address);
                    self.remaining_length.set(length);
                    self.buffer_index.set(0);
                    self.driver.read_page(address / page_size, pagebuffer)
                }
            },
        )
    }
}

impl<F: hil::flash::Flash> hil::flash::Client<F> for NonvolatileToPages<'a, F> {
    fn read_complete(&self, pagebuffer: &'static mut F::Page, _error: hil::flash::Error) {
        match self.state.get() {
            State::Read => {
                // OK we got a page from flash. Copy what we actually want from it
                // out of it.
                self.buffer.take().map(move |buffer| {
                    let page_size = pagebuffer.as_mut().len();
                    // This will get us our offset into the page.
                    let page_index = self.address.get() % page_size;
                    // Length is either the rest of the page or how much we have left.
                    let len = cmp::min(page_size - page_index, self.remaining_length.get());
                    // And where we left off in the user buffer.
                    let buffer_index = self.buffer_index.get();

                    // Copy what we read from the page buffer to the user buffer.
                    for i in 0..len {
                        buffer[buffer_index + i] = pagebuffer.as_mut()[page_index + i];
                    }

                    // Decide if we are done.
                    let new_len = self.remaining_length.get() - len;
                    if new_len == 0 {
                        // Nothing more to do. Put things back and issue callback.
                        self.pagebuffer.replace(pagebuffer);
                        self.state.set(State::Idle);
                        self.client.map(move |client| {
                            client.read_done(buffer, self.length.get())
                        });
                    } else {
                        // More to do!
                        self.buffer.replace(buffer);
                        // Increment all buffer pointers and state.
                        self.remaining_length.subtract(len);
                        self.address.add(len);
                        self.buffer_index.set(buffer_index + len);
                        self.driver.read_page(
                            self.address.get() / page_size,
                            pagebuffer,
                        );
                    }
                });
            }
            State::Write => {
                // We did a read because we're not page aligned on either or
                // both ends.
                self.buffer.take().map(move |buffer| {
                    let page_size = pagebuffer.as_mut().len();
                    // This will get us our offset into the page.
                    let page_index = self.address.get() % page_size;
                    // Length is either the rest of the page or how much we have left.
                    let len = cmp::min(page_size - page_index, self.remaining_length.get());
                    // And where we left off in the user buffer.
                    let buffer_index = self.buffer_index.get();
                    // Which page we read and which we are going to write back to.
                    let page_number = self.address.get() / page_size;

                    // Copy what we read from the page buffer to the user buffer.
                    for i in 0..len {
                        pagebuffer.as_mut()[page_index + i] = buffer[buffer_index + i];
                    }

                    // Do the write.
                    self.buffer.replace(buffer);
                    self.remaining_length.subtract(len);
                    self.address.add(len);
                    self.buffer_index.set(buffer_index + len);
                    self.driver.write_page(page_number, pagebuffer);
                });
            }
            _ => {}
        }
    }

    fn write_complete(&self, pagebuffer: &'static mut F::Page, _error: hil::flash::Error) {
        // After a write we could be done, need to do another write, or need to
        // do a read.
        self.buffer.take().map(move |buffer| {
            let page_size = pagebuffer.as_mut().len();

            if self.remaining_length.get() == 0 {
                // Done!
                self.pagebuffer.replace(pagebuffer);
                self.state.set(State::Idle);
                self.client.map(move |client| {
                    client.write_done(buffer, self.length.get())
                });
            } else if self.remaining_length.get() >= page_size {
                // Write an entire page!
                let buffer_index = self.buffer_index.get();
                let page_number = self.address.get() / page_size;

                // Copy data into page buffer.
                for i in 0..page_size {
                    pagebuffer.as_mut()[i] = buffer[buffer_index + i];
                }

                self.buffer.replace(buffer);
                self.remaining_length.subtract(page_size);
                self.address.add(page_size);
                self.buffer_index.set(buffer_index + page_size);
                self.driver.write_page(page_number, pagebuffer);
            } else {
                // Write a partial page!
                self.buffer.replace(buffer);
                self.driver.read_page(
                    self.address.get() / page_size,
                    pagebuffer,
                );
            }
        });
    }

    fn erase_complete(&self, _error: hil::flash::Error) {}
}
