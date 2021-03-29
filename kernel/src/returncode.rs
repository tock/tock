use crate::errorcode::ErrorCode;
pub type ReturnCode = Result<(), ErrorCode>;
pub fn retcode_into_usize(original: ReturnCode) -> usize {
    let out = match original {
        Ok(()) => 0,
        Err(e) => match e {
            ErrorCode::FAIL => -1,
            ErrorCode::BUSY => -2,
            ErrorCode::ALREADY => -3,
            ErrorCode::OFF => -4,
            ErrorCode::RESERVE => -5,
            ErrorCode::INVAL => -6,
            ErrorCode::SIZE => -7,
            ErrorCode::CANCEL => -8,
            ErrorCode::NOMEM => -9,
            ErrorCode::NOSUPPORT => -10,
            ErrorCode::NODEVICE => -11,
            ErrorCode::UNINSTALLED => -12,
            ErrorCode::NOACK => -13,
        },
    };
    out as usize
}
