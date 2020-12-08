//! The standard success codes used by TicKV.

/// Standard success codes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SuccessCode {
    /// Operation complete, no changes have been made to flash.
    Complete,
    /// All changes have been written to flash. The operation is complete.
    Written,
    /// The write operation has been queued
    Queued,
}

impl From<SuccessCode> for isize {
    fn from(original: SuccessCode) -> isize {
        match original {
            SuccessCode::Complete => -1,
            SuccessCode::Written => -2,
            SuccessCode::Queued => -3,
        }
    }
}

impl From<SuccessCode> for usize {
    fn from(original: SuccessCode) -> usize {
        isize::from(original) as usize
    }
}
