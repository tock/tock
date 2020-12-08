//! The standard error codes used by TicKV.

/// Standard error codes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorCode {
    /// We found a header in the flash that we don't support
    UnsupportedVersion,
    /// Some of the data in flash appears to be corrupt
    CorruptData,
    /// The check sum doesn't match
    /// Note that the value buffer is still filled
    InvalidCheckSum,
    /// The requested key couldn't be found
    KeyNotFound,
    /// Indicates that we can't add this key as one with
    /// the same key hash already exists.
    KeyAlreadyExists,
    /// Indicates that the region where this object should be added
    /// is full. In future this error should be handled internally
    /// by allocating the object in a different region.
    RegionFull,
    /// Unable to add a key, the flash is full. Note that the flash
    /// might not be full after running a garbage collection.
    FlashFull,
    /// Unable to read the flash region
    ReadFail,
    /// Unable to write the buffer to the flash address
    WriteFail,
    /// Unable to erase the flash region
    EraseFail,
    /// The object is larger then 0x7FFF
    ObjectTooLarge,
    /// The supplied buffer is too small.
    /// The error code includes the total length of the value.
    BufferTooSmall(usize),
    /// Indicates that the flash read operation is not yet ready.
    /// The process can be retried by calling `continue_operation()`.
    ReadNotReady(usize),
    /// Indicates that the flash write operation is not yet ready.
    WriteNotReady(usize),
    /// Indicates that the flash erase operation is not yet ready.
    EraseNotReady(usize),
}

impl From<ErrorCode> for isize {
    fn from(original: ErrorCode) -> isize {
        match original {
            ErrorCode::UnsupportedVersion => -1,
            ErrorCode::CorruptData => -2,
            ErrorCode::InvalidCheckSum => -3,
            ErrorCode::KeyNotFound => -4,
            ErrorCode::KeyAlreadyExists => -5,
            ErrorCode::RegionFull => -6,
            ErrorCode::FlashFull => -7,
            ErrorCode::ReadFail => -8,
            ErrorCode::WriteFail => -9,
            ErrorCode::EraseFail => -10,
            ErrorCode::ObjectTooLarge => -11,
            ErrorCode::BufferTooSmall(_) => -12,
            ErrorCode::ReadNotReady(_) => -13,
            ErrorCode::WriteNotReady(_) => -14,
            ErrorCode::EraseNotReady(_) => -15,
        }
    }
}

impl From<ErrorCode> for usize {
    fn from(original: ErrorCode) -> usize {
        isize::from(original) as usize
    }
}
