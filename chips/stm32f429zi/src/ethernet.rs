use kernel::utilities::StaticRef;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::ErrorCode;
use kernel::debug;
use kernel::utilities::cells::OptionalCell;

register_structs! {
    /// Ethernet: media access control
/// (MAC)
    Ethernet_MacRegisters {
        /// Ethernet MAC configuration
/// register
        (0x000 => maccr: ReadWrite<u32, MACCR::Register>),
        /// Ethernet MAC frame filter
/// register
        (0x004 => macffr: ReadWrite<u32, MACFFR::Register>),
        /// Ethernet MAC hash table high
/// register
        (0x008 => machthr: ReadWrite<u32>),
        /// Ethernet MAC hash table low
/// register
        (0x00C => machtlr: ReadWrite<u32>),
        /// Ethernet MAC MII address
/// register
        (0x010 => macmiiar: ReadWrite<u32, MACMIIAR::Register>),
        /// Ethernet MAC MII data register
        (0x014 => macmiidr: ReadWrite<u32>),
        /// Ethernet MAC flow control
/// register
        (0x018 => macfcr: ReadWrite<u32, MACFCR::Register>),
        /// Ethernet MAC VLAN tag register
        (0x01C => macvlantr: ReadWrite<u32, MACVLANTR::Register>),
        (0x020 => _reserved0),
        /// Ethernet MAC PMT control and status
/// register
        (0x02C => macpmtcsr: ReadWrite<u32, MACPMTCSR::Register>),
        (0x030 => _reserved1),
        /// Ethernet MAC debug register
        (0x034 => macdbgr: ReadOnly<u32, MACDBGR::Register>),
        /// Ethernet MAC interrupt status
/// register
        (0x038 => macsr: ReadWrite<u32, MACSR::Register>),
        /// Ethernet MAC interrupt mask
/// register
        (0x03C => macimr: ReadWrite<u32, MACIMR::Register>),
        /// Ethernet MAC address 0 high
/// register
        (0x040 => maca0hr: ReadWrite<u32, MACA0HR::Register>),
        /// Ethernet MAC address 0 low
/// register
        (0x044 => maca0lr: ReadWrite<u32>),
        /// Ethernet MAC address 1 high
/// register
        (0x048 => maca1hr: ReadWrite<u32, MACA1HR::Register>),
        /// Ethernet MAC address1 low
/// register
        (0x04C => maca1lr: ReadWrite<u32>),
        /// Ethernet MAC address 2 high
/// register
        (0x050 => maca2hr: ReadWrite<u32, MACA2HR::Register>),
        /// Ethernet MAC address 2 low
/// register
        (0x054 => maca2lr: ReadWrite<u32>),
        /// Ethernet MAC address 3 high
/// register
        (0x058 => maca3hr: ReadWrite<u32, MACA3HR::Register>),
        /// Ethernet MAC address 3 low
/// register
        (0x05C => maca3lr: ReadWrite<u32>),
        (0x060 => @END),
    }
}

register_bitfields![u32,
MACCR [
    /// RE
    RE OFFSET(2) NUMBITS(1) [],
    /// TE
    TE OFFSET(3) NUMBITS(1) [],
    /// DC
    DC OFFSET(4) NUMBITS(1) [],
    /// BL
    BL OFFSET(5) NUMBITS(2) [],
    /// APCS
    APCS OFFSET(7) NUMBITS(1) [],
    /// RD
    RD OFFSET(9) NUMBITS(1) [],
    /// IPCO
    IPCO OFFSET(10) NUMBITS(1) [],
    /// DM
    DM OFFSET(11) NUMBITS(1) [],
    /// LM
    LM OFFSET(12) NUMBITS(1) [],
    /// ROD
    ROD OFFSET(13) NUMBITS(1) [],
    /// FES
    FES OFFSET(14) NUMBITS(1) [],
    /// CSD
    CSD OFFSET(16) NUMBITS(1) [],
    /// IFG
    IFG OFFSET(17) NUMBITS(3) [],
    /// JD
    JD OFFSET(22) NUMBITS(1) [],
    /// WD
    WD OFFSET(23) NUMBITS(1) [],
    /// CSTF
    CSTF OFFSET(25) NUMBITS(1) []
],
MACFFR [
    /// PM
    PM OFFSET(0) NUMBITS(1) [],
    /// HU
    HU OFFSET(1) NUMBITS(1) [],
    /// HM
    HM OFFSET(2) NUMBITS(1) [],
    /// DAIF
    DAIF OFFSET(3) NUMBITS(1) [],
    /// RAM
    RAM OFFSET(4) NUMBITS(1) [],
    /// BFD
    BFD OFFSET(5) NUMBITS(1) [],
    /// PCF
    PCF OFFSET(6) NUMBITS(1) [],
    /// SAIF
    SAIF OFFSET(7) NUMBITS(1) [],
    /// SAF
    SAF OFFSET(8) NUMBITS(1) [],
    /// HPF
    HPF OFFSET(9) NUMBITS(1) [],
    /// RA
    RA OFFSET(31) NUMBITS(1) []
],
MACHTHR [
    /// HTH
    HTH OFFSET(0) NUMBITS(32) []
],
MACHTLR [
    /// HTL
    HTL OFFSET(0) NUMBITS(32) []
],
MACMIIAR [
    /// MB
    MB OFFSET(0) NUMBITS(1) [],
    /// MW
    MW OFFSET(1) NUMBITS(1) [],
    /// CR
    CR OFFSET(2) NUMBITS(3) [],
    /// MR
    MR OFFSET(6) NUMBITS(5) [],
    /// PA
    PA OFFSET(11) NUMBITS(5) []
],
MACMIIDR [
    /// TD
    TD OFFSET(0) NUMBITS(16) []
],
MACFCR [
    /// FCB
    FCB OFFSET(0) NUMBITS(1) [],
    /// TFCE
    TFCE OFFSET(1) NUMBITS(1) [],
    /// RFCE
    RFCE OFFSET(2) NUMBITS(1) [],
    /// UPFD
    UPFD OFFSET(3) NUMBITS(1) [],
    /// PLT
    PLT OFFSET(4) NUMBITS(2) [],
    /// ZQPD
    ZQPD OFFSET(7) NUMBITS(1) [],
    /// PT
    PT OFFSET(16) NUMBITS(16) []
],
MACVLANTR [
    /// VLANTI
    VLANTI OFFSET(0) NUMBITS(16) [],
    /// VLANTC
    VLANTC OFFSET(16) NUMBITS(1) []
],
MACPMTCSR [
    /// PD
    PD OFFSET(0) NUMBITS(1) [],
    /// MPE
    MPE OFFSET(1) NUMBITS(1) [],
    /// WFE
    WFE OFFSET(2) NUMBITS(1) [],
    /// MPR
    MPR OFFSET(5) NUMBITS(1) [],
    /// WFR
    WFR OFFSET(6) NUMBITS(1) [],
    /// GU
    GU OFFSET(9) NUMBITS(1) [],
    /// WFFRPR
    WFFRPR OFFSET(31) NUMBITS(1) []
],
MACDBGR [
    MMRPEA OFFSET(0) NUMBITS(1) [],
    MSFRWCS OFFSET(1) NUMBITS(2) [],
    RFWRA OFFSET(4) NUMBITS(1) [],
    RFRCS OFFSET(5) NUMBITS(2) [
        Idle = 0,
        ReadingFrameDate = 1,
        ReadingFrameStatus = 2,
        FlushingFrameDataAndStatus = 3,
    ],
    RFFL OFFSET(8) NUMBITS(2) [
        Empty = 0,
        BelowThreshold = 1,
        AboveThreshold = 2,
        Full = 3,
    ],
    MMTEA OFFSET(16) NUMBITS(1) [],
    MTFCS OFFSET(17) NUMBITS(2) [
        Idle = 0,
        WaitingStatusOrBackoff = 1,
        GeneratingAndTransmitingPauseFrame = 2,
        TransferringInputFrame = 3,
    ],
    MTP OFFSET(19) NUMBITS(1) [],
    TFRS OFFSET(20) NUMBITS(2) [
        Idle = 0,
        Reading = 1,
        WaitingForStatus = 2,
        WritingStatusOrFlushing = 3,
    ],
    TFWA OFFSET(22) NUMBITS(1) [],
    TFNE OFFSET(24) NUMBITS(1) [],
    TFF OFFSET(25) NUMBITS(1) [],
],
MACSR [
    /// PMTS
    PMTS OFFSET(3) NUMBITS(1) [],
    /// MMCS
    MMCS OFFSET(4) NUMBITS(1) [],
    /// MMCRS
    MMCRS OFFSET(5) NUMBITS(1) [],
    /// MMCTS
    MMCTS OFFSET(6) NUMBITS(1) [],
    /// TSTS
    TSTS OFFSET(9) NUMBITS(1) []
],
MACIMR [
    /// PMTIM
    PMTIM OFFSET(3) NUMBITS(1) [],
    /// TSTIM
    TSTIM OFFSET(9) NUMBITS(1) []
],
MACA0HR [
    /// MAC address0 high
    MACA0H OFFSET(0) NUMBITS(16) [],
    /// Always 1
    MO OFFSET(31) NUMBITS(1) []
],
MACA0LR [
    /// 0
    MACA0L OFFSET(0) NUMBITS(32) []
],
MACA1HR [
    /// MACA1H
    MACA1H OFFSET(0) NUMBITS(16) [],
    /// MBC
    MBC OFFSET(24) NUMBITS(6) [],
    /// SA
    SA OFFSET(30) NUMBITS(1) [],
    /// AE
    AE OFFSET(31) NUMBITS(1) []
],
MACA1LR [
    /// MACA1LR
    MACA1LR OFFSET(0) NUMBITS(32) []
],
MACA2HR [
    /// MAC2AH
    MAC2AH OFFSET(0) NUMBITS(16) [],
    /// MBC
    MBC OFFSET(24) NUMBITS(6) [],
    /// SA
    SA OFFSET(30) NUMBITS(1) [],
    /// AE
    AE OFFSET(31) NUMBITS(1) []
],
MACA2LR [
    /// MACA2L
    MACA2L OFFSET(0) NUMBITS(31) []
],
MACA3HR [
    /// MACA3H
    MACA3H OFFSET(0) NUMBITS(16) [],
    /// MBC
    MBC OFFSET(24) NUMBITS(6) [],
    /// SA
    SA OFFSET(30) NUMBITS(1) [],
    /// AE
    AE OFFSET(31) NUMBITS(1) []
],
MACA3LR [
    /// MBCA3L
    MBCA3L OFFSET(0) NUMBITS(32) []
]
];

const ETHERNET_MAC_BASE: StaticRef<Ethernet_MacRegisters> =
    unsafe { StaticRef::new(0x40028000 as *const Ethernet_MacRegisters) };

register_structs! {
    /// Ethernet: DMA controller operation
    Ethernet_DmaRegisters {
        /// Ethernet DMA bus mode register
        (0x000 => dmabmr: ReadWrite<u32, DMABMR::Register>),
        /// Ethernet DMA transmit poll demand
/// register
        (0x004 => dmatpdr: ReadWrite<u32>),
        /// EHERNET DMA receive poll demand
/// register
        (0x008 => dmarpdr: ReadWrite<u32>),
        /// Ethernet DMA receive descriptor list address
/// register
        (0x00C => dmardlar: ReadWrite<u32>),
        /// Ethernet DMA transmit descriptor list
/// address register
        (0x010 => dmatdlar: ReadWrite<u32>),
        /// Ethernet DMA status register
        (0x014 => dmasr: ReadWrite<u32, DMASR::Register>),
        /// Ethernet DMA operation mode
/// register
        (0x018 => dmaomr: ReadWrite<u32, DMAOMR::Register>),
        /// Ethernet DMA interrupt enable
/// register
        (0x01C => dmaier: ReadWrite<u32, DMAIER::Register>),
        /// Ethernet DMA missed frame and buffer
/// overflow counter register
        (0x020 => dmamfbocr: ReadWrite<u32, DMAMFBOCR::Register>),
        /// Ethernet DMA receive status watchdog timer
/// register
        (0x024 => dmarswtr: ReadWrite<u32>),
        (0x028 => _reserved0),
        /// Ethernet DMA current host transmit
/// descriptor register
        (0x048 => dmachtdr: ReadOnly<u32>),
        /// Ethernet DMA current host receive descriptor
/// register
        (0x04C => dmachrdr: ReadOnly<u32>),
        /// Ethernet DMA current host transmit buffer
/// address register
        (0x050 => dmachtbar: ReadOnly<u32>),
        /// Ethernet DMA current host receive buffer
/// address register
        (0x054 => dmachrbar: ReadOnly<u32>),
        (0x058 => @END),
    }
}
register_bitfields![u32,
DMABMR [
    /// SR
    SR OFFSET(0) NUMBITS(1) [],
    /// DA
    DA OFFSET(1) NUMBITS(1) [],
    /// DSL
    DSL OFFSET(2) NUMBITS(5) [],
    /// EDFE
    EDFE OFFSET(7) NUMBITS(1) [],
    /// PBL
    PBL OFFSET(8) NUMBITS(6) [],
    /// RTPR
    RTPR OFFSET(14) NUMBITS(2) [],
    /// FB
    FB OFFSET(16) NUMBITS(1) [],
    /// RDP
    RDP OFFSET(17) NUMBITS(6) [],
    /// USP
    USP OFFSET(23) NUMBITS(1) [],
    /// FPM
    FPM OFFSET(24) NUMBITS(1) [],
    /// AAB
    AAB OFFSET(25) NUMBITS(1) [],
    /// MB
    MB OFFSET(26) NUMBITS(1) []
],
DMATPDR [
    /// TPD
    TPD OFFSET(0) NUMBITS(32) []
],
DMARPDR [
    /// RPD
    RPD OFFSET(0) NUMBITS(32) []
],
DMARDLAR [
    /// SRL
    SRL OFFSET(0) NUMBITS(32) []
],
DMATDLAR [
    /// STL
    STL OFFSET(0) NUMBITS(32) []
],
DMASR [
    /// TS
    TS OFFSET(0) NUMBITS(1) [],
    /// TPSS
    TPSS OFFSET(1) NUMBITS(1) [],
    /// TBUS
    TBUS OFFSET(2) NUMBITS(1) [],
    /// TJTS
    TJTS OFFSET(3) NUMBITS(1) [],
    /// ROS
    ROS OFFSET(4) NUMBITS(1) [],
    /// TUS
    TUS OFFSET(5) NUMBITS(1) [],
    /// RS
    RS OFFSET(6) NUMBITS(1) [],
    /// RBUS
    RBUS OFFSET(7) NUMBITS(1) [],
    /// RPSS
    RPSS OFFSET(8) NUMBITS(1) [],
    /// PWTS
    PWTS OFFSET(9) NUMBITS(1) [],
    /// ETS
    ETS OFFSET(10) NUMBITS(1) [],
    /// FBES
    FBES OFFSET(13) NUMBITS(1) [],
    /// ERS
    ERS OFFSET(14) NUMBITS(1) [],
    /// AIS
    AIS OFFSET(15) NUMBITS(1) [],
    /// NIS
    NIS OFFSET(16) NUMBITS(1) [],
    /// RPS
    RPS OFFSET(17) NUMBITS(3) [],
    /// TPS
    TPS OFFSET(20) NUMBITS(3) [],
    /// EBS
    EBS OFFSET(23) NUMBITS(3) [],
    /// MMCS
    MMCS OFFSET(27) NUMBITS(1) [],
    /// PMTS
    PMTS OFFSET(28) NUMBITS(1) [],
    /// TSTS
    TSTS OFFSET(29) NUMBITS(1) []
],
DMAOMR [
    /// SR
    SR OFFSET(1) NUMBITS(1) [],
    /// OSF
    OSF OFFSET(2) NUMBITS(1) [],
    /// RTC
    RTC OFFSET(3) NUMBITS(2) [],
    /// FUGF
    FUGF OFFSET(6) NUMBITS(1) [],
    /// FEF
    FEF OFFSET(7) NUMBITS(1) [],
    /// ST
    ST OFFSET(13) NUMBITS(1) [],
    /// TTC
    TTC OFFSET(14) NUMBITS(3) [],
    /// FTF
    FTF OFFSET(20) NUMBITS(1) [],
    /// TSF
    TSF OFFSET(21) NUMBITS(1) [],
    /// DFRF
    DFRF OFFSET(24) NUMBITS(1) [],
    /// RSF
    RSF OFFSET(25) NUMBITS(1) [],
    /// DTCEFD
    DTCEFD OFFSET(26) NUMBITS(1) []
],
DMAIER [
    /// TIE
    TIE OFFSET(0) NUMBITS(1) [],
    /// TPSIE
    TPSIE OFFSET(1) NUMBITS(1) [],
    /// TBUIE
    TBUIE OFFSET(2) NUMBITS(1) [],
    /// TJTIE
    TJTIE OFFSET(3) NUMBITS(1) [],
    /// ROIE
    ROIE OFFSET(4) NUMBITS(1) [],
    /// TUIE
    TUIE OFFSET(5) NUMBITS(1) [],
    /// RIE
    RIE OFFSET(6) NUMBITS(1) [],
    /// RBUIE
    RBUIE OFFSET(7) NUMBITS(1) [],
    /// RPSIE
    RPSIE OFFSET(8) NUMBITS(1) [],
    /// RWTIE
    RWTIE OFFSET(9) NUMBITS(1) [],
    /// ETIE
    ETIE OFFSET(10) NUMBITS(1) [],
    /// FBEIE
    FBEIE OFFSET(13) NUMBITS(1) [],
    /// ERIE
    ERIE OFFSET(14) NUMBITS(1) [],
    /// AISE
    AISE OFFSET(15) NUMBITS(1) [],
    /// NISE
    NISE OFFSET(16) NUMBITS(1) []
],
DMAMFBOCR [
    /// MFC
    MFC OFFSET(0) NUMBITS(16) [],
    /// OMFC
    OMFC OFFSET(16) NUMBITS(1) [],
    /// MFA
    MFA OFFSET(17) NUMBITS(11) [],
    /// OFOC
    OFOC OFFSET(28) NUMBITS(1) []
],
DMARSWTR [
    /// RSWTC
    RSWTC OFFSET(0) NUMBITS(8) []
],
DMACHTDR [
    /// HTDAP
    HTDAP OFFSET(0) NUMBITS(32) []
],
DMACHRDR [
    /// HRDAP
    HRDAP OFFSET(0) NUMBITS(32) []
],
DMACHTBAR [
    /// HTBAP
    HTBAP OFFSET(0) NUMBITS(32) []
],
DMACHRBAR [
    /// HRBAP
    HRBAP OFFSET(0) NUMBITS(32) []
]
];

const ETHERNET_DMA_BASE: StaticRef<Ethernet_DmaRegisters> =
    unsafe { StaticRef::new(0x40029000 as *const Ethernet_DmaRegisters) };

#[derive(PartialEq, Debug)]
pub enum EthernetSpeed {
    Speed10Mbs = 0b0,
    Speed100Mbs = 0b1,
}

#[derive(PartialEq, Debug)]
pub enum OperationMode {
    HalfDuplex = 0b0,
    FullDuplex = 0b1,
}

#[derive(PartialEq, Debug)]
pub enum MacTxReaderStatus {
    Idle = 0b00,
    Reading = 0b01,
    WaitingForStatus = 0b10,
    WritingStatusOrFlushing = 0b11,
}

#[derive(PartialEq, Debug)]
pub enum MacTxStatus {
    Idle = 0b00,
    WaitingForStatusOrBackoff = 0b01,
    GeneratingAndTransmitingPauseFrame = 0b10,
    TransferringInputFrame = 0b11,
}

#[derive(PartialEq, Debug)]
pub enum DmaTransmitProcessState {
    Stopped = 0b000,
    FetchingTransmitDescriptor = 0b001,
    WaitingForStatus = 0b010,
    ReadingData = 0b011,
    Suspended = 0b110,
    ClosingTransmitDescriptor = 0b111,
}

pub enum DmaTransmitThreshold {
    Threshold64 = 0b000,
    Threshold128 = 0b001,
    Threshold192 = 0b010,
    Threshold256 = 0b011,
    Threshold40 = 0b100,
    Threshold32 = 0b101,
    Threshold24 = 0b110,
    Threshold16 = 0b111,
}

pub struct Ethernet {
    mac_registers: StaticRef<Ethernet_MacRegisters>,
    dma_registers: StaticRef<Ethernet_DmaRegisters>,
    init_error: OptionalCell<bool>,
}

const DEFAULT_MAC_ADDRESS: u64 = 0x123456;

impl Ethernet {
    pub fn new() -> Self {
        let ethernet = Self {
            mac_registers: ETHERNET_MAC_BASE,
            dma_registers: ETHERNET_DMA_BASE,
            init_error: OptionalCell::new(false),
        };
        ethernet.init();
        // TODO: Remove these functions call
        ethernet
    }

    fn init(&self) {
        self.init_error.set(false);
        self.init_dma();
        self.init_mac();
    }

    fn init_dma(&self) {
        if self.reset_dma().is_err() {
            self.init_error.set(true);
            return;
        }

        if self.flush_dma_transmit_fifo().is_err() {
            self.init_error.set(true);
            return;
        }
    }

    fn init_mac(&self) {
        self.set_mac_address0(DEFAULT_MAC_ADDRESS);
    }

    /* === MAC methods === */

    fn set_ethernet_speed(&self, speed: EthernetSpeed) {
        self.mac_registers.maccr.modify(MACCR::FES.val(speed as u32));
    }

    fn get_ethernet_speed(&self) -> EthernetSpeed {
        match self.mac_registers.maccr.read(MACCR::FES) {
            0 => EthernetSpeed::Speed10Mbs,
            _ => EthernetSpeed::Speed100Mbs,
        }
    }

    fn enable_loopback_mode(&self) {
        self.mac_registers.maccr.modify(MACCR::LM::SET);
    }

    fn disable_loopback_mode(&self) {
        self.mac_registers.maccr.modify(MACCR::LM::CLEAR);
    }

    fn is_loopback_mode_enabled(&self) -> bool {
        match self.mac_registers.maccr.read(MACCR::LM) {
            0 => false,
            _ => true,
        }
    }

    fn set_operation_mode(&self, operation_mode: OperationMode) {
        self.mac_registers.maccr.modify(MACCR::DM.val(operation_mode as u32));
    }

    fn get_operation_mode(&self) -> OperationMode {
        match self.mac_registers.maccr.read(MACCR::DM) {
            0 => OperationMode::HalfDuplex,
            _ => OperationMode::FullDuplex,
        }
    }

    fn enable_mac_transmitter(&self) {
        self.mac_registers.maccr.modify(MACCR::TE::SET);
    }

    fn disable_mac_transmitter(&self) {
        self.mac_registers.maccr.modify(MACCR::TE::CLEAR);
    }

    fn is_mac_transmiter_enabled(&self) -> bool {
        match self.mac_registers.maccr.read(MACCR::TE) {
            0 => false,
            _ => true,
        }
    }

    fn enable_mac_receiver(&self) {
        self.mac_registers.maccr.modify(MACCR::RE::SET);
    }

    fn disable_mac_receiver(&self) {
        self.mac_registers.maccr.modify(MACCR::RE::CLEAR);
    }

    fn is_mac_receiver_enabled(&self) -> bool {
        match self.mac_registers.maccr.read(MACCR::RE) {
            0 => false,
            _ => true,
        }
    }

    fn enable_address_filter(&self) {
        // TODO: Decide whether to use receive all or promiscuous mode
        // TODO: Add source address filtering and hash filtering
        self.mac_registers.macffr.modify(MACFFR::PM::CLEAR);
    }

    fn disable_address_filter(&self) {
        // TODO: Same as above
        self.mac_registers.macffr.modify(MACFFR::PM::SET);
    }

    fn is_address_filter_enabled(&self) -> bool {
        match self.mac_registers.macffr.read(MACFFR::PM) {
            0 => false,
            _ => true,
        }
    }

    fn is_mac_tx_full(&self) -> bool {
        match self.mac_registers.macdbgr.read(MACDBGR::TFF) {
           0 => false,
           _ => true,
        }
    }

    fn is_mac_tx_empty(&self) -> bool {
        match self.mac_registers.macdbgr.read(MACDBGR::TFNE) {
            0 => true,
            _ => false,
        }
    }

    fn is_mac_tx_writer_active(&self) -> bool {
        match self.mac_registers.macdbgr.read(MACDBGR::TFWA) {
            0 => false,
            _ => true,
        }
    }

    fn get_mac_tx_reader_status(&self) -> MacTxReaderStatus {
        match self.mac_registers.macdbgr.read(MACDBGR::TFRS) {
            0b00 => MacTxReaderStatus::Idle,
            0b01 => MacTxReaderStatus::Reading,
            0b10 => MacTxReaderStatus::WaitingForStatus,
            _ => MacTxReaderStatus::WritingStatusOrFlushing,
        }
    }

    fn is_mac_tx_in_pause(&self) -> bool {
        match self.mac_registers.macdbgr.read(MACDBGR::MTP) {
            0 => false,
            _ => true,
        }
    }

    fn get_mac_tx_status(&self) -> MacTxStatus {
        match self.mac_registers.macdbgr.read(MACDBGR::MTFCS) {
            0b00 => MacTxStatus::Idle,
            0b01 => MacTxStatus::WaitingForStatusOrBackoff,
            0b10 => MacTxStatus::GeneratingAndTransmitingPauseFrame,
            _ => MacTxStatus::TransferringInputFrame,
        }
    }

    fn is_mac_mii_active(&self) -> bool {
        match self.mac_registers.macdbgr.read(MACDBGR::MMTEA) {
            0 => false,
            _ => true,
        }
    }

    fn set_mac_address0_high_register(&self, value: u16) {
        self.mac_registers.maca0hr.modify(MACA0HR::MACA0H.val(value as u32));
    }

    fn set_mac_address0_low_register(&self, value: u32) {
        self.mac_registers.maca0lr.set(value);
    }

    fn set_mac_address0(&self, address: u64) {
        let high_bits = ((address & 0xFFFF00000000) >> 32) as u16;
        self.set_mac_address0_high_register(high_bits);
        self.set_mac_address0_low_register((address & 0xFFFFFFFF) as u32);
    }

    fn get_mac_address0(&self) -> u64 {
        (self.mac_registers.maca0hr.read(MACA0HR::MACA0H) as u64) << 32 | self.mac_registers.maca0lr.get() as u64
    }

    fn is_mac_address1_enabled(&self) -> bool {
        match self.mac_registers.maca1hr.read(MACA1HR::AE) {
            0 => false,
            _ => true,
        }
    }

    fn is_mac_address2_enabled(&self) -> bool {
        match self.mac_registers.maca2hr.read(MACA2HR::AE) {
            0 => false,
            _ => true,
        }
    }

    fn is_mac_address3_enabled(&self) -> bool {
        match self.mac_registers.maca3hr.read(MACA3HR::AE) {
            0 => false,
            _ => true,
        }
    }

    /* === DMA methods === */
    fn reset_dma(&self) -> Result<(), ErrorCode> {
        self.dma_registers.dmabmr.modify(DMABMR::SR::SET);

        for _ in 0..1000 {
            if self.dma_registers.dmabmr.read(DMABMR::SR) == 0 {
                return Ok(());
            }
        }

        Err(ErrorCode::FAIL)
    }

    fn dma_transmit_poll_demand(&self) {
        self.dma_registers.dmatpdr.set(1);
    }

    // TODO: Add receive demand pool request

    fn set_transmit_descriptor_list_address(&self, address: u32) -> Result<(), ErrorCode> {
        if self.is_dma_transmition_enabled() == true {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmatdlar.set(address);

        Ok(())
    }

    fn get_transmit_descriptor_list_address(&self) -> u32 {
        self.dma_registers.dmatdlar.get()
    }

    fn get_transmit_process_state(&self) -> DmaTransmitProcessState {
        match self.dma_registers.dmasr.read(DMASR::TPS) {
            0b000 => DmaTransmitProcessState::Stopped,
            0b001 => DmaTransmitProcessState::FetchingTransmitDescriptor,
            0b010 => DmaTransmitProcessState::WaitingForStatus,
            0b011 => DmaTransmitProcessState::ReadingData,
            0b110 => DmaTransmitProcessState::Suspended,
            _ => DmaTransmitProcessState::ClosingTransmitDescriptor,
        }
    }

    fn dma_abnormal_interruption(&self) -> bool {
        match self.dma_registers.dmasr.read(DMASR::AIS) {
            0 => false,
            _ => true,
        }
    }

    fn has_dma_transmition_finished(&self) -> bool {
        match self.dma_registers.dmasr.read(DMASR::TS) {
            0 => false,
            _ => true,
        }
    }

    fn clear_dma_transmition_completion_status(&self) {
        self.dma_registers.dmasr.modify(DMASR::TS::CLEAR);
    }

    fn enable_transmit_store_and_forward(&self) -> Result<(), ErrorCode> {
        if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::TSF::SET);

        Ok(())
    }

    fn disable_transmit_store_and_forward(&self) -> Result<(), ErrorCode> {
        if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::TSF::CLEAR);

        Ok(())
    }

    fn flush_dma_transmit_fifo(&self) -> Result<(), ErrorCode> {
        if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::FTF::SET);

        // TODO: Adjust this value
        for _ in 0..1000 {
            if self.dma_registers.dmaomr.read(DMAOMR::FTF) == 0 {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    fn set_dma_transmition_threshold_control(&self, threshold: DmaTransmitThreshold)  {
        self.dma_registers.dmaomr.modify(DMAOMR::TTC.val(threshold as u32));
    }

    fn start_dma_transmition(&self) -> Result<(), ErrorCode> {
        if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::ST::SET);

        Ok(())
    }

    fn stop_dma_transmition(&self) -> Result<(), ErrorCode> {
        if self.get_transmit_process_state() != DmaTransmitProcessState::Suspended {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::ST::CLEAR);

        Ok(())
    }

    fn is_dma_transmition_enabled(&self) -> bool {
        match self.dma_registers.dmaomr.read(DMAOMR::ST) {
            0 => false,
            _ => true,
        }
    }
}

pub mod tests {
    use super::*;

    fn test_mac_default_values(ethernet: &Ethernet) {
        assert_eq!(Some(false), ethernet.init_error.extract());
        assert_eq!(EthernetSpeed::Speed10Mbs, ethernet.get_ethernet_speed());
        assert_eq!(false, ethernet.is_loopback_mode_enabled());
        assert_eq!(OperationMode::HalfDuplex, ethernet.get_operation_mode());
        assert_eq!(false, ethernet.is_mac_transmiter_enabled());
        assert_eq!(false, ethernet.is_mac_receiver_enabled());
        assert_eq!(false, ethernet.is_address_filter_enabled());
        assert_eq!(false, ethernet.is_mac_tx_full());
        assert_eq!(true, ethernet.is_mac_tx_empty());
        assert_eq!(false, ethernet.is_mac_tx_writer_active());
        assert_eq!(MacTxReaderStatus::Idle, ethernet.get_mac_tx_reader_status());
        assert_eq!(false, ethernet.is_mac_tx_in_pause());
        assert_eq!(MacTxStatus::Idle, ethernet.get_mac_tx_status());
        assert_eq!(false, ethernet.is_mac_mii_active());
        // NOTE: Why this address is 0 and not DEFAULT_MAC_ADDRESS
        assert_eq!(0, ethernet.get_mac_address0());
        assert_eq!(false, ethernet.is_mac_address1_enabled());
        assert_eq!(false, ethernet.is_mac_address2_enabled());
        assert_eq!(false, ethernet.is_mac_address3_enabled());
    }

    fn test_dma_default_values(ethernet: &Ethernet) {
        assert_eq!(0, ethernet.get_transmit_descriptor_list_address());
        assert_eq!(DmaTransmitProcessState::Stopped, ethernet.get_transmit_process_state());
        assert_eq!(0, ethernet.dma_registers.dmaomr.read(DMAOMR::TSF));
        assert_eq!(false, ethernet.is_dma_transmition_enabled());
    }

    pub fn test_ethernet_init(ethernet: &Ethernet) {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing Ethernet initialization...");

        ethernet.init();
        test_mac_default_values(ethernet);
        test_dma_default_values(ethernet);

        debug!("Finished testing Ethernet initialization");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }

    pub fn run_all(ethernet: &Ethernet) {
        debug!("");
        debug!("================================================");
        debug!("Starting testing the Ethernet...");
        test_ethernet_init(ethernet);
        debug!("================================================");
        debug!("Finished testing the Ethernet. Everything is alright!");
        debug!("");
    }
}