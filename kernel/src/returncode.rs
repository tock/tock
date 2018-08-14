//! Standard return type for invoking operations, returning success or an error
//! code.
//!
//!  Author: Philip Levis <pal@cs.stanford.edu>
//!  Date: Dec 22, 2016

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReturnCode {
    SuccessWithValue { value: usize }, // Success value must be positive
    SUCCESS,
    FAIL,         // Generic failure condition
    EBUSY,        // Underlying system is busy; retry
    EALREADY,     // The state requested is already set
    EOFF,         // The component is powered down
    ERESERVE,     // Reservation required before use
    EINVAL,       // An invalid parameter was passed
    ESIZE,        // Parameter passed was too large
    ECANCEL,      // Operation canceled by a call
    ENOMEM,       // Memory required not available
    ENOSUPPORT,   // Operation or command is unsupported
    ENODEVICE,    // Device does not exist
    EUNINSTALLED, // Device is not physically installed
    ENOACK,       // Packet transmission not acknowledged
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
