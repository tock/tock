// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! HASH core computing unit performing digest calculations for various algorithms.

use core::cell::Cell;

use crate::dma::{ChannelId, Dma};
use crate::hash::regs::HashRegisters;
use crate::hash::regs::{CR, IMR, SR, STR};
use crate::hash::utils::{DataType, HashClient, Leftover, State, TransferMode};

use cortexm33::dma_fence::CortexMDmaFence;
use kernel::ErrorCode;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::crypto::digest::{Digest, DigestClient};
use kernel::processbuffer::ReadableProcessSlice;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::dma_slice::DmaSubSliceMut;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};

const LONG_HMAC_KEY_LEN: usize = 64;
pub const FIFO_SIZE: usize = 17 * 4;

pub struct Hash<'a> {
    regs: StaticRef<HashRegisters>,
    dma: OptionalCell<&'a Dma>,
    dma_channel: Cell<Option<ChannelId>>,
    dma_buffer: MapCell<DmaSubSliceMut<'static, u8>>,
    data_buffer: MapCell<SubSliceMut<'static, u8>>,
    transfer_mode: Cell<TransferMode>,
    state: Cell<Option<State>>,
    leftover: Leftover,
    data_length: Cell<usize>,
    key_length: Cell<usize>,
    client: OptionalCell<HashClient<'a>>,
    deferred_call: DeferredCall,
}

impl<'a> Hash<'a> {
    // Associates a DMA controller and channels with the HASH driver
    pub fn set_dma(
        hash: &'static Self,
        dma: &'a Dma,
        channel: ChannelId,
        buffer: &'static mut [u8],
    ) {
        hash.dma.set(dma);
        hash.dma_channel.set(Some(channel));
        hash.data_buffer.put(SubSliceMut::new(buffer));
        dma.set_client(channel, hash);
    }

    fn start_dma_transfer(
        &self,
        dma: &'a Dma,
        dma_channel: ChannelId,
        dma_buffer: SubSliceMut<'static, u8>,
    ) {
        let fence = unsafe { CortexMDmaFence::new() };
        // Convert subslice into DmaSlice
        let dma_slice = unsafe { DmaSubSliceMut::new(dma_buffer, fence) };
        // Extract the physical pointer and length for MMIO
        let ptr = dma_slice.as_mut_ptr() as u32;
        let len = dma_slice.len() as u32;

        // Save DmaSlice in the peripheral struct
        self.dma_buffer.replace(dma_slice);
        dma.setup(dma_channel, crate::dma::DmaPeripheral::Hash, ptr, len);
        self.regs.cr.modify(CR::DMAE::SET);
    }
}

impl Hash<'_> {
    pub fn new(base: StaticRef<HashRegisters>) -> Self {
        Self {
            regs: base,
            dma: OptionalCell::empty(),
            dma_channel: Cell::new(None),
            dma_buffer: MapCell::empty(),
            data_buffer: MapCell::empty(),
            transfer_mode: Cell::new(TransferMode::DirectStream),
            state: Cell::new(None),
            data_length: Cell::new(0),
            key_length: Cell::new(0),
            leftover: Leftover::new(),
            client: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
        }
    }

    pub(crate) fn handle_interupts(&self) {
        // This function contains the state machine around the HASH core that orchestrates
        // the whole process around the digest calculation
        //
        // Simple digest calculation:
        // Add -> Callback -> (if FIFO is not empty) PreRun -> Run -> Callback
        //
        // HMAC digest calculation:
        // HmacInit -> HmacPreAuth -> Add -> Callback ->
        // (if FIFO is not empty) PreRun -> Run -> HmacPostAuth -> HmacFinalize -> Callback

        let regs = self.regs;
        // Disable all the interrupts
        regs.imr.modify(IMR::DCIE::CLEAR + IMR::DINIE::CLEAR);
        if let Some(client) = self.client.get() {
            if let Some(state) = self.state.get() {
                // Is final digest ready?
                if regs.sr.is_set(SR::DCIS) {
                    match state {
                        State::Run(true, DataType::Input) => {
                            // | State::Run(true, DataType::OuterKey) => {
                            self.finish(self.get_digest(), client);
                        }
                        _ => self.finish(Err(ErrorCode::FAIL), client),
                    }
                }
                // Is FIFO ready?
                else if regs.sr.is_set(SR::DINIS) {
                    match state {
                        // State::Run(true, DataType::InnerKey) => {
                        //     self.state.set(state.update(Some(self.data_length.get())));
                        //     self.deferred_call.set();
                        // }
                        State::Run(true, DataType::Input) => {
                            self.state.set(state.update(Some(self.key_length.get())));
                            self.deferred_call.set();
                        }
                        State::Run(false, data_type) => match self.run(data_type) {
                            Ok(true) => self.state.set(state.update(None)),
                            Ok(false) => self.finish(Err(ErrorCode::FAIL), client),
                            Err(e) => self.finish(Err(e), client),
                        },
                        State::Add(left, data_type) => {
                            let res = (|| {
                                match self.transfer_mode.get() {
                                    TransferMode::DirectStream => {
                                        let (bytes_loaded, _) =
                                            self.load_data(client, data_type)?;
                                        let updated_length = left.checked_sub(bytes_loaded);
                                        self.state.set(state.update(updated_length));
                                        self.deferred_call.set();
                                        Ok(())
                                    }
                                    TransferMode::Dma(false) => {
                                        if self.leftover.is_full() {
                                            self.regs.din.set(self.leftover.to_le());
                                            let mut dma_buffer =
                                                self.data_buffer.take().ok_or(ErrorCode::FAIL)?;
                                            if let (Some(dma), Some(dma_channel)) =
                                                (self.dma.get(), self.dma_channel.get())
                                            {
                                                self.truncate_dma_subslice(&mut dma_buffer);
                                                self.start_dma_transfer(
                                                    dma,
                                                    dma_channel,
                                                    dma_buffer,
                                                );
                                                self.transfer_mode.set(TransferMode::Dma(true));
                                                Ok(())
                                            } else {
                                                Err(ErrorCode::NODEVICE)
                                            }
                                        } else {
                                            // This should never happen
                                            Err(ErrorCode::FAIL)
                                        }
                                    }
                                    _ => {
                                        // This should never happen
                                        Err(ErrorCode::FAIL)
                                    }
                                }
                            })();
                            if res.is_err() {
                                self.finish(res, client);
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn handle_dma_interrupt(&self) {
        let regs = self.regs;
        // Disable the DMA trigger to release the channel
        regs.cr.modify(CR::DMAE::CLEAR);
        if let Some(dma_slice) = self.dma_buffer.take() {
            let fence = unsafe { CortexMDmaFence::new() };
            let mut subslice = unsafe { dma_slice.take(fence) };
            subslice.reset();
            // Reset DMA readiness for new operations
            self.transfer_mode.set(TransferMode::Dma(false));
            self.data_buffer.put(subslice);
            self.deferred_call.set();
        }
    }

    /// Write digest back to client.
    ///
    /// Called when the final digest is ready.
    fn get_digest(&self) -> Result<(), ErrorCode> {
        if let Some(client) = self.client.get() {
            match client {
                HashClient::Md5(client) => {
                    client.write_digest(&mut |out| out.copy_from_slice(&self.output::<16>()))?
                }
                HashClient::Sha1(client) => {
                    client.write_digest(&mut |out| out.copy_from_slice(&self.output::<20>()))?
                }
                HashClient::Sha224(client) => {
                    client.write_digest(&mut |out| out.copy_from_slice(&self.output::<28>()))?
                }
                HashClient::Sha256(client) => {
                    client.write_digest(&mut |out| out.copy_from_slice(&self.output::<32>()))?
                }
            }
            Ok(())
        } else {
            Err(ErrorCode::FAIL)
        }
    }

    fn output<const LEN: usize>(&self) -> [u8; LEN] {
        let mut output = [0; LEN];
        for (bytes, reg) in output
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(self.regs.hr.iter())
        {
            bytes.copy_from_slice(&reg.get().to_be_bytes());
        }
        output
    }

    /// Load data into the accelerator.
    ///
    /// Handles both DMA and Direct Stream operations.
    fn load_data(
        &self,
        client: HashClient<'_>,
        _data_type: DataType,
    ) -> Result<(usize, bool), ErrorCode> {
        // let read_from_client = |buf| match data_type {
        //     DataType::Input => client.read_input(buf),
        //     DataType::InnerKey | DataType::OuterKey => client.read_key(buf),
        // };

        match self.transfer_mode.get() {
            // CPU
            TransferMode::DirectStream => {
                let fifo_space = (self.regs.sr.read(SR::NBWE) * 4) as usize;
                let mut bytes_read = fifo_space.min(FIFO_SIZE);
                client.read_input(&mut |buffer| {
                    bytes_read = bytes_read.min(buffer.len());
                    let buf = buffer.get(..bytes_read).ok_or(ErrorCode::NOMEM)?;
                    self.write_data_cpu(buf)?;
                    Ok(bytes_read)
                })?;
                Ok((bytes_read, true))
            }
            // DMA
            TransferMode::Dma(_) => {
                let mut dma_buffer = self.data_buffer.take().ok_or(ErrorCode::FAIL)?;
                dma_buffer.reset();
                let mut bytes_read = dma_buffer.len();
                client.read_input(&mut |buf| {
                    bytes_read = bytes_read.min(buf.len());
                    buf.copy_to_slice_or_err(dma_buffer.as_mut_slice())?;
                    Ok(bytes_read)
                })?;
                dma_buffer.slice(..bytes_read);

                if let (Some(dma), Some(dma_channel)) = (self.dma.get(), self.dma_channel.get()) {
                    if !self.leftover.is_empty() {
                        // If the leftover is not empty, it has to be filled first and written to the accelerator
                        if self.trim_dma_subslice(&mut dma_buffer) {
                            if dma_buffer.len() == 0 {
                                // Already finished with everything, no need for DMA transfer
                                return Ok((bytes_read, true));
                            }
                        } else {
                            // Failed to write, accelerator is busy
                            // Try to write later
                            self.data_buffer.put(dma_buffer);
                            return Ok((bytes_read, false));
                        }
                    }
                    // As DMA slice can be of arbitrary size whereas DMA can send only 32-bit words,
                    // it has to be ensured that DMA buffer contains only 32-bit words
                    if !dma_buffer.len().is_multiple_of(4) {
                        self.truncate_dma_subslice(&mut dma_buffer);
                        if dma_buffer.len() == 0 {
                            return Ok((bytes_read, true));
                        }
                    }
                    self.start_dma_transfer(dma, dma_channel, dma_buffer);
                    Ok((bytes_read, true))
                } else {
                    Err(ErrorCode::NODEVICE)
                }
            }
        }
    }

    /// Write data using CPU.
    ///
    /// Usually writes only words into the accelerator.
    ///
    /// Only the last iteration accumulates bytes into leftover.
    fn write_data_cpu(&self, buffer: &ReadableProcessSlice) -> Result<(), ErrorCode> {
        let regs = self.regs;
        let aligned_len = (buffer.len() >> 2) << 2;

        // Write 32-bit words
        if let Some(words) = buffer.get(..aligned_len) {
            for chunk in words.chunks(4) {
                let mut w = [0u8; 4];
                chunk.copy_to_slice_or_err(&mut w)?;
                regs.din.set(u32::from_le_bytes(w));
            }
        }

        // Accumulate leftover bytes
        // This code is invoked only in at the last iteration of reading data from client
        if let Some(left) = buffer.get(aligned_len..) {
            left.iter().map(|b| self.leftover.add(b.get()));
        }

        Ok(())
    }

    /// Trim the subslice to get rid of the old leftover bytes.
    ///
    /// Fill the leftover buffer with bytes from the beginning of subslice.
    /// Return boolean value showing if the write operation was successful
    /// and there is no need to wait for the interrupt
    /// when the FIFO is free.
    ///
    /// Slice the passed subslice respectively.
    fn trim_dma_subslice(&self, dma_buffer: &mut SubSliceMut<'_, u8>) -> bool {
        let bytes_to_write = self.leftover.bytes_left().min(dma_buffer.len());
        for data in dma_buffer[..bytes_to_write].iter() {
            self.leftover.add(*data);
        }
        dma_buffer.slice(bytes_to_write..);

        // Leftover buffer is full, it is time to empty it
        if self.leftover.is_full() {
            let regs = self.regs;
            if regs.sr.read(SR::NBWE) == 0 {
                // New data cannot be written at the moment
                // Wait for an interrupt when FIFO is empty
                return false;
            } else {
                regs.din.set(self.leftover.to_le());
                // Leftover was written successfully
                return true;
            }
        }
        true
    }

    /// Truncate the subslice to make its size divisible by 4.
    ///
    /// Fill the leftover buffer with bytes from the end of subslice.
    /// Slice the passed subslice respectively.
    fn truncate_dma_subslice(&self, dma_buffer: &mut SubSliceMut<'_, u8>) {
        let bytes_written = dma_buffer.len() % 4;
        for data in dma_buffer[dma_buffer.len() - bytes_written..].iter() {
            self.leftover.add(*data);
        }
        dma_buffer.slice(..dma_buffer.len() - bytes_written);
    }

    /// Starts the final digest computation.
    ///
    /// Responsible for writing the leftover and starting the computation.
    fn run(&self, data_type: DataType) -> Result<bool, ErrorCode> {
        // No computations without the algorithm set
        if self.client.get().is_none() {
            return Err(ErrorCode::INVAL);
        }
        let regs = self.regs;

        // If the leftover is not empty, write it right now
        if !self.leftover.is_empty() {
            // Check if it is possible to write it at the moment
            //
            // There might be some computations in action
            if !regs.sr.is_set(SR::BUSY) {
                regs.din.set(self.leftover.to_le());
            } else {
                // Final digest calculation hasn't started, wait for the
                return Ok(false);
            }
        }

        if let Some(state) = self.state.take() {
            self.state.set(state.update(None));
        } else {
            return Err(ErrorCode::FAIL);
        }

        // Start the digest calculation
        let data_length = match data_type {
            DataType::Input => {
                if self.key_length.get() > 0 {
                    regs.imr.modify(IMR::DINIE::SET);
                } else {
                    regs.imr.modify(IMR::DCIE::SET);
                }
                self.data_length.get() as u32
            } // DataType::InnerKey => {
              //     regs.imr.modify(IMR::DINIE::SET);
              //     self.key_length.get() as u32
              // }
              // DataType::OuterKey => {
              //     regs.imr.modify(IMR::DCIE::SET);
              //     self.key_length.get() as u32
              // }
        };
        regs.str.modify(STR::NBLW.val((data_length % 4) * 8));
        regs.str.modify(STR::DCAL::SET);

        Ok(true)
    }

    // /// Helper function for validating requested algorithms and returning the corresponding register value.
    // fn parse_algo(&self, algorithm: Algorithm) -> Result<FieldValue<u32, CR::Register>, ErrorCode> {
    //     match algorithm {
    //         Algorithm::Md5 => Ok(CR::ALGO::MD5),
    //         Algorithm::Sha1 => Ok(CR::ALGO::SHA_1),
    //         Algorithm::Sha224 => Ok(CR::ALGO::SHA2_224),
    //         Algorithm::Sha256 => Ok(CR::ALGO::SHA2_256),
    //         _ => Err(ErrorCode::NOSUPPORT),
    //     }
    // }

    /// Finish the hashing / HMAC operation.
    ///
    /// Called to signal the client that the operation is done regardless of its result.
    fn finish(&self, result: Result<(), ErrorCode>, client: HashClient<'_>) {
        self.state.take();
        client.hash_done(result);
    }
}

impl crate::dma::DmaClient for Hash<'_> {
    fn transfer_done(&self, channel: ChannelId) {
        if let Some(ch) = self.dma_channel.get() {
            if ch == channel {
                self.handle_dma_interrupt();
            }
        }
    }
}

impl DeferredCallClient for Hash<'_> {
    fn handle_deferred_call(&self) {
        if let (Some(state), Some(client)) = (self.state.get(), self.client.get()) {
            let result = (|| {
                match state {
                    State::Add(left, data_type) => {
                        let (bytes_loaded, is_dma_ready) = self.load_data(client, data_type)?;
                        let updated_length =
                            left.checked_sub(bytes_loaded).ok_or(ErrorCode::FAIL)?;
                        if matches!(self.transfer_mode.get(), TransferMode::Dma(_)) {
                            self.transfer_mode.set(TransferMode::Dma(is_dma_ready));
                        }
                        let new_state = state.update(Some(updated_length));
                        self.state.set(new_state);
                        match (self.transfer_mode.get(), data_type) {
                            (TransferMode::DirectStream, _) => {
                                // | (_, DataType::InnerKey | DataType::OuterKey) => {
                                if !self.regs.sr.is_set(SR::BUSY) {
                                    self.deferred_call.set();
                                } else {
                                    self.regs.imr.modify(IMR::DINIE::SET);
                                }
                            }
                            (TransferMode::Dma(false), DataType::Input) => {
                                self.regs.imr.modify(IMR::DINIE::SET);
                            }
                            (TransferMode::Dma(true), DataType::Input) => {
                                if let Some(State::Run(_, _)) = new_state {
                                    self.deferred_call.set();
                                }
                            }
                        }

                        Ok(())
                    }
                    State::Run(false, data_type) => {
                        // no need for setting deferred call, wait for an interrupt
                        self.run(data_type).map(|is_sent| {
                            if !is_sent {
                                self.regs.imr.modify(IMR::DINIE::SET);
                            }
                        })
                    }
                    State::Run(true, _) => Err(ErrorCode::FAIL),
                }
            })();
            if result.is_err() {
                self.finish(result, client);
            }
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

// impl Digest for Hash<'_> {
//     fn hash(&self, algorithm: Algorithm, len: usize) -> Result<(), ErrorCode> {
//         if self.state.get().is_some() {
//             return Err(ErrorCode::BUSY);
//         }
//
//         let algo_val = self.parse_algo(algorithm)?;
//         let regs = self.regs;
//         self.data_length.set(len);
//         self.state.set(Some(State::Add(len, DataType::Input)));
//         self.algorithm.set(Some(algorithm));
//
//         regs.cr.modify(
//             algo_val + CR::MDMAT::SET + CR::DATATYPE::_8bitData + CR::MODE::CLEAR + CR::INIT::SET,
//         );
//
//         if self.dma.is_some() && self.dma_channel.get().is_some() {
//             self.transfer_mode.set(TransferMode::Dma(false));
//         } else {
//             self.transfer_mode.set(TransferMode::DirectStream);
//         }
//         self.deferred_call.set();
//         Ok(())
//     }
//
//     fn clear_data(&self) {
//         if self.state.get().is_none() {
//             // No operation at the moment -> reset the peripheral keeping the settings
//             self.regs.cr.modify(CR::INIT::SET);
//             self.data_length.take();
//             self.key_length.take();
//             self.data_buffer.map(|buf| buf.reset());
//             self.leftover.reset();
//         } else {
//             // Set the cancellation flag and wait for the interrupt / deferred call
//             self.cancelled.set(true);
//         }
//     }
//
//     fn set_client(&self, client: &'static dyn kernel::hil::crypto::digest::Client) {
//         self.client.set(HashClient::Hash(client));
//     }
// }

// impl Hmac for Hash<'_> {
//     fn authenticate(
//         &self,
//         algorithm: Algorithm,
//         input_len: usize,
//         key_len: usize,
//     ) -> Result<(), ErrorCode> {
//         if self.state.get().is_some() {
//             return Err(ErrorCode::BUSY);
//         }
//
//         let algo_val = self.parse_algo(algorithm)?;
//
//         let key_val = if key_len > LONG_HMAC_KEY_LEN {
//             CR::LKEY::SET
//         } else {
//             CR::LKEY::CLEAR
//         };
//         self.key_length.set(key_len);
//         self.data_length.set(input_len);
//         self.regs.cr.modify(
//             algo_val
//                 + key_val
//                 + CR::MDMAT::SET
//                 + CR::DATATYPE::_8bitData
//                 + CR::MODE::SET
//                 + CR::INIT::SET,
//         );
//
//         self.state
//             .set(Some(State::Add(key_len, DataType::InnerKey)));
//         self.algorithm.set(Some(algorithm));
//
//         if self.dma.is_some() && self.dma_channel.get().is_some() && self.data_buffer.is_some() {
//             self.transfer_mode.set(TransferMode::Dma(false));
//         } else {
//             self.transfer_mode.set(TransferMode::DirectStream);
//         }
//         self.deferred_call.set();
//         Ok(())
//     }
//
//     fn set_hmac_client(&self, client: &'static dyn HmacClient) {
//         self.client.set(HashClient::Hmac(client));
//     }
// }
