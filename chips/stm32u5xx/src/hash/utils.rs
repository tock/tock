// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! HASH utilities for leftovers, HMAC key, tracking message loading and handling different client types.

use core::cell::Cell;

use kernel::ErrorCode;
use kernel::hil::crypto::digest::{DigestClient, Md5, Sha1, Sha224, Sha256};
use kernel::processbuffer::ReadableProcessSlice;

#[derive(Debug, Clone, Copy)]
pub(crate) enum State {
    Add(usize, DataType),
    Run(bool, DataType),
}

impl State {
    pub(crate) fn update(self, updated_len: Option<usize>) -> Option<Self> {
        match (self, updated_len) {
            // (State::Run(true, DataType::Input), Some(len)) => {
            //     Some(State::Add(len, DataType::OuterKey))
            // }
            (State::Run(false, data_type), None) => Some(State::Run(true, data_type)),
            (State::Add(0, datatype), _) | (State::Add(_, datatype), Some(0)) => {
                Some(State::Run(false, datatype))
            }
            (State::Add(_, datatype), Some(len)) => Some(State::Add(len, datatype)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TransferMode {
    DirectStream,
    Dma(bool),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DataType {
    Input,
}

#[derive(Clone, Copy)]
pub(crate) enum HashClient<'a> {
    Md5(&'a dyn DigestClient<Md5>),
    Sha1(&'a dyn DigestClient<Sha1>),
    Sha224(&'a dyn DigestClient<Sha224>),
    Sha256(&'a dyn DigestClient<Sha256>),
}

impl HashClient<'_> {
    pub(crate) fn read_input(
        &self,
        sink: &mut dyn FnMut(&ReadableProcessSlice) -> Result<usize, ErrorCode>,
    ) -> Result<(), ErrorCode> {
        match self {
            HashClient::Md5(client) => client.read_input(sink),
            HashClient::Sha1(client) => client.read_input(sink),
            HashClient::Sha224(client) => client.read_input(sink),
            HashClient::Sha256(client) => client.read_input(sink),
        }
    }

    pub(crate) fn hash_done(&self, result: Result<(), ErrorCode>) {
        match self {
            HashClient::Md5(client) => client.digest_done(result),
            HashClient::Sha1(client) => client.digest_done(result),
            HashClient::Sha224(client) => client.digest_done(result),
            HashClient::Sha256(client) => client.digest_done(result),
        }
    }
}

pub(crate) struct Leftover {
    buffer: Cell<Option<u32>>,
    index: Cell<usize>,
}

impl Leftover {
    pub fn new() -> Self {
        Leftover {
            buffer: Cell::new(None),
            index: Cell::new(0),
        }
    }

    /// Add a new byte to the leftover buffer.
    pub fn add(&self, byte: u8) {
        if !self.is_full() {
            self.buffer.update(|buf| match buf {
                // Example of the operation
                // 01 -> 01xxxxxx
                // 02 -> 0201xxxx
                // 03 -> 030201xx
                // 04 -> 04030201
                Some(b) => Some(b >> 8 | (byte as u32).rotate_right(8)),
                None => Some((byte as u32).rotate_right(8)),
            });
        }

        self.index.update(|index| (index + 1) % 5);
    }

    /// Empty the buffer
    pub fn reset(&self) {
        self.buffer.take();
        self.index.take();
    }

    /// Return the contents of the buffer in little endian format
    pub fn to_le(&self) -> u32 {
        match self.buffer.take() {
            Some(b) => {
                let value = b >> (8 * self.bytes_left());
                self.index.update(|idx| idx.saturating_sub(4));
                value
            }
            None => 0,
        }
    }

    /// Returns how many bytes are left to fill the buffer up.
    pub fn bytes_left(&self) -> usize {
        4 - self.index.get()
    }

    /// Returns if the buffer full or not.
    pub fn is_full(&self) -> bool {
        self.index.get() == 4 && self.buffer.get().is_some()
    }

    /// Returns if the buffer empty or not.
    pub fn is_empty(&self) -> bool {
        self.buffer.get().is_none()
    }
}
