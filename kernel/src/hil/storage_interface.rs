//! Interfaces for storage devices.

use crate::returncode::ReturnCode;

/// Cookies represent seekable positions within a storage interface. `SeekBeginning` allows a
/// client to seek to the very beginning of the interface. `Cookie` allows the client to seek to a
/// particular position within the interface. How a `Cookie` internally represents a position is up
/// to the implementer.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StorageCookie {
    SeekBeginning,
    Cookie(usize),
}

/// An interface for reading from log storage.
pub trait LogRead<'a> {
    /// Set the client for reading from a log. The client will be called when operations complete.
    fn set_read_client(&'a self, read_client: &'a dyn LogReadClient);

    /// Read log data starting from the current read cookie.
    fn read(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)>;

    /// Get cookie representing current read cookie.
    fn current_read_cookie(&self) -> StorageCookie;

    /// Seek to a new read position.
    fn seek(&self, offset: StorageCookie) -> ReturnCode;

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
    /// Set the client for appending from a log. The client will be called when operations complete.
    fn set_append_client(&'a self, append_client: &'a dyn LogWriteClient);

    /// Append bytes to the end of the log.
    fn append(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ReturnCode, Option<&'static mut [u8]>)>;

    /// Get cookie representing current append cookie.
    fn current_append_cookie(&self) -> StorageCookie;

    /// Sync log to storage.
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
