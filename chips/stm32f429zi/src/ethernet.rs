use kernel::utilities::StaticRef;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite, InMemoryRegister};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::ErrorCode;
use kernel::debug;
use kernel::platform::chip::ClockInterface;

use crate::rcc;
use crate::rcc::PeripheralClock;
use crate::rcc::PeripheralClockType;

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

pub struct MacAddress {
    address: [u8; 6],
}

impl MacAddress {
    pub fn new() -> Self {
        Self {
            address: [0; 6],
        }
    }

    pub fn set_address(&mut self, address: u64)  {
        let mask: u64 = 0xFF0000000000;
        for index in 0..6 {
            self.address[index] = ((address & (mask >> (index * 8))) >> (40 - 8 * index)) as u8;
        }
    }

    pub fn get_address(&self) -> [u8; 6] {
        self.address
    }
}

register_bitfields![u32,
TDES0 [
    OWN OFFSET(31) NUMBITS(1) [],
    IC OFFSET(30) NUMBITS(1) [],
    LS OFFSET(29) NUMBITS(1) [],
    FS OFFSET(28) NUMBITS(1) [],
    DC OFFSET(27) NUMBITS(1) [],
    DP OFFSET(26) NUMBITS(1) [],
    TTSE OFFSET(25) NUMBITS(1) [],
    CIC OFFSET(22) NUMBITS(2) [
        ChecksumInsertionDisabled = 0,
        IpHeaderChecksumInserionOnly = 1,
        IpHeaderAndPayloadChecksumInsertion = 2,
        IpHeaderPayloadAndPseudoHeaderChecksumInserion = 3,
    ],
    TER OFFSET(21) NUMBITS(1) [],
    TCH OFFSET(20) NUMBITS(1) [],
    TTSS OFFSET(17) NUMBITS(1) [],
    IHE OFFSET(16) NUMBITS(1) [],
    ES OFFSET(15) NUMBITS(1) [],
    JT OFFSET(14) NUMBITS(1) [],
    FF OFFSET(13) NUMBITS(1) [],
    IPE OFFSET(12) NUMBITS(1) [],
    LCA OFFSET(11) NUMBITS(1) [],
    NC OFFSET(10) NUMBITS(1) [],
    LCO OFFSET(9) NUMBITS(1) [],
    EC OFFSET(8) NUMBITS(1) [],
    VF OFFSET(7) NUMBITS(1) [],
    CC OFFSET(3) NUMBITS(4) [],
    ED OFFSET(2) NUMBITS(1) [],
    UF OFFSET(1) NUMBITS(1) [],
    DB OFFSET(1) NUMBITS(1) [],
],
TDES1 [
    TBS2 OFFSET(16) NUMBITS(13) [],
    TBS1 OFFSET(0) NUMBITS(13) [],
],
];

register_structs! {
    TransmitDescriptor {
        (0x000 => tdes0: InMemoryRegister<u32, TDES0::Register>),
        (0x004 => tdes1: InMemoryRegister<u32, TDES1::Register>),
        (0x008 => tdes2: InMemoryRegister<u32, ()>),
        (0x00C => tdes3: InMemoryRegister<u32, ()>),
        (0x010 => @END),
    }
}

impl TransmitDescriptor {
    fn new() -> Self {
        Self {
            tdes0: InMemoryRegister::new(0),
            tdes1: InMemoryRegister::new(0),
            tdes2: InMemoryRegister::new(0),
            tdes3: InMemoryRegister::new(0),
        }
    }

    fn acquire(&self) {
        self.tdes0.modify(TDES0::OWN::SET);
    }

    fn release(&self) {
        self.tdes0.modify(TDES0::OWN::CLEAR);
    }

    fn is_acquired(&self) -> bool {
        self.tdes0.is_set(TDES0::OWN)
    }

    fn set_as_last_segment(&self) {
        self.tdes0.modify(TDES0::LS::SET);
    }

    fn clear_as_last_segment(&self) {
        self.tdes0.modify(TDES0::LS::CLEAR);
    }

    fn is_last_segment(&self) -> bool {
        self.tdes0.is_set(TDES0::LS)
    }

    fn set_as_first_segment(&self) {
        self.tdes0.modify(TDES0::FS::SET);
    }

    fn clear_as_first_segment(&self) {
        self.tdes0.modify(TDES0::FS::CLEAR);
    }

    fn is_first_segment(&self) -> bool {
        self.tdes0.is_set(TDES0::FS)
    }

    fn enable_crc(&self) {
        self.tdes0.modify(TDES0::DC::CLEAR);
    }

    fn disable_crc(&self) {
        self.tdes0.modify(TDES0::DC::SET);
    }

    fn is_crc_disabled(&self) -> bool {
        self.tdes0.is_set(TDES0::DC)
    }

    fn enable_pad(&self) {
        self.tdes0.modify(TDES0::DP::CLEAR);
    }

    fn disable_pad(&self) {
        self.tdes0.modify(TDES0::DP::SET);
    }

    fn is_pad_disabled(&self) -> bool {
        self.tdes0.is_set(TDES0::DP)
    }

    fn set_transmit_end_of_ring(&self) {
        self.tdes0.modify(TDES0::TER::SET);
    }

    fn clear_transmit_end_of_ring(&self) {
        self.tdes0.modify(TDES0::TER::CLEAR);
    }

    fn is_transmit_end_of_ring(&self) -> bool {
        self.tdes0.is_set(TDES0::TER)
    }

    fn set_buffer1_size(&self, size: u16) -> Result<(), ErrorCode> {
        if size >= 1 << 14 {
            return Err(ErrorCode::SIZE);
        }

        self.tdes1.modify(TDES1::TBS1.val(size as u32));

        Ok(())
    }

    fn get_buffer1_size(&self) -> u16 {
        self.tdes1.read(TDES1::TBS1) as u16
    }

    fn set_buffer1_address(&self, pointer: *const u32) {
        self.tdes2.set(pointer as u32);
    }

    fn get_buffer1_address(&self) -> u32 {
        self.tdes2.get()
    }
}

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

pub struct Ethernet<'a> {
    mac_registers: StaticRef<Ethernet_MacRegisters>,
    dma_registers: StaticRef<Ethernet_DmaRegisters>,
    transmit_descriptor: TransmitDescriptor,
    clocks: EthernetClocks<'a>,
}

const DEFAULT_MAC_ADDRESS: u64 = 0x123456789ABC;

impl<'a> Ethernet<'a> {
    pub fn new(rcc: &'a rcc::Rcc) -> Self {
        Self {
            mac_registers: ETHERNET_MAC_BASE,
            dma_registers: ETHERNET_DMA_BASE,
            transmit_descriptor: TransmitDescriptor::new(),
            clocks: EthernetClocks::new(rcc),
        }
    }

    pub(crate) fn init(&self) -> Result<(), ErrorCode> {
        self.clocks.enable();
        self.transmit_descriptor.release();
        self.init_dma()?;
        self.init_mac();

        Ok(())
    }

    fn init_dma(&self) -> Result<(), ErrorCode> {
        self.reset_dma()?;
        self.flush_dma_transmit_fifo()?;
        self.set_transmit_descriptor_list_address(&self.transmit_descriptor as *const TransmitDescriptor as u32)?;

        Ok(())
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

    // TODO: Add receive demand pool request

    fn set_transmit_descriptor_list_address(&self, address: u32) -> Result<(), ErrorCode> {
        if self.is_dma_transmission_enabled() == true {
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
        self.dma_registers.dmasr.is_set(DMASR::AIS)
    }

    fn has_dma_transmission_finished(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::TS)
    }

    fn clear_dma_transmission_completion_status(&self) {
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

    fn start_dma_transmission(&self) -> Result<(), ErrorCode> {
        if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers.dmaomr.modify(DMAOMR::ST::SET);

        Ok(())
    }

    fn stop_dma_transmission(&self) -> Result<(), ErrorCode> {
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

    /* === High-level functions */

    fn enable_transmission(&self) -> Result<(), ErrorCode> {
        self.enable_mac_transmitter();
        self.start_dma_transmission()?;

        for _ in 0..10 {
            if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    fn disable_transmission(&self) -> Result<(), ErrorCode> {
        self.disable_mac_transmitter();
        self.stop_dma_transmission()?;

        return Ok(())
    }

    fn send_frame_sync(&self) -> Result<(), ErrorCode> {
        // If DMA and MAC are off, return an error
        if !self.is_mac_transmiter_enabled() || self.get_transmit_process_state() == DmaTransmitProcessState::Stopped {
            return Err(ErrorCode::OFF);
        }

        // Check if transmiter is busy
        if self.get_transmit_process_state() != DmaTransmitProcessState::Suspended {
            return Err(ErrorCode::BUSY);
        }

        Ok(())
    }
}

pub mod tests {
    use super::*;

    fn test_mac_default_values(ethernet: &Ethernet) {
        assert_eq!(EthernetSpeed::Speed10Mbs, ethernet.get_ethernet_speed());
        assert_eq!(false, ethernet.is_loopback_mode_enabled());
        assert_eq!(OperationMode::HalfDuplex, ethernet.get_operation_mode());
        assert_eq!(false, ethernet.is_mac_transmiter_enabled());
        assert_eq!(false, ethernet.is_mac_receiver_enabled());
        assert_eq!(true, ethernet.is_address_filter_enabled());
        assert_eq!(false, ethernet.is_mac_tx_full());
        assert_eq!(true, ethernet.is_mac_tx_empty());
        assert_eq!(false, ethernet.is_mac_tx_writer_active());
        assert_eq!(MacTxReaderStatus::Idle, ethernet.get_mac_tx_reader_status());
        assert_eq!(false, ethernet.is_mac_tx_in_pause());
        assert_eq!(MacTxStatus::Idle, ethernet.get_mac_tx_status());
        assert_eq!(false, ethernet.is_mac_mii_active());
        assert_eq!(DEFAULT_MAC_ADDRESS, ethernet.get_mac_address0());
        assert_eq!(false, ethernet.is_mac_address1_enabled());
        assert_eq!(false, ethernet.is_mac_address2_enabled());
        assert_eq!(false, ethernet.is_mac_address3_enabled());
    }

    fn test_dma_default_values(ethernet: &Ethernet) {
        assert_eq!(&ethernet.transmit_descriptor as *const TransmitDescriptor as u32,
            ethernet.get_transmit_descriptor_list_address());
        assert_eq!(DmaTransmitProcessState::Stopped, ethernet.get_transmit_process_state());
        assert_eq!(false, ethernet.dma_abnormal_interruption());
        assert_eq!(false, ethernet.is_transmit_store_and_forward_enabled());
        assert_eq!(false, ethernet.is_dma_transmission_enabled());
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

    pub fn test_mac_address() {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing Ethernet MAC address struct...");

        let mut mac_address = MacAddress::new();
        assert_eq!([0; 6], mac_address.get_address());
        mac_address.set_address(DEFAULT_MAC_ADDRESS);
        debug!("{:?}", mac_address.get_address());
        assert_eq!([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC], mac_address.get_address());

        debug!("Finished testing Ethernet MAC address struct");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
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

        ethernet.enable_mac_transmitter();
        assert_eq!(true, ethernet.is_mac_transmiter_enabled());
        ethernet.disable_mac_transmitter();
        assert_eq!(false, ethernet.is_mac_transmiter_enabled());

        ethernet.enable_mac_receiver();
        assert_eq!(true, ethernet.is_mac_receiver_enabled());
        ethernet.disable_mac_receiver();
        assert_eq!(false, ethernet.is_mac_receiver_enabled());

        ethernet.enable_address_filter();
        assert_eq!(true, ethernet.is_address_filter_enabled());
        ethernet.disable_address_filter();
        assert_eq!(false, ethernet.is_address_filter_enabled());

        ethernet.set_mac_address0_high_register(0x4321);
        // NOTE: The actual value of this assert depends on the DEFAULT_MAC_ADDRESS
        assert_eq!(0x432156789ABC, ethernet.get_mac_address0());
        ethernet.set_mac_address0_low_register(0xCBA98765);
        assert_eq!(0x4321CBA98765, ethernet.get_mac_address0());
        ethernet.set_mac_address0(0x112233445566);
        assert_eq!(0x112233445566, ethernet.get_mac_address0());
        ethernet.set_mac_address0(DEFAULT_MAC_ADDRESS);
        assert_eq!(DEFAULT_MAC_ADDRESS, ethernet.get_mac_address0());

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

        debug!("Finished testing Ethernet basic configuration...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }

    pub fn test_transmit_descriptor() {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing Ethernet basic configuration...");

        let transmit_descriptor = TransmitDescriptor::new();

        transmit_descriptor.acquire();
        assert_eq!(true, transmit_descriptor.is_acquired());
        transmit_descriptor.release();
        assert_eq!(false, transmit_descriptor.is_acquired());

        transmit_descriptor.set_as_last_segment();
        assert_eq!(true, transmit_descriptor.is_last_segment());
        transmit_descriptor.clear_as_last_segment();
        assert_eq!(false, transmit_descriptor.is_last_segment());

        transmit_descriptor.set_as_first_segment();
        assert_eq!(true, transmit_descriptor.is_first_segment());
        transmit_descriptor.clear_as_first_segment();
        assert_eq!(false, transmit_descriptor.is_first_segment());

        transmit_descriptor.disable_crc();
        assert_eq!(true, transmit_descriptor.is_crc_disabled());
        transmit_descriptor.enable_crc();
        assert_eq!(false, transmit_descriptor.is_crc_disabled());

        transmit_descriptor.disable_pad();
        assert_eq!(true, transmit_descriptor.is_pad_disabled());
        transmit_descriptor.enable_pad();
        assert_eq!(false, transmit_descriptor.is_pad_disabled());

        transmit_descriptor.set_transmit_end_of_ring();
        assert_eq!(true, transmit_descriptor.is_transmit_end_of_ring());
        transmit_descriptor.clear_transmit_end_of_ring();
        assert_eq!(false, transmit_descriptor.is_transmit_end_of_ring());

        assert_eq!(Ok(()), transmit_descriptor.set_buffer1_size(1234));
        assert_eq!(1234, transmit_descriptor.get_buffer1_size());
        assert_eq!(Err(ErrorCode::SIZE), transmit_descriptor.set_buffer1_size(60102));
        assert_eq!(1234, transmit_descriptor.get_buffer1_size());

        let x: u32 = 8;
        transmit_descriptor.set_buffer1_address(&x as *const u32);
        assert_eq!(&x as *const u32 as u32, transmit_descriptor.get_buffer1_address());

        debug!("Finished testing transmit descriptor...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }

    pub fn test_frame_transmission(ethernet: &Ethernet) {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing frame transmission...");
        // Impossible to send a frame while transmission is disabled
        assert_eq!(Err(ErrorCode::OFF), ethernet.send_frame_sync());

        // Enable Ethernet transmission
        assert_eq!(Ok(()), ethernet.enable_transmission());

        // Now, a frame can be send
        assert_eq!(Ok(()), ethernet.send_frame_sync());

        // Disable transmission
        assert_eq!(Ok(()), ethernet.disable_transmission());

        debug!("Finished testing frame transmission...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }

    pub fn run_all(ethernet: &Ethernet) {
        debug!("");
        debug!("================================================");
        debug!("Starting testing the Ethernet...");
        test_mac_address();
        //test_ethernet_init(ethernet);
        //test_ethernet_basic_configuration(ethernet);
        //test_transmit_descriptor();
        //test_frame_transmission(ethernet);
        debug!("================================================");
        debug!("Finished testing the Ethernet. Everything is alright!");
        debug!("");
    }
}
