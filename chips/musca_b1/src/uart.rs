// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::hil::uart::ReceiveClient;
use kernel::hil::uart::{
    Configure, Parameters, Parity, Receive, StopBits, Transmit, TransmitClient, Width,
};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::clocks;

register_structs! {

    UartRegisters {
        /// Data Register, UARTDR
        (0x000 => uartdr: ReadWrite<u32, UARTDR::Register>),
        /// Receive Status Register/Error Clear Register, UARTRSR/UARTECR
        (0x004 => uartrsr: ReadWrite<u32, UARTRSR::Register>),
        (0x008 => _reserved0),
        /// Flag Register, UARTFR
        (0x018 => uartfr: ReadWrite<u32, UARTFR::Register>),
        (0x01C => _reserved1),
        /// IrDA Low-Power Counter Register, UARTILPR
        (0x020 => uartilpr: ReadWrite<u32, UARTILPR::Register>),
        /// Integer Baud Rate Register, UARTIBRD
        (0x024 => uartibrd: ReadWrite<u32, UARTIBRD::Register>),
        /// Fractional Baud Rate Register, UARTFBRD
        (0x028 => uartfbrd: ReadWrite<u32, UARTFBRD::Register>),
        /// Line Control Register, UARTLCR_H
        (0x02C => uartlcr_h: ReadWrite<u32, UARTLCR_H::Register>),
        /// Control Register, UARTCR
        (0x030 => uartcr: ReadWrite<u32, UARTCR::Register>),
        /// Interrupt FIFO Level Select Register, UARTIFLS
        (0x034 => uartifls: ReadWrite<u32, UARTIFLS::Register>),
        /// Interrupt Mask Set/Clear Register, UARTIMSC
        (0x038 => uartimsc: ReadWrite<u32, UARTIMSC::Register>),
        /// Raw Interrupt Status Register, UARTRIS
        (0x03C => uartris: ReadWrite<u32, UARTRIS::Register>),
        /// Masked Interrupt Status Register, UARTMIS
        (0x040 => uartmis: ReadWrite<u32, UARTMIS::Register>),
        /// Interrupt Clear Register, UARTICR
        (0x044 => uarticr: ReadWrite<u32, UARTICR::Register>),
        /// DMA Control Register, UARTDMACR
        (0x048 => uartdmacr: ReadWrite<u32, UARTDMACR::Register>),
        (0x04C => _reserved2),
        /// UARTPeriphID0 Register
        (0xFE0 => uartperiphid0: ReadWrite<u32, UARTPERIPHID0::Register>),
        /// UARTPeriphID1 Register
        (0xFE4 => uartperiphid1: ReadWrite<u32, UARTPERIPHID1::Register>),
        /// UARTPeriphID2 Register
        (0xFE8 => uartperiphid2: ReadWrite<u32, UARTPERIPHID2::Register>),
        /// UARTPeriphID3 Register
        (0xFEC => uartperiphid3: ReadWrite<u32, UARTPERIPHID3::Register>),
        /// UARTPCellID0 Register
        (0xFF0 => uartpcellid0: ReadWrite<u32, UARTPCELLID0::Register>),
        /// UARTPCellID1 Register
        (0xFF4 => uartpcellid1: ReadWrite<u32, UARTPCELLID1::Register>),
        /// UARTPCellID2 Register
        (0xFF8 => uartpcellid2: ReadWrite<u32, UARTPCELLID2::Register>),
        /// UARTPCellID3 Register
        (0xFFC => uartpcellid3: ReadWrite<u32, UARTPCELLID3::Register>),
        (0x1000 => @END),
    }
}
register_bitfields![u32,
UARTDR [
    /// Overrun error. This bit is set to 1 if data is received and the receive FIFO is already full. This is cleared to 0 once there is an empty space in the FIFO and a new character can be written to it.
    OE OFFSET(11) NUMBITS(1) [],
    /// Break error. This bit is set to 1 if a break condition was detected, indicating that the received data input was held LOW for longer than a full-word transmission time (defined as start, data, parity and stop bits). In FIFO mode, this error is associated with the character at the top of the FIFO. When a break occurs, only one 0 character is loaded into the FIFO. The next character is only enabled after the receive data input goes to a 1 (marking state), and the next valid start bit is received.
    BE OFFSET(10) NUMBITS(1) [],
    /// Parity error. When set to 1, it indicates that the parity of the received data character does not match the parity that the EPS and SPS bits in the Line Control Register, UARTLCR_H. In FIFO mode, this error is associated with the character at the top of the FIFO.
    PE OFFSET(9) NUMBITS(1) [],
    /// Framing error. When set to 1, it indicates that the received character did not have a valid stop bit (a valid stop bit is 1). In FIFO mode, this error is associated with the character at the top of the FIFO.
    FE OFFSET(8) NUMBITS(1) [],
    /// Receive (read) data character. Transmit (write) data character.
    DATA OFFSET(0) NUMBITS(8) []
],
UARTRSR [
    /// Overrun error. This bit is set to 1 if data is received and the FIFO is already full. This bit is cleared to 0 by a write to UARTECR. The FIFO contents remain valid because no more data is written when the FIFO is full, only the contents of the shift register are overwritten. The CPU must now read the data, to empty the FIFO.
    OE OFFSET(3) NUMBITS(1) [],
    /// Break error. This bit is set to 1 if a break condition was detected, indicating that the received data input was held LOW for longer than a full-word transmission time (defined as start, data, parity, and stop bits). This bit is cleared to 0 after a write to UARTECR. In FIFO mode, this error is associated with the character at the top of the FIFO. When a break occurs, only one 0 character is loaded into the FIFO. The next character is only enabled after the receive data input goes to a 1 (marking state) and the next valid start bit is received.
    BE OFFSET(2) NUMBITS(1) [],
    /// Parity error. When set to 1, it indicates that the parity of the received data character does not match the parity that the EPS and SPS bits in the Line Control Register, UARTLCR_H. This bit is cleared to 0 by a write to UARTECR. In FIFO mode, this error is associated with the character at the top of the FIFO.
    PE OFFSET(1) NUMBITS(1) [],
    /// Framing error. When set to 1, it indicates that the received character did not have a valid stop bit (a valid stop bit is 1). This bit is cleared to 0 by a write to UARTECR. In FIFO mode, this error is associated with the character at the top of the FIFO.
    FE OFFSET(0) NUMBITS(1) []
],
UARTFR [
    /// Ring indicator. This bit is the complement of the UART ring indicator, nUARTRI, modem status input. That is, the bit is 1 when nUARTRI is LOW.
    RI OFFSET(8) NUMBITS(1) [],
    /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the Line Control Register, UARTLCR_H. If the FIFO is disabled, this bit is set when the transmit holding register is empty. If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty. This bit does not indicate if there is data in the transmit shift register.
    TXFE OFFSET(7) NUMBITS(1) [],
    /// Receive FIFO full. The meaning of this bit depends on the state of the FEN bit in the UARTLCR_H Register. If the FIFO is disabled, this bit is set when the receive holding register is full. If the FIFO is enabled, the RXFF bit is set when the receive FIFO is full.
    RXFF OFFSET(6) NUMBITS(1) [],
    /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the UARTLCR_H Register. If the FIFO is disabled, this bit is set when the transmit holding register is full. If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
    TXFF OFFSET(5) NUMBITS(1) [],
    /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the UARTLCR_H Register. If the FIFO is disabled, this bit is set when the receive holding register is empty. If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
    RXFE OFFSET(4) NUMBITS(1) [],
    /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains set until the complete byte, including all the stop bits, has been sent from the shift register. This bit is set as soon as the transmit FIFO becomes non-empty, regardless of whether the UART is enabled or not.
    BUSY OFFSET(3) NUMBITS(1) [],
    /// Data carrier detect. This bit is the complement of the UART data carrier detect, nUARTDCD, modem status input. That is, the bit is 1 when nUARTDCD is LOW.
    DCD OFFSET(2) NUMBITS(1) [],
    /// Data set ready. This bit is the complement of the UART data set ready, nUARTDSR, modem status input. That is, the bit is 1 when nUARTDSR is LOW.
    DSR OFFSET(1) NUMBITS(1) [],
    /// Clear to send. This bit is the complement of the UART clear to send, nUARTCTS, modem status input. That is, the bit is 1 when nUARTCTS is LOW.
    CTS OFFSET(0) NUMBITS(1) []
],
UARTILPR [
    /// 8-bit low-power divisor value. These bits are cleared to 0 at reset.
    ILPDVSR OFFSET(0) NUMBITS(8) []
],
UARTIBRD [
    /// The integer baud rate divisor. These bits are cleared to 0 on reset.
    BAUD_DIVINT OFFSET(0) NUMBITS(16) []
],
UARTFBRD [
    /// The fractional baud rate divisor. These bits are cleared to 0 on reset.
    BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
],
UARTLCR_H [
    /// Stick parity select. 0 = stick parity is disabled 1 = either: * if the EPS bit is 0 then the parity bit is transmitted and checked as a 1 * if the EPS bit is 1 then the parity bit is transmitted and checked as a 0. This bit has no effect when the PEN bit disables parity checking and generation.
    SPS OFFSET(7) NUMBITS(1) [],
    /// Word length. These bits indicate the number of data bits transmitted or received in a frame as follows: b11 = 8 bits b10 = 7 bits b01 = 6 bits b00 = 5 bits.
    WLEN OFFSET(5) NUMBITS(2) [
        BITS_5 = 0b00,
        BITS_6 = 0b01,
        BITS_7 = 0b10,
        BITS_8 = 0b11,
    ],
    /// Enable FIFOs: 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding registers 1 = transmit and receive FIFO buffers are enabled (FIFO mode).
    FEN OFFSET(4) NUMBITS(1) [],
    /// Two stop bits select. If this bit is set to 1, two stop bits are transmitted at the end of the frame. The receive logic does not check for two stop bits being received.
    STP2 OFFSET(3) NUMBITS(1) [],
    /// Even parity select. Controls the type of parity the UART uses during transmission and reception: 0 = odd parity. The UART generates or checks for an odd number of 1s in the data and parity bits. 1 = even parity. The UART generates or checks for an even number of 1s in the data and parity bits. This bit has no effect when the PEN bit disables parity checking and generation.
    EPS OFFSET(2) NUMBITS(1) [],
    /// Parity enable: 0 = parity is disabled and no parity bit added to the data frame 1 = parity checking and generation is enabled.
    PEN OFFSET(1) NUMBITS(1) [],
    /// Send break. If this bit is set to 1, a low-level is continually output on the UARTTXD output, after completing transmission of the current character. For the proper execution of the break command, the software must set this bit for at least two complete frames. For normal use, this bit must be cleared to 0.
    BRK OFFSET(0) NUMBITS(1) []
],
UARTCR [
    /// CTS hardware flow control enable. If this bit is set to 1, CTS hardware flow control is enabled. Data is only transmitted when the nUARTCTS signal is asserted.
    CTSEN OFFSET(15) NUMBITS(1) [],
    /// RTS hardware flow control enable. If this bit is set to 1, RTS hardware flow control is enabled. Data is only requested when there is space in the receive FIFO for it to be received.
    RTSEN OFFSET(14) NUMBITS(1) [],
    /// This bit is the complement of the UART Out2 (nUARTOut2) modem status output. That is, when the bit is programmed to a 1, the output is 0. For DTE this can be used as Ring Indicator (RI).
    OUT2 OFFSET(13) NUMBITS(1) [],
    /// This bit is the complement of the UART Out1 (nUARTOut1) modem status output. That is, when the bit is programmed to a 1 the output is 0. For DTE this can be used as Data Carrier Detect (DCD).
    OUT1 OFFSET(12) NUMBITS(1) [],
    /// Request to send. This bit is the complement of the UART request to send, nUARTRTS, modem status output. That is, when the bit is programmed to a 1 then nUARTRTS is LOW.
    RTS OFFSET(11) NUMBITS(1) [],
    /// Data transmit ready. This bit is the complement of the UART data transmit ready, nUARTDTR, modem status output. That is, when the bit is programmed to a 1 then nUARTDTR is LOW.
    DTR OFFSET(10) NUMBITS(1) [],
    /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled. Data reception occurs for either UART signals or SIR signals depending on the setting of the SIREN bit. When the UART is disabled in the middle of reception, it completes the current character before stopping.
    RXE OFFSET(9) NUMBITS(1) [],
    /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled. Data transmission occurs for either UART signals, or SIR signals depending on the setting of the SIREN bit. When the UART is disabled in the middle of transmission, it completes the current character before stopping.
    TXE OFFSET(8) NUMBITS(1) [],
    /// Loopback enable. If this bit is set to 1 and the SIREN bit is set to 1 and the SIRTEST bit in the Test Control Register, UARTTCR is set to 1, then the nSIROUT path is inverted, and fed through to the SIRIN path. The SIRTEST bit in the test register must be set to 1 to override the normal half-duplex SIR operation. This must be the requirement for accessing the test registers during normal operation, and SIRTEST must be cleared to 0 when loopback testing is finished. This feature reduces the amount of external coupling required during system test. If this bit is set to 1, and the SIRTEST bit is set to 0, the UARTTXD path is fed through to the UARTRXD path. In either SIR mode or UART mode, when this bit is set, the modem outputs are also fed through to the modem inputs. This bit is cleared to 0 on reset, to disable loopback.
    LBE OFFSET(7) NUMBITS(1) [],
    /// SIR low-power IrDA mode. This bit selects the IrDA encoding mode. If this bit is cleared to 0, low-level bits are transmitted as an active high pulse with a width of 3 / 16th of the bit period. If this bit is set to 1, low-level bits are transmitted with a pulse width that is 3 times the period of the IrLPBaud16 input signal, regardless of the selected bit rate. Setting this bit uses less power, but might reduce transmission distances.
    SIRLP OFFSET(2) NUMBITS(1) [],
    /// SIR enable: 0 = IrDA SIR ENDEC is disabled. nSIROUT remains LOW (no light pulse generated), and signal transitions on SIRIN have no effect. 1 = IrDA SIR ENDEC is enabled. Data is transmitted and received on nSIROUT and SIRIN. UARTTXD remains HIGH, in the marking state. Signal transitions on UARTRXD or modem status inputs have no effect. This bit has no effect if the UARTEN bit disables the UART.
    SIREN OFFSET(1) NUMBITS(1) [],
    /// UART enable: 0 = UART is disabled. If the UART is disabled in the middle of transmission or reception, it completes the current character before stopping. 1 = the UART is enabled. Data transmission and reception occurs for either UART signals or SIR signals depending on the setting of the SIREN bit.
    UARTEN OFFSET(0) NUMBITS(1) []
],
UARTIFLS [
    /// Receive interrupt FIFO level select. The trigger points for the receive interrupt are as follows: b000 = Receive FIFO becomes >= 1 / 8 full b001 = Receive FIFO becomes >= 1 / 4 full b010 = Receive FIFO becomes >= 1 / 2 full b011 = Receive FIFO becomes >= 3 / 4 full b100 = Receive FIFO becomes >= 7 / 8 full b101-b111 = reserved.
    RXIFLSEL OFFSET(3) NUMBITS(3) [
        FIFO_1_8 = 0b000,
        FIFO_1_4 = 0b001,
        FIFO_1_2 = 0b010,
        FIFO_3_4 = 0b011,
        FIFO_7_8 = 0b100,
    ],
    /// Transmit interrupt FIFO level select. The trigger points for the transmit interrupt are as follows: b000 = Transmit FIFO becomes <= 1 / 8 full b001 = Transmit FIFO becomes <= 1 / 4 full b010 = Transmit FIFO becomes <= 1 / 2 full b011 = Transmit FIFO becomes <= 3 / 4 full b100 = Transmit FIFO becomes <= 7 / 8 full b101-b111 = reserved.
    TXIFLSEL OFFSET(0) NUMBITS(3) []
],
UARTIMSC [
    /// Overrun error interrupt mask. A read returns the current mask for the UARTOEINTR interrupt. On a write of 1, the mask of the UARTOEINTR interrupt is set. A write of 0 clears the mask.
    OEIM OFFSET(10) NUMBITS(1) [],
    /// Break error interrupt mask. A read returns the current mask for the UARTBEINTR interrupt. On a write of 1, the mask of the UARTBEINTR interrupt is set. A write of 0 clears the mask.
    BEIM OFFSET(9) NUMBITS(1) [],
    /// Parity error interrupt mask. A read returns the current mask for the UARTPEINTR interrupt. On a write of 1, the mask of the UARTPEINTR interrupt is set. A write of 0 clears the mask.
    PEIM OFFSET(8) NUMBITS(1) [],
    /// Framing error interrupt mask. A read returns the current mask for the UARTFEINTR interrupt. On a write of 1, the mask of the UARTFEINTR interrupt is set. A write of 0 clears the mask.
    FEIM OFFSET(7) NUMBITS(1) [],
    /// Receive timeout interrupt mask. A read returns the current mask for the UARTRTINTR interrupt. On a write of 1, the mask of the UARTRTINTR interrupt is set. A write of 0 clears the mask.
    RTIM OFFSET(6) NUMBITS(1) [],
    /// Transmit interrupt mask. A read returns the current mask for the UARTTXINTR interrupt. On a write of 1, the mask of the UARTTXINTR interrupt is set. A write of 0 clears the mask.
    TXIM OFFSET(5) NUMBITS(1) [],
    /// Receive interrupt mask. A read returns the current mask for the UARTRXINTR interrupt. On a write of 1, the mask of the UARTRXINTR interrupt is set. A write of 0 clears the mask.
    RXIM OFFSET(4) NUMBITS(1) [],
    /// nUARTDSR modem interrupt mask. A read returns the current mask for the UARTDSRINTR interrupt. On a write of 1, the mask of the UARTDSRINTR interrupt is set. A write of 0 clears the mask.
    DSRMIM OFFSET(3) NUMBITS(1) [],
    /// nUARTDCD modem interrupt mask. A read returns the current mask for the UARTDCDINTR interrupt. On a write of 1, the mask of the UARTDCDINTR interrupt is set. A write of 0 clears the mask.
    DCDMIM OFFSET(2) NUMBITS(1) [],
    /// nUARTCTS modem interrupt mask. A read returns the current mask for the UARTCTSINTR interrupt. On a write of 1, the mask of the UARTCTSINTR interrupt is set. A write of 0 clears the mask.
    CTSMIM OFFSET(1) NUMBITS(1) [],
    /// nUARTRI modem interrupt mask. A read returns the current mask for the UARTRIINTR interrupt. On a write of 1, the mask of the UARTRIINTR interrupt is set. A write of 0 clears the mask.
    RIMIM OFFSET(0) NUMBITS(1) []
],
UARTRIS [
    /// Overrun error interrupt status. Returns the raw interrupt state of the UARTOEINTR interrupt.
    OERIS OFFSET(10) NUMBITS(1) [],
    /// Break error interrupt status. Returns the raw interrupt state of the UARTBEINTR interrupt.
    BERIS OFFSET(9) NUMBITS(1) [],
    /// Parity error interrupt status. Returns the raw interrupt state of the UARTPEINTR interrupt.
    PERIS OFFSET(8) NUMBITS(1) [],
    /// Framing error interrupt status. Returns the raw interrupt state of the UARTFEINTR interrupt.
    FERIS OFFSET(7) NUMBITS(1) [],
    /// Receive timeout interrupt status. Returns the raw interrupt state of the UARTRTINTR interrupt. a
    RTRIS OFFSET(6) NUMBITS(1) [],
    /// Transmit interrupt status. Returns the raw interrupt state of the UARTTXINTR interrupt.
    TXRIS OFFSET(5) NUMBITS(1) [],
    /// Receive interrupt status. Returns the raw interrupt state of the UARTRXINTR interrupt.
    RXRIS OFFSET(4) NUMBITS(1) [],
    /// nUARTDSR modem interrupt status. Returns the raw interrupt state of the UARTDSRINTR interrupt.
    DSRRMIS OFFSET(3) NUMBITS(1) [],
    /// nUARTDCD modem interrupt status. Returns the raw interrupt state of the UARTDCDINTR interrupt.
    DCDRMIS OFFSET(2) NUMBITS(1) [],
    /// nUARTCTS modem interrupt status. Returns the raw interrupt state of the UARTCTSINTR interrupt.
    CTSRMIS OFFSET(1) NUMBITS(1) [],
    /// nUARTRI modem interrupt status. Returns the raw interrupt state of the UARTRIINTR interrupt.
    RIRMIS OFFSET(0) NUMBITS(1) []
],
UARTMIS [
    /// Overrun error masked interrupt status. Returns the masked interrupt state of the UARTOEINTR interrupt.
    OEMIS OFFSET(10) NUMBITS(1) [],
    /// Break error masked interrupt status. Returns the masked interrupt state of the UARTBEINTR interrupt.
    BEMIS OFFSET(9) NUMBITS(1) [],
    /// Parity error masked interrupt status. Returns the masked interrupt state of the UARTPEINTR interrupt.
    PEMIS OFFSET(8) NUMBITS(1) [],
    /// Framing error masked interrupt status. Returns the masked interrupt state of the UARTFEINTR interrupt.
    FEMIS OFFSET(7) NUMBITS(1) [],
    /// Receive timeout masked interrupt status. Returns the masked interrupt state of the UARTRTINTR interrupt.
    RTMIS OFFSET(6) NUMBITS(1) [],
    /// Transmit masked interrupt status. Returns the masked interrupt state of the UARTTXINTR interrupt.
    TXMIS OFFSET(5) NUMBITS(1) [],
    /// Receive masked interrupt status. Returns the masked interrupt state of the UARTRXINTR interrupt.
    RXMIS OFFSET(4) NUMBITS(1) [],
    /// nUARTDSR modem masked interrupt status. Returns the masked interrupt state of the UARTDSRINTR interrupt.
    DSRMMIS OFFSET(3) NUMBITS(1) [],
    /// nUARTDCD modem masked interrupt status. Returns the masked interrupt state of the UARTDCDINTR interrupt.
    DCDMMIS OFFSET(2) NUMBITS(1) [],
    /// nUARTCTS modem masked interrupt status. Returns the masked interrupt state of the UARTCTSINTR interrupt.
    CTSMMIS OFFSET(1) NUMBITS(1) [],
    /// nUARTRI modem masked interrupt status. Returns the masked interrupt state of the UARTRIINTR interrupt.
    RIMMIS OFFSET(0) NUMBITS(1) []
],
UARTICR [
    /// Overrun error interrupt clear. Clears the UARTOEINTR interrupt.
    OEIC OFFSET(10) NUMBITS(1) [],
    /// Break error interrupt clear. Clears the UARTBEINTR interrupt.
    BEIC OFFSET(9) NUMBITS(1) [],
    /// Parity error interrupt clear. Clears the UARTPEINTR interrupt.
    PEIC OFFSET(8) NUMBITS(1) [],
    /// Framing error interrupt clear. Clears the UARTFEINTR interrupt.
    FEIC OFFSET(7) NUMBITS(1) [],
    /// Receive timeout interrupt clear. Clears the UARTRTINTR interrupt.
    RTIC OFFSET(6) NUMBITS(1) [],
    /// Transmit interrupt clear. Clears the UARTTXINTR interrupt.
    TXIC OFFSET(5) NUMBITS(1) [],
    /// Receive interrupt clear. Clears the UARTRXINTR interrupt.
    RXIC OFFSET(4) NUMBITS(1) [],
    /// nUARTDSR modem interrupt clear. Clears the UARTDSRINTR interrupt.
    DSRMIC OFFSET(3) NUMBITS(1) [],
    /// nUARTDCD modem interrupt clear. Clears the UARTDCDINTR interrupt.
    DCDMIC OFFSET(2) NUMBITS(1) [],
    /// nUARTCTS modem interrupt clear. Clears the UARTCTSINTR interrupt.
    CTSMIC OFFSET(1) NUMBITS(1) [],
    /// nUARTRI modem interrupt clear. Clears the UARTRIINTR interrupt.
    RIMIC OFFSET(0) NUMBITS(1) []
],
UARTDMACR [
    /// DMA on error. If this bit is set to 1, the DMA receive request outputs, UARTRXDMASREQ or UARTRXDMABREQ, are disabled when the UART error interrupt is asserted.
    DMAONERR OFFSET(2) NUMBITS(1) [],
    /// Transmit DMA enable. If this bit is set to 1, DMA for the transmit FIFO is enabled.
    TXDMAE OFFSET(1) NUMBITS(1) [],
    /// Receive DMA enable. If this bit is set to 1, DMA for the receive FIFO is enabled.
    RXDMAE OFFSET(0) NUMBITS(1) []
],
UARTPERIPHID0 [
    /// These bits read back as 0x11
    PARTNUMBER0 OFFSET(0) NUMBITS(8) []
],
UARTPERIPHID1 [
    /// These bits read back as 0x1
    DESIGNER0 OFFSET(4) NUMBITS(4) [],
    /// These bits read back as 0x0
    PARTNUMBER1 OFFSET(0) NUMBITS(4) []
],
UARTPERIPHID2 [
    /// This field depends on the revision of the UART: r1p0 0x0 r1p1 0x1 r1p3 0x2 r1p4 0x2 r1p5 0x3
    REVISION OFFSET(4) NUMBITS(4) [],
    /// These bits read back as 0x4
    DESIGNER1 OFFSET(0) NUMBITS(4) []
],
UARTPERIPHID3 [
    /// These bits read back as 0x00
    CONFIGURATION OFFSET(0) NUMBITS(8) []
],
UARTPCELLID0 [
    /// These bits read back as 0x0D
    UARTPCELLID0 OFFSET(0) NUMBITS(8) []
],
UARTPCELLID1 [
    /// These bits read back as 0xF0
    UARTPCELLID1 OFFSET(0) NUMBITS(8) []
],
UARTPCELLID2 [
    /// These bits read back as 0x05
    UARTPCELLID2 OFFSET(0) NUMBITS(8) []
],
UARTPCELLID3 [
    /// These bits read back as 0xB1
    UARTPCELLID3 OFFSET(0) NUMBITS(8) []
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

const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x40070000 as *const UartRegisters) };

const UART1_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x40078000 as *const UartRegisters) };

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
    clocks: OptionalCell<&'a clocks::Clocks>,

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

    deferred_call: DeferredCall,
}

impl<'a> Uart<'a> {
    pub fn new_uart0() -> Self {
        Self {
            registers: UART0_BASE,
            clocks: OptionalCell::empty(),

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

            deferred_call: DeferredCall::new(),
        }
    }
    pub fn new_uart1() -> Self {
        Self {
            registers: UART1_BASE,
            clocks: OptionalCell::empty(),

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

            deferred_call: DeferredCall::new(),
        }
    }

    pub(crate) fn set_clocks(&self, clocks: &'a clocks::Clocks) {
        self.clocks.set(clocks);
    }

    pub fn enable(&self) {
        self.registers.uartcr.modify(UARTCR::UARTEN::SET);
    }

    pub fn disable(&self) {
        self.registers.uartcr.modify(UARTCR::UARTEN::CLEAR);
    }

    pub fn enable_transmit_interrupt(&self) {
        self.registers.uartimsc.modify(UARTIMSC::TXIM::SET);
    }

    pub fn disable_transmit_interrupt(&self) {
        self.registers.uartimsc.modify(UARTIMSC::TXIM::CLEAR);
    }

    pub fn enable_receive_interrupt(&self) {
        self.registers.uartifls.modify(UARTIFLS::RXIFLSEL::FIFO_1_8);

        self.registers.uartimsc.modify(UARTIMSC::RXIM::SET);
    }

    pub fn disable_receive_interrupt(&self) {
        self.registers.uartimsc.modify(UARTIMSC::RXIM::CLEAR);
    }

    fn uart_is_writable(&self) -> bool {
        !self.registers.uartfr.is_set(UARTFR::TXFF)
    }

    pub fn send_byte(&self, data: u8) {
        while !self.uart_is_writable() {}
        self.registers.uartdr.write(UARTDR::DATA.val(data as u32));
    }

    pub fn handle_interrupt(&self) {
        if self.registers.uartimsc.is_set(UARTIMSC::TXIM) {
            if self.registers.uartfr.is_set(UARTFR::TXFE) {
                if self.tx_status.get() == UARTStateTX::Idle {
                    kernel::debug!("No data to transmit");
                } else if self.tx_status.get() == UARTStateTX::Transmitting {
                    self.disable_transmit_interrupt();
                    if self.tx_position.get() < self.tx_len.get() {
                        self.fill_fifo();
                        self.enable_transmit_interrupt();
                    }
                    // Transmission is done
                    else {
                        self.tx_status.set(UARTStateTX::Idle);
                        self.tx_client.map(|client| {
                            self.tx_buffer.take().map(|buf| {
                                client.transmitted_buffer(buf, self.tx_position.get(), Ok(()));
                            });
                        });
                    }
                }
            }
        }

        if self.registers.uartimsc.is_set(UARTIMSC::RXIM) {
            if self.registers.uartfr.is_set(UARTFR::RXFF) {
                let byte = self.registers.uartdr.get() as u8;

                self.disable_receive_interrupt();
                if self.rx_status.get() == UARTStateRX::Receiving {
                    if self.rx_position.get() < self.rx_len.get() {
                        self.rx_buffer.map(|buf| {
                            buf[self.rx_position.get()] = byte;
                            self.rx_position.replace(self.rx_position.get() + 1);
                        });
                    }
                    if self.rx_position.get() == self.rx_len.get() {
                        // reception done
                        self.rx_status.replace(UARTStateRX::Idle);
                    } else {
                        self.enable_receive_interrupt();
                    }
                    // notify client if transfer is done
                    if self.rx_status.get() == UARTStateRX::Idle {
                        self.rx_client.map(|client| {
                            if let Some(buf) = self.rx_buffer.take() {
                                client.received_buffer(
                                    buf,
                                    self.rx_len.get(),
                                    Ok(()),
                                    hil::uart::Error::None,
                                );
                            }
                        });
                    }
                }
            }
        }
    }

    fn fill_fifo(&self) {
        while self.uart_is_writable() && self.tx_position.get() < self.tx_len.get() {
            self.tx_buffer.map(|buf| {
                self.registers
                    .uartdr
                    .set(buf[self.tx_position.get()].into());
                self.tx_position.replace(self.tx_position.get() + 1);
            });
        }
    }

    pub fn is_configured(&self) -> bool {
        self.registers.uartcr.is_set(UARTCR::UARTEN)
            && (self.registers.uartcr.is_set(UARTCR::RXE)
                || self.registers.uartcr.is_set(UARTCR::TXE))
    }

    pub fn debug_configure(&self, params: Parameters) -> Result<(), ErrorCode> {
        self.disable();
        self.registers.uartlcr_h.modify(UARTLCR_H::FEN::CLEAR);

        let clk = self.clocks.map_or(125_000_000, |clocks| {
            clocks.get_frequency(clocks::Clock::Peripheral)
        });

        // Calculate baud rate
        let baud_rate_div = 8 * clk / params.baud_rate;
        let mut baud_ibrd = baud_rate_div >> 7;
        let mut baud_fbrd = (baud_rate_div & 0x7f).div_ceil(2);

        if baud_ibrd == 0 {
            baud_ibrd = 1;
            baud_fbrd = 0;
        } else if baud_ibrd >= 65535 {
            baud_ibrd = 65535;
            baud_fbrd = 0;
        }

        self.registers
            .uartibrd
            .write(UARTIBRD::BAUD_DIVINT.val(baud_ibrd));
        self.registers
            .uartfbrd
            .write(UARTFBRD::BAUD_DIVFRAC.val(baud_fbrd));

        self.registers.uartlcr_h.modify(UARTLCR_H::BRK::SET);
        // Configure the word length
        match params.width {
            Width::Six => self.registers.uartlcr_h.modify(UARTLCR_H::WLEN::BITS_6),
            Width::Seven => self.registers.uartlcr_h.modify(UARTLCR_H::WLEN::BITS_7),
            Width::Eight => self.registers.uartlcr_h.modify(UARTLCR_H::WLEN::BITS_8),
        }

        // Configure parity
        match params.parity {
            Parity::None => {
                self.registers.uartlcr_h.modify(UARTLCR_H::PEN::CLEAR);
                self.registers.uartlcr_h.modify(UARTLCR_H::EPS::CLEAR);
            }

            Parity::Odd => {
                self.registers.uartlcr_h.modify(UARTLCR_H::PEN::SET);
                self.registers.uartlcr_h.modify(UARTLCR_H::EPS::CLEAR);
            }
            Parity::Even => {
                self.registers.uartlcr_h.modify(UARTLCR_H::PEN::SET);
                self.registers.uartlcr_h.modify(UARTLCR_H::EPS::SET);
            }
        }

        // Set the stop bit length - 2 stop bits
        match params.stop_bits {
            StopBits::One => self.registers.uartlcr_h.modify(UARTLCR_H::STP2::CLEAR),
            StopBits::Two => self.registers.uartlcr_h.modify(UARTLCR_H::STP2::SET),
        }

        // Set flow control
        if params.hw_flow_control {
            self.registers.uartcr.modify(UARTCR::RTSEN::SET);
            self.registers.uartcr.modify(UARTCR::CTSEN::SET);
        } else {
            self.registers.uartcr.modify(UARTCR::RTSEN::CLEAR);
            self.registers.uartcr.modify(UARTCR::CTSEN::CLEAR);
        }
        self.registers.uartlcr_h.modify(UARTLCR_H::BRK::CLEAR);

        // FIFO is not precise enough for receive
        self.registers.uartlcr_h.modify(UARTLCR_H::FEN::CLEAR);

        // Enable uart and transmit
        self.registers
            .uartcr
            .modify(UARTCR::UARTEN::SET + UARTCR::TXE::SET + UARTCR::RXE::SET);

        self.registers
            .uartdmacr
            .write(UARTDMACR::TXDMAE::SET + UARTDMACR::RXDMAE::SET);

        Ok(())
    }
}

impl DeferredCallClient for Uart<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self)
    }

    fn handle_deferred_call(&self) {
        if self.tx_status.get() == UARTStateTX::AbortRequested {
            // alert client
            self.tx_client.map(|client| {
                self.tx_buffer.take().map(|buf| {
                    client.transmitted_buffer(buf, self.tx_position.get(), Err(ErrorCode::CANCEL));
                });
            });
            self.tx_status.set(UARTStateTX::Idle);
        }

        if self.rx_status.get() == UARTStateRX::AbortRequested {
            // alert client
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|buf| {
                    client.received_buffer(
                        buf,
                        self.rx_position.get(),
                        Err(ErrorCode::CANCEL),
                        hil::uart::Error::Aborted,
                    );
                });
            });
            self.rx_status.set(UARTStateRX::Idle);
        }
    }
}

impl Configure for Uart<'_> {
    fn configure(&self, params: Parameters) -> Result<(), ErrorCode> {
        self.disable();
        self.registers.uartlcr_h.modify(UARTLCR_H::FEN::CLEAR);

        let clk = self.clocks.map_or(125_000_000, |clocks| {
            clocks.get_frequency(clocks::Clock::Peripheral)
        });

        // Calculate baud rate
        let baud_rate_div = 8 * clk / params.baud_rate;
        let mut baud_ibrd = baud_rate_div >> 7;
        let mut baud_fbrd = (baud_rate_div & 0x7f).div_ceil(2);

        if baud_ibrd == 0 {
            baud_ibrd = 1;
            baud_fbrd = 0;
        } else if baud_ibrd >= 65535 {
            baud_ibrd = 65535;
            baud_fbrd = 0;
        }

        self.registers
            .uartibrd
            .write(UARTIBRD::BAUD_DIVINT.val(baud_ibrd));
        self.registers
            .uartfbrd
            .write(UARTFBRD::BAUD_DIVFRAC.val(baud_fbrd));

        self.registers.uartlcr_h.modify(UARTLCR_H::BRK::SET);
        // Configure the word length
        match params.width {
            Width::Six => self.registers.uartlcr_h.modify(UARTLCR_H::WLEN::BITS_6),
            Width::Seven => self.registers.uartlcr_h.modify(UARTLCR_H::WLEN::BITS_7),
            Width::Eight => self.registers.uartlcr_h.modify(UARTLCR_H::WLEN::BITS_8),
        }

        // Configure parity
        match params.parity {
            Parity::None => {
                self.registers.uartlcr_h.modify(UARTLCR_H::PEN::CLEAR);
                self.registers.uartlcr_h.modify(UARTLCR_H::EPS::CLEAR);
            }

            Parity::Odd => {
                self.registers.uartlcr_h.modify(UARTLCR_H::PEN::SET);
                self.registers.uartlcr_h.modify(UARTLCR_H::EPS::CLEAR);
            }
            Parity::Even => {
                self.registers.uartlcr_h.modify(UARTLCR_H::PEN::SET);
                self.registers.uartlcr_h.modify(UARTLCR_H::EPS::SET);
            }
        }

        // Set the stop bit length - 2 stop bits
        match params.stop_bits {
            StopBits::One => self.registers.uartlcr_h.modify(UARTLCR_H::STP2::CLEAR),
            StopBits::Two => self.registers.uartlcr_h.modify(UARTLCR_H::STP2::SET),
        }

        // Set flow control
        if params.hw_flow_control {
            self.registers.uartcr.modify(UARTCR::RTSEN::SET);
            self.registers.uartcr.modify(UARTCR::CTSEN::SET);
        } else {
            self.registers.uartcr.modify(UARTCR::RTSEN::CLEAR);
            self.registers.uartcr.modify(UARTCR::CTSEN::CLEAR);
        }
        self.registers.uartlcr_h.modify(UARTLCR_H::BRK::CLEAR);

        // FIFO is not precise enough for receive
        self.registers.uartlcr_h.modify(UARTLCR_H::FEN::CLEAR);

        // Enable uart and transmit
        self.registers
            .uartcr
            .modify(UARTCR::UARTEN::SET + UARTCR::TXE::SET + UARTCR::RXE::SET);

        self.registers
            .uartdmacr
            .write(UARTDMACR::TXDMAE::SET + UARTDMACR::RXDMAE::SET);

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
                self.tx_buffer.put(Some(tx_buffer));
                self.tx_position.set(0);
                self.tx_len.set(tx_len);
                self.tx_status.set(UARTStateTX::Transmitting);
                self.enable_transmit_interrupt();
                self.fill_fifo();
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
            self.disable_transmit_interrupt();
            self.tx_status.set(UARTStateTX::AbortRequested);

            self.deferred_call.set();

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
        if self.rx_status.get() == UARTStateRX::Idle {
            if rx_len <= rx_buffer.len() {
                self.rx_buffer.put(Some(rx_buffer));
                self.rx_position.set(0);
                self.rx_len.set(rx_len);
                self.rx_status.set(UARTStateRX::Receiving);
                self.enable_receive_interrupt();
                Ok(())
            } else {
                Err((ErrorCode::SIZE, rx_buffer))
            }
        } else {
            Err((ErrorCode::BUSY, rx_buffer))
        }
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if self.rx_status.get() != UARTStateRX::Idle {
            self.disable_receive_interrupt();
            self.rx_status.set(UARTStateRX::AbortRequested);

            self.deferred_call.set();

            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }
}
