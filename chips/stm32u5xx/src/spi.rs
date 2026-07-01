// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;

register_structs! {
    pub SpiRegisters {
        //Control register 1
        (0x000 => cr1: ReadWrite<u32, CR1::Register>),
        //Control register 2
        (0x004 => cr2: ReadWrite<u32, CR2::Register>),
        //Configuration register 1
        (0x008 => cfg1: ReadWrite<u32, CFG1::Register>),
        //Configuration register 2
        (0x00C => cfg2: ReadWrite<u32, CFG2::Register>),
        //Interrupt enable register
        (0x010 => ier: ReadWrite<u32, IER::Register>),
        //Status register
        (0x014 => sr: ReadOnly<u32, SR::Register>),
        //Interrupt/status flags clear register
        (0x018 => ifcr: WriteOnly<u32, IFCR::Register>),
        //Autonomous mode control register
        (0x01C => autocr: ReadWrite<u32, AUTOCR::Register>),
        //Transmit data register
        (0x020 => txdr: WriteOnly<u32, TXDR::Register>),
        (0x024 => _reserved0),
        //Receive data register
        (0x030 => rxdr: ReadOnly<u32, RXDR::Register>),
        (0x034 => _reserved1),
        //Polynomial register
        (0x040 => crcpoly: ReadWrite<u32, CRCPOLY::Register>),
        //Transmitter CRC register
        (0x044 => txcrc: ReadOnly<u32, TXCRC::Register>),
        //Receiver CRC register
        (0x048 => rxcrc: ReadOnly<u32, RXCRC::Register>),
        //Underrun data register
        (0x04C => udrdr: ReadWrite<u32, UDRDR::Register>),
        (0x050 => @END),
    }
}

// Base addresses for SPI1, 2 and 3 in Secure Alias mode
pub const SPI1_BASE: StaticRef<SpiRegisters> =
    unsafe { StaticRef::new(0x5001_3000 as *const SpiRegisters) };
// pub const SPI2_BASE: StaticRef<SpiRegisters> =
//     unsafe { StaticRef::new(0x5000_3800 as *const SpiRegisters) }; //not used
// pub const SPI3_BASE: StaticRef<SpiRegisters> =
//     unsafe { StaticRef::new(0x5600_2000 as *const SpiRegisters) }; //not used

register_bitfields![u32,
    // Control Register 1
    pub CR1 [
        /// Locking the AF configuration of associated I/Os
        /// 0: AF configuration is not locked
        /// 1: AF configuration is locked
        IOLOCK OFFSET(16) NUMBITS(1) [],

        /// CRC calculation initialization pattern control for transmitter
        /// 0: all zero pattern is applied
        /// 1: all ones pattern is applied
        TCRCINI OFFSET(15) NUMBITS(1) [],

        /// CRC calculation initialization pattern control for receiver
        /// 0: All zero pattern is applied
        /// 1: All ones pattern is applied
        RCRCINI OFFSET(14) NUMBITS(1) [],

        /// Full size (33-bit or 17-bit) CRC polynomial configuration
        /// 0: Full size (33-bit or 17-bit) CRC polynomial is not used
        /// 1: Full size (33-bit or 17-bit) CRC polynomial is used
        CRC33_17 OFFSET(13) NUMBITS(1) [],

        /// Internal slave select signal input level
        /// This bit has an effect only when the SSM bit is set. The value of this bit
        /// is forced onto the peripheral NSS input internally.
        SSI OFFSET(12) NUMBITS(1) [],

        /// Rx/Tx direction at half-duplex mode
        /// 0: SPI is receiver
        /// 1: SPI is transmitter
        HDDIR OFFSET(11) NUMBITS(1) [],

        /// Master suspend request
        /// In master mode, when this bit is set by software, the CSTART bit is reset
        /// at the end of the current frame and communication is suspended.
        CSUSP OFFSET(10) NUMBITS(1) [],

        /// Master transfer start
        /// 0: master transfer is at idle
        /// 1: master transfer is ongoing or temporary suspended by automatic suspend
        CSTART OFFSET(9) NUMBITS(1) [],

        /// Master automatic suspension in Receive mode
        /// 0: SPI flow/clock generation is continuous, regardless of overrun condition
        /// 1: SPI flow is suspended temporary on RxFIFO full condition
        MASRX OFFSET(8) NUMBITS(1) [],

        /// Serial peripheral enable
        /// 0: Serial peripheral disabled
        /// 1: Serial peripheral enabled
        SPE OFFSET(0) NUMBITS(1) []
    ],

    // Control Register 2
    pub CR2 [
        ///Number of data at current transfer
        TSIZE OFFSET(0)  NUMBITS(16) []
    ],

    // SPI configuration register 1
    pub CFG1 [
        /// Bypass of the prescaler at master baud rate clock generator
        /// 0: bypass is disabled
        /// 1: bypass is enabled
        BPASS OFFSET(31) NUMBITS(1) [],

        /// Master baud rate prescaler setting
        /// 000: SPI master clock/2
        /// 001: SPI master clock/4
        /// ...
        /// 111: SPI master clock/256
        MBR OFFSET(28) NUMBITS(3) [
            Div2 = 0b000,
            Div4 = 0b001,
            Div8 = 0b010,
            Div16 = 0b011,
            Div32 = 0b100,
            Div64 = 0b101,
            Div128 = 0b110,
            Div256 = 0b111
        ],

        /// Hardware CRC computation enable
        /// 0: CRC calculation disabled
        /// 1: CRC calculation enabled
        CRCEN OFFSET(22) NUMBITS(1) [],

        /// Length of CRC frame to be transferred and compared
        CRCSIZE OFFSET(16) NUMBITS(5) [],

        /// Tx DMA stream enable
        /// 0: Tx DMA disabled
        /// 1: Tx DMA enabled
        TXDMAEN OFFSET(15) NUMBITS(1) [],

        /// Rx DMA stream enable
        /// 0: Rx-DMA disabled
        /// 1: Rx-DMA enabled
        RXDMAEN OFFSET(14) NUMBITS(1) [],

        /// Behavior of slave transmitter at underrun condition
        /// 0: slave sends a constant pattern defined by the user in the SPI_UDRDR register
        /// 1: Slave repeats the last received data from master.
        UDRCFG OFFSET(9) NUMBITS(1) [],

        /// FIFO threshold level
        /// Defines number of data frames in a single data packet.
        FTHLV OFFSET(5) NUMBITS(4) [],

        /// Number of bits in a single SPI data frame
        DSIZE OFFSET(0) NUMBITS(5) []
    ],

    // SPI configuration register 2
    pub CFG2 [
        /// Alternate function GPIOs control
        /// This bit is taken into account when SPE = 0 only.
        /// 0: The peripheral takes no control of GPIOs while it is disabled
        /// 1: The peripheral keeps always control of all associated GPIOs
        AFCNTR OFFSET(31) NUMBITS(1) [],

        /// NSS output management in master mode
        /// 0: NSS is kept at active level until data transfer is complete, it becomes inactive with EOT flag
        /// 1: SPI data frames are interleaved with NSS nonactive pulses when MIDI[3:0] > 1
        SSOM OFFSET(30) NUMBITS(1) [],

        /// NSS output enable
        /// This bit is taken into account in master mode only
        /// 0: NSS output is disabled and the SPI can work in multimaster configuration
        /// 1: NSS output is enabled. The SPI cannot work in a multimaster environment.
        SSOE OFFSET(29) NUMBITS(1) [],

        /// NSS input/output polarity
        /// 0: low level is active for NSS signal
        /// 1: high level is active for NSS signal
        SSIOP OFFSET(28) NUMBITS(1) [],

        /// Software management of internal slave select signal input
        /// 0: Input value of the internal slave select signal is determined by the external NSS hardware pin
        /// 1: Input value of the internal slave select signal is determined by the SSI bit controlled by software
        SSM OFFSET(26) NUMBITS(1) [],

        // Clock polarity
        // 0: SCK signal is at 0 when idle
        // 1: SCK signal is at 1 when idle
        CPOL OFFSET(25) NUMBITS(1) [],

        // clock phase
        // 0: the first clock transition is the first data capture edge
        // 1: the second clock transition is the first data capture edge
        CPHA OFFSET(24) NUMBITS(1) [],

        // data frame format
        // 0: MSB transmitted first
        // 1: LSB transmitted first
        LSBFRST OFFSET(23) NUMBITS(1) [],

        // SPI master
        // 0: SPI slave
        // 1: SPI master
        MASTER OFFSET(22) NUMBITS(1) [],

        // Serial protocol
        // 000: SPI Motorola
        // 001: SPI TI
        SP OFFSET(19) NUMBITS(3) [
            Motorola = 0b000,
            Ti = 0b001
        ],

        // SPI Communication Mode
        // 00: full-duplex
        // 01: simplex transmitter
        // 10: simplex receiver
        // 11: half-duplex
        COMM OFFSET(17) NUMBITS(2) [
            FullDuplex = 0b00,
            SimplexTx = 0b01,
            SimplexRx = 0b10,
            HalfDuplex = 0b11
        ],

        // Swap functionality of MISO and MOSI pins
        // 0: no swap
        // 1: MOSI and MISO are swapped
        IOSWP OFFSET(15) NUMBITS(1) [],

        // RDY signal input/output polarity
        // 0: high level of the signal means the slave is ready for communication
        // 1: low level of the signal means the slave is ready for communication
        RDIOP OFFSET(14) NUMBITS(1) [],

        // RDY signal input/output management
        // 0: RDY signal is defined internally fixed as permanently active (RDIOP setting has no effect)
        // 1: RDY signal is overtaken from alternate function input (at master case) or output (at slave case)
        RDIOM OFFSET(13) NUMBITS(1) [],

        // Master Inter-Data Idleness
        // Specifies minimum time delay (expressed in SPI clock cycles periods) inserted between two
        // consecutive data frames in master mode.
        MIDI OFFSET(4) NUMBITS(4) [],

        // Master NSS Idleness
        // Specifies an extra delay, expressed in number of SPI clock cycle periods, inserted
        // additionally between active edge of NSS opening a session and the beginning of the first
        // data frame of the session in master mode when SSOE is enabled.
        MSSI OFFSET(0) NUMBITS(4) []
    ],

    // SPI interrupt enable register
    pub IER [
        // Mode Fault interrupt enable
        MODFIE OFFSET(9) NUMBITS(1) [],

        // TIFRE interrupt enable
        TIFREIE OFFSET(8) NUMBITS(1) [],

        // CRC error interrupt enable
        CRCEIE OFFSET(7) NUMBITS(1) [],

        // OVR interrupt enable
        OVRIE OFFSET(6) NUMBITS(1) [],

        // UDR interrupt enable
        UDRIE OFFSET(5) NUMBITS(1) [],

        // TXTF interrupt enable
        TXTFIE OFFSET(4) NUMBITS(1) [],

        // EOT, SUSP and TXC interrupt enable
        EOTIE OFFSET(3) NUMBITS(1) [],

        // DXP interrupt enabled
        DXPIE OFFSET(2) NUMBITS(1) [],

        // TXP interrupt enabled
        TXPIE OFFSET(1) NUMBITS(1) [],

        // RXP interrupt enabled
        RXPIE OFFSET(0) NUMBITS(1) []
    ],

    // SPI status register
    pub SR [
        // number of data frames remaining in current TSIZE session
        CTSIZE OFFSET(16) NUMBITS(16) [],

        // RxFIFO word not empty
        RXWNE OFFSET(15) NUMBITS(1) [],

        // RxFIFO packing level
        RXPLVL OFFSET(13) NUMBITS(2) [],

        // TxFIFO transmission complete
        TXC OFFSET(12) NUMBITS(1) [],

        // Suspension status
        SUSP OFFSET(11) NUMBITS(1) [],

        // Mode fault
        MODF OFFSET(9) NUMBITS(1) [],

        // TI frame format error
        TIFRE OFFSET(8) NUMBITS(1) [],

        // CRC error
        CRCE OFFSET(7) NUMBITS(1) [],

        // Overrun
        OVR OFFSET(6) NUMBITS(1) [],

        // Underrun
        UDR OFFSET(5) NUMBITS(1) [],

        // Transmission transfer filled
        TXTF OFFSET(4) NUMBITS(1) [],

        // End of transfer
        EOT OFFSET(3) NUMBITS(1) [],

        // Duplex packet
        DXP OFFSET(2) NUMBITS(1) [],

        // Tx-packet space available
        TXP OFFSET(1) NUMBITS(1) [],

        // RXP: Rx-packet available
        RXP OFFSET(0) NUMBITS(1) []
    ],

    /// SPI interrupt/status flags clear register
    pub IFCR [
        // Suspend flag clear
        SUSPC OFFSET(11) NUMBITS(1) [],
        // Mode fault flag clear
        MODFC OFFSET(9) NUMBITS(1) [],
        // TI frame format error flag clear
        TIFREC OFFSET(8) NUMBITS(1) [],
        // CRC error flag clear
        CRCEC OFFSET(7) NUMBITS(1) [],
        // Overrun flag clear
        OVRC OFFSET(6) NUMBITS(1) [],
        // Underrun flag clear
        UDRC OFFSET(5) NUMBITS(1) [],
        // Transmission transfer filled flag clear
        TXTFC OFFSET(4) NUMBITS(1) [],
        // End of transfer flag clear
        EOTC OFFSET(3) NUMBITS(1) []
    ],

    /// SPI autonomous mode control register
    pub AUTOCR [
        // Hardware control of CSTART triggering enable
        TRIGEN OFFSET(21) NUMBITS(1) [],
        // Trigger polarity
        TRIGPOL OFFSET(20) NUMBITS(1) [],
        // Trigger selection
        TRIGSEL OFFSET(16) NUMBITS(4) []
    ],

    /// SPI transmit data register
    pub TXDR[
        // Transmit data register
        TXDR OFFSET(0) NUMBITS(32) []
    ],

    /// SPI receive data register
    pub RXDR [
        // Receive data register
        RXDR OFFSET(0) NUMBITS(32) []
    ],

    // SPI polynomial register
    pub CRCPOLY [
        // CRC polynomial register
        CRCPOLY OFFSET(0) NUMBITS(32) []
    ],

    // SPI transmitter CRC register
    pub TXCRC [
        // CRC register for transmitter
        TXCRC OFFSET(0) NUMBITS(32) []
    ],

    // SPI receiver CRC register
    pub RXCRC [
        // CRC register for receiver
        RXCRC OFFSET(0) NUMBITS(32) []
    ],

    // SPI underrun data register
    pub UDRDR [
        // Data at slave underrun condition
        UDRDR OFFSET(0) NUMBITS(32) []
    ]
];
