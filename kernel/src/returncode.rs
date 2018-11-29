//! Standard return type for invoking operations, returning success or an error
//! code.
//!
//! - Author: Philip Levis <pal@cs.stanford.edu>
//! - Date: Dec 22, 2016

pub type ReturnCode = Result<Success, Error>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Success {
    Success,
    WithValue { value: usize },
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Error {
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

impl From<Success> for isize {
    fn from(s: Success) -> isize {
        match s {
            Success::WithValue { value } => value as isize,
            Success::Success => 0,
        }
    }
}

impl From<Error> for isize {
    fn from(err: Error) -> isize {
        match err {
            Error::FAIL => -1,
            Error::EBUSY => -2,
            Error::EALREADY => -3,
            Error::EOFF => -4,
            Error::ERESERVE => -5,
            Error::EINVAL => -6,
            Error::ESIZE => -7,
            Error::ECANCEL => -8,
            Error::ENOMEM => -9,
            Error::ENOSUPPORT => -10,
            Error::ENODEVICE => -11,
            Error::EUNINSTALLED => -12,
            Error::ENOACK => -13,
        }
    }
}

impl From<Success> for usize {
    fn from(s: Success) -> usize {
        isize::from(s) as usize
    }
}

impl From<Error> for usize {
    fn from(err: Error) -> usize {
        isize::from(err) as usize
    }
}
