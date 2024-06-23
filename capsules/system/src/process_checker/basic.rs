// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Sample implementations of application credentials checkers, used
//! to decide whether an application can be loaded. See
//| the [AppID TRD](../../doc/reference/trd-appid.md).

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::digest::{ClientData, ClientHash, ClientVerify};
use kernel::hil::digest::{DigestDataVerify, Sha256};
use kernel::process::{Process, ProcessBinary, ShortId};
use kernel::process_checker::CheckResult;
use kernel::process_checker::{AppCredentialsPolicy, AppCredentialsPolicyClient};
use kernel::process_checker::{AppUniqueness, Compress};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfFooterV2CredentialsType;

/// A sample Credentials Checking Policy that approves all apps.
pub struct AppCheckerNull {}

impl AppCheckerNull {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> AppCredentialsPolicy<'a> for AppCheckerNull {
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

    fn set_client(&self, _client: &'a dyn AppCredentialsPolicyClient<'a>) {}
}

pub struct AppIdAssignerSimulated {}

impl AppUniqueness for AppIdAssignerSimulated {
    // This checker doesn't allow you to run two processes with the
    // same name.
    fn different_identifier(&self, process_a: &ProcessBinary, process_b: &ProcessBinary) -> bool {
        let a = process_a.header.get_package_name().unwrap_or("");
        let b = process_b.header.get_package_name().unwrap_or("");
        !a.eq(b)
    }

    fn different_identifier_process(
        &self,
        process_binary: &ProcessBinary,
        process: &dyn Process,
    ) -> bool {
        let a = process_binary.header.get_package_name().unwrap_or("");
        let b = process.get_process_name();
        !a.eq(b)
    }

    fn different_identifier_processes(
        &self,
        process_a: &dyn Process,
        process_b: &dyn Process,
    ) -> bool {
        let a = process_a.get_process_name();
        let b = process_b.get_process_name();
        !a.eq(b)
    }
}

impl Compress for AppIdAssignerSimulated {
    fn to_short_id(&self, _process: &ProcessBinary) -> ShortId {
        ShortId::LocallyUnique
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
    client: OptionalCell<&'static dyn AppCredentialsPolicyClient<'static>>,
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

impl AppCredentialsPolicy<'static> for AppCheckerSha256 {
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

    fn set_client(&self, client: &'static dyn AppCredentialsPolicyClient<'static>) {
        self.client.replace(client);
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
                if let Err((e, _)) = self.hasher.verify(hash) { panic!("Failed invoke hash verification in process credential checking: {:?}", e) }
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

/// A sample AppID Assignment tool that assigns pseudo-unique AppIDs and
/// ShortIds based on the process name.
///
/// ShortIds are assigned as a non-secure hash of the process name.
///
/// ### Usage
///
/// ```rust,ignore
/// let assigner = static_init!(
///     kernel::process_checker::basic::AppIdAssignerNames<fn(&'static str) -> u32>,
///     kernel::process_checker::basic::AppIdAssignerNames::new(
///         &((|s| { kernel::utilities::helpers::crc32_posix(s.as_bytes()) })
///         as fn(&'static str) -> u32)
///     )
/// );
/// ```
pub struct AppIdAssignerNames<'a, F: Fn(&'static str) -> u32> {
    hasher: &'a F,
}

impl<'a, F: Fn(&'static str) -> u32> AppIdAssignerNames<'a, F> {
    pub fn new(hasher: &'a F) -> Self {
        Self { hasher }
    }
}

impl<'a, F: Fn(&'static str) -> u32> AppUniqueness for AppIdAssignerNames<'a, F> {
    fn different_identifier(&self, process_a: &ProcessBinary, process_b: &ProcessBinary) -> bool {
        self.to_short_id(process_a) != self.to_short_id(process_b)
    }

    fn different_identifier_process(
        &self,
        process_a: &ProcessBinary,
        process_b: &dyn Process,
    ) -> bool {
        self.to_short_id(process_a) != process_b.short_app_id()
    }

    fn different_identifier_processes(
        &self,
        process_a: &dyn Process,
        process_b: &dyn Process,
    ) -> bool {
        process_a.short_app_id() != process_b.short_app_id()
    }
}

impl<'a, F: Fn(&'static str) -> u32> Compress for AppIdAssignerNames<'a, F> {
    fn to_short_id(&self, process: &ProcessBinary) -> ShortId {
        let name = process.header.get_package_name().unwrap_or("");
        let sum = (self.hasher)(name);
        core::num::NonZeroU32::new(sum).into()
    }
}

/// A sample Credentials Checking Policy that loads and runs Userspace
/// Binaries that have RSA3072 or RSA4096 credentials. It uses the
/// public key stored in the credentials as the Application
/// Identifier, and the bottom 31 bits of the public key as the
/// ShortId. WARNING: this policy does not actually check the RSA
/// signature: it always blindly assumes it is correct. This checker
/// exists to test that the Tock boot sequence correctly handles
/// ID collisions and version numbers.
pub struct AppCheckerRsaSimulated<'a> {
    deferred_call: DeferredCall,
    client: OptionalCell<&'a dyn AppCredentialsPolicyClient<'a>>,
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

impl<'a> AppCredentialsPolicy<'a> for AppCheckerRsaSimulated<'a> {
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

    fn set_client(&self, client: &'a dyn AppCredentialsPolicyClient<'a>) {
        self.client.replace(client);
    }
}
