// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! Interrupt-driven STM32U5 message digests.
//!
//! The four algorithm interfaces share one engine. Input is copied into the
//! hardware FIFO in bounded batches; no client buffer is retained or used for
//! DMA. Callbacks run only from interrupts or deferred calls. The active client
//! is snapshotted when an operation is accepted, so later client registration
//! cannot redirect an in-flight operation.

use core::cell::Cell;

use super::regs::{CR, HashRegisters, IMR, SR, STR};
use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::crypto::digest::{Digest, DigestClient, Md5, Sha1, Sha224, Sha256};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};

#[derive(Clone, Copy)]
enum Client {
    Md5(&'static dyn DigestClient<Md5>),
    Sha1(&'static dyn DigestClient<Sha1>),
    Sha224(&'static dyn DigestClient<Sha224>),
    Sha256(&'static dyn DigestClient<Sha256>),
}

impl Client {
    fn read_input(self, input: &mut [u8]) -> Result<usize, ErrorCode> {
        let count = match self {
            Self::Md5(client) => client.read_input(input),
            Self::Sha1(client) => client.read_input(input),
            Self::Sha224(client) => client.read_input(input),
            Self::Sha256(client) => client.read_input(input),
        }?;
        if count == 0 || count > input.len() {
            Err(ErrorCode::SIZE)
        } else {
            Ok(count)
        }
    }

    fn done(self, result: Result<(), ErrorCode>) {
        match self {
            Self::Md5(client) => client.digest_done(result),
            Self::Sha1(client) => client.digest_done(result),
            Self::Sha224(client) => client.digest_done(result),
            Self::Sha256(client) => client.digest_done(result),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Feeding,
    Finishing,
}

/// One HASH peripheral implementing MD5, SHA-1, SHA-224, and SHA-256.
pub struct Hash {
    regs: StaticRef<HashRegisters>,
    md5: OptionalCell<&'static dyn DigestClient<Md5>>,
    sha1: OptionalCell<&'static dyn DigestClient<Sha1>>,
    sha224: OptionalCell<&'static dyn DigestClient<Sha224>>,
    sha256: OptionalCell<&'static dyn DigestClient<Sha256>>,
    active: OptionalCell<Client>,
    state: Cell<State>,
    remaining: Cell<usize>,
    partial: Cell<[u8; 4]>,
    partial_len: Cell<usize>,
    deferred_call: DeferredCall,
}

impl Hash {
    /// Create an engine; register its deferred call before accepting requests.
    pub fn new(regs: StaticRef<HashRegisters>) -> Self {
        Self {
            regs,
            md5: OptionalCell::empty(),
            sha1: OptionalCell::empty(),
            sha224: OptionalCell::empty(),
            sha256: OptionalCell::empty(),
            active: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            remaining: Cell::new(0),
            partial: Cell::new([0; 4]),
            partial_len: Cell::new(0),
            deferred_call: DeferredCall::new(),
        }
    }

    fn start(&self, len: usize, client: Option<Client>) -> Result<(), ErrorCode> {
        if self.state.get() != State::Idle {
            return Err(ErrorCode::BUSY);
        }
        let client = client.ok_or(ErrorCode::RESERVE)?;
        let algorithm = match client {
            Client::Md5(_) => CR::ALGO::MD5,
            Client::Sha1(_) => CR::ALGO::SHA_1,
            Client::Sha224(_) => CR::ALGO::SHA2_224,
            Client::Sha256(_) => CR::ALGO::SHA2_256,
        };
        self.regs.imr.set(0);
        self.regs
            .cr
            .write(algorithm + CR::DATATYPE::_8bitData + CR::INIT::SET);
        self.regs.sr.modify(SR::DCIS::CLEAR);
        self.regs.str.write(STR::NBLW.val(((len % 4) * 8) as u32));
        self.remaining.set(len);
        self.partial.set([0; 4]);
        self.partial_len.set(0);
        self.active.set(client);
        self.state.set(State::Feeding);
        self.deferred_call.set();
        Ok(())
    }

    fn feed(&self) {
        if self.state.get() != State::Feeding {
            return;
        }
        let Some(client) = self.active.get() else {
            return;
        };
        let remaining = self.remaining.get();
        if remaining != 0 {
            let words = (self.regs.sr.read(SR::NBWE) as usize).min(16);
            if words == 0 {
                self.regs.imr.write(IMR::DINIE::SET);
                return;
            }
            let mut input = [0; 64];
            let available = words * 4 - self.partial_len.get();
            let requested = remaining.min(available);
            let count = match client.read_input(&mut input[..requested]) {
                Ok(count) => count,
                Err(error) => {
                    self.finish(Err(error));
                    return;
                }
            };
            let mut partial = self.partial.get();
            let mut partial_len = self.partial_len.get();
            for byte in &input[..count] {
                partial[partial_len] = *byte;
                partial_len += 1;
                if partial_len == 4 {
                    self.regs.din.set(u32::from_le_bytes(partial));
                    partial = [0; 4];
                    partial_len = 0;
                }
            }
            self.partial.set(partial);
            self.partial_len.set(partial_len);
            self.remaining.set(remaining - count);
        }

        if self.remaining.get() == 0 {
            let partial_len = self.partial_len.get();
            if partial_len != 0 {
                if self.regs.sr.read(SR::NBWE) == 0 {
                    self.regs.imr.write(IMR::DINIE::SET);
                    return;
                }
                self.regs.din.set(u32::from_le_bytes(self.partial.get()));
            }
            self.state.set(State::Finishing);
            self.regs.imr.write(IMR::DCIE::SET);
            self.regs.str.modify(STR::DCAL::SET);
        } else if self.regs.sr.read(SR::NBWE) == 0 {
            self.regs.imr.write(IMR::DINIE::SET);
        } else {
            self.deferred_call.set();
        }
    }

    fn output<const LEN: usize>(&self) -> [u8; LEN] {
        let mut output = [0; LEN];
        for (bytes, register) in output
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(self.regs.hr.iter())
        {
            bytes.copy_from_slice(&register.get().to_be_bytes());
        }
        output
    }

    fn finish(&self, result: Result<(), ErrorCode>) {
        self.regs.imr.set(0);
        self.regs.cr.modify(CR::INIT::SET);
        self.regs.sr.modify(SR::DCIS::CLEAR);
        self.partial.set([0; 4]);
        self.partial_len.set(0);
        self.remaining.set(0);
        self.state.set(State::Idle);
        if let Some(client) = self.active.take() {
            client.done(result);
        }
    }

    pub(crate) fn handle_interrupt(&self) {
        self.regs.imr.set(0);
        match self.state.get() {
            State::Finishing if self.regs.sr.is_set(SR::DCIS) => {
                if let Some(client) = self.active.get() {
                    let result = match client {
                        Client::Md5(client) => client.write_digest(&self.output::<16>()),
                        Client::Sha1(client) => client.write_digest(&self.output::<20>()),
                        Client::Sha224(client) => client.write_digest(&self.output::<28>()),
                        Client::Sha256(client) => client.write_digest(&self.output::<32>()),
                    };
                    self.finish(result);
                }
            }
            State::Finishing => self.regs.imr.write(IMR::DCIE::SET),
            State::Feeding => self.feed(),
            State::Idle => (),
        }
    }
}

impl DeferredCallClient for Hash {
    fn handle_deferred_call(&'static self) {
        self.feed();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl Digest<Md5> for Hash {
    fn digest(&self, len: usize) -> Result<(), ErrorCode> {
        self.start(len, self.md5.get().map(Client::Md5))
    }

    fn set_client(&self, client: &'static dyn DigestClient<Md5>) {
        self.md5.set(client);
    }
}

impl Digest<Sha1> for Hash {
    fn digest(&self, len: usize) -> Result<(), ErrorCode> {
        self.start(len, self.sha1.get().map(Client::Sha1))
    }

    fn set_client(&self, client: &'static dyn DigestClient<Sha1>) {
        self.sha1.set(client);
    }
}

impl Digest<Sha224> for Hash {
    fn digest(&self, len: usize) -> Result<(), ErrorCode> {
        self.start(len, self.sha224.get().map(Client::Sha224))
    }

    fn set_client(&self, client: &'static dyn DigestClient<Sha224>) {
        self.sha224.set(client);
    }
}

impl Digest<Sha256> for Hash {
    fn digest(&self, len: usize) -> Result<(), ErrorCode> {
        self.start(len, self.sha256.get().map(Client::Sha256))
    }

    fn set_client(&self, client: &'static dyn DigestClient<Sha256>) {
        self.sha256.set(client);
    }
}
