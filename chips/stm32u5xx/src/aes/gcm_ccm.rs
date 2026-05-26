// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use crate::aes::registers::{Control, Interrupt};
use crate::aes::{AESMode, Aes, CryptoContext, DeferredOp, State};
use crate::dma::ChannelId;
use crate::dma::Dma;
use crate::dma::DmaPeripheral;
use kernel::hil::symmetric_encryption::{
    AESKeySize, GCMClient, AES, AES128_IV_SIZE, AES_BLOCK_SIZE,
};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::ErrorCode;

impl<K: AESKeySize> Aes<'_, K> {
    /// Configures and initiates a DMA-backed transfer for GCM or CCM modes.
    /// It determines whether the operation requires an AAD (Header) phase or
    /// should skip directly to the Payload phase, extracts any non-block-aligned
    /// trailing bytes for later insertion.
    pub(crate) fn setup_dma_gcm_ccm(
        &self,
        mut ctx: CryptoContext,
        buf: &'static mut [u8],
        in_ch: ChannelId,
        out_ch: ChannelId,
        dma: &Dma,
    ) {
        ctx.using_dma = true;
        // If the payload isn't a multiple of the block size, we need to add 0-padding
        let (msg_len, msg_pad) =
            Self::extract_dma_padding(buf, ctx.message_start, ctx.message_end - ctx.message_start);
        if let Some(pad) = msg_pad {
            self.dma_utils.dma_message_buff.replace(pad);
        }

        // Determine whether to start with Header (AAD) or Payload phase.
        // AAD exists when aad_offset != message_offset.
        let (len, start) = if ctx.aad_offset != ctx.message_start {
            // Hardware requirement: DMA processes block-aligned chunks. Trailing bytes are saved
            // and fed manually via interrupts. If the header isn't a multiple of AES_BLOCK_SIZE,
            // we need 0-padding
            let (aad_len, aad_pad) =
                Self::extract_dma_padding(buf, ctx.aad_offset, ctx.message_start - ctx.aad_offset);

            if let Some(pad) = aad_pad {
                self.dma_utils.dma_aad_buff.replace(pad);
            }
            ctx.current_idx = aad_len;
            self.state.set(State::Header(ctx));
            self.registers.cr.modify(Control::GCMPH::Header);
            (aad_len, ctx.aad_offset)
        } else {
            ctx.current_idx = msg_len;
            self.state.set(State::Payload(ctx));
            self.registers.cr.modify(Control::GCMPH::Payload);

            (msg_len, ctx.message_start)
        };

        // Wrap the entire buffer in a DMA slice for later use
        let (in_slice, in_ptr) = self.setup_dma_buf(buf, start, len);
        self.dma_utils.dma_in_buf.replace(in_slice);

        // Setup DMA Channels
        dma.setup(in_ch, DmaPeripheral::AESIN, in_ptr, len as u32);
        // Payload phase needs both input and output DMA; Header is input-only
        if ctx.aad_offset == ctx.message_start {
            dma.setup(out_ch, DmaPeripheral::AESOUT, in_ptr, len as u32);
        }

        self.registers.cr.modify(Control::EN::SET);
        // Start Hardware — enable DMAINEN always; DMAOUTEN only for Payload
        if ctx.aad_offset == ctx.message_start {
            self.registers
                .cr
                .modify(Control::DMAINEN::SET + Control::DMAOUTEN::SET);
        } else {
            self.registers.cr.modify(Control::DMAINEN::SET);
        }
    }

    /// The primary interrupt router for DMA-based GCM/CCM operations.
    /// It catches the completion of DMA transfers and manually feeds any saved,
    /// remaining bytes to the peripheral after padding with 0s.
    pub(crate) fn handle_dma_gcm_ccm(&self, is_input: bool) {
        match self.state.get() {
            State::Header(mut ctx) => {
                // Handle padded last AAD block if present.
                if let Some(buf) = self.dma_utils.dma_aad_buff.take() {
                    let aad_offset = ctx.aad_offset;
                    let start_idx = ctx.message_start;
                    let remaining_aad = start_idx - (aad_offset + ctx.current_idx);
                    self.write_padded_to_dinr(&buf);
                    ctx.current_idx += remaining_aad;
                    self.state.set(State::DmaHeaderPadding(ctx));
                } else {
                    self.continue_dma_payload_setup(ctx);
                }
            }
            State::Payload(ctx) => {
                // if the dma interrupt arrived from the input channel the output may not have finished
                if is_input {
                    return;
                }
                // Handle padded partial block if present
                if let Some(pad_buf) = self.dma_utils.dma_message_buff.take() {
                    let start_idx = ctx.message_start;
                    let end_idx = ctx.message_end;
                    let message_len = end_idx - start_idx;
                    let block_len = message_len % AES_BLOCK_SIZE;

                    // Set NPBLB to indicate the number of padding bytes when
                    // required by the mode (GCM encrypt or CCM decrypt).
                    if self.uses_npblb() {
                        self.registers
                            .cr
                            .modify(Control::NPBLB.val((AES_BLOCK_SIZE - block_len) as u32));
                    }

                    // Manually write the padded block to DINR
                    self.write_padded_to_dinr(&pad_buf);

                    self.state.set(State::DmaPayloadPadding(ctx));
                } else {
                    // Perform the Final phase
                    self.dma_start_tag_computation(ctx);
                }
            }
            _ => {}
        }
    }

    /// Specific to CCM mode. Handles the completion of the mandatory B1 block (which encodes
    /// the AAD length) before starting the DMA transfer for the remainder of the AAD.
    pub(crate) fn ccm_dma_b1_finish(&self, mut ctx: CryptoContext) {
        let aad_offset = ctx.aad_offset;
        let start_idx = ctx.message_start;
        let aad_len = start_idx - aad_offset;
        let block_len = ctx.current_idx;
        let remaining_aad = aad_len - block_len;

        if let (Some(dma), Some(in_ch)) = (
            self.dma_utils.dma.get(),
            self.dma_utils.dma_in_channel.get(),
        ) {
            if let Some(buf) = self.output.take() {
                let (len, aad_pad) =
                    Self::extract_dma_padding(buf, aad_offset + block_len, remaining_aad);
                if let Some(pad) = aad_pad {
                    self.dma_utils.dma_aad_buff.replace(pad);
                }
                ctx.current_idx = block_len + len;
                self.state.set(State::Header(ctx));
                if len > 0 {
                    let (slice, ptr) = self.setup_dma_buf(buf, aad_offset + block_len, len);
                    self.dma_utils.dma_in_buf.replace(slice);
                    dma.setup(in_ch, DmaPeripheral::AESIN, ptr, len as u32);
                    self.registers.cr.modify(Control::EN::SET);
                    self.registers.cr.modify(Control::DMAINEN::SET);
                } else {
                    // If len is 0, we don't need DMA. Just put the buffer back directly
                    // using a 0-length call so the buffer is saved in dma_in_buf
                    // on future reads even when this branch was followed
                    let (slice, _) = self.setup_dma_buf(buf, aad_offset + block_len, 0);
                    self.dma_utils.dma_in_buf.replace(slice);
                    self.handle_dma_gcm_ccm(true);
                }
            }
        }
    }

    /// Bridges the transition from the Header (AAD) phase to the Payload phase in a DMA-backed operation,
    /// reconfiguring the DMA channels to handle both AESIN and AESOUT.
    pub(crate) fn continue_dma_payload_setup(&self, mut ctx: CryptoContext) {
        let start_idx = ctx.message_start;
        let message_len = ctx.message_end - start_idx;

        if message_len == 0 {
            ctx.current_idx = 0;
            self.dma_start_tag_computation(ctx);
            return;
        }

        // Safely extract all asynchronous resources at once
        if let (Some(buf), Some(in_ch), Some(out_ch), Some(dma)) = (
            self.take_dma_in_buf(),
            self.dma_utils.dma_in_channel.get(),
            self.dma_utils.dma_out_channel.get(),
            self.dma_utils.dma.get(),
        ) {
            let (len, msg_pad) = Self::extract_dma_padding(buf, start_idx, message_len);
            if let Some(pad) = msg_pad {
                self.dma_utils.dma_message_buff.replace(pad);
            }
            ctx.current_idx = len;
            self.state.set(State::Payload(ctx));

            self.registers
                .cr
                .modify(Control::DMAINEN::CLEAR + Control::DMAOUTEN::CLEAR);

            if len > 0 {
                // Wrap the entire buffer in a DMA slice for later use
                let (in_slice, ptr) = self.setup_dma_buf(buf, start_idx, len);
                self.dma_utils.dma_in_buf.replace(in_slice);

                dma.setup(out_ch, DmaPeripheral::AESOUT, ptr, len as u32);
                dma.setup(in_ch, DmaPeripheral::AESIN, ptr, len as u32);

                if !self.registers.cr.any_matching_bits_set(Control::EN::SET) {
                    self.registers.cr.modify(Control::EN::SET);
                }

                self.registers.cr.modify(Control::GCMPH::Payload);
                self.registers
                    .cr
                    .modify(Control::DMAINEN::SET + Control::DMAOUTEN::SET);
            } else {
                // If len is 0, we don't need DMA. Just put the buffer back directly
                // using a 0-length call so the buffer is inside dma_in_buf on future
                // reads even when this branch was followed
                let (in_slice, _) = self.setup_dma_buf(buf, start_idx, 0);
                self.dma_utils.dma_in_buf.replace(in_slice);

                self.registers.cr.modify(Control::GCMPH::Payload);
                self.handle_dma_gcm_ccm(false);
            }
        } else {
            self.disable();
        }
    }

    /// Triggers the Final phase of the AES hardware to compute the authentication tag. For GCM, this
    /// involves writing the length block; for both modes, it triggers tag computation.
    pub(crate) fn dma_start_tag_computation(&self, ctx: CryptoContext) {
        self.registers
            .cr
            .modify(Control::DMAINEN::CLEAR + Control::DMAOUTEN::CLEAR + Control::GCMPH::Final);

        self.registers.intclr.write(Interrupt::CCI::SET);

        if self.mode.get() == AESMode::GCM {
            // GCM Final: write the 128-bit lengths block to DINR
            let aad_len_bits = ((ctx.message_start - ctx.aad_offset) * 8) as u32;
            let msg_len_bits = ((ctx.message_end - ctx.message_start) * 8) as u32;
            self.registers.dinr.set(0);
            self.registers.dinr.set(aad_len_bits);
            self.registers.dinr.set(0);
            self.registers.dinr.set(msg_len_bits);
        }

        self.state.set(State::DmaFinalize(ctx));
    }

    /// Finalizes the DMA-backed cryptographic operation. For encryption, it appends the computed tag
    /// to the output. For decryption, it executes a constant-time comparison against the provided tag
    pub(crate) fn dma_gcm_ccm_finish(&self, ctx: CryptoContext) {
        let hardware_tag = self.get_output();

        if let Some(buf) = self.take_dma_in_buf() {
            self.registers.cr.modify(Control::GCMPH::CLEAR);
            self.registers.cr.modify(Control::EN::CLEAR);

            let end_idx = ctx.message_end;
            let start_idx = ctx.message_start;

            if let Some(padded_msg) = self.dma_utils.dma_message_buff.take() {
                let pad_len = (end_idx - start_idx) % AES_BLOCK_SIZE;
                if pad_len > 0 {
                    buf[end_idx - pad_len..end_idx].copy_from_slice(&padded_msg[0..pad_len]);
                }
            }

            let tag_len = ctx.tag_len;
            let tag_is_valid = if self.encrypting.get() {
                buf[end_idx..end_idx + tag_len].copy_from_slice(&hardware_tag[..tag_len]);
                true
            } else {
                let tag = &buf[end_idx..end_idx + tag_len];
                self.check_tag(tag, &hardware_tag[..tag_len])
            };

            self.state.set(State::Idle);

            if self.mode.get() == AESMode::GCM {
                self.gcm_client
                    .map(|client| client.crypt_done(buf, Ok(()), tag_is_valid));
            } else {
                self.ccm_client
                    .map(|client| client.crypt_done(buf, Ok(()), tag_is_valid));
            }
        } else {
            self.disable();
        }
    }

    // End of DMA specific functions

    /// Resets the AES peripheral and configures the chaining mode (CHMOD) specifically for CCM
    pub(crate) fn init_ccm(&self) {
        self.enable();
        // ECB mode has value 00. The CCM mode thould be 100, 1 in CHMOD_2 and 00 in CHMOD
        self.registers.cr.modify(Control::CHMOD::ECB);
        self.registers.cr.modify(Control::CHMOD_2::SET);
        self.registers.cr.modify(Control::GCMPH::Init);
        self.mode.set(AESMode::CCM);
    }

    /// The execution entry point for GCM cryptography. Evaluates whether valid DMA channels are attached
    /// and routes execution to the DMA pipeline or falls back to the CPU-driven interrupt pipeline.
    pub(crate) fn start_gcm_crypt(&self, mut ctx: CryptoContext, buf: &'static mut [u8]) {
        // test for 0 len aad and message just to generate tag
        if let (Some(dma), Some(in_ch), Some(out_ch)) = (
            self.dma_utils.dma.get(),
            self.dma_utils.dma_in_channel.get(),
            self.dma_utils.dma_out_channel.get(),
        ) {
            ctx.using_dma = true;
            self.setup_dma_gcm_ccm(ctx, buf, in_ch, out_ch, dma);
        } else {
            self.output.replace(buf);
            // if aad exists, we continue to the header phase, otherwise we go straight to payload
            if ctx.aad_offset != ctx.message_start {
                self.registers.cr.modify(Control::GCMPH::Header);
                self.registers.cr.modify(Control::EN::SET);
                self.aad_phase(ctx);
            } else {
                self.registers.cr.modify(Control::GCMPH::Payload);
                self.registers.cr.modify(Control::EN::SET);
                self.start_payload_phase(ctx);
            }
        }
    }

    /// Resets the AES peripheral and configures the chaining mode (CHMOD) specifically for GCM,
    /// pushing the state machine into the GCMInit phase.
    pub(crate) fn init_gcm(&self) {
        self.enable();
        self.registers.cr.modify(Control::CHMOD::GCM_CCM);
        self.registers.cr.modify(Control::CHMOD_2::CLEAR);
        self.registers.cr.modify(Control::GCMPH::Init);
        self.state.set(State::GCMInit(DeferredOp::None));
        self.mode.set(AESMode::GCM);
    }

    /// Executes a single step of the CPU-driven GCM/CCM state machine, feeding
    /// exactly one block of data to the hardware during for the header (aad) phase.
    pub(crate) fn aad_phase(&self, mut ctx: CryptoContext) {
        let offset = ctx.aad_offset + ctx.current_idx;
        let remaining_aad = ctx.message_start - offset;
        let chunk_len = remaining_aad.min(AES_BLOCK_SIZE);

        self.output.map(|buf| {
            self.write_padded_to_dinr(&buf[offset..offset + chunk_len]);
        });

        ctx.current_idx += AES_BLOCK_SIZE;
        self.state.set(State::Header(ctx));
    }

    /// Checks whether the payload phase is over, if not continues it
    pub(crate) fn advance_aad_phase(&self, mut ctx: CryptoContext) {
        if ctx.current_idx + ctx.aad_offset >= ctx.message_start {
            ctx.current_idx = 0;
            if ctx.message_start != ctx.message_end {
                self.registers.cr.modify(Control::GCMPH::Payload);
                self.state.set(State::Payload(ctx));
                self.start_payload_phase(ctx);
            } else {
                if self.mode.get() == AESMode::GCM {
                    self.insert_lengths_gcm(ctx);
                } else {
                    self.state.set(State::Final(ctx));
                    self.registers.cr.modify(Control::GCMPH::Final);
                }
            }
        } else {
            self.aad_phase(ctx);
        }
    }

    /// Initiates the Payload phase by feeding the first block of data. Unlike subsequent payload
    /// steps, it does not read a prior output or advance the byte index.
    pub(crate) fn start_payload_phase(&self, ctx: CryptoContext) {
        let start_idx = ctx.message_start;
        let block_len = (ctx.message_end - start_idx).min(AES_BLOCK_SIZE);
        if block_len < AES_BLOCK_SIZE && self.uses_npblb() {
            self.registers
                .cr
                .modify(Control::NPBLB.val((AES_BLOCK_SIZE - block_len) as u32));
        }
        self.output.map(|output| {
            self.write_padded_to_dinr(&output[start_idx..start_idx + block_len]);
        });
        self.state.set(State::Payload(ctx));
    }

    /// Executes a single step of payload phase, feeding exactly one block of data  and reading the
    /// finished one.
    pub(crate) fn payload_phase(&self, mut ctx: CryptoContext) {
        let start_idx = ctx.message_start;

        self.write_output_block(start_idx + ctx.current_idx);

        ctx.current_idx += AES_BLOCK_SIZE;
        self.state.set(State::Payload(ctx));

        // Set padding hardware bits if necessary
        let remaining_bytes = ctx.message_end - (start_idx + ctx.current_idx);
        let block_len = remaining_bytes.min(AES_BLOCK_SIZE);

        // The NPBLB register must be programmed with the number of padding bytes in
        // the final block so the hardware can accurately compute the GCM/CCM tag.
        if block_len < AES_BLOCK_SIZE && self.uses_npblb() {
            self.registers
                .cr
                .modify(Control::NPBLB.val((AES_BLOCK_SIZE - block_len) as u32));
        }

        self.output.map(|output| {
            self.write_padded_to_dinr(
                &output[start_idx + ctx.current_idx..start_idx + ctx.current_idx + block_len],
            );
        });
    }

    /// Checks whether the payload phase is over, if not continues it
    pub(crate) fn advance_payload_phase(&self, ctx: CryptoContext) {
        let start_idx = ctx.message_start;
        let current_idx = ctx.current_idx;
        let end_idx = ctx.message_end;
        let block_len = end_idx - current_idx - start_idx;

        if block_len <= AES_BLOCK_SIZE {
            let block = self.get_output();
            self.output.map(|output| {
                output[start_idx + current_idx..end_idx].copy_from_slice(&block[..block_len]);
            });
            if self.mode.get() == AESMode::CCM {
                self.state.set(State::Final(ctx));
                self.registers.cr.modify(Control::GCMPH::Final);
            } else {
                self.insert_lengths_gcm(ctx);
            }
        } else {
            self.payload_phase(ctx);
        }
    }

    /// Executes the final write of the GCM mode, where, before reading the tag, the length of the
    /// aad and the message are sent to the peripheral
    pub(crate) fn insert_lengths_gcm(&self, ctx: CryptoContext) {
        self.state.set(State::Final(ctx));
        self.registers.cr.modify(Control::GCMPH::Final);

        let aad_len_bits = ((ctx.message_start - ctx.aad_offset) * 8) as u32;
        let msg_len_bits = ((ctx.message_end - ctx.message_start) * 8) as u32;
        self.registers.dinr.set(0);
        self.registers.dinr.set(aad_len_bits);
        self.registers.dinr.set(0);
        self.registers.dinr.set(msg_len_bits);
    }

    /// final phase for GCM and CCM modes, computes and either writes or checks the tag
    /// and returns the result to the client
    pub(crate) fn final_phase(&self, ctx: CryptoContext) {
        self.state.set(State::Idle);
        let tag_is_valid = if self.encrypting.get() {
            self.compute_tag(ctx);
            true
        } else {
            self.output.map_or(false, |output| {
                let tag = &output[ctx.message_end..ctx.message_end + ctx.tag_len];
                let hardware_tag = self.get_output();
                self.check_tag(tag, &hardware_tag[..ctx.tag_len])
            })
        };

        self.registers.cr.modify(Control::GCMPH::CLEAR);
        self.registers.cr.modify(Control::EN::CLEAR);

        if let Some(output) = self.output.take() {
            if self.mode.get() == AESMode::GCM {
                self.gcm_client
                    .map(|client| client.crypt_done(output, Ok(()), tag_is_valid));
            } else {
                self.ccm_client
                    .map(|client| client.crypt_done(output, Ok(()), tag_is_valid));
            }
        }
    }

    /// Performs a constant-time verification for decryption
    pub(crate) fn check_tag(&self, tag: &[u8], hardware_tag: &[u8]) -> bool {
        let mut diff = 0u8;
        for (a, b) in tag.iter().zip(hardware_tag.iter()) {
            diff |= a ^ b;
        }
        diff == 0
    }

    /// Retrieves the AES_BLOCK_SIZE-byte authentication tag from the DOUTR register and
    /// appends it to the ciphertext for encryption.
    pub(crate) fn compute_tag(&self, ctx: CryptoContext) {
        let end_idx = ctx.message_end;
        self.output.map(|output| {
            let tag_len = ctx.tag_len;
            let hardware_tag = self.get_output();
            output[end_idx..end_idx + tag_len].copy_from_slice(&hardware_tag[..tag_len]);
        });
    }

    pub(crate) fn handle_gcm_client(&self) {
        match self.state.get() {
            State::GCMInit(deferred_op) => match deferred_op {
                DeferredOp::Gcm(ctx) => {
                    if let Some(buf) = self.output.take() {
                        self.apply_crypto_direction(self.encrypting.get());
                        self.start_gcm_crypt(ctx, buf);
                    } else {
                        self.state.set(State::Idle);
                    }
                }
                _ => {
                    self.state.set(State::Idle);
                }
            },
            State::DmaHeaderPadding(ctx) => {
                self.continue_dma_payload_setup(ctx);
            }
            State::DmaPayloadPadding(ctx) => {
                let block = self.get_output();
                self.dma_utils.dma_message_buff.replace(block);
                self.dma_start_tag_computation(ctx);
            }
            State::DmaFinalize(ctx) => {
                self.dma_gcm_ccm_finish(ctx);
            }
            State::Header(ctx) => {
                if !ctx.using_dma {
                    self.advance_aad_phase(ctx)
                }
            }
            State::Payload(ctx) => {
                if !ctx.using_dma {
                    self.advance_payload_phase(ctx)
                }
            }
            State::Final(ctx) => {
                if !ctx.using_dma {
                    self.final_phase(ctx)
                }
            }
            _ => {}
        }
    }

    pub(crate) fn handle_ccm_client(&self) {
        match self.state.get() {
            State::DmaCcmB1(ctx) => {
                self.ccm_dma_b1_finish(ctx);
            }
            State::DmaHeaderPadding(ctx) => {
                self.continue_dma_payload_setup(ctx);
            }
            State::DmaPayloadPadding(ctx) => {
                let block = self.get_output();
                self.dma_utils.dma_message_buff.replace(block);
                self.dma_start_tag_computation(ctx);
            }
            State::DmaFinalize(ctx) => {
                self.dma_gcm_ccm_finish(ctx);
            }
            State::CCMInit(mut ctx) => {
                let aad_offset = ctx.aad_offset;
                let start_idx = ctx.message_start;

                // on this branch we skip the header phase
                if aad_offset == start_idx {
                    // DMA version
                    if let (Some(dma), Some(in_ch), Some(out_ch)) = (
                        self.dma_utils.dma.get(),
                        self.dma_utils.dma_in_channel.get(),
                        self.dma_utils.dma_out_channel.get(),
                    ) {
                        if let Some(buf) = self.output.take() {
                            self.setup_dma_gcm_ccm(ctx, buf, in_ch, out_ch, dma);
                        }
                        // normal version using interrupts
                    } else {
                        self.registers.cr.modify(Control::GCMPH::Payload);
                        self.registers.cr.modify(Control::EN::SET);
                        self.state.set(State::Payload(ctx));
                        self.start_payload_phase(ctx);
                    }
                }
                // on this branch the header phase is continued
                else {
                    // the first block in the header phase needs to have the length of the header
                    // phase included
                    let mut b1 = [0u8; AES_BLOCK_SIZE];
                    let aad_len = start_idx - aad_offset;
                    let offset;
                    // The first block of CCM AAD (B1) must encode the total AAD length using
                    // specific byte markers.
                    match aad_len {
                        0..crate::aes::CCM_AAD_L16_MAX => {
                            let len_bytes = (aad_len as u16).to_be_bytes();
                            b1[0..2].copy_from_slice(&len_bytes);
                            offset = 2;
                        }
                        crate::aes::CCM_AAD_L16_MAX..=0xFFFFFFFF => {
                            b1[0] = crate::aes::CCM_AAD_L32_MARKER_0;
                            b1[1] = crate::aes::CCM_AAD_L32_MARKER_1;
                            let len_bytes = (aad_len as u32).to_be_bytes();
                            b1[2..6].copy_from_slice(&len_bytes);
                            offset = 6;
                        }
                        _ => {
                            // aad buffer size cannot exceed 0xFFFFFFFF on stm32
                            unreachable!("");
                        }
                    }
                    let block_len = (AES_BLOCK_SIZE - offset).min(aad_len);
                    ctx.current_idx = block_len;
                    self.state.set(State::Header(ctx));
                    self.registers.cr.modify(Control::GCMPH::Header);
                    self.registers.cr.modify(Control::EN::SET);
                    self.output.map(|output| {
                        b1[offset..offset + block_len]
                            .copy_from_slice(&output[aad_offset..aad_offset + block_len]);
                    });

                    self.write_padded_to_dinr(&b1);

                    if self.dma_utils.dma.get().is_some() {
                        ctx.using_dma = true;
                        self.state.set(State::DmaCcmB1(ctx));
                    }
                }
            }
            State::Header(ctx) => {
                if !ctx.using_dma {
                    self.advance_aad_phase(ctx)
                }
            }
            State::Payload(ctx) => {
                if !ctx.using_dma {
                    self.advance_payload_phase(ctx)
                }
            }
            State::Final(ctx) => {
                if !ctx.using_dma {
                    self.final_phase(ctx)
                }
            }
            _ => {}
        }
    }
}

impl<'a, K: AESKeySize> kernel::hil::symmetric_encryption::AESGCM<'a, K> for Aes<'a, K> {
    fn set_client(&'a self, client: &'a dyn GCMClient) {
        self.gcm_client.set(client);
    }

    /// AES GCM init phase and key set phase
    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        self.init_gcm();
        AES::set_key(self, key)?;
        Ok(())
    }

    /// Formats the provided nonce (12 bytes) into a 16-byte GCM
    /// initialization block and asynchronously configures the hardware.
    fn set_iv(&self, nonce: &[u8]) -> Result<(), ErrorCode> {
        // GCM standard nonce length is 12 bytes
        if nonce.len() != 12 {
            return Err(ErrorCode::INVAL);
        }

        if self.registers.cr.any_matching_bits_set(Control::EN::SET) {
            return Err(ErrorCode::BUSY);
        }
        let mut full_gcm_iv = [0u8; AES128_IV_SIZE];
        full_gcm_iv[..nonce.len()].copy_from_slice(nonce);
        full_gcm_iv[12..AES128_IV_SIZE].copy_from_slice(&2u32.to_be_bytes());
        AES::set_iv(self, &full_gcm_iv)?;

        self.registers.cr.modify(Control::EN::SET);

        Ok(())
    }

    /// Initiates a GCM encryption or decryption payload. If the hardware is currently busy initializing the
    /// GCM parameters (set_iv) or classic keys, this safely queues the operation inside the DeferredOp
    /// state machine to be executed automatically when initialization completes.
    fn crypt(
        &self,
        buf: &'static mut [u8],
        aad_offset: usize,
        message_offset: usize,
        message_len: usize,
        tag_len: usize,
        encrypting: bool,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        let state = self.state.get();

        // If GCMInit phase is ongoing, we can queue if it's GCMInit(DeferredOp::None)
        if !matches!(state, State::GCMInit(DeferredOp::None) | State::Idle) {
            return Err((ErrorCode::BUSY, buf));
        }

        if message_offset - aad_offset + message_len + tag_len > buf.len()
            || aad_offset > message_offset
            || (message_offset == aad_offset && message_len == 0)
        {
            return Err((ErrorCode::INVAL, buf));
        }
        let ctx = CryptoContext {
            aad_offset,
            message_start: message_offset,
            message_end: message_offset + message_len,
            current_idx: 0,
            tag_len,
            confidential: true,
            using_dma: false,
        };
        self.apply_crypto_direction(encrypting);
        if let State::GCMInit(_) = state {
            self.output.replace(buf);
            self.state.set(State::GCMInit(DeferredOp::Gcm(ctx)));
            return Ok(());
        }

        self.start_gcm_crypt(ctx, buf);
        Ok(())
    }
}

impl<'a, K: AESKeySize> kernel::hil::symmetric_encryption::AESCCM<'a, K> for Aes<'a, K> {
    fn set_client(&'a self, client: &'a dyn kernel::hil::symmetric_encryption::CCMClient) {
        self.ccm_client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> Result<(), ErrorCode> {
        self.init_ccm();
        AES::set_key(self, key)?;
        Ok(())
    }

    /// Validates and caches a CCM nonce (7 to 13 bytes). The actual hardware configuration
    /// is deferred until `crypt` is called, as the CCM initial block depends on the message
    /// and tag lengths.
    fn set_nonce(&self, nonce: &[u8]) -> Result<(), ErrorCode> {
        if nonce.len() < 7 || nonce.len() > 13 {
            return Err(ErrorCode::INVAL);
        }

        if self.registers.cr.any_matching_bits_set(Control::EN::SET) {
            return Err(ErrorCode::BUSY);
        }

        // save nonce length in iv[0]
        let mut iv = [0u8; AES128_IV_SIZE];
        iv[0] = nonce.len() as u8;
        iv[1..nonce.len() + 1].copy_from_slice(nonce);
        self.iv.set(iv);

        Ok(())
    }

    /// Formats the CCM B0 block, configures hardware flags, and initiates the cryptographic operation.
    fn crypt(
        &self,
        buf: &'static mut [u8],
        a_off: usize,
        m_off: usize,
        m_len: usize,
        mic_len: usize,
        confidential: bool,
        encrypting: bool,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.state.get() != State::Idle || self.registers.cr.is_set(Control::EN) {
            return Err((ErrorCode::BUSY, buf));
        }
        if m_off - a_off + m_len + mic_len > buf.len() || a_off > m_off {
            return Err((ErrorCode::INVAL, buf));
        }
        self.encrypting.set(encrypting);
        let ctx = CryptoContext {
            aad_offset: a_off,
            message_start: m_off,
            message_end: m_off + m_len,
            current_idx: 0,
            tag_len: mic_len,
            confidential,
            using_dma: false,
        };
        self.output.replace(buf);
        // should always work as this function is called after `set_nonce` which sets the IV
        // to a valid size
        let mut iv = self.iv.get();
        let iv_len = iv[0] as usize;

        let q = 15 - iv_len - 1;
        let m: u8 = (mic_len as u8 - 2) / 2;
        // Flags
        iv[0] = q as u8;
        iv[0] |= m << 3;
        if a_off != m_off {
            iv[0] |= 0b0100_0000;
        }
        let q_len = 15 - iv_len;

        let m_len_bytes = (m_len as u64).to_be_bytes();

        // Q
        iv[iv_len + 1..AES128_IV_SIZE].copy_from_slice(&m_len_bytes[(8 - q_len)..]);

        // write IV to registers
        self.write_iv_registers(&iv);

        if encrypting {
            self.registers.cr.modify(Control::MODE::Encrypt);
        } else {
            self.registers.cr.modify(Control::MODE::Decrypt);
        }
        self.state.set(State::CCMInit(ctx));
        self.registers.cr.modify(Control::EN::SET);

        Ok(())
    }
}
