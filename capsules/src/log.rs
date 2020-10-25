//! Implements a log storage abstraction for storing persistent data in flash.
//!
//! Data entries can be appended to the end of a log and read back in-order. Logs may be linear
//! (denying writes when full) or circular (overwriting the oldest entries with the newest entries
//! when the underlying flash volume is full). The storage volumes that logs operate upon are
//! statically allocated at compile time and cannot be dynamically created at runtime.
//!
//! Entries can be identified and seeked-to with their unique Entry IDs. Entry IDs maintain the
//! ordering of the underlying entries, and an entry with a larger entry ID is newer and comes
//! after an entry with a smaller ID. IDs can also be used to determine the physical position of
//! entries within the log's underlying storage volume - taking the ID modulo the size of the
//! underlying storage volume yields the position of the entry's header relative to the start of
//! the volume. Entry IDs should not be created manually by clients, only retrieved through the
//! `log_start()`, `log_end()`, and `next_read_entry_id()` functions.
//!
//! Entry IDs are not explicitly stored in the log. Instead, each page of the log contains a header
//! containing the page's offset relative to the start of the log (i.e. if the page size is 512
//! bytes, then page #0 will have an offset of 0, page #1 an offset of 512, etc.). The offsets
//! continue to increase even after a circular log wraps around (so if 5 512-byte pages of data are
//! written to a 4 page log, then page #0 will now have an offset of 2048). Thus, the ID of an
//! entry can be calculated by taking the offset of the page within the log and adding the offset
//! of the entry within the page to find the position of the entry within the log (which is the
//! ID). Entries also have a header of their own, which contains the length of the entry.
//!
//! Logs support the following basic operations:
//!
//! * Read:     Read back previously written entries in whole. Entries are read in their entirety
//!             (no partial reads) from oldest to newest.
//! * Seek:     Seek to different entries to begin reading from a different entry (can only seek to
//!             the start of entries).
//! * Append:   Append new data entries onto the end of a log. Can fail if the new entry is too
//!             large to fit within the log.
//! * Sync:     Sync a log to flash to ensure that all changes are persistent.
//! * Erase:    Erase a log in its entirety, clearing the underlying flash volume.
//!
//! See the documentation for each individual function for more detail on how they operate.
//!
//! Note that while logs persist across reboots, they will be erased upon flashing a new kernel.
//!
//! Usage
//! -----
//!
//! ```rust
//! storage_volume!(VOLUME, 2);
//! static mut PAGEBUFFER: sam4l::flashcalw::Sam4lPage = sam4l::flashcalw::Sam4lPage::new();
//!
//! let dynamic_deferred_call_clients =
//!     static_init!([DynamicDeferredCallClientState; 2], Default::default());
//! let dynamic_deferred_caller = static_init!(
//!     DynamicDeferredCall,
//!     DynamicDeferredCall::new(dynamic_deferred_call_clients)
//! );
//!
//! let log = static_init!(
//!     capsules::log::Log,
//!     capsules::log::Log::new(
//!         &VOLUME,
//!         &mut sam4l::flashcalw::FLASH_CONTROLLER,
//!         &mut PAGEBUFFER,
//!         dynamic_deferred_caller,
//!         true
//!     )
//! );
//! kernel::hil::flash::HasClient::set_client(&sam4l::flashcalw::FLASH_CONTROLLER, log);
//! log.initialize_callback_handle(
//!     dynamic_deferred_caller
//!         .register(log)
//!         .expect("no deferred call slot available for log storage")
//! );
//!
//! log.set_read_client(log_storage_read_client);
//! log.set_append_client(log_storage_append_client);
//! ```

use core::cell::Cell;
use core::convert::TryFrom;
use core::mem::size_of;
use core::unreachable;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::hil::flash::{self, Flash};
use kernel::hil::log::{LogRead, LogReadClient, LogWrite, LogWriteClient};
use kernel::ReturnCode;

/// Globally declare entry ID type.
type EntryID = usize;

/// Maximum page header size.
pub const PAGE_HEADER_SIZE: usize = size_of::<EntryID>();
/// Maximum entry header size.
pub const ENTRY_HEADER_SIZE: usize = size_of::<usize>();

/// Byte used to pad the end of a page.
const PAD_BYTE: u8 = 0xFF;

/// Log state keeps track of any in-progress asynchronous operations.
#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Read,
    Seek,
    Append,
    Sync,
    Erase,
}

pub struct Log<'a, F: Flash + 'static> {
    /// Underlying storage volume.
    volume: &'static [u8],
    /// Capacity of log in bytes.
    capacity: usize,
    /// Flash interface.
    driver: &'a F,
    /// Buffer for a flash page.
    pagebuffer: TakeCell<'static, F::Page>,
    /// Size of a flash page.
    page_size: usize,
    /// Whether or not the log is circular.
    circular: bool,
    /// Read client using Log.
    read_client: OptionalCell<&'a dyn LogReadClient>,
    /// Append client using Log.
    append_client: OptionalCell<&'a dyn LogWriteClient>,

    /// Current operation being executed, if asynchronous.
    state: Cell<State>,
    /// Entry ID of oldest entry remaining in log.
    oldest_entry_id: Cell<EntryID>,
    /// Entry ID of next entry to read.
    read_entry_id: Cell<EntryID>,
    /// Entry ID of next entry to append.
    append_entry_id: Cell<EntryID>,

    /// Deferred caller for deferring client callbacks.
    deferred_caller: &'a DynamicDeferredCall,
    /// Handle for deferred caller.
    handle: OptionalCell<DeferredCallHandle>,

    // Note: for saving state across stack ripping.
    /// Client-provided buffer to write from.
    buffer: TakeCell<'static, [u8]>,
    /// Length of data within buffer.
    length: Cell<usize>,
    /// Whether or not records were lost in the previous append.
    records_lost: Cell<bool>,
    /// Error returned by previously executed operation (or SUCCESS).
    error: Cell<ReturnCode>,
}

impl<'a, F: Flash + 'static> Log<'a, F> {
    pub fn new(
        volume: &'static [u8],
        driver: &'a F,
        pagebuffer: &'static mut F::Page,
        deferred_caller: &'a DynamicDeferredCall,
        circular: bool,
    ) -> Log<'a, F> {
        let page_size = pagebuffer.as_mut().len();
        let capacity = volume.len() - PAGE_HEADER_SIZE * (volume.len() / page_size);

        let log: Log<'a, F> = Log {
            volume,
            capacity,
            driver,
            pagebuffer: TakeCell::new(pagebuffer),
            page_size,
            circular,
            read_client: OptionalCell::empty(),
            append_client: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            oldest_entry_id: Cell::new(PAGE_HEADER_SIZE),
            read_entry_id: Cell::new(PAGE_HEADER_SIZE),
            append_entry_id: Cell::new(PAGE_HEADER_SIZE),
            deferred_caller,
            handle: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            length: Cell::new(0),
            records_lost: Cell::new(false),
            error: Cell::new(ReturnCode::ENODEVICE),
        };

        log.reconstruct();
        log
    }

    /// Returns the page number of the page containing the entry with the given ID.
    fn page_number(&self, entry_id: EntryID) -> usize {
        (self.volume.as_ptr() as usize + entry_id % self.volume.len()) / self.page_size
    }

    /// Gets the buffer containing the byte at the given position in the log.
    fn get_buffer<'b>(&self, pos: usize, pagebuffer: &'b mut F::Page) -> &'b [u8] {
        // Subtract 1 from append entry ID to get position of last bit written. This is needed
        // because the pagebuffer always contains the last written bit, but not necessarily the
        // position represented by the append entry ID (i.e. the pagebuffer isn't flushed yet when
        // `append_entry_id % page_size == 0`).
        if pos / self.page_size == (self.append_entry_id.get() - 1) / self.page_size {
            pagebuffer.as_mut()
        } else {
            self.volume
        }
    }

    /// Gets the byte at the given position in the log.
    fn get_byte(&self, pos: usize, pagebuffer: &mut F::Page) -> u8 {
        let buffer = self.get_buffer(pos, pagebuffer);
        buffer[pos % buffer.len()]
    }

    /// Gets a `num_bytes` long slice of bytes starting from a position within the log.
    fn get_bytes<'b>(&self, pos: usize, num_bytes: usize, pagebuffer: &'b mut F::Page) -> &'b [u8] {
        let buffer = self.get_buffer(pos, pagebuffer);
        let offset = pos % buffer.len();
        &buffer[offset..offset + num_bytes]
    }

    /// Resets a log back to an empty log. Returns whether or not the log was reset successfully.
    fn reset(&self) -> bool {
        self.oldest_entry_id.set(PAGE_HEADER_SIZE);
        self.read_entry_id.set(PAGE_HEADER_SIZE);
        self.append_entry_id.set(PAGE_HEADER_SIZE);
        self.pagebuffer.take().map_or(false, move |pagebuffer| {
            for e in pagebuffer.as_mut().iter_mut() {
                *e = 0;
            }
            self.pagebuffer.replace(pagebuffer);
            true
        })
    }

    /// Reconstructs a log from flash.
    fn reconstruct(&self) {
        // Read page headers, get IDs of oldest and newest pages.
        let mut oldest_page_id: EntryID = core::usize::MAX;
        let mut newest_page_id: EntryID = 0;
        for header_pos in (0..self.volume.len()).step_by(self.page_size) {
            let page_id = {
                const ID_SIZE: usize = size_of::<EntryID>();
                let id_bytes = &self.volume[header_pos..header_pos + ID_SIZE];
                let id_bytes = <[u8; ID_SIZE]>::try_from(id_bytes).unwrap();
                usize::from_ne_bytes(id_bytes)
            };

            // Validate page ID read from header.
            if page_id % self.volume.len() == header_pos {
                if page_id < oldest_page_id {
                    oldest_page_id = page_id;
                }
                if page_id > newest_page_id {
                    newest_page_id = page_id;
                }
            }
        }

        // Reconstruct log if at least one valid page was found (meaning oldest page ID was set to
        // something not usize::MAX).
        if oldest_page_id != core::usize::MAX {
            // Walk entries in last (newest) page to calculate last page length.
            let mut last_page_len = PAGE_HEADER_SIZE;
            loop {
                // Check if next byte is start of valid entry.
                let volume_offset = newest_page_id % self.volume.len() + last_page_len;
                if self.volume[volume_offset] == 0 || self.volume[volume_offset] == PAD_BYTE {
                    break;
                }

                // Get next entry length.
                let entry_length = {
                    const LENGTH_SIZE: usize = size_of::<usize>();
                    let length_bytes = &self.volume[volume_offset..volume_offset + LENGTH_SIZE];
                    let length_bytes = <[u8; LENGTH_SIZE]>::try_from(length_bytes).unwrap();
                    usize::from_ne_bytes(length_bytes)
                } + ENTRY_HEADER_SIZE;

                // Add to page length if length is valid (fits within remainder of page.
                if last_page_len + entry_length <= self.page_size {
                    last_page_len += entry_length;
                    if last_page_len == self.page_size {
                        break;
                    }
                } else {
                    break;
                }
            }

            // Set tracked entry IDs.
            self.oldest_entry_id.set(oldest_page_id + PAGE_HEADER_SIZE);
            self.read_entry_id.set(oldest_page_id + PAGE_HEADER_SIZE);
            self.append_entry_id.set(newest_page_id + last_page_len);

            // Populate page buffer.
            self.pagebuffer
                .take()
                .map(move |pagebuffer| {
                    // Determine if pagebuffer should be reset or copied from flash.
                    let mut copy_pagebuffer = last_page_len % self.page_size != 0;
                    if !copy_pagebuffer {
                        // Last page full, reset pagebuffer for next page.
                        copy_pagebuffer = !self.reset_pagebuffer(pagebuffer);
                    }
                    if copy_pagebuffer {
                        // Copy last page into pagebuffer.
                        for i in 0..self.page_size {
                            pagebuffer.as_mut()[i] =
                                self.volume[newest_page_id % self.volume.len() + i];
                        }
                    }
                    self.pagebuffer.replace(pagebuffer);
                })
                .unwrap();
        } else {
            // No valid pages found, create fresh log.
            self.reset();
        }
    }

    /// Returns the ID of the next entry to read or an error if no entry could be retrieved.
    /// ReturnCodes used:
    ///     * FAIL: reached end of log, nothing to read.
    ///     * ERESERVE: client or internal pagebuffer missing.
    fn get_next_entry(&self) -> Result<EntryID, ReturnCode> {
        self.pagebuffer
            .take()
            .map_or(Err(ReturnCode::ERESERVE), move |pagebuffer| {
                let mut entry_id = self.read_entry_id.get();

                // Skip page header if at start of page or skip padded bytes if at end of page.
                if entry_id % self.page_size == 0 {
                    entry_id += PAGE_HEADER_SIZE;
                } else if self.get_byte(entry_id, pagebuffer) == PAD_BYTE {
                    entry_id += self.page_size - entry_id % self.page_size + PAGE_HEADER_SIZE;
                }

                // Check if end of log was reached and return.
                self.pagebuffer.replace(pagebuffer);
                if entry_id >= self.append_entry_id.get() {
                    Err(ReturnCode::FAIL)
                } else {
                    Ok(entry_id)
                }
            })
    }

    /// Reads and returns the contents of an entry header with the given ID. Fails if the header
    /// data is invalid.
    /// ReturnCodes used:
    ///     * FAIL: entry header invalid.
    ///     * ERESERVE: client or internal pagebuffer missing.
    fn read_entry_header(&self, entry_id: EntryID) -> Result<usize, ReturnCode> {
        self.pagebuffer
            .take()
            .map_or(Err(ReturnCode::ERESERVE), move |pagebuffer| {
                // Get length.
                const LENGTH_SIZE: usize = size_of::<usize>();
                let length_bytes = self.get_bytes(entry_id, LENGTH_SIZE, pagebuffer);
                let length_bytes = <[u8; LENGTH_SIZE]>::try_from(length_bytes).unwrap();
                let length = usize::from_ne_bytes(length_bytes);

                // Return length of next entry.
                self.pagebuffer.replace(pagebuffer);
                if length == 0 || length > self.page_size - PAGE_HEADER_SIZE - ENTRY_HEADER_SIZE {
                    Err(ReturnCode::FAIL)
                } else {
                    Ok(length)
                }
            })
    }

    /// Reads the next entry into a buffer. Returns the number of bytes read on success, or an
    /// error otherwise.
    /// ReturnCodes used:
    ///     * FAIL: reached end of log, nothing to read.
    ///     * ERESERVE: internal pagebuffer missing, log is presumably broken.
    ///     * ESIZE: buffer not large enough to contain entry being read.
    fn read_entry(&self, buffer: &mut [u8], length: usize) -> Result<usize, ReturnCode> {
        // Get next entry to read. Immediately returns FAIL in event of failure.
        let entry_id = self.get_next_entry()?;
        let entry_length = self.read_entry_header(entry_id)?;

        // Read entry into buffer.
        self.pagebuffer
            .take()
            .map_or(Err(ReturnCode::ERESERVE), move |pagebuffer| {
                // Ensure buffer is large enough to hold log entry.
                if entry_length > length {
                    self.pagebuffer.replace(pagebuffer);
                    return Err(ReturnCode::ESIZE);
                }
                let entry_id = entry_id + ENTRY_HEADER_SIZE;

                // Copy data into client buffer.
                let data = self.get_bytes(entry_id, entry_length, pagebuffer);
                for i in 0..entry_length {
                    buffer[i] = data[i];
                }

                // Update read entry ID and return number of bytes read.
                self.read_entry_id.set(entry_id + entry_length);
                self.pagebuffer.replace(pagebuffer);
                Ok(entry_length)
            })
    }

    /// Writes an entry header at the given position within a page. Must write at most
    /// ENTRY_HEADER_SIZE bytes.
    fn write_entry_header(&self, length: usize, pos: usize, pagebuffer: &mut F::Page) {
        let mut offset = 0;
        for byte in &length.to_ne_bytes() {
            pagebuffer.as_mut()[pos + offset] = *byte;
            offset += 1;
        }
    }

    /// Appends data from a buffer onto the end of the log. Requires that there is enough space
    /// remaining in the pagebuffer for the entry (including metadata).
    fn append_entry(
        &self,
        buffer: &'static mut [u8],
        length: usize,
        pagebuffer: &'static mut F::Page,
    ) {
        // Offset within page to append to.
        let append_entry_id = self.append_entry_id.get();
        let mut page_offset = append_entry_id % self.page_size;

        // Write entry header to pagebuffer.
        self.write_entry_header(length, page_offset, pagebuffer);
        page_offset += ENTRY_HEADER_SIZE;

        // Copy data to pagebuffer.
        for offset in 0..length {
            pagebuffer.as_mut()[page_offset + offset] = buffer[offset];
        }

        // Increment append offset by number of bytes appended.
        let append_entry_id = append_entry_id + length + ENTRY_HEADER_SIZE;
        self.append_entry_id.set(append_entry_id);

        // Replace pagebuffer and callback client.
        self.pagebuffer.replace(pagebuffer);
        self.buffer.replace(buffer);
        self.records_lost
            .set(self.oldest_entry_id.get() != PAGE_HEADER_SIZE);
        self.error.set(ReturnCode::SUCCESS);
        self.client_callback();
    }

    /// Flushes the pagebuffer to flash. Log state must be non-idle before calling, else data races
    /// may occur due to asynchronous page write.
    /// ReturnCodes used:
    ///     * SUCCESS: flush started successfully.
    ///     * FAIL: flash driver not configured.
    ///     * EBUSY: flash driver busy.
    fn flush_pagebuffer(&self, pagebuffer: &'static mut F::Page) -> ReturnCode {
        // Pad end of page.
        let mut pad_ptr = self.append_entry_id.get();
        while pad_ptr % self.page_size != 0 {
            pagebuffer.as_mut()[pad_ptr % self.page_size] = PAD_BYTE;
            pad_ptr += 1;
        }

        // Get flash page to write to and log page being overwritten. Subtract page_size since
        // padding pointer points to start of the page following the one we want to flush after the
        // padding operation.
        let page_number = self.page_number(pad_ptr - self.page_size);
        let overwritten_page = (pad_ptr - self.volume.len() - self.page_size) / self.page_size;

        // Advance read and oldest entry IDs, if within flash page being overwritten.
        let read_entry_id = self.read_entry_id.get();
        if read_entry_id / self.page_size == overwritten_page {
            // Move read entry ID to start of next page.
            self.read_entry_id.set(
                read_entry_id + self.page_size + PAGE_HEADER_SIZE - read_entry_id % self.page_size,
            );
        }

        let oldest_entry_id = self.oldest_entry_id.get();
        if oldest_entry_id / self.page_size == overwritten_page {
            self.oldest_entry_id.set(oldest_entry_id + self.page_size);
        }

        // Sync page to flash.
        match self.driver.write_page(page_number, pagebuffer) {
            Ok(()) => ReturnCode::SUCCESS,
            Err((return_code, pagebuffer)) => {
                self.pagebuffer.replace(pagebuffer);
                return_code
            }
        }
    }

    /// Resets the pagebuffer so that new data can be written. Note that this also increments the
    /// append entry ID to point to the start of writable data in this new page. Does not reset
    /// pagebuffer or modify append entry ID if the end of a non-circular log is reached. Returns
    /// whether or not the pagebuffer was reset.
    fn reset_pagebuffer(&self, pagebuffer: &mut F::Page) -> bool {
        // Make sure this is not the last page of a non-circular buffer.
        let mut append_entry_id = self.append_entry_id.get();
        if !self.circular && append_entry_id + self.page_size > self.volume.len() {
            return false;
        }

        // Increment append entry ID to point at start of next page.
        if append_entry_id % self.page_size != 0 {
            append_entry_id += self.page_size - append_entry_id % self.page_size;
        }

        // Write page header to pagebuffer.
        let id_bytes = append_entry_id.to_ne_bytes();
        for index in 0..id_bytes.len() {
            pagebuffer.as_mut()[index] = id_bytes[index];
        }

        // Note: this is the only place where the append entry ID can cross page boundaries.
        self.append_entry_id.set(append_entry_id + PAGE_HEADER_SIZE);
        true
    }

    /// Erases a single page from storage.
    fn erase_page(&self) -> ReturnCode {
        // Uses oldest entry ID to keep track of which page to erase. Thus, the oldest pages will be
        // erased first and the log will remain in a valid state even if it fails to be erased
        // completely.
        self.driver
            .erase_page(self.page_number(self.oldest_entry_id.get()))
    }

    /// Initializes a callback handle for deferred callbacks.
    pub fn initialize_callback_handle(&self, handle: DeferredCallHandle) {
        self.handle.replace(handle);
    }

    /// Defers a client callback until later.
    fn deferred_client_callback(&self) {
        self.handle.map(|handle| self.deferred_caller.set(*handle));
    }

    /// Resets the log state to idle and makes a client callback. The values returned by via the
    /// callback must be saved within the log's state before making a callback.
    fn client_callback(&self) {
        let state = self.state.get();
        match state {
            State::Read | State::Seek => {
                self.state.set(State::Idle);
                self.read_client
                    .map(move |read_client| match state {
                        State::Read => self
                            .buffer
                            .take()
                            .map(move |buffer| {
                                read_client.read_done(buffer, self.length.get(), self.error.get());
                            })
                            .unwrap(),
                        State::Seek => read_client.seek_done(self.error.get()),
                        _ => unreachable!(),
                    })
                    .unwrap();
            }
            State::Append | State::Sync | State::Erase => {
                self.state.set(State::Idle);
                self.append_client
                    .map(move |append_client| match state {
                        State::Append => self
                            .buffer
                            .take()
                            .map(move |buffer| {
                                append_client.append_done(
                                    buffer,
                                    self.length.get(),
                                    self.records_lost.get(),
                                    self.error.get(),
                                );
                            })
                            .unwrap(),
                        State::Sync => append_client.sync_done(self.error.get()),
                        State::Erase => append_client.erase_done(self.error.get()),
                        _ => unreachable!(),
                    })
                    .unwrap();
            }
            State::Idle => (),
        }
    }
}

impl<'a, F: Flash + 'static> LogRead<'a> for Log<'a, F> {
    type EntryID = EntryID;

    /// Set the client for read operation callbacks.
    fn set_read_client(&self, read_client: &'a dyn LogReadClient) {
        self.read_client.set(read_client);
    }

    /// Read an entire log entry into a buffer, if there are any remaining. Updates the read entry
    /// ID to point at the next entry when done.
    ///
    /// Returns:
    /// * Ok(()) on success.
    /// * Err((ReturnCode, Option<buffer>)) on failure. The buffer will only be `None` if the error
    ///     is due to a loss of the buffer.
    ///
    /// ReturnCodes used:
    /// * FAIL: reached end of log, nothing to read.
    /// * EBUSY: log busy with another operation, try again later.
    /// * EINVAL: provided client buffer is too small.
    /// * ECANCEL: invalid internal state, read entry ID was reset to start of log.
    /// * ERESERVE: client or internal pagebuffer missing.
    /// * ESIZE: buffer not large enough to contain entry being read.
    ///
    /// ReturnCodes used in read_done callback:
    /// * SUCCESS: read succeeded.
    fn read(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)> {
        // Check for failure cases.
        if self.state.get() != State::Idle {
            // Log busy, try reading again later.
            return Err((ReturnCode::EBUSY, Some(buffer)));
        } else if buffer.len() < length {
            // Client buffer too small for provided length.
            return Err((ReturnCode::EINVAL, Some(buffer)));
        } else if self.read_entry_id.get() > self.append_entry_id.get() {
            // Read entry ID beyond append entry ID, must be invalid.
            self.read_entry_id.set(self.oldest_entry_id.get());
            return Err((ReturnCode::ECANCEL, Some(buffer)));
        } else if self.read_client.is_none() {
            // No client for callback.
            return Err((ReturnCode::ERESERVE, Some(buffer)));
        }

        // Try reading next entry.
        match self.read_entry(buffer, length) {
            Ok(bytes_read) => {
                self.state.set(State::Read);
                self.buffer.replace(buffer);
                self.length.set(bytes_read);
                self.error.set(ReturnCode::SUCCESS);
                self.deferred_client_callback();
                Ok(())
            }
            Err(return_code) => Err((return_code, Some(buffer))),
        }
    }

    /// Returns the ID of the oldest remaining entry in the log.
    fn log_start(&self) -> Self::EntryID {
        self.oldest_entry_id.get()
    }

    /// Returns the ID of the newest entry in the log.
    fn log_end(&self) -> Self::EntryID {
        self.append_entry_id.get()
    }

    /// Returns the ID of the next entry to be read.
    fn next_read_entry_id(&self) -> Self::EntryID {
        self.read_entry_id.get()
    }

    /// Seek to a new read entry ID. It is only legal to seek to entry IDs retrieved through the
    /// `log_start()`, `log_end()`, and `next_read_entry_id()` functions.
    ///
    /// ReturnCodes used:
    /// * SUCCESS: seek succeeded.
    /// * EINVAL: entry ID not valid seek position within current log.
    /// * ERESERVE: no log client set.
    fn seek(&self, entry_id: Self::EntryID) -> ReturnCode {
        if entry_id <= self.append_entry_id.get() && entry_id >= self.oldest_entry_id.get() {
            self.read_entry_id.set(entry_id);

            self.state.set(State::Seek);
            self.error.set(ReturnCode::SUCCESS);
            self.deferred_client_callback();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EINVAL
        }
    }

    /// Get approximate log capacity in bytes.
    fn get_size(&self) -> usize {
        self.capacity
    }
}

impl<'a, F: Flash + 'static> LogWrite<'a> for Log<'a, F> {
    /// Set the client for append operation callbacks.
    fn set_append_client(&self, append_client: &'a dyn LogWriteClient) {
        self.append_client.set(append_client);
    }

    /// Appends an entry onto the end of the log. Entry must fit within a page (including log
    /// metadata).
    ///
    /// Returns:
    /// * Ok(()) on success.
    /// * Err((ReturnCode, Option<buffer>)) on failure. The buffer will only be `None` if the error
    ///     error is due to a loss of the buffer.
    ///
    /// ReturnCodes used:
    /// * FAIL: end of non-circular log reached, cannot append any more entries.
    /// * EBUSY: log busy with another operation, try again later.
    /// * EINVAL: provided client buffer is too small.
    /// * ERESERVE: client or internal pagebuffer missing.
    /// * ESIZE: entry too large to append to log.
    ///
    /// ReturnCodes used in append_done callback:
    /// * SUCCESS: append succeeded.
    /// * FAIL: write failed due to flash error.
    /// * ECANCEL: write failed due to reaching the end of a non-circular log.
    fn append(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)> {
        let entry_size = length + ENTRY_HEADER_SIZE;

        // Check for failure cases.
        if self.state.get() != State::Idle {
            // Log busy, try appending again later.
            return Err((ReturnCode::EBUSY, Some(buffer)));
        } else if length == 0 || buffer.len() < length {
            // Invalid length provided.
            return Err((ReturnCode::EINVAL, Some(buffer)));
        } else if entry_size + PAGE_HEADER_SIZE > self.page_size {
            // Entry too big, won't fit within a single page.
            return Err((ReturnCode::ESIZE, Some(buffer)));
        } else if !self.circular && self.append_entry_id.get() + entry_size > self.volume.len() {
            // End of non-circular log has been reached.
            return Err((ReturnCode::FAIL, Some(buffer)));
        }

        // Perform append.
        match self.pagebuffer.take() {
            Some(pagebuffer) => {
                self.state.set(State::Append);
                self.length.set(length);

                // Check if previous page needs to be flushed and new entry will fit within space
                // remaining in current page.
                let append_entry_id = self.append_entry_id.get();
                let flush_prev_page = append_entry_id % self.page_size == 0;
                let space_remaining = self.page_size - append_entry_id % self.page_size;
                if !flush_prev_page && entry_size <= space_remaining {
                    // Entry fits, append it.
                    self.append_entry(buffer, length, pagebuffer);
                    Ok(())
                } else {
                    // Need to sync pagebuffer first, then append to new page.
                    self.buffer.replace(buffer);
                    let return_code = self.flush_pagebuffer(pagebuffer);
                    if return_code == ReturnCode::SUCCESS {
                        Ok(())
                    } else {
                        self.state.set(State::Idle);
                        self.buffer
                            .take()
                            .map_or(Err((return_code, None)), move |buffer| {
                                Err((return_code, Some(buffer)))
                            })
                    }
                }
            }
            None => Err((ReturnCode::ERESERVE, Some(buffer))),
        }
    }

    /// Sync log to storage.
    /// ReturnCodes used:
    ///     * SUCCESS: flush started successfully.
    ///     * FAIL: flash driver not configured.
    ///     * EBUSY: log or flash driver busy, try again later.
    ///     * ERESERVE: no log client set.
    /// ReturnCodes used in sync_done callback:
    ///     * SUCCESS: append succeeded.
    ///     * FAIL: write failed due to flash error.
    fn sync(&self) -> ReturnCode {
        if self.append_entry_id.get() % self.page_size == PAGE_HEADER_SIZE {
            // Pagebuffer empty, don't need to flush.
            return ReturnCode::SUCCESS;
        } else if self.state.get() != State::Idle {
            // Log busy, try appending again later.
            return ReturnCode::EBUSY;
        }

        self.pagebuffer
            .take()
            .map_or(ReturnCode::ERESERVE, move |pagebuffer| {
                self.state.set(State::Sync);
                let return_code = self.flush_pagebuffer(pagebuffer);
                if return_code != ReturnCode::SUCCESS {
                    self.state.set(State::Idle);
                }
                return_code
            })
    }

    /// Erase the entire log.
    /// ReturnCodes used:
    ///     * SUCCESS: flush started successfully.
    ///     * EBUSY: log busy, try again later.
    /// ReturnCodes used in erase_done callback:
    ///     * SUCCESS: erase succeeded.
    ///     * EBUSY: erase interrupted by busy flash driver. Call erase again to resume.
    fn erase(&self) -> ReturnCode {
        if self.state.get() != State::Idle {
            // Log busy, try appending again later.
            return ReturnCode::EBUSY;
        }

        self.state.set(State::Erase);
        self.erase_page()
    }
}

impl<'a, F: Flash + 'static> flash::Client<F> for Log<'a, F> {
    fn read_complete(&self, _read_buffer: &'static mut F::Page, _error: flash::Error) {
        // Reads are made directly from the storage volume, not through the flash interface.
        unreachable!();
    }

    /// If in the middle of a write operation, reset pagebuffer and finish write. If syncing, make
    /// successful client callback.
    fn write_complete(&self, pagebuffer: &'static mut F::Page, error: flash::Error) {
        match error {
            flash::Error::CommandComplete => {
                match self.state.get() {
                    State::Append => {
                        // Reset pagebuffer and finish writing on the new page.
                        if self.reset_pagebuffer(pagebuffer) {
                            self.buffer
                                .take()
                                .map(move |buffer| {
                                    self.append_entry(buffer, self.length.get(), pagebuffer);
                                })
                                .unwrap();
                        } else {
                            self.pagebuffer.replace(pagebuffer);
                            self.length.set(0);
                            self.records_lost.set(false);
                            self.error.set(ReturnCode::ECANCEL);
                            self.client_callback();
                        }
                    }
                    State::Sync => {
                        // Reset pagebuffer if synced page was full.
                        if self.append_entry_id.get() % self.page_size == 0 {
                            self.reset_pagebuffer(pagebuffer);
                        }

                        self.pagebuffer.replace(pagebuffer);
                        self.error.set(ReturnCode::SUCCESS);
                        self.client_callback();
                    }
                    _ => unreachable!(),
                }
            }
            flash::Error::FlashError => {
                // Make client callback with FAIL return code.
                self.pagebuffer.replace(pagebuffer);
                match self.state.get() {
                    State::Append => {
                        self.length.set(0);
                        self.records_lost.set(false);
                        self.error.set(ReturnCode::FAIL);
                        self.client_callback();
                    }
                    State::Sync => {
                        self.error.set(ReturnCode::FAIL);
                        self.client_callback();
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Erase next page if log erase complete, else make client callback. Fails with EBUSY if flash
    /// is busy and erase cannot be completed.
    fn erase_complete(&self, error: flash::Error) {
        match error {
            flash::Error::CommandComplete => {
                let oldest_entry_id = self.oldest_entry_id.get();
                if oldest_entry_id >= self.append_entry_id.get() - self.page_size {
                    // Erased all pages. Reset state and callback client.
                    if self.reset() {
                        self.error.set(ReturnCode::SUCCESS);
                    } else {
                        self.error.set(ReturnCode::ERESERVE);
                    }
                    self.client_callback();
                } else {
                    // Not done, erase next page.
                    self.oldest_entry_id.set(oldest_entry_id + self.page_size);
                    let status = self.erase_page();

                    // Abort and alert client if flash driver is busy.
                    if status == ReturnCode::EBUSY {
                        self.read_entry_id
                            .set(core::cmp::max(self.read_entry_id.get(), oldest_entry_id));
                        self.error.set(ReturnCode::EBUSY);
                        self.client_callback();
                    }
                }
            }
            flash::Error::FlashError => {
                self.error.set(ReturnCode::FAIL);
                self.client_callback();
            }
        }
    }
}

impl<'a, F: Flash + 'static> DynamicDeferredCallClient for Log<'a, F> {
    fn call(&self, _handle: DeferredCallHandle) {
        self.client_callback();
    }
}
