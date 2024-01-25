// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Sample implementations of application credentials checkers, used
//! to decide whether an application can be loaded. See
//| the [AppID TRD](../../doc/reference/trd-appid.md).

use crate::deferred_call::{DeferredCall, DeferredCallClient};
use crate::hil::digest::{ClientData, ClientHash, ClientVerify};
use crate::hil::digest::{DigestDataVerify, Sha256};
use crate::process::{Process, ShortID};
use crate::process_checker::{AppCredentialsChecker, AppUniqueness};
use crate::process_checker::{CheckResult, Client, Compress};
use crate::utilities::cells::OptionalCell;
use crate::utilities::cells::TakeCell;
use crate::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use crate::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfFooterV2CredentialsType;

/// A sample Credentials Checking Policy that loads and runs Userspace
/// Binaries with unique process names; if it encounters a Userspace
/// Binary with the same process name as an existing one it fails the
/// uniqueness check and is not run.
pub struct AppCheckerSimulated<'a> {
    deferred_call: DeferredCall,
    client: OptionalCell<&'a dyn Client<'a>>,
    credentials: OptionalCell<TbfFooterV2Credentials>,
    binary: OptionalCell<&'a [u8]>,
}

impl<'a> AppCheckerSimulated<'a> {
    pub fn new() -> Self {
        Self {
            deferred_call: DeferredCall::new(),
            client: OptionalCell::empty(),
            credentials: OptionalCell::empty(),
            binary: OptionalCell::empty(),
        }
    }
}

impl<'a> DeferredCallClient for AppCheckerSimulated<'a> {
    fn handle_deferred_call(&self) {
        self.client.map(|c| {
            c.check_done(
                Ok(CheckResult::Pass),
                self.credentials.take().unwrap(),
                self.binary.take().unwrap(),
            )
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
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
        if self.credentials.is_none() {
            self.credentials.replace(credentials);
            self.binary.replace(binary);
            self.deferred_call.set();
            Ok(())
        } else {
            Err((ErrorCode::BUSY, credentials, binary))
        }
    }

    fn set_client(&self, client: &'a dyn Client<'a>) {
        self.client.replace(client);
    }
}

impl AppUniqueness for AppCheckerSimulated<'_> {
    // This checker doesn't allow you to run two processes with the
    // same name.
    fn different_identifier(&self, process_a: &dyn Process, process_b: &dyn Process) -> bool {
        let a = process_a.get_process_name();
        let b = process_b.get_process_name();
        !a.eq(b)
    }
}

impl Compress for AppCheckerSimulated<'_> {
    fn to_short_id(
        &self,
        _process: &dyn Process,
        _credentials: &TbfFooterV2Credentials,
    ) -> ShortID {
        ShortID::LocallyUnique
    }
}

pub trait Sha256Verifier<'a>: DigestDataVerify<'a, 32_usize> + Sha256 {}
impl<'a, T: DigestDataVerify<'a, 32_usize> + Sha256> Sha256Verifier<'a> for T {}

/// A Credentials Checking Policy that only runs Userspace Binaries
/// which have a unique SHA256 credential. A Userspace Binary without
/// a SHA256 credential fails checking, and only one Userspace Binary
/// with a particular SHA256 hash runs at any time.
pub struct AppCheckerSha256 {
    hasher: &'static dyn Sha256Verifier<'static>,
    client: OptionalCell<&'static dyn Client<'static>>,
    hash: TakeCell<'static, [u8; 32]>,
    binary: OptionalCell<&'static [u8]>,
    credentials: OptionalCell<TbfFooterV2Credentials>,
}

impl AppCheckerSha256 {
    pub fn new(
        hash: &'static dyn Sha256Verifier<'static>,
        buffer: &'static mut [u8; 32],
    ) -> AppCheckerSha256 {
        AppCheckerSha256 {
            hasher: hash,
            client: OptionalCell::empty(),
            hash: TakeCell::new(buffer),
            credentials: OptionalCell::empty(),
            binary: OptionalCell::empty(),
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
            TbfFooterV2CredentialsType::SHA256 => {
                self.hash.map(|h| {
                    h[..32].copy_from_slice(&credentials.data()[..32]);
                });
                self.hasher.clear_data();
                match self.hasher.add_data(SubSlice::new(binary)) {
                    Ok(()) => Ok(()),
                    Err((e, b)) => Err((e, credentials, b.take())),
                }
            }
            _ => Err((ErrorCode::NOSUPPORT, credentials, binary)),
        }
    }

    fn set_client(&self, client: &'static dyn Client<'static>) {
        self.client.replace(client);
    }
}

impl AppUniqueness for AppCheckerSha256 {
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
    fn add_mut_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSliceMut<'static, u8>) {}

    fn add_data_done(&self, result: Result<(), ErrorCode>, data: SubSlice<'static, u8>) {
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

impl ClientVerify<32_usize> for AppCheckerSha256 {
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

impl ClientHash<32_usize> for AppCheckerSha256 {
    fn hash_done(&self, _result: Result<(), ErrorCode>, _digest: &'static mut [u8; 32_usize]) {}
}

impl Compress for AppCheckerSha256 {
    // This checker generates a short ID from the first 32 bits of the
    // hash and sets the first bit to be 1 to ensure it is non-zero.
    // Note that since these identifiers are only 31 bits, they do not
    // provide sufficient collision resistance to verify a unique identity.
    fn to_short_id(&self, _process: &dyn Process, credentials: &TbfFooterV2Credentials) -> ShortID {
        let id: u32 = 0x8000000_u32
            | (credentials.data()[0] as u32) << 24
            | (credentials.data()[1] as u32) << 16
            | (credentials.data()[2] as u32) << 8
            | (credentials.data()[3] as u32);
        match core::num::NonZeroU32::new(id) {
            Some(nzid) => ShortID::Fixed(nzid),
            None => ShortID::LocallyUnique, // Should never be generated
        }
    }
}

/// A sample Credentials Checking Policy that loads and runs all processes and
/// assigns pseudo-unique ShortIDs.
///
/// ShortIDs are assigned as a non-secure hash of the process name.
///
/// Note, this checker relies on there being at least one credential (of any
/// type) installed so that we can accept the credential and `to_short_id()`
/// will be called.
///
/// ### Usage
///
/// ```rust,ignore
/// let checker = static_init!(
///     kernel::process_checker::basic::AppCheckerNames,
///     kernel::process_checker::basic::AppCheckerNames::new(
///         crate::utilities::helpers::addhash_str
///     )
/// );
/// kernel::deferred_call::DeferredCallClient::register(checker);
/// ```
pub struct AppCheckerNames<'a, F: Fn(&'static str) -> u32> {
    hasher: &'a F,
    deferred_call: DeferredCall,
    client: OptionalCell<&'a dyn Client<'a>>,
    credentials: OptionalCell<TbfFooterV2Credentials>,
    binary: OptionalCell<&'a [u8]>,
}

impl<'a, F: Fn(&'static str) -> u32> AppCheckerNames<'a, F> {
    pub fn new(hasher: &'a F) -> Self {
        Self {
            hasher,
            deferred_call: DeferredCall::new(),
            client: OptionalCell::empty(),
            credentials: OptionalCell::empty(),
            binary: OptionalCell::empty(),
        }
    }
}

impl<'a, F: Fn(&'static str) -> u32> DeferredCallClient for AppCheckerNames<'a, F> {
    fn handle_deferred_call(&self) {
        self.client.map(|c| {
            c.check_done(
                Ok(CheckResult::Accept),
                self.credentials.take().unwrap(),
                self.binary.take().unwrap(),
            )
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<'a, F: Fn(&'static str) -> u32> AppCredentialsChecker<'a> for AppCheckerNames<'a, F> {
    fn require_credentials(&self) -> bool {
        false
    }

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])> {
        if self.credentials.is_none() {
            self.credentials.replace(credentials);
            self.binary.replace(binary);
            self.deferred_call.set();
            Ok(())
        } else {
            Err((ErrorCode::BUSY, credentials, binary))
        }
    }

    fn set_client(&self, client: &'a dyn Client<'a>) {
        self.client.replace(client);
    }
}

impl<'a, F: Fn(&'static str) -> u32> AppUniqueness for AppCheckerNames<'a, F> {
    fn different_identifier(&self, _process_a: &dyn Process, _process_b: &dyn Process) -> bool {
        true
    }
}

impl<'a, F: Fn(&'static str) -> u32> Compress for AppCheckerNames<'a, F> {
    fn to_short_id(&self, process: &dyn Process, _credentials: &TbfFooterV2Credentials) -> ShortID {
        let name = process.get_process_name();
        let sum = (self.hasher)(name);
        match core::num::NonZeroU32::new(sum) {
            Some(id) => ShortID::Fixed(id),
            None => ShortID::LocallyUnique,
        }
    }
}

/// A sample Credentials Checking Policy that loads and runs Userspace
/// Binaries that have RSA3072 or RSA4096 credentials. It uses the
/// public key stored in the credentials as the Application
/// Identifier, and the bottom 31 bits of the public key as the
/// ShortID. WARNING: this policy does not actually check the RSA
/// signature: it always blindly assumes it is correct. This checker
/// exists to test that the Tock boot sequence correctly handles
/// ID collisions and version numbers.
pub struct AppCheckerRsaSimulated<'a> {
    deferred_call: DeferredCall,
    client: OptionalCell<&'a dyn Client<'a>>,
    credentials: OptionalCell<TbfFooterV2Credentials>,
    binary: OptionalCell<&'a [u8]>,
}

impl<'a> AppCheckerRsaSimulated<'a> {
    pub fn new() -> AppCheckerRsaSimulated<'a> {
        Self {
            deferred_call: DeferredCall::new(),
            client: OptionalCell::empty(),
            credentials: OptionalCell::empty(),
            binary: OptionalCell::empty(),
        }
    }
}

impl<'a> DeferredCallClient for AppCheckerRsaSimulated<'a> {
    fn handle_deferred_call(&self) {
        // This checker does not actually verify the RSA signature; it
        // assumes the signature is valid and so accepts any RSA
        // signature. This checker is intended for testing kernel
        // process loading logic, and not for real uses requiring
        // integrity or authenticity.
        self.client.map(|c| {
            let binary = self.binary.take().unwrap();
            let cred = self.credentials.take().unwrap();
            let result = if cred.format() == TbfFooterV2CredentialsType::Rsa3072Key
                || cred.format() == TbfFooterV2CredentialsType::Rsa4096Key
            {
                Ok(CheckResult::Accept)
            } else {
                Ok(CheckResult::Pass)
            };

            c.check_done(result, cred, binary)
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<'a> AppCredentialsChecker<'a> for AppCheckerRsaSimulated<'a> {
    fn require_credentials(&self) -> bool {
        true
    }

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'a [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'a [u8])> {
        if self.credentials.is_none() {
            self.credentials.replace(credentials);
            self.binary.replace(binary);
            self.deferred_call.set();
            Ok(())
        } else {
            Err((ErrorCode::BUSY, credentials, binary))
        }
    }

    fn set_client(&self, client: &'a dyn Client<'a>) {
        self.client.replace(client);
    }
}

impl AppUniqueness for AppCheckerRsaSimulated<'_> {
    fn different_identifier(&self, process_a: &dyn Process, process_b: &dyn Process) -> bool {
        let cred_a = process_a.get_credentials();
        let cred_b = process_b.get_credentials();

        // If it doesn't have credentials, it is by definition
        // different. It should not be runnable (this checker requires
        // credentials), but if this returned false it could block
        // runnable processes from running.
        cred_a.map_or(true, |a| {
            cred_b.map_or(true, |b| {
                // Two IDs are different if they have a different format,
                // different length (should not happen, but worth checking for
                // the next test), or any byte of them differs.
                if a.format() != b.format() {
                    true
                } else if a.data().len() != b.data().len() {
                    true
                } else {
                    for (aval, bval) in a.data().iter().zip(b.data().iter()) {
                        if aval != bval {
                            return true;
                        }
                    }
                    false
                }
            })
        })
    }
}

impl Compress for AppCheckerRsaSimulated<'_> {
    fn to_short_id(&self, _process: &dyn Process, credentials: &TbfFooterV2Credentials) -> ShortID {
        // Should never trigger, as we only approve RSA3072 and RSA4096 credentials.
        let data = credentials.data();
        if data.len() < 4 {
            return ShortID::LocallyUnique;
        }
        let id: u32 = 0x8000000_u32
            | (data[0] as u32) << 24
            | (data[1] as u32) << 16
            | (data[2] as u32) << 8
            | (data[3] as u32);
        match core::num::NonZeroU32::new(id) {
            Some(nzid) => ShortID::Fixed(nzid),
            None => ShortID::LocallyUnique, // Should never be generated
        }
    }
}
