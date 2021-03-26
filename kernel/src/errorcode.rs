//! Standard errors in Tock.

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
            Ok(()) => Err(()),
            Err(ErrorCode::FAIL) => Ok(ErrorCode::FAIL),
            Err(ErrorCode::BUSY) => Ok(ErrorCode::BUSY),
            Err(ErrorCode::ALREADY) => Ok(ErrorCode::ALREADY),
            Err(ErrorCode::OFF) => Ok(ErrorCode::OFF),
            Err(ErrorCode::RESERVE) => Ok(ErrorCode::RESERVE),
            Err(ErrorCode::INVAL) => Ok(ErrorCode::INVAL),
            Err(ErrorCode::SIZE) => Ok(ErrorCode::SIZE),
            Err(ErrorCode::CANCEL) => Ok(ErrorCode::CANCEL),
            Err(ErrorCode::NOMEM) => Ok(ErrorCode::NOMEM),
            Err(ErrorCode::NOSUPPORT) => Ok(ErrorCode::NOSUPPORT),
            Err(ErrorCode::NODEVICE) => Ok(ErrorCode::NODEVICE),
            Err(ErrorCode::UNINSTALLED) => Ok(ErrorCode::UNINSTALLED),
            Err(ErrorCode::NOACK) => Ok(ErrorCode::NOACK),
        }
    }
}

impl From<ErrorCode> for ReturnCode {
    fn from(ec: ErrorCode) -> Self {
        match ec {
            ErrorCode::FAIL => Err(ErrorCode::FAIL),
            ErrorCode::BUSY => Err(ErrorCode::BUSY),
            ErrorCode::ALREADY => Err(ErrorCode::ALREADY),
            ErrorCode::OFF => Err(ErrorCode::OFF),
            ErrorCode::RESERVE => Err(ErrorCode::RESERVE),
            ErrorCode::INVAL => Err(ErrorCode::INVAL),
            ErrorCode::SIZE => Err(ErrorCode::SIZE),
            ErrorCode::CANCEL => Err(ErrorCode::CANCEL),
            ErrorCode::NOMEM => Err(ErrorCode::NOMEM),
            ErrorCode::NOSUPPORT => Err(ErrorCode::NOSUPPORT),
            ErrorCode::NODEVICE => Err(ErrorCode::NODEVICE),
            ErrorCode::UNINSTALLED => Err(ErrorCode::UNINSTALLED),
            ErrorCode::NOACK => Err(ErrorCode::NOACK),
        }
    }
}
