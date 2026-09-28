// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Direct Memory Access (DMA) hardware.
//!
//! This chip's DMA block is close enough to the RP2040's to read like it and
//! different enough that it is not the same driver. `CTRL_TRIG` gains
//! `INCR_READ_REV` at bit 5 and `INCR_WRITE_REV` at bit 7, moving most fields
//! above bit 4 up by two and `INCR_WRITE` up by one. `TRANS_COUNT` gives up
//! its top four bits to a `MODE` field. The interrupt block carries four
//! sets of enable, force and status registers where the RP2040 has two, and
//! the two extra sets fill 0x420 to 0x43c, which the RP2040 gives to its
//! timers and sniff registers. Those move to 0x440 and beyond here.
//!
//! This chip also gives the DMA per channel and per interrupt security
//! configuration at 0x480 and 0x4c0, and an MPU of its own at 0x500, none of
//! which the RP2040 has. Nothing here declares or writes them. Their reset
//! state suits a kernel running Secure, which is where the RP2350 starts and
//! where Tock stays; a port that moved to Non-secure would have to revisit
//! them.
//!
//! Refer to the RP2350 Datasheet, Section 12.
//! RP2350 Datasheet [1].
//!
//! [1]: https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf

use crate::pio;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{FieldValue, ReadWrite, register_bitfields, register_structs};

/// The RP2350 has 16 DMA channels, four more than the RP2040.
pub const NUM_CHANNELS: usize = 16;

register_structs! {
    pub ChannelRegisters {
        (0x000 => read_addr: ReadWrite<u32>),
        (0x004 => write_addr: ReadWrite<u32>),
        (0x008 => trans_count: ReadWrite<u32, TRANS_COUNT::Register>),
        (0x00C => ctrl_trig: ReadWrite<u32, CTRL_TRIG::Register>),
        (0x010 => _reserved0),
        (0x040 => @END),
    },

    /// Declared as far as the interrupt block and no further. The timers,
    /// sniff, abort and per channel debug registers that follow are ones the
    /// RP2040's driver declares and never reads, so they are left out rather
    /// than transcribed. The sixteen channels fill 0x000 to 0x3ff exactly,
    /// taking the space the RP2040 leaves reserved after its twelve.
    pub DmaRegisters {
        (0x000 => channels: [ChannelRegisters; NUM_CHANNELS]),

        // Interrupt enable, force and status, one set per IRQ. The RP2040
        // has two of these and puts its timers at 0x420; this chip has four
        // and its timers move out to 0x440.
        (0x400 => intr: ReadWrite<u32>),
        (0x404 => inte0: ReadWrite<u32>),
        (0x408 => intf0: ReadWrite<u32>),
        (0x40C => ints0: ReadWrite<u32>),
        (0x410 => _reserved0),
        (0x414 => inte1: ReadWrite<u32>),
        (0x418 => intf1: ReadWrite<u32>),
        (0x41C => ints1: ReadWrite<u32>),
        (0x420 => _reserved1),
        (0x424 => inte2: ReadWrite<u32>),
        (0x428 => intf2: ReadWrite<u32>),
        (0x42C => ints2: ReadWrite<u32>),
        (0x430 => _reserved2),
        (0x434 => inte3: ReadWrite<u32>),
        (0x438 => intf3: ReadWrite<u32>),
        (0x43C => ints3: ReadWrite<u32>),
        (0x440 => @END),
    }
}

register_bitfields![u32,
    /// Transfer count. Unlike the RP2040's, this is not a plain 32 bit
    /// count: the top four bits select how the channel behaves when the
    /// count reaches zero.
    TRANS_COUNT [
        MODE OFFSET(28) NUMBITS(4) [
            /// Count down, then trigger CHAIN_TO.
            Normal = 0x0,
            /// Count down, then re-trigger this channel as well as CHAIN_TO.
            TriggerSelf = 0x1,
            /// Never decrement. Transfer until aborted.
            Endless = 0xf
        ],
        COUNT OFFSET(0) NUMBITS(28) []
    ],

    /// Channel control and status.
    ///
    /// Bit 4 and below sit where the RP2040 puts them. `INCR_READ_REV` at 5
    /// and `INCR_WRITE_REV` at 7 are new, and because they are not adjacent
    /// `INCR_WRITE` moves up by one where everything from `RING_SIZE` up
    /// moves by two. Datasheet Table 1151.
    CTRL_TRIG [
        AHB_ERROR OFFSET(31) NUMBITS(1) [],
        READ_ERROR OFFSET(30) NUMBITS(1) [],
        WRITE_ERROR OFFSET(29) NUMBITS(1) [],
        BUSY OFFSET(26) NUMBITS(1) [],
        SNIFF_EN OFFSET(25) NUMBITS(1) [],
        BSWAP OFFSET(24) NUMBITS(1) [],
        IRQ_QUIET OFFSET(23) NUMBITS(1) [],
        TREQ_SEL OFFSET(17) NUMBITS(6) [
            /// Select PIO0's TX FIFO 0 as TREQ
            SelectPIO0STXFIFO0AsTREQ = 0,
            /// Select PIO0's TX FIFO 1 as TREQ
            SelectPIO0STXFIFO1AsTREQ = 1,
            /// Select PIO0's TX FIFO 2 as TREQ
            SelectPIO0STXFIFO2AsTREQ = 2,
            /// Select PIO0's TX FIFO 3 as TREQ
            SelectPIO0STXFIFO3AsTREQ = 3,
            /// Select PIO0's RX FIFO 0 as TREQ
            SelectPIO0SRXFIFO0AsTREQ = 4,
            /// Select PIO0's RX FIFO 1 as TREQ
            SelectPIO0SRXFIFO1AsTREQ = 5,
            /// Select PIO0's RX FIFO 2 as TREQ
            SelectPIO0SRXFIFO2AsTREQ = 6,
            /// Select PIO0's RX FIFO 3 as TREQ
            SelectPIO0SRXFIFO3AsTREQ = 7,
            /// Select PIO1's TX FIFO 0 as TREQ
            SelectPIO1STXFIFO0AsTREQ = 8,
            /// Select PIO1's TX FIFO 1 as TREQ
            SelectPIO1STXFIFO1AsTREQ = 9,
            /// Select PIO1's TX FIFO 2 as TREQ
            SelectPIO1STXFIFO2AsTREQ = 10,
            /// Select PIO1's TX FIFO 3 as TREQ
            SelectPIO1STXFIFO3AsTREQ = 11,
            /// Select PIO1's RX FIFO 0 as TREQ
            SelectPIO1SRXFIFO0AsTREQ = 12,
            /// Select PIO1's RX FIFO 1 as TREQ
            SelectPIO1SRXFIFO1AsTREQ = 13,
            /// Select PIO1's RX FIFO 2 as TREQ
            SelectPIO1SRXFIFO2AsTREQ = 14,
            /// Select PIO1's RX FIFO 3 as TREQ
            SelectPIO1SRXFIFO3AsTREQ = 15,
            /// Select PIO2's TX FIFO 0 as TREQ
            SelectPIO2STXFIFO0AsTREQ = 16,
            /// Select PIO2's TX FIFO 1 as TREQ
            SelectPIO2STXFIFO1AsTREQ = 17,
            /// Select PIO2's TX FIFO 2 as TREQ
            SelectPIO2STXFIFO2AsTREQ = 18,
            /// Select PIO2's TX FIFO 3 as TREQ
            SelectPIO2STXFIFO3AsTREQ = 19,
            /// Select PIO2's RX FIFO 0 as TREQ
            SelectPIO2SRXFIFO0AsTREQ = 20,
            /// Select PIO2's RX FIFO 1 as TREQ
            SelectPIO2SRXFIFO1AsTREQ = 21,
            /// Select PIO2's RX FIFO 2 as TREQ
            SelectPIO2SRXFIFO2AsTREQ = 22,
            /// Select PIO2's RX FIFO 3 as TREQ
            SelectPIO2SRXFIFO3AsTREQ = 23
        ],
        CHAIN_TO OFFSET(13) NUMBITS(4) [],
        RING_SEL OFFSET(12) NUMBITS(1) [],
        RING_SIZE OFFSET(8) NUMBITS(4) [],
        INCR_WRITE_REV OFFSET(7) NUMBITS(1) [],
        INCR_WRITE OFFSET(6) NUMBITS(1) [],
        INCR_READ_REV OFFSET(5) NUMBITS(1) [],
        INCR_READ OFFSET(4) NUMBITS(1) [],
        DATA_SIZE OFFSET(2) NUMBITS(2) [
            SIZE_BYTE = 0,
            SIZE_HALFWORD = 1,
            SIZE_WORD = 2
        ],
        HIGH_PRIORITY OFFSET(1) NUMBITS(1) [],
        EN OFFSET(0) NUMBITS(1) []
    ]
];

const DMA_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x50000000 as *const DmaRegisters) };

/// Which DMA channel. The RP2350 has sixteen.
#[derive(Clone, Copy)]
pub enum Channel {
    Ch0 = 0,
    Ch1 = 1,
    Ch2 = 2,
    Ch3 = 3,
    Ch4 = 4,
    Ch5 = 5,
    Ch6 = 6,
    Ch7 = 7,
    Ch8 = 8,
    Ch9 = 9,
    Ch10 = 10,
    Ch11 = 11,
    Ch12 = 12,
    Ch13 = 13,
    Ch14 = 14,
    Ch15 = 15,
}

/// Which way a transfer moves data.
pub enum Transfer {
    MemoryToPeripheral,
    PeripheralToMemory,
}

/// Size of each bus access a transfer makes.
pub enum DataSize {
    Byte,
    HalfWord,
    Word,
}

impl From<DataSize> for FieldValue<u32, CTRL_TRIG::Register> {
    fn from(value: DataSize) -> Self {
        match value {
            DataSize::Byte => CTRL_TRIG::DATA_SIZE::SIZE_BYTE,
            DataSize::HalfWord => CTRL_TRIG::DATA_SIZE::SIZE_HALFWORD,
            DataSize::Word => CTRL_TRIG::DATA_SIZE::SIZE_WORD,
        }
    }
}

/// Which peripheral FIFO paces a transfer.
///
/// The DREQ numbers for PIO0 and PIO1 are the same as the RP2040's; PIO2 is
/// appended at 16 to 23. Every non-PIO DREQ moves up by eight on this chip,
/// which does not reach here because only PIO FIFOs are named.
pub enum DmaPeripheral {
    /// The RX FIFO of one state machine.
    PioRxFifo(pio::PIONumber, pio::SMNumber),
    /// The TX FIFO of one state machine.
    PioTxFifo(pio::PIONumber, pio::SMNumber),
}

impl From<DmaPeripheral> for FieldValue<u32, CTRL_TRIG::Register> {
    fn from(value: DmaPeripheral) -> Self {
        use pio::PIONumber::{PIO0, PIO1, PIO2};
        use pio::SMNumber::{SM0, SM1, SM2, SM3};
        match value {
            DmaPeripheral::PioTxFifo(PIO0, SM0) => CTRL_TRIG::TREQ_SEL::SelectPIO0STXFIFO0AsTREQ,
            DmaPeripheral::PioTxFifo(PIO0, SM1) => CTRL_TRIG::TREQ_SEL::SelectPIO0STXFIFO1AsTREQ,
            DmaPeripheral::PioTxFifo(PIO0, SM2) => CTRL_TRIG::TREQ_SEL::SelectPIO0STXFIFO2AsTREQ,
            DmaPeripheral::PioTxFifo(PIO0, SM3) => CTRL_TRIG::TREQ_SEL::SelectPIO0STXFIFO3AsTREQ,
            DmaPeripheral::PioRxFifo(PIO0, SM0) => CTRL_TRIG::TREQ_SEL::SelectPIO0SRXFIFO0AsTREQ,
            DmaPeripheral::PioRxFifo(PIO0, SM1) => CTRL_TRIG::TREQ_SEL::SelectPIO0SRXFIFO1AsTREQ,
            DmaPeripheral::PioRxFifo(PIO0, SM2) => CTRL_TRIG::TREQ_SEL::SelectPIO0SRXFIFO2AsTREQ,
            DmaPeripheral::PioRxFifo(PIO0, SM3) => CTRL_TRIG::TREQ_SEL::SelectPIO0SRXFIFO3AsTREQ,
            DmaPeripheral::PioTxFifo(PIO1, SM0) => CTRL_TRIG::TREQ_SEL::SelectPIO1STXFIFO0AsTREQ,
            DmaPeripheral::PioTxFifo(PIO1, SM1) => CTRL_TRIG::TREQ_SEL::SelectPIO1STXFIFO1AsTREQ,
            DmaPeripheral::PioTxFifo(PIO1, SM2) => CTRL_TRIG::TREQ_SEL::SelectPIO1STXFIFO2AsTREQ,
            DmaPeripheral::PioTxFifo(PIO1, SM3) => CTRL_TRIG::TREQ_SEL::SelectPIO1STXFIFO3AsTREQ,
            DmaPeripheral::PioRxFifo(PIO1, SM0) => CTRL_TRIG::TREQ_SEL::SelectPIO1SRXFIFO0AsTREQ,
            DmaPeripheral::PioRxFifo(PIO1, SM1) => CTRL_TRIG::TREQ_SEL::SelectPIO1SRXFIFO1AsTREQ,
            DmaPeripheral::PioRxFifo(PIO1, SM2) => CTRL_TRIG::TREQ_SEL::SelectPIO1SRXFIFO2AsTREQ,
            DmaPeripheral::PioRxFifo(PIO1, SM3) => CTRL_TRIG::TREQ_SEL::SelectPIO1SRXFIFO3AsTREQ,
            DmaPeripheral::PioTxFifo(PIO2, SM0) => CTRL_TRIG::TREQ_SEL::SelectPIO2STXFIFO0AsTREQ,
            DmaPeripheral::PioTxFifo(PIO2, SM1) => CTRL_TRIG::TREQ_SEL::SelectPIO2STXFIFO1AsTREQ,
            DmaPeripheral::PioTxFifo(PIO2, SM2) => CTRL_TRIG::TREQ_SEL::SelectPIO2STXFIFO2AsTREQ,
            DmaPeripheral::PioTxFifo(PIO2, SM3) => CTRL_TRIG::TREQ_SEL::SelectPIO2STXFIFO3AsTREQ,
            DmaPeripheral::PioRxFifo(PIO2, SM0) => CTRL_TRIG::TREQ_SEL::SelectPIO2SRXFIFO0AsTREQ,
            DmaPeripheral::PioRxFifo(PIO2, SM1) => CTRL_TRIG::TREQ_SEL::SelectPIO2SRXFIFO1AsTREQ,
            DmaPeripheral::PioRxFifo(PIO2, SM2) => CTRL_TRIG::TREQ_SEL::SelectPIO2SRXFIFO2AsTREQ,
            DmaPeripheral::PioRxFifo(PIO2, SM3) => CTRL_TRIG::TREQ_SEL::SelectPIO2SRXFIFO3AsTREQ,
        }
    }
}

/// Which of the four DMA interrupts a channel is routed to.
pub enum Irq {
    Irq0,
    Irq1,
    Irq2,
    Irq3,
}

/// Notified when a channel finishes a transfer.
///
/// Defined in `rp2xxx` and re-exported here: nothing about it differs between
/// the chips, and a driver above DMA has to be able to name one trait.
pub use rp2xxx::dma::DmaChannelClient;

#[derive(Clone, Copy)]
pub struct DmaChannel<'a> {
    dma: &'a Dma<'a>,
    ch: Channel,
}

impl<'a> DmaChannel<'a> {
    pub const fn new(dma: &'a Dma<'a>, ch: Channel) -> Self {
        Self { dma, ch }
    }

    pub fn set_client(&self, client: &'a dyn DmaChannelClient) {
        self.dma.set_channel_client(self.ch, client);
    }
}

pub struct Dma<'a> {
    registers: StaticRef<DmaRegisters>,
    clients: [OptionalCell<&'a dyn DmaChannelClient>; NUM_CHANNELS],
}

impl<'a> Dma<'a> {
    pub const fn new() -> Self {
        Self {
            registers: DMA_BASE,
            clients: [const { OptionalCell::empty() }; NUM_CHANNELS],
        }
    }

    pub fn channel(&'a self, ch: Channel) -> DmaChannel<'a> {
        DmaChannel::new(self, ch)
    }

    pub fn handle_interrupt(&self, irq: Irq) {
        let ints = self.status_register(irq);
        let value = ints.get();
        ints.set(value);
        self.handle_channels(value);
    }

    #[inline]
    fn handle_channels(&self, mut ints: u32) {
        // One bit per channel, so anything above NUM_CHANNELS is not ours.
        ints &= (1 << NUM_CHANNELS) - 1;
        while ints != 0 {
            let channel = ints.trailing_zeros();
            self.clients[channel as usize].map(|client| client.transfer_done());
            ints ^= 1 << channel;
        }
    }

    fn enable_register(&self, irq: Irq) -> &ReadWrite<u32> {
        match irq {
            Irq::Irq0 => &self.registers.inte0,
            Irq::Irq1 => &self.registers.inte1,
            Irq::Irq2 => &self.registers.inte2,
            Irq::Irq3 => &self.registers.inte3,
        }
    }

    fn status_register(&self, irq: Irq) -> &ReadWrite<u32> {
        match irq {
            Irq::Irq0 => &self.registers.ints0,
            Irq::Irq1 => &self.registers.ints1,
            Irq::Irq2 => &self.registers.ints2,
            Irq::Irq3 => &self.registers.ints3,
        }
    }

    fn enable_interrupt(&self, channel: Channel, irq: Irq) {
        let reg = self.enable_register(irq);
        reg.set(reg.get() | 1 << (channel as usize));
    }

    fn disable_interrupt(&self, channel: Channel, irq: Irq) {
        let reg = self.enable_register(irq);
        reg.set(reg.get() & !(1 << (channel as usize)));
    }

    fn channel_registers(&self, channel: Channel) -> &ChannelRegisters {
        &self.registers.channels[channel as usize]
    }

    fn set_channel_client(&self, channel: Channel, client: &'a dyn DmaChannelClient) {
        self.clients[channel as usize].set(client)
    }
}

impl DmaChannel<'_> {
    /// Transfers still to run in the current sequence.
    ///
    /// Only the low 28 bits are the count on this chip; the top four are
    /// `MODE` and are masked off here.
    pub fn trans_count(&self) -> u32 {
        self.dma
            .channel_registers(self.ch)
            .trans_count
            .read(TRANS_COUNT::COUNT)
    }

    pub fn busy(&self) -> bool {
        self.dma
            .channel_registers(self.ch)
            .ctrl_trig
            .is_set(CTRL_TRIG::BUSY)
    }

    pub fn set_read_addr(&self, addr: u32) {
        self.dma.channel_registers(self.ch).read_addr.set(addr);
    }

    pub fn set_write_addr(&self, addr: u32) {
        self.dma.channel_registers(self.ch).write_addr.set(addr);
    }

    /// Set how many transfers the next sequence runs.
    ///
    /// `MODE` is written alongside the count rather than left alone: it
    /// shares this register on the RP2350, and `Normal` is the behaviour the
    /// RP2040's plain 32 bit count always had.
    pub fn set_len(&self, len: u32) {
        self.dma
            .channel_registers(self.ch)
            .trans_count
            .write(TRANS_COUNT::COUNT.val(len) + TRANS_COUNT::MODE::Normal);
    }

    pub fn enable_interrupt(&self, irq: Irq) {
        self.dma.enable_interrupt(self.ch, irq);
    }

    pub fn disable_interrupt(&self, irq: Irq) {
        self.dma.disable_interrupt(self.ch, irq);
    }

    pub fn enable(
        &self,
        treq: DmaPeripheral,
        data_size: DataSize,
        transfer: Transfer,
        bswap: bool,
    ) {
        self.dma
            .channel_registers(self.ch)
            .ctrl_trig
            .write(ctrl_word(treq, data_size, transfer, bswap, self.ch));
    }
}

/// The `CTRL_TRIG` word `enable` writes.
///
/// Split out so a host test can compose what the driver composes rather than
/// rebuilding it alongside. A test that rebuilds the word passes whatever the
/// driver does with it.
fn ctrl_word(
    treq: DmaPeripheral,
    data_size: DataSize,
    transfer: Transfer,
    bswap: bool,
    chain_to: Channel,
) -> FieldValue<u32, CTRL_TRIG::Register> {
    let bswap = match bswap {
        true => CTRL_TRIG::BSWAP::SET,
        false => CTRL_TRIG::BSWAP::CLEAR,
    };
    let (incr_rd, incr_wr) = match transfer {
        Transfer::MemoryToPeripheral => (CTRL_TRIG::INCR_READ::SET, CTRL_TRIG::INCR_WRITE::CLEAR),
        Transfer::PeripheralToMemory => (CTRL_TRIG::INCR_READ::CLEAR, CTRL_TRIG::INCR_WRITE::SET),
    };
    // A channel chaining to itself is how the datasheet says "chain to
    // nothing".
    FieldValue::from(treq)
        + FieldValue::from(data_size)
        + bswap
        + incr_rd
        + incr_wr
        + CTRL_TRIG::CHAIN_TO.val(chain_to as u32)
        + CTRL_TRIG::EN::SET
}

impl rp2xxx::dma::DmaChannel for DmaChannel<'_> {
    type Block = crate::pio::PIONumber;

    fn set_read_addr(&self, addr: u32) {
        DmaChannel::set_read_addr(self, addr)
    }

    fn set_write_addr(&self, addr: u32) {
        DmaChannel::set_write_addr(self, addr)
    }

    fn set_len(&self, len: u32) {
        DmaChannel::set_len(self, len)
    }

    fn read_word_from_pio(&self, block: Self::Block, sm: pio::SMNumber) {
        self.enable(
            DmaPeripheral::PioRxFifo(block, sm),
            DataSize::Word,
            Transfer::PeripheralToMemory,
            false,
        )
    }

    fn write_word_to_pio(&self, block: Self::Block, sm: pio::SMNumber) {
        self.enable(
            DmaPeripheral::PioTxFifo(block, sm),
            DataSize::Word,
            Transfer::MemoryToPeripheral,
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pio::PIONumber::{PIO0, PIO1, PIO2};
    use crate::pio::SMNumber::{SM0, SM1, SM2, SM3};

    fn treq_of(p: DmaPeripheral) -> u32 {
        FieldValue::<u32, CTRL_TRIG::Register>::from(p).value
    }

    /// Every `CTRL_TRIG` field, at the bit the RP2350 datasheet gives it.
    ///
    /// `register_bitfields!` does not check an offset against anything, so a
    /// field copied from the wrong chip still builds and still passes every
    /// test that does not look at the encoded word. Every offset in this file
    /// was transcribed by hand from Table 1151, so every one is pinned here,
    /// including the ten the driver does not yet read. Those cost nothing
    /// today and would be a trap for whoever reads them first.
    #[test]
    fn ctrl_trig_fields_sit_where_the_rp2350_puts_them() {
        // Written or read by the driver.
        assert_eq!(CTRL_TRIG::TREQ_SEL.val(0x3f).value, 0x3f << 17);
        assert_eq!(CTRL_TRIG::CHAIN_TO.val(0xf).value, 0xf << 13);
        assert_eq!(CTRL_TRIG::BSWAP::SET.value, 1 << 24);
        assert_eq!(CTRL_TRIG::BUSY::SET.value, 1 << 26);
        assert_eq!(CTRL_TRIG::INCR_WRITE::SET.value, 1 << 6);
        assert_eq!(CTRL_TRIG::INCR_READ::SET.value, 1 << 4);
        assert_eq!(CTRL_TRIG::EN::SET.value, 1);

        // Declared but not yet read. The two REV bits are why the fields
        // above them moved, so they are worth pinning even unused.
        assert_eq!(CTRL_TRIG::AHB_ERROR::SET.value, 1 << 31);
        assert_eq!(CTRL_TRIG::READ_ERROR::SET.value, 1 << 30);
        assert_eq!(CTRL_TRIG::WRITE_ERROR::SET.value, 1 << 29);
        assert_eq!(CTRL_TRIG::SNIFF_EN::SET.value, 1 << 25);
        assert_eq!(CTRL_TRIG::IRQ_QUIET::SET.value, 1 << 23);
        assert_eq!(CTRL_TRIG::RING_SEL::SET.value, 1 << 12);
        assert_eq!(CTRL_TRIG::RING_SIZE.val(0xf).value, 0xf << 8);
        assert_eq!(CTRL_TRIG::INCR_WRITE_REV::SET.value, 1 << 7);
        assert_eq!(CTRL_TRIG::INCR_READ_REV::SET.value, 1 << 5);
        assert_eq!(CTRL_TRIG::HIGH_PRIORITY::SET.value, 1 << 1);
    }

    /// Every value the driver can encode, not just the one the radio uses.
    ///
    /// `DataSize` has three arms and the gSPI path only ever asks for `Word`,
    /// so a transposed `Byte` and `HalfWord` would reach hardware unexamined.
    /// Datasheet Table 1151: 0 byte, 1 halfword, 2 word.
    #[test]
    fn every_data_size_encodes_to_its_datasheet_value() {
        let enc = |d| FieldValue::<u32, CTRL_TRIG::Register>::from(d).value;
        assert_eq!(enc(DataSize::Byte), 0 << 2);
        assert_eq!(enc(DataSize::HalfWord), 1 << 2);
        assert_eq!(enc(DataSize::Word), 2 << 2);
    }

    /// The RP2040 positions, asserted as *wrong* for this chip.
    ///
    /// This is the test that fails if someone copies the sibling driver's
    /// bitfields across. `INCR_READ` is deliberately absent: it does not
    /// move, and asserting it moved would be false.
    #[test]
    fn the_fields_that_move_really_did_move() {
        assert_ne!(CTRL_TRIG::TREQ_SEL.val(1).value, 1 << 15);
        assert_ne!(CTRL_TRIG::CHAIN_TO.val(1).value, 1 << 11);
        assert_ne!(CTRL_TRIG::BSWAP::SET.value, 1 << 22);
        assert_ne!(CTRL_TRIG::BUSY::SET.value, 1 << 24);
        assert_ne!(CTRL_TRIG::INCR_WRITE::SET.value, 1 << 5);
        // INCR_WRITE moves by one, not two, because the two inserted bits
        // are not adjacent and it sits between them.
        assert_eq!(CTRL_TRIG::INCR_WRITE::SET.value, 1 << (5 + 1));
        assert_eq!(CTRL_TRIG::BUSY::SET.value, 1 << (24 + 2));
    }

    /// Every transfer request number, against the datasheet's DREQ table.
    ///
    /// Table 1146 numbers them `PIOn TXm` at `8n + m` and `PIOn RXm` at
    /// `8n + 4 + m`, so PIO0 and PIO1 are numbered as they are on the RP2040
    /// and PIO2 is appended at 16 to 23.
    ///
    /// All twenty four are checked rather than a sample. The table they check
    /// is twenty four hand written match arms, and a transposed pair inside
    /// one block would still satisfy any smaller selection of them.
    #[test]
    fn treq_numbers_match_the_datasheet_table() {
        let pios = [(PIO0, 0), (PIO1, 1), (PIO2, 2)];
        let sms = [(SM0, 0), (SM1, 1), (SM2, 2), (SM3, 3)];
        for (pio, n) in pios {
            for (sm, m) in sms {
                let tx = 8 * n + m;
                let rx = 8 * n + 4 + m;
                assert_eq!(
                    treq_of(DmaPeripheral::PioTxFifo(pio, sm)),
                    tx << 17,
                    "PIO{n} TX{m} should be DREQ {tx}"
                );
                assert_eq!(
                    treq_of(DmaPeripheral::PioRxFifo(pio, sm)),
                    rx << 17,
                    "PIO{n} RX{m} should be DREQ {rx}"
                );
            }
        }
    }

    /// The two control words the CYW43439 gSPI transport asks for.
    ///
    /// If either is wrong the radio does not come up, so they are worth
    /// pinning as whole words rather than field by field.
    #[test]
    fn the_gspi_control_words_are_right() {
        // Composed by the driver's own ctrl_word rather than rebuilt here, so
        // a change to what enable writes fails this instead of passing it.

        // Memory to peripheral, PIO0 TX FIFO 0, channel 0 chaining to itself.
        let push = ctrl_word(
            DmaPeripheral::PioTxFifo(PIO0, SM0),
            DataSize::Word,
            Transfer::MemoryToPeripheral,
            false,
            Channel::Ch0,
        )
        .value;
        // TREQ 0 << 17 | DATA_SIZE 2 << 2 | INCR_READ 1 << 4 | EN 1
        assert_eq!(push, (2 << 2) | (1 << 4) | 1);

        // Peripheral to memory, PIO0 RX FIFO 0.
        let pull = ctrl_word(
            DmaPeripheral::PioRxFifo(PIO0, SM0),
            DataSize::Word,
            Transfer::PeripheralToMemory,
            false,
            Channel::Ch0,
        )
        .value;
        // TREQ 4 << 17 | DATA_SIZE 2 << 2 | INCR_WRITE 1 << 6 | EN 1
        assert_eq!(pull, (4 << 17) | (2 << 2) | (1 << 6) | 1);

        // A channel chains to itself, so a different channel is a different
        // word in CHAIN_TO and nothing else.
        let on_ch5 = ctrl_word(
            DmaPeripheral::PioRxFifo(PIO0, SM0),
            DataSize::Word,
            Transfer::PeripheralToMemory,
            false,
            Channel::Ch5,
        )
        .value;
        assert_eq!(on_ch5, pull | (5 << 13));
    }

    /// `TRANS_COUNT` is not a plain 32 bit count on this chip.
    ///
    /// The top four bits are `MODE`, so a length that overflowed 28 bits
    /// would set it. `0xf` is `Endless`, which never stops transferring.
    #[test]
    fn trans_count_keeps_mode_out_of_the_count() {
        assert_eq!(TRANS_COUNT::COUNT.val(0xffff_ffff).value, 0x0fff_ffff);
        assert_eq!(TRANS_COUNT::MODE::Normal.value, 0);
        assert_eq!(TRANS_COUNT::MODE::TriggerSelf.value, 1 << 28);
        assert_eq!(TRANS_COUNT::MODE::Endless.value, 0xf << 28);
        // A length that would have been a valid count on the RP2040 must not
        // reach MODE here.
        let written = (TRANS_COUNT::COUNT.val(0x1000_0001) + TRANS_COUNT::MODE::Normal).value;
        assert_eq!(written >> 28, 0);
    }
}
