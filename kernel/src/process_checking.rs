use crate::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use crate::hil::digest::{ClientData, ClientVerify};
use crate::hil::digest::{DigestDataVerify, Sha512};
use crate::ErrorCode;
use crate::process::{Process, State};
use crate::utilities::cells::OptionalCell;
use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfFooterV2CredentialsType;


#[derive(Debug)]
pub enum CheckResult {
    Accept,
    Pass,
    Reject,
}

pub trait Client<'a> {
    fn check_done(
        &self,
        result: Result<CheckResult, ErrorCode>,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    );
}

pub trait AppCredentialsChecker<'a> {
    fn set_client(&self, client: &'a dyn Client<'a>);
    fn require_credentials(&self) -> bool;

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])>;
}

pub trait AppIdentification {
    fn different_identifier(&self, 
	                    process_a: &dyn Process,
			    process_b: &dyn Process) -> bool;

    // Return whether there is a currently running process that has
    // the same application identifier as `process`. This means that
    // if `process` is currently running, `has_unique_identifier`
    // returns false.
    fn has_unique_identifier(&self,
                             process: &dyn Process,
                             processes: &[Option<&dyn Process>]) -> bool {
        let len = processes.len();
        if process.get_state() != State::Unstarted && process.get_state() != State::Terminated {
            return false;
        }

        // Note that this causes `process` to compare against itself;
        // however, since `process` should not be running, it will
        // not check the identifiers and say they are different. This means
        // this method returns false if the process is running.
        for i in 0..len {
            let checked_process = processes[i];
            let diff = checked_process
                .map_or(true, |other| {
                    !other.is_running() ||
                        self.different_identifier(process, other)
                });
            if !diff {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShortID {
    id: u32
}

pub trait Compress {
    fn to_short_id(&self, _credentials: &TbfFooterV2Credentials) -> Option<ShortID>;
}

pub trait AppVerifier<'a>: AppCredentialsChecker<'a> + Compress + AppIdentification {}
impl<'a, T: AppCredentialsChecker<'a> + Compress + AppIdentification> AppVerifier<'a> for T {}


pub struct AppCheckerPermissive<'a> {
    pub client: OptionalCell<&'a dyn Client<'a>>,
}

impl<'a> AppCheckerPermissive<'a> {
    pub fn new() -> AppCheckerPermissive<'a> {
        AppCheckerPermissive {
            client: OptionalCell::empty(),
        }
    }
}

impl<'a> AppCredentialsChecker<'a> for AppCheckerPermissive<'a> {
    fn require_credentials(&self) -> bool {
        false
    }

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])> {
        Err((ErrorCode::NOSUPPORT, credentials, binary))
    }

    fn set_client(&self, client: &'a dyn Client<'a>) {
        self.client.replace(client);
    }
}

impl AppIdentification for AppCheckerPermissive<'_> {
    fn different_identifier(&self, 
	                    _process_a: &dyn Process,
			    _process_b: &dyn Process) -> bool {
        true
    }
}

impl Compress for AppCheckerPermissive<'_> {
    fn to_short_id(&self, _credentials: &TbfFooterV2Credentials) -> Option<ShortID> {
        None
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
            binary: OptionalCell::empty(),
        }
    }

    pub fn initialize_callback_handle(&self, handle: DeferredCallHandle) {
        self.handle.replace(handle);
    }
}

impl<'a> DynamicDeferredCallClient for AppCheckerSimulated<'a> {
    fn call(&self, _handle: DeferredCallHandle) {
        self.client.map(|c| {
            c.check_done(
                Ok(CheckResult::Pass),
                self.credentials.take().unwrap(),
                self.binary.take().unwrap(),
            )
        });
    }
}

impl<'a> AppCredentialsChecker<'a> for AppCheckerSimulated<'a> {
    fn require_credentials(&self) -> bool {
        false
    }

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])> {
        self.handle
            .map_or(Err((ErrorCode::FAIL, credentials, binary)), |handle| {
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

impl AppIdentification for AppCheckerSimulated<'_> {
    // This checker doesn't allow you to run two processes with the
    // same name.
    fn different_identifier(&self, 
	                    process_a: &dyn Process,
			    process_b: &dyn Process) -> bool {
        let a = process_a.get_process_name();
        let b = process_b.get_process_name();
        !a.eq(b)
    }
}

impl Compress for AppCheckerSimulated<'_> {
    fn to_short_id(&self, _credentials: &TbfFooterV2Credentials) -> Option<ShortID> {
        None
    }
}

trait AppCheckerHMAC: DigestDataVerify<'static, 64_usize> + Sha512 {}

pub struct AppCheckerSha512<'a> {
    _hmac: &'a dyn AppCheckerHMAC,
    client: OptionalCell<&'a dyn Client<'a>>,
}

impl<'a> AppCredentialsChecker<'a> for AppCheckerSha512<'a> {
    fn require_credentials(&self) -> bool {
        true
    }

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])> {
        match credentials.format() {
            TbfFooterV2CredentialsType::Padding | TbfFooterV2CredentialsType::CleartextID => {
                Err((ErrorCode::ALREADY, credentials, binary))
            }
            TbfFooterV2CredentialsType::Rsa3072Key
            | TbfFooterV2CredentialsType::Rsa4096Key
            | TbfFooterV2CredentialsType::Rsa3072KeyWithID
            | TbfFooterV2CredentialsType::Rsa4096KeyWithID
            | TbfFooterV2CredentialsType::SHA256
            | TbfFooterV2CredentialsType::SHA384
            | TbfFooterV2CredentialsType::SHA512 => {
                Err((ErrorCode::NOSUPPORT, credentials, binary))
            }
        }
    }

    fn set_client(&self, client: &'a dyn Client<'a>) {
        self.client.replace(client);
    }
}

impl AppIdentification for AppCheckerSha512<'_> {
    fn different_identifier(&self, 
	                    process_a: &dyn Process,
			    process_b: &dyn Process) -> bool {
        let credentials_a = process_a.get_credentials();
        let credentials_b = process_b.get_credentials();
        credentials_a.map_or(true, |a|
          credentials_b.map_or(true, |b| {
              if a.format() != b.format() {
                  return true;
              } else {
                  let data_a = a.data();
                  let data_b = b.data();
                  for (p1, p2) in data_a.iter().zip(data_b.iter()) {
                      if p1 != p2 {
                          return true;
                      }
                  }
              }
              false
          }))
    }
}

impl<'a> ClientData<'a, 64_usize> for AppCheckerSha512<'a> {
    fn add_data_done(&'a self, _result: Result<(), ErrorCode>, _data: &'static mut [u8]) {}
}
impl<'a> ClientVerify<'a, 64_usize> for AppCheckerSha512<'a> {
    fn verification_done(
        &'a self,
        _result: Result<bool, ErrorCode>,
        _compare: &'static mut [u8; 64_usize],
    ) {
    }
}
