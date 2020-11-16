//! The standard success codes used by TickFS.

/// Standard success codes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SuccessCode {
    /// The key was written to flash, the operation is complete
    Written,
    /// The write operation has been queued
    Queued,
}

impl From<SuccessCode> for isize {
    fn from(original: SuccessCode) -> isize {
        match original {
            SuccessCode::Written => -1,
            SuccessCode::Queued => -2,
        }
    }
}

impl From<SuccessCode> for usize {
    fn from(original: SuccessCode) -> usize {
        isize::from(original) as usize
    }
}
