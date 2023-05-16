// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Standard errors in Tock.

use core::convert::TryFrom;

/// Standard errors in Tock.
///
/// In contrast to [`Result<(), ErrorCode>`](crate::Result<(), ErrorCode>) this
/// does not feature any success cases and is therefore more appropriate for the
/// Tock 2.0 system call interface, where success payloads and errors are not
/// packed into the same 32-bit wide register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum ErrorCode {
    // Reserved value, for when "no error" / "success" should be
    // encoded in the same numeric representation as ErrorCode
    //
    // Ok(()) = 0,
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
    /// Operation is not supported
    NOSUPPORT = 10,
    /// Device is not available
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

impl TryFrom<Result<(), ErrorCode>> for ErrorCode {
    type Error = ();

    fn try_from(rc: Result<(), ErrorCode>) -> Result<Self, Self::Error> {
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

impl From<ErrorCode> for Result<(), ErrorCode> {
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

/// Convert a `Result<(), ErrorCode>` to a StatusCode (usize) for userspace.
///
/// StatusCode is a useful "pseudotype" (there is no actual Rust type called
/// StatusCode in Tock) for three reasons:
///
/// 1. It can be represented in a single `usize`. This allows StatusCode to be
///    easily passed across the syscall interface between the kernel and
///    userspace.
///
/// 2. It extends ErrorCode, but keeps the same error-to-number mappings as
///    ErrorCode. For example, in both StatusCode and ErrorCode, the `SIZE`
///    error is always represented as 7.
///
/// 3. It can encode success values, whereas ErrorCode can only encode errors.
///    Number 0 in ErrorCode is reserved, and is used for `SUCCESS` in
///    StatusCode.
///
/// This helper function converts the Tock and Rust convention for a
/// success/error type to a StatusCode. StatusCode is represented as a usize
/// which is sufficient to send to userspace via an upcall.
///
/// The key to this conversion and portability between the kernel and userspace
/// is that `ErrorCode`, which only expresses errors, is assigned fixed values,
/// but does not use value 0 by convention. This allows us to use 0 as success
/// in ReturnCode.
pub fn into_statuscode(r: Result<(), ErrorCode>) -> usize {
    match r {
        Ok(()) => 0,
        Err(e) => e as usize,
    }
}
