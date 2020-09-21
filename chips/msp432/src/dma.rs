//! Direct Memory Access (DMA)

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{
    register_bitfields, register_structs, InMemoryRegister, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;

pub static mut DMA_CHANNELS: [DmaChannel<'static>; 8] = [
    DmaChannel::new(0),
    DmaChannel::new(1),
    DmaChannel::new(2),
    DmaChannel::new(3),
    DmaChannel::new(4),
    DmaChannel::new(5),
    DmaChannel::new(6),
    DmaChannel::new(7),
];

const DMA_BASE: StaticRef<DmaRegisters> =
    unsafe { StaticRef::new(0x4000_E000 as *const DmaRegisters) };

static DMA_CONFIG: DmaConfigBlock = DmaConfigBlock([
    // Unfortunately the Default-trait does not support constant functions and the InMemoryRegister
    // structs do not implement the copy-trait, so it's necessary to initialize this array by hand.
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
    DmaChannelControl::const_default(),
]);

/// Although there are 8bits reserved for selecting a source for the DMA trigger, the
/// MSP432P4x family only supports numbers from 1 to 7. 0 doesn't cause an error, but it's
/// marked as reserved. According to the device-specific datasheet 'reserved' can be used for
/// transfering data from one location in the RAM to another one.
const MAX_SRC_NR: u8 = 7;

/// The MSP432 chips contain 8 DMA channels
const AVAILABLE_DMA_CHANNELS: usize = 8;

/// The DMA can perform up to 1024 transfers before it needs to rearbitrate the bus
const MAX_TRANSFERS_LEN: usize = 1024;

register_structs! {
    /// DMA
    DmaRegisters {
        /// Device Configuration Status
        (0x0000 => device_cfg: ReadOnly<u32, DMA_DEVICE_CFG::Register>),
        /// Software Channel Trigger Register
        (0x0004 => sw_chtrig: ReadWrite<u32, DMA_SW_CHTRIG::Register>),
        (0x0008 => _reserved0),
        /// Channel n Source Configuration Registers
        (0x0010 => ch_srccfg: [ReadWrite<u32>; 32]),
        (0x0090 => _reserved1),
        /// Interrupt 1 Source Channel Configuration
        (0x0100 => int1_srccfg: ReadWrite<u32, DMA_INT1_SRCCFG::Register>),
        /// Interrupt 2 Source Channel Configuration Register
        (0x0104 => int2_srccfg: ReadWrite<u32, DMA_INT2_SRCCFG::Register>),
        /// Interrupt 3 Source Channel Configuration Register
        (0x0108 => int3_srccfg: ReadWrite<u32, DMA_INT3_SRCCFG::Register>),
        (0x010C => _reserved2),
        /// Interrupt 0 Source Channel Flag Register
        (0x0110 => int0_srcflg: ReadOnly<u32, DMA_INT0_SRCFLG::Register>),
        /// Interrupt 0 Source Channel Clear Flag Register
        (0x0114 => int0_clrflg: WriteOnly<u32, DMA_INT0_CLRFLG::Register>),
        (0x0118 => _reserved3),
        /// Status Register
        (0x1000 => stat: ReadOnly<u32, DMA_STAT::Register>),
        /// Configuration Register
        (0x1004 => cfg: WriteOnly<u32, DMA_CFG::Register>),
        /// Channel Control Data Base Pointer Register
        (0x1008 => ctlbase: ReadWrite<u32>),
        /// Channel Alternate Control Data Base Pointer Register
        (0x100C => altbase: ReadOnly<u32>),
        /// Channel Wait on Request Status Register
        (0x1010 => waitstat: ReadOnly<u32>),
        /// Channel Software Request Register
        (0x1014 => wreq: WriteOnly<u32>),
        /// Channel Useburst Set Register
        (0x1018 => useburstset: ReadWrite<u32>),
        /// Channel Useburst Clear Register
        (0x101C => useburstclr: WriteOnly<u32>),
        /// Channel Request Mask Set Register
        (0x1020 => reqmaskset: ReadWrite<u32>),
        /// Channel Request Mask Clear Register
        (0x1024 => reqmaskclr: WriteOnly<u32>),
        /// Channel Enable Set Register
        (0x1028 => enaset: ReadWrite<u32>),
        /// Channel Enable Clear Register
        (0x102C => enaclr: WriteOnly<u32>),
        /// Channel Primary-Alternate Set Register
        (0x1030 => altset: ReadWrite<u32>),
        /// Channel Primary-Alternate Clear Register
        (0x1034 => altclr: WriteOnly<u32>),
        /// Channel Priority Set Register
        (0x1038 => prioset: ReadWrite<u32>),
        /// Channel Priority Clear Register
        (0x103C => prioclr: WriteOnly<u32>),
        (0x1040 => _reserved4),
        /// Bus Error Clear Register
        (0x104C => errclr: ReadWrite<u32>),
        (0x1050 => @END),
    }
}

register_bitfields![u32,
    DMA_DEVICE_CFG [
        /// Number of DMA channels available
        NUM_DMA_CHANNELS OFFSET(0) NUMBITS(8) [],
        /// Number of DMA sources per channel
        NUM_SRC_PER_CHANNEL OFFSET(8) NUMBITS(8) []
    ],
    DMA_SW_CHTRIG [
        /// Write 1, triggers DMA_CHANNEL0
        CH0 OFFSET(0) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL1
        CH1 OFFSET(1) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL2
        CH2 OFFSET(2) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL3
        CH3 OFFSET(3) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL4
        CH4 OFFSET(4) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL5
        CH5 OFFSET(5) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL6
        CH6 OFFSET(6) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL7
        CH7 OFFSET(7) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL8
        CH8 OFFSET(8) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL9
        CH9 OFFSET(9) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL10
        CH10 OFFSET(10) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL11
        CH11 OFFSET(11) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL12
        CH12 OFFSET(12) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL13
        CH13 OFFSET(13) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL14
        CH14 OFFSET(14) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL15
        CH15 OFFSET(15) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL16
        CH16 OFFSET(16) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL17
        CH17 OFFSET(17) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL18
        CH18 OFFSET(18) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL19
        CH19 OFFSET(19) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL20
        CH20 OFFSET(20) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL21
        CH21 OFFSET(21) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL22
        CH22 OFFSET(22) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL23
        CH23 OFFSET(23) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL24
        CH24 OFFSET(24) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL25
        CH25 OFFSET(25) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL26
        CH26 OFFSET(26) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL27
        CH27 OFFSET(27) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL28
        CH28 OFFSET(28) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL29
        CH29 OFFSET(29) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL30
        CH30 OFFSET(30) NUMBITS(1) [],
        /// Write 1, triggers DMA_CHANNEL31
        CH31 OFFSET(31) NUMBITS(1) []
    ],
    DMA_INT1_SRCCFG [
        /// Controls which channel's completion event is mapped as a source of this Interrup
        INT_SRC OFFSET(0) NUMBITS(5) [],
        /// Enables DMA_INT1 mapping
        EN OFFSET(5) NUMBITS(1) []
    ],
    DMA_INT2_SRCCFG [
        /// Controls which channel's completion event is mapped as a source of this Interrup
        INT_SRC OFFSET(0) NUMBITS(5) [],
        /// Enables DMA_INT2 mapping
        EN OFFSET(5) NUMBITS(1) []
    ],
    DMA_INT3_SRCCFG [
        /// Controls which channel's completion event is mapped as a source of this Interrup
        INT_SRC OFFSET(0) NUMBITS(5) [],
        /// Enables DMA_INT3 mapping
        EN OFFSET(5) NUMBITS(1) []
    ],
    DMA_INT0_SRCFLG [
        /// Channel 0 was the source of DMA_INT0
        CH0 OFFSET(0) NUMBITS(1) [],
        /// Channel 1 was the source of DMA_INT0
        CH1 OFFSET(1) NUMBITS(1) [],
        /// Channel 2 was the source of DMA_INT0
        CH2 OFFSET(2) NUMBITS(1) [],
        /// Channel 3 was the source of DMA_INT0
        CH3 OFFSET(3) NUMBITS(1) [],
        /// Channel 4 was the source of DMA_INT0
        CH4 OFFSET(4) NUMBITS(1) [],
        /// Channel 5 was the source of DMA_INT0
        CH5 OFFSET(5) NUMBITS(1) [],
        /// Channel 6 was the source of DMA_INT0
        CH6 OFFSET(6) NUMBITS(1) [],
        /// Channel 7 was the source of DMA_INT0
        CH7 OFFSET(7) NUMBITS(1) [],
        /// Channel 8 was the source of DMA_INT0
        CH8 OFFSET(8) NUMBITS(1) [],
        /// Channel 9 was the source of DMA_INT0
        CH9 OFFSET(9) NUMBITS(1) [],
        /// Channel 10 was the source of DMA_INT0
        CH10 OFFSET(10) NUMBITS(1) [],
        /// Channel 11 was the source of DMA_INT0
        CH11 OFFSET(11) NUMBITS(1) [],
        /// Channel 12 was the source of DMA_INT0
        CH12 OFFSET(12) NUMBITS(1) [],
        /// Channel 13 was the source of DMA_INT0
        CH13 OFFSET(13) NUMBITS(1) [],
        /// Channel 14 was the source of DMA_INT0
        CH14 OFFSET(14) NUMBITS(1) [],
        /// Channel 15 was the source of DMA_INT0
        CH15 OFFSET(15) NUMBITS(1) [],
        /// Channel 16 was the source of DMA_INT0
        CH16 OFFSET(16) NUMBITS(1) [],
        /// Channel 17 was the source of DMA_INT0
        CH17 OFFSET(17) NUMBITS(1) [],
        /// Channel 18 was the source of DMA_INT0
        CH18 OFFSET(18) NUMBITS(1) [],
        /// Channel 19 was the source of DMA_INT0
        CH19 OFFSET(19) NUMBITS(1) [],
        /// Channel 20 was the source of DMA_INT0
        CH20 OFFSET(20) NUMBITS(1) [],
        /// Channel 21 was the source of DMA_INT0
        CH21 OFFSET(21) NUMBITS(1) [],
        /// Channel 22 was the source of DMA_INT0
        CH22 OFFSET(22) NUMBITS(1) [],
        /// Channel 23 was the source of DMA_INT0
        CH23 OFFSET(23) NUMBITS(1) [],
        /// Channel 24 was the source of DMA_INT0
        CH24 OFFSET(24) NUMBITS(1) [],
        /// Channel 25 was the source of DMA_INT0
        CH25 OFFSET(25) NUMBITS(1) [],
        /// Channel 26 was the source of DMA_INT0
        CH26 OFFSET(26) NUMBITS(1) [],
        /// Channel 27 was the source of DMA_INT0
        CH27 OFFSET(27) NUMBITS(1) [],
        /// Channel 28 was the source of DMA_INT0
        CH28 OFFSET(28) NUMBITS(1) [],
        /// Channel 29 was the source of DMA_INT0
        CH29 OFFSET(29) NUMBITS(1) [],
        /// Channel 30 was the source of DMA_INT0
        CH30 OFFSET(30) NUMBITS(1) [],
        /// Channel 31 was the source of DMA_INT0
        CH31 OFFSET(31) NUMBITS(1) []
    ],
    DMA_INT0_CLRFLG [
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH0 OFFSET(0) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH1 OFFSET(1) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH2 OFFSET(2) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH3 OFFSET(3) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH4 OFFSET(4) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH5 OFFSET(5) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH6 OFFSET(6) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH7 OFFSET(7) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH8 OFFSET(8) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH9 OFFSET(9) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH10 OFFSET(10) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH11 OFFSET(11) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH12 OFFSET(12) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH13 OFFSET(13) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH14 OFFSET(14) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH15 OFFSET(15) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH16 OFFSET(16) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH17 OFFSET(17) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH18 OFFSET(18) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH19 OFFSET(19) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH20 OFFSET(20) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH21 OFFSET(21) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH22 OFFSET(22) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH23 OFFSET(23) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH24 OFFSET(24) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH25 OFFSET(25) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH26 OFFSET(26) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH27 OFFSET(27) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH28 OFFSET(28) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH29 OFFSET(29) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH30 OFFSET(30) NUMBITS(1) [],
        /// Clear corresponding DMA_INT0_SRCFLG_REG
        CH31 OFFSET(31) NUMBITS(1) []
    ],
    DMA_STAT [
        /// Enable status of the controller
        MASTEN OFFSET(0) NUMBITS(1) [
            /// Controller disabled
            ControllerDisabled = 0,
            /// Controller enabled
            ControllerEnabled = 1
        ],
        /// Current state of the control state machine.
        /// State can be one of the following:
        STATE OFFSET(4) NUMBITS(4) [
            /// idle
            Idle = 0,
            /// reading channel controller data
            ReadingChannelControllerData = 1,
            /// reading source data end pointer
            ReadingSourceDataEndPointer = 2,
            /// reading destination data end pointer
            ReadingDestinationDataEndPointer = 3,
            /// reading source data
            ReadingSourceData = 4,
            /// writing destination data
            WritingDestinationData = 5,
            /// waiting for DMA request to clear
            WaitingForDMARequestToClear = 6,
            /// writing channel controller data
            WritingChannelControllerData = 7,
            /// stalled
            Stalled = 8,
            /// done
            Done = 9,
            /// peripheral scatter-gather transition
            PeripheralScatterGatherTransition = 10
        ],
        /// Number of available DMA channels minus one.
        DMACHANS OFFSET(16) NUMBITS(5) [
            /// Controller configured to use 1 DMA channel
            ControllerConfiguredToUse1DMAChannel = 0,
            /// Controller configured to use 2 DMA channels
            ControllerConfiguredToUse2DMAChannels = 1,
            /// Controller configured to use 31 DMA channels
            ControllerConfiguredToUse31DMAChannels = 30,
            /// Controller configured to use 32 DMA channels
            ControllerConfiguredToUse32DMAChannels = 31
        ],
        /// To reduce the gate count the controller can be configured to exclude the integra
        TESTSTAT OFFSET(28) NUMBITS(4) [
            /// Controller does not include the integration test logic
            ControllerDoesNotIncludeTheIntegrationTestLogic = 0,
            /// Controller includes the integration test logic
            ControllerIncludesTheIntegrationTestLogic = 1
        ]
    ],
    DMA_CFG [
        /// Enable status of the controller
        MASTEN OFFSET(0) NUMBITS(1) [
            /// Controller disabled
            ControllerDisabled = 0,
            /// Controller enabled
            ControllerEnabled = 1
        ],
        /// Sets the AHB-Lite protection by controlling the HPROT[3:1] signal levels as fol
        CHPROTCTRL OFFSET(5) NUMBITS(3) []
    ]
];

register_bitfields![u32,
    /// DMA control data configuration
    DMA_CTRL [
        /// Cycle control
        CYCLE_CTRL OFFSET(0) NUMBITS(3) [
            /// Stop. indicates that the data-structure is invalid
            Stop = 0,
            /// Basic transfer mode
            Basic = 1,
            /// Auto-request mode
            Auto = 2,
            /// Ping-pong mode
            PingPong = 3,
            /// Memory scatter-gather mode, which uses the primary data-structure
            MemoryScatterGatherPrimary = 4,
            /// Memory scatter-gather mode, which uses the alternate data-structure
            MemoryScatterGatherAlternate = 5,
            /// Peripheral scatter-gather mode, which uses the primary data-structure
            PeripheralScatterGatherPrimary = 6,
            /// Peripheral scatter-gather mode, which uses the alternate data-structure
            PeripheralScatterGatherAlternate = 7
        ],
        /// Controls if the chnl_useburst_set bit is is set to 1
        NEXT_USEBURST OFFSET(3) NUMBITS(1) [],
        /// These bits represent the total number of DMA transfers minus 1
        /// that the DMA cycle contains.
        N_MINUS_1 OFFSET(4) NUMBITS(10) [],
        /// These bits control how many DMA transfers can occur before the controller rearbitrates
        /// the bus. The register-value is the 'ld' of the number of cycles. E.g. 128 cycles ->
        /// regval = ld(128) = 7. The maximum number of cycles is 1024 -> regval = 10
        R_POWER OFFSET(14) NUMBITS(4) [],
        /// These bits set the control state of HPROT[3:1] when the controller reads data. For the
        /// MSP432 family these bits can be ignored, because they have no effect. The DMA in the
        /// MSP432-devices can access all memory, no matter how these bits are set.
        SRC_PROT_CTRL OFFSET(18) NUMBITS(3) [],
        /// These bits set the control state of HPROT[3:1] when the controller writes data. For the
        /// MSP432 family these bits can be ignored, because they have no effect. The DMA in the
        /// MSP432-devices can access all memory, no matter how these bits are set.
        DST_PROT_CTRL OFFSET(21) NUMBITS(3) [],
        /// These bits control the size of the source-data
        SRC_SIZE OFFSET(24) NUMBITS(2) [
            /// Byte -> 8bit
            Byte = 0,
            /// Half-word -> 16bit
            HalfWord = 1,
            /// Word -> 32bit
            Word = 2
        ],
        /// These bits set the source address-increment
        SRC_INC OFFSET(26) NUMBITS(2) [
            /// Byte -> +1
            Byte = 0,
            /// Half-word -> +2
            HalfWord = 1,
            /// Word -> +4
            Word = 2,
            /// No increment -> +0
            NoIncrement = 3
        ],
        /// These bits control the size of the destination-data.
        /// NOTE: DST_SIZE must be the same as SRC_SIZE!
        DST_SIZE OFFSET(28) NUMBITS(2) [
            /// Byte -> 8bit
            Byte = 0,
            /// Half-word -> 16bit
            HalfWord = 1,
            /// Word -> 32bit
            Word = 2
        ],
        /// These bits set the destination address-increment
        DST_INC OFFSET(30) NUMBITS(2) [
            /// Byte -> +1
            Byte = 0,
            /// Half-word -> +2
            HalfWord = 1,
            /// Word -> +4
            Word = 2,
            /// No increment -> +0
            NoIncrement = 3
        ]
    ]
];

/// The uDMA of the MSP432 family don't offer own registers where the configuration of the
/// individual DMA channels is stored, they require a pointer to a block of memory in the RAM
/// where the actual configuration is stored. Within this block of memory the pointer to the
/// data-source, the pointer to the destination and the configuration is stored. Probably due to
/// alignment reasons the 4th word is unused.
#[repr(align(16))]
struct DmaChannelControl {
    src_ptr: InMemoryRegister<u32>,
    dst_ptr: InMemoryRegister<u32>,
    ctrl: InMemoryRegister<u32, DMA_CTRL::Register>,
    _unused: InMemoryRegister<u32>,
}

/// It's necessary to allocate twice as much buffers as DMA channels are available. This is because
/// the DMA supports modes where a primary buffer and an alternative one can be used.
#[repr(align(256))]
struct DmaConfigBlock([DmaChannelControl; 2 * AVAILABLE_DMA_CHANNELS]);

/// It's necessary to implement `Sync`, otherwise the `DMA_CONFIG` array cannot be instantiated
/// because static variables require the `Sync` trait, otherwise they are not threadsafe.
unsafe impl Sync for DmaConfigBlock {}

/// Trait for handling the callbacks if a DMA transfer finished
pub trait DmaClient {
    fn transfer_done(
        &self,
        tx_buf: Option<&'static mut [u8]>,
        rx_buf: Option<&'static mut [u8]>,
        transmitted_bytes: usize,
    );
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
pub enum DmaMode {
    Basic = 1,
    AutoRequest = 2,
    PingPong = 3,
    MemoryScatterGather = 4,
    PeripheralScatterGather = 6,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum DmaDataWidth {
    Width8Bit,
    Width16Bit,
    Width32Bit,
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
pub enum DmaPtrIncrement {
    Incr8Bit,
    Incr16Bit,
    Incr32Bit,
    NoIncr,
}

#[derive(Copy, Clone)]
pub struct DmaConfig {
    pub src_chan: u8,
    pub mode: DmaMode,
    pub width: DmaDataWidth,
    pub src_incr: DmaPtrIncrement,
    pub dst_incr: DmaPtrIncrement,
}

#[derive(Copy, Clone, PartialEq)]
enum DmaTransferType {
    PeripheralToMemory,
    MemoryToPeripheral,
    MemoryToMemory,
    None,
}

pub struct DmaChannel<'a> {
    registers: StaticRef<DmaRegisters>,
    chan_nr: usize,
    in_use: Cell<bool>,
    config: Cell<DmaConfig>,
    transfer_type: Cell<DmaTransferType>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    bytes_to_transmit: Cell<usize>,
    remaining_words: Cell<usize>,
    client: OptionalCell<&'a dyn DmaClient>,
}

impl DmaChannelControl {
    const fn const_default() -> Self {
        Self {
            src_ptr: InMemoryRegister::new(0),
            dst_ptr: InMemoryRegister::new(0),
            ctrl: InMemoryRegister::new(0),
            _unused: InMemoryRegister::new(0),
        }
    }
}

impl DmaConfig {
    const fn const_default() -> Self {
        Self {
            src_chan: 1,
            mode: DmaMode::Basic,
            width: DmaDataWidth::Width8Bit,
            // Default is to copy data from a hardware to a buffer
            src_incr: DmaPtrIncrement::NoIncr,
            dst_incr: DmaPtrIncrement::Incr8Bit,
        }
    }
}

impl<'a> DmaChannel<'a> {
    const fn new(chan_nr: usize) -> DmaChannel<'a> {
        DmaChannel {
            registers: DMA_BASE,
            chan_nr: chan_nr,
            in_use: Cell::new(false),
            config: Cell::new(DmaConfig::const_default()),
            transfer_type: Cell::new(DmaTransferType::None),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            bytes_to_transmit: Cell::new(0),
            remaining_words: Cell::new(0),
            client: OptionalCell::empty(),
        }
    }

    fn dma_is_enabled(&self) -> bool {
        self.registers.stat.is_set(DMA_STAT::MASTEN)
    }

    fn enable_dma(&self) {
        // Enable the DMA module
        self.registers.cfg.write(DMA_CFG::MASTEN::ControllerEnabled);

        // Set the pointer to the configuration-memory
        // Since the config needs exactly 256 bytes, mask out the lower 256 bytes
        let addr = (&DMA_CONFIG.0[0] as *const DmaChannelControl as u32) & (!0xFFu32);
        self.registers.ctlbase.set(addr);
    }

    fn apply_config(&self) {
        let conf = self.config.get();

        if conf.mode == DmaMode::PingPong {
            panic!("DMA: Ping Pong mode currently not supported!");
        }
        if conf.mode == DmaMode::MemoryScatterGather {
            panic!("DMA: Memory scatter-gather mode currently not supported!");
        }
        if conf.mode == DmaMode::PeripheralScatterGather {
            panic!("DMA: Peripheral scatter-gather mode currently not supported!");
        }

        // The memory acces protection fields 'dst_prot_ctrl' and 'src_prot_ctrl' are not necessary
        // to configure for the MSP432P4x chips because they don't affect the privileges of the
        // DMA module. In other words, the DMA can access every memory and register at every time.
        // For more information see datasheet p. 625 section 11.2.2.3.

        DMA_CONFIG.0[self.chan_nr].ctrl.modify(
            DMA_CTRL::SRC_SIZE.val(conf.width as u32)
                + DMA_CTRL::DST_SIZE.val(conf.width as u32)
                + DMA_CTRL::SRC_INC.val(conf.src_incr as u32)
                + DMA_CTRL::DST_INC.val(conf.dst_incr as u32),
        );

        // Set the source-peripheral for the DMA channel
        self.registers.ch_srccfg[self.chan_nr].set((conf.src_chan % (MAX_SRC_NR + 1)) as u32);
    }

    fn enable_dma_channel(&self) {
        self.registers
            .enaset
            .set(self.registers.enaset.get() | ((1 << self.chan_nr) as u32));
    }

    fn setup_transfer(&self, src_end_ptr: u32, dst_end_ptr: u32, len: usize) {
        let conf = self.config.get();
        let width = conf.width as u32;

        // Divide the byte-length by the width to get the number of necessary transfers
        let transfers = len >> width;

        DMA_CONFIG.0[self.chan_nr].src_ptr.set(src_end_ptr);
        DMA_CONFIG.0[self.chan_nr].dst_ptr.set(dst_end_ptr);

        DMA_CONFIG.0[self.chan_nr].ctrl.modify(
            // The DMA can only transmit 1024 words with 1 transfer
            DMA_CTRL::N_MINUS_1.val(((transfers - 1) % MAX_TRANSFERS_LEN) as u32)
            // Reset the bits in case they were set before to a different value
            + DMA_CTRL::R_POWER.val(0)
            // Set the DMA mode since it the DMA module sets it back to to stop after every cycle
            + DMA_CTRL::CYCLE_CTRL.val(conf.mode as u32),
        );

        // Store to transmitting bytes
        self.bytes_to_transmit.set(len);

        // Store the remaining words
        if transfers > MAX_TRANSFERS_LEN {
            self.remaining_words.set(transfers - MAX_TRANSFERS_LEN);
        } else {
            self.remaining_words.set(0);
        }
    }

    fn handle_interrupt(&self) {
        let len = self.bytes_to_transmit.get();
        let rem_words = self.remaining_words.get();
        let tt = self.transfer_type.get();
        let conf = self.config.get();

        if rem_words > 0 {
            // Update the buffer-pointers
            if (tt == DmaTransferType::PeripheralToMemory)
                || (tt == DmaTransferType::MemoryToMemory)
            {
                // Update the destination-buffer pointer
                DMA_CONFIG.0[self.chan_nr].dst_ptr.set(
                    DMA_CONFIG.0[self.chan_nr].dst_ptr.get()
                        + ((MAX_TRANSFERS_LEN as u32) << (conf.width as u32)),
                );
            }

            if (tt == DmaTransferType::MemoryToPeripheral)
                || (tt == DmaTransferType::MemoryToMemory)
            {
                // Update the source-buffer pointer
                DMA_CONFIG.0[self.chan_nr].src_ptr.set(
                    DMA_CONFIG.0[self.chan_nr].src_ptr.get()
                        + ((MAX_TRANSFERS_LEN as u32) << (conf.width as u32)),
                );
            }

            // Update the remaining words
            if rem_words > MAX_TRANSFERS_LEN {
                self.remaining_words.set(rem_words - MAX_TRANSFERS_LEN);
            } else {
                self.remaining_words.set(0);
            }

            // If the transfer type is MemoryToMemory, the R_POWER register can have a different
            // value than 0, since the source- and destination address are incremented and it's not
            // necessary to wait for any hardware module to process or 'generate' data.
            let r_power = if tt == DmaTransferType::MemoryToMemory {
                if rem_words > MAX_TRANSFERS_LEN {
                    31 - (MAX_TRANSFERS_LEN as u32).leading_zeros()
                } else {
                    31 - (len as u32).leading_zeros()
                }
            } else {
                0
            };

            DMA_CONFIG.0[self.chan_nr].ctrl.modify(
                // Set the DMA mode since the DMA module sets it back to stop after every cycle
                DMA_CTRL::CYCLE_CTRL.val(conf.mode as u32)
                // Set the DMA cycles to the amount of remaining words
                + DMA_CTRL::N_MINUS_1.val(((rem_words - 1) % MAX_TRANSFERS_LEN) as u32)
                // Set the number of DMA-transfers after the DMA has to rearbitrate the bus
                + DMA_CTRL::R_POWER.val(r_power),
            );
        } else {
            // Disable the DMA channel since the data transfer has finished
            self.registers.enaclr.set((1 << self.chan_nr) as u32);

            // Fire the callback and return the buffer-references
            match tt {
                DmaTransferType::PeripheralToMemory => {
                    self.client.map(|cl| {
                        self.rx_buf
                            .take()
                            .map(|rx_buf| cl.transfer_done(None, Some(rx_buf), len))
                    });
                }
                DmaTransferType::MemoryToPeripheral => {
                    self.client.map(|cl| {
                        self.tx_buf
                            .take()
                            .map(|tx_buf| cl.transfer_done(Some(tx_buf), None, len))
                    });
                }
                DmaTransferType::MemoryToMemory => {
                    self.client.map(|cl| {
                        self.tx_buf.take().map(|tx_buf| {
                            self.rx_buf.take().map(move |rx_buf| {
                                cl.transfer_done(Some(tx_buf), Some(rx_buf), len)
                            })
                        })
                    });
                }
                _ => {}
            }
        }
    }

    pub fn set_client(&self, client: &'a dyn DmaClient) {
        if self.client.is_some() {
            panic!("DMA: channel {} is already in use!", self.chan_nr);
        }
        self.client.set(client);
    }

    pub fn initialize(&self, config: &DmaConfig) {
        if self.in_use.get() {
            panic!("DMA: channel {} is already in use!", self.chan_nr);
        }

        if !self.dma_is_enabled() {
            self.enable_dma();
        }

        self.in_use.set(true);
        self.config.set(*config);
        self.apply_config();
    }

    /// Start a DMA transfer where one buffer is copied into another one
    pub fn transfer_mem_to_mem(
        &self,
        src_buf: &'static mut [u8],
        dst_buf: &'static mut [u8],
        len: usize,
    ) {
        // Note: This function is currently entirely untested, since there is currently no driver
        // available where this function might be used.

        let width = self.config.get().width as u32;

        // Divide the byte-length by the width to get the number of necessary transfers
        let transfers = len >> width;

        // The pointers must point to the end of the buffer, for detailed calculation see
        // datasheet p. 646, section 11.2.4.4.
        let src_end_ptr = (&src_buf[0] as *const u8 as u32) + ((len as u32) - 1);
        let dst_end_ptr = (&dst_buf[0] as *const u8 as u32) + ((len as u32) - 1);

        self.setup_transfer(src_end_ptr, dst_end_ptr, len);

        // Set the the number of cycles after the module rearbitrates the bus.
        // The 'register-value' for this is ld(cycles), e.g. cycles = 256 -> ld(256) = 8
        // In order to get the number of cycles, just get the closest 2^n value of transfers
        let r_power = if transfers > MAX_TRANSFERS_LEN {
            31 - (MAX_TRANSFERS_LEN as u32).leading_zeros()
        } else {
            31 - (len as u32).leading_zeros()
        };

        // Set the number of cycles before a bus rearbitration
        DMA_CONFIG.0[self.chan_nr]
            .ctrl
            .modify(DMA_CTRL::R_POWER.val(r_power));

        // Store the buffers
        self.rx_buf.replace(dst_buf);
        self.tx_buf.replace(src_buf);

        // Store transfer-type
        self.transfer_type.set(DmaTransferType::MemoryToMemory);

        // Enable the DMA channel
        self.enable_dma_channel();
    }

    /// Start a DMA transfer where the contents of any register will be copied into a provided buffer
    pub fn transfer_periph_to_mem(&self, src_reg: *const (), buf: &'static mut [u8], len: usize) {
        // The pointers must point to the end of the buffer, for detailed calculation see
        // datasheet p. 646, section 11.2.4.4.
        let src_end_ptr = src_reg as u32;
        let dst_end_ptr = (&buf[0] as *const u8 as u32) + ((len as u32) - 1);

        self.setup_transfer(src_end_ptr, dst_end_ptr, len);

        // Store the buffer
        self.rx_buf.replace(buf);

        // Store transfer-type
        self.transfer_type.set(DmaTransferType::PeripheralToMemory);

        self.enable_dma_channel();
    }

    /// Start a DMA transfer where the contents of a buffer will be copied into a register
    pub fn transfer_mem_to_periph(&self, dst_reg: *const (), buf: &'static mut [u8], len: usize) {
        // The pointers must point to the end of the buffer, for detailed calculation see
        // datasheet p. 646, section 11.2.4.4.
        let src_end_ptr = (&buf[0] as *const u8 as u32) + ((len as u32) - 1);
        let dst_end_ptr = dst_reg as u32;

        self.setup_transfer(src_end_ptr, dst_end_ptr, len);

        // Store the buffer
        self.tx_buf.replace(buf);

        // Store transfer-type
        self.transfer_type.set(DmaTransferType::MemoryToPeripheral);

        self.enable_dma_channel();
    }
}

pub fn handle_interrupt(int_nr: isize) {
    if int_nr == 0 {
        // For now only use the INT0 because I don't know how to prioritize the channels in order
        // to give them 1 out of 3 'own' interrupts.
        let int = DMA_BASE.int0_srcflg.get();

        for i in 0..AVAILABLE_DMA_CHANNELS {
            let bit = (1 << i) as u32;
            if (bit & int) > 0 {
                // Clear interrupt-bit
                DMA_BASE.int0_clrflg.set(bit);

                // This access must be unsafe because DMA_CHANNELS is a global mutable variable
                unsafe {
                    DMA_CHANNELS[i].handle_interrupt();
                }
            }
        }
    } else if int_nr < 0 {
        panic!("DMA: error interrupt");
    } else {
        panic!("DMA: unhandled interrupt-nr: {}", int_nr);
    }
}
