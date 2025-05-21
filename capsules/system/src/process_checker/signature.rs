// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Signature credential checker for checking process credentials.

use kernel::hil;
use kernel::process_checker::CheckResult;
use kernel::process_checker::{AppCredentialsPolicy, AppCredentialsPolicyClient};
use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfFooterV2CredentialsType;

/// Checker that validates a correct signature credential.
///
/// This checker provides the scaffolding on top of a hasher (`&H`) and a
/// verifier (`&S`) for a given `TbfFooterV2CredentialsType`.
///
/// This assumes the `TbfFooterV2CredentialsType` data format only contains the
/// signature (i.e. the data length of the credential in the TBF footer is the
/// same as `SIGNATURE_LEN`).
pub struct AppCheckerSignature<
    'a,
    S: hil::public_key_crypto::signature::SignatureVerify<'static, HASH_LEN, SIGNATURE_LEN>,
    H: hil::digest::DigestDataHash<'a, HASH_LEN>,
    const HASH_LEN: usize,
    const SIGNATURE_LEN: usize,
> {
    hasher: &'a H,
    verifier: &'a S,
    hash: MapCell<&'static mut [u8; HASH_LEN]>,
    signature: MapCell<&'static mut [u8; SIGNATURE_LEN]>,
    client: OptionalCell<&'static dyn AppCredentialsPolicyClient<'static>>,
    credential_type: TbfFooterV2CredentialsType,
    credentials: OptionalCell<TbfFooterV2Credentials>,
    binary: OptionalCell<&'static [u8]>,
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HASH_LEN, SIGNATURE_LEN>,
        H: hil::digest::DigestDataHash<'a, HASH_LEN>,
        const HASH_LEN: usize,
        const SIGNATURE_LEN: usize,
    > AppCheckerSignature<'a, S, H, HASH_LEN, SIGNATURE_LEN>
{
    pub fn new(
        hasher: &'a H,
        verifier: &'a S,
        hash_buffer: &'static mut [u8; HASH_LEN],
        signature_buffer: &'static mut [u8; SIGNATURE_LEN],
        credential_type: TbfFooterV2CredentialsType,
    ) -> AppCheckerSignature<'a, S, H, HASH_LEN, SIGNATURE_LEN> {
        Self {
            hasher,
            verifier,
            hash: MapCell::new(hash_buffer),
            signature: MapCell::new(signature_buffer),
            client: OptionalCell::empty(),
            credential_type,
            credentials: OptionalCell::empty(),
            binary: OptionalCell::empty(),
        }
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HASH_LEN, SIGNATURE_LEN>,
        H: hil::digest::DigestDataHash<'a, HASH_LEN>,
        const HASH_LEN: usize,
        const SIGNATURE_LEN: usize,
    > hil::digest::ClientData<HASH_LEN> for AppCheckerSignature<'a, S, H, HASH_LEN, SIGNATURE_LEN>
{
    fn add_mut_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSliceMut<'static, u8>) {}

    fn add_data_done(&self, result: Result<(), ErrorCode>, data: SubSlice<'static, u8>) {
        self.binary.set(data.take());

        // We added the binary data to the hasher, now we can compute the hash.
        match result {
            Err(e) => {
                self.client.map(|c| {
                    let binary = self.binary.take().unwrap();
                    let cred = self.credentials.take().unwrap();
                    c.check_done(Err(e), cred, binary)
                });
            }
            Ok(()) => {
                self.hash.take().map(|h| {
                    if let Err((e, _)) = self.hasher.run(h) {
                        self.client.map(|c| {
                            let binary = self.binary.take().unwrap();
                            let cred = self.credentials.take().unwrap();
                            c.check_done(Err(e), cred, binary)
                        });
                    }
                });
            }
        }
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HASH_LEN, SIGNATURE_LEN>,
        H: hil::digest::DigestDataHash<'a, HASH_LEN>,
        const HASH_LEN: usize,
        const SIGNATURE_LEN: usize,
    > hil::digest::ClientHash<HASH_LEN> for AppCheckerSignature<'a, S, H, HASH_LEN, SIGNATURE_LEN>
{
    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut [u8; HASH_LEN]) {
        match result {
            Err(e) => {
                self.hash.replace(digest);
                self.client.map(|c| {
                    let binary = self.binary.take().unwrap();
                    let cred = self.credentials.take().unwrap();
                    c.check_done(Err(e), cred, binary)
                });
            }
            Ok(()) => match self.signature.take() {
                Some(sig) => {
                    if let Err((e, d, s)) = self.verifier.verify(digest, sig) {
                        self.hash.replace(d);
                        self.signature.replace(s);
                        self.client.map(|c| {
                            let binary = self.binary.take().unwrap();
                            let cred = self.credentials.take().unwrap();
                            c.check_done(Err(e), cred, binary)
                        });
                    }
                }
                None => {
                    self.hash.replace(digest);
                    self.client.map(|c| {
                        let binary = self.binary.take().unwrap();
                        let cred = self.credentials.take().unwrap();
                        c.check_done(Err(ErrorCode::FAIL), cred, binary)
                    });
                }
            },
        }
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HASH_LEN, SIGNATURE_LEN>,
        H: hil::digest::DigestDataHash<'a, HASH_LEN>,
        const HASH_LEN: usize,
        const SIGNATURE_LEN: usize,
    > hil::digest::ClientVerify<HASH_LEN>
    for AppCheckerSignature<'a, S, H, HASH_LEN, SIGNATURE_LEN>
{
    fn verification_done(
        &self,
        _result: Result<bool, ErrorCode>,
        _compare: &'static mut [u8; HASH_LEN],
    ) {
        // Unused for this checker.
        // Needed to make the sha256 client work.
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HASH_LEN, SIGNATURE_LEN>,
        H: hil::digest::DigestDataHash<'a, HASH_LEN>,
        const HASH_LEN: usize,
        const SIGNATURE_LEN: usize,
    > hil::public_key_crypto::signature::ClientVerify<HASH_LEN, SIGNATURE_LEN>
    for AppCheckerSignature<'a, S, H, HASH_LEN, SIGNATURE_LEN>
{
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        hash: &'static mut [u8; HASH_LEN],
        signature: &'static mut [u8; SIGNATURE_LEN],
    ) {
        self.hash.replace(hash);
        self.signature.replace(signature);

        self.client.map(|c| {
            let binary = self.binary.take().unwrap();
            let cred = self.credentials.take().unwrap();
            let check_result = if result.unwrap_or(false) {
                Ok(CheckResult::Accept(None))
            } else {
                Ok(CheckResult::Pass)
            };

            c.check_done(check_result, cred, binary)
        });
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HASH_LEN, SIGNATURE_LEN>,
        H: hil::digest::DigestDataHash<'a, HASH_LEN>,
        const HASH_LEN: usize,
        const SIGNATURE_LEN: usize,
    > AppCredentialsPolicy<'static> for AppCheckerSignature<'a, S, H, HASH_LEN, SIGNATURE_LEN>
{
    fn require_credentials(&self) -> bool {
        true
    }

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'static [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'static [u8])> {
        self.credentials.set(credentials);

        if credentials.format() == self.credential_type {
            // Save the signature we are trying to compare with.
            self.signature.map(|b| {
                b.as_mut_slice()[..SIGNATURE_LEN]
                    .copy_from_slice(&credentials.data()[..SIGNATURE_LEN]);
            });

            // Add the process binary to compute the hash.
            self.hasher.clear_data();
            match self.hasher.add_data(SubSlice::new(binary)) {
                Ok(()) => Ok(()),
                Err((e, b)) => Err((e, credentials, b.take())),
            }
        } else {
            Err((ErrorCode::NOSUPPORT, credentials, binary))
        }
    }

    fn set_client(&self, client: &'static dyn AppCredentialsPolicyClient<'static>) {
        self.client.replace(client);
    }
}
