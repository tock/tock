//! Standard return type for invoking operations, returning success or an error
//! code.
//!
//! - Author: Philip Levis <pal@cs.stanford.edu>
//! - Date: Dec 22, 2016

/// Standard return errors in Tock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReturnCode {
    /// Success value must be positive
    SuccessWithValue { value: usize },
    /// Operation completed successfully
    SUCCESS,
    /// Generic failure condition
    FAIL,
    /// Underlying system is busy; retry
    EBUSY,
    /// The state requested is already set
    EALREADY,
    /// The component is powered down
    EOFF,
    /// Reservation required before use
    ERESERVE,
    /// An invalid parameter was passed
    EINVAL,
    /// Parameter passed was too large
    ESIZE,
    /// Operation canceled by a call
    ECANCEL,
    /// Memory required not available
    ENOMEM,
    /// Operation or command is unsupported
    ENOSUPPORT,
    /// Device does not exist
    ENODEVICE,
    /// Device is not physically installed
    EUNINSTALLED,
    /// Packet transmission not acknowledged
    ENOACK,
}

impl From<ReturnCode> for isize {
    fn from(original: ReturnCode) -> isize {
        match original {
            ReturnCode::SuccessWithValue { value } => value as isize,
            ReturnCode::SUCCESS => 0,
            ReturnCode::FAIL => -1,
            ReturnCode::EBUSY => -2,
            ReturnCode::EALREADY => -3,
            ReturnCode::EOFF => -4,
            ReturnCode::ERESERVE => -5,
            ReturnCode::EINVAL => -6,
            ReturnCode::ESIZE => -7,
            ReturnCode::ECANCEL => -8,
            ReturnCode::ENOMEM => -9,
            ReturnCode::ENOSUPPORT => -10,
            ReturnCode::ENODEVICE => -11,
            ReturnCode::EUNINSTALLED => -12,
            ReturnCode::ENOACK => -13,
        }
    }
}

impl From<ReturnCode> for usize {
    fn from(original: ReturnCode) -> usize {
        isize::from(original) as usize
    }
}

pub type ReturnCodeResult = Result<(), u32>;
