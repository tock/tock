use crate::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfFooterV2CredentialsType;
use crate::hil::digest::{ClientData, ClientVerify};
use crate::hil::digest::{DigestDataVerify, Sha512};
use crate::utilities::cells::OptionalCell;
use crate::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};


#[derive(Debug)]
pub enum CheckResult {
    Accept,
    Pass,
    Reject
}

pub trait Client<'a> {
    fn check_done(&self,
                  result: Result<CheckResult, ErrorCode>,
                  credentials: TbfFooterV2Credentials,
                  binary: &'a [u8]);
}

pub trait AppCredentialsChecker<'a> {
    fn set_client(&self, client: &'a dyn Client<'a>);
    fn require_credentials(&self) -> bool;
    fn check_credentials(&self,
                         credentials: TbfFooterV2Credentials,
                         binary: &'a [u8]) ->
        Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])>;
}

pub struct AppCheckerPermissive<'a> {
    pub client: OptionalCell<&'a dyn Client<'a>>
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

    fn set_client(&self, client: &'a dyn Client<'a>) {
        self.client.replace(client);
    }
    
}

pub struct AppCheckerSimulated<'a> {
    deferred_caller: &'a DynamicDeferredCall,
    handle: OptionalCell<DeferredCallHandle>,
    client: OptionalCell<&'a dyn Client<'a>>,
    credentials: OptionalCell<TbfFooterV2Credentials>,
    binary: OptionalCell<&'a [u8]>,
}

impl<'a> AppCheckerSimulated<'a> {
    pub fn new(call: &'a DynamicDeferredCall) -> AppCheckerSimulated<'a> {
        AppCheckerSimulated {
            deferred_caller: call,
            handle: OptionalCell::empty(),
            client: OptionalCell::empty(),
            credentials: OptionalCell::empty(),
            binary: OptionalCell::empty()
        }
    }

    pub fn initialize_callback_handle(&self, handle: DeferredCallHandle) {
        self.handle.replace(handle);
    }
}

impl<'a> DynamicDeferredCallClient for AppCheckerSimulated<'a> {
    fn call(&self, _handle: DeferredCallHandle) {
        self.client.map(|c| c.check_done(Ok(CheckResult::Pass),
                                         self.credentials.take().unwrap(),
                                         self.binary.take().unwrap()));
    }
}

impl<'a> AppCredentialsChecker<'a> for AppCheckerSimulated<'a> {
    fn require_credentials(&self) -> bool {
        false
    }
    
    fn check_credentials(&self,
                         credentials: TbfFooterV2Credentials,
                         binary: &'a [u8])  ->
        Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])> {
            self.handle.map_or(Err((ErrorCode::FAIL, credentials, binary)), |handle| {
                if self.credentials.is_none() {
                    self.credentials.replace(credentials);
                    self.binary.replace(binary);
                    self.deferred_caller.set(*handle);
                    Ok(())
                } else {
                    Err((ErrorCode::BUSY, credentials, binary))
                }
            })

    }

    fn set_client(&self, client: &'a dyn Client<'a>) {
        self.client.replace(client);
    }
    
}

trait AppCheckerHMAC: DigestDataVerify<'static, 64_usize> + Sha512 {}

pub struct AppCheckerSha512<'a> {
    _hmac: &'a dyn AppCheckerHMAC,
    client: OptionalCell<&'a dyn Client<'a>>
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

    fn set_client(&self, client: &'a dyn Client<'a>) {
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

