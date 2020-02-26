//! Interface for a persistent log that stores distinct log entries.
//!
//! Log entries are appended to the end of a log and read back sequentially. Logs should be
//! designed in such a way that they persist across device reboots.

use crate::returncode::ReturnCode;

/// Cookies represent seekable positions within a storage interface. `SeekBeginning` allows a
/// client to seek to the very beginning of the interface. `Cookie` allows the client to seek to a
/// particular position within the interface. How a `Cookie` internally represents a position is up
/// to the implementer. `Cookie`s should not be manually created by the client, and should only be
/// retrieved through the `LogRead::current_read_cookie()` and `LogWrite::current_append_cookie()`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LogCookie {
    SeekBeginning,
    Cookie(usize),
}

/// An interface for reading from log storage.
pub trait LogRead<'a> {
    /// Set the client for reading from a log. The client will be called when reading operations complete.
    fn set_read_client(&'a self, read_client: &'a dyn LogReadClient);

    /// Read the next entry from the log. The log advances to the next entry after a successful
    /// read.
    fn read(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)>;

    /// Get the cookie representing the next entry to be read.
    fn current_read_cookie(&self) -> LogCookie;

    /// Seek to a new entry to read. The `LogCookie` for the entry should either be
    /// `LogCookie::SeekBeginning` or a cookie retrieved through
    /// `LogRead::current_read_cookie()` or `LogRead::current_append_cookie()`.
    fn seek(&self, entry: LogCookie) -> ReturnCode;

    /// Get approximate log capacity in bytes.
    fn get_size(&self) -> usize;
}

/// Receive callbacks from `LogRead`.
pub trait LogReadClient {
    fn read_done(&self, buffer: &'static mut [u8], length: usize, error: ReturnCode);

    fn seek_done(&self, error: ReturnCode);
}

/// An interface for writing to log storage.
pub trait LogWrite<'a> {
    /// Set the client for appending from a log. The client will be called when writing operations complete.
    fn set_append_client(&'a self, append_client: &'a dyn LogWriteClient);

    /// Append an entry to the end of the log.
    fn append(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)>;

    /// Get the cookie representing the next entry that will be written.
    fn current_append_cookie(&self) -> LogCookie;

    /// Sync log to storage, making all entries persistent (not including any entries that were
    /// previously overwritten). There is no guarantee that any changes to the log are persistent
    /// until it is synced.
    fn sync(&self) -> ReturnCode;

    /// Erase the entire log.
    fn erase(&self) -> ReturnCode;
}

/// Receive callbacks from `LogWrite`.
pub trait LogWriteClient {
    fn append_done(
        &self,
        buffer: &'static mut [u8],
        length: usize,
        records_lost: bool,
        error: ReturnCode,
    );

    fn sync_done(&self, error: ReturnCode);

    fn erase_done(&self, error: ReturnCode);
}
