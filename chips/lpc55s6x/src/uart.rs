// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Universal Asynchronous Receiver/Transmitter (UART) driver for the LPC55S6x family.
//!
//! The UART peripheral provides full‑duplex asynchronous serial communication,
//! typically used for console I/O, debugging, or external device interfaces.
//! On the LPC55S6x, UART functionality is implemented through the Flexcomm
//! blocks when configured in USART mode.
//!
//! Features supported:
//! - Standard 8‑N‑1 asynchronous communication
//! - Configurable baud rate generation via fractional rate generator (FRG)
//! - Interrupts for transmit, receive, and error conditions
//! - FIFO support for buffered TX/RX
//!
//! Reference: *LPC55S6x/LPC55S2x/LPC552x User Manual* (NXP).

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use kernel::hil::uart::ReceiveClient;
use kernel::hil::uart::{
    Configure, Parameters, Parity, Receive, StopBits, Transmit, TransmitClient, Width,
};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::{hil, ErrorCode};

use crate::clocks::FrgId;
use crate::clocks::{Clock, FrgClockSource};
use crate::flexcomm::Flexcomm;

register_structs! {
    /// USARTs
    pub UsartRegisters {
        /// USART Configuration register. Basic USART configuration settings that typically
        (0x000 => cfg: ReadWrite<u32, CFG::Register>),
        /// USART Control register. USART control settings that are more likely to change du
        (0x004 => ctl: ReadWrite<u32, CTL::Register>),
        /// USART Status register. The complete status value can be read here. Writing ones
        (0x008 => stat: ReadWrite<u32, STAT::Register>),
        /// Interrupt Enable read and Set register for USART (not FIFO) status. Contains ind
        (0x00C => intenset: ReadWrite<u32, INTENSET::Register>),
        /// Interrupt Enable Clear register. Allows clearing any combination of bits in the
        (0x010 => intenclr: WriteOnly<u32, INTENCLR::Register>),
        (0x014 => _reserved0),
        /// Baud Rate Generator register. 16-bit integer baud rate divisor value.
        (0x020 => brg: ReadWrite<u32>),
        /// Interrupt status register. Reflects interrupts that are currently enabled.
        (0x024 => intstat: ReadOnly<u32, INTSTAT::Register>),
        /// Oversample selection register for asynchronous communication.
        (0x028 => osr: ReadWrite<u32>),
        /// Address register for automatic address matching.
        (0x02C => addr: ReadWrite<u32>),
        (0x030 => _reserved1),
        /// FIFO configuration and enable register.
        (0xE00 => fifocfg: ReadWrite<u32, FIFOCFG::Register>),
        /// FIFO status register.
        (0xE04 => fifostat: ReadWrite<u32, FIFOSTAT::Register>),
        /// FIFO trigger settings for interrupt and DMA request.
        (0xE08 => fifotrig: ReadWrite<u32, FIFOTRIG::Register>),
        (0xE0C => _reserved2),
        /// FIFO interrupt enable set (enable) and read register.
        (0xE10 => fifointenset: ReadWrite<u32, FIFOINTENSET::Register>),
        /// FIFO interrupt enable clear (disable) and read register.
        (0xE14 => fifointenclr: ReadWrite<u32, FIFOINTENCLR::Register>),
        /// FIFO interrupt status register.
        (0xE18 => fifointstat: ReadOnly<u32, FIFOINTSTAT::Register>),
        (0xE1C => _reserved3),
        /// FIFO write data.
        (0xE20 => fifowr: WriteOnly<u32>),
        (0xE24 => _reserved4),
        /// FIFO read data.
        (0xE30 => fiford: ReadOnly<u32, FIFORD::Register>),
        (0xE34 => _reserved5),
        /// FIFO data read with no FIFO pop.
        (0xE40 => fifordnopop: ReadOnly<u32, FIFORDNOPOP::Register>),
        (0xE44 => _reserved6),
        /// FIFO size register
        (0xE48 => fifosize: ReadWrite<u32>),
        (0xE4C => _reserved7),
        /// Peripheral identification register.
        (0xFFC => id: ReadOnly<u32, ID::Register>),
        (0x1000 => @END),
    }
}
register_bitfields![u32,
CFG [
    /// USART Enable.
    ENABLE OFFSET(0) NUMBITS(1) [
        /// Disabled. The USART is disabled and the internal state machine and counters are
        DISABLED = 0,
        /// Enabled. The USART is enabled for operation.
        EnabledTheUSARTIsEnabledForOperation = 1
    ],
    /// Selects the data size for the USART.
    DATALEN OFFSET(2) NUMBITS(2) [
        /// 7 bit Data length.
        _7BitDataLength = 0,
        /// 8 bit Data length.
        _8BitDataLength = 1,
        /// 9 bit data length. The 9th bit is commonly used for addressing in multidrop mode
        _9BitDataLength = 2
    ],
    /// Selects what type of parity is used by the USART.
    PARITYSEL OFFSET(4) NUMBITS(2) [
        /// No parity.
        NO_PARITY = 0,
        /// Even parity. Adds a bit to each character such that the number of 1s in a transm
        EVEN_PARITY = 2,
        /// Odd parity. Adds a bit to each character such that the number of 1s in a transmi
        ODD_PARITY = 3
    ],
    /// Number of stop bits appended to transmitted data. Only a single stop bit is requ
    STOPLEN OFFSET(6) NUMBITS(1) [
        /// 1 stop bit.
        _1StopBit = 0,
        /// 2 stop bits. This setting should only be used for asynchronous communication.
        _2StopBits = 1
    ],
    /// Selects standard or 32 kHz clocking mode.
    MODE32K OFFSET(7) NUMBITS(1) [
        /// Disabled. USART uses standard clocking.
        DisabledUSARTUsesStandardClocking = 0,
        /// Enabled. USART uses the 32 kHz clock from the RTC oscillator as the clock source
        ENABLED = 1
    ],
    /// LIN break mode enable.
    LINMODE OFFSET(8) NUMBITS(1) [
        /// Disabled. Break detect and generate is configured for normal operation.
        DisabledBreakDetectAndGenerateIsConfiguredForNormalOperation = 0,
        /// Enabled. Break detect and generate is configured for LIN bus operation.
        EnabledBreakDetectAndGenerateIsConfiguredForLINBusOperation = 1
    ],
    /// CTS Enable. Determines whether CTS is used for flow control. CTS can be from the
    CTSEN OFFSET(9) NUMBITS(1) [
        /// No flow control. The transmitter does not receive any automatic flow control sig
        NoFlowControlTheTransmitterDoesNotReceiveAnyAutomaticFlowControlSignal = 0,
        /// Flow control enabled. The transmitter uses the CTS input (or RTS output in loopb
        ENABLED = 1
    ],
    /// Selects synchronous or asynchronous operation.
    SYNCEN OFFSET(11) NUMBITS(1) [
        /// Asynchronous mode.
        AsynchronousMode = 0,
        /// Synchronous mode.
        SynchronousMode = 1
    ],
    /// Selects the clock polarity and sampling edge of received data in synchronous mod
    CLKPOL OFFSET(12) NUMBITS(1) [
        /// Falling edge. Un_RXD is sampled on the falling edge of SCLK.
        FallingEdgeUn_RXDIsSampledOnTheFallingEdgeOfSCLK = 0,
        /// Rising edge. Un_RXD is sampled on the rising edge of SCLK.
        RisingEdgeUn_RXDIsSampledOnTheRisingEdgeOfSCLK = 1
    ],
    /// Synchronous mode Master select.
    SYNCMST OFFSET(14) NUMBITS(1) [
        /// Slave. When synchronous mode is enabled, the USART is a slave.
        SlaveWhenSynchronousModeIsEnabledTheUSARTIsASlave = 0,
        /// Master. When synchronous mode is enabled, the USART is a master.
        MasterWhenSynchronousModeIsEnabledTheUSARTIsAMaster = 1
    ],
    /// Selects data loopback mode.
    LOOP OFFSET(15) NUMBITS(1) [
        /// Normal operation.
        NormalOperation = 0,
        /// Loopback mode. This provides a mechanism to perform diagnostic loopback testing
        LOOPBACK = 1
    ],
    /// Output Enable Turnaround time enable for RS-485 operation.
    OETA OFFSET(18) NUMBITS(1) [
        /// Disabled. If selected by OESEL, the Output Enable signal deasserted at the end o
        DISABLED = 0,
        /// Enabled. If selected by OESEL, the Output Enable signal remains asserted for one
        ENABLED = 1
    ],
    /// Automatic Address matching enable.
    AUTOADDR OFFSET(19) NUMBITS(1) [
        /// Disabled. When addressing is enabled by ADDRDET, address matching is done by sof
        DISABLED = 0,
        /// Enabled. When addressing is enabled by ADDRDET, address matching is done by hard
        ENABLED = 1
    ],
    /// Output Enable Select.
    OESEL OFFSET(20) NUMBITS(1) [
        /// Standard. The RTS signal is used as the standard flow control function.
        StandardTheRTSSignalIsUsedAsTheStandardFlowControlFunction = 0,
        /// RS-485. The RTS signal configured to provide an output enable signal to control
        RS_485 = 1
    ],
    /// Output Enable Polarity.
    OEPOL OFFSET(21) NUMBITS(1) [
        /// Low. If selected by OESEL, the output enable is active low.
        LowIfSelectedByOESELTheOutputEnableIsActiveLow = 0,
        /// High. If selected by OESEL, the output enable is active high.
        HighIfSelectedByOESELTheOutputEnableIsActiveHigh = 1
    ],
    /// Receive data polarity.
    RXPOL OFFSET(22) NUMBITS(1) [
        /// Standard. The RX signal is used as it arrives from the pin. This means that the
        STANDARD = 0,
        /// Inverted. The RX signal is inverted before being used by the USART. This means t
        INVERTED = 1
    ],
    /// Transmit data polarity.
    TXPOL OFFSET(23) NUMBITS(1) [
        /// Standard. The TX signal is sent out without change. This means that the TX rest
        STANDARD = 0,
        /// Inverted. The TX signal is inverted by the USART before being sent out. This mea
        INVERTED = 1
    ]
],
CTL [
    /// Break Enable.
    TXBRKEN OFFSET(1) NUMBITS(1) [
        /// Normal operation.
        NormalOperation = 0,
        /// Continuous break. Continuous break is sent immediately when this bit is set, and
        CONTINOUS = 1
    ],
    /// Enable address detect mode.
    ADDRDET OFFSET(2) NUMBITS(1) [
        /// Disabled. The USART presents all incoming data.
        DisabledTheUSARTPresentsAllIncomingData = 0,
        /// Enabled. The USART receiver ignores incoming data that does not have the most si
        ENABLED = 1
    ],
    /// Transmit Disable.
    TXDIS OFFSET(6) NUMBITS(1) [
        /// Not disabled. USART transmitter is not disabled.
        NotDisabledUSARTTransmitterIsNotDisabled = 0,
        /// Disabled. USART transmitter is disabled after any character currently being tran
        DISABLED = 1
    ],
    /// Continuous Clock generation. By default, SCLK is only output while data is being
    CC OFFSET(8) NUMBITS(1) [
        /// Clock on character. In synchronous mode, SCLK cycles only when characters are be
        CLOCK_ON_CHARACTER = 0,
        /// Continuous clock. SCLK runs continuously in synchronous mode, allowing character
        CONTINOUS_CLOCK = 1
    ],
    /// Clear Continuous Clock.
    CLRCCONRX OFFSET(9) NUMBITS(1) [
        /// No effect. No effect on the CC bit.
        NoEffectNoEffectOnTheCCBit = 0,
        /// Auto-clear. The CC bit is automatically cleared when a complete character has be
        AUTO_CLEAR = 1
    ],
    /// Autobaud enable.
    AUTOBAUD OFFSET(16) NUMBITS(1) [
        /// Disabled. USART is in normal operating mode.
        DisabledUSARTIsInNormalOperatingMode = 0,
        /// Enabled. USART is in autobaud mode. This bit should only be set when the USART r
        ENABLED = 1
    ]
],
STAT [
    /// Receiver Idle. When 0, indicates that the receiver is currently in the process o
    RXIDLE OFFSET(1) NUMBITS(1) [],
    /// Transmitter Idle. When 0, indicates that the transmitter is currently in the pro
    TXIDLE OFFSET(3) NUMBITS(1) [],
    /// This bit reflects the current state of the CTS signal, regardless of the setting
    CTS OFFSET(4) NUMBITS(1) [],
    /// This bit is set when a change in the state is detected for the CTS flag above. T
    DELTACTS OFFSET(5) NUMBITS(1) [],
    /// Transmitter Disabled Status flag. When 1, this bit indicates that the USART tran
    TXDISSTAT OFFSET(6) NUMBITS(1) [],
    /// Received Break. This bit reflects the current state of the receiver break detect
    RXBRK OFFSET(10) NUMBITS(1) [],
    /// This bit is set when a change in the state of receiver break detection occurs. C
    DELTARXBRK OFFSET(11) NUMBITS(1) [],
    /// This bit is set when a start is detected on the receiver input. Its purpose is p
    START OFFSET(12) NUMBITS(1) [],
    /// Framing Error interrupt flag. This flag is set when a character is received with
    FRAMERRINT OFFSET(13) NUMBITS(1) [],
    /// Parity Error interrupt flag. This flag is set when a parity error is detected in
    PARITYERRINT OFFSET(14) NUMBITS(1) [],
    /// Received Noise interrupt flag. Three samples of received data are taken in order
    RXNOISEINT OFFSET(15) NUMBITS(1) [],
    /// Auto baud Error. An auto baud error can occur if the BRG counts to its limit bef
    ABERR OFFSET(16) NUMBITS(1) []
],
INTENSET [
    /// When 1, enables an interrupt when the transmitter becomes idle (TXIDLE = 1).
    TXIDLEEN OFFSET(3) NUMBITS(1) [],
    /// When 1, enables an interrupt when there is a change in the state of the CTS inpu
    DELTACTSEN OFFSET(5) NUMBITS(1) [],
    /// When 1, enables an interrupt when the transmitter is fully disabled as indicated
    TXDISEN OFFSET(6) NUMBITS(1) [],
    /// When 1, enables an interrupt when a change of state has occurred in the detectio
    DELTARXBRKEN OFFSET(11) NUMBITS(1) [],
    /// When 1, enables an interrupt when a received start bit has been detected.
    STARTEN OFFSET(12) NUMBITS(1) [],
    /// When 1, enables an interrupt when a framing error has been detected.
    FRAMERREN OFFSET(13) NUMBITS(1) [],
    /// When 1, enables an interrupt when a parity error has been detected.
    PARITYERREN OFFSET(14) NUMBITS(1) [],
    /// When 1, enables an interrupt when noise is detected. See description of the RXNO
    RXNOISEEN OFFSET(15) NUMBITS(1) [],
    /// When 1, enables an interrupt when an auto baud error occurs.
    ABERREN OFFSET(16) NUMBITS(1) []
],
INTENCLR [
    /// Writing 1 clears the corresponding bit in the INTENSET register.
    TXIDLECLR OFFSET(3) NUMBITS(1) [],
    /// Writing 1 clears the corresponding bit in the INTENSET register.
    DELTACTSCLR OFFSET(5) NUMBITS(1) [],
    /// Writing 1 clears the corresponding bit in the INTENSET register.
    TXDISCLR OFFSET(6) NUMBITS(1) [],
    /// Writing 1 clears the corresponding bit in the INTENSET register.
    DELTARXBRKCLR OFFSET(11) NUMBITS(1) [],
    /// Writing 1 clears the corresponding bit in the INTENSET register.
    STARTCLR OFFSET(12) NUMBITS(1) [],
    /// Writing 1 clears the corresponding bit in the INTENSET register.
    FRAMERRCLR OFFSET(13) NUMBITS(1) [],
    /// Writing 1 clears the corresponding bit in the INTENSET register.
    PARITYERRCLR OFFSET(14) NUMBITS(1) [],
    /// Writing 1 clears the corresponding bit in the INTENSET register.
    RXNOISECLR OFFSET(15) NUMBITS(1) [],
    /// Writing 1 clears the corresponding bit in the INTENSET register.
    ABERRCLR OFFSET(16) NUMBITS(1) []
],
BRG [
    /// This value is used to divide the USART input clock to determine the baud rate, b
    BRGVAL OFFSET(0) NUMBITS(16) []
],
INTSTAT [
    /// Transmitter Idle status.
    TXIDLE OFFSET(3) NUMBITS(1) [],
    /// This bit is set when a change in the state of the CTS input is detected.
    DELTACTS OFFSET(5) NUMBITS(1) [],
    /// Transmitter Disabled Interrupt flag.
    TXDISINT OFFSET(6) NUMBITS(1) [],
    /// This bit is set when a change in the state of receiver break detection occurs.
    DELTARXBRK OFFSET(11) NUMBITS(1) [],
    /// This bit is set when a start is detected on the receiver input.
    START OFFSET(12) NUMBITS(1) [],
    /// Framing Error interrupt flag.
    FRAMERRINT OFFSET(13) NUMBITS(1) [],
    /// Parity Error interrupt flag.
    PARITYERRINT OFFSET(14) NUMBITS(1) [],
    /// Received Noise interrupt flag.
    RXNOISEINT OFFSET(15) NUMBITS(1) [],
    /// Auto baud Error Interrupt flag.
    ABERRINT OFFSET(16) NUMBITS(1) []
],
OSR [
    /// Oversample Selection Value. 0 to 3 = not supported 0x4 = 5 function clocks are u
    OSRVAL OFFSET(0) NUMBITS(4) []
],
ADDR [
    /// 8-bit address used with automatic address matching. Used when address detection
    ADDRESS OFFSET(0) NUMBITS(8) []
],
FIFOCFG [
    /// Enable the transmit FIFO.
    ENABLETX OFFSET(0) NUMBITS(1) [
        /// The transmit FIFO is not enabled.
        TheTransmitFIFOIsNotEnabled = 0,
        /// The transmit FIFO is enabled.
        TheTransmitFIFOIsEnabled = 1
    ],
    /// Enable the receive FIFO.
    ENABLERX OFFSET(1) NUMBITS(1) [
        /// The receive FIFO is not enabled.
        TheReceiveFIFOIsNotEnabled = 0,
        /// The receive FIFO is enabled.
        TheReceiveFIFOIsEnabled = 1
    ],
    /// FIFO size configuration. This is a read-only field. 0x0 = FIFO is configured as
    SIZE OFFSET(4) NUMBITS(2) [],
    /// DMA configuration for transmit.
    DMATX OFFSET(12) NUMBITS(1) [
        /// DMA is not used for the transmit function.
        DMAIsNotUsedForTheTransmitFunction = 0,
        /// Trigger DMA for the transmit function if the FIFO is not full. Generally, data i
        ENABLED = 1
    ],
    /// DMA configuration for receive.
    DMARX OFFSET(13) NUMBITS(1) [
        /// DMA is not used for the receive function.
        DMAIsNotUsedForTheReceiveFunction = 0,
        /// Trigger DMA for the receive function if the FIFO is not empty. Generally, data i
        ENABLED = 1
    ],
    /// Wake-up for transmit FIFO level. This allows the device to be woken from reduced
    WAKETX OFFSET(14) NUMBITS(1) [
        /// Only enabled interrupts will wake up the device form reduced power modes.
        OnlyEnabledInterruptsWillWakeUpTheDeviceFormReducedPowerModes = 0,
        /// A device wake-up for DMA will occur if the transmit FIFO level reaches the value
        ENABLED = 1
    ],
    /// Wake-up for receive FIFO level. This allows the device to be woken from reduced
    WAKERX OFFSET(15) NUMBITS(1) [
        /// Only enabled interrupts will wake up the device form reduced power modes.
        OnlyEnabledInterruptsWillWakeUpTheDeviceFormReducedPowerModes = 0,
        /// A device wake-up for DMA will occur if the receive FIFO level reaches the value
        ENABLED = 1
    ],
    /// Empty command for the transmit FIFO. When a 1 is written to this bit, the TX FIF
    EMPTYTX OFFSET(16) NUMBITS(1) [],
    /// Empty command for the receive FIFO. When a 1 is written to this bit, the RX FIFO
    EMPTYRX OFFSET(17) NUMBITS(1) []
],
FIFOSTAT [
    /// TX FIFO error. Will be set if a transmit FIFO error occurs. This could be an ove
    TXERR OFFSET(0) NUMBITS(1) [],
    /// RX FIFO error. Will be set if a receive FIFO overflow occurs, caused by software
    RXERR OFFSET(1) NUMBITS(1) [],
    /// Peripheral interrupt. When 1, this indicates that the peripheral function has as
    PERINT OFFSET(3) NUMBITS(1) [],
    /// Transmit FIFO empty. When 1, the transmit FIFO is empty. The peripheral may stil
    TXEMPTY OFFSET(4) NUMBITS(1) [],
    /// Transmit FIFO not full. When 1, the transmit FIFO is not full, so more data can
    TXNOTFULL OFFSET(5) NUMBITS(1) [],
    /// Receive FIFO not empty. When 1, the receive FIFO is not empty, so data can be re
    RXNOTEMPTY OFFSET(6) NUMBITS(1) [],
    /// Receive FIFO full. When 1, the receive FIFO is full. Data needs to be read out t
    RXFULL OFFSET(7) NUMBITS(1) [],
    /// Transmit FIFO current level. A 0 means the TX FIFO is currently empty, and the T
    TXLVL OFFSET(8) NUMBITS(5) [],
    /// Receive FIFO current level. A 0 means the RX FIFO is currently empty, and the RX
    RXLVL OFFSET(16) NUMBITS(5) []
],
FIFOTRIG [
    /// Transmit FIFO level trigger enable. This trigger will become an interrupt if ena
    TXLVLENA OFFSET(0) NUMBITS(1) [
        /// Transmit FIFO level does not generate a FIFO level trigger.
        TransmitFIFOLevelDoesNotGenerateAFIFOLevelTrigger = 0,
        /// An trigger will be generated if the transmit FIFO level reaches the value specif
        ENABLED = 1
    ],
    /// Receive FIFO level trigger enable. This trigger will become an interrupt if enab
    RXLVLENA OFFSET(1) NUMBITS(1) [
        /// Receive FIFO level does not generate a FIFO level trigger.
        ReceiveFIFOLevelDoesNotGenerateAFIFOLevelTrigger = 0,
        /// An trigger will be generated if the receive FIFO level reaches the value specifi
        ENABLED = 1
    ],
    /// Transmit FIFO level trigger point. This field is used only when TXLVLENA = 1. If
    TXLVL OFFSET(8) NUMBITS(4) [],
    /// Receive FIFO level trigger point. The RX FIFO level is checked when a new piece
    RXLVL OFFSET(16) NUMBITS(4) []
],
FIFOINTENSET [
    /// Determines whether an interrupt occurs when a transmit error occurs, based on th
    TXERR OFFSET(0) NUMBITS(1) [
        /// No interrupt will be generated for a transmit error.
        NoInterruptWillBeGeneratedForATransmitError = 0,
        /// An interrupt will be generated when a transmit error occurs.
        AnInterruptWillBeGeneratedWhenATransmitErrorOccurs = 1
    ],
    /// Determines whether an interrupt occurs when a receive error occurs, based on the
    RXERR OFFSET(1) NUMBITS(1) [
        /// No interrupt will be generated for a receive error.
        NoInterruptWillBeGeneratedForAReceiveError = 0,
        /// An interrupt will be generated when a receive error occurs.
        AnInterruptWillBeGeneratedWhenAReceiveErrorOccurs = 1
    ],
    /// Determines whether an interrupt occurs when a the transmit FIFO reaches the leve
    TXLVL OFFSET(2) NUMBITS(1) [
        /// No interrupt will be generated based on the TX FIFO level.
        NoInterruptWillBeGeneratedBasedOnTheTXFIFOLevel = 0,
        /// If TXLVLENA in the FIFOTRIG register = 1, an interrupt will be generated when th
        ENABLED = 1
    ],
    /// Determines whether an interrupt occurs when a the receive FIFO reaches the level
    RXLVL OFFSET(3) NUMBITS(1) [
        /// No interrupt will be generated based on the RX FIFO level.
        NoInterruptWillBeGeneratedBasedOnTheRXFIFOLevel = 0,
        /// If RXLVLENA in the FIFOTRIG register = 1, an interrupt will be generated when th
        ENABLED = 1
    ]
],
FIFOINTENCLR [
    /// Writing one clears the corresponding bits in the FIFOINTENSET register.
    TXERR OFFSET(0) NUMBITS(1) [],
    /// Writing one clears the corresponding bits in the FIFOINTENSET register.
    RXERR OFFSET(1) NUMBITS(1) [],
    /// Writing one clears the corresponding bits in the FIFOINTENSET register.
    TXLVL OFFSET(2) NUMBITS(1) [],
    /// Writing one clears the corresponding bits in the FIFOINTENSET register.
    RXLVL OFFSET(3) NUMBITS(1) []
],
FIFOINTSTAT [
    /// TX FIFO error.
    TXERR OFFSET(0) NUMBITS(1) [],
    /// RX FIFO error.
    RXERR OFFSET(1) NUMBITS(1) [],
    /// Transmit FIFO level interrupt.
    TXLVL OFFSET(2) NUMBITS(1) [],
    /// Receive FIFO level interrupt.
    RXLVL OFFSET(3) NUMBITS(1) [],
    /// Peripheral interrupt.
    PERINT OFFSET(4) NUMBITS(1) []
],
FIFOWR [
    /// Transmit data to the FIFO.
    TXDATA OFFSET(0) NUMBITS(9) []
],
FIFORD [
    /// Received data from the FIFO. The number of bits used depends on the DATALEN and
    RXDATA OFFSET(0) NUMBITS(9) [],
    /// Framing Error status flag. This bit reflects the status for the data it is read
    FRAMERR OFFSET(13) NUMBITS(1) [],
    /// Parity Error status flag. This bit reflects the status for the data it is read a
    PARITYERR OFFSET(14) NUMBITS(1) [],
    /// Received Noise flag. See description of the RxNoiseInt bit in Table 354.
    RXNOISE OFFSET(15) NUMBITS(1) []
],
FIFORDNOPOP [
    /// Received data from the FIFO. The number of bits used depends on the DATALEN and
    RXDATA OFFSET(0) NUMBITS(9) [],
    /// Framing Error status flag. This bit reflects the status for the data it is read
    FRAMERR OFFSET(13) NUMBITS(1) [],
    /// Parity Error status flag. This bit reflects the status for the data it is read a
    PARITYERR OFFSET(14) NUMBITS(1) [],
    /// Received Noise flag. See description of the RxNoiseInt bit in Table 354.
    RXNOISE OFFSET(15) NUMBITS(1) []
],
FIFOSIZE [
    /// Provides the size of the FIFO for software. The size of the SPI FIFO is 8 entrie
    FIFOSIZE OFFSET(0) NUMBITS(5) []
],
ID [
    /// Aperture: encoded as (aperture size/4K) -1, so 0x00 means a 4K aperture.
    APERTURE OFFSET(0) NUMBITS(8) [],
    /// Minor revision of module implementation.
    MINOR_REV OFFSET(8) NUMBITS(4) [],
    /// Major revision of module implementation.
    MAJOR_REV OFFSET(12) NUMBITS(4) [],
    /// Module identifier for the selected function.
    ID OFFSET(16) NUMBITS(16) []
]
];

#[derive(Copy, Clone, PartialEq)]
enum UARTStateTX {
    Idle,
    Transmitting,
    AbortRequested,
}

#[derive(Copy, Clone, PartialEq)]
enum UARTStateRX {
    Idle,
    Receiving,
    AbortRequested,
}

const USART0_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40086000 as *const UsartRegisters) };

const USART4_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x4008A000 as *const UsartRegisters) };

pub struct Uart<'a> {
    registers: StaticRef<UsartRegisters>,
    instance: u8,
    clocks: OptionalCell<&'a Clock>,
    flexcomm: OptionalCell<&'a Flexcomm>,

    uart_clock_source: Cell<FrgClockSource>,

    tx_client: OptionalCell<&'a dyn TransmitClient>,
    rx_client: OptionalCell<&'a dyn ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_position: Cell<usize>,
    tx_len: Cell<usize>,
    tx_status: Cell<UARTStateTX>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_position: Cell<usize>,
    rx_len: Cell<usize>,
    rx_status: Cell<UARTStateRX>,
}

impl<'a> Uart<'a> {
    pub fn new(registers: StaticRef<UsartRegisters>, instance: u8) -> Self {
        Self {
            registers,
            instance,
            clocks: OptionalCell::empty(),
            flexcomm: OptionalCell::empty(),

            uart_clock_source: Cell::new(FrgClockSource::Fro12Mhz),

            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),

            tx_buffer: TakeCell::empty(),
            tx_position: Cell::new(0),
            tx_len: Cell::new(0),
            tx_status: Cell::new(UARTStateTX::Idle),

            rx_buffer: TakeCell::empty(),
            rx_position: Cell::new(0),
            rx_len: Cell::new(0),
            rx_status: Cell::new(UARTStateRX::Idle),
        }
    }
    pub fn new_uart0() -> Self {
        Self::new(USART0_BASE, 0)
    }

    pub fn new_uart4() -> Self {
        Self::new(USART4_BASE, 4)
    }

    pub fn set_clocks(&self, clocks: &'a Clock) {
        self.clocks.set(clocks);
    }

    pub fn set_flexcomm(&self, flexcomm: &'a Flexcomm) {
        self.flexcomm.set(flexcomm);
    }

    pub fn set_clock_source(&self, source: FrgClockSource) {
        self.uart_clock_source.set(source);
    }

    pub fn enable(&self) {
        self.registers.cfg.modify(CFG::ENABLE::SET);
    }

    pub fn disable(&self) {
        self.registers.cfg.modify(CFG::ENABLE::CLEAR);
    }

    fn set_interrupts_for_transmitting(&self) {
        // We want to know when the FIFO has space.
        self.registers
            .fifointenset
            .write(FIFOINTENSET::TXLVL::SET + FIFOINTENSET::TXERR::SET);
        // We do NOT care about the final TXIDLE state yet.
        self.registers.intenclr.write(INTENCLR::TXIDLECLR::SET);
    }

    fn set_interrupts_for_finishing(&self) {
        // We no longer care if the FIFO has space.
        self.registers.fifointenclr.write(FIFOINTENCLR::TXLVL::SET);
        // We ONLY care about when the transmission is truly complete.
        self.registers.intenset.write(INTENSET::TXIDLEEN::SET);
    }

    /// Disables all UART transmit-related interrupts.
    fn disable_all_tx_interrupts(&self) {
        self.registers
            .fifointenclr
            .write(FIFOINTENCLR::TXLVL::SET + FIFOINTENCLR::TXERR::SET);
        self.registers.intenclr.write(INTENCLR::TXIDLECLR::SET);
    }

    pub fn is_transmitting(&self) -> bool {
        self.tx_status.get() == UARTStateTX::Transmitting
    }

    pub fn enable_receive_interrupt(&self) {
        self.registers
            .fifointenset
            .modify(FIFOINTENSET::RXLVL::SET + FIFOINTENSET::RXERR::SET);

        self.registers
            .intenset
            .modify(INTENSET::FRAMERREN::SET + INTENSET::PARITYERREN::SET);
    }

    pub fn disable_receive_interrupt(&self) {
        self.registers
            .fifointenclr
            .modify(FIFOINTENCLR::RXLVL::SET + FIFOINTENCLR::RXERR::SET);

        self.registers
            .intenclr
            .write(INTENCLR::FRAMERRCLR::SET + INTENCLR::PARITYERRCLR::SET);
    }
    pub fn uart_is_writable(&self) -> bool {
        self.registers.fifostat.is_set(FIFOSTAT::TXNOTFULL)
    }

    pub fn uart_is_readable(&self) -> bool {
        self.registers.fifostat.is_set(FIFOSTAT::RXNOTEMPTY)
    }

    pub fn send_byte(&self, data: u8) {
        self.registers.fifowr.set(data as u32);
    }

    pub fn receive_byte(&self) -> u8 {
        (self.registers.fiford.get() & 0xFF) as u8
    }

    pub fn handle_interrupt(&self) {
        // --- Handle Errors (RX-only clears) ---
        let framing_error = self.registers.stat.is_set(STAT::FRAMERRINT);
        let parity_error = self.registers.stat.is_set(STAT::PARITYERRINT);
        let rx_fifo_error = self.registers.fifostat.is_set(FIFOSTAT::RXERR);

        if framing_error || parity_error || rx_fifo_error {
            // Clear RX-related status bits; DO NOT touch TX FIFO or TX interrupts here.
            self.registers.stat.write(
                STAT::FRAMERRINT::SET
                    + STAT::PARITYERRINT::SET
                    + STAT::RXBRK::SET
                    + STAT::DELTACTS::SET,
            );
            self.registers.fifostat.write(FIFOSTAT::RXERR::SET);

            // If no receive is active, turn off RX interrupts; otherwise leave them on.
            if self.rx_status.get() != UARTStateRX::Receiving {
                self.disable_receive_interrupt();
            }
            // Return; TX remains untouched so the process console can still print.
            return;
        }

        // --- Handle Transmit ---
        let tx_level_triggered = self.registers.fifointstat.is_set(FIFOINTSTAT::TXLVL);
        let tx_idle_triggered = self.registers.intstat.is_set(INTSTAT::TXIDLE);

        if self.tx_status.get() == UARTStateTX::Transmitting {
            if tx_level_triggered {
                // Fill TX FIFO from software buffer
                self.fill_fifo();

                // If we finished sending the software buffer, wait for TXIDLE to signal completion.
                if self.tx_position.get() == self.tx_len.get() {
                    self.set_interrupts_for_finishing(); // enable INTENSET::TXIDLEEN, disable FIFO TXLVL
                }
            }
            if tx_idle_triggered {
                // Acknowledge TXIDLE and finish TX operation.
                self.registers.stat.write(STAT::TXIDLE::SET);
                self.disable_all_tx_interrupts();
                self.tx_status.set(UARTStateTX::Idle);

                // Notify the TX client (console/process_console expects this)
                self.tx_client.map(|client| {
                    self.tx_buffer.take().map(|buf| {
                        client.transmitted_buffer(buf, self.tx_position.get(), Ok(()));
                    });
                });
            }
        }

        // --- Handle Receive ---
        if self.registers.fifointstat.is_set(FIFOINTSTAT::RXLVL) {
            if self.rx_status.get() == UARTStateRX::Receiving {
                if self.uart_is_readable() && self.rx_position.get() < self.rx_len.get() {
                    let byte = self.receive_byte();
                    let pos = self.rx_position.get();

                    self.rx_buffer.map(|buf| {
                        buf[pos] = byte;
                    });
                    self.rx_position.set(pos + 1);

                    // If buffer is complete, finish and notify client.
                    if self.rx_position.get() == self.rx_len.get() {
                        self.disable_receive_interrupt();
                        self.rx_status.set(UARTStateRX::Idle);

                        self.rx_client.map(|client| {
                            if let Some(buf) = self.rx_buffer.take() {
                                client.received_buffer(
                                    buf,
                                    self.rx_position.get(),
                                    Ok(()),
                                    hil::uart::Error::None,
                                );
                            }
                        });
                    }
                }
            }
            // If no receive is active, ignore spurious RX level interrupts.
        }
    }

    fn fill_fifo(&self) {
        self.tx_buffer.map(|buf| {
            while self.uart_is_writable() && self.tx_position.get() < self.tx_len.get() {
                let byte = buf[self.tx_position.get()];
                self.send_byte(byte);
                self.tx_position.set(self.tx_position.get() + 1);
            }
        });
    }

    pub fn is_configured(&self) -> bool {
        self.registers.cfg.is_set(CFG::ENABLE)
            && (self.registers.fifocfg.is_set(FIFOCFG::ENABLERX)
                || self.registers.fifocfg.is_set(FIFOCFG::ENABLETX))
    }

    pub fn get_stat_raw(&self) -> u32 {
        self.registers.stat.get()
    }

    pub fn get_fifostat_raw(&self) -> u32 {
        self.registers.fifostat.get()
    }

    pub fn clear_fifo_errors(&self) {
        self.registers
            .fifostat
            .write(FIFOSTAT::TXERR::SET + FIFOSTAT::RXERR::SET);

        self.registers.stat.write(
            STAT::DELTACTS::SET
                + STAT::FRAMERRINT::SET
                + STAT::PARITYERRINT::SET
                + STAT::RXBRK::SET,
        );
    }

    pub fn clear_status_flags_and_fifos(&self) {
        self.registers.stat.write(
            STAT::DELTACTS::SET
                + STAT::FRAMERRINT::SET
                + STAT::PARITYERRINT::SET
                + STAT::RXBRK::SET,
        );

        self.registers
            .fifocfg
            .modify(FIFOCFG::EMPTYTX::SET + FIFOCFG::EMPTYRX::SET);
    }
}

impl Configure for Uart<'_> {
    fn configure(&self, params: Parameters) -> Result<(), ErrorCode> {
        let clocks = self.clocks.get().ok_or(ErrorCode::OFF)?;
        let flexcomm = self.flexcomm.get().ok_or(ErrorCode::OFF)?;
        let clock_source = self.uart_clock_source.get();
        let frg_id = FrgId::from_u32(self.instance.into()).ok_or(ErrorCode::INVAL)?;
        clocks.setup_uart_clock(frg_id, clock_source);
        flexcomm.configure_for_uart();

        // --- Disable USART before configuration ---
        self.registers.cfg.modify(CFG::ENABLE::CLEAR);

        let clk = clocks.get_frg_clock_frequency(clock_source);
        let brg_val = (clk / (16 * params.baud_rate)).saturating_sub(1);
        if brg_val > 0xFFFF {
            return Err(ErrorCode::INVAL); // Baud rate not possible
        }

        self.registers.osr.set(15);
        self.registers.brg.set(51);

        // --- Configure Frame Format (width, parity, stop bits) ---
        let datalen = match params.width {
            Width::Seven => CFG::DATALEN::_7BitDataLength,
            Width::Eight => CFG::DATALEN::_8BitDataLength,
            _ => return Err(ErrorCode::NOSUPPORT), // 6 and 9 bit not handled here
        };

        let paritysel = match params.parity {
            Parity::None => CFG::PARITYSEL::NO_PARITY,
            Parity::Odd => CFG::PARITYSEL::ODD_PARITY,
            Parity::Even => CFG::PARITYSEL::EVEN_PARITY,
        };

        let stoplen = match params.stop_bits {
            StopBits::One => CFG::STOPLEN::_1StopBit,
            StopBits::Two => CFG::STOPLEN::_2StopBits,
        };

        // Write all configuration bits at once
        self.registers.cfg.write(datalen + paritysel + stoplen);

        // --- Configure and Enable FIFOs ---
        // Clear any old data
        self.registers
            .fifocfg
            .modify(FIFOCFG::EMPTYTX::SET + FIFOCFG::EMPTYRX::SET);
        // Enable both TX and RX FIFOs
        self.registers
            .fifocfg
            .modify(FIFOCFG::ENABLETX::SET + FIFOCFG::ENABLERX::SET);
        // Set interrupt trigger levels.
        self.registers
            .fifotrig
            .write(FIFOTRIG::TXLVL.val(1) + FIFOTRIG::RXLVL.val(0));

        // --- Re-enable USART ---
        self.registers.cfg.modify(CFG::ENABLE::SET);

        // A short busy-wait loop is required to allow the peripheral clock
        // to propagate and the internal logic to settle after being re-enabled
        for _ in 0..1500 {
            cortexm33::support::nop();
        }

        Ok(())
    }
}

impl<'a> Transmit<'a> for Uart<'a> {
    fn set_transmit_client(&self, client: &'a dyn TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.tx_status.get() == UARTStateTX::Idle {
            if tx_len <= tx_buffer.len() {
                self.tx_buffer.replace(tx_buffer);
                self.tx_position.set(0);
                self.tx_len.set(tx_len);
                self.tx_status.set(UARTStateTX::Transmitting);

                self.fill_fifo();

                if self.tx_position.get() == self.tx_len.get() {
                    // The entire message fit in the FIFO at once.
                    // Move directly to the "finishing" state.
                    self.set_interrupts_for_finishing();
                } else {
                    // There's more data to send.
                    // Go to the "transmitting" state.
                    self.set_interrupts_for_transmitting();
                }
                Ok(())
            } else {
                Err((ErrorCode::SIZE, tx_buffer))
            }
        } else {
            Err((ErrorCode::BUSY, tx_buffer))
        }
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if self.tx_status.get() != UARTStateTX::Idle {
            self.disable_all_tx_interrupts();
            self.tx_status.set(UARTStateTX::AbortRequested);

            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }
}

impl<'a> Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Check if we are already in the middle of a receive operation.
        if self.rx_status.get() != UARTStateRX::Idle {
            return Err((ErrorCode::BUSY, rx_buffer));
        }

        // Check if the requested length is valid for the provided buffer.
        if rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        self.rx_buffer.replace(rx_buffer);

        // Set up the state for the interrupt handler.
        self.rx_position.set(0);
        self.rx_len.set(rx_len);
        self.rx_status.set(UARTStateRX::Receiving);

        // Enable the hardware interrupt that fires when data arrives.
        self.enable_receive_interrupt();

        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if self.rx_status.get() != UARTStateRX::Idle {
            self.disable_receive_interrupt();
            self.rx_status.set(UARTStateRX::AbortRequested);

            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }
}
