//! Standard error enum for invoking operations

use core::convert::TryFrom;

use crate::ReturnCode;

/// Standard errors in Tock.
///
/// In contrast to [`ReturnCode`](crate::ReturnCode) this does not
/// feature any success cases and is therefore more approriate for the
/// Tock 2.0 system call interface, where success payloads and errors
/// are not packed into the same 32-bit wide register.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(usize)]
pub enum ErrorCode {
    /// Generic failure condition
    FAIL = 0,
    /// Underlying system is busy; retry
    EBUSY = 1,
    /// The state requested is already set
    EALREADY = 2,
    /// The component is powered down
    EOFF = 3,
    /// Reservation required before use
    ERESERVE = 4,
    /// An invalid parameter was passed
    EINVAL = 5,
    /// Parameter passed was too large
    ESIZE = 6,
    /// Operation canceled by a call
    ECANCEL = 7,
    /// Memory required not available
    ENOMEM = 8,
    /// Operation or command is unsupported
    ENOSUPPORT = 9,
    /// Device does not exist
    ENODEVICE = 10,
    /// Device is not physically installed
    EUNINSTALLED = 11,
    /// Packet transmission not acknowledged
    ENOACK = 12,
}

impl From<ErrorCode> for usize {
    fn from(err: ErrorCode) -> usize {
        err as usize
    }
}

impl TryFrom<ReturnCode> for ErrorCode {
    type Error = ();

    fn try_from(rc: ReturnCode) -> Result<Self, Self::Error> {
        match rc {
            ReturnCode::SuccessWithValue { .. } => Err(()),
            ReturnCode::SUCCESS => Err(()),
            ReturnCode::FAIL => Ok(ErrorCode::FAIL),
            ReturnCode::EBUSY => Ok(ErrorCode::EBUSY),
            ReturnCode::EALREADY => Ok(ErrorCode::EALREADY),
            ReturnCode::EOFF => Ok(ErrorCode::EOFF),
            ReturnCode::ERESERVE => Ok(ErrorCode::ERESERVE),
            ReturnCode::EINVAL => Ok(ErrorCode::EINVAL),
            ReturnCode::ESIZE => Ok(ErrorCode::ESIZE),
            ReturnCode::ECANCEL => Ok(ErrorCode::ECANCEL),
            ReturnCode::ENOMEM => Ok(ErrorCode::ENOMEM),
            ReturnCode::ENOSUPPORT => Ok(ErrorCode::ENOSUPPORT),
            ReturnCode::ENODEVICE => Ok(ErrorCode::ENODEVICE),
            ReturnCode::EUNINSTALLED => Ok(ErrorCode::EUNINSTALLED),
            ReturnCode::ENOACK => Ok(ErrorCode::ENOACK),
        }
    }
}
