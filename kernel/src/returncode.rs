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

impl ReturnCode {
    pub fn is_ok(&self) -> bool {
        match self {
            ReturnCode::SUCCESS | ReturnCode::SuccessWithValue { .. } => true,
            _ => false,
        }
    }
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

impl From<Result<(), crate::ErrorCode>> for ReturnCode {
    fn from(result: Result<(), crate::ErrorCode>) -> Self {
        match result {
            Ok(_) => ReturnCode::SUCCESS,
            Err(e) => e.into(),
        }
    }
}

impl From<Result<usize, crate::ErrorCode>> for ReturnCode {
    fn from(result: Result<usize, crate::ErrorCode>) -> Self {
        match result {
            Ok(value) => ReturnCode::SuccessWithValue { value },
            Err(e) => e.into(),
        }
    }
}

impl From<ReturnCode> for Result<(), crate::ErrorCode> {
    fn from(result: ReturnCode) -> Self {
        use core::convert::TryInto;
        result
            .try_into()
            .map(|e: crate::ErrorCode| Err(e))
            .unwrap_or(Ok(()))
    }
}

impl From<ReturnCode> for usize {
    fn from(original: ReturnCode) -> usize {
        isize::from(original) as usize
    }
}

impl From<ReturnCode> for Result<ReturnCode, ReturnCode> {
    fn from(original: ReturnCode) -> Result<ReturnCode, ReturnCode> {
        match original {
            ReturnCode::SUCCESS => Ok(ReturnCode::SUCCESS),
            ReturnCode::SuccessWithValue { value } => Ok(ReturnCode::SuccessWithValue { value }),
            error => Err(error),
        }
    }
}

impl<T: Into<ReturnCode>> core::ops::FromResidual<T> for ReturnCode {
    fn from_residual(residual: T) -> Self {
        residual.into()
    }
}

impl<T: Into<ReturnCode>> core::ops::FromResidual<Result<core::convert::Infallible, T>>
    for ReturnCode
{
    fn from_residual(residual: Result<core::convert::Infallible, T>) -> Self {
        match residual {
            Err(err) => err.into(),
            Ok(never) => match never {},
        }
    }
}

impl<T, F: From<ReturnCode>> core::ops::FromResidual<ReturnCode> for Result<T, F> {
    fn from_residual(residual: ReturnCode) -> Self {
        Err(residual.into())
    }
}

impl core::ops::Try for ReturnCode {
    type Output = Self;
    type Residual = Self;

    fn from_output(output: Self::Output) -> Self {
        output
    }

    fn branch(self) -> core::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            ReturnCode::SUCCESS => core::ops::ControlFlow::Continue(ReturnCode::SUCCESS),
            ReturnCode::SuccessWithValue { value } => {
                core::ops::ControlFlow::Continue(ReturnCode::SuccessWithValue { value })
            }
            error => core::ops::ControlFlow::Break(error),
        }
    }
}
