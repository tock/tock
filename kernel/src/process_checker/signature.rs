// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Signature credential checker for checking process credentials.

use crate::hil;
use crate::process::{Process, ShortID};
use crate::process_checker::{AppCredentialsChecker, AppUniqueness};
use crate::process_checker::{CheckResult, Client, Compress};
use crate::utilities::cells::MapCell;
use crate::utilities::cells::OptionalCell;
use crate::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use crate::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfFooterV2CredentialsType;

/// Checker that validates a correct signature credential.
///
/// This checker provides the scaffolding on top of a hasher (`&H`) and a
/// verifier (`&S`) for a given `TbfFooterV2CredentialsType`.
///
/// This assumes the `TbfFooterV2CredentialsType` data format only contains the
/// signature (i.e. the data length of the credential in the TBF footer is the
/// same as `SL`).
pub struct AppCheckerSignature<
    'a,
    S: hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
    H: hil::digest::DigestDataHash<'a, HL>,
    const HL: usize,
    const SL: usize,
> {
    hasher: &'a H,
    verifier: &'a S,
    hash: MapCell<&'static mut [u8; HL]>,
    signature: MapCell<&'static mut [u8; SL]>,
    client: OptionalCell<&'static dyn Client<'static>>,
    credential_type: TbfFooterV2CredentialsType,
    credentials: OptionalCell<TbfFooterV2Credentials>,
    binary: OptionalCell<&'static [u8]>,
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: hil::digest::DigestDataHash<'a, HL>,
        const HL: usize,
        const SL: usize,
    > AppCheckerSignature<'a, S, H, HL, SL>
{
    pub fn new(
        hasher: &'a H,
        verifier: &'a S,
        hash_buffer: &'static mut [u8; HL],
        signature_buffer: &'static mut [u8; SL],
        credential_type: TbfFooterV2CredentialsType,
    ) -> AppCheckerSignature<'a, S, H, HL, SL> {
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
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: hil::digest::DigestDataHash<'a, HL>,
        const HL: usize,
        const SL: usize,
    > hil::digest::ClientData<HL> for AppCheckerSignature<'a, S, H, HL, SL>
{
    fn add_mut_data_done(&self, _result: Result<(), ErrorCode>, _data: SubSliceMut<'static, u8>) {}

    fn add_data_done(&self, result: Result<(), ErrorCode>, data: SubSlice<'static, u8>) {
        // We added the binary data to the hasher, now we can compute the hash.
        match result {
            Err(_e) => {}
            Ok(()) => {
                self.binary.set(data.take());

                self.hash.take().map(|h| match self.hasher.run(h) {
                    Err((_e, _)) => {}
                    Ok(()) => {}
                });
            }
        }
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: hil::digest::DigestDataHash<'a, HL>,
        const HL: usize,
        const SL: usize,
    > hil::digest::ClientHash<HL> for AppCheckerSignature<'a, S, H, HL, SL>
{
    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut [u8; HL]) {
        match result {
            Err(_e) => {}
            Ok(()) => match self.signature.take() {
                Some(sig) => match self.verifier.verify(digest, sig) {
                    Err((_e, _, _)) => {}
                    Ok(()) => {}
                },
                None => {}
            },
        }
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: hil::digest::DigestDataHash<'a, HL>,
        const HL: usize,
        const SL: usize,
    > hil::digest::ClientVerify<HL> for AppCheckerSignature<'a, S, H, HL, SL>
{
    fn verification_done(&self, _result: Result<bool, ErrorCode>, _compare: &'static mut [u8; HL]) {
        // Unused for this checker.
        // Needed to make the sha256 client work.
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: hil::digest::DigestDataHash<'a, HL>,
        const HL: usize,
        const SL: usize,
    > hil::public_key_crypto::signature::ClientVerify<HL, SL>
    for AppCheckerSignature<'a, S, H, HL, SL>
{
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        hash: &'static mut [u8; HL],
        signature: &'static mut [u8; SL],
    ) {
        self.hash.replace(hash);
        self.signature.replace(signature);

        self.client.map(|c| {
            let binary = self.binary.take().unwrap();
            let cred = self.credentials.take().unwrap();
            let check_result = if result.unwrap_or(false) {
                Ok(CheckResult::Accept)
            } else {
                Ok(CheckResult::Pass)
            };

            c.check_done(check_result, cred, binary)
        });
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: hil::digest::DigestDataHash<'a, HL>,
        const HL: usize,
        const SL: usize,
    > AppCredentialsChecker<'static> for AppCheckerSignature<'a, S, H, HL, SL>
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
                b.as_mut_slice()[..SL].copy_from_slice(&credentials.data()[..SL]);
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

    fn set_client(&self, client: &'static dyn Client<'static>) {
        self.client.replace(client);
    }
}

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: hil::digest::DigestDataHash<'a, HL>,
        const HL: usize,
        const SL: usize,
    > AppUniqueness for AppCheckerSignature<'a, S, H, HL, SL>
{
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

impl<
        'a,
        S: hil::public_key_crypto::signature::SignatureVerify<'static, HL, SL>,
        H: hil::digest::DigestDataHash<'a, HL>,
        const HL: usize,
        const SL: usize,
    > Compress for AppCheckerSignature<'a, S, H, HL, SL>
{
    fn to_short_id(&self, _process: &dyn Process, credentials: &TbfFooterV2Credentials) -> ShortID {
        let data = credentials.data();
        if data.len() < 4 {
            // Should never trigger, as we only approve signature credentials.
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
