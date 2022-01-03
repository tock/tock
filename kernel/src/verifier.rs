use crate::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;
use crate::hil::digest::{DigestDataVerify, HMACSha256};

pub enum CheckResult {
    Accept,
    Pass,
    Reject
}

pub trait Client {
    fn check_done(&self,
                  result: Result<CheckResult, ErrorCode>,
                  credentials: &TbfFooterV2Credentials,
                  binary: &[u8]);
}

pub trait AppCredentialsChecker<'a> {
    //fn set_client(&self, client: &'a dyn Client);
    fn require_credentials(&self) -> bool;
    fn check_credentials(&self,
                         credentials: &'a TbfFooterV2Credentials,
                         binary: &'static [u8]) ->
        Result<(), (ErrorCode, &'a TbfFooterV2Credentials, &'a [u8])>;
}

pub struct AppCheckerPermissive {}

impl<'a> AppCredentialsChecker<'a> for AppCheckerPermissive {
    fn require_credentials(&self) -> bool {
        false
    }
    
    fn check_credentials(&self,
                         credentials: &'a TbfFooterV2Credentials,
                         binary: &'a [u8])  ->
        Result<(), (ErrorCode, &'a TbfFooterV2Credentials, &'a [u8])> {
            Err((ErrorCode::NOSUPPORT,
                 credentials,
                 binary))
    }
}
trait AppCheckerHMAC: DigestDataVerify<'static, 32_usize> + HMACSha256 {}

pub struct AppCheckerHMACSha256 {
  hmac: &'static dyn AppCheckerHMAC
}


