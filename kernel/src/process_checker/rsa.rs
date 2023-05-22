// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! RSA credential checkers for checking process credentials.

use crate::hil;
use crate::process::{Process, ShortID};
use crate::process_checker::{AppCredentialsChecker, AppUniqueness};
use crate::process_checker::{CheckResult, Client, Compress};
use crate::utilities::cells::OptionalCell;
use crate::utilities::cells::TakeCell;
use crate::utilities::leasable_buffer::{LeasableBuffer, LeasableMutableBuffer};
use crate::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfFooterV2CredentialsType;

pub trait Sha256Hasher<'a>:
    hil::digest::DigestDataHash<'a, 32_usize> + hil::digest::Sha256
{
}
impl<'a, T: hil::digest::DigestDataHash<'a, 32_usize> + hil::digest::Sha256> Sha256Hasher<'a>
    for T
{
}

pub trait Rsa2048Verifier<'a, const H: usize, const S: usize>:
    hil::public_key_crypto::signature::SignatureVerify<'static, 32, 256>
{
}
impl<'a, T: hil::public_key_crypto::signature::SignatureVerify<'static, 32, 256>>
    Rsa2048Verifier<'a, 32, 256> for T
{
}

/// Checker that validates correct RSA2048 credentials using a SHA256 hash.
pub struct AppCheckerRsa2048 {
    hasher: &'static dyn Sha256Hasher<'static>,
    verifier: &'static dyn Rsa2048Verifier<'static, 32, 256>,
    hash: TakeCell<'static, [u8; 32]>,
    signature: TakeCell<'static, [u8; 256]>,
    client: OptionalCell<&'static dyn Client<'static>>,
    credentials: OptionalCell<TbfFooterV2Credentials>,
    binary: OptionalCell<&'static [u8]>,
}

impl AppCheckerRsa2048 {
    pub fn new(
        hasher: &'static dyn Sha256Hasher<'static>,
        verifier: &'static dyn Rsa2048Verifier<'static, 32, 256>,
        hash_buffer: &'static mut [u8; 32],
        signature_buffer: &'static mut [u8; 256],
    ) -> AppCheckerRsa2048 {
        Self {
            hasher,
            verifier,
            hash: TakeCell::new(hash_buffer),
            signature: TakeCell::new(signature_buffer),
            client: OptionalCell::empty(),
            credentials: OptionalCell::empty(),
            binary: OptionalCell::empty(),
        }
    }
}

impl hil::digest::ClientData<32_usize> for AppCheckerRsa2048 {
    fn add_mut_data_done(
        &self,
        _result: Result<(), ErrorCode>,
        _data: LeasableMutableBuffer<'static, u8>,
    ) {
    }

    fn add_data_done(&self, result: Result<(), ErrorCode>, data: LeasableBuffer<'static, u8>) {
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

impl hil::digest::ClientHash<32_usize> for AppCheckerRsa2048 {
    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut [u8; 32]) {
        match result {
            Err(_e) => {}
            Ok(()) => {
                self.signature
                    .take()
                    .map(|s| match self.verifier.verify(digest, s) {
                        Err((_e, _, _)) => {}
                        Ok(()) => {}
                    });
            }
        }
    }
}

impl<'a> hil::digest::ClientVerify<32_usize> for AppCheckerRsa2048 {
    fn verification_done(
        &self,
        _result: Result<bool, ErrorCode>,
        _compare: &'static mut [u8; 32_usize],
    ) {
        // Unused for this checker.
        // Needed to make the sha256 client work.
    }
}

impl hil::public_key_crypto::signature::ClientVerify<32_usize, 256_usize> for AppCheckerRsa2048 {
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        hash: &'static mut [u8; 32],
        signature: &'static mut [u8; 256],
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

impl AppCredentialsChecker<'static> for AppCheckerRsa2048 {
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
            TbfFooterV2CredentialsType::Rsa2048 => {
                // Save the signature we are trying to compare with.
                self.signature.map(|b| {
                    for i in 0..256 {
                        b[i] = credentials.data()[i];
                    }
                });

                // Add the process binary to compute the hash.
                self.hasher.clear_data();
                match self.hasher.add_data(LeasableBuffer::new(binary)) {
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

impl AppUniqueness for AppCheckerRsa2048 {
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

impl Compress for AppCheckerRsa2048 {
    fn to_short_id(&self, credentials: &TbfFooterV2Credentials) -> ShortID {
        // Should never trigger, as we only approve RSA3072 and RSA4096 credentials.
        let data = credentials.data();
        if data.len() < 4 {
            return ShortID::LocallyUnique;
        }
        let id: u32 = 0x8000000 as u32
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
