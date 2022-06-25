use crate::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use crate::hil::digest::{ClientData, ClientHash, ClientVerify};
use crate::hil::digest::{DigestDataVerify, Sha256};
use crate::process::{Process, State};
use crate::utilities::cells::OptionalCell;
use crate::utilities::cells::TakeCell;
use crate::utilities::leasable_buffer::{LeasableBuffer, LeasableMutableBuffer};
use crate::ErrorCode;
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
    fn different_identifier(&self, process_a: &dyn Process, process_b: &dyn Process) -> bool;

    // Return whether there is a currently running process that has
    // the same application identifier as `process`. This means that
    // if `process` is currently running, `has_unique_identifier`
    // returns false.
    fn has_unique_identifier(
        &self,
        process: &dyn Process,
        processes: &[Option<&dyn Process>],
    ) -> bool {
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
            let diff = checked_process.map_or(true, |other| {
                !other.is_running() || self.different_identifier(process, other)
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
    id: core::num::NonZeroU32,
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
    fn different_identifier(&self, _process_a: &dyn Process, _process_b: &dyn Process) -> bool {
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
    fn different_identifier(&self, process_a: &dyn Process, process_b: &dyn Process) -> bool {
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

pub trait Sha256Verifier<'a>: DigestDataVerify<'a, 32_usize> + Sha256 {}
impl<'a, T: DigestDataVerify<'a, 32_usize> + Sha256> Sha256Verifier<'a> for T {}

pub struct AppCheckerSha256 {
    hasher: &'static dyn Sha256Verifier<'static>,
    client: OptionalCell<&'static dyn Client<'static>>,
    hash: TakeCell<'static, [u8; 32]>,
    binary: OptionalCell<&'static [u8]>,
    credentials: OptionalCell<TbfFooterV2Credentials>,
}

static mut CREDS: [u8; 32] = [0; 32];

impl AppCheckerSha256 {
    pub fn new(hash: &'static dyn Sha256Verifier<'static>) -> AppCheckerSha256 {
        unsafe {
            AppCheckerSha256 {
                hasher: hash,
                client: OptionalCell::empty(),
                hash: TakeCell::new(&mut CREDS),
                credentials: OptionalCell::empty(),
                binary: OptionalCell::empty(),
            }
        }
    }
}

impl AppCredentialsChecker<'static> for AppCheckerSha256 {
    fn require_credentials(&self) -> bool {
        true
    }

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'static [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'static [u8])> {
        self.credentials.set(credentials);
        match credentials.format() {
            TbfFooterV2CredentialsType::Padding | TbfFooterV2CredentialsType::CleartextID => {
                Err((ErrorCode::ALREADY, credentials, binary))
            }
            TbfFooterV2CredentialsType::Rsa3072Key
            | TbfFooterV2CredentialsType::Rsa4096Key
            | TbfFooterV2CredentialsType::Rsa3072KeyWithID
            | TbfFooterV2CredentialsType::Rsa4096KeyWithID
            | TbfFooterV2CredentialsType::SHA384
            | TbfFooterV2CredentialsType::SHA512 => {
                Err((ErrorCode::NOSUPPORT, credentials, binary))
            }
            TbfFooterV2CredentialsType::SHA256 => {
                self.hash.map(|h| {
                    for i in 0..32 {
                        h[i] = credentials.data()[i];
                    }
                });
                self.hasher.clear_data();
                match self.hasher.add_data(LeasableBuffer::new(binary)) {
                    Ok(()) => Ok(()),
                    Err((e, b)) => Err((e, credentials, b.take())),
                }
            }
        }
    }

    fn set_client(&self, client: &'static dyn Client<'static>) {
        self.client.replace(client);
    }
}

impl AppIdentification for AppCheckerSha256 {
    fn different_identifier(&self, process_a: &dyn Process, process_b: &dyn Process) -> bool {
        let credentials_a = process_a.get_credentials();
        let credentials_b = process_b.get_credentials();
        credentials_a.map_or(true, |a| {
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
            })
        })
    }
}

impl ClientData<32_usize> for AppCheckerSha256 {
    fn add_mut_data_done(
        &self,
        _result: Result<(), ErrorCode>,
        _data: LeasableMutableBuffer<'static, u8>,
    ) {
    }

    fn add_data_done(&self, result: Result<(), ErrorCode>, data: LeasableBuffer<'static, u8>) {
        match result {
            Err(e) => panic!("Internal error during application binary checking. SHA256 engine threw error in adding data: {:?}", e),
            Ok(()) => {
                self.binary.set(data.take());
                let hash: &'static mut [u8; 32_usize] = self.hash.take().unwrap();
                match self.hasher.verify(hash) {
                    Err((e, _)) => panic!("Failed invoke hash verification in process credential checking: {:?}", e),
                    Ok(()) => {},
                }
            }
        }
    }
}

impl<'a> ClientVerify<32_usize> for AppCheckerSha256 {
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        compare: &'static mut [u8; 32_usize],
    ) {
        self.hash.replace(compare);
        match result {
            Ok(true) => {
                self.client.map(|c| {
                    c.check_done(
                        Ok(CheckResult::Accept),
                        self.credentials.take().unwrap(),
                        self.binary.take().unwrap(),
                    );
                });
            }
            Ok(false) => {
                self.client.map(|c| {
                    c.check_done(
                        Ok(CheckResult::Reject),
                        self.credentials.take().unwrap(),
                        self.binary.take().unwrap(),
                    );
                });
            }
            Err(e) => {
                panic!("Error {:?} in processing application credentials.", e);
            }
        }
    }
}

impl<'a> ClientHash<32_usize> for AppCheckerSha256 {
    fn hash_done(&self, _result: Result<(), ErrorCode>, _digest: &'static mut [u8; 32_usize]) {}
}

impl Compress for AppCheckerSha256 {
    fn to_short_id(&self, credentials: &TbfFooterV2Credentials) -> Option<ShortID> {
        let id: u32 = (credentials.data()[0] as u32) << 24
            | (credentials.data()[1] as u32) << 16
            | (credentials.data()[2] as u32) << 8
            | (credentials.data()[3] as u32);
        match core::num::NonZeroU32::new(id) {
            Some(nzid) => Some(ShortID { id: nzid }),
            None => None,
        }
    }
}
