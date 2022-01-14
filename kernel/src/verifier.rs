use crate::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfFooterV2CredentialsType;
use crate::hil::digest::{ClientData, ClientVerify};
use crate::hil::digest::{DigestDataVerify, Sha512};
use crate::utilities::cells::OptionalCell;

pub enum CheckResult {
    Accept,
    Pass,
    Reject
}

pub trait Client {
    fn check_done(&self,
                  result: Result<CheckResult, ErrorCode>,
                  credentials: TbfFooterV2Credentials,
                  binary: &[u8]);
}

pub trait AppCredentialsChecker<'a> {
    fn set_client(&self, client: &'a dyn Client);
    fn require_credentials(&self) -> bool;
    fn check_credentials(&self,
                         credentials: TbfFooterV2Credentials,
                         binary: &'static [u8]) ->
        Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])>;
}

pub struct AppCheckerPermissive<'a> {
    pub client: OptionalCell<&'a dyn Client>
}

impl<'a> AppCheckerPermissive<'a> {
    pub fn new() -> AppCheckerPermissive<'a> {
        AppCheckerPermissive {
            client: OptionalCell::empty()
        }
    }
}

impl<'a> AppCredentialsChecker<'a> for AppCheckerPermissive<'a> {
    fn require_credentials(&self) -> bool {
        false
    }
    
    fn check_credentials(&self,
                         credentials: TbfFooterV2Credentials,
                         binary: &'a [u8])  ->
        Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])> {
            Err((ErrorCode::NOSUPPORT,
                 credentials,
                 binary))
    }

    fn set_client(&self, client: &'a dyn Client) {
        self.client.replace(client);
    }
    
}
trait AppCheckerHMAC: DigestDataVerify<'static, 64_usize> + Sha512 {}

pub struct AppCheckerSha512<'a> {
    _hmac: &'a dyn AppCheckerHMAC,
    client: OptionalCell<&'a dyn Client>
}

impl<'a> AppCredentialsChecker<'a> for AppCheckerSha512<'a>   {
   fn require_credentials(&self) -> bool {
        true
    }
    
    fn check_credentials(&self,
                         credentials: TbfFooterV2Credentials,
                         binary: &'a [u8])  ->
        Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])> {
            match credentials.format() {
                TbfFooterV2CredentialsType::Padding |
                TbfFooterV2CredentialsType::CleartextID => {
                Err((ErrorCode::ALREADY,
                         credentials,
                         binary))
                },
                TbfFooterV2CredentialsType::Rsa3072Key  |
                TbfFooterV2CredentialsType::Rsa4096Key  |
                TbfFooterV2CredentialsType::Rsa3072KeyWithID |
                TbfFooterV2CredentialsType::Rsa4096KeyWithID |
                TbfFooterV2CredentialsType::SHA256           |
                TbfFooterV2CredentialsType::SHA384           |
                TbfFooterV2CredentialsType::SHA512           => {
                    Err((ErrorCode::NOSUPPORT,
                         credentials,
                         binary))
                }
            }
        }

    fn set_client(&self, client: &'a dyn Client) {
        self.client.replace(client);
    }
}

impl<'a> ClientData<'a, 64_usize> for AppCheckerSha512<'a> {
    fn add_data_done(&'a self, _result: Result<(), ErrorCode>, _data: &'static mut [u8]) {
        
    }

}
impl<'a> ClientVerify<'a, 64_usize> for AppCheckerSha512<'a> {
    fn verification_done(&'a self, _result: Result<bool, ErrorCode>, _compare: &'static mut [u8; 64_usize]) {

    }
}

