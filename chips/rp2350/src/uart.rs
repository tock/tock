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
    /// controls serial port
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
        (0xFE0 => uartperiphid0: ReadWrite<u32>),
        /// UARTPeriphID1 Register
        (0xFE4 => uartperiphid1: ReadWrite<u32, UARTPERIPHID1::Register>),
        /// UARTPeriphID2 Register
        (0xFE8 => uartperiphid2: ReadWrite<u32, UARTPERIPHID2::Register>),
        /// UARTPeriphID3 Register
        (0xFEC => uartperiphid3: ReadWrite<u32>),
        /// UARTPCellID0 Register
        (0xFF0 => uartpcellid0: ReadWrite<u32>),
        /// UARTPCellID1 Register
        (0xFF4 => uartpcellid1: ReadWrite<u32>),
        /// UARTPCellID2 Register
        (0xFF8 => uartpcellid2: ReadWrite<u32>),
        /// UARTPCellID3 Register
        (0xFFC => uartpcellid3: ReadWrite<u32>),
        (0x1000 => @END),
    }
}

register_bitfields![u32,
UARTDR [
    /// Overrun error. This bit is set to 1 if data is received and the receive FIFO is
    OE OFFSET(11) NUMBITS(1) [],
    /// Break error. This bit is set to 1 if a break condition was detected, indicating
    BE OFFSET(10) NUMBITS(1) [],
    /// Parity error. When set to 1, it indicates that the parity of the received data c
    PE OFFSET(9) NUMBITS(1) [],
    /// Framing error. When set to 1, it indicates that the received character did not h
    FE OFFSET(8) NUMBITS(1) [],
    /// Receive (read) data character. Transmit (write) data character.
    DATA OFFSET(0) NUMBITS(8) []
],
UARTRSR [
    /// Overrun error. This bit is set to 1 if data is received and the FIFO is already
    OE OFFSET(3) NUMBITS(1) [],
    /// Break error. This bit is set to 1 if a break condition was detected, indicating
    BE OFFSET(2) NUMBITS(1) [],
    /// Parity error. When set to 1, it indicates that the parity of the received data c
    PE OFFSET(1) NUMBITS(1) [],
    /// Framing error. When set to 1, it indicates that the received character did not h
    FE OFFSET(0) NUMBITS(1) []
],
UARTFR [
    /// Ring indicator. This bit is the complement of the UART ring indicator, nUARTRI,
    RI OFFSET(8) NUMBITS(1) [],
    /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit
    TXFE OFFSET(7) NUMBITS(1) [],
    /// Receive FIFO full. The meaning of this bit depends on the state of the FEN bit i
    RXFF OFFSET(6) NUMBITS(1) [],
    /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit
    TXFF OFFSET(5) NUMBITS(1) [],
    /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit
    RXFE OFFSET(4) NUMBITS(1) [],
    /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit
    BUSY OFFSET(3) NUMBITS(1) [],
    /// Data carrier detect. This bit is the complement of the UART data carrier detect,
    DCD OFFSET(2) NUMBITS(1) [],
    /// Data set ready. This bit is the complement of the UART data set ready, nUARTDSR,
    DSR OFFSET(1) NUMBITS(1) [],
    /// Clear to send. This bit is the complement of the UART clear to send, nUARTCTS, m
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
    /// Stick parity select. 0 = stick parity is disabled 1 = either: * if the EPS bit i
    SPS OFFSET(7) NUMBITS(1) [],
    /// Word length. These bits indicate the number of data bits transmitted or received
    WLEN OFFSET(5) NUMBITS(2) [
        BITS_5 = 0b00,
        BITS_6 = 0b01,
        BITS_7 = 0b10,
        BITS_8 = 0b11,
    ],
    /// Enable FIFOs: 0 = FIFOs are disabled (character mode) that is, the FIFOs become
    FEN OFFSET(4) NUMBITS(1) [],
    /// Two stop bits select. If this bit is set to 1, two stop bits are transmitted at
    STP2 OFFSET(3) NUMBITS(1) [],
    /// Even parity select. Controls the type of parity the UART uses during transmissio
    EPS OFFSET(2) NUMBITS(1) [],
    /// Parity enable: 0 = parity is disabled and no parity bit added to the data frame
    PEN OFFSET(1) NUMBITS(1) [],
    /// Send break. If this bit is set to 1, a low-level is continually output on the UA
    BRK OFFSET(0) NUMBITS(1) []
],
UARTCR [
    /// CTS hardware flow control enable. If this bit is set to 1, CTS hardware flow con
    CTSEN OFFSET(15) NUMBITS(1) [],
    /// RTS hardware flow control enable. If this bit is set to 1, RTS hardware flow con
    RTSEN OFFSET(14) NUMBITS(1) [],
    /// This bit is the complement of the UART Out2 (nUARTOut2) modem status output. Tha
    OUT2 OFFSET(13) NUMBITS(1) [],
    /// This bit is the complement of the UART Out1 (nUARTOut1) modem status output. Tha
    OUT1 OFFSET(12) NUMBITS(1) [],
    /// Request to send. This bit is the complement of the UART request to send, nUARTRT
    RTS OFFSET(11) NUMBITS(1) [],
    /// Data transmit ready. This bit is the complement of the UART data transmit ready,
    DTR OFFSET(10) NUMBITS(1) [],
    /// Receive enable. If this bit is set to 1, the receive section of the UART is enab
    RXE OFFSET(9) NUMBITS(1) [],
    /// Transmit enable. If this bit is set to 1, the transmit section of the UART is en
    TXE OFFSET(8) NUMBITS(1) [],
    /// Loopback enable. If this bit is set to 1 and the SIREN bit is set to 1 and the S
    LBE OFFSET(7) NUMBITS(1) [],
    /// SIR low-power IrDA mode. This bit selects the IrDA encoding mode. If this bit is
    SIRLP OFFSET(2) NUMBITS(1) [],
    /// SIR enable: 0 = IrDA SIR ENDEC is disabled. nSIROUT remains LOW (no light pulse
    SIREN OFFSET(1) NUMBITS(1) [],
    /// UART enable: 0 = UART is disabled. If the UART is disabled in the middle of tran
    UARTEN OFFSET(0) NUMBITS(1) []
],
UARTIFLS [
    /// Receive interrupt FIFO level select. The trigger points for the receive interrup
    RXIFLSEL OFFSET(3) NUMBITS(3) [
        FIFO_1_8 = 0b000,
        FIFO_1_4 = 0b001,
        FIFO_1_2 = 0b010,
        FIFO_3_4 = 0b011,
        FIFO_7_8 = 0b100,
    ],
    /// Transmit interrupt FIFO level select. The trigger points for the transmit interr
    TXIFLSEL OFFSET(0) NUMBITS(3) []
],
UARTIMSC [
    /// Overrun error interrupt mask. A read returns the current mask for the UARTOEINTR
    OEIM OFFSET(10) NUMBITS(1) [],
    /// Break error interrupt mask. A read returns the current mask for the UARTBEINTR i
    BEIM OFFSET(9) NUMBITS(1) [],
    /// Parity error interrupt mask. A read returns the current mask for the UARTPEINTR
    PEIM OFFSET(8) NUMBITS(1) [],
    /// Framing error interrupt mask. A read returns the current mask for the UARTFEINTR
    FEIM OFFSET(7) NUMBITS(1) [],
    /// Receive timeout interrupt mask. A read returns the current mask for the UARTRTIN
    RTIM OFFSET(6) NUMBITS(1) [],
    /// Transmit interrupt mask. A read returns the current mask for the UARTTXINTR inte
    TXIM OFFSET(5) NUMBITS(1) [],
    /// Receive interrupt mask. A read returns the current mask for the UARTRXINTR inter
    RXIM OFFSET(4) NUMBITS(1) [],
    /// nUARTDSR modem interrupt mask. A read returns the current mask for the UARTDSRIN
    DSRMIM OFFSET(3) NUMBITS(1) [],
    /// nUARTDCD modem interrupt mask. A read returns the current mask for the UARTDCDIN
    DCDMIM OFFSET(2) NUMBITS(1) [],
    /// nUARTCTS modem interrupt mask. A read returns the current mask for the UARTCTSIN
    CTSMIM OFFSET(1) NUMBITS(1) [],
    /// nUARTRI modem interrupt mask. A read returns the current mask for the UARTRIINTR
    RIMIM OFFSET(0) NUMBITS(1) []
],
UARTRIS [
    /// Overrun error interrupt status. Returns the raw interrupt state of the UARTOEINT
    OERIS OFFSET(10) NUMBITS(1) [],
    /// Break error interrupt status. Returns the raw interrupt state of the UARTBEINTR
    BERIS OFFSET(9) NUMBITS(1) [],
    /// Parity error interrupt status. Returns the raw interrupt state of the UARTPEINTR
    PERIS OFFSET(8) NUMBITS(1) [],
    /// Framing error interrupt status. Returns the raw interrupt state of the UARTFEINT
    FERIS OFFSET(7) NUMBITS(1) [],
    /// Receive timeout interrupt status. Returns the raw interrupt state of the UARTRTI
    RTRIS OFFSET(6) NUMBITS(1) [],
    /// Transmit interrupt status. Returns the raw interrupt state of the UARTTXINTR int
    TXRIS OFFSET(5) NUMBITS(1) [],
    /// Receive interrupt status. Returns the raw interrupt state of the UARTRXINTR inte
    RXRIS OFFSET(4) NUMBITS(1) [],
    /// nUARTDSR modem interrupt status. Returns the raw interrupt state of the UARTDSRI
    DSRRMIS OFFSET(3) NUMBITS(1) [],
    /// nUARTDCD modem interrupt status. Returns the raw interrupt state of the UARTDCDI
    DCDRMIS OFFSET(2) NUMBITS(1) [],
    /// nUARTCTS modem interrupt status. Returns the raw interrupt state of the UARTCTSI
    CTSRMIS OFFSET(1) NUMBITS(1) [],
    /// nUARTRI modem interrupt status. Returns the raw interrupt state of the UARTRIINT
    RIRMIS OFFSET(0) NUMBITS(1) []
],
UARTMIS [
    /// Overrun error masked interrupt status. Returns the masked interrupt state of the
    OEMIS OFFSET(10) NUMBITS(1) [],
    /// Break error masked interrupt status. Returns the masked interrupt state of the U
    BEMIS OFFSET(9) NUMBITS(1) [],
    /// Parity error masked interrupt status. Returns the masked interrupt state of the
    PEMIS OFFSET(8) NUMBITS(1) [],
    /// Framing error masked interrupt status. Returns the masked interrupt state of the
    FEMIS OFFSET(7) NUMBITS(1) [],
    /// Receive timeout masked interrupt status. Returns the masked interrupt state of t
    RTMIS OFFSET(6) NUMBITS(1) [],
    /// Transmit masked interrupt status. Returns the masked interrupt state of the UART
    TXMIS OFFSET(5) NUMBITS(1) [],
    /// Receive masked interrupt status. Returns the masked interrupt state of the UARTR
    RXMIS OFFSET(4) NUMBITS(1) [],
    /// nUARTDSR modem masked interrupt status. Returns the masked interrupt state of th
    DSRMMIS OFFSET(3) NUMBITS(1) [],
    /// nUARTDCD modem masked interrupt status. Returns the masked interrupt state of th
    DCDMMIS OFFSET(2) NUMBITS(1) [],
    /// nUARTCTS modem masked interrupt status. Returns the masked interrupt state of th
    CTSMMIS OFFSET(1) NUMBITS(1) [],
    /// nUARTRI modem masked interrupt status. Returns the masked interrupt state of the
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
    /// DMA on error. If this bit is set to 1, the DMA receive request outputs, UARTRXDM
    DMAONERR OFFSET(2) NUMBITS(1) [],
    /// Transmit DMA enable. If this bit is set to 1, DMA for the transmit FIFO is enabl
    TXDMAE OFFSET(1) NUMBITS(1) [],
    /// Receive DMA enable. If this bit is set to 1, DMA for the receive FIFO is enabled
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
    /// This field depends on the revision of the UART: r1p0 0x0 r1p1 0x1 r1p3 0x2 r1p4
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
                    panic!("No data to transmit");
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
