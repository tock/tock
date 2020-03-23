//! Interface for a persistent log that stores distinct log entries.
//!
//! Log entries are appended to the end of a log and read back sequentially. Log data persists
//! across device reboots.

use crate::returncode::ReturnCode;

/// An interface for reading from log storage.
pub trait LogRead<'a> {
    /// Unique identifier for log entries.
    type EntryID;

    /// Set the client for reading from a log. The client will be called when reading operations complete.
    fn set_read_client(&'a self, read_client: &'a dyn LogReadClient);

    /// Read the next entry from the log. The log advances to the next entry after a successful
    /// read. State does not change in the event of a failure.
    fn read(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)>;

    /// Returns the entry ID at the start of the log. This is the ID of the oldest remaining entry.
    fn log_start(&self) -> Self::EntryID;

    /// Returns the entry ID at the end of the log. This is the ID of the next entry to be
    /// appended.
    fn log_end(&self) -> Self::EntryID;

    /// Returns the ID of the next entry to be read.
    fn next_read_entry_id(&self) -> Self::EntryID;

    /// Seek to the entry with the given entry ID and begin reading from there. Fails without
    /// modifying the read position if the given entry ID is invalid or no longer in the log.
    fn seek(&self, entry: Self::EntryID) -> ReturnCode;

    /// Get approximate log capacity in bytes.
    fn get_size(&self) -> usize;
}

/// Receive callbacks from `LogRead`.
pub trait LogReadClient {
    /// Returns a buffer containing data read and the length of the number of bytes read or an error
    /// code if the read failed.
    fn read_done(&self, buffer: &'static mut [u8], length: usize, error: ReturnCode);

    /// Returns whether the seek succeeded or failed.
    fn seek_done(&self, error: ReturnCode);
}

/// An interface for writing to log storage.
pub trait LogWrite<'a> {
    /// Set the client for appending from a log. The client will be called when writing operations complete.
    fn set_append_client(&'a self, append_client: &'a dyn LogWriteClient);

    /// Append an entry to the end of the log. May fail if the entry is too large.
    fn append(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)>;

    /// Sync log to storage, making all entries persistent (not including any entries that were
    /// previously overwritten). There is no guarantee that any changes to the log are persistent
    /// until it is synced. In the event of an error, not all pages may be synced, but the log will
    /// remain in a valid state.
    fn sync(&self) -> ReturnCode;

    /// Erase the entire log. In the event of a failure, only some pages may be erased, but the log
    /// will remain in a valid state.
    fn erase(&self) -> ReturnCode;
}

/// Receive callbacks from `LogWrite`.
pub trait LogWriteClient {
    /// Returns the original buffer that contained the data to write, the number of bytes written,
    /// and whether any old entries in the log were lost (due to a circular log being filled up).
    fn append_done(
        &self,
        buffer: &'static mut [u8],
        length: usize,
        records_lost: bool,
        error: ReturnCode,
    );

    /// Returns whether or not all pages were correctly synced, making all changes persistent.
    fn sync_done(&self, error: ReturnCode);

    /// Returns whether or not all pages of the log were erased.
    fn erase_done(&self, error: ReturnCode);
}
