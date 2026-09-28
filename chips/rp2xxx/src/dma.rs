// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! What both chips' DMA blocks look like from above.
//!
//! The blocks themselves are not shared. `CTRL_TRIG` moves five of the fields
//! this driver family uses, `TRANS_COUNT` gives four bits to a `MODE` field on
//! the RP2350, and the interrupt block is twice the size there, so each chip
//! declares its own registers in its own `dma` module.
//!
//! None of that reaches a driver sitting on top of DMA, which only wants a
//! source, a destination, a length and a peripheral to pace against. That much
//! is identical, so it lives here and `pio_gspi` is written against it.

use crate::pio::{PioBlock, SMNumber};

/// Notified when a channel finishes a transfer.
pub trait DmaChannelClient {
    fn transfer_done(&self);
}

/// One DMA channel, as a driver above it needs to see one.
///
/// The two transfer methods name a PIO FIFO rather than taking a peripheral
/// selector, because that selector is a chip specific enum whose encoding
/// moved between the chips. Both are word sized, which is all the gSPI
/// transport asks for.
pub trait DmaChannel {
    /// The PIO block type of the chip this channel belongs to.
    type Block: PioBlock;

    /// Where the channel reads from.
    fn set_read_addr(&self, addr: u32);

    /// Where the channel writes to.
    fn set_write_addr(&self, addr: u32);

    /// How many transfers the next sequence runs.
    fn set_len(&self, len: u32);

    /// Run a word sized transfer paced by a state machine's RX FIFO, reading
    /// from the FIFO into memory.
    fn read_word_from_pio(&self, block: Self::Block, sm: SMNumber);

    /// Run a word sized transfer paced by a state machine's TX FIFO, writing
    /// from memory into the FIFO.
    fn write_word_to_pio(&self, block: Self::Block, sm: SMNumber);
}
