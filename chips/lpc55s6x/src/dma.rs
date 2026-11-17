// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Direct Memory Access (DMA) controllers of the LPC55S6x family.
//!
//! The chip has two instances of the same controller (UM11126, Chapter 22):
//! DMA0 with 23 channels and DMA1 with 10 channels. Each channel moves up to
//! 1024 items of 8, 16 or 32 bits with a single descriptor.
//!
//! ## Channel assignment
//!
//! Peripheral request lines are hardwired to channels (UM11126, Tables 428 and
//! 429), so a peripheral transfer must use the channel listed in
//! [`DmaRequest`]. Memory-to-memory transfers do not use the request line, so
//! they can use a channel that has one; channel 1 of each controller has no
//! request connected. Channel 20 of DMA0 has no request either, but its
//! registers are listed as reserved (UM11126, Table 436), so do not use it.
//!
//! ## Usage
//!
//! The board creates a [`DmaDescriptorTable`] per controller with
//! `static_init!`, starts the controller clocks with
//! [`Clock::start_dma_clocks`](crate::clocks::Clock::start_dma_clocks), and
//! calls [`Dma::init`]. A driver then selects a channel with
//! [`Dma::channel_for`] or [`Dma::channel`], registers a [`DmaClient`], and
//! starts a transfer. The buffers are passed as [`SubSliceMut`]s, and the
//! active range of each one is the memory the controller accesses. The
//! channel holds them as [`DmaSubSliceMut`]s while the transfer runs, and
//! [`DmaClient::transfer_done`] returns them with their active ranges
//! unchanged.
//!
//! Before a Flexcomm transfer, its driver must also enable DMA on the
//! peripheral side (`FIFOCFG.DMATX` or `FIFOCFG.DMARX`).

use core::cell::Cell;
use core::ptr;

use cortexm33::dma_fence::CortexMDmaFence;
use kernel::ErrorCode;
use kernel::platform::dma_fence::DmaFence;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::dma_slice::DmaSubSliceMut;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    InMemoryRegister, ReadOnly, ReadWrite, WriteOnly, register_bitfields, register_structs,
};

/// Number of channels of the DMA0 controller.
pub const DMA0_CHANNELS: usize = 23;
/// Number of channels of the DMA1 controller.
pub const DMA1_CHANNELS: usize = 10;

/// Maximum number of items moved by one descriptor. `XFERCOUNT` is 10 bits wide
/// and encodes the count minus one.
pub const MAX_TRANSFERS: usize = 1024;

/// Lowest channel priority. `0` is the highest.
pub const LOWEST_PRIORITY: u8 = 7;

/// `XFERCOUNT` value once a channel has moved all of its items.
const XFERCOUNT_DONE: u32 = 0x3FF;

/// Offset of `FIFOWR` in a Flexcomm interface.
const FLEXCOMM_FIFOWR: u32 = 0xE20;
/// Offset of `FIFORD` in a Flexcomm interface.
const FLEXCOMM_FIFORD: u32 = 0xE30;

register_structs! {
    /// Registers of one DMA channel.
    ChannelRegisters {
        /// Channel configuration.
        (0x0 => cfg: ReadWrite<u32, CFG::Register>),
        /// Channel control and status.
        (0x4 => ctlstat: ReadOnly<u32, CTLSTAT::Register>),
        /// Channel transfer configuration.
        (0x8 => xfercfg: ReadWrite<u32, XFERCFG::Register>),
        (0xC => _reserved),
        (0x10 => @END),
    }
}

register_structs! {
    /// DMA controller registers (UM11126, Table 436).
    ///
    /// In the shared registers, bit `n` refers to channel `n`.
    DmaRegisters {
        /// DMA control.
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        /// Interrupt status.
        (0x004 => intstat: ReadOnly<u32, INTSTAT::Register>),
        /// Address of the channel descriptor table.
        (0x008 => srambase: ReadWrite<u32>),
        (0x00C => _reserved0),
        /// Channel enable read and set.
        (0x020 => enableset0: ReadWrite<u32>),
        (0x024 => _reserved1),
        /// Channel enable clear.
        (0x028 => enableclr0: WriteOnly<u32>),
        (0x02C => _reserved2),
        /// Channel active status.
        (0x030 => active0: ReadOnly<u32>),
        (0x034 => _reserved3),
        /// Channel busy status.
        (0x038 => busy0: ReadOnly<u32>),
        (0x03C => _reserved4),
        /// Error interrupt status. Write 1 to clear.
        (0x040 => errint0: ReadWrite<u32>),
        (0x044 => _reserved5),
        /// Interrupt enable read and set.
        (0x048 => intenset0: ReadWrite<u32>),
        (0x04C => _reserved6),
        /// Interrupt enable clear.
        (0x050 => intenclr0: WriteOnly<u32>),
        (0x054 => _reserved7),
        /// Interrupt A status. Write 1 to clear.
        (0x058 => inta0: ReadWrite<u32>),
        (0x05C => _reserved8),
        /// Interrupt B status. Write 1 to clear.
        (0x060 => intb0: ReadWrite<u32>),
        (0x064 => _reserved9),
        /// Set the valid pending bit of a channel.
        (0x068 => setvalid0: WriteOnly<u32>),
        (0x06C => _reserved10),
        /// Set the trigger of a channel.
        (0x070 => settrig0: WriteOnly<u32>),
        (0x074 => _reserved11),
        /// Abort a channel.
        (0x078 => abort0: WriteOnly<u32>),
        (0x07C => _reserved12),
        /// Per-channel registers.
        (0x400 => channel: [ChannelRegisters; DMA0_CHANNELS]),
        (0x570 => @END),
    }
}

register_bitfields![u32,
    CTRL [
        /// DMA controller master enable.
        ENABLE OFFSET(0) NUMBITS(1) []
    ],
    INTSTAT [
        /// At least one enabled interrupt, other than errors, is pending.
        ACTIVEINT OFFSET(1) NUMBITS(1) [],
        /// At least one error interrupt is pending.
        ACTIVEERRINT OFFSET(2) NUMBITS(1) []
    ],
    CFG [
        /// Pace the transfer with the peripheral DMA request of this channel.
        PERIPHREQEN OFFSET(0) NUMBITS(1) [],
        /// Use hardware triggering.
        HWTRIGEN OFFSET(1) NUMBITS(1) [],
        /// Hardware trigger polarity.
        TRIGPOL OFFSET(4) NUMBITS(1) [
            ActiveLowFallingEdge = 0,
            ActiveHighRisingEdge = 1
        ],
        /// Hardware trigger type.
        TRIGTYPE OFFSET(5) NUMBITS(1) [
            Edge = 0,
            Level = 1
        ],
        /// A hardware trigger causes a burst transfer instead of a single one.
        TRIGBURST OFFSET(6) NUMBITS(1) [],
        /// Burst size and address wrap size, as a power of two.
        BURSTPOWER OFFSET(8) NUMBITS(4) [],
        /// Wrap the source address on each burst.
        SRCBURSTWRAP OFFSET(14) NUMBITS(1) [],
        /// Wrap the destination address on each burst.
        DSTBURSTWRAP OFFSET(15) NUMBITS(1) [],
        /// Channel priority. 0 is the highest and 7 the lowest.
        CHPRIORITY OFFSET(16) NUMBITS(3) []
    ],
    CTLSTAT [
        /// Valid pending flag.
        VALIDPENDING OFFSET(0) NUMBITS(1) [],
        /// The trigger of this channel is set.
        TRIG OFFSET(2) NUMBITS(1) []
    ],
    XFERCFG [
        /// The channel descriptor is valid.
        CFGVALID OFFSET(0) NUMBITS(1) [],
        /// Reload the channel from the next descriptor when this one is exhausted.
        RELOAD OFFSET(1) NUMBITS(1) [],
        /// Set the trigger of this channel immediately.
        SWTRIG OFFSET(2) NUMBITS(1) [],
        /// Clear the trigger when this descriptor is exhausted.
        CLRTRIG OFFSET(3) NUMBITS(1) [],
        /// Set the interrupt A flag when this descriptor is exhausted.
        SETINTA OFFSET(4) NUMBITS(1) [],
        /// Set the interrupt B flag when this descriptor is exhausted.
        SETINTB OFFSET(5) NUMBITS(1) [],
        /// Width of each transfer.
        WIDTH OFFSET(8) NUMBITS(2) [
            Bit8 = 0,
            Bit16 = 1,
            Bit32 = 2
        ],
        /// Source address increment, in multiples of the width.
        SRCINC OFFSET(12) NUMBITS(2) [
            None = 0,
            Width1x = 1,
            Width2x = 2,
            Width4x = 3
        ],
        /// Destination address increment, in multiples of the width.
        DSTINC OFFSET(14) NUMBITS(2) [
            None = 0,
            Width1x = 1,
            Width2x = 2,
            Width4x = 3
        ],
        /// Number of transfers, minus one.
        XFERCOUNT OFFSET(16) NUMBITS(10) []
    ]
];

const DMA0_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x4008_2000 as *const DmaRegisters) };

const DMA1_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x400A_7000 as *const DmaRegisters) };

/// Channel descriptor, read by the controller from memory (UM11126, Table 432).
#[repr(C, align(16))]
struct DmaDescriptor {
    /// Transfer configuration. Reserved on the head descriptor of a channel,
    /// which takes it from the `XFERCFG` register instead.
    xfercfg: InMemoryRegister<u32, XFERCFG::Register>,
    /// Address of the last source item, or the data register of a fixed
    /// peripheral source (UM11126, Section 22.5.2).
    src_end: InMemoryRegister<u32>,
    /// Address of the last destination item, or the data register of a fixed
    /// peripheral destination.
    dst_end: InMemoryRegister<u32>,
    /// Link to the next descriptor. Unused without `XFERCFG::RELOAD`.
    next: InMemoryRegister<u32>,
}

impl DmaDescriptor {
    const fn new() -> Self {
        Self {
            xfercfg: InMemoryRegister::new(0),
            src_end: InMemoryRegister::new(0),
            dst_end: InMemoryRegister::new(0),
            next: InMemoryRegister::new(0),
        }
    }
}

/// Channel descriptor table of a controller.
///
/// `SRAMBASE` holds only address bits 31:9, so the table must be 512-byte
/// aligned (UM11126, Table 439). The controller reads this memory, so the table
/// must live for as long as the controller is enabled.
#[repr(C, align(512))]
pub struct DmaDescriptorTable<const N: usize>([DmaDescriptor; N]);

impl<const N: usize> DmaDescriptorTable<N> {
    pub const fn new() -> Self {
        Self([const { DmaDescriptor::new() }; N])
    }
}

/// Hardwired peripheral request lines, with the channel each one is connected
/// to (UM11126, Tables 428 and 429).
///
/// Use [`Dma::channel_for`] to get the channel of a request. DMA1 only has the
/// requests below channel 10, up to `Flexcomm3Tx`.
///
/// Flexcomm 3 comes before Flexcomm 2 in the hardware.
#[repr(usize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DmaRequest {
    HsSpiRx = 2,
    HsSpiTx = 3,
    Flexcomm0Rx = 4,
    Flexcomm0Tx = 5,
    Flexcomm1Rx = 6,
    Flexcomm1Tx = 7,
    Flexcomm3Rx = 8,
    Flexcomm3Tx = 9,
    Flexcomm2Rx = 10,
    Flexcomm2Tx = 11,
    Flexcomm4Rx = 12,
    Flexcomm4Tx = 13,
    Flexcomm5Rx = 14,
    Flexcomm5Tx = 15,
    Flexcomm6Rx = 16,
    Flexcomm6Tx = 17,
    Flexcomm7Rx = 18,
    Flexcomm7Tx = 19,
}

/// Base address of the Flexcomm interface connected to a channel, if any.
const fn flexcomm_base(channel: usize) -> Option<u32> {
    match channel / 2 {
        1 => Some(0x4009_F000), // High Speed SPI (Flexcomm 8)
        2 => Some(0x4008_6000), // Flexcomm 0
        3 => Some(0x4008_7000), // Flexcomm 1
        4 => Some(0x4008_9000), // Flexcomm 3
        5 => Some(0x4008_8000), // Flexcomm 2
        6 => Some(0x4008_A000), // Flexcomm 4
        7 => Some(0x4009_6000), // Flexcomm 5
        8 => Some(0x4009_7000), // Flexcomm 6
        9 => Some(0x4009_8000), // Flexcomm 7
        _ => None,
    }
}

/// Width of each item moved by a channel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DmaWidth {
    Bits8,
    Bits16,
    Bits32,
}

impl DmaWidth {
    const fn bytes(self) -> usize {
        match self {
            DmaWidth::Bits8 => 1,
            DmaWidth::Bits16 => 2,
            DmaWidth::Bits32 => 4,
        }
    }
}

/// Where the data of a transfer flows.
#[derive(Copy, Clone)]
enum TransferPath {
    MemToMem,
    MemToPeriph,
    PeriphToMem,
}

/// A validated transfer, ready to be programmed into a channel.
struct TransferSetup {
    path: TransferPath,
    width: DmaWidth,
    /// Number of items to move, from 1 to [`MAX_TRANSFERS`].
    items: u32,
    /// Source address for the descriptor.
    src_end: u32,
    /// Destination address for the descriptor.
    dst_end: u32,
}

/// What is left of a transfer stopped by [`DmaChannel::abort`].
pub struct AbortedTransfer {
    /// Number of bytes moved before the transfer stopped.
    pub transferred: usize,
    /// Source buffer, if the source was memory.
    pub source: Option<SubSliceMut<'static, u8>>,
    /// Destination buffer, if the destination was memory.
    pub destination: Option<SubSliceMut<'static, u8>>,
}

/// Receives the buffers of a finished transfer.
pub trait DmaClient {
    /// A transfer finished, or stopped on a bus error.
    ///
    /// `source` and `destination` hold the memory buffers of the transfer: both
    /// for memory-to-memory, and only the one on the memory side otherwise.
    /// `transferred` is the number of bytes moved. `result` is
    /// `Err(ErrorCode::FAIL)` if the controller reported an error, in which
    /// case `transferred` is the best available estimate.
    fn transfer_done(
        &self,
        source: Option<SubSliceMut<'static, u8>>,
        destination: Option<SubSliceMut<'static, u8>>,
        transferred: usize,
        result: Result<(), ErrorCode>,
    );
}

/// One channel of a DMA controller.
pub struct DmaChannel<'a> {
    registers: StaticRef<DmaRegisters>,
    index: usize,
    descriptor: &'static DmaDescriptor,
    client: OptionalCell<&'a dyn DmaClient>,
    fence: CortexMDmaFence,
    /// Source buffer of the transfer in progress, if the source is memory.
    source: MapCell<DmaSubSliceMut<'static, u8>>,
    /// Destination buffer of the transfer in progress, if the destination is
    /// memory.
    destination: MapCell<DmaSubSliceMut<'static, u8>>,
    /// Length in bytes of the transfer in progress, or 0 if the channel is idle.
    ///
    /// The hardware status cannot replace this: `ACTIVE0` clears when the
    /// transfer ends, but the buffers still belong to the channel until the
    /// interrupt has been serviced.
    len: Cell<usize>,
}

impl<'a> DmaChannel<'a> {
    fn new(
        registers: StaticRef<DmaRegisters>,
        index: usize,
        descriptor: &'static DmaDescriptor,
        fence: CortexMDmaFence,
    ) -> Self {
        Self {
            registers,
            index,
            descriptor,
            client: OptionalCell::empty(),
            fence,
            source: MapCell::empty(),
            destination: MapCell::empty(),
            len: Cell::new(0),
        }
    }

    fn mask(&self) -> u32 {
        1 << self.index
    }

    fn channel_registers(&self) -> &ChannelRegisters {
        &self.registers.channel[self.index]
    }

    /// Whether this is the TX channel of an RX/TX pair. Peripheral channels
    /// are paired with RX on the even channel and TX on the odd one.
    fn is_tx(&self) -> bool {
        self.index % 2 == 1
    }

    pub fn set_client(&self, client: &'a dyn DmaClient) {
        self.client.set(client);
    }

    /// Set the arbitration priority of this channel, from 0 (highest) to
    /// [`LOWEST_PRIORITY`].
    pub fn set_priority(&self, priority: u8) -> Result<(), ErrorCode> {
        if priority > LOWEST_PRIORITY {
            return Err(ErrorCode::INVAL);
        }
        self.channel_registers()
            .cfg
            .modify(CFG::CHPRIORITY.val(priority as u32));
        Ok(())
    }

    /// Whether this channel owns buffers for a transfer in progress.
    pub fn is_busy(&self) -> bool {
        self.len.get() != 0
    }

    /// Check that a transfer can start, returning the number of items.
    fn validate(&self, width: DmaWidth, len: usize, addresses: &[usize]) -> Result<u32, ErrorCode> {
        if !self.registers.ctrl.is_set(CTRL::ENABLE) {
            return Err(ErrorCode::OFF);
        }
        if self.is_busy() {
            return Err(ErrorCode::BUSY);
        }

        let bytes = width.bytes();
        let items = len / bytes;
        if len == 0 || !len.is_multiple_of(bytes) || items > MAX_TRANSFERS {
            return Err(ErrorCode::SIZE);
        }
        if addresses.iter().any(|&a| !a.is_multiple_of(bytes)) {
            return Err(ErrorCode::INVAL);
        }
        Ok(items as u32)
    }

    /// Program the channel and start it.
    ///
    /// The descriptor holds the address of the *last* item on each incrementing
    /// side (UM11126, Section 22.5.2). On a fixed peripheral side the address
    /// is the data register itself.
    ///
    /// ### Safety
    ///
    /// Once started, the controller reads and writes the addresses in `setup`
    /// as another bus master, outside the control of the compiler. The caller
    /// must ensure that:
    ///
    /// - Every memory side covers a buffer owned by this channel: the
    ///   `items * width` bytes ending at `src_end` (or `dst_end`) lie inside the
    ///   active range of the [`DmaSubSliceMut`] held in `self.source` (or
    ///   `self.destination`). The channel holds the buffers until
    ///   [`DmaChannel::abort`] or the transfer interrupt hands them back, so no
    ///   other code can access them while the controller does.
    /// - Every peripheral side is the Flexcomm FIFO register wired to the
    ///   request line of this channel, which the controller may access at any
    ///   time during the transfer.
    /// - `items` is from 1 to [`MAX_TRANSFERS`]. The hardware stores `items - 1`
    ///   in the 10-bit `XFERCOUNT` field (UM11126, Section 22.6.18), and the
    ///   value is masked to fit: `items == 0` wraps to 1024 transfers and
    ///   `items > 1024` truncates. Either way the controller would move a
    ///   different number of items than the buffer bounds were checked for, and
    ///   could read or write outside the buffers.
    /// - Both addresses are aligned to `width`, as the controller requires
    ///   (UM11126, Section 22.5.7).
    /// - `self.len` is already non-zero, so that no other transfer can be
    ///   started on this channel and rewrite the descriptor while the
    ///   controller uses it.
    /// - The controller is enabled with `SRAMBASE` pointing at the descriptor
    ///   table that holds `self.descriptor`, as done by [`Dma::init`].
    ///
    /// [`DmaChannel::validate`] checks the size, alignment, busy and enable
    /// conditions.
    unsafe fn start(&self, setup: TransferSetup) {
        let width = match setup.width {
            DmaWidth::Bits8 => XFERCFG::WIDTH::Bit8,
            DmaWidth::Bits16 => XFERCFG::WIDTH::Bit16,
            DmaWidth::Bits32 => XFERCFG::WIDTH::Bit32,
        };
        // A peripheral side has a fixed address and paces the transfer with
        // the request line of the channel.
        let (src_inc, dst_inc, periph_request) = match setup.path {
            TransferPath::MemToMem => (
                XFERCFG::SRCINC::Width1x,
                XFERCFG::DSTINC::Width1x,
                CFG::PERIPHREQEN::CLEAR,
            ),
            TransferPath::MemToPeriph => (
                XFERCFG::SRCINC::Width1x,
                XFERCFG::DSTINC::None,
                CFG::PERIPHREQEN::SET,
            ),
            TransferPath::PeriphToMem => (
                XFERCFG::SRCINC::None,
                XFERCFG::DSTINC::Width1x,
                CFG::PERIPHREQEN::SET,
            ),
        };
        let xfercfg = XFERCFG::CFGVALID::SET
            + XFERCFG::SETINTA::SET
            + width
            + src_inc
            + dst_inc
            + XFERCFG::XFERCOUNT.val(setup.items - 1);

        let regs = self.channel_registers();
        regs.cfg.modify(
            periph_request
                + CFG::HWTRIGEN::CLEAR
                + CFG::TRIGBURST::CLEAR
                + CFG::SRCBURSTWRAP::CLEAR
                + CFG::DSTBURSTWRAP::CLEAR,
        );

        self.descriptor.xfercfg.write(xfercfg);
        self.descriptor.src_end.set(setup.src_end);
        self.descriptor.dst_end.set(setup.dst_end);
        self.descriptor.next.set(0);

        // The controller reads the descriptor from memory, so the writes above
        // must be visible to it before the channel is enabled. A completed
        // transfer clears the trigger of the channel (UM11126, Table 455), but
        // an aborted one may leave it set, and the channel would then start as
        // soon as it is enabled, before `SWTRIG`. The channel is disabled here,
        // as every transfer ends through `disable`, so writing `XFERCFG`
        // cannot start it. The interrupt is still enabled first, so that it
        // is on before anything that could start the channel. The buffers
        // were released when their `DmaSubSliceMut`s were created.
        self.fence
            .release(ptr::from_ref(core::slice::from_ref(self.descriptor)).cast_mut());

        self.registers.intenset0.set(self.mask());
        regs.xfercfg.write(xfercfg);
        self.registers.enableset0.set(self.mask());
        regs.xfercfg.modify(XFERCFG::SWTRIG::SET);
    }

    /// Copy the active range of `source` to the start of the active range of
    /// `destination`.
    ///
    /// The length of `source` must be a multiple of the width, fit in
    /// `destination`, and be at most [`MAX_TRANSFERS`] items. Both active ranges
    /// must start at an address aligned to the width (UM11126, Section 22.5.7).
    ///
    /// On failure the buffers are returned with one of:
    ///
    /// - `OFF`: [`Dma::init`] has not been called.
    /// - `BUSY`: the channel has a transfer in progress.
    /// - `SIZE`: the length of `source` is invalid.
    /// - `INVAL`: a buffer is not aligned to the width.
    pub fn transfer_mem_to_mem(
        &self,
        width: DmaWidth,
        source: SubSliceMut<'static, u8>,
        destination: SubSliceMut<'static, u8>,
    ) -> Result<
        (),
        (
            ErrorCode,
            SubSliceMut<'static, u8>,
            SubSliceMut<'static, u8>,
        ),
    > {
        let len = source.len();
        let items = match self.validate(
            width,
            len,
            &[source.as_ptr().addr(), destination.as_ptr().addr()],
        ) {
            Ok(_) if destination.len() < len => {
                return Err((ErrorCode::SIZE, source, destination));
            }
            Ok(items) => items,
            Err(e) => return Err((e, source, destination)),
        };

        let source = DmaSubSliceMut::new_static(source, self.fence);
        let destination = DmaSubSliceMut::new_static(destination, self.fence);
        let last_item = (len - width.bytes()) as u32;
        let src_end = source.ptr_addr() as u32 + last_item;
        let dst_end = destination.ptr_addr() as u32 + last_item;
        self.len.set(len);
        self.source.replace(source);
        self.destination.replace(destination);
        // SAFETY: `len` is the length of the active range of `source`, and was
        // checked above to fit in the active range of `destination`.
        // `validate` checked that it is `items` items of `width`, and that
        // both active ranges are aligned to `width`. Each range ends
        // `last_item` bytes into the active range of a `DmaSubSliceMut` that is
        // now held by this channel, and `self.len` is set. `validate` also
        // checked that the controller is enabled.
        unsafe {
            self.start(TransferSetup {
                path: TransferPath::MemToMem,
                width,
                items,
                src_end,
                dst_end,
            })
        };
        Ok(())
    }

    /// Send the active range of `buffer` to the Flexcomm FIFO of this channel,
    /// paced by the peripheral request line.
    ///
    /// The channel must be a TX channel of [`DmaRequest`], otherwise this
    /// returns `NOSUPPORT`. See [`DmaChannel::transfer_mem_to_mem`] for the
    /// other errors.
    ///
    /// # Safety Considerations
    ///
    /// The controller writes `FIFOWR` of the Flexcomm wired to this channel
    /// directly, outside of any driver for that Flexcomm. This cannot break
    /// memory safety, as the controller only reads `buffer`, which the channel
    /// holds, and writes a fixed MMIO register. For the transfer to be correct,
    /// the caller must ensure that, until the transfer ends or is aborted:
    ///
    /// - The caller owns the TX FIFO of that Flexcomm, and no other code
    ///   writes `FIFOWR` or reconfigures, resets or disables the TX FIFO.
    ///   Otherwise the data sent is interleaved with other writes, or lost.
    /// - The Flexcomm is powered, clocked, set to the function the data is
    ///   meant for, and has its TX FIFO and DMA TX request (`FIFOCFG.ENABLETX`
    ///   and `FIFOCFG.DMATX`) enabled. Without the request the transfer never
    ///   ends, and the channel keeps `buffer` until [`DmaChannel::abort`].
    pub fn transfer_mem_to_periph(
        &self,
        width: DmaWidth,
        buffer: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        let fifo = match flexcomm_base(self.index) {
            Some(base) if self.is_tx() => base + FLEXCOMM_FIFOWR,
            _ => return Err((ErrorCode::NOSUPPORT, buffer)),
        };
        let len = buffer.len();
        let items = match self.validate(width, len, &[buffer.as_ptr().addr()]) {
            Ok(items) => items,
            Err(e) => return Err((e, buffer)),
        };

        let buffer = DmaSubSliceMut::new_static(buffer, self.fence);
        let src_end = buffer.ptr_addr() as u32 + (len - width.bytes()) as u32;
        self.len.set(len);
        self.source.replace(buffer);
        // SAFETY: `validate` checked that `len` is `items` items of `width`,
        // and that the active range of `buffer` is aligned to `width`. The
        // source range is that active range, of a `DmaSubSliceMut` that is now
        // held by this channel, and `self.len` is set. `fifo` is the `FIFOWR`
        // register of the Flexcomm wired to this TX channel. `validate` also
        // checked that the controller is enabled.
        unsafe {
            self.start(TransferSetup {
                path: TransferPath::MemToPeriph,
                width,
                items,
                src_end,
                dst_end: fifo,
            })
        };
        Ok(())
    }

    /// Receive into the active range of `buffer` from the Flexcomm FIFO of this
    /// channel, paced by the peripheral request line.
    ///
    /// The channel must be an RX channel of [`DmaRequest`], otherwise this
    /// returns `NOSUPPORT`. See [`DmaChannel::transfer_mem_to_mem`] for the
    /// other errors.
    ///
    /// # Safety Considerations
    ///
    /// The controller reads `FIFORD` of the Flexcomm wired to this channel
    /// directly, outside of any driver for that Flexcomm, and each read pops an
    /// entry from the RX FIFO. This cannot break memory safety, as the
    /// controller only writes `buffer`, which the channel holds, and reads a
    /// fixed MMIO register. For the transfer to be correct, the caller must
    /// ensure that, until the transfer ends or is aborted:
    ///
    /// - The caller owns the RX FIFO of that Flexcomm, and no other code reads
    ///   `FIFORD` or reconfigures, resets or disables the RX FIFO. Otherwise
    ///   entries are split between the readers.
    /// - The Flexcomm is powered, clocked, set to the function the data comes
    ///   from, and has its RX FIFO and DMA RX request (`FIFOCFG.ENABLERX` and
    ///   `FIFOCFG.DMARX`) enabled. Without the request the transfer never
    ///   ends, and the channel keeps `buffer` until [`DmaChannel::abort`].
    pub fn transfer_periph_to_mem(
        &self,
        width: DmaWidth,
        buffer: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        let fifo = match flexcomm_base(self.index) {
            Some(base) if !self.is_tx() => base + FLEXCOMM_FIFORD,
            _ => return Err((ErrorCode::NOSUPPORT, buffer)),
        };
        let len = buffer.len();
        let items = match self.validate(width, len, &[buffer.as_ptr().addr()]) {
            Ok(items) => items,
            Err(e) => return Err((e, buffer)),
        };

        let buffer = DmaSubSliceMut::new_static(buffer, self.fence);
        let dst_end = buffer.ptr_addr() as u32 + (len - width.bytes()) as u32;
        self.len.set(len);
        self.destination.replace(buffer);
        // SAFETY: `validate` checked that `len` is `items` items of `width`,
        // and that the active range of `buffer` is aligned to `width`. The
        // destination range is that active range, of a `DmaSubSliceMut` that
        // is now held by this channel, and `self.len` is set. `fifo` is the
        // `FIFORD` register of the Flexcomm wired to this RX channel.
        // `validate` also checked that the controller is enabled.
        unsafe {
            self.start(TransferSetup {
                path: TransferPath::PeriphToMem,
                width,
                items,
                src_end: fifo,
                dst_end,
            })
        };
        Ok(())
    }

    /// Take back the buffers of the transfer, making the writes of the
    /// controller visible to Rust.
    ///
    /// ### Safety
    ///
    /// The controller must no longer access the buffers: the channel has been
    /// stopped with [`disable`](Self::disable), which waits until its pipeline
    /// has drained.
    unsafe fn take_buffers(
        &self,
    ) -> (
        Option<SubSliceMut<'static, u8>>,
        Option<SubSliceMut<'static, u8>>,
    ) {
        // SAFETY: The caller guarantees that the controller no longer accesses
        // either buffer.
        let source = self.source.take().map(|b| unsafe { b.take(self.fence) });
        // SAFETY: As above.
        let destination = self
            .destination
            .take()
            .map(|b| unsafe { b.take(self.fence) });
        (source, destination)
    }

    /// Disable the channel and its interrupt, and wait until the controller
    /// no longer accesses the buffers.
    ///
    /// Clearing `ENABLE0` stops the channel from starting new operations, but
    /// operations already in the controller's pipeline still complete.
    /// `BUSY0` stays set until they have (UM11126, Section 22.6.15). The wait
    /// is unbounded for that reason, but it only covers operations already in
    /// the pipeline.
    fn disable(&self) {
        self.registers.intenclr0.set(self.mask());
        self.registers.enableclr0.set(self.mask());
        while self.registers.busy0.get() & self.mask() != 0 {}
    }

    /// Stop the channel (UM11126, Section 22.6.15), returning the number of
    /// bytes it moved.
    ///
    /// When this returns, the controller no longer accesses the buffers, as
    /// [`disable`](Self::disable) has waited for it.
    fn halt(&self) -> usize {
        self.disable();

        let xfercfg = self.channel_registers().xfercfg.extract();
        let count = xfercfg.read(XFERCFG::XFERCOUNT);
        let active = self.registers.active0.get() & self.mask() != 0;
        self.registers.abort0.set(self.mask());

        // `XFERCOUNT` wraps to 0x3FF after the last item, which is also the
        // encoding of 1024 items left. Only `ACTIVE0` tells the two apart.
        let items_left = if count == XFERCOUNT_DONE && !active {
            0
        } else {
            count as usize + 1
        };
        let width = match xfercfg.read_as_enum(XFERCFG::WIDTH) {
            Some(XFERCFG::WIDTH::Value::Bit16) => DmaWidth::Bits16,
            Some(XFERCFG::WIDTH::Value::Bit32) => DmaWidth::Bits32,
            _ => DmaWidth::Bits8,
        };
        self.len
            .get()
            .saturating_sub(items_left.saturating_mul(width.bytes()))
    }

    /// Stop the transfer in progress without calling the client.
    ///
    /// On an idle channel the result has no buffers and 0 bytes moved.
    pub fn abort(&self) -> AbortedTransfer {
        if !self.is_busy() {
            return AbortedTransfer {
                transferred: 0,
                source: None,
                destination: None,
            };
        }

        let transferred = self.halt();
        // The transfer may have ended just before it was stopped. Clear its
        // flags so a stale interrupt is not taken for the next transfer.
        self.registers.inta0.set(self.mask());
        self.registers.errint0.set(self.mask());
        self.len.set(0);
        // SAFETY: `halt` called `disable`, which waited until `BUSY0` cleared,
        // so the controller no longer accesses the buffers.
        let (source, destination) = unsafe { self.take_buffers() };
        AbortedTransfer {
            transferred,
            source,
            destination,
        }
    }

    /// Finish the transfer in progress and return the buffers to the client.
    ///
    /// ### Safety
    ///
    /// This hands the buffers of the transfer back to the client, so the
    /// controller must no longer access them. Both paths go through
    /// [`disable`](Self::disable), which waits until the controller's pipeline
    /// has drained. With `error` set, [`halt`](Self::halt) also aborts the
    /// channel. Without `error`, the transfer is reported as complete, so the
    /// caller must ensure that the controller has finished it: `INTA0` of this
    /// channel was set by the transfer that is in progress, and not left over
    /// from an earlier one.
    ///
    /// `INTA0` is set when the descriptor is exhausted (UM11126, Section
    /// 22.6.11), while `ACTIVE0` stays set until the pipeline is empty
    /// (Section 22.6.6).
    unsafe fn complete(&self, error: bool) {
        let len = self.len.get();
        if len == 0 {
            return;
        }

        let (transferred, result) = if error {
            (self.halt(), Err(ErrorCode::FAIL))
        } else {
            // `halt` cannot report this: `ACTIVE0` may still be set while the
            // pipeline drains, which reads as 1024 items left. `disable` still
            // waits for the pipeline to drain before the buffers are taken.
            self.disable();
            (len, Ok(()))
        };

        self.len.set(0);
        // SAFETY: Both paths called `disable`, which waited until `BUSY0`
        // cleared, so the controller no longer accesses the buffers. Without
        // `error`, the caller also guarantees that the transfer has finished.
        let (source, destination) = unsafe { self.take_buffers() };
        self.client
            .map(|client| client.transfer_done(source, destination, transferred, result));
    }
}

/// A DMA controller with `N` channels.
pub struct Dma<'a, const N: usize> {
    registers: StaticRef<DmaRegisters>,
    descriptors: &'static DmaDescriptorTable<N>,
    channels: [DmaChannel<'a>; N],
}

/// The DMA0 controller.
pub type Dma0<'a> = Dma<'a, DMA0_CHANNELS>;
/// The DMA1 controller.
pub type Dma1<'a> = Dma<'a, DMA1_CHANNELS>;

impl<'a> Dma0<'a> {
    /// Create the driver of the DMA0 controller.
    ///
    /// # Safety
    ///
    /// This driver controls DMA hardware, and its soundness relies on being
    /// the only code that starts transfers on DMA0: each channel tracks the
    /// buffers it owns, and a second instance could program the same channel
    /// to access memory that the first instance has handed back to Rust. This
    /// requires:
    ///
    /// - This constructor must be called at most once.
    /// - There must not be any other code that accesses the registers of the
    ///   DMA0 controller, or the descriptor table passed in `descriptors`.
    pub unsafe fn new_dma0(descriptors: &'static DmaDescriptorTable<DMA0_CHANNELS>) -> Self {
        Self::new(DMA0_BASE, descriptors)
    }

    /// Get the channel wired to a peripheral request.
    ///
    /// Unlike [`Dma::channel_for`] this cannot fail, because DMA0 has a channel
    /// for every [`DmaRequest`].
    pub fn request_channel(&self, request: DmaRequest) -> &DmaChannel<'a> {
        &self.channels[request as usize]
    }
}

impl Dma1<'_> {
    /// Create the driver of the DMA1 controller.
    ///
    /// # Safety
    ///
    /// This driver controls DMA hardware, and its soundness relies on being
    /// the only code that starts transfers on DMA1: each channel tracks the
    /// buffers it owns, and a second instance could program the same channel
    /// to access memory that the first instance has handed back to Rust. This
    /// requires:
    ///
    /// - This constructor must be called at most once.
    /// - There must not be any other code that accesses the registers of the
    ///   DMA1 controller, or the descriptor table passed in `descriptors`.
    pub unsafe fn new_dma1(descriptors: &'static DmaDescriptorTable<DMA1_CHANNELS>) -> Self {
        Self::new(DMA1_BASE, descriptors)
    }
}

impl<'a, const N: usize> Dma<'a, N> {
    fn new(
        registers: StaticRef<DmaRegisters>,
        descriptors: &'static DmaDescriptorTable<N>,
    ) -> Self {
        // SAFETY: The LPC55S6x has an ARM Cortex-M33 core, for which
        // `CortexMDmaFence` is the architecture-provided `DmaFence`. The chip has no data cache in
        // front of SRAM, so the `DMB` issued by this fence is sufficient to
        // order CPU and DMA accesses.
        let fence = unsafe { CortexMDmaFence::new() };
        Self {
            registers,
            descriptors,
            channels: core::array::from_fn(|index| {
                DmaChannel::new(registers, index, &descriptors.0[index], fence)
            }),
        }
    }

    /// Point the controller at its descriptor table and enable it.
    ///
    /// The controller clock must be running, see
    /// [`Clock::start_dma_clocks`](crate::clocks::Clock::start_dma_clocks).
    pub fn init(&self) {
        let table = core::ptr::from_ref(self.descriptors);
        self.registers.srambase.set(table as u32);
        self.registers.ctrl.write(CTRL::ENABLE::SET);
    }

    /// Get a channel by number, or `None` if the controller does not have it.
    ///
    /// Use this for memory-to-memory transfers, and [`Dma::channel_for`] for
    /// peripheral transfers.
    pub fn channel(&self, index: usize) -> Option<&DmaChannel<'a>> {
        self.channels.get(index)
    }

    /// Get the channel wired to a peripheral request, or `None` if this
    /// controller does not have that request.
    pub fn channel_for(&self, request: DmaRequest) -> Option<&DmaChannel<'a>> {
        self.channel(request as usize)
    }

    pub fn handle_interrupt(&self) {
        for channel in &self.channels {
            let mask = channel.mask();

            // Read the flags of this channel only now, not once for all
            // channels: the client callback of an earlier channel may have
            // aborted this channel and started a new transfer on it. `abort`
            // clears the flags of the stopped transfer, so reading them here
            // never takes a flag of an earlier transfer for the one in
            // progress.
            let done = self.registers.inta0.get() & mask;
            let error = self.registers.errint0.get() & mask;
            if (done | error) == 0 {
                continue;
            }

            // Clear only the flags read above, and before the callback, so
            // that a flag raised in between, or by a transfer the callback
            // starts, is kept and raises the interrupt again.
            self.registers.inta0.set(done);
            self.registers.errint0.set(error);

            // SAFETY: If the error flag of the channel is set, `complete`
            // halts the channel before returning the buffers. Otherwise
            // `INTA0` of the channel is set, and it belongs to the transfer in
            // progress: it was read just above, after any callback that could
            // have restarted the channel. `start` is only called on an idle
            // channel, and `abort` clears both flags when it stops a transfer,
            // so no flag of an earlier transfer is left.
            unsafe { channel.complete(error != 0) };
        }
    }
}
