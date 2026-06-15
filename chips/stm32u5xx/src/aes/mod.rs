// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

pub mod gcm_ccm;

use crate::dma::{ChannelId, DmaPeripheral};
use core::cell::Cell;
use core::marker::PhantomData;
use kernel::hil::symmetric_encryption::{AESKeySize, AES, AES_BLOCK_SIZE, AES_IV_SIZE};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::ErrorCode;
use stm32u5xx_unsafe::aes::{AesRegistersManager, Control, DMABuffers, Data, Interrupt};

use crate::dma::Dma;

// If 0 < a < 2^16 - 2^8, length is encoded in 2 bytes.
pub(crate) const CCM_AAD_L16_MAX: usize = 0xFF00;

// If 2^16 - 2^8 <= a < 2^32, length is encoded in 6 bytes,
// preceded by the 0xFF 0xFE marker.
pub(crate) const CCM_AAD_L32_MARKER_0: u8 = 0xFF;
pub(crate) const CCM_AAD_L32_MARKER_1: u8 = 0xFE;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AESMode {
    ECB,
    CBC,
    CTR,
    GCM,
    CCM,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DeferredOp {
    None,
    WriteIvx,
    Classic(CryptoContext),
    Gcm(CryptoContext),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum State {
    Idle,
    KeyPreparation(DeferredOp),
    Classic(CryptoContext),
    CCMInit(CryptoContext),
    GCMInit(DeferredOp),
    Header(CryptoContext),
    Payload(CryptoContext),
    Final(CryptoContext),
    DmaHeaderPadding(CryptoContext),
    DmaPayloadPadding(CryptoContext),
    DmaFinalize(CryptoContext),
    DmaCcmB1(CryptoContext),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CryptoContext {
    pub aad_offset: usize,
    pub message_start: usize,
    pub message_end: usize,
    pub current_idx: usize,
    pub tag_len: usize,
    pub confidential: bool,
    pub using_dma: bool,
}

pub struct Aes<'a, K: AESKeySize> {
    pub(crate) register_manager: AesRegistersManager,
    pub(crate) mode: Cell<AESMode>,
    pub(crate) state: Cell<State>,
    pub(crate) encrypting: Cell<bool>,
    pub(crate) classic_client: OptionalCell<&'a dyn kernel::hil::symmetric_encryption::Client<'a>>,
    pub(crate) gcm_client: OptionalCell<&'a dyn kernel::hil::symmetric_encryption::GCMClient>,
    pub(crate) ccm_client: OptionalCell<&'a dyn kernel::hil::symmetric_encryption::CCMClient>,
    pub(crate) input: TakeCell<'static, [u8]>,
    pub(crate) output: TakeCell<'static, [u8]>,
    pub(crate) iv: Cell<[u8; AES_IV_SIZE]>,
    pub(crate) dma: OptionalCell<&'static Dma>,
    pub(crate) dma_in_channel: Cell<Option<ChannelId>>,
    pub(crate) dma_out_channel: Cell<Option<ChannelId>>,
    pub(crate) dma_bufs: DMABuffers,
    pub(crate) _phantom: PhantomData<K>,
}

impl DMABuffers {
    /// Helper function designed to calculate the length of the buffer as a multiple of AES_BLOCK_SIZE
    /// and return the remaining bytes inside a 0-padded buffer. If the length of the buffer, beginning
    /// from start is a multiple of AES_BLOCK_SIZE, will return total_len and None
    pub fn extract_dma_padding(
        buf: &[u8],
        start: usize,
        total_len: usize,
    ) -> (usize, Option<[u8; AES_BLOCK_SIZE]>) {
        // check whether the buffer needs 0-padding
        if total_len > 0 && !total_len.is_multiple_of(AES_BLOCK_SIZE) {
            // length multiple of AES_BLOCK_SIZE
            let len = total_len - (total_len % AES_BLOCK_SIZE);
            // remainder of the buffer, padded with 0s
            let mut pad = [0u8; AES_BLOCK_SIZE];
            let rem = total_len - len;
            pad[..rem].copy_from_slice(&buf[start + len..start + total_len]);
            (len, Some(pad))
        } else {
            (total_len, None)
        }
    }
}

impl<'a, K: AESKeySize> Aes<'a, K> {
    // default mode: ECB , encrypting
    pub const fn new(base: AesRegistersManager) -> Aes<'a, K> {
        Aes {
            register_manager: base,
            mode: Cell::new(AESMode::ECB),
            encrypting: Cell::new(true),
            state: Cell::new(State::Idle),
            classic_client: OptionalCell::empty(),
            gcm_client: OptionalCell::empty(),
            ccm_client: OptionalCell::empty(),
            input: TakeCell::empty(),
            output: TakeCell::empty(),
            iv: Cell::new([0u8; AES_IV_SIZE]),
            dma: OptionalCell::empty(),
            dma_in_channel: Cell::new(None),
            dma_out_channel: Cell::new(None),
            dma_bufs: DMABuffers::new(),
            _phantom: PhantomData::<K>,
        }
    }

    /// Sets up the in and out channels, and sets the peripheral up as the dma client
    pub fn set_dma(
        aes: &'static Self,
        dma: &'static Dma,
        in_channel: ChannelId,
        out_channel: ChannelId,
    ) {
        aes.dma.set(dma);
        aes.dma_in_channel.set(Some(in_channel));
        aes.dma_out_channel.set(Some(out_channel));
        dma.set_client(in_channel, aes);
        dma.set_client(out_channel, aes);
    }

    /// Function that handles sending the buffer back to the client with the result inside after
    /// a successful dma transfer to and from the peripheral
    pub(crate) fn handle_dma_ecb_cbc_ctr(&self, channel: ChannelId, out_ch: ChannelId) {
        if out_ch != channel {
            return;
        }

        self.state.set(State::Idle);
        self.register_manager
            .registers
            .cr
            .modify(Control::DMAINEN::CLEAR + Control::DMAOUTEN::CLEAR);

        if let Some(output) = self.dma_bufs.take_dma_out_buf() {
            let input = self.dma_bufs.take_dma_in_buf();

            self.classic_client
                .map(move |client| client.crypt_done(input, output));
        } else {
            self.disable();
        }
    }

    pub(crate) fn enable_interrupts(&self) {
        self.register_manager
            .registers
            .intenr
            .modify(Interrupt::CCI::SET + Interrupt::KE::SET + Interrupt::RWE::SET);
    }

    pub(crate) fn disable_interrupts(&self) {
        self.register_manager
            .registers
            .intenr
            .modify(Interrupt::CCI::CLEAR + Interrupt::KE::CLEAR + Interrupt::RWE::CLEAR);
    }

    pub(crate) fn apply_crypto_direction(&self, encrypting: bool) {
        self.encrypting.set(encrypting);
        if encrypting {
            self.register_manager
                .registers
                .cr
                .modify(Control::MODE::Encrypt);
        } else {
            self.register_manager
                .registers
                .cr
                .modify(Control::MODE::Decrypt);
        }
    }

    /// Returns true if the NPBLB register needs to be set for the current mode and encryption direction
    pub(crate) fn uses_npblb(&self) -> bool {
        (self.mode.get() == AESMode::GCM && self.encrypting.get())
            || (self.mode.get() == AESMode::CCM && !self.encrypting.get())
    }

    /// Writes a slice to the data input register. If the slice is smaller
    /// than AES_BLOCK_SIZE bytes, it automatically pads the remainder with zeros.
    pub(crate) fn write_padded_to_dinr(&self, slice: &[u8]) {
        let mut buf = [0u8; AES_BLOCK_SIZE];
        buf[..slice.len()].copy_from_slice(slice);

        for chunk in buf.chunks_exact(4) {
            let word = u32::from_le_bytes(chunk.try_into().unwrap());
            self.register_manager.registers.dinr.set(word);
        }
    }

    /// writes AES_BLOCK_SIZE bytes to DINR register starting at ctx.current_idx + ctx.message_start
    /// from dest buffer if operation is in-place, or at ctx.current_idx in the input buffer
    pub(crate) fn write_input(&self, ctx: CryptoContext) {
        let current_idx = ctx.current_idx;
        if self.input.is_some() {
            // Out-of-place
            self.input.map(|buf| {
                self.write_padded_to_dinr(&buf[current_idx..current_idx + AES_BLOCK_SIZE])
            });
        } else {
            // In-place
            let offset = ctx.message_start + current_idx;
            self.output
                .map(|buf| self.write_padded_to_dinr(&buf[offset..offset + AES_BLOCK_SIZE]));
        }
    }

    /// returns AES_BLOCK_SIZE bytes of data from the DOUTR register
    pub(crate) fn get_output(&self) -> [u8; AES_BLOCK_SIZE] {
        let mut block = [0u8; AES_BLOCK_SIZE];
        for chunk in block.chunks_exact_mut(4) {
            let word = self.register_manager.registers.doutr.get();
            chunk.copy_from_slice(&word.to_le_bytes());
        }
        block
    }

    /// Helper to write a 128-bit or 256-bit key into the hardware key registers
    pub(crate) fn write_key_registers(&self, key: &[u8]) {
        // Default to using the first 16 bytes for the lower registers (AES-128 behavior)
        let mut lower_key_chunk = &key[0..16];

        if K::LENGTH == 32 {
            // AES-256: Write KEYR7 down to KEYR4 first
            for (reg, chunk) in self
                .register_manager
                .registers
                .keyr2
                .iter()
                .rev()
                .zip(key[0..16].chunks_exact(4))
            {
                let word = u32::from_be_bytes(chunk.try_into().unwrap());
                reg.write(Data::DATA.val(word));
            }

            // Update the slice so the lower registers get the second half of the 256-bit key
            lower_key_chunk = &key[16..32];
        }

        // Write KEYR3 down to KEYR0
        for (reg, chunk) in self
            .register_manager
            .registers
            .keyr
            .iter()
            .rev()
            .zip(lower_key_chunk.chunks_exact(4))
        {
            let word = u32::from_be_bytes(chunk.try_into().unwrap());
            reg.write(Data::DATA.val(word));
        }
    }

    /// Helper to write a AES128_IV_SIZE-byte IV into the hardware IV registers
    pub(crate) fn write_iv_registers(&self, iv: &[u8; AES_IV_SIZE]) {
        for (reg, chunk) in self
            .register_manager
            .registers
            .ivr
            .iter()
            .rev()
            .zip(iv.chunks_exact(4))
        {
            let word = u32::from_be_bytes(chunk.try_into().expect("IV chunk len mismatch"));
            reg.write(Data::DATA.val(word));
        }
    }

    /// Helper to move data from the hardware registers into the output buffer.
    pub(crate) fn write_output_block(&self, offset: usize) {
        let block = self.get_output();
        self.output.map(|output| {
            output[offset..offset + AES_BLOCK_SIZE].copy_from_slice(&block);
        });
    }

    /// Handles the beginning of encryption/decryption process for ECB CTR and CBC, sets state
    /// accordingly
    pub(crate) fn start_classic_crypt(&self, mut ctx: CryptoContext) {
        if let (Some(dma), Some(in_ch), Some(out_ch)) = (
            self.dma.get(),
            self.dma_in_channel.get(),
            self.dma_out_channel.get(),
        ) {
            ctx.using_dma = true;
            let len = (ctx.message_end - ctx.message_start) as u32;

            // prepare Output Buffer
            let dest = self.output.take().unwrap();

            let (out_slice, out_ptr) = DMABuffers::setup_dma_buf(
                dest,
                ctx.message_start,
                ctx.message_end - ctx.message_start,
            );
            self.dma_bufs.dma_out_buf.replace(out_slice);

            // prepare Input Buffer
            let in_ptr = if let Some(src) = self.input.take() {
                let (in_slice, ptr) =
                    DMABuffers::setup_dma_buf(src, 0, ctx.message_end - ctx.message_start);
                self.dma_bufs.dma_in_buf.replace(in_slice); // Put it directly into in_buf!
                ptr
            } else {
                // in-place: Source pointer mirrors the output pointer
                out_ptr
            };

            // setup DMA Channels
            dma.setup(in_ch, DmaPeripheral::AESIN, in_ptr, len);
            dma.setup(out_ch, DmaPeripheral::AESOUT, out_ptr, len);

            self.state.set(State::Classic(ctx));
            if !self
                .register_manager
                .registers
                .cr
                .any_matching_bits_set(Control::EN::SET)
            {
                self.register_manager.registers.cr.modify(Control::EN::SET);
            }
            self.register_manager
                .registers
                .cr
                .modify(Control::DMAINEN::SET + Control::DMAOUTEN::SET);
        } else {
            // interrupt fallback
            self.state.set(State::Classic(ctx));
            if !self
                .register_manager
                .registers
                .cr
                .any_matching_bits_set(Control::EN::SET)
            {
                self.register_manager.registers.cr.modify(Control::EN::SET);
            }
            self.write_input(ctx);
        }
    }

    /// Function for ECB and CBC decryption modes which goes though Key derivation operation
    pub(crate) fn prepare_decryption_key(&self, key: &[u8]) {
        self.state.set(State::KeyPreparation(DeferredOp::None));
        self.register_manager
            .registers
            .cr
            .modify(Control::EN::CLEAR);
        self.register_manager
            .registers
            .cr
            .modify(Control::MODE::KeyDerivation + Control::KMOD::Normal);

        self.write_key_registers(key);

        self.register_manager.registers.cr.modify(Control::EN::SET);
    }

    /// Main state machine handler for ECB, CBC and CTR modes.
    pub(crate) fn handle_classic_client(&self) {
        match self.state.get() {
            State::KeyPreparation(deferred_op) => {
                self.register_manager
                    .registers
                    .cr
                    .modify(Control::EN::CLEAR);
                self.register_manager
                    .registers
                    .cr
                    .modify(Control::MODE::Decrypt);

                match deferred_op {
                    DeferredOp::Classic(ctx) => {
                        self.write_iv_registers(&self.iv.get());
                        self.iv.set([0; AES_IV_SIZE]);
                        self.start_classic_crypt(ctx);
                    }
                    DeferredOp::WriteIvx => {
                        self.write_iv_registers(&self.iv.get());
                        self.iv.set([0; AES_IV_SIZE]);
                        self.state.set(State::Idle);
                    }
                    DeferredOp::None => {
                        // No operations pending, safe to clear to Idle
                        self.state.set(State::Idle);
                    }
                    DeferredOp::Gcm(_) => unreachable!(),
                }
            }
            State::Classic(mut ctx) => {
                if ctx.using_dma {
                    return;
                }
                let start_idx = ctx.message_start;
                let end_idx = ctx.message_end;

                self.write_output_block(start_idx + ctx.current_idx);

                ctx.current_idx += AES_BLOCK_SIZE;

                // if encoding is finished, return the buffer to the client
                if start_idx + ctx.current_idx >= end_idx {
                    self.state.set(State::Idle);
                    self.output.take().map(|output| {
                        self.classic_client
                            .map(move |client| client.crypt_done(self.input.take(), output));
                    });
                } else {
                    self.state.set(State::Classic(ctx));
                    self.write_input(ctx);
                }
            }
            _ => {}
        }
    }

    pub fn handle_interrupt(&self) {
        if self
            .register_manager
            .registers
            .intstr
            .is_set(Interrupt::CCI)
        {
            self.register_manager
                .registers
                .intclr
                .write(Interrupt::CCI::SET);
            match self.mode.get() {
                AESMode::ECB | AESMode::CBC | AESMode::CTR => self.handle_classic_client(),
                AESMode::GCM => self.handle_gcm_client(),
                AESMode::CCM => self.handle_ccm_client(),
            }
        }

        // triggered on unexpected reads/writes to AES peripheral. These do
        // not stop the AES peripheral computation
        if self
            .register_manager
            .registers
            .intstr
            .is_set(Interrupt::RWE)
        {
            self.register_manager
                .registers
                .intclr
                .write(Interrupt::RWE::SET);
        }

        // Important for SAES sharing, otherwise states that would trigger it
        // are handled in set_key()
        if self.register_manager.registers.intstr.is_set(Interrupt::KE) {
            self.register_manager
                .registers
                .intclr
                .write(Interrupt::KE::SET);
        }
    }
}

impl<'a, K: AESKeySize> kernel::hil::symmetric_encryption::AES<'a, K> for Aes<'a, K> {
    fn enable(&self) {
        self.register_manager
            .registers
            .cr
            .modify(Control::IPRST::SET);
        self.register_manager.registers.cr.write(Control::EN::CLEAR);
        self.register_manager
            .registers
            .cr
            .modify(Control::DATATYPE::Byte);
        self.state.set(State::Idle);
        self.enable_interrupts();
    }

    fn disable(&self) {
        self.register_manager.registers.cr.write(Control::EN::CLEAR);
        self.disable_interrupts();
        self.register_manager
            .registers
            .cr
            .modify(Control::IPRST::SET);
        self.state.set(State::Idle);
    }

    fn set_client(&'a self, client: &'a dyn kernel::hil::symmetric_encryption::Client<'a>) {
        self.classic_client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if key.len() != K::LENGTH {
            return Err(ErrorCode::INVAL);
        }

        if K::LENGTH == 16 {
            self.register_manager
                .registers
                .cr
                .modify(Control::KEYSIZE::AES128);
        } else {
            self.register_manager
                .registers
                .cr
                .modify(Control::KEYSIZE::AES256);
        }

        if self
            .register_manager
            .registers
            .cr
            .any_matching_bits_set(Control::EN::SET)
        {
            return Err(ErrorCode::BUSY);
        }

        if !self.encrypting.get()
            && (self.mode.get() == AESMode::ECB || self.mode.get() == AESMode::CBC)
        {
            self.prepare_decryption_key(key);
        } else {
            self.write_key_registers(key);
        }

        Ok(())
    }

    fn set_iv(&self, iv: &[u8]) -> Result<(), ErrorCode> {
        if iv.len() != AES_IV_SIZE {
            return Err(ErrorCode::INVAL);
        }

        if self
            .register_manager
            .registers
            .cr
            .any_matching_bits_set(Control::EN::SET)
        {
            return Err(ErrorCode::BUSY);
        }

        match self.state.get() {
            State::Idle | State::GCMInit(_) => self.write_iv_registers(iv.try_into().unwrap()),
            State::KeyPreparation(_) => {
                self.iv.set(iv.try_into().unwrap());
                self.state.set(State::KeyPreparation(DeferredOp::WriteIvx));
            }
            _ => return Err(ErrorCode::BUSY),
        }

        Ok(())
    }

    fn start_message(&self) {}

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
        let state = self.state.get();

        //  Hardware busy check
        if self.output.is_some() || !matches!(state, State::Idle | State::KeyPreparation(_)) {
            return Some((Err(ErrorCode::BUSY), source, dest));
        }

        // Bounds checking
        if start_index >= stop_index || !(stop_index - start_index).is_multiple_of(AES_BLOCK_SIZE) {
            return Some((Err(ErrorCode::INVAL), source, dest));
        }
        if let Some(src) = &source {
            if src.len() < stop_index - start_index {
                return Some((Err(ErrorCode::INVAL), source, dest));
            }
        }
        if dest.len() < stop_index {
            return Some((Err(ErrorCode::INVAL), source, dest));
        }

        let ctx = CryptoContext {
            aad_offset: 0,
            message_start: start_index,
            message_end: stop_index,
            current_idx: 0,
            tag_len: 0,
            confidential: true,
            using_dma: false, // Set in start_classic_crypt
        };

        if let Some(src) = source {
            self.input.replace(src);
        }
        self.output.replace(dest);

        // Queue or Execute
        if let State::KeyPreparation(_) = state {
            self.state
                .set(State::KeyPreparation(DeferredOp::Classic(ctx)));
            return None;
        }

        self.start_classic_crypt(ctx);
        None
    }
}

impl<K: AESKeySize> kernel::hil::symmetric_encryption::AESECB for Aes<'_, K> {
    fn set_mode_aesecb(&self, encrypting: bool) -> Result<(), ErrorCode> {
        self.mode.set(AESMode::ECB);
        self.register_manager
            .registers
            .cr
            .modify(Control::CHMOD::ECB + Control::CHMOD_2::CLEAR);
        self.apply_crypto_direction(encrypting);
        Ok(())
    }
}

impl<K: AESKeySize> kernel::hil::symmetric_encryption::AESCtr for Aes<'_, K> {
    fn set_mode_aesctr(&self, _encrypting: bool) -> Result<(), ErrorCode> {
        self.mode.set(AESMode::CTR);
        self.register_manager
            .registers
            .cr
            .modify(Control::CHMOD::CTR + Control::CHMOD_2::CLEAR);
        self.register_manager
            .registers
            .cr
            .modify(Control::MODE::Encrypt);
        Ok(())
    }
}

impl<K: AESKeySize> kernel::hil::symmetric_encryption::AESCBC for Aes<'_, K> {
    fn set_mode_aescbc(&self, encrypting: bool) -> Result<(), ErrorCode> {
        self.mode.set(AESMode::CBC);
        self.register_manager
            .registers
            .cr
            .modify(Control::CHMOD::CBC + Control::CHMOD_2::CLEAR);
        self.apply_crypto_direction(encrypting);
        Ok(())
    }
}

impl<K: AESKeySize> crate::dma::DmaClient for Aes<'_, K> {
    fn transfer_done(&self, channel: ChannelId) {
        if let Some(out_ch) = self.dma_out_channel.get() {
            match self.mode.get() {
                AESMode::ECB | AESMode::CBC | AESMode::CTR => {
                    self.handle_dma_ecb_cbc_ctr(channel, out_ch)
                }
                AESMode::GCM | AESMode::CCM => self.handle_dma_gcm_ccm(out_ch != channel),
            }
        }
    }
}
