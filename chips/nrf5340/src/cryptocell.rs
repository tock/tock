// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Arm CryptoCell-312 of the nRF5340 application core: TRNG, SHA-256 and
//! AES-128 (ECB, CBC and CTR).
//!
//! [`Cc312`] drives the hardware synchronously. AES and SHA-256 data goes
//! through the CPU data registers rather than the CryptoCell DMA, so no
//! buffer is ever shared with hardware. [`CryptoCell`] puts the Tock entropy,
//! digest and symmetric encryption HILs on top: the TRNG is interrupt driven,
//! and AES and SHA-256 finish within the call and report through a deferred
//! call.
//!
//! The CryptoCell accepts only secure, privileged accesses, so this works only
//! in secure state.
//!
//! Register offsets and sequences follow Arm's open-source CryptoCell-312
//! runtime and cc3xx driver, and the Nordic MDK.

use core::cell::Cell;
use core::sync::atomic::{AtomicBool, Ordering};
use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::digest;
use kernel::hil::entropy::{self, Continue};
use kernel::hil::symmetric_encryption::{self, AES_BLOCK_SIZE, AES128};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut};
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly, register_structs};

register_structs! {
    /// Nordic's wrapper around the CryptoCell.
    pub CryptoCellRegisters {
        (0x000 => _reserved0),
        (0x500 => enable: ReadWrite<u32>),
        (0x504 => @END),
    }
}

register_structs! {
    /// The CryptoCell-312 registers this driver uses.
    pub Cc312Registers {
        (0x000 => _reserved0),
        (0x100 => rng_imr: ReadWrite<u32>),
        (0x104 => rng_isr: ReadOnly<u32>),
        (0x108 => rng_icr: WriteOnly<u32>),
        (0x10C => trng_config: ReadWrite<u32>),
        (0x110 => _reserved1),
        (0x114 => ehr_data: [ReadOnly<u32>; 6]),
        (0x12C => rnd_source_enable: ReadWrite<u32>),
        (0x130 => sample_cnt1: ReadWrite<u32>),
        (0x134 => _reserved2),
        (0x138 => trng_debug_control: ReadWrite<u32>),
        (0x13C => _reserved3),
        (0x140 => rng_sw_reset: WriteOnly<u32>),
        (0x144 => _reserved4),
        (0x1BC => rst_bits_counter: WriteOnly<u32>),
        (0x1C0 => _reserved5),
        (0x1C4 => rng_clk_enable: WriteOnly<u32>),
        (0x1C8 => _reserved6),
        (0x400 => aes_key_0: [WriteOnly<u32>; 8]),
        (0x420 => _reserved7),
        (0x440 => aes_iv_0: [ReadWrite<u32>; 4]),
        (0x450 => _reserved8),
        (0x460 => aes_ctr_0: [ReadWrite<u32>; 4]),
        (0x470 => _reserved9),
        (0x4BC => aes_remaining_bytes: ReadWrite<u32>),
        (0x4C0 => aes_control: ReadWrite<u32>),
        (0x4C4 => _reserved10),
        (0x640 => hash_h: [ReadWrite<u32>; 8]),
        (0x660 => _reserved11),
        (0x684 => auto_hw_padding: WriteOnly<u32>),
        (0x688 => hash_xor_din: ReadWrite<u32>),
        (0x68C => _reserved12),
        (0x6A4 => hash_sel_aes_mac: WriteOnly<u32>),
        (0x6A8 => _reserved13),
        (0x7C0 => hash_control: ReadWrite<u32>),
        (0x7C4 => hash_pad_en: ReadWrite<u32>),
        (0x7C8 => hash_pad_cfg: ReadWrite<u32>),
        (0x7CC => hash_cur_len: [ReadWrite<u32>; 2]),
        (0x7D4 => _reserved14),
        (0x810 => aes_clk_enable: WriteOnly<u32>),
        (0x814 => _reserved15),
        (0x818 => hash_clk_enable: WriteOnly<u32>),
        (0x81C => _reserved16),
        (0x820 => dma_clk_enable: WriteOnly<u32>),
        (0x824 => _reserved24),
        (0x900 => crypto_ctl: WriteOnly<u32>),
        (0x904 => _reserved17),
        (0x910 => crypto_busy: ReadOnly<u32>),
        (0x914 => _reserved18),
        (0x91C => hash_busy: ReadOnly<u32>),
        (0x920 => _reserved19),
        (0xA00 => host_irr: ReadOnly<u32>),
        (0xA04 => host_imr: ReadWrite<u32>),
        (0xA08 => host_icr: WriteOnly<u32>),
        (0xA0C => host_rgf_endian: ReadWrite<u32>),
        (0xA10 => _reserved20),
        (0xC00 => din_buffer: WriteOnly<u32>),
        (0xC04 => _reserved21),
        (0xC48 => din_cpu_data_size: WriteOnly<u32>),
        (0xC4C => din_write_align: WriteOnly<u32>),
        (0xC50 => _reserved22),
        (0xD00 => dout_buffer: ReadOnly<u32>),
        (0xD04 => _reserved23),
        (0xD50 => dout_fifo_empty: ReadOnly<u32>),
        (0xD54 => @END),
    }
}

// CRYPTO_CTL engines.
const ENGINE_BYPASS: u32 = 0;
const ENGINE_AES: u32 = 1;
const ENGINE_HASH: u32 = 7;

// HOST_IRR and HOST_ICR bit for the TRNG.
const HOST_RNG: u32 = 1 << 10;

// RNG_ISR bits.
const RNG_EHR_VALID: u32 = 1 << 0;
const RNG_ERRORS: u32 = 0b1110;

// TRNG_CONFIG SOP_SEL, which a CryptoCell without a PRNG must set.
const TRNG_SOP_SEL: u32 = 1 << 2;
// Clock cycles between TRNG samples, as Arm's CC312 configuration uses. It
// must not be 0xFFFF, the reset value that tells when the reset has finished.
const TRNG_SAMPLE_RATE: u32 = 0x1337;

// HASH_CONTROL value for SHA-256.
const HASH_SHA256: u32 = 2;
// HASH_PAD_CFG DO_PAD, which pads an empty message.
const HASH_DO_PAD: u32 = 1 << 2;

const SHA256_IV: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Largest number of bytes for one CPU data transfer, kept a multiple of the
/// AES and SHA-256 block sizes.
const MAX_TRANSFER: usize = 0xFFC0;

#[derive(Clone, Copy, PartialEq)]
pub enum AesMode {
    Ecb = 0,
    Cbc = 1,
    Ctr = 2,
}

/// SHA-256 state between calls: the chaining value and the bytes hashed.
#[derive(Clone, Copy)]
pub struct Sha256State {
    h: [u32; 8],
    len: u64,
}

impl Sha256State {
    pub const fn new() -> Self {
        Self {
            h: SHA256_IV,
            len: 0,
        }
    }
}

fn words(bytes: &[u8; 16]) -> [u32; 4] {
    let mut out = [0; 4];
    for (word, chunk) in out.iter_mut().zip(bytes.as_chunks::<4>().0) {
        *word = u32::from_le_bytes(*chunk);
    }
    out
}

/// Synchronous access to the CryptoCell-312.
pub struct Cc312 {
    wrapper: StaticRef<CryptoCellRegisters>,
    registers: StaticRef<Cc312Registers>,
    enabled: Cell<bool>,
}

impl Cc312 {
    pub const fn new(
        wrapper: StaticRef<CryptoCellRegisters>,
        registers: StaticRef<Cc312Registers>,
    ) -> Self {
        Self {
            wrapper,
            registers,
            enabled: Cell::new(false),
        }
    }

    /// Power the CryptoCell up and set it little-endian, once.
    fn enable(&self) {
        if !self.enabled.get() {
            self.wrapper.enable.set(1);
            self.registers.host_rgf_endian.set(0);
            self.registers.host_icr.set(0xFFFF_FFFF);
            self.select(ENGINE_BYPASS);
            self.enabled.set(true);
        }
    }

    fn select(&self, engine: u32) {
        while self.registers.crypto_busy.get() != 0 {}
        self.registers.crypto_ctl.set(engine);
        while self.registers.crypto_busy.get() != 0 {}
    }

    /// Write `len` bytes from `data` to the data input, zero-padding the last
    /// word.
    fn write_din(&self, data: &[u8]) {
        self.registers.din_cpu_data_size.set(data.len() as u32);
        let mut chunks = data.chunks(4).peekable();
        while let Some(chunk) = chunks.next() {
            let mut word = [0u8; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            if chunks.peek().is_none() && chunk.len() < 4 {
                self.registers.din_write_align.set(1);
            }
            self.registers.din_buffer.set(u32::from_le_bytes(word));
        }
    }

    /// AES-128 on `data` in place, whose length must be a multiple of 16.
    /// `iv` is the CBC IV or CTR counter, and the returned value continues the
    /// message.
    pub fn aes128(
        &self,
        mode: AesMode,
        encrypt: bool,
        key: &[u8; 16],
        iv: &[u8; 16],
        data: &mut [u8],
    ) -> [u8; 16] {
        self.enable();
        let r = &self.registers;
        // The CPU data registers go through the DMA block, so it needs its
        // clock too.
        r.aes_clk_enable.set(1);
        r.dma_clk_enable.set(1);
        self.select(ENGINE_AES);
        r.aes_remaining_bytes.set(0);
        // Direction and mode before the key.
        r.aes_control
            .set(((mode as u32) << 2) | u32::from(!encrypt));
        for (register, word) in r.aes_key_0.iter().zip(words(key)) {
            register.set(word);
        }
        let chaining = match mode {
            AesMode::Ecb => None,
            AesMode::Cbc => Some(&r.aes_iv_0),
            AesMode::Ctr => Some(&r.aes_ctr_0),
        };
        if let Some(chaining) = chaining {
            for (register, word) in chaining.iter().zip(words(iv)) {
                register.set(word);
            }
        }

        for transfer in data.chunks_mut(MAX_TRANSFER) {
            r.aes_remaining_bytes.set(transfer.len() as u32);
            r.din_cpu_data_size.set(transfer.len() as u32);
            for block in transfer.as_chunks_mut::<AES_BLOCK_SIZE>().0 {
                for word in block.as_chunks::<4>().0 {
                    r.din_buffer.set(u32::from_le_bytes(*word));
                }
                for word in block.as_chunks_mut::<4>().0 {
                    while r.dout_fifo_empty.get() & 1 != 0 {}
                    word.copy_from_slice(&r.dout_buffer.get().to_le_bytes());
                }
            }
        }

        let mut next = *iv;
        if let Some(chaining) = chaining {
            for (bytes, register) in next.as_chunks_mut::<4>().0.iter_mut().zip(chaining.iter()) {
                bytes.copy_from_slice(&register.get().to_le_bytes());
                register.set(0);
            }
        }
        r.aes_control.set(0);
        self.select(ENGINE_BYPASS);
        r.dma_clk_enable.set(0);
        r.aes_clk_enable.set(0);
        next
    }

    /// Select the hash engine with `state` loaded.
    fn sha256_load(&self, state: &Sha256State) {
        self.enable();
        let r = &self.registers;
        r.hash_clk_enable.set(1);
        r.dma_clk_enable.set(1);
        self.select(ENGINE_HASH);
        r.hash_sel_aes_mac.set(0);
        r.hash_pad_en.set(1);
        r.auto_hw_padding.set(0);
        r.hash_control.set(HASH_SHA256);
        r.hash_cur_len[0].set(state.len as u32);
        r.hash_cur_len[1].set((state.len >> 32) as u32);
        r.hash_xor_din.set(0);
        // The chaining value goes in from H7 down to H0.
        for (register, word) in r.hash_h.iter().zip(state.h).rev() {
            register.set(word);
        }
    }

    fn sha256_unload(&self) {
        let r = &self.registers;
        for register in r.hash_h.iter() {
            register.set(0);
        }
        r.hash_pad_cfg.set(0);
        r.auto_hw_padding.set(0);
        self.select(ENGINE_BYPASS);
        r.dma_clk_enable.set(0);
        r.hash_clk_enable.set(0);
    }

    /// Hash `blocks`, whose length must be a multiple of 64, into `state`.
    pub fn sha256_blocks(&self, state: &mut Sha256State, blocks: &[u8]) {
        self.sha256_load(state);
        let r = &self.registers;
        for transfer in blocks.chunks(MAX_TRANSFER) {
            self.write_din(transfer);
            while r.hash_busy.get() != 0 {}
        }
        for (word, register) in state.h.iter_mut().zip(r.hash_h.iter()) {
            *word = register.get();
        }
        state.len += blocks.len() as u64;
        self.sha256_unload();
    }

    /// Hash the final `tail`, at most 64 bytes and empty only for an empty
    /// message, and return the digest.
    pub fn sha256_finish(&self, state: &Sha256State, tail: &[u8]) -> [u8; 32] {
        self.sha256_load(state);
        let r = &self.registers;
        if tail.is_empty() {
            r.hash_pad_cfg.set(HASH_DO_PAD);
            r.din_cpu_data_size.set(0);
        } else {
            r.auto_hw_padding.set(1);
            self.write_din(tail);
        }
        while r.hash_busy.get() != 0 {}
        let mut digest = [0; 32];
        for (bytes, register) in digest
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(r.hash_h.iter())
        {
            bytes.copy_from_slice(&register.get().to_be_bytes());
        }
        self.sha256_unload();
        digest
    }

    /// Start the TRNG. See `trng_ready` for how its samples are noticed.
    pub fn trng_start(&self) {
        self.enable();
        self.trng_restart(0, TRNG_SAMPLE_RATE);
    }

    fn trng_restart(&self, rosc: u32, rate: u32) {
        let r = &self.registers;
        r.rng_clk_enable.set(1);
        r.rng_sw_reset.set(1);
        // The reset has finished when the sample count keeps a write.
        loop {
            r.rng_clk_enable.set(1);
            r.sample_cnt1.set(rate);
            if r.sample_cnt1.get() == rate {
                break;
            }
        }
        r.rnd_source_enable.set(0);
        r.rng_icr.set(0xFFFF_FFFF);
        // The status bits latch whether or not they are masked.
        r.rng_imr.set(0x3F);
        r.trng_config.set(TRNG_SOP_SEL | rosc);
        r.trng_debug_control.set(0);
        r.rnd_source_enable.set(1);
    }

    pub fn trng_stop(&self) {
        let r = &self.registers;
        r.rnd_source_enable.set(0);
        r.rng_icr.set(0xFFFF_FFFF);
        r.host_icr.set(HOST_RNG);
        r.rng_clk_enable.set(0);
    }

    /// Take a TRNG result: the 192 random bits, or `None` after a failed health
    /// test, which restarts the TRNG with the next ring oscillator.
    pub fn trng_take(&self) -> Option<[u32; 6]> {
        let r = &self.registers;
        let status = r.rng_isr.get();
        let result = if status & RNG_ERRORS != 0 {
            let rosc = r.trng_config.get() & 3;
            let rate = r.sample_cnt1.get();
            r.rnd_source_enable.set(0);
            r.rst_bits_counter.set(1);
            let (rosc, rate) = match rosc {
                3 => (0, rate.saturating_mul(2).min(0xFFFE)),
                n => (n + 1, rate),
            };
            self.trng_restart(rosc, rate);
            None
        } else if status & RNG_EHR_VALID != 0 {
            let rosc = r.trng_config.get() & 3;
            let rate = r.sample_cnt1.get();
            r.rnd_source_enable.set(0);
            let mut bits = [0; 6];
            for (word, register) in bits.iter_mut().zip(r.ehr_data.iter()) {
                *word = register.get();
            }
            // Collect the next sample from a fresh start, as Arm's runtime
            // does.
            self.trng_restart(rosc, rate);
            Some(bits)
        } else {
            None
        };
        // The host's copy of the RNG status is unused; keep it clear.
        r.host_icr.set(HOST_RNG);
        result
    }
}

/// An operation that has finished and waits for its deferred callback.
enum Done {
    Aes(Option<&'static mut [u8]>, &'static mut [u8]),
    AddData(SubSlice<'static, u8>),
    AddMutData(SubSliceMut<'static, u8>),
    Hash(&'static mut [u8; 32]),
    Verify(&'static mut [u8; 32], bool),
}

/// Set while the TRNG collects entropy.
static TRNG_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Whether the TRNG collects entropy. The CryptoCell stops while the core
/// sleeps, so [`crate::chip::Nrf5340`] does not sleep then.
pub(crate) fn trng_active() -> bool {
    TRNG_ACTIVE.load(Ordering::Relaxed)
}

/// Whether the TRNG has a result. Its interrupt to the host latches only once
/// after a start, so [`crate::chip::Nrf5340`] polls this instead and then
/// services the CryptoCell interrupt.
pub(crate) fn trng_ready() -> bool {
    trng_active() && crate::chip::CC312_BASE.rng_isr.get() & (RNG_EHR_VALID | RNG_ERRORS) != 0
}

/// The CryptoCell as Tock entropy, SHA-256 and AES-128 drivers.
pub struct CryptoCell<'a> {
    cc: Cc312,
    deferred_call: DeferredCall,
    done: OptionalCell<Done>,

    rng_client: OptionalCell<&'a dyn entropy::Client32>,
    rng_running: Cell<bool>,
    // The first sample after a start is dropped.
    rng_warm: Cell<bool>,

    aes_client: OptionalCell<&'a dyn symmetric_encryption::Client<'a>>,
    aes_key: Cell<[u8; 16]>,
    aes_iv: Cell<[u8; 16]>,
    aes_next_iv: Cell<[u8; 16]>,
    aes_mode: Cell<AesMode>,
    aes_encrypt: Cell<bool>,

    data_client: OptionalCell<&'a dyn digest::ClientData<32>>,
    hash_client: OptionalCell<&'a dyn digest::ClientHash<32>>,
    verify_client: OptionalCell<&'a dyn digest::ClientVerify<32>>,
    sha_state: Cell<Sha256State>,
    // Data not hashed yet: always the last 1 to 64 bytes of the message, so
    // that the final transfer is never empty.
    sha_pending: MapCell<[u8; 64]>,
    sha_pending_len: Cell<usize>,
}

impl CryptoCell<'_> {
    pub fn new(cc: Cc312) -> Self {
        Self {
            cc,
            deferred_call: DeferredCall::new(),
            done: OptionalCell::empty(),
            rng_client: OptionalCell::empty(),
            rng_running: Cell::new(false),
            rng_warm: Cell::new(false),
            aes_client: OptionalCell::empty(),
            aes_key: Cell::new([0; 16]),
            aes_iv: Cell::new([0; 16]),
            aes_next_iv: Cell::new([0; 16]),
            aes_mode: Cell::new(AesMode::Ecb),
            aes_encrypt: Cell::new(true),
            data_client: OptionalCell::empty(),
            hash_client: OptionalCell::empty(),
            verify_client: OptionalCell::empty(),
            sha_state: Cell::new(Sha256State::new()),
            sha_pending: MapCell::new([0; 64]),
            sha_pending_len: Cell::new(0),
        }
    }

    pub fn handle_interrupt(&self) {
        let Some(bits) = self.cc.trng_take() else {
            return;
        };
        if !self.rng_running.get() {
            return;
        }
        if !self.rng_warm.replace(true) {
            return;
        }
        let mut entropy = bits.into_iter();
        let more = self.rng_client.map_or(Continue::Done, |client| {
            client.entropy_available(&mut entropy, Ok(()))
        });
        if more == Continue::Done {
            self.rng_running.set(false);
            TRNG_ACTIVE.store(false, Ordering::Relaxed);
            self.cc.trng_stop();
        }
    }

    fn finish(&self, done: Done) {
        self.done.set(done);
        self.deferred_call.set();
    }

    /// Add `data` to the message, hashing every whole block but the last.
    fn sha_add(&self, mut data: &[u8]) {
        self.sha_pending.map(|pending| {
            let mut state = self.sha_state.get();
            let mut len = self.sha_pending_len.get();
            while !data.is_empty() {
                if len == 64 {
                    self.cc.sha256_blocks(&mut state, pending);
                    len = 0;
                    // Hash the whole blocks of `data` but one directly.
                    let direct = (data.len() - 1) / 64 * 64;
                    self.cc.sha256_blocks(&mut state, &data[..direct]);
                    data = &data[direct..];
                }
                let n = data.len().min(64 - len);
                pending[len..len + n].copy_from_slice(&data[..n]);
                len += n;
                data = &data[n..];
            }
            self.sha_state.set(state);
            self.sha_pending_len.set(len);
        });
    }

    fn sha_finish(&self) -> [u8; 32] {
        let digest = self.sha_pending.map_or([0; 32], |pending| {
            self.cc.sha256_finish(
                &self.sha_state.get(),
                &pending[..self.sha_pending_len.get()],
            )
        });
        self.sha_state.set(Sha256State::new());
        self.sha_pending_len.set(0);
        digest
    }
}

impl DeferredCallClient for CryptoCell<'_> {
    fn handle_deferred_call(&self) {
        match self.done.take() {
            Some(Done::Aes(source, dest)) => {
                self.aes_client
                    .map(|client| client.crypt_done(source, dest));
            }
            Some(Done::AddData(data)) => {
                self.data_client
                    .map(|client| client.add_data_done(Ok(()), data));
            }
            Some(Done::AddMutData(data)) => {
                self.data_client
                    .map(|client| client.add_mut_data_done(Ok(()), data));
            }
            Some(Done::Hash(digest)) => {
                self.hash_client
                    .map(|client| client.hash_done(Ok(()), digest));
            }
            Some(Done::Verify(compare, matches)) => {
                self.verify_client
                    .map(|client| client.verification_done(Ok(matches), compare));
            }
            None => (),
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<'a> entropy::Entropy32<'a> for CryptoCell<'a> {
    fn get(&self) -> Result<(), ErrorCode> {
        if !self.rng_running.replace(true) {
            self.rng_warm.set(false);
            TRNG_ACTIVE.store(true, Ordering::Relaxed);
            self.cc.trng_start();
        }
        Ok(())
    }

    fn cancel(&self) -> Result<(), ErrorCode> {
        if self.rng_running.replace(false) {
            TRNG_ACTIVE.store(false, Ordering::Relaxed);
            self.cc.trng_stop();
        }
        Ok(())
    }

    fn set_client(&'a self, client: &'a dyn entropy::Client32) {
        self.rng_client.set(client);
    }
}

impl<'a> symmetric_encryption::AES<'a, AES128> for CryptoCell<'a> {
    fn enable(&self) {}

    fn disable(&self) {}

    fn set_client(&'a self, client: &'a dyn symmetric_encryption::Client<'a>) {
        self.aes_client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        self.aes_key
            .set(key.try_into().map_err(|_| ErrorCode::INVAL)?);
        Ok(())
    }

    fn set_iv(&self, iv: &[u8]) -> Result<(), ErrorCode> {
        let iv: [u8; 16] = iv.try_into().map_err(|_| ErrorCode::INVAL)?;
        self.aes_iv.set(iv);
        self.aes_next_iv.set(iv);
        Ok(())
    }

    fn start_message(&self) {
        if self.done.is_none() {
            self.aes_next_iv.set(self.aes_iv.get());
        }
    }

    fn crypt(
        &self,
        source: Option<&'static mut [u8]>,
        dest: &'static mut [u8],
        start_index: usize,
        stop_index: usize,
    ) -> Option<(
        Result<(), ErrorCode>,
        Option<&'static mut [u8]>,
        &'static mut [u8],
    )> {
        if self.done.is_some() {
            return Some((Err(ErrorCode::BUSY), source, dest));
        }
        let valid = start_index <= stop_index
            && stop_index <= dest.len()
            && (stop_index - start_index).is_multiple_of(AES_BLOCK_SIZE)
            && source
                .as_ref()
                .is_none_or(|source| source.len() == stop_index - start_index);
        if !valid {
            return Some((Err(ErrorCode::INVAL), source, dest));
        }
        let data = &mut dest[start_index..stop_index];
        if let Some(source) = source.as_ref() {
            data.copy_from_slice(source);
        }
        let next = self.cc.aes128(
            self.aes_mode.get(),
            self.aes_encrypt.get(),
            &self.aes_key.get(),
            &self.aes_next_iv.get(),
            data,
        );
        self.aes_next_iv.set(next);
        self.finish(Done::Aes(source, dest));
        None
    }
}

impl symmetric_encryption::AESECB for CryptoCell<'_> {
    fn set_mode_aesecb(&self, encrypting: bool) -> Result<(), ErrorCode> {
        self.aes_mode.set(AesMode::Ecb);
        self.aes_encrypt.set(encrypting);
        Ok(())
    }
}

impl symmetric_encryption::AESCBC for CryptoCell<'_> {
    fn set_mode_aescbc(&self, encrypting: bool) -> Result<(), ErrorCode> {
        self.aes_mode.set(AesMode::Cbc);
        self.aes_encrypt.set(encrypting);
        Ok(())
    }
}

impl symmetric_encryption::AESCtr for CryptoCell<'_> {
    fn set_mode_aesctr(&self, encrypting: bool) -> Result<(), ErrorCode> {
        self.aes_mode.set(AesMode::Ctr);
        self.aes_encrypt.set(encrypting);
        Ok(())
    }
}

impl<'a> digest::DigestData<'a, 32> for CryptoCell<'a> {
    fn set_data_client(&'a self, client: &'a dyn digest::ClientData<32>) {
        self.data_client.set(client);
    }

    fn add_data(
        &self,
        mut data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)> {
        if self.done.is_some() {
            return Err((ErrorCode::BUSY, data));
        }
        if data.len() == 0 {
            return Err((ErrorCode::SIZE, data));
        }
        self.sha_add(data.as_slice());
        // Consumed: the client gets back an empty active range.
        data.slice(data.len()..);
        self.finish(Done::AddData(data));
        Ok(())
    }

    fn add_mut_data(
        &self,
        mut data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        if self.done.is_some() {
            return Err((ErrorCode::BUSY, data));
        }
        if data.len() == 0 {
            return Err((ErrorCode::SIZE, data));
        }
        self.sha_add(data.as_slice());
        data.slice(data.len()..);
        self.finish(Done::AddMutData(data));
        Ok(())
    }

    fn clear_data(&self) {
        self.sha_state.set(Sha256State::new());
        self.sha_pending_len.set(0);
    }
}

impl<'a> digest::DigestHash<'a, 32> for CryptoCell<'a> {
    fn set_hash_client(&'a self, client: &'a dyn digest::ClientHash<32>) {
        self.hash_client.set(client);
    }

    fn run(
        &'a self,
        digest: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        if self.done.is_some() {
            return Err((ErrorCode::BUSY, digest));
        }
        *digest = self.sha_finish();
        self.finish(Done::Hash(digest));
        Ok(())
    }
}

impl<'a> digest::DigestVerify<'a, 32> for CryptoCell<'a> {
    fn set_verify_client(&'a self, client: &'a dyn digest::ClientVerify<32>) {
        self.verify_client.set(client);
    }

    fn verify(
        &'a self,
        compare: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        if self.done.is_some() {
            return Err((ErrorCode::BUSY, compare));
        }
        let digest = self.sha_finish();
        // Compare every byte, so the time does not depend on where they differ.
        let matches = digest
            .iter()
            .zip(compare.iter())
            .fold(0, |diff, (a, b)| diff | (a ^ b))
            == 0;
        self.finish(Done::Verify(compare, matches));
        Ok(())
    }
}

impl<'a> digest::Digest<'a, 32> for CryptoCell<'a> {
    fn set_client(&'a self, client: &'a dyn digest::Client<32>) {
        self.data_client.set(client);
        self.hash_client.set(client);
        self.verify_client.set(client);
    }
}

impl digest::Sha256 for CryptoCell<'_> {
    fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
        Ok(())
    }
}
