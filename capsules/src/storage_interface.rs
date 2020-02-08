use kernel::ReturnCode;

pub type StorageLen = usize;
pub type OperationResult = Result<(), (ReturnCode, Option<&'static mut [u8]>)>;

/// Cookies represent seekable positions within a storage interface. `SeekBeginning` allows a
/// client to seek to the very beginning of the interface. `Cookie` allows the client to seek to a
/// particular position within the interface. How a `Cookie` internally represents a position is up
/// to the implementer. `Invalid` represents an invalid position that cannot be seeked.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StorageCookie {
    SeekBeginning,
    Cookie(usize),
    Invalid,
}

pub trait HasClient<'a, C> {
    /// Set the client for a storage interface. The client will be called when
    /// operations complete.
    fn set_client(&'a self, client: &'a C);
}

/// An interface for reading from log storage.
pub trait LogRead {
    /// Read log data starting from the current read position.
    fn read(&self, buffer: &'static mut [u8], length: StorageLen) -> OperationResult;

    /// Get cookie representing current read position.
    fn current_read_offset(&self) -> StorageCookie;

    /// Seek to a new read position.
    fn seek(&self, offset: StorageCookie) -> ReturnCode;

    /// Get approximate log capacity in bytes.
    fn get_size(&self) -> StorageLen;
}

/// Receive callbacks from `LogRead`.
pub trait LogReadClient {
    fn read_done(&self, buffer: &'static mut [u8], length: StorageLen, error: ReturnCode);

    fn seek_done(&self, error: ReturnCode);
}

/// An interface for writing to log storage.
pub trait LogWrite {
    /// Append bytes to the end of the log.
    fn append(&self, buffer: &'static mut [u8], length: StorageLen) -> OperationResult;

    /// Get cookie representing current append position.
    fn current_append_offset(&self) -> StorageCookie;

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
        length: StorageLen,
        records_lost: bool,
        error: ReturnCode,
    );

    fn sync_done(&self, error: ReturnCode);

    fn erase_done(&self, error: ReturnCode);
}
