// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Software implementation of HMAC-SHA256.

use core::cell::Cell;

use kernel::hil;
use kernel::hil::digest::DigestData;
use kernel::hil::digest::{DigestAlgorithm, HmacSha256Hmac, Sha256Hash};
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSlice;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::leasable_buffer::SubSliceMutImmut;
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    InnerHashAddKeyPending,
    InnerHashAddKey,
    InnerHashAddData,
    InnerHash,
    OuterHashAddKey,
    OuterHashAddHash,
    OuterHash,
}

#[derive(Copy, Clone)]
pub enum RunMode {
    Hash,
    Verify,
}

/// Value to XOR the key with on the inner hash.
const INNER_PAD_BYTE: u8 = 0x36;
/// Value to XOR the key with on the outer hash.
const OUTER_PAD_BYTE: u8 = 0x5c;

const SHA_BLOCK_LEN_BYTES: usize = 64;

pub struct HmacSha256Software<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> {
    /// SHA256 hasher implementation.
    sha256: &'a S,
    /// The current operation for the internal state machine in this capsule.
    state: Cell<State>,
    /// The current mode of operation as requested by a call to either
    /// [`DigestHash::run`] or [`DigestVerify::verify`].
    mode: Cell<RunMode>,
    /// Location to store incoming temporarily before we are able to pass it to
    /// the hasher.
    input_data: OptionalCell<SubSliceMutImmut<'static, u8>>,
    /// Static buffer to store the key and to pass to the hasher. This must be
    /// at least `SHA_BLOCK_LEN_BYTES` bytes.
    data_buffer: TakeCell<'static, [u8]>,
    /// Storage buffer to keep a copy of the key. This allows us to keep it
    /// persistent if the user wants to do multiple HMACs with the same key.
    key_buffer: MapCell<[u8; SHA_BLOCK_LEN_BYTES]>,
    /// Hash digest buffer used by the SHA256 hasher.
    sha_buffer: MapCell<&'static mut Sha256Hash>,
    /// Holding cell for the output digest buffer while we calculate the HMAC.
    digest_buffer: MapCell<&'static mut HmacSha256Hmac>,
    /// Buffer-slot used for a _verify_ operation. When not active, this
    /// contains a buffer to place the current digest in. On a call to `verify`,
    /// where the digest to compare to is provided in another buffer, this
    /// buffer is swapped into this TakeCell. When the operation completes, we
    /// swap them back and compare:
    verify_buffer: MapCell<&'static mut HmacSha256Hmac>,
    /// Clients for callbacks.
    // error[E0658]: cannot cast `dyn kernel::hil::digest::Client<32>` to `dyn ClientData<32>`, trait upcasting coercion is experimental
    // data_client: OptionalCell<&'a dyn hil::digest::ClientData<HmacSha256Hmac>>,
    // hash_client: OptionalCell<&'a dyn hil::digest::ClientHash<HmacSha256Hmac>>,
    // verify_client: OptionalCell<&'a dyn hil::digest::ClientVerify<HmacSha256Hmac>>,
    client: OptionalCell<&'a dyn hil::digest::Client<HmacSha256Hmac>>,
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> HmacSha256Software<'a, S> {
    pub fn new(
        sha256: &'a S,
        data_buffer: &'static mut [u8],
        sha_buffer: &'static mut Sha256Hash,
        verify_buffer: &'static mut HmacSha256Hmac,
    ) -> Self {
        Self {
            sha256,
            state: Cell::new(State::Idle),
            mode: Cell::new(RunMode::Hash),
            input_data: OptionalCell::empty(),
            data_buffer: TakeCell::new(data_buffer),
            key_buffer: MapCell::new([0; SHA_BLOCK_LEN_BYTES]),
            sha_buffer: MapCell::new(sha_buffer),
            digest_buffer: MapCell::empty(),
            verify_buffer: MapCell::new(verify_buffer),
            // data_client: OptionalCell::empty(),
            // hash_client: OptionalCell::empty(),
            // verify_client: OptionalCell::empty(),
            client: OptionalCell::empty(),
        }
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> hil::digest::DigestData<'a, HmacSha256Hmac>
    for HmacSha256Software<'a, S>
{
    fn add_data(
        &self,
        data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)> {
        match self.state.get() {
            State::InnerHashAddKeyPending => {
                // We need to write the key before we write the data.
                if let Some(data_buf) = self.data_buffer.take() {
                    self.key_buffer.map(|key_buf| {
                        // Copy the key XOR with inner pad (0x36).
                        for i in 0..64 {
                            data_buf[i] = key_buf[i] ^ INNER_PAD_BYTE;
                        }
                    });

                    let mut lease_buf = SubSliceMut::new(data_buf);
                    lease_buf.slice(0..64);

                    match self.sha256.add_mut_data(lease_buf) {
                        Ok(()) => {
                            self.state.set(State::InnerHashAddKey);
                            // Save the incoming data to add to the hasher
                            // on the next iteration.
                            self.input_data.set(SubSliceMutImmut::Immutable(data));
                            Ok(())
                        }
                        Err((e, leased_data_buf)) => {
                            self.data_buffer.replace(leased_data_buf.take());
                            Err((e, data))
                        }
                    }
                } else {
                    Err((ErrorCode::BUSY, data))
                }
            }

            State::InnerHashAddData => {
                // In this state the hasher is ready to take more input data so
                // we can provide more input data. This is the only state after
                // setting the key we can accept new data in.
                self.sha256.add_data(data)
            }

            State::Idle => {
                // We need a key before we can accept data, so we must return
                // error here. `OFF` is the closest error to this issue so we
                // return that.
                Err((ErrorCode::OFF, data))
            }

            _ => {
                // Any other state we cannot accept new data.
                Err((ErrorCode::BUSY, data))
            }
        }
    }

    fn add_mut_data(
        &self,
        data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        match self.state.get() {
            State::InnerHashAddKeyPending => {
                // We need to write the key before we write the data.

                if let Some(data_buf) = self.data_buffer.take() {
                    // Copy the key XOR with inner pad (0x36).
                    self.key_buffer.map(|key_buf| {
                        // Copy the key XOR with inner pad (0x36).
                        for i in 0..64 {
                            data_buf[i] = key_buf[i] ^ INNER_PAD_BYTE;
                        }
                    });

                    let mut lease_buf = SubSliceMut::new(data_buf);
                    lease_buf.slice(0..64);

                    match self.sha256.add_mut_data(lease_buf) {
                        Ok(()) => {
                            self.state.set(State::InnerHashAddKey);
                            // Save the incoming data to add to the hasher
                            // on the next iteration.
                            self.input_data.set(SubSliceMutImmut::Mutable(data));
                            Ok(())
                        }
                        Err((e, leased_data_buf)) => {
                            self.data_buffer.replace(leased_data_buf.take());
                            Err((e, data))
                        }
                    }
                } else {
                    Err((ErrorCode::BUSY, data))
                }
            }

            State::InnerHashAddData => {
                // In this state the hasher is ready to take more input data so
                // we can provide more input data. This is the only state after
                // setting the key we can accept new data in.
                self.sha256.add_mut_data(data)
            }

            State::Idle => {
                // We need a key before we can accept data, so we must return
                // error here. `OFF` is the closest error to this issue so we
                // return that.
                Err((ErrorCode::OFF, data))
            }

            _ => {
                // Any other state we cannot accept new data.
                Err((ErrorCode::BUSY, data))
            }
        }
    }

    fn clear_data(&self) {
        self.state.set(State::Idle);
        self.sha256.clear_data();
    }

    fn set_data_client(&'a self, _client: &'a dyn hil::digest::ClientData<HmacSha256Hmac>) {
        // self.data_client.set(client);
        unimplemented!()
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> hil::digest::DigestHash<'a, HmacSha256Hmac>
    for HmacSha256Software<'a, S>
{
    fn run(
        &'a self,
        digest: &'static mut HmacSha256Hmac,
    ) -> Result<(), (ErrorCode, &'static mut HmacSha256Hmac)> {
        // User called run, we start with the inner hash.
        self.state.set(State::InnerHash);
        self.mode.set(RunMode::Hash);

        // Call the SHA256 hasher, saving the hmac digest, or return the proper
        // error.
        let sha_digest = self.sha_buffer.take().unwrap();
        match self.sha256.run(sha_digest) {
            Ok(()) => {
                self.digest_buffer.replace(digest);
                Ok(())
            }
            Err((e, d)) => {
                self.sha_buffer.replace(d);
                Err((e, digest))
            }
        }
    }

    fn set_hash_client(&'a self, _client: &'a dyn hil::digest::ClientHash<HmacSha256Hmac>) {
        // self.hash_client.set(client);
        unimplemented!()
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>>
    hil::digest::DigestVerify<'a, HmacSha256Hmac> for HmacSha256Software<'a, S>
{
    fn verify(
        &'a self,
        compare: &'static mut HmacSha256Hmac,
    ) -> Result<(), (ErrorCode, &'static mut HmacSha256Hmac)> {
        // User called verify, we start with the inner hash.
        self.state.set(State::InnerHash);
        self.mode.set(RunMode::Verify);

        let sha_digest = self.sha_buffer.take().unwrap();
        match self.sha256.run(sha_digest) {
            Ok(()) => {
                // Swap the `compare` buffer into `self.verify_buffer`,
                // and then save our other Hmac buffer in
                // `self.digest_buffer`.
                let digest = self.verify_buffer.replace(compare).unwrap();
                self.digest_buffer.replace(digest);
                Ok(())
            }
            Err((e, d)) => {
                self.sha_buffer.replace(d);
                Err((e, compare))
            }
        }
    }

    fn set_verify_client(&'a self, _client: &'a dyn hil::digest::ClientVerify<HmacSha256Hmac>) {
        // self.verify_client.set(client);
        unimplemented!()
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>>
    hil::digest::DigestDataHash<'a, HmacSha256Hmac> for HmacSha256Software<'a, S>
{
    fn set_client(&'a self, _client: &'a dyn hil::digest::ClientDataHash<HmacSha256Hmac>) {
        // self.data_client.set(client);
        // self.hash_client.set(client);
        unimplemented!()
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> hil::digest::Digest<'a, HmacSha256Hmac>
    for HmacSha256Software<'a, S>
{
    fn set_client(&'a self, client: &'a dyn hil::digest::Client<HmacSha256Hmac>) {
        // self.data_client.set(client);
        // self.hash_client.set(client);
        // self.verify_client.set(client);
        self.client.set(client);
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> hil::digest::ClientData<Sha256Hash>
    for HmacSha256Software<'a, S>
{
    fn add_data_done(&self, result: Result<(), ErrorCode>, data: SubSlice<'static, u8>) {
        // This callback is only used for the user to pass in additional data
        // for the HMAC, we do not use `add_data()` internally in this capsule
        // so we can just directly issue the callback.
        // self.data_client.map(|client| {
        self.client.map(|client| {
            client.add_data_done(result, data);
        });
    }

    fn add_mut_data_done(&self, result: Result<(), ErrorCode>, data: SubSliceMut<'static, u8>) {
        if result.is_err() {
            // self.data_client.map(|client| {
            self.client.map(|client| {
                client.add_mut_data_done(result, data);
            });
        } else {
            match self.state.get() {
                State::InnerHashAddKey => {
                    self.data_buffer.replace(data.take());

                    // We just added the key, so we can now add the stored data.
                    self.input_data.take().map(|in_data| match in_data {
                        SubSliceMutImmut::Mutable(buffer) => {
                            match self.sha256.add_mut_data(buffer) {
                                Ok(()) => {
                                    self.state.set(State::InnerHashAddData);
                                }
                                Err((e, leased_data_buf)) => {
                                    self.clear_data();
                                    // self.data_client.map(|c| {
                                    self.client.map(|c| {
                                        c.add_mut_data_done(Err(e), leased_data_buf);
                                    });
                                }
                            }
                        }
                        SubSliceMutImmut::Immutable(buffer) => match self.sha256.add_data(buffer) {
                            Ok(()) => {
                                self.state.set(State::InnerHashAddData);
                            }
                            Err((e, leased_data_buf)) => {
                                self.clear_data();
                                self.client.map(|c| {
                                    c.add_data_done(Err(e), leased_data_buf);
                                });
                            }
                        },
                    });
                }
                State::OuterHashAddKey => {
                    // We just added the key, now we add the result of the first
                    // hash.
                    self.sha_buffer.take().map(|sha_buf| {
                        let data_buf = data.take();

                        // Copy the digest result into our data buffer. We must
                        // use our data buffer because it does not have a fixed
                        // size and we can use it with `SubSliceMut`.
                        data_buf[..32].copy_from_slice(&sha_buf.as_slice()[..32]);
                        self.sha_buffer.replace(sha_buf);

                        let mut lease_buf = SubSliceMut::new(data_buf);
                        lease_buf.slice(0..32);

                        match self.sha256.add_mut_data(lease_buf) {
                            Ok(()) => {
                                self.state.set(State::OuterHashAddHash);
                            }
                            Err((e, leased_data_buf)) => {
                                self.data_buffer.replace(leased_data_buf.take());
                                self.clear_data();
                                // self.data_client.map(|c| {
                                self.client.map(|c| {
                                    self.digest_buffer.take().map(|digest_buf| {
                                        c.hash_done(Err(e), digest_buf);
                                    })
                                });
                            }
                        }
                    });
                }
                State::OuterHashAddHash => {
                    // We've now added both the key and the result of the first
                    // hash, so we can run the second hash to get our HMAC.
                    self.data_buffer.replace(data.take());

                    self.sha_buffer
                        .take()
                        .map(|sha_buf| match self.sha256.run(sha_buf) {
                            Ok(()) => {
                                self.state.set(State::OuterHash);
                            }
                            Err((e, s)) => {
                                self.sha_buffer.replace(s);
                                self.clear_data();
                                self.digest_buffer.take().map(|digest| {
                                    // self.data_client.map(|c| {
                                    self.client.map(|c| {
                                        c.hash_done(Err(e), digest);
                                    });
                                });
                            }
                        });
                }
                _ => {
                    // In other states, we can just issue the callback like
                    // normal.
                    // self.data_client.map(|client| {
                    self.client.map(|client| {
                        client.add_mut_data_done(Ok(()), data);
                    });
                }
            }
        }
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> hil::digest::ClientHash<Sha256Hash>
    for HmacSha256Software<'a, S>
{
    fn hash_done(&self, result: Result<(), ErrorCode>, digest: &'static mut Sha256Hash) {
        self.sha_buffer.replace(digest);

        let hash_done_error =
            |error: Result<(), ErrorCode>, error_digest: &'static mut HmacSha256Hmac| {
                match self.mode.get() {
                    RunMode::Hash => {
                        // self.hash_client.map(|c| {
                        self.client.map(|c| {
                            c.hash_done(error, error_digest);
                        })
                    }
                    RunMode::Verify => {
                        // Also swap back the verify_buffer, and return the original
                        // buffer to the client:
                        let compare = self.verify_buffer.replace(error_digest).unwrap();
                        // self.verify_client.map(|c| {
                        self.client.map(|c| {
                            // Convert to Result<bool, ErrorCode>
                            c.verification_done(error.map(|()| false), compare);
                        })
                    }
                }
            };

        if result.is_err() {
            // If hashing fails, we have to propagate that error up with a
            // callback.
            self.clear_data();
            self.digest_buffer.take().map(|digest_buffer| {
                hash_done_error(result, digest_buffer);
            });
        } else {
            match self.state.get() {
                State::InnerHash => {
                    // Completed inner hash, now work on outer hash.
                    self.sha256.clear_data();

                    self.data_buffer.take().map(|data_buf| {
                        self.key_buffer.map(|key_buf| {
                            // Copy the key XOR with outer pad (0x5c).
                            for i in 0..64 {
                                data_buf[i] = key_buf[i] ^ OUTER_PAD_BYTE;
                            }
                        });

                        let mut lease_buf = SubSliceMut::new(data_buf);
                        lease_buf.slice(0..64);

                        match self.sha256.add_mut_data(lease_buf) {
                            Ok(()) => {
                                self.state.set(State::OuterHashAddKey);
                            }
                            Err((e, leased_data_buf)) => {
                                // If we cannot add data, we need to replace the
                                // buffer and issue a callback with an error.
                                self.data_buffer.replace(leased_data_buf.take());
                                self.clear_data();
                                self.digest_buffer.take().map(|digest_buffer| {
                                    hash_done_error(Err(e), digest_buffer);
                                });
                            }
                        }
                    });
                }

                State::OuterHash => {
                    self.sha_buffer.map(|sha_buf| {
                        self.digest_buffer.take().map(|digest_buf| {
                            // Copy final digest from the SHA buffer to the HMAC
                            // digest buffer.
                            digest_buf.as_mut_slice()[..32]
                                .copy_from_slice(&sha_buf.as_slice()[..32]);

                            match self.mode.get() {
                                RunMode::Hash => {
                                    // self.hash_client.map(|c| {
                                    self.client.map(|c| {
                                        c.hash_done(Ok(()), digest_buf);
                                    });
                                }

                                RunMode::Verify => {
                                    let compare = self.verify_buffer.take().unwrap();
                                    let res = compare == digest_buf;
                                    self.verify_buffer.replace(digest_buf);
                                    // self.verify_client.map(|c| {
                                    self.client.map(|c| {
                                        c.verification_done(Ok(res), compare);
                                    });
                                }
                            }
                        });
                    });
                }
                _ => {}
            }
        }
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> hil::digest::ClientVerify<Sha256Hash>
    for HmacSha256Software<'a, S>
{
    fn verification_done(
        &self,
        _result: Result<bool, ErrorCode>,
        _compare: &'static mut Sha256Hash,
    ) {
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> hil::digest::HmacSha256
    for HmacSha256Software<'a, S>
{
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if key.len() > SHA_BLOCK_LEN_BYTES {
            // Key size must be no longer than the internal block size (which is
            // 64 bytes).
            Err(ErrorCode::SIZE)
        } else {
            self.key_buffer.map_or(Err(ErrorCode::FAIL), |key_buf| {
                // Save the key in our key buffer.
                for i in 0..64 {
                    key_buf[i] = *key.get(i).unwrap_or(&0);
                }

                // Mark that we have the key pending which we can add once we
                // get additional data to add. We can't add the key in the
                // underlying hash now because we don't have a callback to use,
                // so we have to just store the key. We need to use the key
                // again anyway, so this is ok.
                self.state.set(State::InnerHashAddKeyPending);
                Ok(())
            })
        }
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> hil::digest::HmacSha384
    for HmacSha256Software<'a, S>
{
    fn set_mode_hmacsha384(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl<'a, S: hil::digest::DigestDataHash<'a, Sha256Hash>> hil::digest::HmacSha512
    for HmacSha256Software<'a, S>
{
    fn set_mode_hmacsha512(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}
