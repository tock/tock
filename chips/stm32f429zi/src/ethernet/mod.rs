use core::cell::Cell;
use cortexm4::support::nop;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::ErrorCode;
use kernel::platform::chip::ClockInterface;
use kernel::hil::ethernet::TransmitClient;
use kernel::hil::ethernet::ReceiveClient;

use crate::rcc;
use crate::rcc::PeripheralClock;
use crate::rcc::PeripheralClockType;

pub mod mac_address;
use crate::ethernet::mac_address::MacAddress;

pub mod transmit_descriptor;
use crate::ethernet::transmit_descriptor::TransmitDescriptor;

pub mod receive_descriptor;
use crate::ethernet::receive_descriptor::ReceiveDescriptor;

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
    /// RWTS
    RWTS OFFSET(9) NUMBITS(1) [],
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

register_structs! {
    /// Ethernet: MAC management counters
    Ethernet_MmcRegisters {
        /// Ethernet MMC control register
        (0x000 => mmccr: ReadWrite<u32, MMCCR::Register>),
        /// Ethernet MMC receive interrupt
/// register
        (0x004 => mmcrir: ReadWrite<u32, MMCRIR::Register>),
        /// Ethernet MMC transmit interrupt
/// register
        (0x008 => mmctir: ReadOnly<u32, MMCTIR::Register>),
        /// Ethernet MMC receive interrupt mask
/// register
        (0x00C => mmcrimr: ReadWrite<u32, MMCRIMR::Register>),
        /// Ethernet MMC transmit interrupt mask
/// register
        (0x010 => mmctimr: ReadWrite<u32, MMCTIMR::Register>),
        (0x014 => _reserved0),
        /// Ethernet MMC transmitted good frames after a
/// single collision counter
        (0x04C => mmctgfsccr: ReadOnly<u32>),
        /// Ethernet MMC transmitted good frames after
/// more than a single collision
        (0x050 => mmctgfmsccr: ReadOnly<u32>),
        (0x054 => _reserved1),
        /// Ethernet MMC transmitted good frames counter
/// register
        (0x068 => mmctgfcr: ReadOnly<u32>),
        (0x06C => _reserved2),
        /// Ethernet MMC received frames with CRC error
/// counter register
        (0x094 => mmcrfcecr: ReadOnly<u32>),
        /// Ethernet MMC received frames with alignment
/// error counter register
        (0x098 => mmcrfaecr: ReadOnly<u32>),
        (0x09C => _reserved3),
        /// MMC received good unicast frames counter
/// register
        (0x0C4 => mmcrgufcr: ReadOnly<u32>),
        (0x0C8 => @END),
    }
}
register_bitfields![u32,
MMCCR [
    /// CR
    CR OFFSET(0) NUMBITS(1) [],
    /// CSR
    CSR OFFSET(1) NUMBITS(1) [],
    /// ROR
    ROR OFFSET(2) NUMBITS(1) [],
    /// MCF
    MCF OFFSET(3) NUMBITS(1) [],
    /// MCP
    MCP OFFSET(4) NUMBITS(1) [],
    /// MCFHP
    MCFHP OFFSET(5) NUMBITS(1) []
],
MMCRIR [
    /// RFCES
    RFCES OFFSET(5) NUMBITS(1) [],
    /// RFAES
    RFAES OFFSET(6) NUMBITS(1) [],
    /// RGUFS
    RGUFS OFFSET(17) NUMBITS(1) []
],
MMCTIR [
    /// TGFSCS
    TGFSCS OFFSET(14) NUMBITS(1) [],
    /// TGFMSCS
    TGFMSCS OFFSET(15) NUMBITS(1) [],
    /// TGFS
    TGFS OFFSET(21) NUMBITS(1) []
],
MMCRIMR [
    /// RFCEM
    RFCEM OFFSET(5) NUMBITS(1) [],
    /// RFAEM
    RFAEM OFFSET(6) NUMBITS(1) [],
    /// RGUFM
    RGUFM OFFSET(17) NUMBITS(1) []
],
MMCTIMR [
    /// TGFSCM
    TGFSCM OFFSET(14) NUMBITS(1) [],
    /// TGFMSCM
    TGFMSCM OFFSET(15) NUMBITS(1) [],
    /// TGFM
    TGFM OFFSET(16) NUMBITS(1) []
],
MMCTGFSCCR [
    /// TGFSCC
    TGFSCC OFFSET(0) NUMBITS(32) []
],
MMCTGFMSCCR [
    /// TGFMSCC
    TGFMSCC OFFSET(0) NUMBITS(32) []
],
MMCTGFCR [
    /// HTL
    TGFC OFFSET(0) NUMBITS(32) []
],
MMCRFCECR [
    /// RFCFC
    RFCFC OFFSET(0) NUMBITS(32) []
],
MMCRFAECR [
    /// RFAEC
    RFAEC OFFSET(0) NUMBITS(32) []
],
MMCRGUFCR [
    /// RGUFC
    RGUFC OFFSET(0) NUMBITS(32) []
]
];

const ETHERNET_MMC_BASE: StaticRef<Ethernet_MmcRegisters> =
    unsafe { StaticRef::new(0x40028100 as *const Ethernet_MmcRegisters) };

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
pub enum MacTxWriterStatus {
    Idle = 0b00,
    WaitingForStatusOrBackoff = 0b01,
    GeneratingAndTransmitingPauseFrame = 0b10,
    TransferringInputFrame = 0b11,
}

#[derive(PartialEq, Debug)]
pub enum RxFifoLevel {
    Empty = 0b00,
    BelowThreshold = 0b01,
    AboveThreshold = 0b10,
    Full = 0b11,
}

#[derive(PartialEq, Debug)]
pub enum MacRxReaderStatus {
    Idle = 0b00,
    ReadingFrame = 0b01,
    ReadingFrameStatusOrTimeStamp = 0b10,
    FlushingFrameDataAndStatus  = 0b11,
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

#[derive(PartialEq, Debug)]
pub enum DmaReceiveProcessState {
    Stopped = 0b000,
    FetchingReceiveDescriptor = 0b001,
    WaitingForReceivePacket = 0b011,
    Suspended = 0b100,
    ClosingReceiveDescriptor = 0b101,
    TransferringReceivePacketDataToHostMemory = 0b111,
}

#[derive(PartialEq, Debug)]
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

#[derive(PartialEq, Debug)]
pub enum DmaReceiveThreshold {
    Threshold64 = 0b00,
    Threshold32 = 0b01,
    Threshold96 = 0b10,
    Threshold128 = 0b11,
}

struct EthernetClocks<'a> {
    mac: PeripheralClock<'a>,
    mac_tx: PeripheralClock<'a>,
    mac_rx: PeripheralClock<'a>,
    mac_ptp: PeripheralClock<'a>,
}

impl<'a> EthernetClocks<'a> {
    fn new(rcc: &'a rcc::Rcc) -> Self {
        Self {
            mac: PeripheralClock::new(PeripheralClockType::AHB1(rcc::HCLK1::ETHMACEN), rcc),
            mac_tx: PeripheralClock::new(PeripheralClockType::AHB1(rcc::HCLK1::ETHMACTXEN), rcc),
            mac_rx: PeripheralClock::new(PeripheralClockType::AHB1(rcc::HCLK1::ETHMACRXEN), rcc),
            mac_ptp: PeripheralClock::new(PeripheralClockType::AHB1(rcc::HCLK1::ETHMACPTPEN), rcc),
        }
    }

    fn enable(&self) {
        self.mac.enable();
        self.mac_rx.enable();
        self.mac_tx.enable();
        self.mac_ptp.enable();
    }
}

pub const MAX_BUFFER_SIZE: usize = 1524;

pub struct Ethernet<'a> {
    mac_registers: StaticRef<Ethernet_MacRegisters>,
    mmc_registers: StaticRef<Ethernet_MmcRegisters>,
    dma_registers: StaticRef<Ethernet_DmaRegisters>,
    transmit_descriptor: TransmitDescriptor,
    receive_descriptor: ReceiveDescriptor,
    transmit_buffer: TakeCell<'a, [u8; MAX_BUFFER_SIZE]>,
    transmit_client: OptionalCell<&'a dyn TransmitClient>,
    receive_client: OptionalCell<&'a dyn ReceiveClient>,
    clocks: EthernetClocks<'a>,
    mac_address0: OptionalCell<MacAddress>,
}

pub const DEFAULT_MAC_ADDRESS: u64 = 0x4D5D6462951B;

impl<'a> Ethernet<'a> {
    pub fn new(
        rcc: &'a rcc::Rcc,
        transmit_buffer: &'a mut [u8; MAX_BUFFER_SIZE],
    ) -> Self {
        Self {
            mac_registers: ETHERNET_MAC_BASE,
            mmc_registers: ETHERNET_MMC_BASE,
            dma_registers: ETHERNET_DMA_BASE,
            transmit_descriptor: TransmitDescriptor::new(),
            receive_descriptor: ReceiveDescriptor::new(),
            transmit_buffer: TakeCell::new(transmit_buffer),
            transmit_client: OptionalCell::empty(),
            receive_client: OptionalCell::empty(),
            clocks: EthernetClocks::new(rcc),
            mac_address0: OptionalCell::new(MacAddress::new()),
        }
    }

    pub fn init(&self) -> Result<(), ErrorCode> {
        self.clocks.enable();
        self.init_transmit_descriptors();
        self.init_receive_descriptors();
        self.init_dma()?;
        self.init_mac();

        Ok(())
    }

    fn init_transmit_descriptors(&self) {
        self.transmit_descriptor.release();
        self.transmit_descriptor.enable_interrupt_on_completion();
        self.transmit_descriptor.set_as_first_segment();
        self.transmit_descriptor.set_as_last_segment();
        self.transmit_descriptor.enable_pad();
        self.transmit_descriptor.enable_crc();
        self.transmit_descriptor.set_transmit_end_of_ring();
    }

    fn init_receive_descriptors(&self) {
        self.receive_descriptor.release();
        self.receive_descriptor.enable_interrupt_on_completion();
        self.receive_descriptor.set_receive_end_of_ring();
    }

    fn init_dma(&self) -> Result<(), ErrorCode> {
        self.reset_dma()?;
        self.flush_dma_transmit_fifo()?;
        self.disable_flushing_received_frames();
        self.forward_error_frames();
        self.forward_undersized_good_frames();
        self.enable_transmit_store_and_forward()?;
        self.enable_receive_store_and_forward()?;

        self.set_transmit_descriptor_list_address(&self.transmit_descriptor as *const TransmitDescriptor as u32)?;
        self.set_receive_descriptor_list_address(&self.receive_descriptor as *const ReceiveDescriptor as u32)?;

        self.enable_all_interrupts();

        Ok(())
    }

    fn init_mac(&self) {
        self.set_mac_address0(DEFAULT_MAC_ADDRESS.into());
        self.disable_mac_watchdog();
        self.set_ethernet_speed(EthernetSpeed::Speed10Mbs);
        self.disable_loopback_mode();
        self.set_operation_mode(OperationMode::FullDuplex);
        self.disable_address_filter();
    }

    /* === MAC methods === */

    fn disable_mac_watchdog(&self) {
        self.mac_registers.maccr.modify(MACCR::WD::SET);
    }

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
        self.mac_registers.maccr.is_set(MACCR::LM)
    }

    fn set_operation_mode(&self, operation_mode: OperationMode) {
        self.mac_registers.maccr.modify(MACCR::DM.val(operation_mode as u32));
    }

    fn get_operation_mode(&self) -> OperationMode {
        match self.mac_registers.maccr.is_set(MACCR::DM) {
            false => OperationMode::HalfDuplex,
            true => OperationMode::FullDuplex,
        }
    }

    fn enable_mac_transmitter(&self) {
        self.mac_registers.maccr.modify(MACCR::TE::SET);
    }

    fn disable_mac_transmitter(&self) {
        self.mac_registers.maccr.modify(MACCR::TE::CLEAR);
    }

    fn is_mac_transmiter_enabled(&self) -> bool {
        self.mac_registers.maccr.is_set(MACCR::TE)
    }

    fn enable_mac_receiver(&self) {
        self.mac_registers.maccr.modify(MACCR::RE::SET);
    }

    fn disable_mac_receiver(&self) {
        self.mac_registers.maccr.modify(MACCR::RE::CLEAR);
    }

    fn is_mac_receiver_enabled(&self) -> bool {
        self.mac_registers.maccr.is_set(MACCR::RE)
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
        !self.mac_registers.macffr.is_set(MACFFR::PM)
    }

    fn is_mac_tx_full(&self) -> bool {
        self.mac_registers.macdbgr.is_set(MACDBGR::TFF)
    }

    fn is_mac_tx_empty(&self) -> bool {
        !self.mac_registers.macdbgr.is_set(MACDBGR::TFNE)
    }

    fn is_mac_tx_writer_active(&self) -> bool {
        self.mac_registers.macdbgr.is_set(MACDBGR::TFWA)
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

    fn get_mac_tx_writer_status(&self) -> MacTxWriterStatus {
        match self.mac_registers.macdbgr.read(MACDBGR::MTFCS) {
            0b00 => MacTxWriterStatus::Idle,
            0b01 => MacTxWriterStatus::WaitingForStatusOrBackoff,
            0b10 => MacTxWriterStatus::GeneratingAndTransmitingPauseFrame,
            _ => MacTxWriterStatus::TransferringInputFrame,
        }
    }

    fn is_mac_mii_active(&self) -> bool {
        match self.mac_registers.macdbgr.read(MACDBGR::MMTEA) {
            0 => false,
            _ => true,
        }
    }

    fn get_rx_fifo_fill_level(&self) -> RxFifoLevel {
        match self.mac_registers.macdbgr.read(MACDBGR::RFFL) {
            0b00 => RxFifoLevel::Empty,
            0b01 => RxFifoLevel::BelowThreshold,
            0b10 => RxFifoLevel::AboveThreshold,
            _ => RxFifoLevel::Full,
        }
    }

    fn get_mac_rx_reader_status(&self) -> MacRxReaderStatus {
        match self.mac_registers.macdbgr.read(MACDBGR::RFRCS) {
            0b00 => MacRxReaderStatus::Idle,
            0b01 => MacRxReaderStatus::ReadingFrame,
            0b10 => MacRxReaderStatus::ReadingFrameStatusOrTimeStamp,
            _ => MacRxReaderStatus::FlushingFrameDataAndStatus,
        }
    }

    fn is_mac_rx_writer_active(&self) -> bool {
        self.mac_registers.macdbgr.is_set(MACDBGR::RFWRA)
    }

    fn set_mac_address0_high_register(&self, value: u16) {
        self.mac_registers.maca0hr.modify(MACA0HR::MACA0H.val(value as u32));
    }

    fn set_mac_address0_low_register(&self, value: u32) {
        self.mac_registers.maca0lr.set(value);
    }

    fn set_mac_address0(&self, mac_address: MacAddress) {
        let address: u64 = mac_address.into();
        let high_bits = (address >> 32) as u16;
        self.set_mac_address0_high_register(high_bits);
        self.set_mac_address0_low_register((address & 0xFFFFFFFF) as u32);

        let mut new_address = self.get_mac_address0();
        new_address.set_address(address);
        self.mac_address0.set(new_address);
    }

    fn get_mac_address0(&self) -> MacAddress {
        self.mac_address0.extract().unwrap()
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

        for _ in 0..100 {
            if self.dma_registers.dmabmr.read(DMABMR::SR) == 0 {
                return Ok(());
            }
        }

        Err(ErrorCode::FAIL)
    }

    fn dma_transmit_poll_demand(&self) {
        self.dma_registers.dmatpdr.set(1);
    }

    fn dma_receive_poll_demand(&self) {
        self.dma_registers.dmarpdr.set(1);
    }

    fn set_transmit_descriptor_list_address(&self, address: u32) -> Result<(), ErrorCode> {
        if self.is_dma_transmission_enabled() == true {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmatdlar.set(address);

        Ok(())
    }

    fn set_receive_descriptor_list_address(&self, address: u32) -> Result<(), ErrorCode> {
        if self.is_dma_reception_enabled() == true {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmardlar.set(address);

        Ok(())
    }

    fn get_transmit_descriptor_list_address(&self) -> u32 {
        self.dma_registers.dmatdlar.get()
    }

    fn get_receive_descriptor_list_address(&self) -> u32 {
        self.dma_registers.dmardlar.get()
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

    fn get_receive_process_state(&self) -> DmaReceiveProcessState {
        match self.dma_registers.dmasr.read(DMASR::RPS) {
            0b000 => DmaReceiveProcessState::Stopped,
            0b001 => DmaReceiveProcessState::FetchingReceiveDescriptor,
            0b011 => DmaReceiveProcessState::WaitingForReceivePacket,
            0b100 => DmaReceiveProcessState::Suspended,
            0b101 => DmaReceiveProcessState::ClosingReceiveDescriptor,
            _ => DmaReceiveProcessState::TransferringReceivePacketDataToHostMemory,
        }
    }

    fn did_normal_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::NIS)
    }

    // TODO: Am I allowed to clear this bit
    //fn clear_dma_normal_interrupt(&self) {
        //self.dma_registers.dmasr.modify(DMASR::NIS::SET);
    //}

    fn did_abnormal_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::AIS)
    }

    // TODO: Am I allowed to clear this bit?
    //#[allow(dead_code)]
    //fn clear_dma_abnormal_interrupt(&self) {
        //self.dma_registers.dmasr.modify(DMASR::AIS::SET);
    //}
    fn did_early_receive_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::ERS)
    }

    fn clear_early_receive_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::ERS::SET);
    }

    fn did_fatal_bus_error_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::FBES)
    }

    fn clear_fatal_bus_error_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::FBES::SET);
    }

    fn did_early_transmit_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::ETS)
    }

    fn clear_early_transmit_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::ETS::SET);
    }

    fn did_receive_watchdog_timeout_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::RWTS)
    }

    fn clear_receive_watchdog_timeout_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::RWTS::SET);
    }

    fn did_receive_process_stopped_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::RPSS)
    }

    fn clear_receive_process_stopped_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::RPSS::SET);
    }

    fn did_receive_buffer_unavailable_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::RBUS)
    }

    fn clear_receive_buffer_unavailable_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::RBUS::SET);
    }
    
    fn did_receive_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::RS)
    }

    fn clear_receive_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::RS::SET);
    }

    fn did_transmit_buffer_underflow_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::TUS)
    }

    fn clear_transmit_buffer_underflow_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::TUS::SET);
    }

    fn did_receive_fifo_overflow_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::ROS)
    }

    fn clear_receive_fifo_overflow_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::ROS::SET);
    }

    fn did_transmit_jabber_timeout_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::TJTS)
    }

    fn clear_transmit_jabber_timeout_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::TJTS::SET);
    }

    fn did_transmit_buffer_unavailable_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::TBUS)
    }

    fn clear_transmit_buffer_unavailable_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::TBUS::SET);
    }

    fn did_transmit_process_stopped_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::TPSS)
    }

    fn clear_transmit_process_stopped_interrupt_occur(&self) {
        self.dma_registers.dmasr.modify(DMASR::TPSS::SET);
    }

    fn did_transmit_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::TS)
    }

    fn clear_transmit_interrupt(&self) {
        self.dma_registers.dmasr.modify(DMASR::TS::SET);
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

    fn is_transmit_store_and_forward_enabled(&self) -> bool {
        self.dma_registers.dmaomr.is_set(DMAOMR::TSF)
    }

    fn disable_flushing_received_frames(&self) {
        self.dma_registers.dmaomr.modify(DMAOMR::DFRF::SET);
    }

    fn forward_error_frames(&self) {
        self.dma_registers.dmaomr.modify(DMAOMR::FEF::SET);
    }

    fn forward_undersized_good_frames(&self) {
        self.dma_registers.dmaomr.modify(DMAOMR::FUGF::SET);
    }

    fn enable_receive_store_and_forward(&self) -> Result<(), ErrorCode> {
        if self.get_receive_process_state() != DmaReceiveProcessState::Stopped {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::RSF::SET);

        Ok(())
    }

    fn disable_receive_store_and_forward(&self) -> Result<(), ErrorCode> {
        if self.get_receive_process_state() != DmaReceiveProcessState::Stopped {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::RSF::CLEAR);

        Ok(())
    }

    fn is_receive_store_and_forward_enabled(&self) -> bool {
        self.dma_registers.dmaomr.is_set(DMAOMR::RSF)
    }

    fn flush_dma_transmit_fifo(&self) -> Result<(), ErrorCode> {
        if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::FTF::SET);

        // TODO: Adjust this value
        for _ in 0..100 {
            if self.dma_registers.dmaomr.read(DMAOMR::FTF) == 0 {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    fn set_dma_transmission_threshold_control(&self, threshold: DmaTransmitThreshold) -> Result<(), ErrorCode> {
        if self.is_dma_transmission_enabled() {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::TTC.val(threshold as u32));

        Ok(())
    }

    fn get_dma_transmission_threshold_control(&self) -> DmaTransmitThreshold {
        match self.dma_registers.dmaomr.read(DMAOMR::TTC) {
            0b000 => DmaTransmitThreshold::Threshold64,
            0b001 => DmaTransmitThreshold::Threshold128,
            0b010 => DmaTransmitThreshold::Threshold192,
            0b011 => DmaTransmitThreshold::Threshold256,
            0b100 => DmaTransmitThreshold::Threshold40,
            0b101 => DmaTransmitThreshold::Threshold32,
            0b110 => DmaTransmitThreshold::Threshold24,
            _ => DmaTransmitThreshold::Threshold16,
        }
    }

    fn set_dma_receive_treshold_control(&self, threshold: DmaReceiveThreshold) -> Result<(), ErrorCode> {
        if self.is_dma_reception_enabled() {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::RTC.val(threshold as u32));

        Ok(())
    }

    fn get_dma_receive_threshold_control(&self) -> DmaReceiveThreshold {
        match self.dma_registers.dmaomr.read(DMAOMR::RTC)  {
            0b00 => DmaReceiveThreshold::Threshold64,
            0b01 => DmaReceiveThreshold::Threshold32,
            0b10 => DmaReceiveThreshold::Threshold96,
            _ => DmaReceiveThreshold::Threshold128,
        }
    }

    fn enable_dma_transmission(&self) -> Result<(), ErrorCode> {
        if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
            return Err(ErrorCode::ALREADY);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::ST::SET);

        Ok(())
    }

    fn disable_dma_transmission(&self) -> Result<(), ErrorCode> {
        if self.get_transmit_process_state() != DmaTransmitProcessState::Suspended {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::ST::CLEAR);

        Ok(())
    }

    fn is_dma_transmission_enabled(&self) -> bool {
        match self.dma_registers.dmaomr.read(DMAOMR::ST) {
            0 => false,
            _ => true,
        }
    }

    fn enable_dma_reception(&self) -> Result<(), ErrorCode> {
        if self.get_receive_process_state() != DmaReceiveProcessState::Stopped {
            return Err(ErrorCode::ALREADY);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::SR::SET);

        Ok(())
    }

    fn disable_dma_reception(&self) -> Result<(), ErrorCode> {
        if self.get_receive_process_state() != DmaReceiveProcessState::Suspended {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::SR::CLEAR);

        Ok(())
    }

    fn is_dma_reception_enabled(&self) -> bool {
        self.dma_registers.dmaomr.is_set(DMAOMR::ST) 
    }

    fn enable_normal_interrupts(&self) {
        self.dma_registers.dmaier.modify(DMAIER::NISE::SET);
    }

    fn disable_normal_interrupts(&self) {
        self.dma_registers.dmaier.modify(DMAIER::NISE::CLEAR);
    }

    fn are_normal_interrupts_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::NISE)
    }

    fn enable_abnormal_interrupt_summary(&self) {
        self.dma_registers.dmaier.modify(DMAIER::AISE::SET);
    }

    fn disable_abnormal_interrupt_summary(&self) {
        self.dma_registers.dmaier.modify(DMAIER::AISE::CLEAR);
    }

    fn are_abnormal_interrupts_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::AISE)
    }

    fn enable_early_receive_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::ERIE::SET);
    }

    fn disable_early_receive_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::ERIE::CLEAR);
    }

    fn is_early_receive_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::ERIE)
    }

    fn enable_fatal_bus_error_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::FBEIE::SET);
    }

    fn disable_fatal_bus_error_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::FBEIE::CLEAR);
    }

    fn is_fatal_bus_error_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::FBEIE)
    }

    fn enable_early_transmit_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::ETIE::SET);
    }

    fn disable_early_transmit_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::ETIE::CLEAR);
    }

    fn is_early_transmit_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::ETIE)
    }

    fn enable_receive_watchdog_timeout_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::RWTIE::SET);
    }

    fn disable_receive_watchdog_timeout_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::RWTIE::CLEAR);
    }

    fn is_receive_watchdog_timeout_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::RWTIE)
    }

    fn enable_receive_process_stopped_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::RPSIE::SET);
    }

    fn disable_receive_process_stopped_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::RPSIE::CLEAR);
    }

    fn is_receive_process_stopped_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::RPSIE)
    }

    fn enable_receive_buffer_unavailable(&self) {
        self.dma_registers.dmaier.modify(DMAIER::RBUIE::SET);
    }

    fn disable_receive_buffer_unavailable(&self) {
        self.dma_registers.dmaier.modify(DMAIER::RBUIE::CLEAR);
    }

    fn is_receive_buffer_unavailable(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::RBUIE)
    }

    fn enable_underflow_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TUIE::SET);
    }

    fn disable_underflow_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TUIE::CLEAR);
    }

    fn is_underflow_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::TUIE)
    }

    fn enable_overflow_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::ROIE::SET);
    }

    fn disable_overflow_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::ROIE::CLEAR);
    }

    fn is_overflow_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::ROIE)
    }

    fn enable_transmit_jabber_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TJTIE::SET);
    }

    fn disable_transmit_jabber_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TJTIE::CLEAR);
    }

    fn is_transmit_jabber_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::TJTIE)
    }

    fn enable_transmit_process_stopped_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TPSIE::SET);
    }

    fn disable_transmit_process_stopped_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TPSIE::CLEAR);
    }

    fn is_transmit_process_stopped(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::TPSIE)
    }

    fn enable_transmit_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TIE::SET);
    }

    fn disable_transmit_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TIE::CLEAR);
    }

    fn is_transmit_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::TIE)
    }

    fn enable_receive_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::RIE::SET);
    }

    fn disable_receive_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::RIE::CLEAR);
    }

    fn is_receive_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::RIE)
    }

    fn enable_transmit_buffer_unavailable_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TBUIE::SET);
    }

    fn disable_transmit_buffer_unavailable_interrupt(&self) {
        self.dma_registers.dmaier.modify(DMAIER::TBUIE::CLEAR);
    }

    fn is_transmit_buffer_unavailable_interrupt_enabled(&self) -> bool {
        self.dma_registers.dmaier.is_set(DMAIER::TBUIE)
    }

    #[allow(dead_code)]
    fn get_current_host_transmit_descriptor_address(&self) -> u32 {
        self.dma_registers.dmachtdr.get()
    }

    #[allow(dead_code)]
    fn get_current_host_transmit_buffer_address(&self) -> u32 {
        self.dma_registers.dmachtbar.get()
    }

    /* === High-level functions */

    fn enable_transmission(&self) -> Result<(), ErrorCode> {
        self.enable_dma_transmission()?;
        self.enable_mac_transmitter();

        for _ in 0..10 {
            if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    fn disable_transmission(&self) -> Result<(), ErrorCode> {
        self.disable_dma_transmission()?;
        self.disable_mac_transmitter();

        Ok(())
    }

    fn enable_reception(&self) -> Result<(), ErrorCode> {
        self.enable_dma_reception()?;
        self.enable_mac_receiver();

        for _ in 0..10 {
            if self.get_receive_process_state() != DmaReceiveProcessState::Stopped {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    fn disable_reception(&self) -> Result<(), ErrorCode> {
        self.disable_dma_reception()?;
        self.disable_mac_receiver();

        Ok(())
    }

    fn is_reception_enabled(&self) -> bool {
        self.is_mac_receiver_enabled()  && self.get_receive_process_state() != DmaReceiveProcessState::Stopped
    }

    fn start_interface(&self) -> Result<(), ErrorCode> {
        self.enable_transmission()?;
        self.enable_reception()?;

        Ok(())
    }

    fn stop_interface(&self) -> Result<(), ErrorCode> {
        self.disable_transmission()?;
        self.disable_reception()?;

        Ok(())
    }

    fn enable_all_normal_interrupts(&self) {
        self.enable_normal_interrupts();
        self.enable_early_receive_interrupt();
        self.enable_receive_interrupt();
        self.enable_transmit_buffer_unavailable_interrupt();
        self.enable_transmit_interrupt();
    }

    fn enable_all_error_interrupts(&self) {
        self.enable_abnormal_interrupt_summary();
        self.enable_early_receive_interrupt();
        self.enable_fatal_bus_error_interrupt();
        self.enable_early_transmit_interrupt();
        self.enable_receive_watchdog_timeout_interrupt();
        self.enable_receive_process_stopped_interrupt();
        self.enable_receive_buffer_unavailable();
        self.enable_underflow_interrupt();
        self.enable_overflow_interrupt();
        self.enable_transmit_jabber_interrupt();
        self.enable_transmit_process_stopped_interrupt();
    }

    fn enable_all_interrupts(&self) {
        self.enable_all_normal_interrupts();
        self.enable_all_error_interrupts();
    }

    fn handle_normal_interrupt(&self) {
        if self.did_transmit_interrupt_occur() {
            self.clear_transmit_interrupt();
            self.transmit_client.map(|client| client.transmitted_frame(Ok(())));
        } if self.did_transmit_buffer_unavailable_interrupt_occur() {
            self.clear_transmit_buffer_unavailable_interrupt();
        } if self.did_receive_interrupt_occur() {
            self.receive_client.map(|client| client.received_frame(Ok(()), self.receive_descriptor.get_frame_length()));
            self.clear_receive_interrupt();
        } if self.did_early_receive_interrupt_occur() {
            self.clear_early_receive_interrupt();
        }
    }

    fn handle_abnormal_interrupt(&self) {
        if self.did_fatal_bus_error_interrupt_occur() {
            self.clear_fatal_bus_error_interrupt();
            panic!("Fatal bus error");
        } if self.did_early_transmit_interrupt_occur() {
            self.clear_early_transmit_interrupt();
        } if self.did_receive_watchdog_timeout_interrupt_occur() {
            self.clear_receive_watchdog_timeout_interrupt();
            panic!("Receive watchdog timeout interrupt");
        } if self.did_receive_process_stopped_interrupt_occur() {
            self.clear_receive_process_stopped_interrupt();
        } if self.did_receive_buffer_unavailable_interrupt_occur() {
            self.clear_receive_buffer_unavailable_interrupt();
        } if self.did_transmit_buffer_underflow_interrupt_occur() {
            self.clear_transmit_buffer_underflow_interrupt();
            panic!("Transmit buffer underflow interrupt");
        } if self.did_receive_fifo_overflow_interrupt_occur() {
            self.clear_receive_fifo_overflow_interrupt();
            panic!("Receive buffer overflow interrupt");
        } if self.did_transmit_jabber_timeout_interrupt_occur() {
            self.clear_transmit_jabber_timeout_interrupt();
            panic!("Transmit buffer jabber timeout interrupt");
        } if self.did_transmit_process_stopped_interrupt_occur() {
            self.clear_transmit_process_stopped_interrupt_occur();
        }
    }

    pub(crate) fn handle_interrupt(&self) {
        if self.did_normal_interrupt_occur() {
            self.handle_normal_interrupt();
        }
        if self.did_abnormal_interrupt_occur() {
            self.handle_abnormal_interrupt();
        }
    }

    fn transmit_frame(&self,
        transmit_client: &'a dyn TransmitClient,
        destination_address: MacAddress,
        data: &[u8]
    ) -> Result<(), ErrorCode> {
        // If DMA and MAC are off, return an error
        if !self.is_mac_transmiter_enabled() || self.get_transmit_process_state() == DmaTransmitProcessState::Stopped {
            return Err(ErrorCode::OFF);
        }

        // Check if transmiter is busy
        if self.get_transmit_process_state() != DmaTransmitProcessState::Suspended {
            return Err(ErrorCode::BUSY);
        }

        // Set the buffer size and return an error if it is too big
        let data_length = data.len();
        let buffer_length = data_length + 14;
        // WARNING: this assumes automatic padding and CRC generation
        let frame_length = if buffer_length < 60 {
            64
        } else {
            buffer_length + 4
        };

        self.transmit_descriptor.set_buffer1_size(buffer_length)?;

        // Prepare buffer
        // Can't panic since the transmit buffer is set when the driver is created
        let transmit_buffer = self.transmit_buffer.take().unwrap();
        transmit_buffer[0..6].copy_from_slice(&self.get_mac_address0().get_address());
        transmit_buffer[6..12].copy_from_slice(&destination_address.get_address());
        transmit_buffer[12] = (frame_length >> 8) as u8;
        transmit_buffer[13] = frame_length as u8;
        transmit_buffer[14..(data_length + 14)].copy_from_slice(data);

        // Prepare transmit descriptor
        self.transmit_descriptor.set_buffer1_address(transmit_buffer.as_ptr() as u32);
        self.transmit_buffer.put(Some(transmit_buffer));

        // Set the transmit client
        self.transmit_client.set(transmit_client);

        // Acquire the transmit descriptor
        self.transmit_descriptor.acquire();

        // Wait 4 CPU cycles until everything is written to the RAM
        for _ in 0..4 {
            nop();
        }

        // Send a poll request to the DMA
        self.dma_transmit_poll_demand();

        Ok(())
    }

    fn receive_frame(&self, receive_client: &'a dyn ReceiveClient, buffer: &mut [u8]) -> Result<(), ErrorCode> {
        // If DMA and MAC receptions are off, return an error
        if !self.is_reception_enabled() {
            return Err(ErrorCode::OFF);
        }

        // Check if reception is busy
        if self.get_receive_process_state() != DmaReceiveProcessState::Suspended {
            return Err(ErrorCode::BUSY);
        }

        self.receive_client.set(receive_client);

        // Setup receive descriptor
        self.receive_descriptor.set_buffer1_address(buffer.as_mut_ptr() as u32);
        self.receive_descriptor.set_buffer2_address(buffer.as_mut_ptr() as u32);
        self.receive_descriptor.set_buffer1_size(buffer.len())?;
        self.receive_descriptor.set_buffer2_size(0)?;

        // DMA becomes the owner of the descriptor
        self.receive_descriptor.acquire();

        // Send a poll request to the DMA
        self.dma_receive_poll_demand();

        Ok(())
    }
}

pub mod tests {
    use super::*;
    use kernel::debug;

    pub struct DummyTransmitClient {
        pub(self) transmit_status: OptionalCell<Result<(), ErrorCode>>
    }

    impl DummyTransmitClient {
        pub fn new() -> Self {
            Self {
                transmit_status: OptionalCell::empty()
            }
        }
    }

    impl TransmitClient for DummyTransmitClient {
        fn transmitted_frame(&self, transmit_status: Result<(), ErrorCode>) {
            self.transmit_status.set(transmit_status);
            debug!("DummyTransmitClient::transmitted_frame() called!");
        }
    }

    pub struct DummyReceiveClient<'a> {
        pub(self) receive_status: OptionalCell<Result<(), ErrorCode>>,
        pub(self) receive_buffer: TakeCell<'a, [u8; MAX_BUFFER_SIZE]>,
        pub(self) bytes_received: Cell<usize>
    }

    impl<'a> DummyReceiveClient<'a> {
        pub fn new(receive_buffer: &'a mut [u8; MAX_BUFFER_SIZE]) -> Self {
            Self {
                receive_status: OptionalCell::empty(),
                receive_buffer: TakeCell::new(receive_buffer),
                bytes_received: Cell::new(0)
            }
        }
    }

    impl<'a> ReceiveClient for DummyReceiveClient<'a> {
        fn received_frame(&self,
            receive_status: Result<(), ErrorCode>,
            received_frame_length: usize
        ) {
            self.receive_status.set(receive_status);
            self.bytes_received.replace(self.bytes_received.get() + received_frame_length);
        }
    }

    fn test_mac_default_values(ethernet: &Ethernet) {
        assert_eq!(EthernetSpeed::Speed10Mbs, ethernet.get_ethernet_speed());
        assert_eq!(false, ethernet.is_loopback_mode_enabled());
        assert_eq!(OperationMode::FullDuplex, ethernet.get_operation_mode());
        assert_eq!(false, ethernet.is_mac_transmiter_enabled());
        assert_eq!(false, ethernet.is_mac_receiver_enabled());
        assert_eq!(false, ethernet.is_address_filter_enabled());
        assert_eq!(false, ethernet.is_mac_tx_full());
        assert_eq!(true, ethernet.is_mac_tx_empty());
        assert_eq!(false, ethernet.is_mac_tx_writer_active());
        assert_eq!(MacTxReaderStatus::Idle, ethernet.get_mac_tx_reader_status());
        assert_eq!(false, ethernet.is_mac_tx_in_pause());
        assert_eq!(MacTxWriterStatus::Idle, ethernet.get_mac_tx_writer_status());

        assert_eq!(RxFifoLevel::Empty, ethernet.get_rx_fifo_fill_level());
        assert_eq!(MacRxReaderStatus::Idle, ethernet.get_mac_rx_reader_status());
        assert_eq!(false, ethernet.is_mac_rx_writer_active());

        assert_eq!(false, ethernet.is_mac_mii_active());
        assert_eq!(MacAddress::from(DEFAULT_MAC_ADDRESS), ethernet.get_mac_address0());
        assert_eq!(false, ethernet.is_mac_address1_enabled());
        assert_eq!(false, ethernet.is_mac_address2_enabled());
        assert_eq!(false, ethernet.is_mac_address3_enabled());
    }

    fn test_dma_default_values(ethernet: &Ethernet) {
        assert_eq!(&ethernet.transmit_descriptor as *const TransmitDescriptor as u32,
            ethernet.get_transmit_descriptor_list_address());
        assert_eq!(DmaTransmitProcessState::Stopped, ethernet.get_transmit_process_state());
        assert_eq!(true, ethernet.is_transmit_store_and_forward_enabled());
        assert_eq!(false, ethernet.is_dma_transmission_enabled());
        assert_eq!(DmaTransmitThreshold::Threshold64, ethernet.get_dma_transmission_threshold_control());

        assert_eq!(&ethernet.receive_descriptor as *const ReceiveDescriptor as u32, ethernet.get_receive_descriptor_list_address());
        assert_eq!(false, ethernet.is_receive_store_and_forward_enabled());
        assert_eq!(false, ethernet.is_dma_reception_enabled());
        assert_eq!(DmaReceiveThreshold::Threshold64, ethernet.get_dma_receive_threshold_control());

        assert_eq!(false, ethernet.did_normal_interrupt_occur());
        assert_eq!(false, ethernet.did_abnormal_interrupt_occur());
    }

    pub fn test_ethernet_init(ethernet: &Ethernet) {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing Ethernet initialization...");

        assert_eq!(Ok(()), ethernet.init());
        test_mac_default_values(ethernet);
        test_dma_default_values(ethernet);

        debug!("Finished testing Ethernet initialization");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }

    fn test_ethernet_transmission_configuration(ethernet: &Ethernet) {
        ethernet.enable_mac_transmitter();
        assert_eq!(true, ethernet.is_mac_transmiter_enabled());
        ethernet.disable_mac_transmitter();
        assert_eq!(false, ethernet.is_mac_transmiter_enabled());

        assert_eq!(Ok(()), ethernet.set_transmit_descriptor_list_address(0x12345));
        // The last two bits are ignore since the bus width is 32 bits
        assert_eq!(0x12344, ethernet.get_transmit_descriptor_list_address());

        assert_eq!(Ok(()), ethernet.enable_transmit_store_and_forward());
        assert_eq!(true, ethernet.is_transmit_store_and_forward_enabled());
        assert_eq!(Ok(()), ethernet.disable_transmit_store_and_forward());
        assert_eq!(false, ethernet.is_transmit_store_and_forward_enabled());

        assert_eq!(Ok(()), ethernet.set_dma_transmission_threshold_control(DmaTransmitThreshold::Threshold192));
        assert_eq!(DmaTransmitThreshold::Threshold192, ethernet.get_dma_transmission_threshold_control());
        assert_eq!(Ok(()), ethernet.set_dma_transmission_threshold_control(DmaTransmitThreshold::Threshold32));
        assert_eq!(DmaTransmitThreshold::Threshold32, ethernet.get_dma_transmission_threshold_control());
        assert_eq!(Ok(()), ethernet.set_dma_transmission_threshold_control(DmaTransmitThreshold::Threshold64));
        assert_eq!(DmaTransmitThreshold::Threshold64, ethernet.get_dma_transmission_threshold_control());


        ethernet.enable_transmit_interrupt();
        assert_eq!(true, ethernet.is_transmit_interrupt_enabled());
        ethernet.disable_transmit_interrupt();
        assert_eq!(false, ethernet.is_transmit_interrupt_enabled());
    }

    fn test_ethernet_reception_configuration(ethernet: &Ethernet) {
        ethernet.enable_mac_receiver();
        assert_eq!(true, ethernet.is_mac_receiver_enabled());
        ethernet.disable_mac_receiver();
        assert_eq!(false, ethernet.is_mac_receiver_enabled());

        assert_eq!(Ok(()), ethernet.set_receive_descriptor_list_address(0x12345));
        assert_eq!(0x12344, ethernet.get_receive_descriptor_list_address());

        assert_eq!(Ok(()), ethernet.enable_receive_store_and_forward());
        assert_eq!(true, ethernet.is_receive_store_and_forward_enabled());
        assert_eq!(Ok(()), ethernet.disable_receive_store_and_forward());
        assert_eq!(false, ethernet.is_receive_store_and_forward_enabled());

        assert_eq!(Ok(()), ethernet.set_dma_receive_treshold_control(DmaReceiveThreshold::Threshold32));
        assert_eq!(DmaReceiveThreshold::Threshold32, ethernet.get_dma_receive_threshold_control());
        assert_eq!(Ok(()), ethernet.set_dma_receive_treshold_control(DmaReceiveThreshold::Threshold128));
        assert_eq!(DmaReceiveThreshold::Threshold128, ethernet.get_dma_receive_threshold_control());
        assert_eq!(Ok(()), ethernet.set_dma_receive_treshold_control(DmaReceiveThreshold::Threshold64));
        assert_eq!(DmaReceiveThreshold::Threshold64, ethernet.get_dma_receive_threshold_control());
    }

    pub fn test_ethernet_basic_configuration(ethernet: &Ethernet) {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing Ethernet basic configuration...");

        assert_eq!(Ok(()), ethernet.init());

        ethernet.set_ethernet_speed(EthernetSpeed::Speed100Mbs);
        assert_eq!(EthernetSpeed::Speed100Mbs, ethernet.get_ethernet_speed());
        ethernet.set_ethernet_speed(EthernetSpeed::Speed10Mbs);
        assert_eq!(EthernetSpeed::Speed10Mbs, ethernet.get_ethernet_speed());

        ethernet.enable_loopback_mode();
        assert_eq!(true, ethernet.is_loopback_mode_enabled());
        ethernet.disable_loopback_mode();
        assert_eq!(false, ethernet.is_loopback_mode_enabled());

        ethernet.set_operation_mode(OperationMode::FullDuplex);
        assert_eq!(OperationMode::FullDuplex, ethernet.get_operation_mode());
        ethernet.set_operation_mode(OperationMode::HalfDuplex);
        assert_eq!(OperationMode::HalfDuplex, ethernet.get_operation_mode());

        ethernet.enable_address_filter();
        assert_eq!(true, ethernet.is_address_filter_enabled());
        ethernet.disable_address_filter();
        assert_eq!(false, ethernet.is_address_filter_enabled());

        ethernet.set_mac_address0_high_register(0x4321);
        // NOTE: The actual value of this assert depends on the DEFAULT_MAC_ADDRESS
        assert_eq!(0x4321, ethernet.mac_registers.maca0hr.read(MACA0HR::MACA0H));
        ethernet.set_mac_address0_low_register(0xCBA98765);
        assert_eq!(0xCBA98765, ethernet.mac_registers.maca0lr.get());

        ethernet.set_mac_address0(0x112233445566.into());
        assert_eq!(MacAddress::from(0x112233445566), ethernet.get_mac_address0());
        ethernet.set_mac_address0(DEFAULT_MAC_ADDRESS.into());
        assert_eq!(MacAddress::from(DEFAULT_MAC_ADDRESS), ethernet.get_mac_address0());

        ethernet.enable_normal_interrupts();
        assert_eq!(true, ethernet.are_normal_interrupts_enabled());
        ethernet.disable_normal_interrupts();
        assert_eq!(false, ethernet.are_normal_interrupts_enabled());

        test_ethernet_transmission_configuration(ethernet);
        test_ethernet_reception_configuration(ethernet);

        // Restore Ethernet to its initial state
        assert_eq!(Ok(()), ethernet.init());

        debug!("Finished testing Ethernet basic configuration...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }

    pub fn test_ethernet_interrupts(ethernet: &Ethernet) {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing frame transmission...");

        /* Normal interrupts */

        ethernet.enable_normal_interrupts();
        assert_eq!(true, ethernet.are_normal_interrupts_enabled());
        ethernet.disable_normal_interrupts();
        assert_eq!(false, ethernet.are_normal_interrupts_enabled());

        ethernet.enable_early_receive_interrupt();
        assert_eq!(true, ethernet.is_early_receive_interrupt_enabled());
        ethernet.disable_early_receive_interrupt();
        assert_eq!(false, ethernet.is_early_receive_interrupt_enabled());

        ethernet.enable_receive_interrupt();
        assert_eq!(true, ethernet.is_receive_interrupt_enabled());
        ethernet.disable_receive_interrupt();
        assert_eq!(false, ethernet.is_receive_interrupt_enabled());

        ethernet.enable_transmit_buffer_unavailable_interrupt();
        assert_eq!(true, ethernet.is_transmit_buffer_unavailable_interrupt_enabled());
        ethernet.disable_transmit_buffer_unavailable_interrupt();
        assert_eq!(false, ethernet.is_transmit_buffer_unavailable_interrupt_enabled());

        ethernet.enable_transmit_interrupt();
        assert_eq!(true, ethernet.is_transmit_interrupt_enabled());
        ethernet.disable_transmit_interrupt();
        assert_eq!(false, ethernet.is_transmit_interrupt_enabled());

        /* Abnormal interrupts */

        ethernet.enable_abnormal_interrupt_summary();
        assert_eq!(true, ethernet.are_abnormal_interrupts_enabled());
        ethernet.disable_abnormal_interrupt_summary();
        assert_eq!(false, ethernet.are_abnormal_interrupts_enabled());

        ethernet.enable_fatal_bus_error_interrupt();
        assert_eq!(true, ethernet.is_fatal_bus_error_interrupt_enabled());
        ethernet.disable_fatal_bus_error_interrupt();
        assert_eq!(false, ethernet.is_fatal_bus_error_interrupt_enabled());

        ethernet.enable_early_transmit_interrupt();
        assert_eq!(true, ethernet.is_early_transmit_interrupt_enabled());
        ethernet.disable_early_transmit_interrupt();
        assert_eq!(false, ethernet.is_early_transmit_interrupt_enabled());

        ethernet.enable_receive_watchdog_timeout_interrupt();
        assert_eq!(true, ethernet.is_receive_watchdog_timeout_interrupt_enabled());
        ethernet.disable_receive_watchdog_timeout_interrupt();
        assert_eq!(false, ethernet.is_receive_watchdog_timeout_interrupt_enabled());

        ethernet.enable_receive_process_stopped_interrupt();
        assert_eq!(true, ethernet.is_receive_process_stopped_interrupt_enabled());
        ethernet.disable_receive_process_stopped_interrupt();
        assert_eq!(false, ethernet.is_receive_process_stopped_interrupt_enabled());

        ethernet.enable_receive_buffer_unavailable();
        assert_eq!(true, ethernet.is_receive_buffer_unavailable());
        ethernet.disable_receive_buffer_unavailable();
        assert_eq!(false, ethernet.is_receive_buffer_unavailable());

        ethernet.enable_underflow_interrupt();
        assert_eq!(true, ethernet.is_underflow_interrupt_enabled());
        ethernet.disable_underflow_interrupt();
        assert_eq!(false, ethernet.is_underflow_interrupt_enabled());

        ethernet.enable_overflow_interrupt();
        assert_eq!(true, ethernet.is_overflow_interrupt_enabled());
        ethernet.disable_overflow_interrupt();
        assert_eq!(false, ethernet.is_overflow_interrupt_enabled());

        ethernet.enable_transmit_jabber_interrupt();
        assert_eq!(true, ethernet.is_transmit_jabber_interrupt_enabled());
        ethernet.disable_transmit_jabber_interrupt();
        assert_eq!(false, ethernet.is_transmit_jabber_interrupt_enabled());

        ethernet.enable_transmit_process_stopped_interrupt();
        assert_eq!(true, ethernet.is_transmit_process_stopped());
        ethernet.disable_transmit_process_stopped_interrupt();
        assert_eq!(false, ethernet.is_transmit_process_stopped());

        debug!("Finished testing frame transmission...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }


    pub fn test_frame_transmission<'a>(ethernet: &'a Ethernet<'a>, transmit_client: &'a DummyTransmitClient) {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing frame transmission...");
        let destination_address: MacAddress = MacAddress::from(0x112233445566);
        // Impossible to send a frame while transmission is disabled
        assert_eq!(Err(ErrorCode::OFF), ethernet.transmit_frame(transmit_client, destination_address, b"Hello!"));
        ethernet.handle_interrupt();

        // Enable Ethernet transmission
        assert_eq!(Ok(()), ethernet.enable_transmission());
        ethernet.handle_interrupt();

        // Now, frames can be send
        for frame_index in 0..100000 {
            assert_eq!(Ok(()), ethernet.transmit_frame(transmit_client, destination_address, b"Hello!"));
            assert_eq!(DmaTransmitProcessState::WaitingForStatus, ethernet.get_transmit_process_state());
            for _ in 0..100 {
                nop();
            }
            assert_eq!(DmaTransmitProcessState::Suspended, ethernet.get_transmit_process_state());
            assert_eq!(frame_index + 1, ethernet.mmc_registers.mmctgfcr.get());
        }

        // Disable transmission
        assert_eq!(Ok(()), ethernet.disable_transmission());

        debug!("Finished testing frame transmission...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }

    pub fn test_frame_reception<'a>(
        ethernet: &'a Ethernet<'a>,
        transmit_client: &'a DummyTransmitClient,
        receive_client: &'a DummyReceiveClient
    ) {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing frame reception...");
        // Impossible to get a frame while reception is disabled
        let receive_buffer = receive_client.receive_buffer.take().unwrap();
        assert_eq!(
            Err(ErrorCode::OFF),
            ethernet.receive_frame(receive_client, receive_buffer)
        );
        ethernet.handle_interrupt();

        // Enable reception
        assert_eq!(Ok(()), ethernet.enable_reception());
        ethernet.handle_interrupt();

        for frame_index in 0..100000 {
            assert_ne!(Err(ErrorCode::OFF), ethernet.receive_frame(receive_client, receive_buffer));
            // Simulate a delay
            for _ in 0..100 {
                nop();
            }
            ethernet.handle_interrupt();
        }
        debug!("Received buffer: {:?}", &receive_buffer[0..64]);
        debug!("RX FIFO fill level: {:?}", ethernet.get_rx_fifo_fill_level());
        debug!("Receive process state: {:?}", ethernet.get_receive_process_state());
        debug!("Good unicast received frames: {:?}", ethernet.mmc_registers.mmcrgufcr.get());
        debug!("CRC errors: {:?}", ethernet.mmc_registers.mmcrfcecr.get());
        debug!("Alignment errors: {:?}", ethernet.mmc_registers.mmcrfaecr.get());
        debug!("Received bytes: {:?}", receive_client.bytes_received.take());

        // Stop reception
        assert_eq!(Ok(()), ethernet.disable_reception());
        ethernet.handle_interrupt();
        receive_client.receive_buffer.put(Some(receive_buffer));

        debug!("Finished testing frame reception...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }

    pub fn run_all<'a>(
        ethernet: &'a Ethernet<'a>,
        transmit_client: &'a DummyTransmitClient,
        receive_client: &'a DummyReceiveClient
    ) {
        debug!("");
        debug!("================================================");
        debug!("Starting testing the Ethernet...");
        //super::mac_address::tests::test_mac_address();
        //test_ethernet_init(ethernet);
        //test_ethernet_basic_configuration(ethernet);
        //test_ethernet_interrupts(ethernet);
        //super::transmit_descriptor::tests::test_transmit_descriptor();
        //super::receive_descriptor::tests::test_receive_descriptor();
        //test_frame_transmission(ethernet, transmit_client);
        test_frame_reception(ethernet, transmit_client, receive_client);
        debug!("Finished testing the Ethernet. Everything is alright!");
        debug!("================================================");
        debug!("");
    }
}