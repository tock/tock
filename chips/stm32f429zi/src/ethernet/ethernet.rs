// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright 2023 OxidOS Automotive SRL
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! Ethernet firmware for STM32F429ZI
//!
//! # Usage
//!
//! The Ethernet requires that AHB frequency must be at least 25MHz. Default boot configuration
//! sets up a 16MHz clock frequency. Nucleo-F429ZI already contains code to setup the Ethernet.
//! Uncomment code sections marked with ETHERNET to use this peripheral.

use core::cell::Cell;

use crate::ethernet::utils::EthernetSpeed;
use crate::ethernet::utils::MacAddress;
use crate::ethernet::utils::OperationMode;
use cortexm4f::support::nop;
use kernel::hil::ethernet::EthernetAdapterDatapath;
use kernel::hil::ethernet::EthernetAdapterDatapathClient;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::{NumericCellExt, OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::clocks::phclk::{self, PeripheralClock, PeripheralClockType};
use crate::clocks::Stm32f4Clocks;

use crate::ethernet::receive_descriptor::ReceiveDescriptor;
use crate::ethernet::transmit_descriptor::TransmitDescriptor;

register_structs! {
    /// Ethernet: media access control (MAC)
    EthernetMacRegisters {
        /// Ethernet MAC configuration register
        (0x000 => maccr: ReadWrite<u32, MACCR::Register>),
        /// Ethernet MAC frame filter register
        (0x004 => macffr: ReadWrite<u32, MACFFR::Register>),
        /// Ethernet MAC hash table high register
        (0x008 => machthr: ReadWrite<u32>),
        /// Ethernet MAC hash table low register
        (0x00C => machtlr: ReadWrite<u32>),
        /// Ethernet MAC MII address register
        (0x010 => macmiiar: ReadWrite<u32, MACMIIAR::Register>),
        /// Ethernet MAC MII data register
        (0x014 => macmiidr: ReadWrite<u32>),
        /// Ethernet MAC flow control register
        (0x018 => macfcr: ReadWrite<u32, MACFCR::Register>),
        /// Ethernet MAC VLAN tag register
        (0x01C => macvlantr: ReadWrite<u32, MACVLANTR::Register>),
        (0x020 => _reserved0),
        /// Ethernet MAC PMT control and status register
        (0x02C => macpmtcsr: ReadWrite<u32, MACPMTCSR::Register>),
        (0x030 => _reserved1),
        /// Ethernet MAC debug register
        (0x034 => macdbgr: ReadOnly<u32, MACDBGR::Register>),
        /// Ethernet MAC interrupt status register
        (0x038 => macsr: ReadWrite<u32, MACSR::Register>),
        /// Ethernet MAC interrupt mask register
        (0x03C => macimr: ReadWrite<u32, MACIMR::Register>),
        /// Ethernet MAC address 0 high register
        (0x040 => maca0hr: ReadWrite<u32, MACA0HR::Register>),
        /// Ethernet MAC address 0 low register
        (0x044 => maca0lr: ReadWrite<u32>),
        /// Ethernet MAC address 1 high register
        (0x048 => maca1hr: ReadWrite<u32, MACA1HR::Register>),
        /// Ethernet MAC address1 low register
        (0x04C => maca1lr: ReadWrite<u32>),
        /// Ethernet MAC address 2 high register
        (0x050 => maca2hr: ReadWrite<u32, MACA2HR::Register>),
        /// Ethernet MAC address 2 low register
        (0x054 => maca2lr: ReadWrite<u32>),
        /// Ethernet MAC address 3 high register
        (0x058 => maca3hr: ReadWrite<u32, MACA3HR::Register>),
        /// Ethernet MAC address 3 low register
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

const ETHERNET_MAC_BASE: StaticRef<EthernetMacRegisters> =
    unsafe { StaticRef::new(0x40028000 as *const EthernetMacRegisters) };

register_structs! {
    /// Ethernet: DMA controller operation
    EthernetDmaRegisters {
        /// Ethernet DMA bus mode register
        (0x000 => dmabmr: ReadWrite<u32, DMABMR::Register>),
        /// Ethernet DMA transmit poll demand register
        (0x004 => dmatpdr: ReadWrite<u32>),
        /// EHERNET DMA receive poll demand register
        (0x008 => dmarpdr: ReadWrite<u32>),
        /// Ethernet DMA receive descriptor list address register
        (0x00C => dmardlar: ReadWrite<u32>),
        /// Ethernet DMA transmit descriptor list address register
        (0x010 => dmatdlar: ReadWrite<u32>),
        /// Ethernet DMA status register
        (0x014 => dmasr: ReadWrite<u32, DMASR::Register>),
        /// Ethernet DMA operation mode register
        (0x018 => dmaomr: ReadWrite<u32, DMAOMR::Register>),
        /// Ethernet DMA interrupt enable register
        (0x01C => dmaier: ReadWrite<u32, DMAIER::Register>),
        /// Ethernet DMA missed frame and buffer overflow counter register
        (0x020 => dmamfbocr: ReadWrite<u32, DMAMFBOCR::Register>),
        /// Ethernet DMA receive status watchdog timer register
        (0x024 => dmarswtr: ReadWrite<u32>),
        (0x028 => _reserved0),
        /// Ethernet DMA current host transmit descriptor register
        (0x048 => dmachtdr: ReadOnly<u32>),
        /// Ethernet DMA current host receive descriptor register
        (0x04C => dmachrdr: ReadOnly<u32>),
        /// Ethernet DMA current host transmit buffer address register
        (0x050 => dmachtbar: ReadOnly<u32>),
        /// Ethernet DMA current host receive buffer address register
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

const ETHERNET_DMA_BASE: StaticRef<EthernetDmaRegisters> =
    unsafe { StaticRef::new(0x40029000 as *const EthernetDmaRegisters) };

register_structs! {
    /// Ethernet: MAC management counters
    EthernetMmcRegisters {
        /// Ethernet MMC control register
        (0x000 => mmccr: ReadWrite<u32, MMCCR::Register>),
        /// Ethernet MMC receive interrupt register
        (0x004 => mmcrir: ReadWrite<u32, MMCRIR::Register>),
        /// Ethernet MMC transmit interrupt register
        (0x008 => mmctir: ReadOnly<u32, MMCTIR::Register>),
        /// Ethernet MMC receive interrupt mask register
        (0x00C => mmcrimr: ReadWrite<u32, MMCRIMR::Register>),
        /// Ethernet MMC transmit interrupt mask register
        (0x010 => mmctimr: ReadWrite<u32, MMCTIMR::Register>),
        (0x014 => _reserved0),
        /// Ethernet MMC transmitted good frames after a single collision counter
        (0x04C => mmctgfsccr: ReadOnly<u32>),
        /// Ethernet MMC transmitted good frames after more than a single collision
        (0x050 => mmctgfmsccr: ReadOnly<u32>),
        (0x054 => _reserved1),
        /// Ethernet MMC transmitted good frames counter register
        (0x068 => mmctgfcr: ReadOnly<u32>),
        (0x06C => _reserved2),
        /// Ethernet MMC received frames with CRC error counter register
        (0x094 => mmcrfcecr: ReadOnly<u32>),
        /// Ethernet MMC received frames with alignment error counter register
        (0x098 => mmcrfaecr: ReadOnly<u32>),
        (0x09C => _reserved3),
        /// MMC received good unicast frames counter register
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

const ETHERNET_MMC_BASE: StaticRef<EthernetMmcRegisters> =
    unsafe { StaticRef::new(0x40028100 as *const EthernetMmcRegisters) };

#[derive(PartialEq, Debug)]
enum MacTxReaderStatus {
    Idle = 0b00,
    Reading = 0b01,
    WaitingForStatus = 0b10,
    WritingStatusOrFlushing = 0b11,
}

#[derive(PartialEq, Debug)]
enum MacTxWriterStatus {
    Idle = 0b00,
    WaitingForStatusOrBackoff = 0b01,
    GeneratingAndTransmitingPauseFrame = 0b10,
    TransferringInputFrame = 0b11,
}

#[derive(PartialEq, Debug)]
enum RxFifoLevel {
    Empty = 0b00,
    BelowThreshold = 0b01,
    AboveThreshold = 0b10,
    Full = 0b11,
}

#[derive(PartialEq, Debug)]
enum MacRxReaderStatus {
    Idle = 0b00,
    ReadingFrame = 0b01,
    ReadingFrameStatusOrTimeStamp = 0b10,
    FlushingFrameDataAndStatus = 0b11,
}

#[derive(PartialEq, Debug)]
enum DmaTransmitProcessState {
    Stopped = 0b000,
    FetchingTransmitDescriptor = 0b001,
    WaitingForStatus = 0b010,
    ReadingData = 0b011,
    Suspended = 0b110,
    ClosingTransmitDescriptor = 0b111,
}

#[derive(PartialEq, Debug)]
enum DmaReceiveProcessState {
    Stopped = 0b000,
    FetchingReceiveDescriptor = 0b001,
    WaitingForReceivePacket = 0b011,
    Suspended = 0b100,
    ClosingReceiveDescriptor = 0b101,
    TransferringReceivePacketDataToHostMemory = 0b111,
}

#[derive(PartialEq, Debug)]
enum DmaTransmitThreshold {
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
enum DmaReceiveThreshold {
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
    fn new(clocks: &'a dyn Stm32f4Clocks) -> Self {
        Self {
            mac: PeripheralClock::new(PeripheralClockType::AHB1(phclk::HCLK1::ETHMACEN), clocks),
            mac_tx: PeripheralClock::new(
                PeripheralClockType::AHB1(phclk::HCLK1::ETHMACTXEN),
                clocks,
            ),
            mac_rx: PeripheralClock::new(
                PeripheralClockType::AHB1(phclk::HCLK1::ETHMACRXEN),
                clocks,
            ),
            mac_ptp: PeripheralClock::new(
                PeripheralClockType::AHB1(phclk::HCLK1::ETHMACPTPEN),
                clocks,
            ),
        }
    }

    fn enable(&self) {
        self.mac.enable();
        self.mac_rx.enable();
        self.mac_tx.enable();
        self.mac_ptp.enable();
    }
}

/// Ethernet peripheral
pub struct Ethernet<'a> {
    mac_registers: StaticRef<EthernetMacRegisters>,
    _mmc_registers: StaticRef<EthernetMmcRegisters>,
    dma_registers: StaticRef<EthernetDmaRegisters>,
    transmit_descriptor: TransmitDescriptor,
    receive_descriptor: ReceiveDescriptor,
    transmit_packet: TakeCell<'static, [u8]>,
    transmit_packet_length: OptionalCell<u16>,
    packet_identifier: OptionalCell<usize>,
    received_packet: TakeCell<'a, [u8]>,
    number_packets_missed: Cell<usize>,
    client: OptionalCell<&'a dyn EthernetAdapterDatapathClient>,
    clocks: EthernetClocks<'a>,
    mac_address0: Cell<MacAddress>,
}

const DEFAULT_MAC_ADDRESS: MacAddress = MacAddress::new([0xD4, 0x5D, 0x64, 0x62, 0x95, 0x1A]);

impl<'a> Ethernet<'a> {
    /// Ethernet constructor
    pub fn new(clocks: &'a dyn Stm32f4Clocks) -> Self {
        Self {
            mac_registers: ETHERNET_MAC_BASE,
            _mmc_registers: ETHERNET_MMC_BASE,
            dma_registers: ETHERNET_DMA_BASE,
            transmit_descriptor: TransmitDescriptor::new(),
            receive_descriptor: ReceiveDescriptor::new(),
            transmit_packet: TakeCell::empty(),
            transmit_packet_length: OptionalCell::empty(),
            packet_identifier: OptionalCell::empty(),
            received_packet: TakeCell::empty(),
            number_packets_missed: Cell::new(0),
            client: OptionalCell::empty(),
            clocks: EthernetClocks::new(clocks),
            mac_address0: Cell::new(DEFAULT_MAC_ADDRESS),
        }
    }

    /// Initializes the Ethernet peripheral
    pub fn init(&self) -> Result<(), ErrorCode> {
        self.clocks.enable();
        self.init_transmit_descriptors();
        self.init_receive_descriptors();
        self.init_dma()?;
        self.init_mac()?;

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

        // This is safe since both TransmitDescriptor and ReceiveDescriptor are #[repr(C)] structs
        // which only contain in memory registers
        self.set_transmit_descriptor_list_address(
            &self.transmit_descriptor as *const TransmitDescriptor as u32,
        )?;
        self.set_receive_descriptor_list_address(
            &self.receive_descriptor as *const ReceiveDescriptor as u32,
        )?;

        self.enable_all_interrupts();

        Ok(())
    }

    fn init_mac(&self) -> Result<(), ErrorCode> {
        self.disable_mac_watchdog();
        self.set_ethernet_speed(EthernetSpeed::Speed100Mbs);
        self.disable_loopback_mode();
        self.internal_set_operation_mode(OperationMode::FullDuplex);
        self.disable_address_filter();

        Ok(())
    }

    /// Set the received packet buffer for incoming packets
    pub fn set_received_packet_buffer(&self, received_packet_buffer: &'a mut [u8]) {
        self.received_packet.put(Some(received_packet_buffer));
    }

    /* === MAC methods === */

    fn disable_mac_watchdog(&self) {
        self.mac_registers.maccr.modify(MACCR::WD::SET);
    }

    fn set_ethernet_speed(&self, speed: EthernetSpeed) {
        self.mac_registers
            .maccr
            .modify(MACCR::FES.val(speed as u32));
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

    fn internal_is_loopback_mode_enabled(&self) -> bool {
        self.mac_registers.maccr.is_set(MACCR::LM)
    }

    fn internal_set_operation_mode(&self, operation_mode: OperationMode) {
        self.mac_registers
            .maccr
            .modify(MACCR::DM.val(operation_mode as u32));
    }

    fn internal_get_operation_mode(&self) -> OperationMode {
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
        self.mac_registers.macffr.modify(MACFFR::RA::CLEAR);
    }

    fn disable_address_filter(&self) {
        self.mac_registers.macffr.modify(MACFFR::RA::SET);
    }

    fn is_address_filter_enabled(&self) -> bool {
        !self.mac_registers.macffr.is_set(MACFFR::RA)
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
        self.mac_registers
            .maca0hr
            .modify(MACA0HR::MACA0H.val(value as u32));
    }

    fn set_mac_address0_low_register(&self, value: u32) {
        self.mac_registers.maca0lr.set(value);
    }

    fn set_mac_address0(&self, mac_address: MacAddress) {
        let address: u64 = mac_address.into();
        let high_bits = (address >> 32) as u16;
        self.set_mac_address0_high_register(high_bits);
        self.set_mac_address0_low_register((address & 0xFFFFFFFF) as u32);

        self.mac_address0.replace(mac_address);
    }

    fn get_mac_address0(&self) -> MacAddress {
        self.mac_address0.get()
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

        // Arbitrary value. May change in the future if required.
        for _ in 0..100 {
            if !self.dma_registers.dmabmr.is_set(DMABMR::SR) {
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

    // TODO: Am I allowed to clear this bit?
    //fn clear_dma_normal_interrupt(&self) {
    //self.dma_registers.dmasr.modify(DMASR::NIS::SET);
    //}

    fn did_abnormal_interrupt_occur(&self) -> bool {
        self.dma_registers.dmasr.is_set(DMASR::AIS)
    }

    // TODO: Am I allowed to clear this bit?
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

    fn clear_transmit_process_stopped_interrupt(&self) {
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

        // Arbitrary value. May change in the future if required.
        for _ in 0..100 {
            if self.dma_registers.dmaomr.read(DMAOMR::FTF) == 0 {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    fn set_dma_transmission_threshold_control(
        &self,
        threshold: DmaTransmitThreshold,
    ) -> Result<(), ErrorCode> {
        if self.is_dma_transmission_enabled() {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers
            .dmaomr
            .modify(DMAOMR::TTC.val(threshold as u32));

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

    fn set_dma_receive_treshold_control(
        &self,
        threshold: DmaReceiveThreshold,
    ) -> Result<(), ErrorCode> {
        if self.is_dma_reception_enabled() {
            return Err(ErrorCode::FAIL);
        }

        self.dma_registers
            .dmaomr
            .modify(DMAOMR::RTC.val(threshold as u32));

        Ok(())
    }

    fn get_dma_receive_threshold_control(&self) -> DmaReceiveThreshold {
        match self.dma_registers.dmaomr.read(DMAOMR::RTC) {
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

    #[allow(dead_code)]
    // When a standard HIL will be implemented, this method will be used
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

    #[allow(dead_code)]
    // When a standard HIL will be implemented, this method will be used
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

    /* === High-level functions */

    fn is_mac_enabled(&self) -> bool {
        self.is_mac_receiver_enabled() || self.is_mac_transmiter_enabled()
    }

    /// Enables transmit engine
    pub fn enable_transmitter(&self) -> Result<(), ErrorCode> {
        self.enable_dma_transmission()?;
        self.enable_mac_transmitter();

        // Arbitrary value. May change in the future if required.
        for _ in 0..10 {
            if self.get_transmit_process_state() != DmaTransmitProcessState::Stopped {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    #[allow(dead_code)]
    // When a standard HIL will be implemented, this method will be used
    fn disable_transmitter(&self) -> Result<(), ErrorCode> {
        self.disable_dma_transmission()?;
        self.disable_mac_transmitter();

        Ok(())
    }

    fn is_transmission_enabled(&self) -> bool {
        self.is_mac_transmiter_enabled() && self.is_dma_transmission_enabled()
    }

    /// Enables receive engine
    pub fn enable_receiver(&self) -> Result<(), ErrorCode> {
        self.enable_dma_reception()?;
        self.enable_mac_receiver();

        // Arbitrary value. May change in the future if required.
        for _ in 0..10 {
            if self.get_receive_process_state() != DmaReceiveProcessState::Stopped {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    #[allow(dead_code)]
    // When a standard HIL will be implemented, this method will be used
    fn disable_receiver(&self) -> Result<(), ErrorCode> {
        self.disable_dma_reception()?;
        self.disable_mac_receiver();

        Ok(())
    }

    fn is_reception_enabled(&self) -> bool {
        self.is_mac_receiver_enabled() && self.is_dma_reception_enabled()
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
            self.client.map(|client| {
                client.transmit_frame_done(
                    Ok(()),
                    self.transmit_packet.take().unwrap(),
                    self.transmit_packet_length.take().unwrap(),
                    self.packet_identifier.take().unwrap(),
                    None,
                )
            });
            self.clear_transmit_interrupt();
        } else if self.did_transmit_buffer_unavailable_interrupt_occur() {
            self.clear_transmit_buffer_unavailable_interrupt();
        } else if self.did_receive_interrupt_occur() {
            self.client.map(|client| {
                let received_packet = self.received_packet.take().unwrap();
                client.received_frame(received_packet, None);
                self.received_packet.put(Some(received_packet));
            });
            // Receive the following packet
            assert_eq!(Ok(()), self.receive_packet());
            self.clear_receive_interrupt();
        } else if self.did_early_receive_interrupt_occur() {
            self.clear_early_receive_interrupt();
        }
    }

    fn handle_abnormal_interrupt(&self) {
        if self.did_fatal_bus_error_interrupt_occur() {
            self.clear_fatal_bus_error_interrupt();
            panic!("Fatal bus error");
        } else if self.did_early_transmit_interrupt_occur() {
            self.clear_early_transmit_interrupt();
        } else if self.did_receive_watchdog_timeout_interrupt_occur() {
            self.clear_receive_watchdog_timeout_interrupt();
            // Watchdog is disabled by default, so this situation will never arrive in practice.
            // Additionally, to receive very large frames, receive must be configured in Threshold
            // mode which for now is not the case.
        } else if self.did_receive_process_stopped_interrupt_occur() {
            self.clear_receive_process_stopped_interrupt();
            // The receive process can't be normally stopped since the current HIL doesn't expose
            // this functionality to the capsule. The following line is added just in case.
            self.enable_receiver().unwrap();
        } else if self.did_receive_buffer_unavailable_interrupt_occur() {
            self.clear_receive_buffer_unavailable_interrupt();
        } else if self.did_transmit_buffer_underflow_interrupt_occur() {
            self.clear_transmit_buffer_underflow_interrupt();
            self.client.map(|client| {
                client.transmit_frame_done(
                    // TODO: Does FAIL describe the error the best?
                    Err(ErrorCode::FAIL),
                    self.transmit_packet.take().unwrap(),
                    self.transmit_packet_length.take().unwrap(),
                    self.packet_identifier.take().unwrap(),
                    None,
                )
            });
        } else if self.did_receive_fifo_overflow_interrupt_occur() {
            self.clear_receive_fifo_overflow_interrupt();
            self.number_packets_missed.add(1);
            assert_eq!(Ok(()), self.receive_packet());
        } else if self.did_transmit_jabber_timeout_interrupt_occur() {
            self.clear_transmit_jabber_timeout_interrupt();
            // When a buffer jabber timeout occurs, the transmit engine is stopped. Attempt to
            // restart it. If it fails, just panic, since the current HIL doesn't allow to
            // communicate to the capsule that the peripheral stopped
            self.enable_transmitter().unwrap();
        } else if self.did_transmit_process_stopped_interrupt_occur() {
            self.clear_transmit_process_stopped_interrupt();
            // The transmit process can't be normally stopped since the current HIL doesn't expose
            // this functionality to the capsule. The following line is added just in case.
            self.enable_transmitter().unwrap();
        }
    }

    pub(crate) fn handle_interrupt(&self) {
        if self.did_normal_interrupt_occur() {
            self.handle_normal_interrupt();
        } else if self.did_abnormal_interrupt_occur() {
            self.handle_abnormal_interrupt();
        }
    }

    /// Schedule the reception of an incoming packet
    ///
    /// # Errors
    ///
    /// + [Err]\([ErrorCode::OFF]\): the reception of the Ethernet peripheral is off
    ///
    /// # Panics
    ///
    /// This method panics if no receive packet buffer has been set
    pub fn receive_packet(&self) -> Result<(), ErrorCode> {
        // Check if DMA and MAC core are enabled
        if !self.is_reception_enabled() {
            return Err(ErrorCode::OFF);
        }

        let received_packet = self.received_packet.take().unwrap();
        self.receive_descriptor
            .set_buffer1_address(received_packet.as_ptr() as u32);
        self.receive_descriptor
            .set_buffer1_size(received_packet.len())?;
        self.receive_descriptor.set_buffer2_size(0)?;
        self.received_packet.put(Some(received_packet));

        // Acquire the receive descriptor
        self.receive_descriptor.acquire();

        // Wait 4 CPU cycles until everything is written to the RAM
        for _ in 0..4 {
            nop();
        }

        self.dma_receive_poll_demand();

        Ok(())
    }
}

impl<'a> EthernetAdapterDatapath<'a> for Ethernet<'a> {
    fn set_client(&self, client: &'a dyn EthernetAdapterDatapathClient) {
        self.client.set(client);
    }

    fn enable_receive(&self) {
        // TODO: the underlying implementation can panic -- should should return
        // a Result!
        self.enable_receiver()
            .expect("Failed to enable Ethernet receive path!");
        self.receive_packet()
            .expect("Failed to start receiving a packet!");
    }

    fn disable_receive(&self) {
        // TODO: the underlying implementation can panic -- should should return
        // a Result!
        self.disable_receiver()
            .expect("Failed to disable Ethernet receive path!");
    }

    fn transmit_frame(
        &self,
        packet: &'static mut [u8],
        len: u16,
        packet_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Check if DMA and MAC core are enabled
        if !self.is_transmission_enabled() {
            return Err((ErrorCode::OFF, packet));
        }

        // Check if transmitter is busy
        if self.get_transmit_process_state() != DmaTransmitProcessState::Suspended
            || self.transmit_packet.is_some()
        {
            return Err((ErrorCode::BUSY, packet));
        }

        // Prepare transmit descriptor
        self.transmit_descriptor
            .set_buffer1_address(packet.as_ptr() as u32);
        if let Err(ErrorCode::SIZE) = self.transmit_descriptor.set_buffer1_size(len as usize) {
            return Err((ErrorCode::SIZE, packet));
        }
        if let Err(ErrorCode::SIZE) = self.transmit_descriptor.set_buffer2_size(0) {
            return Err((ErrorCode::SIZE, packet));
        }

        // Store the transmit packet
        self.transmit_packet.replace(packet);

        // Store the length of the buffer
        self.transmit_packet_length.set(len);

        // Store the packet identifier
        self.packet_identifier.set(packet_identifier);

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
}

/// Tests for the Ethernet peripheral
///
/// This module provides tests for the Ethernet peripheral. If any changes are brought to it,
/// make sure to run these tests to ensure that there is no regression.
///
/// # Usage
///
/// Add the following line before kernel's main loop:
///
/// ```rust,ignore
/// stm32f429zi::ethernet::tests::run_all_unit_tests(&peripherals.ethernet);
/// ```
///
/// If there are no errors, the following output should be printed on the console:
///
/// ```text,ignore
/// ================================================
/// Starting testing the Ethernet...
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Testing Ethernet initialization...
/// Finished testing Ethernet initialization
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Testing Ethernet basic configuration...
/// Finished testing Ethernet basic configuration...
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Testing frame transmission...
/// Finished testing frame transmission...
/// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Finished testing the Ethernet. Everything is alright!
/// ================================================
/// ```
///
/// # Errors
///
/// If there are any errors, open an issue ticket at <https://github.com/tock/tock>. Please provide
/// the output of the test execution.
pub mod tests {
    use super::*;
    use kernel::debug;

    fn test_mac_default_values(ethernet: &Ethernet) {
        assert_eq!(EthernetSpeed::Speed100Mbs, ethernet.get_ethernet_speed());
        assert_eq!(false, ethernet.internal_is_loopback_mode_enabled());
        assert_eq!(
            OperationMode::FullDuplex,
            ethernet.internal_get_operation_mode()
        );
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
        assert_eq!(DEFAULT_MAC_ADDRESS, ethernet.get_mac_address0());
        assert_eq!(false, ethernet.is_mac_address1_enabled());
        assert_eq!(false, ethernet.is_mac_address2_enabled());
        assert_eq!(false, ethernet.is_mac_address3_enabled());
    }

    fn test_dma_default_values(ethernet: &Ethernet) {
        assert_eq!(
            &ethernet.transmit_descriptor as *const TransmitDescriptor as u32,
            ethernet.get_transmit_descriptor_list_address()
        );
        assert_eq!(
            DmaTransmitProcessState::Stopped,
            ethernet.get_transmit_process_state()
        );
        assert_eq!(true, ethernet.is_transmit_store_and_forward_enabled());
        assert_eq!(false, ethernet.is_dma_transmission_enabled());
        assert_eq!(
            DmaTransmitThreshold::Threshold64,
            ethernet.get_dma_transmission_threshold_control()
        );

        assert_eq!(
            &ethernet.receive_descriptor as *const ReceiveDescriptor as u32,
            ethernet.get_receive_descriptor_list_address()
        );
        assert_eq!(true, ethernet.is_receive_store_and_forward_enabled());
        assert_eq!(false, ethernet.is_dma_reception_enabled());
        assert_eq!(
            DmaReceiveThreshold::Threshold64,
            ethernet.get_dma_receive_threshold_control()
        );

        assert_eq!(false, ethernet.did_normal_interrupt_occur());
        assert_eq!(false, ethernet.did_abnormal_interrupt_occur());
    }

    /// Test Ethernet initialization
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

        assert_eq!(
            Ok(()),
            ethernet.set_transmit_descriptor_list_address(0x12345)
        );
        // The last two bits are ignore since the bus width is 32 bits
        assert_eq!(0x12344, ethernet.get_transmit_descriptor_list_address());

        assert_eq!(Ok(()), ethernet.enable_transmit_store_and_forward());
        assert_eq!(true, ethernet.is_transmit_store_and_forward_enabled());
        assert_eq!(Ok(()), ethernet.disable_transmit_store_and_forward());
        assert_eq!(false, ethernet.is_transmit_store_and_forward_enabled());

        assert_eq!(
            Ok(()),
            ethernet.set_dma_transmission_threshold_control(DmaTransmitThreshold::Threshold192)
        );
        assert_eq!(
            DmaTransmitThreshold::Threshold192,
            ethernet.get_dma_transmission_threshold_control()
        );
        assert_eq!(
            Ok(()),
            ethernet.set_dma_transmission_threshold_control(DmaTransmitThreshold::Threshold32)
        );
        assert_eq!(
            DmaTransmitThreshold::Threshold32,
            ethernet.get_dma_transmission_threshold_control()
        );
        assert_eq!(
            Ok(()),
            ethernet.set_dma_transmission_threshold_control(DmaTransmitThreshold::Threshold64)
        );
        assert_eq!(
            DmaTransmitThreshold::Threshold64,
            ethernet.get_dma_transmission_threshold_control()
        );

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

        assert_eq!(
            Ok(()),
            ethernet.set_receive_descriptor_list_address(0x12345)
        );
        assert_eq!(0x12344, ethernet.get_receive_descriptor_list_address());

        assert_eq!(Ok(()), ethernet.enable_receive_store_and_forward());
        assert_eq!(true, ethernet.is_receive_store_and_forward_enabled());
        assert_eq!(Ok(()), ethernet.disable_receive_store_and_forward());
        assert_eq!(false, ethernet.is_receive_store_and_forward_enabled());

        assert_eq!(
            Ok(()),
            ethernet.set_dma_receive_treshold_control(DmaReceiveThreshold::Threshold32)
        );
        assert_eq!(
            DmaReceiveThreshold::Threshold32,
            ethernet.get_dma_receive_threshold_control()
        );
        assert_eq!(
            Ok(()),
            ethernet.set_dma_receive_treshold_control(DmaReceiveThreshold::Threshold128)
        );
        assert_eq!(
            DmaReceiveThreshold::Threshold128,
            ethernet.get_dma_receive_threshold_control()
        );
        assert_eq!(
            Ok(()),
            ethernet.set_dma_receive_treshold_control(DmaReceiveThreshold::Threshold64)
        );
        assert_eq!(
            DmaReceiveThreshold::Threshold64,
            ethernet.get_dma_receive_threshold_control()
        );
    }

    /// Test Ethernet basic configuration
    pub fn test_ethernet_basic_configuration(ethernet: &Ethernet) {
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing Ethernet basic configuration...");

        assert_eq!(Ok(()), ethernet.init());

        ethernet.set_ethernet_speed(EthernetSpeed::Speed100Mbs);
        assert_eq!(EthernetSpeed::Speed100Mbs, ethernet.get_ethernet_speed());
        ethernet.set_ethernet_speed(EthernetSpeed::Speed10Mbs);
        assert_eq!(EthernetSpeed::Speed10Mbs, ethernet.get_ethernet_speed());

        ethernet.enable_loopback_mode();
        assert_eq!(true, ethernet.internal_is_loopback_mode_enabled());
        ethernet.disable_loopback_mode();
        assert_eq!(false, ethernet.internal_is_loopback_mode_enabled());

        ethernet.internal_set_operation_mode(OperationMode::FullDuplex);
        assert_eq!(
            OperationMode::FullDuplex,
            ethernet.internal_get_operation_mode()
        );
        ethernet.internal_set_operation_mode(OperationMode::HalfDuplex);
        assert_eq!(
            OperationMode::HalfDuplex,
            ethernet.internal_get_operation_mode()
        );

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
        assert_eq!(
            MacAddress::from(0x112233445566),
            ethernet.get_mac_address0()
        );
        ethernet.set_mac_address0(DEFAULT_MAC_ADDRESS);
        assert_eq!(DEFAULT_MAC_ADDRESS, ethernet.get_mac_address0());

        ethernet.enable_normal_interrupts();
        assert_eq!(true, ethernet.are_normal_interrupts_enabled());
        ethernet.disable_normal_interrupts();
        assert_eq!(false, ethernet.are_normal_interrupts_enabled());

        test_ethernet_transmission_configuration(ethernet);
        test_ethernet_reception_configuration(ethernet);

        assert_eq!(Ok(()), ethernet.enable_transmitter());
        assert_eq!(true, ethernet.is_mac_transmiter_enabled());
        assert_eq!(Ok(()), ethernet.disable_transmitter());
        assert_eq!(false, ethernet.is_mac_transmiter_enabled());

        assert_eq!(Ok(()), ethernet.enable_receiver());
        assert_eq!(true, ethernet.is_mac_transmiter_enabled());
        assert_eq!(Ok(()), ethernet.disable_receiver());
        assert_eq!(false, ethernet.is_mac_transmiter_enabled());

        assert_eq!(Ok(()), ethernet.enable_transmitter());
        assert_eq!(Ok(()), ethernet.enable_receiver());
        assert_eq!(true, ethernet.is_mac_enabled());

        assert_eq!(Ok(()), ethernet.disable_transmitter());
        assert_eq!(true, ethernet.is_mac_enabled());

        assert_eq!(Ok(()), ethernet.enable_transmitter());
        assert_eq!(Ok(()), ethernet.disable_receiver());
        assert_eq!(true, ethernet.is_mac_enabled());

        assert_eq!(Ok(()), ethernet.disable_transmitter());
        assert_eq!(false, ethernet.is_mac_enabled());

        // Restore Ethernet to its initial state
        assert_eq!(Ok(()), ethernet.init());

        debug!("Finished testing Ethernet basic configuration...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    }

    /// Test Ethernet interrupt configuration
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
        assert_eq!(
            true,
            ethernet.is_transmit_buffer_unavailable_interrupt_enabled()
        );
        ethernet.disable_transmit_buffer_unavailable_interrupt();
        assert_eq!(
            false,
            ethernet.is_transmit_buffer_unavailable_interrupt_enabled()
        );

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
        assert_eq!(
            true,
            ethernet.is_receive_watchdog_timeout_interrupt_enabled()
        );
        ethernet.disable_receive_watchdog_timeout_interrupt();
        assert_eq!(
            false,
            ethernet.is_receive_watchdog_timeout_interrupt_enabled()
        );

        ethernet.enable_receive_process_stopped_interrupt();
        assert_eq!(
            true,
            ethernet.is_receive_process_stopped_interrupt_enabled()
        );
        ethernet.disable_receive_process_stopped_interrupt();
        assert_eq!(
            false,
            ethernet.is_receive_process_stopped_interrupt_enabled()
        );

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

    /// Run all unit tests
    pub fn run_all_unit_tests(ethernet: &Ethernet) {
        debug!("");
        debug!("================================================");
        debug!("Starting testing the Ethernet...");
        test_ethernet_init(ethernet);
        test_ethernet_basic_configuration(ethernet);
        test_ethernet_interrupts(ethernet);
        debug!("Finished testing the Ethernet. Everything is alright!");
        debug!("================================================");
        debug!("");
    }
}
