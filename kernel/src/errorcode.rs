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
    BUSY = 1,
    /// The state requested is already set
    ALREADY = 2,
    /// The component is powered down
    OFF = 3,
    /// Reservation required before use
    RESERVE = 4,
    /// An invalid parameter was passed
    INVAL = 5,
    /// Parameter passed was too large
    SIZE = 6,
    /// Operation canceled by a call
    CANCEL = 7,
    /// Memory required not available
    NOMEM = 8,
    /// Operation or command is unsupported
    NOSUPPORT = 9,
    /// Device does not exist
    NODEVICE = 10,
    /// Device is not physically installed
    UNINSTALLED = 11,
    /// Packet transmission not acknowledged
    NOACK = 12,
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
            ReturnCode::EBUSY => Ok(ErrorCode::BUSY),
            ReturnCode::EALREADY => Ok(ErrorCode::ALREADY),
            ReturnCode::EOFF => Ok(ErrorCode::OFF),
            ReturnCode::ERESERVE => Ok(ErrorCode::RESERVE),
            ReturnCode::EINVAL => Ok(ErrorCode::INVAL),
            ReturnCode::ESIZE => Ok(ErrorCode::SIZE),
            ReturnCode::ECANCEL => Ok(ErrorCode::CANCEL),
            ReturnCode::ENOMEM => Ok(ErrorCode::NOMEM),
            ReturnCode::ENOSUPPORT => Ok(ErrorCode::NOSUPPORT),
            ReturnCode::ENODEVICE => Ok(ErrorCode::NODEVICE),
            ReturnCode::EUNINSTALLED => Ok(ErrorCode::UNINSTALLED),
            ReturnCode::ENOACK => Ok(ErrorCode::NOACK),
        }
    }
}
