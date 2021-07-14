//! Direct Memory Access (DMA) channels and multiplexer
//!
//! ## DMAMUX Channel Configuration Options
//!
//! | ENBL | TRIG | A_ON | Function                                                | Mode                   |
//! |------|------|------|---------------------------------------------------------|------------------------|
//! |   0  |   X  |   X  | DMA channel is disabled                                 | Disabled Mode          |
//! |   1  |   0  |   0  | DMA channel is enabled with no triggering (transparent) | Normal Mode            |
//! |   1  |   1  |   0  | DMA channel is enabled with triggering                  | Periodic Trigger Mode  |
//! |   1  |   0  |   1  | DMA channel is always enabled                           | Always On Mode         |
//! |   1  |   1  |   1  | DMA channel is always enabled with triggering           | Always On Trigger Mode |
//!
//! Implementation assumptions:
//!
//! - No minor loop mapping, assuming we don't need to change addresses on minor loop runs.
//! - The driver exposes 32 DMA channels. This applies for nearly all i.MX RT 10xx chips, except for the 1011.
//!   Accessing any DMA channel beyond 15 will index into reserved memory.
//!
//! When assigning DMA channels to peripherals, consider:
//!
//! - How you could use channels that are 16 channel IDs apart, and complete DMA transfers with signaling
//!   from one DMA interrupt, instead of two separate interrupts.
//! - The first four DMA channels can be periodically scheduled from the four periodic interrupt timer (PIT)
//!   channels. Consider reserving those first four channels if you need to regularly schedule DMA transfers
//!   without CPU intervention.
//! - Channel priorities may come into play when preferring DMA channels. See the reference manual for more
//!   information on channel priorities, and how the DMA controller use priorities for scheduling.

use kernel::common::{
    cells::OptionalCell,
    registers::{
        self,
        interfaces::{ReadWriteable, Readable, Writeable},
        ReadOnly, ReadWrite, WriteOnly,
    },
    StaticRef,
};

use core::cell::Cell;
use core::mem;
use core::ops::Index;

use crate::ccm;

/// DMA Multiplexer.
///
/// The multiplexer is used for routing between DMA channels and hardware
/// peripherals. It's a detail of `DmaChannel`.
#[repr(C)]
struct DmaMultiplexerRegisters {
    /// Channel configuration registers, one per channel.
    chcfg: [ReadWrite<u32, ChannelConfiguration::Register>; 32],
}

const DMA_MUX_BASE: StaticRef<DmaMultiplexerRegisters> =
    unsafe { StaticRef::new(0x400E_C000 as *const DmaMultiplexerRegisters) };

registers::register_bitfields![u32,
    /// Each of the DMA channels can be independently enabled/disabled and associated
    /// with one of the DMA slots (peripheral slots or always-on slots) in the system.
    ///
    /// Note: Setting multiple CHCFG registers with the same source value will result in
    /// unpredictable behavior. This is true, even if a channel is disabled (ENBL==0).
    ///
    /// Note: Before changing the trigger or source settings, a DMA channel must be
    /// disabled via CHCFGn[ENBL].
    ChannelConfiguration [
        /// Enables the channel for DMA Mux. The DMA has separate channel
        /// enables/disables, which should be used to disable or reconfigure
        /// a DMA channel.
        ENBL OFFSET(31) NUMBITS(1) [],
        /// Enables the periodic trigger capability for the triggered DMA channel.
        ///
        /// 0b - Triggering is disabled. If triggering is disabled and ENBL is set,
        /// the DMA Channel will simply route the specified source to the DMA channel.
        /// (Normal mode)
        ///
        /// 1b - Triggering is enabled. If triggering is enabled and ENBL is set,
        /// the DMA_CH_MUX is in Periodic Trigger mode.
        TRIG OFFSET(30) NUMBITS(1) [],
        /// DMA Channel Always Enable
        ///
        /// Enables the DMA Channel to be always ON.
        /// If TRIG bit is set, the module will assert request on every trigger.
        ///
        /// 0b - DMA Channel Always ON function is disabled
        /// 1b - DMA Channel Always ON function is enabled
        A_ON OFFSET(29) NUMBITS(1) [],
        /// DMA Channel Source (Slot Number)
        ///
        /// Specifies which DMA source, if any, is routed to a particular DMA channel.
        /// See the "DMA MUX Mapping" table in the "Interrupts, DMA Events, and XBAR
        /// Assignments" chapter for details about DMA source and channel information.
        SOURCE OFFSET(0) NUMBITS(7) []
    ]
];

#[repr(C, align(32))]
struct TransferControlDescriptor {
    saddr: ReadWrite<u32>,
    soff: ReadWrite<u16>, // Signed number
    attr: ReadWrite<u16, TransferAttributes::Register>,
    nbytes: ReadWrite<u32>, // Assumes minor loop mapping is disabled (EMLM = 0)
    slast: ReadWrite<u32>,  // Signed number
    daddr: ReadWrite<u32>,
    doff: ReadWrite<u16>, // Signed number
    citer: ReadWrite<u16>,
    dlast_sga: ReadWrite<u32>, // Signed number
    csr: ReadWrite<u16, ControlAndStatus::Register>,
    biter: ReadWrite<u16>,
}

impl TransferControlDescriptor {
    fn reset(&self) {
        self.saddr.set(0);
        self.soff.set(0);
        self.attr.set(0);
        self.nbytes.set(0);
        self.slast.set(0);
        self.daddr.set(0);
        self.doff.set(0);
        self.citer.set(0);
        self.dlast_sga.set(0);
        self.csr.set(0);
        self.biter.set(0);
    }
}

const _STATIC_ASSERT_TCD_32_BYTES: [u32; 1] =
    [0; (32 == mem::size_of::<TransferControlDescriptor>()) as usize];

registers::register_bitfields![u16,
    TransferAttributes [
        SMOD OFFSET(11) NUMBITS(5) [],
        SSIZE OFFSET(8) NUMBITS(3) [],
        DMOD OFFSET(3) NUMBITS(5) [],
        DSIZE OFFSET(0) NUMBITS(3) []
    ],

    ControlAndStatus [
        /// Bandwidth control.
        ///
        /// Throttle bandwidth consumed by DMA.
        BWC OFFSET(14) NUMBITS(2) [
            /// No engine stalls
            NoStalls = 0b00,
            /// Stalls for 4 cycles after each R/W
            FourCycles = 0b10,
            /// Stalls for 8 cycles after each R/W
            EightCycles = 0b11
        ],
        /// Major loop link channel number.
        ///
        /// If zero, then no channel-to-channel linking is performed
        /// after major loop count exhaustion.
        ///
        /// Otherwise, the DMA engine initiates a channel service request
        /// at the channel defined here, setting START in that channel.
        MAJORLINKCH OFFSET(8) NUMBITS(5) [],
        /// Channel done.
        ///
        /// Must be clear to write MAJORELINK or ESG
        DONE OFFSET(7) NUMBITS(1) [],
        /// Channel active
        ACTIVE OFFSET(6) NUMBITS(1) [],
        /// Enable channel-to-channel linking on major loop completion.
        MAJORELINK OFFSET(5) NUMBITS(1) [],
        /// Enable scatter/gatter.
        ESG OFFSET(4) NUMBITS(1) [],
        /// Disable request.
        ///
        /// If set, DMA hardware clears ERQ when the current major iteration
        /// count reaches zero.
        DREQ OFFSET(3) NUMBITS(1) [],
        /// Enable interrupt when major count is half complete.
        INTHALF OFFSET(2) NUMBITS(1) [],
        /// Enable an interrupt when major count is complete.
        INTMAJOR OFFSET(1) NUMBITS(1) [],
        /// Channel start.
        ///
        /// When set, channel is requesting service. DMA hardware will clear this
        /// after it starts execution.
        START OFFSET(0) NUMBITS(1) []
    ]
];

/// Wrapper for channel priority registers.
///
/// Channel priority registers cannot be accessed with
/// normal channel indexes. This adapter makes it so that
/// we *can* access them with channel indexes by converting
/// the channel number to a reference to the priority
/// register.
#[repr(transparent)]
struct ChannelPriorityRegisters([ReadWrite<u8, ChannelPriority::Register>; 32]);

impl Index<usize> for ChannelPriorityRegisters {
    type Output = ReadWrite<u8, ChannelPriority::Register>;
    fn index(&self, channel: usize) -> &ReadWrite<u8, ChannelPriority::Register> {
        // Pattern follows
        //
        //   3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, ...
        //
        // for all channels < 32. NXP keeping us on our toes.
        let idx = 4 * (channel / 4) + (3 - (channel % 4));
        &self.0[idx]
    }
}

registers::register_structs! {
    /// DMA registers.
    DmaRegisters {
        /// Control Register
        (0x000 => cr: ReadWrite<u32, Control::Register>),
        /// Error Status Register
        (0x004 => es: ReadOnly<u32, ErrorStatus::Register>),
        (0x008 => _reserved0),
        /// Enable Request Register
        (0x00C => erq: ReadWrite<u32>),
        (0x010 => _reserved1),
        /// Enable Error Interrupt Register
        (0x014 => eei: ReadWrite<u32>),
        /// Clear Enable Error Interrupt Register
        (0x018 => ceei: WriteOnly<u8, MemoryMappedChannel::Register>),
        /// Set Enable Error Interrupt Register
        (0x019 => seei: WriteOnly<u8, MemoryMappedChannel::Register>),
        /// Clear Enable Request Register
        (0x01A => cerq: WriteOnly<u8, MemoryMappedChannel::Register>),
        /// Set Enable Request Register
        (0x01B => serq: WriteOnly<u8, MemoryMappedChannel::Register>),
        /// Clear DONE Status Bit Register
        (0x01C => cdne: WriteOnly<u8, MemoryMappedChannel::Register>),
        /// Set START Bit Register
        (0x01D => ssrt: WriteOnly<u8, MemoryMappedChannel::Register>),
        /// Clear Error Register
        (0x01E => cerr: WriteOnly<u8, MemoryMappedChannel::Register>),
        /// Clear Interrupt Request Register
        (0x01F => cint: WriteOnly<u8, MemoryMappedChannel::Register>),
        (0x020 => _reserved2),
        /// Interrupt Request Register
        (0x024 => int: ReadWrite<u32>),
        (0x028 => _reserved3),
        /// Error Register
        (0x02C => err: ReadWrite<u32>),
        (0x030 => _reserved4),
        /// Hardware Request Status Register
        (0x034 => hrs: ReadOnly<u32>),
        (0x038 => _reserved5),
        /// Enable Asynchronous Request in Stop Register
        (0x044 => ears: ReadWrite<u32>),
        (0x048 => _reserved6),
        (0x0100 => dchpri: ChannelPriorityRegisters),
        (0x0120 => _reserved7),
        (0x1000 => tcd: [TransferControlDescriptor; 32]),
        (0x1400 => @END),
    }
}

registers::register_bitfields![u8,
    /// Used in DCHPRI registers.
    ChannelPriority [
        /// Enable channel premption.
        ///
        /// 0b - Channel n cannot be suspended by a higher priority channel's service request.
        /// 1b - Channel n can be temporarily suspended by the service request of a higher priority channel.
        ECP OFFSET(7) NUMBITS(1) [],
        /// Disable Preempt Ability.
        ///
        /// 0b - Channel n can suspend a lower priority channel.
        /// 1b - Channel n cannot suspend any channel, regardless of channel priority.
        DPA OFFSET(6) NUMBITS(1) [],
        /// Channel current group priority.
        ///
        /// Group priority assigned to this channel group when
        /// fixed-priority arbitration is enabled. This field is
        /// read- only; writes are ignored.
        GRPPRI OFFSET(4) NUMBITS(2) [],
        /// Channel arbitration priority.
        CHPRI OFFSET(0) NUMBITS(4) []
    ],
    /// Generic bitband register for CEEI, SEEI, CERQ, SERQ, ...
    MemoryMappedChannel [
        /// NoOp operation.
        ///
        /// Disable all other bits in this register.
        NOOP OFFSET(7) NUMBITS(1) [],
        /// Perform this register's operation on all 32 channels.
        ALL OFFSET(6) NUMBITS(1) [],
        /// Channel number.
        ///
        /// Specify the channel to act on.
        CHANNEL OFFSET(0) NUMBITS(5) []
    ]
];

registers::register_bitfields![u32,
    Control [
        /// DMA active status.
        ACTIVE OFFSET(31) NUMBITS(1) [],
        /// Cancel the active transfer.
        CX OFFSET(17) NUMBITS(1) [],
        /// Error cancel transfer.
        ///
        /// Like cancel transfer (CX), but it updates the error
        /// status register (ES) for the channel. It optionally
        /// generates an error interrupt.
        ECX OFFSET(16) NUMBITS(1) [],
        /// Channel group 1 priority.
        ///
        /// Group 1 priority level when fixed priority group arbitration is enabled.
        GRP1PRI OFFSET(10) NUMBITS(1) [],
        /// Channel group 9 priority.
        ///
        /// Group 0 priority level when fixed priority group arbitration is enabled.
        GRP0PRI OFFSET(8) NUMBITS(1) [],
        /// Enable minor loop mapping.
        ///
        /// 0b - Disabled. TCDn.word2 is defined as a 32-bit NBYTES field.
        /// 1b - Enabled. TCDn.word2 is redefined to include individual enable fields,
        /// an offset field, and the NBYTES field. The individual enable fields allow
        /// the minor loop offset to be applied to the source address, the destination
        /// address, or both. The NBYTES field is reduced when either offset is enabled.
        EMLM OFFSET(7) NUMBITS(1) [],
        /// Continuous link mode.
        CLM OFFSET(6) NUMBITS(1) [],
        /// Halt DMA operations.
        ///
        /// Writing 1 stalls the start of any new channels. Executing channels may complete. Write
        /// 0 to resume channel execution.
        HALT OFFSET(5) NUMBITS(1) [],
        /// Halt on Error.
        ///
        /// Any error sets HALT bit. Software must clear HALT.
        HOE OFFSET(4) NUMBITS(1) [],
        /// Enable round robin group arbitration.
        ///
        /// 0b - Fixed priority arbitration is used for selection among the groups.
        /// 1b - Round robin arbitration is used for selection among the groups.
        ERGA OFFSET(3) NUMBITS(1) [],
        /// Enable round robin channel arbitration.
        ///
        /// 0b - Fixed priority arbitration is used for channel selection within each group.
        /// 1b - Round robin arbitration is used for channel selection within each group.
        ERCA OFFSET(2) NUMBITS(1) [],
        /// Enable debug.
        ///
        /// Set to stall the start of a new channel when in debug mode.
        EDBG OFFSET(1) NUMBITS(1) []
    ],
    ErrorStatus [
        /// At least one ERR bit is set.
        VLD OFFSET(31) NUMBITS(1) [],
        /// Transfer canceled.
        ///
        /// Last recorded entry was a cancelled transfer by error cancel transfer input.
        ECX OFFSET(16) NUMBITS(1) [],
        /// Group priority error.
        ///
        /// Priority groups are not unique.
        GPE OFFSET(15) NUMBITS(1) [],
        /// Channel priority error.
        ///
        /// Channel priorities within a group are not unique.
        CPE OFFSET(14) NUMBITS(1) [],
        /// Error channel number.
        ///
        /// Channel number of last recorded error, excluding group or channel priority errors,
        /// or last error canceled transfer.
        ERRCHN OFFSET(8) NUMBITS(5) [],
        /// Source address error.
        ///
        /// Configuration error detected in the TCDn_SADDR field. TCDn_SADDR is inconsistent with TCDn_ATTR[SSIZE].
        SAE OFFSET(7) NUMBITS(1) [],
        /// Source offset error.
        ///
        /// Configuration error detected in the TCDn_SOFF field. TCDn_SOFF is inconsistent with TCDn_ATTR[SSIZE].
        SOE OFFSET(6) NUMBITS(1) [],
        /// Destination address error.
        ///
        /// Configuration error detected in the TCDn_DADDR field. TCDn_DADDR is inconsistent with TCDn_ATTR[DSIZE].
        DAE OFFSET(5) NUMBITS(1) [],
        /// Destination offset error.
        ///
        /// Configuration error detected in the TCDn_DOFF field. TCDn_DOFF is inconsistent with TCDn_ATTR[DSIZE].
        DOE OFFSET(4) NUMBITS(1) [],
        /// NBYTES/CITER configuration error.
        NCE OFFSET(3) NUMBITS(1) [],
        /// Scatter/Gather Configuration Error.
        SGE OFFSET(2) NUMBITS(1) [],
        /// Source bus error.
        SBE OFFSET(1) NUMBITS(1) [],
        /// Destination bus error.
        DBE OFFSET(0) NUMBITS(1) []
    ]
];

const DMA_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x400E_8000 as *const DmaRegisters) };

/// A DMA channel.
///
/// `DmaChannel` can coordinate the transfer of data between buffers and
/// peripherals without processor intervention.
pub struct DmaChannel {
    base: StaticRef<DmaRegisters>,
    mux: StaticRef<DmaMultiplexerRegisters>,
    channel: usize,
    client: OptionalCell<&'static dyn DmaClient>,
    hardware_source: Cell<Option<DmaHardwareSource>>,
}

/// Describes a type that can be transferred via DMA.
///
/// This trait is sealed and cannot be implemented outside of this
/// crate. However, it may be used outside of this crate.
pub trait DmaElement: private::Sealed {
    /// An identifier describing the data transfer size
    ///
    /// See TCD\[SSIZE\] and TCD\[DSIZE\] for more information.
    #[doc(hidden)] // Crate implementation detail
    const DATA_TRANSFER_ID: u16;
}

/// Details for the sealed `DmaElement` trait.
///
/// See the Rust API Guidelines, and the Sealed trait pattern,
/// for more information.
///
/// <https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed>
mod private {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
}

impl DmaElement for u8 {
    const DATA_TRANSFER_ID: u16 = 0;
}

impl DmaElement for u16 {
    const DATA_TRANSFER_ID: u16 = 1;
}

impl DmaElement for u32 {
    const DATA_TRANSFER_ID: u16 = 2;
}

impl DmaElement for u64 {
    const DATA_TRANSFER_ID: u16 = 3;
}

impl DmaChannel {
    /// Allocate a new DMA channel.
    ///
    /// Note that channels 0 through 3 are the only channels capable of periodic
    /// transfers. Consider reserving these channels for that use case.
    pub(crate) const fn new(channel: usize) -> Self {
        DmaChannel {
            base: DMA_BASE,
            mux: DMA_MUX_BASE,
            channel,
            client: OptionalCell::empty(),
            hardware_source: Cell::new(None),
        }
    }

    /// Reset the DMA channel's TCD.
    fn reset_tcd(&self) {
        self.base.tcd[self.channel].reset();
    }

    /// Set the client using this DMA channel.
    ///
    /// This should be invoked by the client itself.
    pub(crate) fn set_client(&self, client: &'static dyn DmaClient, source: DmaHardwareSource) {
        self.client.set(client);
        self.trigger_from_hardware(source);
    }

    /// Set this DMA channel to trigger from a hardware source.
    fn trigger_from_hardware(&self, source: DmaHardwareSource) {
        let chcfg = &self.mux.chcfg[self.channel];
        chcfg.set(0);
        chcfg.write(
            ChannelConfiguration::ENBL::SET + ChannelConfiguration::SOURCE.val(source as u32),
        );
        self.hardware_source.set(Some(source));
    }

    /// Manually start the DMA transfer.
    ///
    /// A manual trigger is useful for memory-to-memory DMA transfers. If you're sending
    /// or receiving data from a peripheral, use `trigger_from_hardware()`.
    pub fn trigger_manually(&self) {
        self.base
            .ssrt
            .write(MemoryMappedChannel::CHANNEL.val(self.channel as u8));
    }

    /// Returns `true` is this DMA channel is actively receiving a hardware signal.
    ///
    /// A hardware signal comes from an associated peripheral, indicating a request
    /// for transfer. It's important to deassert the hardware before disabling a
    /// DMA channel. This gives you an opportunity to check for hardware signal.
    ///
    /// Returns `false` if the DMA channel is disabled, or if there's no associated
    /// hardware (see `trigger_from_hardware()`).
    pub fn is_hardware_signaling(&self) -> bool {
        self.base.hrs.get() & (1 << self.channel) != 0
    }

    /// Enables this DMA channel.
    pub fn enable(&self) {
        self.base
            .serq
            .write(MemoryMappedChannel::CHANNEL.val(self.channel as u8));
    }

    /// Disables this DMA channel.
    pub fn disable(&self) {
        self.base
            .cerq
            .write(MemoryMappedChannel::CHANNEL.val(self.channel as u8));
    }

    /// Clear the interrupt associated with this DMA channel.
    fn clear_interrupt(&self) {
        self.base
            .cint
            .write(MemoryMappedChannel::CHANNEL.val(self.channel as u8));
    }

    /// Returns `true` if this DMA channel generated an interrupt.
    pub fn is_interrupt(&self) -> bool {
        self.base.int.get() & (1 << self.channel) != 0
    }

    /// Returns `true` if this DMA channel has completed its transfer.
    pub fn is_complete(&self) -> bool {
        self.base.tcd[self.channel]
            .csr
            .is_set(ControlAndStatus::DONE)
    }

    /// Clears the completion of this DMA channel.
    fn clear_complete(&self) {
        self.base
            .cdne
            .write(MemoryMappedChannel::CHANNEL.val(self.channel as u8));
    }

    /// Returns `true` if this DMA channel is in an error state.
    pub fn is_error(&self) -> bool {
        self.base.err.get() & (1 << self.channel) != 0
    }

    /// Clears the error flag for this channel.
    fn clear_error(&self) {
        self.base
            .cerr
            .write(MemoryMappedChannel::CHANNEL.val(self.channel as u8));
    }

    /// Returns `true` if this DMA channel is in an active transfer.
    pub fn is_active(&self) -> bool {
        self.base.tcd[self.channel]
            .csr
            .is_set(ControlAndStatus::ACTIVE)
    }

    /// Set a buffer of data as the source of a DMA transfer.
    ///
    /// Safety: caller is responsible for ensuring the buffer's lifetime is
    /// valid for the life of the transfer.
    pub unsafe fn set_source_buffer<T: DmaElement>(&self, buffer: &[T]) {
        let tcd = &self.base.tcd[self.channel];
        tcd.saddr.set(buffer.as_ptr() as u32);
        tcd.soff.set(mem::size_of::<T>() as u16);
        tcd.attr.modify(
            TransferAttributes::SSIZE.val(T::DATA_TRANSFER_ID) + TransferAttributes::SMOD.val(0),
        );
        tcd.nbytes.set(mem::size_of::<T>() as u32);
        tcd.slast.set((-1 * (buffer.len() as i32)) as u32);
        let iterations: u16 = buffer.len() as u16;
        tcd.biter.set(iterations);
        tcd.citer.set(iterations);
    }

    /// Set a buffer of data as the destination of a DMA receive.
    ///
    /// Safety: caller is responsible for ensuring the buffer's lifetime is
    /// valid for the life of the transfer.
    pub unsafe fn set_destination_buffer<T: DmaElement>(&self, buffer: &mut [T]) {
        let tcd = &self.base.tcd[self.channel];
        tcd.daddr.set(buffer.as_mut_ptr() as u32);
        tcd.doff.set(mem::size_of::<T>() as u16);
        tcd.attr.modify(
            TransferAttributes::DSIZE.val(T::DATA_TRANSFER_ID) + TransferAttributes::DMOD.val(0),
        );
        tcd.nbytes.set(mem::size_of::<T>() as u32);
        tcd.dlast_sga.set((-1 * (buffer.len() as i32)) as u32);
        let iterations: u16 = buffer.len() as u16;
        tcd.biter.set(iterations);
        tcd.citer.set(iterations);
    }

    /// Set the source of a DMA transfer.
    ///
    /// Use `set_source` if the transfer source is a peripheral register.
    ///
    /// Safety: caller responsible for ensuring pointer's lifetime is valid
    /// for the transfer.
    pub unsafe fn set_source<T: DmaElement>(&self, source: *const T) {
        let tcd = &self.base.tcd[self.channel];
        tcd.saddr.set(source as u32);
        tcd.soff.set(0);
        tcd.attr.modify(
            TransferAttributes::SSIZE.val(T::DATA_TRANSFER_ID) + TransferAttributes::SMOD.val(0),
        );
        tcd.nbytes.set(mem::size_of::<T>() as u32);
        tcd.slast.set(0);
    }

    /// Set the destination of a DMA transfer.
    ///
    /// Use `set_destination` if the tranfer destination is a peripheral register.
    ///
    /// Safety: caller responsible for ensuring pointer's lifetime is valid for
    /// the transfer.
    pub unsafe fn set_destination<T: DmaElement>(&self, dest: *const T) {
        let tcd = &self.base.tcd[self.channel];
        tcd.daddr.set(dest as u32);
        tcd.doff.set(0);
        tcd.attr.modify(
            TransferAttributes::DSIZE.val(T::DATA_TRANSFER_ID) + TransferAttributes::DMOD.val(0),
        );
        tcd.nbytes.set(mem::size_of::<T>() as u32);
        tcd.dlast_sga.set(0);
    }

    /// Configures the DMA channel to automatically disable when the transfer completes.
    pub fn set_disable_on_completion(&self, dreq: bool) {
        self.base.tcd[self.channel]
            .csr
            .modify(ControlAndStatus::DREQ.val(dreq as u16));
    }

    /// Configures the DMA channel to interrupt when complete, or when there
    /// is an error.
    pub fn set_interrupt_on_completion(&self, intr: bool) {
        self.base.tcd[self.channel]
            .csr
            .modify(ControlAndStatus::INTMAJOR.val(intr as u16));
        if intr {
            self.base
                .seei
                .write(MemoryMappedChannel::CHANNEL.val(self.channel as u8));
        } else {
            self.base
                .ceei
                .write(MemoryMappedChannel::CHANNEL.val(self.channel as u8));
        }
    }

    /// Handle an interrupt.
    ///
    /// Assumes that the caller knows that this DMA channel was the source of the
    /// interrupt, or the cause of a DMA error. See `is_interrupt()` and `is_error()`.
    /// The implementation panics if there is neither an error, or an interrupt.
    pub fn handle_interrupt(&self) {
        self.clear_interrupt();
        let hardware_source = self.hardware_source.clone().get().unwrap();
        let result = if self.is_error() {
            self.clear_error();
            self.clear_complete();
            self.disable();
            Err(hardware_source)
        } else if self.is_complete() {
            self.clear_complete();
            Ok(hardware_source)
        } else {
            unreachable!(
                "DMA Channel {} should either be complete, or in an error state",
                self.channel
            );
        };
        self.client.map(|client| client.transfer_complete(result));
    }
}

/// Indicates success or failure when executing a DMA transfer
///
/// An `Ok(source)` describes a successful DMA transfer to / from the hardware
/// source. An `Err(source)` describes a failed DMA transfer.
pub type Result = core::result::Result<DmaHardwareSource, DmaHardwareSource>;

/// A type that responds to DMA completion events
pub trait DmaClient {
    /// Handle the completion of a DMA transfer, which either succeeded or failed.
    fn transfer_complete(&self, source: Result);
}

/// DMA hardware sources.
///
/// Extend this to add support for more DMA-powered peripherals.
/// To understand where the numbers come from, see Chapter 4,
/// DMA Mux, to find the DMA request signals (iMXRT1060RM, Rev 2).
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DmaHardwareSource {
    Lpuart1Transfer = 2,
    Lpuart1Receive = 3,
    Lpuart2Transfer = 66,
    Lpuart2Receive = 67,
}

/// The DMA peripheral exposes DMA channels.
pub struct Dma<'a> {
    /// The DMA channels
    pub channels: [DmaChannel; 32],
    /// DMA clock gate
    clock_gate: ccm::PeripheralClock<'a>,
    /// DMA registers.
    registers: StaticRef<DmaRegisters>,
}

impl<'a> Dma<'a> {
    /// Create a DMA peripheral.
    pub const fn new(ccm: &'a ccm::Ccm) -> Self {
        Dma {
            channels: DMA_CHANNELS,
            clock_gate: ccm::PeripheralClock::ccgr5(ccm, ccm::HCLK5::DMA),
            registers: DMA_BASE,
        }
    }

    /// Returns the interface that controls the DMA clock
    pub fn clock(&self) -> &(impl kernel::platform::chip::ClockInterface + '_) {
        &self.clock_gate
    }

    /// Reset all DMA transfer control descriptors.
    ///
    /// You should reset these descriptors shortly after system
    /// initialization, and before using a DMA channel.
    pub fn reset_tcds(&self) {
        for channel in &self.channels {
            channel.reset_tcd();
        }
    }

    /// Returns a DMA channel that has an error.
    ///
    /// This will be faster than searching all DMA channels
    /// for an error flag. However, if more than one DMA channel
    /// has an error, there's no guarantee which will be returned
    /// first. You should continue calling, and clearing errors,
    /// until this returns `None`.
    pub fn error_channel(&self) -> Option<&DmaChannel> {
        let es = self.registers.es.extract();
        es.is_set(ErrorStatus::VLD).then(|| {
            let idx = es.read(ErrorStatus::ERRCHN) as usize;
            &self.channels[idx]
        })
    }
}

/// Helper constant for allocating DMA channels.
const DMA_CHANNELS: [DmaChannel; 32] = [
    DmaChannel::new(0),
    DmaChannel::new(1),
    DmaChannel::new(2),
    DmaChannel::new(3),
    DmaChannel::new(4),
    DmaChannel::new(5),
    DmaChannel::new(6),
    DmaChannel::new(7),
    DmaChannel::new(8),
    DmaChannel::new(9),
    DmaChannel::new(10),
    DmaChannel::new(11),
    DmaChannel::new(12),
    DmaChannel::new(13),
    DmaChannel::new(14),
    DmaChannel::new(15),
    DmaChannel::new(16),
    DmaChannel::new(17),
    DmaChannel::new(18),
    DmaChannel::new(19),
    DmaChannel::new(20),
    DmaChannel::new(21),
    DmaChannel::new(22),
    DmaChannel::new(23),
    DmaChannel::new(24),
    DmaChannel::new(25),
    DmaChannel::new(26),
    DmaChannel::new(27),
    DmaChannel::new(28),
    DmaChannel::new(29),
    DmaChannel::new(30),
    DmaChannel::new(31),
];
