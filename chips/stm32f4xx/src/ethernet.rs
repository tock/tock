use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub Registers {
        // MAC
        (0x0000 => eth_maccr: ReadWrite<u32, ETH_MACCR::Register>),
        (0x0004 => eth_macffr: ReadWrite<u32, ETH_MACFFR::Register>),
        (0x0008 => eth_machthr: ReadWrite<u32, ETH_MACHTHR::Register>),
        (0x000c => eth_machtlr: ReadWrite<u32, ETH_MACHTLR::Register>),
        (0x0010 => eth_macmiiar: ReadWrite<u32, ETH_MACMIIAR::Register>),
        (0x0014 => eth_macmiidr: ReadWrite<u32, ETH_MACMIIDR::Register>),
        (0x0018 => eth_macfcr: ReadWrite<u32, ETH_MACFCR::Register>),
        (0x001c => eth_macvlantr: ReadWrite<u32, ETH_MACVLANTR::Register>),

        (0x0020 => _reserved0),

        (0x0028 => eth_macrwuffr: ReadWrite<u32, ETH_MACRWUFFR::Register>),
        (0x002c => eth_macpmtcsr: ReadWrite<u32, ETH_MACPMTCSR::Register>),

        (0x0030 => _reserved1),

        (0x0034 => eth_macdbgr: ReadWrite<u32, ETH_MACDGBR::Register>),
        (0x0038 => eth_macsr: ReadWrite<u32, ETH_MACSR::Register>),
        (0x003c => eth_macimr: ReadWrite<u32, ETH_MACIMR::Register>),
        (0x0040 => eth_maca0hr: ReadWrite<u32, ETH_MACA0HR::Register>),
        (0x0044 => eth_maca0lr: ReadWrite<u32, ETH_MACA0LR::Register>),
        (0x0048 => eth_maca1hr: ReadWrite<u32, ETH_MACA1HR::Register>),
        (0x004c => eth_maca1lr: ReadWrite<u32, ETH_MACA1LR::Register>),
        (0x0050 => eth_maca2hr: ReadWrite<u32, ETH_MACA2HR::Register>),
        (0x0054 => eth_maca2lr: ReadWrite<u32, ETH_MACA2LR::Register>),
        (0x0058 => eth_maca3hr: ReadWrite<u32, ETH_MACA3HR::Register>),
        (0x005c => eth_maca3lr: ReadWrite<u32, ETH_MACA3LR::Register>),

        (0x0060 => _reserved2),

        // MMC
        (0x0100 => eth_mmccr: ReadWrite<u32, ETH_MMCCR::Register>),
        (0x0104 => eth_mmcrir: ReadWrite<u32, ETH_MMCRIR::Register>),
        (0x0108 => eth_mmctir: ReadWrite<u32, ETH_MMCTIR::Register>),
        (0x010c => eth_mmcrimr: ReadWrite<u32, ETH_MMCRIMR::Register>),
        (0x0110 => eth_mmctimr: ReadWrite<u32, ETH_MMCTIMR::Register>),

        (0x0114 => _reserved3),

        (0x014c => eth_mmctgfsccr: ReadWrite<u32, ETH_MMCTGFSCCR::Register>),
        (0x0150 => eth_mmctgfmsccr: ReadWrite<u32, ETH_MMCTGFMSCCR::Register>),

        (0x0154 => _reserved4),

        (0x0168 => eth_mmctgfcr: ReadWrite<u32, ETH_MMCTGFCR::Register>),

        (0x016c => _reserved5),

        (0x0194 => eth_mmcrfcecr: ReadWrite<u32, ETH_MMCRFCECR::Register>),
        (0x0198 => eth_mmcrfaecr: ReadWrite<u32, ETH_MMCRFAECR::Register>),

        (0x019c => _reserved6),

        (0x01c4 => eth_mmcrgufcr: ReadWrite<u32, ETH_MMCRGUFCR::Register>),

        (0x01c8 => _reserved7),

        // PTP
        (0x0700 => eth_ptptscr: ReadWrite<u32, ETH_PTPTSCR::Register>),
        (0x0704 => eth_ptpssir: ReadWrite<u32, ETH_PTPSSIR::Register>),
        (0x0708 => eth_ptptshr: ReadWrite<u32, ETH_PTPTSHR::Register>),
        (0x070c => eth_ptptslr: ReadWrite<u32, ETH_PTPTSLR::Register>),
        (0x0710 => eth_ptptshur: ReadWrite<u32, ETH_PTPTSHUR::Register>),
        (0x0714 => eth_ptptslur: ReadWrite<u32, ETH_PTPTSLUR::Register>),
        (0x0718 => eth_ptptsar: ReadWrite<u32, ETH_PTPTSAR::Register>),
        (0x071c => eth_ptptthr: ReadWrite<u32, ETH_PTPTTHR::Register>),
        (0x0720 => eth_ptpttlr: ReadWrite<u32, ETH_PTPTTLR::Register>),

        (0x0724 => _reserved8),

        (0x0728 => eth_ptptssr: ReadWrite<u32, ETH_PTPTSSR::Register>),
        (0x072c => eth_ptpppscr: ReadWrite<u32, ETH_PTPPPSCR::Register>),

        (0x0730 => _reserved9),

        // DMA
        (0x1000 => eth_dmabmr: ReadWrite<u32, ETH_DMABMR::Register>),
        (0x1004 => eth_dmatpdr: ReadWrite<u32, ETH_DMATPDR::Register>),
        (0x1008 => eth_dmarpdr: ReadWrite<u32, ETH_DMARPDR::Register>),
        (0x100c => eth_dmardlar: ReadWrite<u32, ETH_DMARDLAR::Register>),
        (0x1010 => eth_dmatdlar: ReadWrite<u32, ETH_DMATDLAR::Register>),
        (0x1014 => eth_dmasr: ReadWrite<u32, ETH_DMASR::Register>),
        (0x1018 => eth_dmaomr: ReadWrite<u32, ETH_DMAOMR::Register>),
        (0x101c => eth_dmaier: ReadWrite<u32, ETH_DMAIER::Register>),
        (0x1020 => eth_dmamfbocr: ReadWrite<u32, ETH_DMAMFBOCR::Register>),
        (0x1024 => eth_dmarswtr: ReadWrite<u32, ETH_DMARSWTR::Register>),

        (0x1028 => _reserved10),

        (0x1048 => eth_dmachtdr: ReadWrite<u32, ETH_DMACHTDR::Register>),
        (0x104c => eth_dmachrdr: ReadWrite<u32, ETH_DMACHRDR::Register>),
        (0x1050 => eth_dmachtbar: ReadWrite<u32, ETH_DMACHTBAR::Register>),
        (0x1054 => eth_dmachrbar: ReadWrite<u32, ETH_DMACHRBAR::Register>),

        (0x1058 => @END),
    }
}

register_bitfields![u32,
    ETH_MACCR [
        CSTF OFFSET(25) NUMBITS(1) [],
        WD OFFSET(23) NUMBITS(1) [],
        JD OFFSET(22) NUMBITS(1) [],
        IFG OFFSET(17) NUMBITS(3) [],
        CSD OFFSET(16) NUMBITS(1) [],
        FES OFFSET(14) NUMBITS(1) [],
        ROD OFFSET(13) NUMBITS(1) [],
        LM OFFSET(12) NUMBITS(1) [],
        DM OFFSET(11) NUMBITS(1) [],
        IPCO OFFSET(10) NUMBITS(1) [],
        RD OFFSET(9) NUMBITS(1) [],
        APCS OFFSET(7) NUMBITS(1) [],
        BL OFFSET(5) NUMBITS(2) [],
        DC OFFSET(4) NUMBITS(1) [],
        TE OFFSET(3) NUMBITS(1) [],
        RE OFFSET(2) NUMBITS(1) []
    ],
    ETH_MACFFR [
        RA OFFSET(31) NUMBITS(1) [],
        HPF OFFSET(10) NUMBITS(1) [],
        SAF OFFSET(9) NUMBITS(1) [],
        SAIF OFFSET(8) NUMBITS(1) [],
        PCF OFFSET(6) NUMBITS(2) [],
        BFD OFFSET(5) NUMBITS(1) [],
        PAM OFFSET(4) NUMBITS(1) [],
        DAIF OFFSET(3) NUMBITS(1) [],
        HM OFFSET(2) NUMBITS(1) [],
        HU OFFSET(1) NUMBITS(1) [],
        PM OFFSET(0) NUMBITS(1) []
    ],
    ETH_MACHTHR [
        HTH OFFSET(0) NUMBITS(32) []
    ],
    ETH_MACHTLR [
        HTL OFFSET(0) NUMBITS(32) []
    ],
    ETH_MACMIIAR [
        PA OFFSET(11) NUMBITS(5) [],
        MR OFFSET(6) NUMBITS(5) [],
        CR OFFSET(2) NUMBITS(4) [],
        MW OFFSET(1) NUMBITS(1) [],
        MB OFFSET(0) NUMBITS(1) []
    ],
    ETH_MACMIIDR [
        MD OFFSET(0) NUMBITS(16) []
    ],
    ETH_MACFCR [
        PT OFFSET(16) NUMBITS(16) [],
        ZQPD OFFSET(7) NUMBITS(1) [],
        PLT OFFSET(4) NUMBITS(2) [],
        UPFD OFFSET(3) NUMBITS(1) [],
        RFCE OFFSET(2) NUMBITS(1) [],
        TFCE OFFSET(1) NUMBITS(1) [],
        // al3xTODO double meaning bit notation
        FCB_BPA OFFSET(0) NUMBITS(1) []
    ],
    ETH_MACVLANTR [
        VLANTC OFFSET(16) NUMBITS(1) [],
        VLANTI OFFSET(0) NUMBITS(16) []
    ],
    ETH_MACRWUFFR [
        // al3xTODO can delete if bit mapping is empty?
    ],
    ETH_MACPMTCSR [
        WFFRPR OFFSET(31) NUMBITS(1) [],
        GU OFFSET(9) NUMBITS(1) [],
        WFR OFFSET(6) NUMBITS(1) [],
        MPR OFFSET(5) NUMBITS(1) [],
        WFE OFFSET(2) NUMBITS(1) [],
        MPE OFFSET(1) NUMBITS(1) [],
        PD OFFSET(0) NUMBITS(1) []
    ],
    ETH_MACDGBR [
        TFF OFFSET(25) NUMBITS(1) [],
        TFNEGU OFFSET(24) NUMBITS(1) [],
        TFWA OFFSET(22) NUMBITS(1) [],
        TFRS OFFSET(20) NUMBITS(2) [],
        MTP OFFSET(19) NUMBITS(1) [],
        MTFCS OFFSET(17) NUMBITS(2) [],
        MMTEA OFFSET(16) NUMBITS(1) [],
        RFFL OFFSET(8) NUMBITS(2) [],
        RFRCS OFFSET(5) NUMBITS(2) [],
        RFWRA OFFSET(4) NUMBITS(1) [],
        MSFRWCS OFFSET(1) NUMBITS(2) [],
        MMRPEA OFFSET(0) NUMBITS(1) []
    ],
    ETH_MACSR [
        TSTS OFFSET(9) NUMBITS(1) [],
        MMCTS OFFSET(6) NUMBITS(1) [],
        MMCRS OFFSET(5) NUMBITS(1) [],
        MMCS OFFSET(4) NUMBITS(1) [],
        PMTS OFFSET(3) NUMBITS(1) []
    ],
    ETH_MACIMR [
        TSTIM OFFSET(9) NUMBITS(1) [],
        PMTIM OFFSET(3) NUMBITS(1) []
    ],
    ETH_MACA0HR [
        MO OFFSET(31) NUMBITS(1) [],
        MACA0H OFFSET(0) NUMBITS(16) []
    ],
    ETH_MACA0LR [
        MACA0L OFFSET(0) NUMBITS(32) []
    ],
    ETH_MACA1HR [
        AE OFFSET(31) NUMBITS(1) [],
        SA OFFSET(30) NUMBITS(1) [],
        MBC OFFSET(24) NUMBITS(6) [],
        MACA1H OFFSET(0) NUMBITS(16) []
    ],
    ETH_MACA1LR [
        MACA1L OFFSET(0) NUMBITS(32) []
    ],
    ETH_MACA2HR [
        AE OFFSET(31) NUMBITS(1) [],
        SA OFFSET(30) NUMBITS(1) [],
        MBC OFFSET(24) NUMBITS(6) [],
        MACA2H OFFSET(0) NUMBITS(16) []
    ],
    ETH_MACA2LR [
        MACA2L OFFSET(0) NUMBITS(32) []
    ],
    ETH_MACA3HR [
        AE OFFSET(31) NUMBITS(1) [],
        SA OFFSET(30) NUMBITS(1) [],
        MBC OFFSET(24) NUMBITS(6) [],
        MACA2H OFFSET(0) NUMBITS(16) []
    ],
    ETH_MACA3LR [
        MACA3L OFFSET(0) NUMBITS(32) []
    ],

    ETH_MMCCR [
        MCFHP OFFSET(5) NUMBITS(1) [],
        MCP OFFSET(4) NUMBITS(1) [],
        MCF OFFSET(3) NUMBITS(1) [],
        ROR OFFSET(2) NUMBITS(1) [],
        CSR OFFSET(1) NUMBITS(1) [],
        CR OFFSET(0) NUMBITS(1) []
    ],
    ETH_MMCRIR [
        RGUFS OFFSET(17) NUMBITS(1) [],
        RFAES OFFSET(6) NUMBITS(1) [],
        RFCES OFFSET(5) NUMBITS(1) []
    ],
    ETH_MMCTIR [
        TGFS OFFSET(21) NUMBITS(1) [],
        TGFMSCS OFFSET(15) NUMBITS(1) [],
        TGFSCS OFFSET(14) NUMBITS(1) []
    ],
    ETH_MMCRIMR [
        RGUFM OFFSET(17) NUMBITS(1) [],
        RFAEM OFFSET(6) NUMBITS(1) [],
        RFCEM OFFSET(5) NUMBITS(1) []
    ],
    ETH_MMCTIMR [
        TGFM OFFSET(21) NUMBITS(1) [],
        TGFMSCM OFFSET(15) NUMBITS(1) [],
        TGFSCM OFFSET(14) NUMBITS(1) []
    ],
    ETH_MMCTGFSCCR [
        TGFSCC OFFSET(0) NUMBITS(32) []
    ],
    ETH_MMCTGFMSCCR [
        TGFMSCC OFFSET(0) NUMBITS(32) []
    ],
    ETH_MMCTGFCR [
        TGFC OFFSET(0) NUMBITS(32) []
    ],
    ETH_MMCRFCECR [
        RFCEC OFFSET(0) NUMBITS(32) []
    ],
    ETH_MMCRFAECR [
        RFAEC OFFSET(0) NUMBITS(32) []
    ],
    ETH_MMCRGUFCR [
        RGUFC OFFSET(0) NUMBITS(32) []
    ],

    ETH_PTPTSCR [
        TSPFFMAE OFFSET(18) NUMBITS(1) [],
        TSCNT OFFSET(16) NUMBITS(2) [],
        TSSMRME OFFSET(15) NUMBITS(1) [],
        TSSEME OFFSET(14) NUMBITS(1) [],
        TSSIPV4FE OFFSET(13) NUMBITS(1) [],
        TSSIPV6FE OFFSET(12) NUMBITS(1) [],
        TSSPTPOEFE OFFSET(11) NUMBITS(1) [],
        TSPTPPSV2E OFFSET(10) NUMBITS(1) [],
        TSSSR OFFSET(9) NUMBITS(1) [],
        TSSARFE OFFSET(8) NUMBITS(1) [],
        TTSARU OFFSET(5) NUMBITS(1) [],
        TSITETSSTU OFFSET(3) NUMBITS(1) [],
        TSSTI OFFSET(2) NUMBITS(1) [],
        TSFCU OFFSET(1) NUMBITS(1) [],
        TSE OFFSET(0) NUMBITS(1) []
    ],
    ETH_PTPSSIR [
        STSSI OFFSET(0) NUMBITS(8) []
    ],
    ETH_PTPTSHR [
        STS OFFSET(0) NUMBITS(32) []
    ],
    ETH_PTPTSLR [
        STPNS OFFSET(31) NUMBITS(1) [],
        STSS OFFSET(0) NUMBITS(31) []
    ],
    ETH_PTPTSHUR [
        TSUS OFFSET(0) NUMBITS(32) []
    ],
    ETH_PTPTSLUR [
        TSUPNS OFFSET(31) NUMBITS(1) [],
        TSUSS OFFSET(0) NUMBITS(31) []
    ],
    ETH_PTPTSAR [
        TSA OFFSET(0) NUMBITS(32) []
    ],
    ETH_PTPTTHR [
        TTSH OFFSET(0) NUMBITS(32) []
    ],
    ETH_PTPTTLR [
        TTSL OFFSET(0) NUMBITS(32) []
    ],
    ETH_PTPTSSR [
        TSTTR OFFSET(1) NUMBITS(1) [],
        TSSO OFFSET(0) NUMBITS(1) []
    ],
    ETH_PTPPPSCR [
        PPSFREQ OFFSET(0) NUMBITS(3) []
    ],


    ETH_DMABMR [
        MB OFFSET(26) NUMBITS(1) [],
        AAB OFFSET(25) NUMBITS(1) [],
        FPM OFFSET(24) NUMBITS(1) [],
        USP OFFSET(23) NUMBITS(1) [],
        RDP OFFSET(17) NUMBITS(6) [],
        FB OFFSET(16) NUMBITS(1) [],
        PM OFFSET(14) NUMBITS(2) [],
        PBL OFFSET(8) NUMBITS(6) [],
        EDFE OFFSET(7) NUMBITS(1) [],
        DSL OFFSET(2) NUMBITS(5) [],
        DA OFFSET(1) NUMBITS(1) [],
        SR OFFSET(0) NUMBITS(1) []
    ],
    ETH_DMATPDR [
        TTPD OFFSET(0) NUMBITS(32) []
    ],
    ETH_DMARPDR [
        RPD OFFSET(0) NUMBITS(32) []
    ],
    ETH_DMARDLAR [
        SRL OFFSET(0) NUMBITS(32) []
    ],
    ETH_DMATDLAR [
        STL OFFSET(0) NUMBITS(32) []
    ],
    ETH_DMASR [
        TSTS OFFSET(29) NUMBITS(1) [],
        PMTS OFFSET(28) NUMBITS(1) [],
        MMCS OFFSET(27) NUMBITS(1) [],
        EBS OFFSET(23) NUMBITS(3) [],
        TPS OFFSET(20) NUMBITS(3) [],
        RPS OFFSET(17) NUMBITS(3) [],
        NIS OFFSET(16) NUMBITS(1) [],
        AIS OFFSET(15) NUMBITS(1) [],
        ERS OFFSET(14) NUMBITS(1) [],
        FBES OFFSET(13) NUMBITS(1) [],
        ETS OFFSET(10) NUMBITS(1) [],
        RWTS OFFSET(9) NUMBITS(1) [],
        RPSS OFFSET(8) NUMBITS(1) [],
        RBUS OFFSET(7) NUMBITS(1) [],
        RS OFFSET(6) NUMBITS(1) [],
        TUS OFFSET(5) NUMBITS(1) [],
        ROS OFFSET(4) NUMBITS(1) [],
        TJTS OFFSET(3) NUMBITS(1) [],
        TBUS OFFSET(2) NUMBITS(1) [],
        TPSS OFFSET(1) NUMBITS(1) [],
        TS OFFSET(0) NUMBITS(1) []
    ],
    ETH_DMAOMR [
        DTCEFD OFFSET(26) NUMBITS(1) [],
        RSF OFFSET(25) NUMBITS(1) [],
        DFRF OFFSET(24) NUMBITS(1) [],
        TSF OFFSET(21) NUMBITS(1) [],
        FTF OFFSET(20) NUMBITS(1) [],
        TTC OFFSET(14) NUMBITS(3) [],
        ST OFFSET(13) NUMBITS(1) [],
        FEF OFFSET(7) NUMBITS(1) [],
        FUGF OFFSET(6) NUMBITS(1) [],
        RTC OFFSET(3) NUMBITS(2) [],
        OSF OFFSET(2) NUMBITS(1) [],
        SR OFFSET(1) NUMBITS(1) []
    ],
    ETH_DMAIER [
        NISE OFFSET(16) NUMBITS(1) [],
        AISE OFFSET(15) NUMBITS(1) [],
        ERIE OFFSET(14) NUMBITS(1) [],
        FBEIE OFFSET(13) NUMBITS(1) [],
        ETIE OFFSET(10) NUMBITS(1) [],
        RWTIE OFFSET(9) NUMBITS(1) [],
        RPSIE OFFSET(8) NUMBITS(1) [],
        RBUIE OFFSET(7) NUMBITS(1) [],
        RIE OFFSET(6) NUMBITS(1) [],
        TUIE OFFSET(5) NUMBITS(1) [],
        ROIE OFFSET(4) NUMBITS(1) [],
        TJTIE OFFSET(3) NUMBITS(1) [],
        TBUIE OFFSET(2) NUMBITS(1) [],
        TPSIE OFFSET(1) NUMBITS(1) [],
        TIE OFFSET(0) NUMBITS(1) []
    ],
    ETH_DMAMFBOCR [
        OFOC OFFSET(28) NUMBITS(1) [],
        MFA OFFSET(17) NUMBITS(11) [],
        OMFC OFFSET(16) NUMBITS(1) [],
        MFC OFFSET(0) NUMBITS(16) []
    ],
    ETH_DMARSWTR [
        RSWTC OFFSET(0) NUMBITS(8) []
    ],
    ETH_DMACHTDR [
        HTDAP OFFSET(0) NUMBITS(32) []
    ],
    ETH_DMACHRDR [
        HRDAP OFFSET(0) NUMBITS(32) []
    ],
    ETH_DMACHTBAR [
        HTBAP OFFSET(0) NUMBITS(32) []
    ],
    ETH_DMACHRBAR [
        HRBAP OFFSET(0) NUMBITS(32) []
    ]
];


pub struct Ethernet<'a> {
    registers: StaticRef<Registers>,
}
