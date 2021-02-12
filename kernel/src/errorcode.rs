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
    // Reserved value, for when "no error" / "success" should be
    // encoded in the same numeric representation as ErrorCode
    //
    // SUCCESS = 0,
    /// Generic failure condition
    FAIL = 1,
    /// Underlying system is busy; retry
    BUSY = 2,
    /// The state requested is already set
    ALREADY = 3,
    /// The component is powered down
    OFF = 4,
    /// Reservation required before use
    RESERVE = 5,
    /// An invalid parameter was passed
    INVAL = 6,
    /// Parameter passed was too large
    SIZE = 7,
    /// Operation canceled by a call
    CANCEL = 8,
    /// Memory required not available
    NOMEM = 9,
    /// Operation or command is unsupported
    NOSUPPORT = 10,
    /// Device does not exist
    NODEVICE = 11,
    /// Device is not physically installed
    UNINSTALLED = 12,
    /// Packet transmission not acknowledged
    NOACK = 13,
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

impl From<ErrorCode> for ReturnCode {
    fn from(ec: ErrorCode) -> Self {
        match ec {
            ErrorCode::FAIL => ReturnCode::FAIL,
            ErrorCode::BUSY => ReturnCode::EBUSY,
            ErrorCode::ALREADY => ReturnCode::EALREADY,
            ErrorCode::OFF => ReturnCode::EOFF,
            ErrorCode::RESERVE => ReturnCode::ERESERVE,
            ErrorCode::INVAL => ReturnCode::EINVAL,
            ErrorCode::SIZE => ReturnCode::ESIZE,
            ErrorCode::CANCEL => ReturnCode::ECANCEL,
            ErrorCode::NOMEM => ReturnCode::ENOMEM,
            ErrorCode::NOSUPPORT => ReturnCode::ENOSUPPORT,
            ErrorCode::NODEVICE => ReturnCode::ENODEVICE,
            ErrorCode::UNINSTALLED => ReturnCode::EUNINSTALLED,
            ErrorCode::NOACK => ReturnCode::ENOACK,
        }
    }
}
