// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

//! HASH core computing unit performing digest calculations for various modes.

use core::cell::Cell;
use core::cmp::min;
use core::ops::Index;

use crate::dma::{ChannelId, Dma};
use crate::hash::md5::Md5Adapter;
use crate::hash::regs::HashRegisters;
use crate::hash::regs::{CR, IMR, SR, STR};
use crate::hash::sha1::Sha1Adapter;
use crate::hash::sha224::Sha224Adapter;
use crate::hash::sha256::Sha256Adapter;
use crate::hash::utils::{DataWidth, HashAdapter, HmacKey, Leftover, Mode, State};

use cortexm33::dma_fence::CortexMDmaFence;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::digest;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::dma_slice::{DmaSubSlice, DmaSubSliceMut, DmaSubSliceMutImmut};
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut, SubSliceMutImmut};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

const LONG_HMAC_KEY_LEN: usize = 64;

pub struct Hash<'a> {
    regs: StaticRef<HashRegisters>,
    dma: OptionalCell<&'a Dma>,
    dma_channel: Cell<Option<ChannelId>>,
    dma_buffer: MapCell<DmaSubSliceMutImmut<'static, u8>>,
    mode: Cell<Option<Mode>>,
    state: Cell<Option<State>>,
    data_width: Cell<Option<DataWidth>>,
    hmac_key: HmacKey,
    data: Cell<Option<SubSliceMutImmut<'static, u8>>>,
    leftover: Leftover,
    verify: Cell<bool>,
    cancelled: Cell<bool>,
    adapter: OptionalCell<HashAdapter<'a>>,
    digest: OptionalCell<&'static mut [u8]>,
    deferred_call: DeferredCall,
}

impl<'a> Hash<'a> {
    // Associates a DMA controller and channels with the HASH driver
    pub(crate) fn set_dma(hash: &'static Self, dma: &'a Dma, channel: ChannelId) {
        hash.dma.set(dma);
        hash.dma_channel.set(Some(channel));
        dma.set_client(channel, hash);
    }

    fn start_dma_transfer(&self, dma: &'a Dma) -> Result<(), ()> {
        if let Some(mut data) = self.data.take() {
            let mut can_start = true;
            if !self.leftover.is_empty() {
                // Imagine there is a situation when the FIFO is full,
                // and we cannot write more
                let (count, start) = self.trim_subslice(&data, data.len());
                data.slice(count..);
                if let Some(s) = start {
                    can_start = s;
                }
            }
            // Truncate
            let count = self.truncate_subslice(&data, data.len());
            data.slice(..data.len() - count);
            if data.len() == 0 {
                if can_start {
                    self.data.set(Some(data));
                    self.deferred_call.set();
                }
                return Ok(());
            }

            // Trigger HASH
            if let Some(ch) = self.dma_channel.get() {
                let regs = self.regs;
                // Hardware fence
                // Load data only if we have a channel
                // Otherwise, it is meaningless
                let fence = unsafe { CortexMDmaFence::new() };
                // Convert subslice into DmaSlice
                let (dma_slice, ptr, len) = match data {
                    SubSliceMutImmut::Immutable(d) => {
                        let dma_slice = DmaSubSlice::new(d, fence);
                        // Extract the physical pointer and length for MMIO
                        let ptr = dma_slice.as_ptr() as u32;
                        let len = dma_slice.len() as u32;
                        self.data.set(Some(data));
                        (DmaSubSliceMutImmut::Immutable(dma_slice), ptr, len)
                    }
                    SubSliceMutImmut::Mutable(d) => {
                        let dma_slice = unsafe { DmaSubSliceMut::new(d, fence) };
                        // Extract the physical pointer and length for MMIO
                        let ptr = dma_slice.as_mut_ptr() as u32;
                        let len = dma_slice.len() as u32;
                        (DmaSubSliceMutImmut::Mutable(dma_slice), ptr, len)
                    }
                };
                // Save DmaSlice in the peripheral struct
                self.dma_buffer.replace(dma_slice);
                dma.setup(ch, crate::dma::DmaPeripheral::Hash, ptr, len);

                regs.imr.modify(IMR::DINIE::SET);

                if can_start {
                    regs.cr.modify(CR::DMAE::SET);
                }

                Ok(())
            } else {
                Err(())
            }
        } else {
            // No data found
            Err(())
        }
    }
}

impl Hash<'_> {
    pub fn new(base: StaticRef<HashRegisters>) -> Self {
        Self {
            regs: base,
            dma: OptionalCell::empty(),
            dma_channel: Cell::new(None),
            dma_buffer: MapCell::empty(),
            mode: Cell::new(None),
            state: Cell::new(None),
            data_width: Cell::new(None),
            data: Cell::new(None),
            hmac_key: HmacKey::new(),
            verify: Cell::new(false),
            cancelled: Cell::new(false),
            leftover: Leftover::new(),
            digest: OptionalCell::empty(),
            adapter: OptionalCell::empty(),
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

        if let Some(state) = self.state.get() {
            // Has digest calculation completed?
            if regs.sr.read(SR::DCIS) != 0 {
                // If key is present, then we need to process it accordingly
                match (state, self.cancelled.get()) {
                    (State::HmacFinalize | State::Run, true) => {
                        regs.cr.modify(CR::INIT::SET);
                        self.leftover.empty();
                        self.adapter.map(|adapter| {
                            if let Some(digest) = self.digest.take() {
                                if self.verify.get() {
                                    match adapter {
                                        HashAdapter::Md5(md5) => {
                                            md5.verification_done(
                                                Err(kernel::ErrorCode::CANCEL),
                                                digest,
                                            );
                                        }
                                        HashAdapter::Sha1(sha1) => {
                                            sha1.verification_done(
                                                Err(kernel::ErrorCode::CANCEL),
                                                digest,
                                            );
                                        }
                                        HashAdapter::Sha224(sha224) => {
                                            sha224.verification_done(
                                                Err(kernel::ErrorCode::CANCEL),
                                                digest,
                                            );
                                        }
                                        HashAdapter::Sha256(sha256) => {
                                            sha256.verification_done(
                                                Err(kernel::ErrorCode::CANCEL),
                                                digest,
                                            );
                                        }
                                    }
                                } else {
                                    match adapter {
                                        HashAdapter::Md5(md5) => {
                                            md5.hash_done(Err(kernel::ErrorCode::CANCEL), digest);
                                        }
                                        HashAdapter::Sha1(sha1) => {
                                            sha1.hash_done(Err(kernel::ErrorCode::CANCEL), digest);
                                        }
                                        HashAdapter::Sha224(sha224) => {
                                            sha224
                                                .hash_done(Err(kernel::ErrorCode::CANCEL), digest);
                                        }
                                        HashAdapter::Sha256(sha256) => {
                                            sha256
                                                .hash_done(Err(kernel::ErrorCode::CANCEL), digest);
                                        }
                                    }
                                }
                            }
                        });
                    }
                    (State::HmacFinalize | State::Run, false) => {
                        // It's time to return data to the client
                        self.return_data();
                    }
                    _ => (),
                }
            // Is data input buffer empty?
            } else if regs.sr.read(SR::DINIS) != 0 {
                match (state, self.cancelled.get()) {
                    (State::Add | State::HmacInit | State::HmacPreAuth, true) => {
                        self.cancelled.set(false);
                        self.leftover.empty();
                        regs.cr.modify(CR::INIT::SET);
                        self.adapter.map(|adapter| {
                            self.data.take().map(|buf| match buf {
                                SubSliceMutImmut::Immutable(mut b) => {
                                    b.reset();
                                    match adapter {
                                        HashAdapter::Md5(md5) => {
                                            md5.add_data_done(Err(kernel::ErrorCode::CANCEL), b)
                                        }

                                        HashAdapter::Sha1(sha1) => {
                                            sha1.add_data_done(Err(kernel::ErrorCode::CANCEL), b)
                                        }

                                        HashAdapter::Sha224(sha224) => {
                                            sha224.add_data_done(Err(kernel::ErrorCode::CANCEL), b)
                                        }

                                        HashAdapter::Sha256(sha256) => {
                                            sha256.add_data_done(Err(kernel::ErrorCode::CANCEL), b)
                                        }
                                    }
                                }
                                SubSliceMutImmut::Mutable(mut b) => {
                                    b.reset();
                                    match adapter {
                                        HashAdapter::Md5(md5) => {
                                            md5.add_mut_data_done(Err(kernel::ErrorCode::CANCEL), b)
                                        }

                                        HashAdapter::Sha1(sha1) => sha1
                                            .add_mut_data_done(Err(kernel::ErrorCode::CANCEL), b),

                                        HashAdapter::Sha224(sha224) => sha224
                                            .add_mut_data_done(Err(kernel::ErrorCode::CANCEL), b),

                                        HashAdapter::Sha256(sha256) => sha256
                                            .add_mut_data_done(Err(kernel::ErrorCode::CANCEL), b),
                                    }
                                }
                            })
                        });
                    }
                    (
                        State::Run | State::PreRun | State::HmacPostAuth | State::HmacFinalize,
                        true,
                    ) => {
                        self.cancelled.set(false);
                        self.leftover.empty();
                        self.regs.cr.modify(CR::INIT::SET);
                        self.adapter.map(|adapter| {
                            if let Some(digest) = self.digest.take() {
                                if self.verify.get() {
                                    match adapter {
                                        HashAdapter::Md5(md5) => {
                                            md5.verification_done(
                                                Err(kernel::ErrorCode::CANCEL),
                                                digest,
                                            );
                                        }
                                        HashAdapter::Sha1(sha1) => {
                                            sha1.verification_done(
                                                Err(kernel::ErrorCode::CANCEL),
                                                digest,
                                            );
                                        }
                                        HashAdapter::Sha224(sha224) => {
                                            sha224.verification_done(
                                                Err(kernel::ErrorCode::CANCEL),
                                                digest,
                                            );
                                        }
                                        HashAdapter::Sha256(sha256) => {
                                            sha256.verification_done(
                                                Err(kernel::ErrorCode::CANCEL),
                                                digest,
                                            );
                                        }
                                    }
                                } else {
                                    match adapter {
                                        HashAdapter::Md5(md5) => {
                                            md5.hash_done(Err(kernel::ErrorCode::CANCEL), digest);
                                        }
                                        HashAdapter::Sha1(sha1) => {
                                            sha1.hash_done(Err(kernel::ErrorCode::CANCEL), digest);
                                        }
                                        HashAdapter::Sha224(sha224) => {
                                            sha224
                                                .hash_done(Err(kernel::ErrorCode::CANCEL), digest);
                                        }
                                        HashAdapter::Sha256(sha256) => {
                                            sha256
                                                .hash_done(Err(kernel::ErrorCode::CANCEL), digest);
                                        }
                                    }
                                }
                            }
                        });
                    }
                    (State::Add, false) => {
                        if self.dma.is_some() {
                            // Theoretically, there can be a situation
                            // when the FIFO is already filled, but leftovers have to be written
                            // The DMA transfer cannot start until all the previous data is loaded
                            // to the peripheral so that calculated hash is not corrupted.
                            if self.leftover.is_full() {
                                regs.din.set(self.leftover.to_le());
                                // It is the appropriate moment to start the DMA transfer
                                regs.cr.modify(CR::DMAE::SET);
                            }
                        } else {
                            if !self.data_progress() {
                                self.deferred_call.set();
                            } else {
                                regs.imr.modify(IMR::DINIE::SET);
                            }
                        }
                    }
                    (State::HmacInit | State::HmacPostAuth, false) => {
                        self.load_key();
                    }
                    (State::PreRun, false) => {
                        if !self.leftover.is_empty() {
                            self.flush_leftover();
                        } else {
                            // No padding
                            regs.str.modify(STR::NBLW.val(0));
                        }
                        // Enable interrupts
                        if self.hmac_key.is_stored() {
                            // Reset the index to prepare for the outer key loading
                            self.hmac_key.reset_index();
                            regs.imr.modify(IMR::DINIE::SET + IMR::DCIE::SET);
                        } else {
                            regs.imr.modify(IMR::DCIE::SET);
                        }

                        self.state.set(Some(State::Run));
                        // Start the final digest calculation
                        regs.str.modify(STR::DCAL::SET);
                    }
                    (State::Run, false) => {
                        if !self.leftover.is_empty() {
                            self.flush_leftover();

                            if self.hmac_key.is_stored() {
                                self.hmac_key.reset_index();
                                regs.imr.modify(IMR::DINIE::SET + IMR::DCIE::SET);
                            } else {
                                regs.imr.modify(IMR::DCIE::SET);
                            }

                            regs.str.modify(STR::DCAL::SET);
                        } else {
                            self.state.set(Some(State::HmacPostAuth));
                            self.load_key();
                        }
                    }
                    (State::HmacPreAuth, false) => {
                        self.state.set(Some(State::Add));

                        if let Some(dma) = self.dma.get() {
                            if self.start_dma_transfer(dma).is_err() {
                                self.adapter.map(|adapter| {
                                    self.data.take().map(|buf| match buf {
                                        SubSliceMutImmut::Immutable(mut b) => {
                                            b.reset();
                                            match adapter {
                                                HashAdapter::Md5(md5) => md5
                                                    .add_data_done(Err(kernel::ErrorCode::FAIL), b),
                                                HashAdapter::Sha1(sha1) => sha1
                                                    .add_data_done(Err(kernel::ErrorCode::FAIL), b),
                                                HashAdapter::Sha224(sha224) => sha224
                                                    .add_data_done(Err(kernel::ErrorCode::FAIL), b),
                                                HashAdapter::Sha256(sha256) => sha256
                                                    .add_data_done(Err(kernel::ErrorCode::FAIL), b),
                                            }
                                        }
                                        SubSliceMutImmut::Mutable(mut b) => {
                                            b.reset();
                                            match adapter {
                                                HashAdapter::Md5(md5) => md5.add_mut_data_done(
                                                    Err(kernel::ErrorCode::FAIL),
                                                    b,
                                                ),
                                                HashAdapter::Sha1(sha1) => sha1.add_mut_data_done(
                                                    Err(kernel::ErrorCode::FAIL),
                                                    b,
                                                ),
                                                HashAdapter::Sha224(sha224) => sha224
                                                    .add_mut_data_done(
                                                        Err(kernel::ErrorCode::FAIL),
                                                        b,
                                                    ),
                                                HashAdapter::Sha256(sha256) => sha256
                                                    .add_mut_data_done(
                                                        Err(kernel::ErrorCode::FAIL),
                                                        b,
                                                    ),
                                            }
                                        }
                                    })
                                });
                            }
                        } else {
                            if !self.data_progress() {
                                // we added all the data
                                self.deferred_call.set();
                            } else {
                                regs.imr.modify(IMR::DINIE::SET);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    pub(crate) fn handle_dma_interrupt(&self) {
        let regs = self.regs;
        // Disable the DMA trigger to release the channel
        regs.cr.modify(CR::DMAE::CLEAR);
        if let Some(dma_slice) = self.dma_buffer.take() {
            if let Some(State::Add) = self.state.get() {
                self.state.take();
                match dma_slice {
                    DmaSubSliceMutImmut::Immutable(b) => {
                        let mut subslice = b.as_sub_slice();
                        if self.cancelled.get() {
                            self.cancelled.set(false);
                            self.leftover.empty();
                            self.regs.cr.modify(CR::INIT::SET);
                            subslice.reset();
                            self.adapter.map(|adapter| match adapter {
                                HashAdapter::Md5(md5) => {
                                    md5.add_data_done(Err(ErrorCode::CANCEL), subslice);
                                }
                                HashAdapter::Sha1(sha1) => {
                                    sha1.add_data_done(Err(ErrorCode::CANCEL), subslice);
                                }
                                HashAdapter::Sha224(sha224) => {
                                    sha224.add_data_done(Err(ErrorCode::CANCEL), subslice);
                                }
                                HashAdapter::Sha256(sha256) => {
                                    sha256.add_data_done(Err(ErrorCode::CANCEL), subslice);
                                }
                            });
                        } else {
                            // ugly line of code
                            subslice.slice(0..0);
                            self.adapter.map(|adapter| match adapter {
                                HashAdapter::Md5(md5) => {
                                    md5.add_data_done(Ok(()), subslice);
                                }
                                HashAdapter::Sha1(sha1) => {
                                    sha1.add_data_done(Ok(()), subslice);
                                }
                                HashAdapter::Sha224(sha224) => {
                                    sha224.add_data_done(Ok(()), subslice);
                                }
                                HashAdapter::Sha256(sha256) => {
                                    sha256.add_data_done(Ok(()), subslice);
                                }
                            });
                        }
                    }
                    DmaSubSliceMutImmut::Mutable(b) => {
                        let fence = unsafe { CortexMDmaFence::new() };
                        let mut subslice = unsafe { b.take(fence) };
                        if self.cancelled.get() {
                            self.cancelled.set(false);
                            self.leftover.empty();
                            self.regs.cr.modify(CR::INIT::SET);
                            subslice.reset();
                            self.adapter.map(|adapter| match adapter {
                                HashAdapter::Md5(md5) => {
                                    md5.add_mut_data_done(Err(ErrorCode::CANCEL), subslice);
                                }
                                HashAdapter::Sha1(sha1) => {
                                    sha1.add_mut_data_done(Err(ErrorCode::CANCEL), subslice);
                                }
                                HashAdapter::Sha224(sha224) => {
                                    sha224.add_mut_data_done(Err(ErrorCode::CANCEL), subslice);
                                }
                                HashAdapter::Sha256(sha256) => {
                                    sha256.add_mut_data_done(Err(ErrorCode::CANCEL), subslice);
                                }
                            });
                        } else {
                            // ugly line of code
                            subslice.slice(0..0);
                            self.adapter.map(|adapter| match adapter {
                                HashAdapter::Md5(md5) => {
                                    md5.add_mut_data_done(Ok(()), subslice);
                                }
                                HashAdapter::Sha1(sha1) => {
                                    sha1.add_mut_data_done(Ok(()), subslice);
                                }
                                HashAdapter::Sha224(sha224) => {
                                    sha224.add_mut_data_done(Ok(()), subslice);
                                }
                                HashAdapter::Sha256(sha256) => {
                                    sha256.add_mut_data_done(Ok(()), subslice);
                                }
                            });
                        }
                    }
                }
            }
        }
    }

    /// Sends the callback with the digest to the adapter once either hashing or verification is done.
    fn return_data(&self) {
        self.adapter.map(|adapter| {
            let regs = self.regs;
            if let Some(digest) = self.digest.take() {
                // We need to compare the result with the digest received before.
                // If there is no operation, we react in no manner.
                if let Some(mode) = self.mode.get() {
                    if self.verify.get() {
                        let mut equal = true;
                        for i in 0..mode.get_digest_len() {
                            let d = regs.hr[i].get().to_be_bytes();
                            let idx = i * 4;
                            if digest[idx + 0] != d[0]
                                || digest[idx + 1] != d[1]
                                || digest[idx + 2] != d[2]
                                || digest[idx + 3] != d[3]
                            {
                                equal = false;
                            }
                        }

                        // Reset the peripheral
                        regs.cr.modify(CR::INIT::SET);
                        self.state.take();
                        if self.hmac_key.is_stored() {
                            self.hmac_key.reset_index();
                        }
                        self.verify.set(false);
                        match adapter {
                            HashAdapter::Md5(md5) => {
                                md5.verification_done(Ok(equal), digest);
                            }
                            HashAdapter::Sha1(sha1) => {
                                sha1.verification_done(Ok(equal), digest);
                            }
                            HashAdapter::Sha224(sha224) => {
                                sha224.verification_done(Ok(equal), digest);
                            }
                            HashAdapter::Sha256(sha256) => {
                                sha256.verification_done(Ok(equal), digest);
                            }
                        }
                    } else {
                        for i in 0..mode.get_digest_len() {
                            let d = regs.hr[i].get().to_be_bytes();
                            let idx = i * 4;

                            digest[idx + 0] = d[0];
                            digest[idx + 1] = d[1];
                            digest[idx + 2] = d[2];
                            digest[idx + 3] = d[3];
                        }

                        // reset the peripheral
                        regs.cr.modify(CR::INIT::SET);
                        // release the peripheral
                        self.state.take();
                        if self.hmac_key.is_stored() {
                            self.hmac_key.reset_index();
                        }
                        match adapter {
                            HashAdapter::Md5(md5) => {
                                md5.hash_done(Ok(()), digest);
                            }
                            HashAdapter::Sha1(sha1) => {
                                sha1.hash_done(Ok(()), digest);
                            }
                            HashAdapter::Sha224(sha224) => {
                                sha224.hash_done(Ok(()), digest);
                            }
                            HashAdapter::Sha256(sha256) => {
                                sha256.hash_done(Ok(()), digest);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Transfer bytes to FIFO for processing.
    ///
    /// Returns the number of bytes written.
    fn process(&self, data: &dyn Index<usize, Output = u8>, count: usize) -> usize {
        let regs = self.regs;
        // Only 32-bit words can be written in the FIFO
        let words_num = count / 4;
        // Send the 32-bit words first
        for i in 0..words_num {
            if regs.sr.read(SR::NBWE) == 0 {
                // FIFO is full, stop here
                return i * 4;
            }
            let data_idx = i * 4;
            let d = u32::from_le_bytes([
                data[data_idx + 0],
                data[data_idx + 1],
                data[data_idx + 2],
                data[data_idx + 3],
            ]);

            regs.din.set(d);
        }
        // Handle leftover bytes
        // by this moment, it is ensured that the leftover buffer is empty.
        if !count.is_multiple_of(4) {
            for i in 0..(count % 4) {
                if regs.sr.read(SR::NBWE) == 0 {
                    // FIFO is full, stop here
                    return i + words_num;
                }
                let data_idx = (count - (count % 4)) + i;
                // Accumulate leftover bytes
                self.leftover.add(data[data_idx]);
            }
        }
        count
    }

    /// Trim the subslice to get rid of the old leftover bytes.
    ///
    /// Fill the leftover buffer with bytes from the beginning of subslice.
    /// Return the tuple of number of bytes written and boolean values showing
    /// if the write operation was successful and there is no need to wait for the interrupt
    /// when the FIFO is free.
    fn trim_subslice(
        &self,
        data: &dyn Index<usize, Output = u8>,
        count: usize,
    ) -> (usize, Option<bool>) {
        let bytes_to_write = min(self.leftover.bytes_left(), count);
        let mut is_write_successful = None;
        for data_idx in 0..bytes_to_write {
            self.leftover.add(data[data_idx]);
        }
        // Leftover buffer is full, it is time to empty it
        if self.leftover.is_full() {
            let regs = self.regs;
            if regs.sr.read(SR::NBWE) == 0 {
                // New data cannot be written at the moment
                // Wait for an interrupt when FIFO is empty
                is_write_successful = Some(false);
            } else {
                regs.din.set(self.leftover.to_le());
                // Leftover was written successfully
                is_write_successful = Some(true);
            }
        }
        (bytes_to_write, is_write_successful)
    }

    /// Truncate the subslice to make its size divisible by 4.
    ///
    /// Fill the leftover buffer with bytes from the end of subslice.
    /// Return the number of bytes written.
    fn truncate_subslice(&self, data: &dyn Index<usize, Output = u8>, count: usize) -> usize {
        let bytes_written = count % 4;
        for i in 0..bytes_written {
            let data_idx = (count - bytes_written) + i;
            self.leftover.add(data[data_idx]);
        }
        bytes_written
    }

    /// Track the status of data being loaded by software.
    ///
    /// Return true if processing more data, false if the buffer
    /// is completely processed.
    fn data_progress(&self) -> bool {
        self.data.take().is_some_and(|buf| match buf {
            SubSliceMutImmut::Immutable(mut b) => {
                // If it is already empty, all the data has been already processed
                if b.len() == 0 {
                    self.data.set(Some(SubSliceMutImmut::Immutable(b)));
                    false
                } else {
                    // If the leftover buffer is not empty,
                    // it has to be filled with new bytes and written to the FIFO right now.
                    //
                    // Otherwise, go directly to writing 32-bit words
                    if !self.leftover.is_empty() {
                        let (count, is_write_successful) = self.trim_subslice(&b, b.len());
                        b.slice(count..);
                        // FIFO is full, wait for the interrupt when it is empty
                        if let Some(false) = is_write_successful {
                            self.data.set(Some(SubSliceMutImmut::Immutable(b)));
                            return true;
                        }
                    }
                    let count = self.process(&b, b.len());
                    b.slice(count..);

                    if b.len() == 0 {
                        // Finished processing
                        self.data.set(Some(SubSliceMutImmut::Immutable(b)));
                        false
                    } else {
                        // Continue the process when the FIFO is ready
                        self.data.set(Some(SubSliceMutImmut::Immutable(b)));
                        true
                    }
                }
            }
            SubSliceMutImmut::Mutable(mut b) => {
                // If it is already empty, all the data has been already processed
                if b.len() == 0 {
                    self.data.set(Some(SubSliceMutImmut::Mutable(b)));
                    false
                } else {
                    if !self.leftover.is_empty() {
                        // If the leftover buffer is not empty,
                        // it has to be filled with new bytes and written to the FIFO right now.
                        //
                        // Otherwise, go directly to writing 32-bit words.
                        let (count, is_write_successful) = self.trim_subslice(&b, b.len());
                        b.slice(count..);
                        // FIFO is full, wait for the interrupt once the FIFO is empty.
                        if let Some(false) = is_write_successful {
                            self.data.set(Some(SubSliceMutImmut::Mutable(b)));
                            return true;
                        }
                    }
                    let count = self.process(&b, b.len());
                    b.slice(count..);
                    if b.len() == 0 {
                        // Finished processing
                        self.data.set(Some(SubSliceMutImmut::Mutable(b)));
                        false
                    } else {
                        // Continue the process when the FIFO is ready
                        self.data.set(Some(SubSliceMutImmut::Mutable(b)));
                        true
                    }
                }
            }
        })
    }

    /// Load the key to the FIFO
    ///
    /// Similar to `process()`, but without trimming and truncation
    // Loaded only by software, no DMA support for key loading
    fn load_key(&self) {
        let regs = self.regs;

        // Treat the key as simple data
        self.hmac_key.key.map(|buf| {
            let count = self.process(buf, self.hmac_key.left_to_load());
            self.hmac_key.index.update(|idx| idx + count);
        });
        //  It's time to compute digest on it.
        if self.hmac_key.left_to_load() == 0 {
            self.hmac_key.reset_index();
            regs.imr.modify(IMR::DINIE::SET);
            if !self.leftover.is_empty() {
                // FIXME(frihetselsker): handle better
                // Are we sure that we can load more data
                self.flush_leftover();
            }

            self.state.update(|s| match s {
                // inner key
                Some(State::HmacInit) => Some(State::HmacPreAuth),
                // outer key
                Some(State::HmacPostAuth) => Some(State::HmacFinalize),
                _ => s,
            });
            // Set the mask for the key
            regs.str
                .modify(STR::NBLW.val(self.hmac_key.len() as u32 % 4));
            // Start the digest calculation
            regs.str.modify(STR::DCAL::SET);
        } else {
            // We need to process more
            regs.imr.modify(IMR::DINIE::SET);
        }
    }

    /// Flush the leftover.
    ///
    /// Write the leftover bytes to the FIFO and set the corresponding masking.
    fn flush_leftover(&self) {
        let regs = self.regs;
        regs.str
            .modify(STR::NBLW.val(((4 - self.leftover.bytes_left()) * 8) as u32));
        regs.din.set(self.leftover.to_le());
    }

    pub(crate) fn add_data(
        &self,
        data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)> {
        // If there is a state set in the peripheral, return BUSY
        if self.state.get().is_some() {
            return Err((ErrorCode::BUSY, data));
        }
        // If neither mode nor data width is set, return error
        if self.mode.get().is_none() || self.data_width.get().is_none() {
            return Err((ErrorCode::INVAL, data));
        }
        if data.len() == 0 {
            return Err((ErrorCode::SIZE, data));
        }

        // If either data length is larger than the available size of the FIFO buffer
        // or HMAC key is not loaded and the number of bytes left to load is larger as well,
        // set an interrupt.
        if data.len() > self.regs.sr.read(SR::NBWE) as usize
            || (self.hmac_key.left_to_load() > self.regs.sr.read(SR::NBWE) as usize
                && !self.hmac_key.is_loaded())
        {
            self.regs.imr.modify(IMR::DINIE::SET);
        }

        self.data.set(Some(SubSliceMutImmut::Immutable(data)));

        // If we have key and it is not loaded yet, do load it now
        if self.hmac_key.is_stored() && self.hmac_key.left_to_load() > 0 {
            self.state.set(Some(State::HmacInit));
            self.load_key();
        } else {
            self.state.set(Some(State::Add));

            // If DMA is available, then do use it.
            if let Some(dma) = self.dma.get() {
                match self.start_dma_transfer(dma) {
                    Ok(()) => return Ok(()),
                    Err(()) => {
                        if let Some(SubSliceMutImmut::Immutable(data)) = self.data.take() {
                            return Err((ErrorCode::FAIL, data));
                        }
                    }
                }
            } else {
                // Otherwise, act as usual
                if !self.data_progress() {
                    self.deferred_call.set();
                }
            }
        }

        Ok(())
    }

    pub(crate) fn add_mut_data(
        &self,
        data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        // If there is a state set in the peripheral, return BUSY
        if self.state.get().is_some() {
            return Err((ErrorCode::BUSY, data));
        }
        // If neither mode nor data width is set, return error
        if self.mode.get().is_none() || self.data_width.get().is_none() {
            return Err((ErrorCode::INVAL, data));
        }
        if data.len() == 0 {
            return Err((ErrorCode::SIZE, data));
        }

        // If either data length is larger than the available size of the FIFO buffer
        // or HMAC key is not loaded and the number of bytes left to load is larger as well,
        // set an interrupt.
        if data.len() > self.regs.sr.read(SR::NBWE) as usize
            || (self.hmac_key.left_to_load() > self.regs.sr.read(SR::NBWE) as usize
                && !self.hmac_key.is_loaded())
        {
            self.regs.imr.modify(IMR::DINIE::SET);
        }

        self.data.set(Some(SubSliceMutImmut::Mutable(data)));

        // If we have key and it is not loaded yet, do load it now
        if self.hmac_key.is_stored() && self.hmac_key.left_to_load() > 0 {
            self.state.set(Some(State::HmacInit));
            self.load_key();
        } else {
            self.state.set(Some(State::Add));

            // If DMA is available, then do use it.
            if let Some(dma) = self.dma.get() {
                match self.start_dma_transfer(dma) {
                    Ok(()) => return Ok(()),
                    Err(()) => {
                        if let Some(SubSliceMutImmut::Mutable(data)) = self.data.take() {
                            return Err((ErrorCode::FAIL, data));
                        }
                    }
                }
            } else {
                // Otherwise, act as usual
                if !self.data_progress() {
                    self.deferred_call.set();
                }
            }
        }

        Ok(())
    }

    /// Starts the final digest computation.
    ///
    /// Resembles the `run()` contract defined in the Digest HIL for hashing.
    pub(crate) fn run(
        &self,
        digest: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.state.get().is_some() {
            return Err((ErrorCode::BUSY, digest));
        }
        // No computations without the mode set
        if self.mode.get().is_none() {
            return Err((ErrorCode::INVAL, digest));
        }
        let regs = self.regs;
        // Set the padding
        if !self.leftover.is_empty() {
            if regs.sr.read(SR::BUSY) == 0 {
                // Peripheral is free -> flush the leftover bytes
                self.flush_leftover();
            } else {
                // Otherwise, it is busy, no leftover bytes can be written
                // wait for an interrupt when FIFO is empty
                regs.imr.modify(IMR::DINIE::SET);
                self.state.set(Some(State::PreRun));
                self.digest.set(digest);
                return Ok(());
            }
        } else {
            // No padding
            regs.str.modify(STR::NBLW.val(0));
        }
        // Enable interrupts
        if self.hmac_key.is_stored() {
            // Reset the index to prepare for the outer keyh loading
            self.hmac_key.reset_index();
            regs.imr.modify(IMR::DINIE::SET + IMR::DCIE::SET);
        } else {
            regs.imr.modify(IMR::DCIE::SET);
        }

        // Start the final digest calculation
        regs.str.modify(STR::DCAL::SET);
        self.state.set(Some(State::Run));
        self.digest.set(digest);

        Ok(())
    }

    /// Starts the final digest computation with the set verification mode.
    ///
    /// Resembles the `verify()` contract defined in the Digest HIL for hashing.
    pub(crate) fn verify(
        &self,
        compare: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.verify.set(true);
        self.run(compare)
    }

    /// Resets digest registers and empties FIFO.
    pub(crate) fn clear_data(&self) {
        if self.state.get().is_none() {
            // No operation at the moment -> just reset the peripheral keeping the settings
            self.regs.cr.modify(CR::INIT::SET);
        } else {
            // Set the cancellation flag and wait for the interrupt
            self.cancelled.set(true);
        }
    }

    /// Set the peripheral mode to MD5.
    /// By default, the datawidth is 8 bits.
    ///
    /// See more about datawidth at STM32 RM0456 Reference manual, pages 2016-2017.
    ///
    /// Resembles the `set_mode_md5()` contract defined in the Digest HIL for hashing.
    pub(crate) fn set_mode_md5(&self) -> Result<(), ErrorCode> {
        if self.state.get().is_some() {
            return Err(kernel::ErrorCode::BUSY);
        }
        self.mode.set(Some(Mode::MD5));
        self.data_width.set(Some(DataWidth::_8bitData));
        self.hmac_key.clear();
        self.regs.cr.modify(
            CR::ALGO::MD5
                + CR::MODE::CLEAR
                + CR::MDMAT::SET
                + CR::DATATYPE::_8bitData
                + CR::INIT::SET,
        );
        self.hmac_key.clear();
        Ok(())
    }

    /// Set the peripheral mode to SHA1.
    /// By default, the datawidth is 8 bits.
    ///
    /// See more about datawidth at STM32 RM0456 Reference manual, pages 2016-2017.
    ///
    /// Resembles the `set_mode_sha1()` contract defined in the Digest HIL for hashing.
    pub(crate) fn set_mode_sha1(&self) -> Result<(), ErrorCode> {
        if self.state.get().is_some() {
            return Err(kernel::ErrorCode::BUSY);
        }
        self.mode.set(Some(Mode::SHA1));
        self.data_width.set(Some(DataWidth::_8bitData));
        self.hmac_key.clear();
        self.regs.cr.modify(
            CR::ALGO::SHA_1
                + CR::MODE::CLEAR
                + CR::MDMAT::SET
                + CR::DATATYPE::_8bitData
                + CR::INIT::SET,
        );
        Ok(())
    }

    /// Set the peripheral mode to SHA224.
    /// By default, the datawidth is 8 bits.
    ///
    /// See more about datawidth at STM32 RM0456 Reference manual, pages 2016-2017.
    ///
    /// Resembles the `set_mode_sha1()` contract defined in the Digest HIL for hashing.
    pub(crate) fn set_mode_sha224(&self) -> Result<(), ErrorCode> {
        if self.state.get().is_some() {
            return Err(kernel::ErrorCode::BUSY);
        }
        self.mode.set(Some(Mode::SHA2_224));
        self.data_width.set(Some(DataWidth::_8bitData));
        self.hmac_key.clear();
        self.regs.cr.modify(
            CR::ALGO::SHA2_224
                + CR::MODE::CLEAR
                + CR::MDMAT::SET
                + CR::DATATYPE::_8bitData
                + CR::INIT::SET,
        );
        Ok(())
    }

    /// Set the peripheral mode to SHA256.
    /// By default, the datawidth is 8 bits.
    ///
    /// See more about datawidth at STM32 RM0456 Reference manual, pages 2016-2017.
    ///
    /// Resembles the `set_mode_sha256()` contract defined in the Digest HIL for hashing.
    pub(crate) fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
        if self.state.get().is_some() {
            return Err(kernel::ErrorCode::BUSY);
        }
        self.mode.set(Some(Mode::SHA2_256));
        self.data_width.set(Some(DataWidth::_8bitData));
        self.hmac_key.clear();
        self.regs.cr.modify(
            CR::ALGO::SHA2_256
                + CR::MODE::CLEAR
                + CR::MDMAT::SET
                + CR::DATATYPE::_8bitData
                + CR::INIT::SET,
        );
        Ok(())
    }

    /// Set the peripheral mode to HMAC MD5.
    /// By default, the datawidth is 8 bits.
    ///
    /// See more about datawidth at STM32 RM0456 Reference manual, pages 2016-2017.
    ///
    /// Resembles the `set_mode_hmacmd5()` contract defined in the Digest HIL for hashing.
    pub(crate) fn set_mode_hmacmd5(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if self.state.get().is_some() {
            return Err(ErrorCode::BUSY);
        }
        self.mode.set(Some(Mode::MD5));
        self.data_width.set(Some(DataWidth::_8bitData));
        self.hmac_key.set(key)?;
        self.regs
            .cr
            .modify(CR::ALGO::MD5 + CR::MDMAT::SET + CR::DATATYPE::_8bitData + CR::MODE::SET);
        if key.len() > LONG_HMAC_KEY_LEN {
            self.regs.cr.modify(CR::LKEY::SET + CR::INIT::SET);
        } else {
            self.regs.cr.modify(CR::LKEY::CLEAR + CR::INIT::SET);
        }
        Ok(())
    }

    /// Set the peripheral mode to HMAC SHA1.
    /// By default, the datawidth is 8 bits.
    ///
    /// See more about datawidth at STM32 RM0456 Reference manual, pages 2016-2017.
    ///
    /// Resembles the `set_mode_hmacsha1()` contract defined in the Digest HIL for hashing.
    pub(crate) fn set_mode_hmacsha1(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if self.state.get().is_some() {
            return Err(ErrorCode::BUSY);
        }
        self.mode.set(Some(Mode::SHA1));
        self.data_width.set(Some(DataWidth::_8bitData));
        self.hmac_key.set(key)?;
        self.regs
            .cr
            .modify(CR::ALGO::SHA_1 + CR::MDMAT::SET + CR::DATATYPE::_8bitData + CR::MODE::SET);
        if key.len() > LONG_HMAC_KEY_LEN {
            self.regs.cr.modify(CR::LKEY::SET + CR::INIT::SET);
        } else {
            self.regs.cr.modify(CR::LKEY::CLEAR + CR::INIT::SET);
        }
        Ok(())
    }

    /// Set the peripheral mode to HMAC SHA224.
    /// By default, the datawidth is 8 bits.
    ///
    /// See more about datawidth at STM32 RM0456 Reference manual, pages 2016-2017.
    ///
    /// Resembles the `set_mode_hmacsha224()` contract defined in the Digest HIL for hashing.
    pub(crate) fn set_mode_hmacsha224(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if self.state.get().is_some() {
            return Err(ErrorCode::BUSY);
        }
        self.mode.set(Some(Mode::SHA2_224));
        self.data_width.set(Some(DataWidth::_8bitData));
        self.hmac_key.set(key)?;
        self.regs
            .cr
            .modify(CR::ALGO::SHA2_224 + CR::MDMAT::SET + CR::DATATYPE::_8bitData + CR::MODE::SET);
        if key.len() > LONG_HMAC_KEY_LEN {
            self.regs.cr.modify(CR::LKEY::SET + CR::INIT::SET);
        } else {
            self.regs.cr.modify(CR::LKEY::CLEAR + CR::INIT::SET);
        }

        Ok(())
    }

    /// Set the peripheral mode to HMAC SHA256.
    /// By default, the datawidth is 8 bits.
    ///
    /// See more about datawidth at STM32 RM0456 Reference manual, pages 2016-2017.
    ///
    /// Resembles the `set_mode_hmacsha256()` contract defined in the Digest HIL for hashing.
    pub(crate) fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        if self.state.get().is_some() {
            return Err(ErrorCode::BUSY);
        }
        let regs = self.regs;
        self.mode.set(Some(Mode::SHA2_256));
        self.data_width.set(Some(DataWidth::_8bitData));
        self.hmac_key.set(key)?;
        regs.cr
            .modify(CR::ALGO::SHA2_256 + CR::MDMAT::SET + CR::DATATYPE::_8bitData + CR::MODE::SET);
        if key.len() > LONG_HMAC_KEY_LEN {
            regs.cr.modify(CR::LKEY::SET + CR::INIT::SET);
        } else {
            regs.cr.modify(CR::LKEY::CLEAR + CR::INIT::SET);
        }

        Ok(())
    }
}

impl<'a> Hash<'a> {
    /// Set an MD5 adapter for the core unit. Without setting it no client will get any callback.
    ///
    /// Returns an error if the core unit already owns an adapter.
    pub fn set_md5_adapter(&self, adapter: &'a Md5Adapter<'a>) -> Result<(), ErrorCode> {
        if self.adapter.is_none() {
            self.adapter.set(HashAdapter::Md5(adapter));
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    /// Set a SHA1 adapter for the core unit. Without setting it no client will get any callback.
    ///
    /// Returns an error if the core unit already owns an adapter.
    pub fn set_sha1_adapter(&self, adapter: &'a Sha1Adapter<'a>) -> Result<(), ErrorCode> {
        if self.adapter.is_none() {
            self.adapter.set(HashAdapter::Sha1(adapter));
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    /// Set a SHA224 adapter for the core unit. Without setting it no client will get any callback.
    ///
    /// Returns an error if the core unit already owns an adapter.
    pub fn set_sha224_adapter(&self, adapter: &'a Sha224Adapter<'a>) -> Result<(), ErrorCode> {
        if self.adapter.is_none() {
            self.adapter.set(HashAdapter::Sha224(adapter));
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    /// Set a SHA256 adapter for the core unit. Without setting it no client will get any callback.
    ///
    /// Returns an error if the core unit already owns an adapter.
    pub fn set_sha256_adapter(&self, adapter: &'a Sha256Adapter<'a>) -> Result<(), ErrorCode> {
        if self.adapter.is_none() {
            self.adapter.set(HashAdapter::Sha256(adapter));
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
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

impl digest::Bit32Data for Hash<'_> {
    fn set_data_type_32_bit(&self) -> Result<(), ErrorCode> {
        self.data_width.set(Some(DataWidth::_32bitData));
        self.regs.cr.modify(CR::DATATYPE::_32bitData);
        Ok(())
    }
}

impl digest::Bit16Data for Hash<'_> {
    fn set_data_type_16_bit(&self) -> Result<(), ErrorCode> {
        self.data_width.set(Some(DataWidth::_16bitData));
        self.regs.cr.modify(CR::DATATYPE::_16bitData);
        Ok(())
    }
}

impl digest::Bit8Data for Hash<'_> {
    fn set_data_type_8_bit(&self) -> Result<(), ErrorCode> {
        self.data_width.set(Some(DataWidth::_8bitData));
        self.regs.cr.modify(CR::DATATYPE::_8bitData);
        Ok(())
    }
}

impl digest::Bit1Data for Hash<'_> {
    fn set_data_type_1_bit(&self) -> Result<(), ErrorCode> {
        self.data_width.set(Some(DataWidth::_1bitData));
        self.regs.cr.modify(CR::DATATYPE::_1bitData);
        Ok(())
    }
}

impl DeferredCallClient for Hash<'_> {
    fn handle_deferred_call(&self) {
        // A deferred call is handled if the data is loaded by software.
        // It is called once the software has finished loading.
        self.state.take();
        self.adapter.map(|adapter| {
            self.data.take().map(|buf| match buf {
                SubSliceMutImmut::Immutable(mut b) => {
                    if self.cancelled.get() {
                        self.cancelled.set(false);
                        self.leftover.empty();
                        // Reset the hash core
                        self.regs.cr.modify(CR::INIT::SET);
                        b.reset();
                        match adapter {
                            HashAdapter::Md5(md5) => {
                                md5.add_data_done(Err(ErrorCode::CANCEL), b);
                            }
                            HashAdapter::Sha1(sha1) => {
                                sha1.add_data_done(Err(ErrorCode::CANCEL), b);
                            }
                            HashAdapter::Sha224(sha224) => {
                                sha224.add_data_done(Err(ErrorCode::CANCEL), b);
                            }
                            HashAdapter::Sha256(sha256) => {
                                sha256.add_data_done(Err(ErrorCode::CANCEL), b);
                            }
                        }
                    } else {
                        match adapter {
                            HashAdapter::Md5(md5) => {
                                md5.add_data_done(Ok(()), b);
                            }
                            HashAdapter::Sha1(sha1) => {
                                sha1.add_data_done(Ok(()), b);
                            }
                            HashAdapter::Sha224(sha224) => {
                                sha224.add_data_done(Ok(()), b);
                            }
                            HashAdapter::Sha256(sha256) => {
                                sha256.add_data_done(Ok(()), b);
                            }
                        }
                    }
                }
                SubSliceMutImmut::Mutable(mut b) => {
                    if self.cancelled.get() {
                        self.cancelled.set(false);
                        self.leftover.empty();
                        // Reset the hash core
                        self.regs.cr.modify(CR::INIT::SET);
                        b.reset();
                        match adapter {
                            HashAdapter::Md5(md5) => {
                                md5.add_mut_data_done(Err(ErrorCode::CANCEL), b);
                            }
                            HashAdapter::Sha1(sha1) => {
                                sha1.add_mut_data_done(Err(ErrorCode::CANCEL), b);
                            }
                            HashAdapter::Sha224(sha224) => {
                                sha224.add_mut_data_done(Err(ErrorCode::CANCEL), b);
                            }
                            HashAdapter::Sha256(sha256) => {
                                sha256.add_mut_data_done(Err(ErrorCode::CANCEL), b);
                            }
                        }
                    } else {
                        match adapter {
                            HashAdapter::Md5(md5) => {
                                md5.add_mut_data_done(Ok(()), b);
                            }
                            HashAdapter::Sha1(sha1) => {
                                sha1.add_mut_data_done(Ok(()), b);
                            }
                            HashAdapter::Sha224(sha224) => {
                                sha224.add_mut_data_done(Ok(()), b);
                            }
                            HashAdapter::Sha256(sha256) => {
                                sha256.add_mut_data_done(Ok(()), b);
                            }
                        }
                    }
                }
            })
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
