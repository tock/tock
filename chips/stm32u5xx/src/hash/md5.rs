// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! MD5 adapter for the hash core.

use kernel::ErrorCode;
use kernel::hil::digest;
use kernel::utilities::cells::OptionalCell;

use crate::hash::{hash::Hash, utils::HashClient};

const MD5_DIGEST_LEN: usize = 16;

pub struct Md5Adapter<'a> {
    hash: &'a Hash<'a>,
    client: OptionalCell<HashClient<'a, MD5_DIGEST_LEN>>,
}

impl<'a> Md5Adapter<'a> {
    pub fn new(hash: &'a Hash<'a>) -> Self {
        Self {
            hash,
            client: OptionalCell::empty(),
        }
    }
}

impl Md5Adapter<'_> {
    pub(crate) fn add_data_done(
        &self,
        result: Result<(), kernel::ErrorCode>,
        data: kernel::utilities::leasable_buffer::SubSlice<'static, u8>,
    ) {
        self.client.map(|client| {
            client.add_data_done(result, data);
        });
    }

    pub(crate) fn add_mut_data_done(
        &self,
        result: Result<(), kernel::ErrorCode>,
        data: kernel::utilities::leasable_buffer::SubSliceMut<'static, u8>,
    ) {
        self.client.map(|client| {
            client.add_mut_data_done(result, data);
        });
    }

    pub(crate) fn hash_done(
        &self,
        result: Result<(), kernel::ErrorCode>,
        digest: &'static mut [u8],
    ) {
        // terrible because of `unwrap()`
        //
        // TODO: handle conversion of references of arrays better
        self.client
            .map(|client| client.hash_done(result, digest.try_into().unwrap()));
    }

    pub(crate) fn verification_done(
        &self,
        result: Result<bool, kernel::ErrorCode>,
        compare: &'static mut [u8],
    ) {
        // terrible because of `unwrap()`
        //
        // TODO: handle conversion of references of arrays better
        self.client
            .map(|client| client.verification_done(result, compare.try_into().unwrap()));
    }
}

impl<'a> digest::DigestData<'a, MD5_DIGEST_LEN> for Md5Adapter<'a> {
    fn set_data_client(&'a self, client: &'a dyn kernel::hil::digest::ClientData<MD5_DIGEST_LEN>) {
        if let Some(HashClient::Split(_, hash, verify)) = self.client.get() {
            self.client
                .set(HashClient::Split(Some(client), hash, verify));
        } else {
            self.client.set(HashClient::Split(Some(client), None, None));
        }
    }

    fn add_data(
        &self,
        data: kernel::utilities::leasable_buffer::SubSlice<'static, u8>,
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            kernel::utilities::leasable_buffer::SubSlice<'static, u8>,
        ),
    > {
        self.hash.add_data(data)
    }

    fn add_mut_data(
        &self,
        data: kernel::utilities::leasable_buffer::SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            kernel::utilities::leasable_buffer::SubSliceMut<'static, u8>,
        ),
    > {
        self.hash.add_mut_data(data)
    }

    fn clear_data(&self) {
        self.hash.clear_data();
    }
}

impl<'a> digest::DigestHash<'a, MD5_DIGEST_LEN> for Md5Adapter<'a> {
    fn set_hash_client(&'a self, client: &'a dyn kernel::hil::digest::ClientHash<MD5_DIGEST_LEN>) {
        if let Some(HashClient::Split(data, _, verify)) = self.client.get() {
            self.client
                .set(HashClient::Split(data, Some(client), verify));
        } else {
            self.client.set(HashClient::Split(None, Some(client), None));
        }
    }

    fn run(
        &'a self,
        digest: &'static mut [u8; MD5_DIGEST_LEN],
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8; MD5_DIGEST_LEN])> {
        self.hash.run(digest).map_err(|(e, buf)| {
            // terrible because of `unwrap()`
            //
            // TODO: handle conversion of references of arrays better
            let correct_buf: &'static mut [u8; MD5_DIGEST_LEN] = buf.try_into().unwrap();
            (e, correct_buf)
        })
    }
}

impl<'a> digest::DigestVerify<'a, MD5_DIGEST_LEN> for Md5Adapter<'a> {
    fn set_verify_client(
        &'a self,
        client: &'a dyn kernel::hil::digest::ClientVerify<MD5_DIGEST_LEN>,
    ) {
        if let Some(HashClient::Split(data, hash, _)) = self.client.get() {
            self.client.set(HashClient::Split(data, hash, Some(client)));
        } else {
            self.client.set(HashClient::Split(None, None, Some(client)));
        }
    }

    fn verify(
        &'a self,
        compare: &'static mut [u8; MD5_DIGEST_LEN],
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8; MD5_DIGEST_LEN])> {
        self.hash.verify(compare).map_err(|(e, buf)| {
            // terrible because of `unwrap()`
            //
            // TODO: handle conversion of references of arrays better
            let correct_buf: &'static mut [u8; MD5_DIGEST_LEN] = buf.try_into().unwrap();
            (e, correct_buf)
        })
    }
}

impl<'a> digest::Digest<'a, MD5_DIGEST_LEN> for Md5Adapter<'a> {
    fn set_client(&'a self, client: &'a dyn digest::Client<MD5_DIGEST_LEN>) {
        self.client.set(HashClient::AllInOne(client));
    }
}

impl<'a> digest::DigestDataHash<'a, MD5_DIGEST_LEN> for Md5Adapter<'a> {
    fn set_client(&'a self, client: &'a dyn digest::ClientDataHash<MD5_DIGEST_LEN>) {
        self.client.set(HashClient::DataHasher(client));
    }
}

impl<'a> digest::DigestDataVerify<'a, MD5_DIGEST_LEN> for Md5Adapter<'a> {
    fn set_client(&'a self, client: &'a dyn digest::ClientDataVerify<MD5_DIGEST_LEN>) {
        self.client.set(HashClient::DataVerifier(client));
    }
}

impl digest::Sha256 for Md5Adapter<'_> {
    fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::HmacSha256 for Md5Adapter<'_> {
    fn set_mode_hmacsha256(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::Md5 for Md5Adapter<'_> {
    fn set_mode_md5(&self) -> Result<(), ErrorCode> {
        self.hash.set_mode_md5()
    }
}
impl digest::HmacMd5 for Md5Adapter<'_> {
    fn set_mode_hmacmd5(&self, key: &[u8]) -> Result<(), ErrorCode> {
        self.hash.set_mode_hmacmd5(key)
    }
}

impl digest::Sha1 for Md5Adapter<'_> {
    fn set_mode_sha1(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}
impl digest::HmacSha1 for Md5Adapter<'_> {
    fn set_mode_hmacsha1(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::Sha224 for Md5Adapter<'_> {
    fn set_mode_sha224(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}
impl digest::HmacSha224 for Md5Adapter<'_> {
    fn set_mode_hmacsha224(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::Sha384 for Md5Adapter<'_> {
    fn set_mode_sha384(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}
impl digest::HmacSha384 for Md5Adapter<'_> {
    fn set_mode_hmacsha384(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::Sha512 for Md5Adapter<'_> {
    fn set_mode_sha512(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}
impl digest::HmacSha512 for Md5Adapter<'_> {
    fn set_mode_hmacsha512(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}
