//! Flash Controller (FLCTL)

use kernel::common::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

pub static mut FLCTL: FlCtl = FlCtl::new();

const FLCTL_BASE: StaticRef<FlCtlRegisters> =
    unsafe { StaticRef::new(0x4001_1000u32 as *const FlCtlRegisters) };

register_structs! {
    /// FLCTL
    FlCtlRegisters {
        /// Power Status Register
        (0x000 => power_stat: ReadOnly<u32, FLCTL_POWER_STAT::Register>),
        (0x004 => _reserved0),
        /// Bank0 Read Control Register
        (0x010 => bank0_rdctl: ReadWrite<u32, FLCTL_BANK0_RDCTL::Register>),
        /// Bank1 Read Control Register
        (0x014 => bank1_rdctl: ReadWrite<u32, FLCTL_BANK1_RDCTL::Register>),
        (0x018 => _reserved1),
        /// Read Burst/Compare Control and Status Register
        (0x020 => rdbrst_ctlstat: ReadWrite<u32, FLCTL_RDBRST_CTLSTAT::Register>),
        /// Read Burst/Compare Start Address Register
        (0x024 => rdbrst_startaddr: ReadWrite<u32>),
        /// Read Burst/Compare Length Register
        (0x028 => rdbrst_len: ReadWrite<u32>),
        (0x02C => _reserved2),
        /// Read Burst/Compare Fail Address Register
        (0x03C => rdbrst_failaddr: ReadWrite<u32>),
        /// Read Burst/Compare Fail Count Register
        (0x040 => rdbrst_failcnt: ReadWrite<u32>),
        (0x044 => _reserved3),
        /// Program Control and Status Register
        (0x050 => prg_ctlstat: ReadWrite<u32, FLCTL_PRG_CTLSTAT::Register>),
        /// Program Burst Control and Status Register
        (0x054 => prgbrst_ctlstat: ReadWrite<u32, FLCTL_PRGBRST_CTLSTAT::Register>),
        /// Program Burst Start Address Register
        (0x058 => prgbrst_startaddr: ReadWrite<u32>),
        (0x05C => _reserved4),
        /// Program Burst Data0 Register0
        (0x060 => prgbrst_data0_0: ReadWrite<u32>),
        /// Program Burst Data0 Register1
        (0x064 => prgbrst_data0_1: ReadWrite<u32>),
        /// Program Burst Data0 Register2
        (0x068 => prgbrst_data0_2: ReadWrite<u32>),
        /// Program Burst Data0 Register3
        (0x06C => prgbrst_data0_3: ReadWrite<u32>),
        /// Program Burst Data1 Register0
        (0x070 => prgbrst_data1_0: ReadWrite<u32>),
        /// Program Burst Data1 Register1
        (0x074 => prgbrst_data1_1: ReadWrite<u32>),
        /// Program Burst Data1 Register2
        (0x078 => prgbrst_data1_2: ReadWrite<u32>),
        /// Program Burst Data1 Register3
        (0x07C => prgbrst_data1_3: ReadWrite<u32>),
        /// Program Burst Data2 Register0
        (0x080 => prgbrst_data2_0: ReadWrite<u32>),
        /// Program Burst Data2 Register1
        (0x084 => prgbrst_data2_1: ReadWrite<u32>),
        /// Program Burst Data2 Register2
        (0x088 => prgbrst_data2_2: ReadWrite<u32>),
        /// Program Burst Data2 Register3
        (0x08C => prgbrst_data2_3: ReadWrite<u32>),
        /// Program Burst Data3 Register0
        (0x090 => prgbrst_data3_0: ReadWrite<u32>),
        /// Program Burst Data3 Register1
        (0x094 => prgbrst_data3_1: ReadWrite<u32>),
        /// Program Burst Data3 Register2
        (0x098 => prgbrst_data3_2: ReadWrite<u32>),
        /// Program Burst Data3 Register3
        (0x09C => prgbrst_data3_3: ReadWrite<u32>),
        /// Erase Control and Status Register
        (0x0A0 => erase_ctlstat: ReadWrite<u32, FLCTL_ERASE_CTLSTAT::Register>),
        /// Erase Sector Address Register
        (0x0A4 => erase_sectaddr: ReadWrite<u32>),
        (0x0A8 => _reserved5),
        /// Information Memory Bank0 Write/Erase Protection Register
        (0x0B0 => bank0_info_weprot: ReadWrite<u32, FLCTL_BANK0_INFO_WEPROT::Register>),
        /// Main Memory Bank0 Write/Erase Protection Register
        (0x0B4 => bank0_main_weprot: ReadWrite<u32, FLCTL_BANK0_MAIN_WEPROT::Register>),
        (0x0B8 => _reserved6),
        /// Information Memory Bank1 Write/Erase Protection Register
        (0x0C0 => bank1_info_weprot: ReadWrite<u32, FLCTL_BANK1_INFO_WEPROT::Register>),
        /// Main Memory Bank1 Write/Erase Protection Register
        (0x0C4 => bank1_main_weprot: ReadWrite<u32, FLCTL_BANK1_MAIN_WEPROT::Register>),
        (0x0C8 => _reserved7),
        /// Benchmark Control and Status Register
        (0x0D0 => bmrk_ctlstat: ReadWrite<u32, FLCTL_BMRK_CTLSTAT::Register>),
        /// Benchmark Instruction Fetch Count Register
        (0x0D4 => bmrk_ifetch: ReadWrite<u32>),
        /// Benchmark Data Read Count Register
        (0x0D8 => bmrk_dread: ReadWrite<u32>),
        /// Benchmark Count Compare Register
        (0x0DC => bmrk_cmp: ReadWrite<u32>),
        (0x0E0 => _reserved8),
        /// Interrupt Flag Register
        (0x0F0 => ifg: ReadWrite<u32, FLCTL_IFG::Register>),
        /// Interrupt Enable Register
        (0x0F4 => ie: ReadWrite<u32, FLCTL_IE::Register>),
        /// Clear Interrupt Flag Register
        (0x0F8 => clrifg: ReadWrite<u32, FLCTL_CLRIFG::Register>),
        /// Set Interrupt Flag Register
        (0x0FC => setifg: ReadWrite<u32, FLCTL_SETIFG::Register>),
        /// Read Timing Control Register
        (0x100 => read_timctl: ReadOnly<u32, FLCTL_READ_TIMCTL::Register>),
        /// Read Margin Timing Control Register
        (0x104 => readmargin_timctl: ReadOnly<u32>),
        /// Program Verify Timing Control Register
        (0x108 => prgver_timctl: ReadOnly<u32, FLCTL_PRGVER_TIMCTL::Register>),
        /// Erase Verify Timing Control Register
        (0x10C => ersver_timctl: ReadOnly<u32>),
        /// Leakage Verify Timing Control Register
        (0x110 => lkgver_timctl: ReadOnly<u32>),
        /// Program Timing Control Register
        (0x114 => program_timctl: ReadOnly<u32, FLCTL_PROGRAM_TIMCTL::Register>),
        /// Erase Timing Control Register
        (0x118 => erase_timctl: ReadOnly<u32, FLCTL_ERASE_TIMCTL::Register>),
        /// Mass Erase Timing Control Register
        (0x11C => masserase_timctl: ReadOnly<u32, FLCTL_MASSERASE_TIMCTL::Register>),
        /// Burst Program Timing Control Register
        (0x120 => burstprg_timctl: ReadOnly<u32>),
        (0x124 => @END),
    }
}

register_bitfields![u32,
    FLCTL_POWER_STAT [
        /// Flash power status
        PSTAT OFFSET(0) NUMBITS(3) [
            /// Flash IP in power-down mode
            FlashIPInPowerDownMode = 0,
            /// Flash IP Vdd domain power-up in progress
            FlashIPVddDomainPowerUpInProgress = 1,
            /// PSS LDO_GOOD, IREF_OK and VREF_OK check in progress
            PSSLDO_GOODIREF_OKAndVREF_OKCheckInProgress = 2,
            /// Flash IP SAFE_LV check in progress
            FlashIPSAFE_LVCheckInProgress = 3,
            /// Flash IP Active
            FlashIPActive = 4,
            /// Flash IP Active in Low-Frequency Active and Low-Frequency LPM0 modes.
            FlashIPActiveInLowFrequencyActiveAndLowFrequencyLPM0Modes = 5,
            /// Flash IP in Standby mode
            FlashIPInStandbyMode = 6,
            /// Flash IP in Current mirror boost state
            FlashIPInCurrentMirrorBoostState = 7
        ],
        /// PSS FLDO GOOD status
        LDOSTAT OFFSET(3) NUMBITS(1) [
            /// FLDO not GOOD
            FLDONotGOOD = 0,
            /// FLDO GOOD
            FLDOGOOD = 1
        ],
        /// PSS VREF stable status
        VREFSTAT OFFSET(4) NUMBITS(1) [
            /// Flash LDO not stable
            FlashLDONotStable = 0,
            /// Flash LDO stable
            FlashLDOStable = 1
        ],
        /// PSS IREF stable status
        IREFSTAT OFFSET(5) NUMBITS(1) [
            /// IREF not stable
            IREFNotStable = 0,
            /// IREF stable
            IREFStable = 1
        ],
        /// PSS trim done status
        TRIMSTAT OFFSET(6) NUMBITS(1) [
            /// PSS trim not complete
            PSSTrimNotComplete = 0,
            /// PSS trim complete
            PSSTrimComplete = 1
        ],
        /// Indicates if Flash is being accessed in 2T mode
        RD_2T OFFSET(7) NUMBITS(1) [
            /// Flash reads are in 1T mode
            FlashReadsAreIn1TMode = 0,
            /// Flash reads are in 2T mode
            FlashReadsAreIn2TMode = 1
        ]
    ],
    FLCTL_BANK0_RDCTL [
        /// Flash read mode control setting for Bank 0
        RD_MODE OFFSET(0) NUMBITS(4) [
            /// Normal read mode
            NormalReadMode = 0,
            /// Read Margin 0
            ReadMargin0 = 1,
            /// Read Margin 1
            ReadMargin1 = 2,
            /// Program Verify
            ProgramVerify = 3,
            /// Erase Verify
            EraseVerify = 4,
            /// Leakage Verify
            LeakageVerify = 5,
            /// Read Margin 0B
            ReadMargin0B = 9,
            /// Read Margin 1B
            ReadMargin1B = 10
        ],
        /// Enables read buffering feature for instruction fetches to this Bank
        BUFI OFFSET(4) NUMBITS(1) [],
        /// Enables read buffering feature for data reads to this Bank
        BUFD OFFSET(5) NUMBITS(1) [],
        /// Number of wait states for read
        WAIT OFFSET(12) NUMBITS(4) [
            /// 0 wait states
            _0WaitStates = 0,
            /// 1 wait states
            _1WaitStates = 1,
            /// 2 wait states
            _2WaitStates = 2,
            /// 3 wait states
            _3WaitStates = 3,
            /// 4 wait states
            _4WaitStates = 4,
            /// 5 wait states
            _5WaitStates = 5,
            /// 6 wait states
            _6WaitStates = 6,
            /// 7 wait states
            _7WaitStates = 7,
            /// 8 wait states
            _8WaitStates = 8,
            /// 9 wait states
            _9WaitStates = 9,
            /// 10 wait states
            _10WaitStates = 10,
            /// 11 wait states
            _11WaitStates = 11,
            /// 12 wait states
            _12WaitStates = 12,
            /// 13 wait states
            _13WaitStates = 13,
            /// 14 wait states
            _14WaitStates = 14,
            /// 15 wait states
            _15WaitStates = 15
        ],
        /// Read mode
        RD_MODE_STATUS OFFSET(16) NUMBITS(4) [
            /// Normal read mode
            NormalReadMode = 0,
            /// Read Margin 0
            ReadMargin0 = 1,
            /// Read Margin 1
            ReadMargin1 = 2,
            /// Program Verify
            ProgramVerify = 3,
            /// Erase Verify
            EraseVerify = 4,
            /// Leakage Verify
            LeakageVerify = 5,
            /// Read Margin 0B
            ReadMargin0B = 9,
            /// Read Margin 1B
            ReadMargin1B = 10
        ]
    ],
    FLCTL_BANK1_RDCTL [
        /// Flash read mode control setting for Bank 0
        RD_MODE OFFSET(0) NUMBITS(4) [
            /// Normal read mode
            NormalReadMode = 0,
            /// Read Margin 0
            ReadMargin0 = 1,
            /// Read Margin 1
            ReadMargin1 = 2,
            /// Program Verify
            ProgramVerify = 3,
            /// Erase Verify
            EraseVerify = 4,
            /// Leakage Verify
            LeakageVerify = 5,
            /// Read Margin 0B
            ReadMargin0B = 9,
            /// Read Margin 1B
            ReadMargin1B = 10
        ],
        /// Enables read buffering feature for instruction fetches to this Bank
        BUFI OFFSET(4) NUMBITS(1) [],
        /// Enables read buffering feature for data reads to this Bank
        BUFD OFFSET(5) NUMBITS(1) [],
        /// Read mode
        RD_MODE_STATUS OFFSET(16) NUMBITS(4) [
            /// Normal read mode
            NormalReadMode = 0,
            /// Read Margin 0
            ReadMargin0 = 1,
            /// Read Margin 1
            ReadMargin1 = 2,
            /// Program Verify
            ProgramVerify = 3,
            /// Erase Verify
            EraseVerify = 4,
            /// Leakage Verify
            LeakageVerify = 5,
            /// Read Margin 0B
            ReadMargin0B = 9,
            /// Read Margin 1B
            ReadMargin1B = 10
        ],
        /// Number of wait states for read
        WAIT OFFSET(12) NUMBITS(4) [
            /// 0 wait states
            _0WaitStates = 0,
            /// 1 wait states
            _1WaitStates = 1,
            /// 2 wait states
            _2WaitStates = 2,
            /// 3 wait states
            _3WaitStates = 3,
            /// 4 wait states
            _4WaitStates = 4,
            /// 5 wait states
            _5WaitStates = 5,
            /// 6 wait states
            _6WaitStates = 6,
            /// 7 wait states
            _7WaitStates = 7,
            /// 8 wait states
            _8WaitStates = 8,
            /// 9 wait states
            _9WaitStates = 9,
            /// 10 wait states
            _10WaitStates = 10,
            /// 11 wait states
            _11WaitStates = 11,
            /// 12 wait states
            _12WaitStates = 12,
            /// 13 wait states
            _13WaitStates = 13,
            /// 14 wait states
            _14WaitStates = 14,
            /// 15 wait states
            _15WaitStates = 15
        ]
    ],
    FLCTL_RDBRST_CTLSTAT [
        /// Start of burst/compare operation
        START OFFSET(0) NUMBITS(1) [],
        /// Type of memory that burst is carried out on
        MEM_TYPE OFFSET(1) NUMBITS(2) [
            /// Main Memory
            MainMemory = 0,
            /// Information Memory
            InformationMemory = 1,
            /// Engineering Memory
            EngineeringMemory = 3
        ],
        /// Terminate burst/compare operation
        STOP_FAIL OFFSET(3) NUMBITS(1) [],
        /// Data pattern used for comparison against memory read data
        DATA_CMP OFFSET(4) NUMBITS(1) [
            /// 0000_0000_0000_0000_0000_0000_0000_0000
            _0000_0000_0000_0000_0000_0000_0000_0000 = 0,
            /// FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF
            FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF = 1
        ],
        /// Enable comparison against test data compare registers
        TEST_EN OFFSET(6) NUMBITS(1) [],
        /// Status of Burst/Compare operation
        BRST_STAT OFFSET(16) NUMBITS(2) [
            /// Idle
            Idle = 0,
            /// Burst/Compare START bit written, but operation pending
            BurstCompareSTARTBitWrittenButOperationPending = 1,
            /// Burst/Compare in progress
            BurstCompareInProgress = 2,
            /// Burst complete (status of completed burst remains in this state unless explicitl
            BRST_STAT_3 = 3
        ],
        /// Burst/Compare Operation encountered atleast one data
        CMP_ERR OFFSET(18) NUMBITS(1) [],
        /// Burst/Compare Operation was terminated due to access to
        ADDR_ERR OFFSET(19) NUMBITS(1) [],
        /// Clear status bits 19-16 of this register
        CLR_STAT OFFSET(23) NUMBITS(1) []
    ],
    FLCTL_PRG_CTLSTAT [
        /// Master control for all word program operations
        ENABLE OFFSET(0) NUMBITS(1) [
            /// Word program operation disabled
            WordProgramOperationDisabled = 0,
            /// Word program operation enabled
            WordProgramOperationEnabled = 1
        ],
        /// Write mode
        MODE OFFSET(1) NUMBITS(1) [
            /// Write immediate mode. Starts program operation immediately on each write to the
            WriteImmediateModeStartsProgramOperationImmediatelyOnEachWriteToTheFlash = 0,
            /// Full word write mode. Flash controller collates data over multiple writes to com
            MODE_1 = 1
        ],
        /// Controls automatic pre program verify operations
        VER_PRE OFFSET(2) NUMBITS(1) [
            /// No pre program verification
            NoPreProgramVerification = 0,
            /// Pre verify feature automatically invoked for each write operation (irrespective
            PreVerifyFeatureAutomaticallyInvokedForEachWriteOperationIrrespectiveOfTheMode = 1
        ],
        /// Controls automatic post program verify operations
        VER_PST OFFSET(3) NUMBITS(1) [
            /// No post program verification
            NoPostProgramVerification = 0,
            /// Post verify feature automatically invoked for each write operation (irrespective
            PostVerifyFeatureAutomaticallyInvokedForEachWriteOperationIrrespectiveOfTheMode = 1
        ],
        /// Status of program operations in the Flash memory
        STATUS OFFSET(16) NUMBITS(2) [
            /// Idle (no program operation currently active)
            IdleNoProgramOperationCurrentlyActive = 0,
            /// Single word program operation triggered, but pending
            SingleWordProgramOperationTriggeredButPending = 1,
            /// Single word program in progress
            SingleWordProgramInProgress = 2
        ],
        /// Bank active
        BNK_ACT OFFSET(18) NUMBITS(1) [
            /// Word in Bank0 being programmed
            WordInBank0BeingProgrammed = 0,
            /// Word in Bank1 being programmed
            WordInBank1BeingProgrammed = 1
        ]
    ],
    FLCTL_PRGBRST_CTLSTAT [
        /// Trigger start of burst program operation
        START OFFSET(0) NUMBITS(1) [],
        /// Type of memory that burst program is carried out on
        TYPE OFFSET(1) NUMBITS(2) [
            /// Main Memory
            MainMemory = 0,
            /// Information Memory
            InformationMemory = 1,
            /// Engineering Memory
            EngineeringMemory = 3
        ],
        /// Length of burst
        LEN OFFSET(3) NUMBITS(3) [
            /// No burst operation
            NoBurstOperation = 0,
            /// 1 word burst of 128 bits, starting with address in the FLCTL_PRGBRST_STARTADDR R
            _1WordBurstOf128BitsStartingWithAddressInTheFLCTL_PRGBRST_STARTADDRRegister = 1,
            /// 2*128 bits burst write, starting with address in the FLCTL_PRGBRST_STARTADDR Reg
            _2128BitsBurstWriteStartingWithAddressInTheFLCTL_PRGBRST_STARTADDRRegister = 2,
            /// 3*128 bits burst write, starting with address in the FLCTL_PRGBRST_STARTADDR Reg
            _3128BitsBurstWriteStartingWithAddressInTheFLCTL_PRGBRST_STARTADDRRegister = 3,
            /// 4*128 bits burst write, starting with address in the FLCTL_PRGBRST_STARTADDR Reg
            _4128BitsBurstWriteStartingWithAddressInTheFLCTL_PRGBRST_STARTADDRRegister = 4
        ],
        /// Auto-Verify operation before the Burst Program
        AUTO_PRE OFFSET(6) NUMBITS(1) [
            /// No program verify operations carried out
            NoProgramVerifyOperationsCarriedOut = 0,
            /// Causes an automatic Burst Program Verify after the Burst Program Operation
            CausesAnAutomaticBurstProgramVerifyAfterTheBurstProgramOperation = 1
        ],
        /// Auto-Verify operation after the Burst Program
        AUTO_PST OFFSET(7) NUMBITS(1) [
            /// No program verify operations carried out
            NoProgramVerifyOperationsCarriedOut = 0,
            /// Causes an automatic Burst Program Verify before the Burst Program Operation
            CausesAnAutomaticBurstProgramVerifyBeforeTheBurstProgramOperation = 1
        ],
        /// Status of a Burst Operation
        BURST_STATUS OFFSET(16) NUMBITS(3) [
            /// Idle (Burst not active)
            IdleBurstNotActive = 0,
            /// Burst program started but pending
            BurstProgramStartedButPending = 1,
            /// Burst active, with 1st 128 bit word being written into Flash
            BurstActiveWith1st128BitWordBeingWrittenIntoFlash = 2,
            /// Burst active, with 2nd 128 bit word being written into Flash
            BurstActiveWith2nd128BitWordBeingWrittenIntoFlash = 3,
            /// Burst active, with 3rd 128 bit word being written into Flash
            BurstActiveWith3rd128BitWordBeingWrittenIntoFlash = 4,
            /// Burst active, with 4th 128 bit word being written into Flash
            BurstActiveWith4th128BitWordBeingWrittenIntoFlash = 5,
            /// Burst Complete (status of completed burst remains in this state unless explicitl
            BURST_STATUS_7 = 7
        ],
        /// Burst Operation encountered preprogram auto-verify errors
        PRE_ERR OFFSET(19) NUMBITS(1) [],
        /// Burst Operation encountered postprogram auto-verify errors
        PST_ERR OFFSET(20) NUMBITS(1) [],
        /// Burst Operation was terminated due to attempted program of reserved memory
        ADDR_ERR OFFSET(21) NUMBITS(1) [],
        /// Clear status bits 21-16 of this register
        CLR_STAT OFFSET(23) NUMBITS(1) []
    ],
    FLCTL_ERASE_CTLSTAT [
        /// Start of Erase operation
        START OFFSET(0) NUMBITS(1) [],
        /// Erase mode selected by application
        MODE OFFSET(1) NUMBITS(1) [
            /// Sector Erase (controlled by FLTCTL_ERASE_SECTADDR)
            SectorEraseControlledByFLTCTL_ERASE_SECTADDR = 0,
            /// Mass Erase (includes all Main and Information memory sectors that don't have cor
            MODE_1 = 1
        ],
        /// Type of memory that erase operation is carried out on
        TYPE OFFSET(2) NUMBITS(2) [
            /// Main Memory
            MainMemory = 0,
            /// Information Memory
            InformationMemory = 1,
            /// Engineering Memory
            EngineeringMemory = 3
        ],
        /// Status of erase operations in the Flash memory
        STATUS OFFSET(16) NUMBITS(2) [
            /// Idle (no program operation currently active)
            IdleNoProgramOperationCurrentlyActive = 0,
            /// Erase operation triggered to START but pending
            EraseOperationTriggeredToSTARTButPending = 1,
            /// Erase operation in progress
            EraseOperationInProgress = 2,
            /// Erase operation completed (status of completed erase remains in this state unles
            STATUS_3 = 3
        ],
        /// Erase Operation was terminated due to attempted erase of reserved memory address
        ADDR_ERR OFFSET(18) NUMBITS(1) [],
        /// Clear status bits 18-16 of this register
        CLR_STAT OFFSET(19) NUMBITS(1) []
    ],
    FLCTL_BANK0_INFO_WEPROT [
        /// Protects Sector 0 from program or erase
        PROT0 OFFSET(0) NUMBITS(1) [],
        /// Protects Sector 1 from program or erase
        PROT1 OFFSET(1) NUMBITS(1) []
    ],
    FLCTL_BANK0_MAIN_WEPROT [
        /// Protects Sector 0 from program or erase
        PROT0 OFFSET(0) NUMBITS(1) [],
        /// Protects Sector 1 from program or erase
        PROT1 OFFSET(1) NUMBITS(1) [],
        /// Protects Sector 2 from program or erase
        PROT2 OFFSET(2) NUMBITS(1) [],
        /// Protects Sector 3 from program or erase
        PROT3 OFFSET(3) NUMBITS(1) [],
        /// Protects Sector 4 from program or erase
        PROT4 OFFSET(4) NUMBITS(1) [],
        /// Protects Sector 5 from program or erase
        PROT5 OFFSET(5) NUMBITS(1) [],
        /// Protects Sector 6 from program or erase
        PROT6 OFFSET(6) NUMBITS(1) [],
        /// Protects Sector 7 from program or erase
        PROT7 OFFSET(7) NUMBITS(1) [],
        /// Protects Sector 8 from program or erase
        PROT8 OFFSET(8) NUMBITS(1) [],
        /// Protects Sector 9 from program or erase
        PROT9 OFFSET(9) NUMBITS(1) [],
        /// Protects Sector 10 from program or erase
        PROT10 OFFSET(10) NUMBITS(1) [],
        /// Protects Sector 11 from program or erase
        PROT11 OFFSET(11) NUMBITS(1) [],
        /// Protects Sector 12 from program or erase
        PROT12 OFFSET(12) NUMBITS(1) [],
        /// Protects Sector 13 from program or erase
        PROT13 OFFSET(13) NUMBITS(1) [],
        /// Protects Sector 14 from program or erase
        PROT14 OFFSET(14) NUMBITS(1) [],
        /// Protects Sector 15 from program or erase
        PROT15 OFFSET(15) NUMBITS(1) [],
        /// Protects Sector 16 from program or erase
        PROT16 OFFSET(16) NUMBITS(1) [],
        /// Protects Sector 17 from program or erase
        PROT17 OFFSET(17) NUMBITS(1) [],
        /// Protects Sector 18 from program or erase
        PROT18 OFFSET(18) NUMBITS(1) [],
        /// Protects Sector 19 from program or erase
        PROT19 OFFSET(19) NUMBITS(1) [],
        /// Protects Sector 20 from program or erase
        PROT20 OFFSET(20) NUMBITS(1) [],
        /// Protects Sector 21 from program or erase
        PROT21 OFFSET(21) NUMBITS(1) [],
        /// Protects Sector 22 from program or erase
        PROT22 OFFSET(22) NUMBITS(1) [],
        /// Protects Sector 23 from program or erase
        PROT23 OFFSET(23) NUMBITS(1) [],
        /// Protects Sector 24 from program or erase
        PROT24 OFFSET(24) NUMBITS(1) [],
        /// Protects Sector 25 from program or erase
        PROT25 OFFSET(25) NUMBITS(1) [],
        /// Protects Sector 26 from program or erase
        PROT26 OFFSET(26) NUMBITS(1) [],
        /// Protects Sector 27 from program or erase
        PROT27 OFFSET(27) NUMBITS(1) [],
        /// Protects Sector 28 from program or erase
        PROT28 OFFSET(28) NUMBITS(1) [],
        /// Protects Sector 29 from program or erase
        PROT29 OFFSET(29) NUMBITS(1) [],
        /// Protects Sector 30 from program or erase
        PROT30 OFFSET(30) NUMBITS(1) [],
        /// Protects Sector 31 from program or erase
        PROT31 OFFSET(31) NUMBITS(1) []
    ],
    FLCTL_BANK1_INFO_WEPROT [
        /// Protects Sector 0 from program or erase operations
        PROT0 OFFSET(0) NUMBITS(1) [],
        /// Protects Sector 1 from program or erase operations
        PROT1 OFFSET(1) NUMBITS(1) []
    ],
    FLCTL_BANK1_MAIN_WEPROT [
        /// Protects Sector 0 from program or erase operations
        PROT0 OFFSET(0) NUMBITS(1) [],
        /// Protects Sector 1 from program or erase operations
        PROT1 OFFSET(1) NUMBITS(1) [],
        /// Protects Sector 2 from program or erase operations
        PROT2 OFFSET(2) NUMBITS(1) [],
        /// Protects Sector 3 from program or erase operations
        PROT3 OFFSET(3) NUMBITS(1) [],
        /// Protects Sector 4 from program or erase operations
        PROT4 OFFSET(4) NUMBITS(1) [],
        /// Protects Sector 5 from program or erase operations
        PROT5 OFFSET(5) NUMBITS(1) [],
        /// Protects Sector 6 from program or erase operations
        PROT6 OFFSET(6) NUMBITS(1) [],
        /// Protects Sector 7 from program or erase operations
        PROT7 OFFSET(7) NUMBITS(1) [],
        /// Protects Sector 8 from program or erase operations
        PROT8 OFFSET(8) NUMBITS(1) [],
        /// Protects Sector 9 from program or erase operations
        PROT9 OFFSET(9) NUMBITS(1) [],
        /// Protects Sector 10 from program or erase operations
        PROT10 OFFSET(10) NUMBITS(1) [],
        /// Protects Sector 11 from program or erase operations
        PROT11 OFFSET(11) NUMBITS(1) [],
        /// Protects Sector 12 from program or erase operations
        PROT12 OFFSET(12) NUMBITS(1) [],
        /// Protects Sector 13 from program or erase operations
        PROT13 OFFSET(13) NUMBITS(1) [],
        /// Protects Sector 14 from program or erase operations
        PROT14 OFFSET(14) NUMBITS(1) [],
        /// Protects Sector 15 from program or erase operations
        PROT15 OFFSET(15) NUMBITS(1) [],
        /// Protects Sector 16 from program or erase operations
        PROT16 OFFSET(16) NUMBITS(1) [],
        /// Protects Sector 17 from program or erase operations
        PROT17 OFFSET(17) NUMBITS(1) [],
        /// Protects Sector 18 from program or erase operations
        PROT18 OFFSET(18) NUMBITS(1) [],
        /// Protects Sector 19 from program or erase operations
        PROT19 OFFSET(19) NUMBITS(1) [],
        /// Protects Sector 20 from program or erase operations
        PROT20 OFFSET(20) NUMBITS(1) [],
        /// Protects Sector 21 from program or erase operations
        PROT21 OFFSET(21) NUMBITS(1) [],
        /// Protects Sector 22 from program or erase operations
        PROT22 OFFSET(22) NUMBITS(1) [],
        /// Protects Sector 23 from program or erase operations
        PROT23 OFFSET(23) NUMBITS(1) [],
        /// Protects Sector 24 from program or erase operations
        PROT24 OFFSET(24) NUMBITS(1) [],
        /// Protects Sector 25 from program or erase operations
        PROT25 OFFSET(25) NUMBITS(1) [],
        /// Protects Sector 26 from program or erase operations
        PROT26 OFFSET(26) NUMBITS(1) [],
        /// Protects Sector 27 from program or erase operations
        PROT27 OFFSET(27) NUMBITS(1) [],
        /// Protects Sector 28 from program or erase operations
        PROT28 OFFSET(28) NUMBITS(1) [],
        /// Protects Sector 29 from program or erase operations
        PROT29 OFFSET(29) NUMBITS(1) [],
        /// Protects Sector 30 from program or erase operations
        PROT30 OFFSET(30) NUMBITS(1) [],
        /// Protects Sector 31 from program or erase operations
        PROT31 OFFSET(31) NUMBITS(1) []
    ],
    FLCTL_BMRK_CTLSTAT [
        /// When 1, increments the Instruction Benchmark count register on each instruction
        I_BMRK OFFSET(0) NUMBITS(1) [],
        /// When 1, increments the Data Benchmark count register on each data read access to
        D_BMRK OFFSET(1) NUMBITS(1) [],
        /// When 1, enables comparison of the Instruction or Data Benchmark Registers agains
        CMP_EN OFFSET(2) NUMBITS(1) [],
        /// Selects which benchmark register should be compared against the threshold
        CMP_SEL OFFSET(3) NUMBITS(1) [
            /// Compares the Instruction Benchmark Register against the threshold value
            ComparesTheInstructionBenchmarkRegisterAgainstTheThresholdValue = 0,
            /// Compares the Data Benchmark Register against the threshold value
            ComparesTheDataBenchmarkRegisterAgainstTheThresholdValue = 1
        ]
    ],
    FLCTL_IFG [
        /// If set to 1, indicates that the Read Burst/Compare operation is complete
        RDBRST OFFSET(0) NUMBITS(1) [],
        /// If set to 1, indicates that the pre-program verify operation has detected an err
        AVPRE OFFSET(1) NUMBITS(1) [],
        /// If set to 1, indicates that the post-program verify operation has failed compari
        AVPST OFFSET(2) NUMBITS(1) [],
        /// If set to 1, indicates that a word Program operation is complete
        PRG OFFSET(3) NUMBITS(1) [],
        /// If set to 1, indicates that the configured Burst Program operation is complete
        PRGB OFFSET(4) NUMBITS(1) [],
        /// If set to 1, indicates that the Erase operation is complete
        ERASE OFFSET(5) NUMBITS(1) [],
        /// If set to 1, indicates that a Benchmark Compare match occurred
        BMRK OFFSET(8) NUMBITS(1) [],
        /// If set to 1, indicates a word composition error in full word write mode (possibl
        PRG_ERR OFFSET(9) NUMBITS(1) []
    ],
    FLCTL_IE [
        /// If set to 1, enables the Controller to generate an interrupt based on the corres
        RDBRST OFFSET(0) NUMBITS(1) [],
        /// If set to 1, enables the Controller to generate an interrupt based on the corres
        AVPRE OFFSET(1) NUMBITS(1) [],
        /// If set to 1, enables the Controller to generate an interrupt based on the corres
        AVPST OFFSET(2) NUMBITS(1) [],
        /// If set to 1, enables the Controller to generate an interrupt based on the corres
        PRG OFFSET(3) NUMBITS(1) [],
        /// If set to 1, enables the Controller to generate an interrupt based on the corres
        PRGB OFFSET(4) NUMBITS(1) [],
        /// If set to 1, enables the Controller to generate an interrupt based on the corres
        ERASE OFFSET(5) NUMBITS(1) [],
        /// If set to 1, enables the Controller to generate an interrupt based on the corres
        BMRK OFFSET(8) NUMBITS(1) [],
        /// If set to 1, enables the Controller to generate an interrupt based on the corres
        PRG_ERR OFFSET(9) NUMBITS(1) []
    ],
    FLCTL_CLRIFG [
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        RDBRST OFFSET(0) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        AVPRE OFFSET(1) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        AVPST OFFSET(2) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        PRG OFFSET(3) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        PRGB OFFSET(4) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        ERASE OFFSET(5) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        BMRK OFFSET(8) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        PRG_ERR OFFSET(9) NUMBITS(1) []
    ],
    FLCTL_SETIFG [
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        RDBRST OFFSET(0) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        AVPRE OFFSET(1) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        AVPST OFFSET(2) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        PRG OFFSET(3) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        PRGB OFFSET(4) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        ERASE OFFSET(5) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        BMRK OFFSET(8) NUMBITS(1) [],
        /// Write 1 clears the corresponding interrupt flag bit in the FLCTL_IFG
        PRG_ERR OFFSET(9) NUMBITS(1) []
    ],
    FLCTL_READ_TIMCTL [
        /// Configures the length of the Setup phase for this operation
        SETUP OFFSET(0) NUMBITS(8) [],
        /// Length of the IREF_BOOST1 signal of the IP
        IREF_BOOST1 OFFSET(12) NUMBITS(4) [],
        /// Length of the Setup time into read mode when the device is recovering from one o
        SETUP_LONG OFFSET(16) NUMBITS(8) []
    ],
    FLCTL_PRGVER_TIMCTL [
        /// Length of the Setup phase for this operation
        SETUP OFFSET(0) NUMBITS(8) [],
        /// Length of the Active phase for this operation
        ACTIVE OFFSET(8) NUMBITS(4) [],
        /// Length of the Hold phase for this operation
        HOLD OFFSET(12) NUMBITS(4) []
    ],
    FLCTL_PROGRAM_TIMCTL [
        /// Length of the Setup phase for this operation
        SETUP OFFSET(0) NUMBITS(8) [],
        /// Length of the Active phase for this operation
        ACTIVE OFFSET(8) NUMBITS(20) [],
        /// Length of the Hold phase for this operation
        HOLD OFFSET(28) NUMBITS(4) []
    ],
    FLCTL_ERASE_TIMCTL [
        /// Length of the Setup phase for this operation
        SETUP OFFSET(0) NUMBITS(8) [],
        /// Length of the Active phase for this operation
        ACTIVE OFFSET(8) NUMBITS(20) [],
        /// Length of the Hold phase for this operation
        HOLD OFFSET(28) NUMBITS(4) []
    ],
    FLCTL_MASSERASE_TIMCTL [
        /// Length of the time for which LDO Boost Signal is kept active
        BOOST_ACTIVE OFFSET(0) NUMBITS(8) [],
        /// Length for which Flash deactivates the LDO Boost signal before processing any ne
        BOOST_HOLD OFFSET(8) NUMBITS(8) []
    ]
];

/// If the clock runs with a higher frequency than the flash is able to operate, it's possible to
/// configure a certain amount of wait-states which stall the CPU in order to access the data within
/// the flash reliable. For a detailed description see datasheet page 458 section 9.2.2.1.
#[repr(u32)]
#[derive(Copy, Clone)]
pub enum WaitStates {
    _0 = 0,
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    _11 = 11,
    _12 = 12,
    _13 = 13,
    _14 = 14,
    _15 = 15,
}

pub struct FlCtl {
    registers: StaticRef<FlCtlRegisters>,
}

impl FlCtl {
    const fn new() -> FlCtl {
        FlCtl {
            registers: FLCTL_BASE,
        }
    }

    pub fn set_waitstates(&self, ws: WaitStates) {
        self.registers
            .bank0_rdctl
            .modify(FLCTL_BANK0_RDCTL::WAIT.val(ws as u32));
        self.registers
            .bank1_rdctl
            .modify(FLCTL_BANK1_RDCTL::WAIT.val(ws as u32));
    }

    pub fn set_buffering(&self, enable: bool) {
        self.registers.bank0_rdctl.modify(
            FLCTL_BANK0_RDCTL::BUFD.val(enable as u32) + FLCTL_BANK0_RDCTL::BUFI.val(enable as u32),
        );
        self.registers.bank1_rdctl.modify(
            FLCTL_BANK1_RDCTL::BUFD.val(enable as u32) + FLCTL_BANK1_RDCTL::BUFI.val(enable as u32),
        );
    }
}
