// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! SHA-1 and SHA-256 using the HASH-AES (HASHCRYPT) engine of the LPC55S6x.
//!
//! The engine hashes 512-bit blocks (UM11126, Section 48.14.14). Runs of
//! full blocks that start at a word-aligned address are read by the engine
//! itself as an AHB bus master (`MEMADDR`/`MEMCTRL`), up to 2047 blocks per
//! run with a single interrupt at the end. Everything else (partial blocks,
//! unaligned data and the padding) is buffered and written by the CPU to
//! `INDATA` whenever `STATUS.WAITING` is set. Message padding is done in
//! software, so data can be added in chunks of any size and alignment.
//!
//! Unlike some other NXP parts, the LPC55S6x HASHCRYPT block cannot reload
//! an intermediate digest, so the running hash lives only in the hardware.
//! Only one hash can be in progress at a time, and nothing else (e.g. an AES
//! driver) may use the block while a hash is in progress.
//!
//! The same driver implements the digest HIL for both `DIGEST_LEN = 20`
//! (SHA-1) and `DIGEST_LEN = 32` (SHA-256). The algorithm is selected with
//! [`digest::Sha1::set_mode_sha1`] or [`digest::Sha256::set_mode_sha256`]
//! and defaults to SHA-256.
//!
//! Before use, the board must enable and reset the block with
//! `Clock::start_hashcrypt_clock`.

use core::cell::Cell;
use cortexm33::dma_fence::CortexMDmaFence;
use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::digest;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::dma_slice::DmaSubSliceMutImmut;
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut, SubSliceMutImmut};
use tock_registers_v0::interfaces::{Readable, Writeable};
use tock_registers_v0::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers_v0::{register_bitfields, register_structs};

use crate::clocks::Clock;

register_structs! {
    /// HASH-AES (HASHCRYPT)
    HashcryptRegisters {
        /// Control register to enable and operate Hash and Crypto
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        /// Indicates status of Hash peripheral
        (0x004 => status: ReadWrite<u32, STATUS::Register>),
        /// Write 1 to enable interrupts; reads back with which are set
        (0x008 => intenset: ReadWrite<u32, INT::Register>),
        /// Write 1 to clear interrupts
        (0x00C => intenclr: WriteOnly<u32, INT::Register>),
        /// Setup Master to access memory (if available)
        (0x010 => memctrl: ReadWrite<u32, MEMCTRL::Register>),
        /// Address to start memory access from (if available)
        (0x014 => memaddr: ReadWrite<u32>),
        (0x018 => _reserved0),
        /// Input of 16 words at a time to load up buffer
        (0x020 => indata: WriteOnly<u32>),
        /// Aliases of INDATA, so the processor can use store-multiple
        (0x024 => alias: [WriteOnly<u32>; 7]),
        /// Digest (or AES output) words
        (0x040 => digest: [ReadOnly<u32>; 8]),
        (0x060 => _reserved1),
        /// Crypto settings for AES and Salsa and ChaCha
        (0x080 => cryptcfg: ReadWrite<u32>),
        /// Returns the configuration of this block in this chip
        (0x084 => config: ReadOnly<u32>),
        (0x088 => _reserved2),
        /// Lock register allows locking to the current security level
        (0x08C => lock: ReadWrite<u32>),
        /// Random masks for ICB-AES
        (0x090 => mask: [WriteOnly<u32>; 4]),
        (0x0A0 => @END),
    }
}

register_bitfields![u32,
CTRL [
    /// The operational mode to use, or 0 if none.
    MODE OFFSET(0) NUMBITS(3) [
        Disabled = 0,
        Sha1 = 1,
        Sha256 = 2,
        Aes = 4,
        IcbAes = 5
    ],
    /// Written with 1 when starting a new Hash/Crypto. It self clears.
    NEW_HASH OFFSET(4) NUMBITS(1) [],
    /// Use DMA to fill INDATA.
    DMA_I OFFSET(8) NUMBITS(1) [],
    /// Use DMA to drain the digest/output.
    DMA_O OFFSET(9) NUMBITS(1) [],
    /// Swap bytes in the word for SHA hashing.
    HASHSWPB OFFSET(12) NUMBITS(1) []
],
STATUS [
    /// The block is waiting for 16 more words of data.
    WAITING OFFSET(0) NUMBITS(1) [],
    /// A digest is ready and no next block has been started.
    DIGEST OFFSET(1) NUMBITS(1) [],
    /// An error occurred (INDATA overrun or AHB bus error). Write 1 to clear.
    ERROR OFFSET(2) NUMBITS(1) [],
    /// A key is needed (AES).
    NEEDKEY OFFSET(4) NUMBITS(1) [],
    /// An IV/nonce is needed (AES).
    NEEDIV OFFSET(5) NUMBITS(1) [],
    /// ICB-AES block index.
    ICBIDX OFFSET(16) NUMBITS(6) []
],
INT [
    WAITING OFFSET(0) NUMBITS(1) [],
    DIGEST OFFSET(1) NUMBITS(1) [],
    ERROR OFFSET(2) NUMBITS(1) []
],
MEMCTRL [
    /// Enables AHB bus mastering.
    MASTER OFFSET(0) NUMBITS(1) [],
    /// Number of 512-bit blocks to copy starting at MEMADDR.
    COUNT OFFSET(16) NUMBITS(11) []
]
];

const HASHCRYPT_BASE: StaticRef<HashcryptRegisters> =
    unsafe { StaticRef::new(0x400A4000 as *const HashcryptRegisters) };

/// SHA-1 and SHA-256 block size in bytes.
const BLOCK_LEN: usize = 64;

/// Maximum number of blocks the engine reads in one bus-master run
/// (`MEMCTRL.COUNT` is 11 bits wide).
const MAX_MASTER_BLOCKS: usize = 2047;

/// Offset in the final block at which the 64-bit message length is stored.
const LENGTH_OFFSET: usize = BLOCK_LEN - 8;

const SHA1_DIGEST_LEN: usize = 20;
const SHA256_DIGEST_LEN: usize = 32;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Sha1,
    Sha256,
}

impl Mode {
    fn digest_len(self) -> usize {
        match self {
            Mode::Sha1 => SHA1_DIGEST_LEN,
            Mode::Sha256 => SHA256_DIGEST_LEN,
        }
    }
}

/// The digest length, and therefore the set of clients, that the outstanding
/// operation was requested through.
#[derive(Clone, Copy, PartialEq)]
enum Width {
    W20,
    W32,
}

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Idle,
    AddData,
    /// Final padding blocks are still being written.
    HashPad,
    /// All blocks are written; waiting for the final digest.
    HashDigest,
}

/// Clients and the digest buffer for one digest length.
struct Clients<'a, const L: usize> {
    data: OptionalCell<&'a dyn digest::ClientData<L>>,
    hash: OptionalCell<&'a dyn digest::ClientHash<L>>,
    verify: OptionalCell<&'a dyn digest::ClientVerify<L>>,
    /// Buffer passed to `run` or `verify`.
    digest: TakeCell<'static, [u8; L]>,
}

impl<const L: usize> Clients<'_, L> {
    fn new() -> Self {
        Clients {
            data: OptionalCell::empty(),
            hash: OptionalCell::empty(),
            verify: OptionalCell::empty(),
            digest: TakeCell::empty(),
        }
    }
}

pub struct Hashcrypt<'a> {
    registers: StaticRef<HashcryptRegisters>,
    clocks: &'a Clock,
    fence: CortexMDmaFence,
    deferred_call: DeferredCall,

    mode: Cell<Mode>,
    op: Cell<Op>,
    width: Cell<Width>,
    /// Whether the outstanding hash operation is a verification.
    verify: Cell<bool>,
    /// Result reported by the deferred call.
    deferred_result: Cell<Result<(), ErrorCode>>,

    /// Data passed to `add_data` or `add_mut_data`, while the CPU owns it.
    data: MapCell<SubSliceMutImmut<'static, u8>>,
    /// The same data while the engine is reading it as bus master. It is only
    /// moved back into `data` once the engine can no longer access it.
    dma_data: MapCell<DmaSubSliceMutImmut<'static, u8>>,
    /// Bytes not yet written to the engine. Never holds a full block between
    /// operations.
    block: MapCell<[u8; BLOCK_LEN]>,
    block_len: Cell<usize>,
    /// Total message length in bytes.
    total_len: Cell<u64>,
    /// Whether the engine has been started (NEW_HASH written) for this hash.
    started: Cell<bool>,
    /// Number of blocks of `data` the engine is reading as bus master, or 0.
    master_blocks: Cell<usize>,

    clients20: Clients<'a, SHA1_DIGEST_LEN>,
    clients32: Clients<'a, SHA256_DIGEST_LEN>,
}

impl<'a> Hashcrypt<'a> {
    pub fn new(clocks: &'a Clock, fence: CortexMDmaFence) -> Self {
        Hashcrypt {
            registers: HASHCRYPT_BASE,
            clocks,
            fence,
            deferred_call: DeferredCall::new(),
            mode: Cell::new(Mode::Sha256),
            op: Cell::new(Op::Idle),
            width: Cell::new(Width::W32),
            verify: Cell::new(false),
            deferred_result: Cell::new(Ok(())),
            data: MapCell::empty(),
            dma_data: MapCell::empty(),
            block: MapCell::new([0; BLOCK_LEN]),
            block_len: Cell::new(0),
            total_len: Cell::new(0),
            started: Cell::new(false),
            master_blocks: Cell::new(0),
            clients20: Clients::new(),
            clients32: Clients::new(),
        }
    }

    /// Whether a new operation can be started.
    fn check_ready(&self) -> Result<(), ErrorCode> {
        if !self.clocks.is_hashcrypt_clock_enabled() {
            Err(ErrorCode::OFF)
        } else if self.op.get() != Op::Idle {
            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }

    /// Start a new hash in the engine, if not already started.
    fn ensure_started(&self) {
        if !self.started.get() {
            let mode = match self.mode.get() {
                Mode::Sha1 => CTRL::MODE::Sha1,
                Mode::Sha256 => CTRL::MODE::Sha256,
            };
            // NEW_HASH must be set before switching from a previous mode,
            // otherwise the new mode does not work correctly.
            self.registers.ctrl.write(CTRL::NEW_HASH::SET);
            self.registers.ctrl.write(mode + CTRL::NEW_HASH::SET);
            self.started.set(true);
        }
    }

    /// Write the buffered (full) block to the engine. The engine must be
    /// waiting for data.
    fn write_block(&self) {
        let regs = self.registers;
        self.block.map(|block| {
            // The engine converts each little-endian word to big-endian, so
            // the bytes are hashed in memory order (UM11126, Section 48.13.3).
            for word in block.as_chunks::<4>().0 {
                regs.indata.set(u32::from_le_bytes(*word));
            }
        });
        self.block_len.set(0);
    }

    /// Move data into the block buffer and the engine until either all data
    /// has been consumed (returns `true`) or the engine cannot accept another
    /// block yet (returns `false`; the WAITING interrupt is then enabled).
    fn process_data(&self) -> bool {
        let regs = self.registers;
        loop {
            if self.block_len.get() == 0 {
                if let Some(blocks) = self.master_run_len() {
                    self.ensure_started();
                    if !regs.status.is_set(STATUS::WAITING) {
                        regs.intenset.write(INT::WAITING::SET + INT::ERROR::SET);
                        return false;
                    }
                    self.start_master(blocks);
                    return false;
                }
            }

            if self.block_len.get() == BLOCK_LEN {
                self.ensure_started();
                if !regs.status.is_set(STATUS::WAITING) {
                    regs.intenset.write(INT::WAITING::SET + INT::ERROR::SET);
                    return false;
                }
                self.write_block();
                continue;
            }

            let copied = self
                .data
                .map(|data| {
                    let offset = self.block_len.get();
                    let count = core::cmp::min(data.len(), BLOCK_LEN - offset);
                    self.block.map(|block| {
                        block[offset..offset + count].copy_from_slice(&data[..count]);
                    });
                    data.slice(count..);
                    count
                })
                .unwrap_or(0);

            if copied == 0 {
                regs.intenclr.write(INT::WAITING::SET);
                return true;
            }
            self.block_len.set(self.block_len.get() + copied);
            self.total_len.set(self.total_len.get() + copied as u64);
        }
    }

    /// Number of blocks of `data` that the engine can read as bus master: the
    /// full blocks at the start of the active range, if it is word aligned.
    fn master_run_len(&self) -> Option<usize> {
        self.data
            .map(|data| {
                let blocks = data.len() / BLOCK_LEN;
                if blocks > 0 && data.as_ptr().addr().is_multiple_of(4) {
                    Some(core::cmp::min(blocks, MAX_MASTER_BLOCKS))
                } else {
                    None
                }
            })
            .flatten()
    }

    /// Let the engine read `blocks` blocks from the start of `data`. The
    /// engine must be waiting for data.
    fn start_master(&self, blocks: usize) {
        let regs = self.registers;
        let Some(data) = self.data.take() else {
            return;
        };
        // Hand the buffer to the engine. Creating the `DmaSubSliceMutImmut`
        // makes its contents visible to the engine before the MMIO write that
        // starts the run. The engine only reads the buffer, and it is only
        // moved back into `self.data` once the engine has stopped.
        let dma_data = DmaSubSliceMutImmut::new(data, self.fence);
        regs.memaddr.set(dma_data.ptr_addr() as u32);
        self.dma_data.replace(dma_data);
        self.master_blocks.set(blocks);
        regs.intenclr.write(INT::WAITING::SET);
        regs.memctrl
            .write(MEMCTRL::MASTER::SET + MEMCTRL::COUNT.val(blocks as u32));
        // DIGEST is level sensitive, so enabling it after the run started
        // cannot miss the end of the run.
        regs.intenset.write(INT::DIGEST::SET + INT::ERROR::SET);
    }

    /// Account for a finished bus-master run. Returns `false` if the run is
    /// still in progress.
    fn finish_master(&self) -> bool {
        let regs = self.registers;
        // DIGEST is set once `COUNT` reaches 0 and the last block is hashed.
        if !regs.status.is_set(STATUS::DIGEST) || regs.memctrl.read(MEMCTRL::COUNT) != 0 {
            return false;
        }
        regs.intenclr.write(INT::DIGEST::SET);
        regs.memctrl.write(MEMCTRL::MASTER::CLEAR);
        let bytes = self.master_blocks.get() * BLOCK_LEN;
        self.reclaim_dma_data();
        self.data.map(|data| data.slice(bytes..));
        self.total_len.set(self.total_len.get() + bytes as u64);
        true
    }

    /// Take the data back from the engine. The bus-master run must have ended
    /// or been stopped, so that the engine no longer reads the buffer.
    fn reclaim_dma_data(&self) {
        self.master_blocks.set(0);
        if let Some(dma_data) = self.dma_data.take() {
            self.data.replace(dma_data.take());
        }
    }

    /// Write the padding blocks. Returns `false` if the engine cannot accept
    /// a block yet (the WAITING interrupt is then enabled), and `true` once
    /// the last block has been written (the DIGEST interrupt is then
    /// enabled).
    fn process_padding(&self) -> bool {
        let regs = self.registers;
        self.ensure_started();
        loop {
            if !regs.status.is_set(STATUS::WAITING) {
                regs.intenset.write(INT::WAITING::SET + INT::ERROR::SET);
                return false;
            }

            let len = self.block_len.get();
            if len > LENGTH_OFFSET {
                // The length does not fit in this block; it goes in an extra
                // block.
                self.block.map(|block| block[len..].fill(0));
                self.write_block();
                continue;
            }

            let bit_len = self.total_len.get().wrapping_mul(8);
            self.block.map(|block| {
                block[len..LENGTH_OFFSET].fill(0);
                block[LENGTH_OFFSET..].copy_from_slice(&bit_len.to_be_bytes());
            });
            self.write_block();
            regs.intenclr.write(INT::WAITING::SET);
            regs.intenset.write(INT::DIGEST::SET + INT::ERROR::SET);
            return true;
        }
    }

    /// Disable the engine and discard the hash state.
    fn reset_hash(&self) {
        let regs = self.registers;
        regs.intenclr
            .write(INT::WAITING::SET + INT::DIGEST::SET + INT::ERROR::SET);
        regs.memctrl.write(MEMCTRL::MASTER::CLEAR);
        regs.ctrl.write(CTRL::MODE::Disabled);
        regs.status.write(STATUS::ERROR::SET);
        // The engine is stopped, so a cancelled or failed run can return the
        // data to the client.
        self.reclaim_dma_data();
        self.block.map(|block| block.fill(0));
        self.block_len.set(0);
        self.total_len.set(0);
        self.started.set(false);
    }

    /// Check whether `len` bytes of data can be added.
    fn check_add_data(&self, len: usize) -> Result<(), ErrorCode> {
        self.check_ready()?;
        if len == 0 {
            return Err(ErrorCode::SIZE);
        }
        Ok(())
    }

    /// Start adding data. `check_add_data` must have succeeded.
    fn start_add_data(&self, data: SubSliceMutImmut<'static, u8>, width: Width) {
        self.op.set(Op::AddData);
        self.width.set(width);
        self.data.replace(data);
        if self.process_data() {
            // Everything fit in the block buffer or the engine; the callback
            // must not be issued from within this downcall.
            self.deferred_result.set(Ok(()));
            self.deferred_call.set();
        }
    }

    fn start_hash(&self, width: Width, verify: bool) -> Result<(), ErrorCode> {
        self.check_ready()?;
        let expected = match width {
            Width::W20 => SHA1_DIGEST_LEN,
            Width::W32 => SHA256_DIGEST_LEN,
        };
        if self.mode.get().digest_len() != expected {
            return Err(ErrorCode::NOSUPPORT);
        }

        self.op.set(Op::HashPad);
        self.width.set(width);
        self.verify.set(verify);

        // Append the '1' bit. The block buffer never holds a full block
        // between operations, so there is always room.
        let len = self.block_len.get();
        self.block.map(|block| block[len] = 0x80);
        self.block_len.set(len + 1);

        if self.process_padding() {
            self.op.set(Op::HashDigest);
        }
        Ok(())
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let status = regs.status.extract();

        if self.deferred_call.is_pending() {
            // The operation was cancelled by `clear_data` and the engine
            // reset; the deferred call reports the result.
            regs.intenclr
                .write(INT::WAITING::SET + INT::DIGEST::SET + INT::ERROR::SET);
            return;
        }

        if status.is_set(STATUS::ERROR) {
            if self.op.get() != Op::Idle {
                self.reset_hash();
                self.complete(Err(ErrorCode::FAIL));
            } else {
                self.reset_hash();
            }
            return;
        }

        match self.op.get() {
            Op::Idle => {
                regs.intenclr
                    .write(INT::WAITING::SET + INT::DIGEST::SET + INT::ERROR::SET);
            }
            Op::AddData => {
                let ready = if self.master_blocks.get() != 0 {
                    self.finish_master()
                } else {
                    status.is_set(STATUS::WAITING)
                };
                if ready && self.process_data() {
                    regs.intenclr.write(INT::ERROR::SET);
                    self.complete(Ok(()));
                }
            }
            Op::HashPad => {
                if status.is_set(STATUS::WAITING) && self.process_padding() {
                    self.op.set(Op::HashDigest);
                }
            }
            Op::HashDigest => {
                if status.is_set(STATUS::DIGEST) {
                    regs.intenclr.write(INT::DIGEST::SET + INT::ERROR::SET);
                    self.complete(Ok(()));
                }
            }
        }
    }

    /// Finish the outstanding operation and issue its callback.
    fn complete(&self, result: Result<(), ErrorCode>) {
        let op = self.op.get();
        self.op.set(Op::Idle);
        match op {
            Op::Idle => {}
            Op::AddData => {
                if let Some(data) = self.data.take() {
                    match self.width.get() {
                        Width::W20 => notify_data(&self.clients20, result, data),
                        Width::W32 => notify_data(&self.clients32, result, data),
                    }
                }
            }
            Op::HashPad | Op::HashDigest => match self.width.get() {
                Width::W20 => self.finish_hash(&self.clients20, result),
                Width::W32 => self.finish_hash(&self.clients32, result),
            },
        }
    }

    fn finish_hash<const L: usize>(&self, clients: &Clients<'a, L>, result: Result<(), ErrorCode>) {
        let Some(buf) = clients.digest.take() else {
            self.reset_hash();
            return;
        };

        if self.verify.get() {
            let result = result.map(|()| {
                // Accumulate differences so the comparison time does not
                // depend on the position of the first mismatch.
                let mut diff = 0;
                for (word, expected) in self.registers.digest.iter().zip(buf.as_chunks::<4>().0) {
                    for (a, b) in word.get().to_be_bytes().iter().zip(expected) {
                        diff |= a ^ b;
                    }
                }
                diff == 0
            });
            // The hash is complete; start a new one for the next operation.
            self.reset_hash();
            clients
                .verify
                .map(|client| client.verification_done(result, buf));
        } else {
            if result.is_ok() {
                for (word, out) in self.registers.digest.iter().zip(buf.as_chunks_mut::<4>().0) {
                    *out = word.get().to_be_bytes();
                }
            }
            self.reset_hash();
            clients.hash.map(|client| client.hash_done(result, buf));
        }
    }

    fn set_mode(&self, mode: Mode) -> Result<(), ErrorCode> {
        if self.op.get() != Op::Idle {
            return Err(ErrorCode::BUSY);
        }
        if self.mode.get() != mode {
            // The engine mode is fixed when a hash is started, so changing
            // the algorithm discards any data already added.
            self.reset_hash();
            self.mode.set(mode);
        }
        Ok(())
    }

    fn clear(&self) {
        self.reset_hash();
        if self.op.get() != Op::Idle {
            // Report the cancellation from a deferred call, as callbacks
            // must not be issued from within this downcall.
            self.deferred_result.set(Err(ErrorCode::CANCEL));
            self.deferred_call.set();
        }
    }
}

fn notify_data<const L: usize>(
    clients: &Clients<'_, L>,
    result: Result<(), ErrorCode>,
    data: SubSliceMutImmut<'static, u8>,
) {
    clients.data.map(move |client| match data {
        SubSliceMutImmut::Immutable(b) => client.add_data_done(result, b),
        SubSliceMutImmut::Mutable(b) => client.add_mut_data_done(result, b),
    });
}

impl DeferredCallClient for Hashcrypt<'_> {
    fn handle_deferred_call(&self) {
        self.complete(self.deferred_result.get());
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<'a> digest::DigestData<'a, SHA1_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_data_client(&'a self, client: &'a dyn digest::ClientData<SHA1_DIGEST_LEN>) {
        self.clients20.data.set(client);
    }

    fn add_data(
        &self,
        data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)> {
        if let Err(e) = self.check_add_data(data.len()) {
            return Err((e, data));
        }
        self.start_add_data(SubSliceMutImmut::Immutable(data), Width::W20);
        Ok(())
    }

    fn add_mut_data(
        &self,
        data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        if let Err(e) = self.check_add_data(data.len()) {
            return Err((e, data));
        }
        self.start_add_data(SubSliceMutImmut::Mutable(data), Width::W20);
        Ok(())
    }

    fn clear_data(&self) {
        self.clear();
    }
}

impl<'a> digest::DigestData<'a, SHA256_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_data_client(&'a self, client: &'a dyn digest::ClientData<SHA256_DIGEST_LEN>) {
        self.clients32.data.set(client);
    }

    fn add_data(
        &self,
        data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)> {
        if let Err(e) = self.check_add_data(data.len()) {
            return Err((e, data));
        }
        self.start_add_data(SubSliceMutImmut::Immutable(data), Width::W32);
        Ok(())
    }

    fn add_mut_data(
        &self,
        data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        if let Err(e) = self.check_add_data(data.len()) {
            return Err((e, data));
        }
        self.start_add_data(SubSliceMutImmut::Mutable(data), Width::W32);
        Ok(())
    }

    fn clear_data(&self) {
        self.clear();
    }
}

impl<'a> digest::DigestHash<'a, SHA1_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_hash_client(&'a self, client: &'a dyn digest::ClientHash<SHA1_DIGEST_LEN>) {
        self.clients20.hash.set(client);
    }

    fn run(
        &'a self,
        digest: &'static mut [u8; SHA1_DIGEST_LEN],
    ) -> Result<(), (ErrorCode, &'static mut [u8; SHA1_DIGEST_LEN])> {
        match self.start_hash(Width::W20, false) {
            Ok(()) => {
                self.clients20.digest.replace(digest);
                Ok(())
            }
            Err(e) => Err((e, digest)),
        }
    }
}

impl<'a> digest::DigestHash<'a, SHA256_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_hash_client(&'a self, client: &'a dyn digest::ClientHash<SHA256_DIGEST_LEN>) {
        self.clients32.hash.set(client);
    }

    fn run(
        &'a self,
        digest: &'static mut [u8; SHA256_DIGEST_LEN],
    ) -> Result<(), (ErrorCode, &'static mut [u8; SHA256_DIGEST_LEN])> {
        match self.start_hash(Width::W32, false) {
            Ok(()) => {
                self.clients32.digest.replace(digest);
                Ok(())
            }
            Err(e) => Err((e, digest)),
        }
    }
}

impl<'a> digest::DigestVerify<'a, SHA1_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_verify_client(&'a self, client: &'a dyn digest::ClientVerify<SHA1_DIGEST_LEN>) {
        self.clients20.verify.set(client);
    }

    fn verify(
        &'a self,
        compare: &'static mut [u8; SHA1_DIGEST_LEN],
    ) -> Result<(), (ErrorCode, &'static mut [u8; SHA1_DIGEST_LEN])> {
        match self.start_hash(Width::W20, true) {
            Ok(()) => {
                self.clients20.digest.replace(compare);
                Ok(())
            }
            Err(e) => Err((e, compare)),
        }
    }
}

impl<'a> digest::DigestVerify<'a, SHA256_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_verify_client(&'a self, client: &'a dyn digest::ClientVerify<SHA256_DIGEST_LEN>) {
        self.clients32.verify.set(client);
    }

    fn verify(
        &'a self,
        compare: &'static mut [u8; SHA256_DIGEST_LEN],
    ) -> Result<(), (ErrorCode, &'static mut [u8; SHA256_DIGEST_LEN])> {
        match self.start_hash(Width::W32, true) {
            Ok(()) => {
                self.clients32.digest.replace(compare);
                Ok(())
            }
            Err(e) => Err((e, compare)),
        }
    }
}

impl<'a> digest::Digest<'a, SHA1_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_client(&'a self, client: &'a dyn digest::Client<SHA1_DIGEST_LEN>) {
        self.clients20.data.set(client);
        self.clients20.hash.set(client);
        self.clients20.verify.set(client);
    }
}

impl<'a> digest::Digest<'a, SHA256_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_client(&'a self, client: &'a dyn digest::Client<SHA256_DIGEST_LEN>) {
        self.clients32.data.set(client);
        self.clients32.hash.set(client);
        self.clients32.verify.set(client);
    }
}

impl<'a> digest::DigestDataHash<'a, SHA1_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_client(&'a self, client: &'a dyn digest::ClientDataHash<SHA1_DIGEST_LEN>) {
        self.clients20.data.set(client);
        self.clients20.hash.set(client);
    }
}

impl<'a> digest::DigestDataHash<'a, SHA256_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_client(&'a self, client: &'a dyn digest::ClientDataHash<SHA256_DIGEST_LEN>) {
        self.clients32.data.set(client);
        self.clients32.hash.set(client);
    }
}

impl<'a> digest::DigestDataVerify<'a, SHA1_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_client(&'a self, client: &'a dyn digest::ClientDataVerify<SHA1_DIGEST_LEN>) {
        self.clients20.data.set(client);
        self.clients20.verify.set(client);
    }
}

impl<'a> digest::DigestDataVerify<'a, SHA256_DIGEST_LEN> for Hashcrypt<'a> {
    fn set_client(&'a self, client: &'a dyn digest::ClientDataVerify<SHA256_DIGEST_LEN>) {
        self.clients32.data.set(client);
        self.clients32.verify.set(client);
    }
}

impl digest::Sha1 for Hashcrypt<'_> {
    fn set_mode_sha1(&self) -> Result<(), ErrorCode> {
        self.set_mode(Mode::Sha1)
    }
}

impl digest::Sha256 for Hashcrypt<'_> {
    fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
        self.set_mode(Mode::Sha256)
    }
}

impl digest::Md5 for Hashcrypt<'_> {
    fn set_mode_md5(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::Sha224 for Hashcrypt<'_> {
    fn set_mode_sha224(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::Sha384 for Hashcrypt<'_> {
    fn set_mode_sha384(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::Sha512 for Hashcrypt<'_> {
    fn set_mode_sha512(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::HmacMd5 for Hashcrypt<'_> {
    fn set_mode_hmacmd5(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::HmacSha1 for Hashcrypt<'_> {
    fn set_mode_hmacsha1(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::HmacSha224 for Hashcrypt<'_> {
    fn set_mode_hmacsha224(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::HmacSha256 for Hashcrypt<'_> {
    fn set_mode_hmacsha256(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::HmacSha384 for Hashcrypt<'_> {
    fn set_mode_hmacsha384(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl digest::HmacSha512 for Hashcrypt<'_> {
    fn set_mode_hmacsha512(&self, _key: &[u8]) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}
