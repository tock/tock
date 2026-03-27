// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Minimal USART driver for STM32U5xx (interrupt-driven, no DMA).
//!
//! This is intentionally small and mirrors the STM32F303-style interrupt
//! driver: TX/RX use FIFO empty/full interrupts, there is no DMA path yet.

use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::clocks::{phclk, Stm32u5Clocks};
/// Registers for USART/USART-like peripherals on STM32U5.
#[repr(C)]
struct UsartRegisters {
    cr1: ReadWrite<u32, CR1::Register>,
    /// Control register 2
    cr2: ReadWrite<u32, CR2::Register>,
    /// Control register 3
    cr3: ReadWrite<u32, CR3::Register>,
    /// Baud rate register
    brr: ReadWrite<u32>,
    /// Guard time and prescaler
    ///          register
    gtpr: ReadWrite<u32, GTPR::Register>,
    /// Receiver timeout register
    rtor: ReadWrite<u32, RTOR::Register>,
    /// Request register
    rqr: WriteOnly<u32, RQR::Register>,
    /// Interrupt & status
    ///          register
    isr: ReadOnly<u32, ISR::Register>,
    /// Interrupt flag clear register
    icr: WriteOnly<u32, ICR::Register>,
    /// Receive data register
    rdr: ReadOnly<u32>,
    /// Transmit data register
    tdr: ReadWrite<u32>,
    /// PRESC
    presc: ReadWrite<u32, PRESC::Register>,
    /// AUTOCR
    autocr: ReadWrite<u32, AUTOCR::Register>,
}
register_bitfields![u32,
CR1 [
    /// Word length
    M1 OFFSET(28) NUMBITS(1) [],
    /// End of Block interruptenable
    EOBIE OFFSET(27) NUMBITS(1) [],
    /// Receiver timeout interrupt
    RTOIE OFFSET(26) NUMBITS(1) [],
    /// DEAT
    DEAT OFFSET(21) NUMBITS(5) [],
    /// DEDT
    DEDT OFFSET(16) NUMBITS(5) [],
    /// Oversampling mode
    OVER8 OFFSET(15) NUMBITS(1) [],
    /// Character match interrupt
///               enable
    CMIE OFFSET(14) NUMBITS(1) [],
    /// Mute mode enable
    MME OFFSET(13) NUMBITS(1) [],
    /// Word length
    M0 OFFSET(12) NUMBITS(1) [],
    /// Receiver wakeup method
    WAKE OFFSET(11) NUMBITS(1) [],
    /// Parity control enable
    PCE OFFSET(10) NUMBITS(1) [],
    /// Parity selection
    PS OFFSET(9) NUMBITS(1) [],
    /// PE interrupt enable
    PEIE OFFSET(8) NUMBITS(1) [],
    /// TXFIFO not full interrupt enable
    TXFNFIE OFFSET(7) NUMBITS(1) [],
    /// Transmission complete interrupt
///               enable
    TCIE OFFSET(6) NUMBITS(1) [],
    /// RXFIFO not empty interrupt enable
    RXFNEIE OFFSET(5) NUMBITS(1) [],
    /// IDLE interrupt enable
    IDLEIE OFFSET(4) NUMBITS(1) [],
    /// Transmitter enable
    TE OFFSET(3) NUMBITS(1) [],
    /// Receiver enable
    RE OFFSET(2) NUMBITS(1) [],
    /// USART enable in Stop mode
    UESM OFFSET(1) NUMBITS(1) [],
    /// USART enable
    UE OFFSET(0) NUMBITS(1) [],
    /// FIFOEN
    FIFOEN OFFSET(29) NUMBITS(1) [],
    /// TXFEIE
    TXFEIE OFFSET(30) NUMBITS(1) [],
    /// RXFFIE
    RXFFIE OFFSET(31) NUMBITS(1) []
],
CR1_disabled [
    /// Word length
    M1 OFFSET(28) NUMBITS(1) [],
    /// End of Block interrupt
///               enable
    EOBIE OFFSET(27) NUMBITS(1) [],
    /// Receiver timeout interrupt
///               enable
    RTOIE OFFSET(26) NUMBITS(1) [],
    /// DEAT
    DEAT OFFSET(21) NUMBITS(5) [],
    /// DEDT
    DEDT OFFSET(16) NUMBITS(5) [],
    /// Oversampling mode
    OVER8 OFFSET(15) NUMBITS(1) [],
    /// Character match interrupt
///               enable
    CMIE OFFSET(14) NUMBITS(1) [],
    /// Mute mode enable
    MME OFFSET(13) NUMBITS(1) [],
    /// Word length
    M0 OFFSET(12) NUMBITS(1) [],
    /// Receiver wakeup method
    WAKE OFFSET(11) NUMBITS(1) [],
    /// Parity control enable
    PCE OFFSET(10) NUMBITS(1) [],
    /// Parity selection
    PS OFFSET(9) NUMBITS(1) [],
    /// PE interrupt enable
    PEIE OFFSET(8) NUMBITS(1) [],
    /// TXFIFO not full interrupt enable
    TXFNFIE OFFSET(7) NUMBITS(1) [],
    /// Transmission complete interrupt
///               enable
    TCIE OFFSET(6) NUMBITS(1) [],
    /// RXFIFO not empty interrupt enable
    RXFNEIE OFFSET(5) NUMBITS(1) [],
    /// IDLE interrupt enable
    IDLEIE OFFSET(4) NUMBITS(1) [],
    /// Transmitter enable
    TE OFFSET(3) NUMBITS(1) [],
    /// Receiver enable
    RE OFFSET(2) NUMBITS(1) [],
    /// USART enable in Stop mode
    UESM OFFSET(1) NUMBITS(1) [],
    /// USART enable
    UE OFFSET(0) NUMBITS(1) [],
    /// FIFOEN
    FIFOEN OFFSET(29) NUMBITS(1) []
],
CR2 [
    /// Address of the USART node
    ADD OFFSET(24) NUMBITS(8) [],
    /// Receiver timeout enable
    RTOEN OFFSET(23) NUMBITS(1) [],
    /// Auto baud rate mode
    ABRMOD OFFSET(21) NUMBITS(2) [],
    /// Auto baud rate enable
    ABREN OFFSET(20) NUMBITS(1) [],
    /// Most significant bit first
    MSBFIRST OFFSET(19) NUMBITS(1) [],
    /// Binary data inversion
    DATAINV OFFSET(18) NUMBITS(1) [],
    /// TX pin active level
///               inversion
    TXINV OFFSET(17) NUMBITS(1) [],
    /// RX pin active level
///               inversion
    RXINV OFFSET(16) NUMBITS(1) [],
    /// Swap TX/RX pins
    SWAP OFFSET(15) NUMBITS(1) [],
    /// LIN mode enable
    LINEN OFFSET(14) NUMBITS(1) [],
    /// STOP bits
    STOP OFFSET(12) NUMBITS(2) [],
    /// Clock enable
    CLKEN OFFSET(11) NUMBITS(1) [],
    /// Clock polarity
    CPOL OFFSET(10) NUMBITS(1) [],
    /// Clock phase
    CPHA OFFSET(9) NUMBITS(1) [],
    /// Last bit clock pulse
    LBCL OFFSET(8) NUMBITS(1) [],
    /// LIN break detection interrupt
///               enable
    LBDIE OFFSET(6) NUMBITS(1) [],
    /// LIN break detection length
    LBDL OFFSET(5) NUMBITS(1) [],
    /// 7-bit Address Detection/4-bit Address
///               Detection
    ADDM7 OFFSET(4) NUMBITS(1) [],
    /// SLVEN
    SLVEN OFFSET(0) NUMBITS(1) [],
    /// DIS_NSS
    DIS_NSS OFFSET(3) NUMBITS(1) []
],
CR3 [
    /// Smartcard auto-retry count
    SCARCNT OFFSET(17) NUMBITS(3) [],
    /// Driver enable polarity
///               selection
    DEP OFFSET(15) NUMBITS(1) [],
    /// Driver enable mode
    DEM OFFSET(14) NUMBITS(1) [],
    /// DMA Disable on Reception
///               Error
    DDRE OFFSET(13) NUMBITS(1) [],
    /// Overrun Disable
    OVRDIS OFFSET(12) NUMBITS(1) [],
    /// One sample bit method
///               enable
    ONEBIT OFFSET(11) NUMBITS(1) [],
    /// CTS interrupt enable
    CTSIE OFFSET(10) NUMBITS(1) [],
    /// CTS enable
    CTSE OFFSET(9) NUMBITS(1) [],
    /// RTS enable
    RTSE OFFSET(8) NUMBITS(1) [],
    /// DMA enable transmitter
    DMAT OFFSET(7) NUMBITS(1) [],
    /// DMA enable receiver
    DMAR OFFSET(6) NUMBITS(1) [],
    /// Smartcard mode enable
    SCEN OFFSET(5) NUMBITS(1) [],
    /// Smartcard NACK enable
    NACK OFFSET(4) NUMBITS(1) [],
    /// Half-duplex selection
    HDSEL OFFSET(3) NUMBITS(1) [],
    /// Ir low-power
    IRLP OFFSET(2) NUMBITS(1) [],
    /// Ir mode enable
    IREN OFFSET(1) NUMBITS(1) [],
    /// Error interrupt enable
    EIE OFFSET(0) NUMBITS(1) [],
    /// TXFTIE
    TXFTIE OFFSET(23) NUMBITS(1) [],
    /// TCBGTIE
    TCBGTIE OFFSET(24) NUMBITS(1) [],
    /// RXFTCFG
    RXFTCFG OFFSET(25) NUMBITS(3) [],
    /// RXFTIE
    RXFTIE OFFSET(28) NUMBITS(1) [],
    /// TXFTCFG
    TXFTCFG OFFSET(29) NUMBITS(3) []
],
BRR [
    /// BRR
    BRR OFFSET(0) NUMBITS(16) []
],
GTPR [
    /// Guard time value
    GT OFFSET(8) NUMBITS(8) [],
    /// Prescaler value
    PSC OFFSET(0) NUMBITS(8) []
],
RTOR [
    /// Block Length
    BLEN OFFSET(24) NUMBITS(8) [],
    /// Receiver timeout value
    RTO OFFSET(0) NUMBITS(24) []
],
RQR [
    /// Transmit data flush
///               request
    TXFRQ OFFSET(4) NUMBITS(1) [],
    /// Receive data flush request
    RXFRQ OFFSET(3) NUMBITS(1) [],
    /// Mute mode request
    MMRQ OFFSET(2) NUMBITS(1) [],
    /// Send break request
    SBKRQ OFFSET(1) NUMBITS(1) [],
    /// Auto baud rate request
    ABRRQ OFFSET(0) NUMBITS(1) []
],
ISR [
    /// REACK
    REACK OFFSET(22) NUMBITS(1) [],
    /// TEACK
    TEACK OFFSET(21) NUMBITS(1) [],
    /// RWU
    RWU OFFSET(19) NUMBITS(1) [],
    /// SBKF
    SBKF OFFSET(18) NUMBITS(1) [],
    /// CMF
    CMF OFFSET(17) NUMBITS(1) [],
    /// BUSY
    BUSY OFFSET(16) NUMBITS(1) [],
    /// ABRF
    ABRF OFFSET(15) NUMBITS(1) [],
    /// ABRE
    ABRE OFFSET(14) NUMBITS(1) [],
    /// EOBF
    EOBF OFFSET(12) NUMBITS(1) [],
    /// RTOF
    RTOF OFFSET(11) NUMBITS(1) [],
    /// CTS
    CTS OFFSET(10) NUMBITS(1) [],
    /// CTSIF
    CTSIF OFFSET(9) NUMBITS(1) [],
    /// LBDF
    LBDF OFFSET(8) NUMBITS(1) [],
    /// TXFNF
    TXFNF OFFSET(7) NUMBITS(1) [],
    /// TC
    TC OFFSET(6) NUMBITS(1) [],
    /// RXFNE
    RXFNE OFFSET(5) NUMBITS(1) [],
    /// IDLE
    IDLE OFFSET(4) NUMBITS(1) [],
    /// ORE
    ORE OFFSET(3) NUMBITS(1) [],
    /// NE
    NE OFFSET(2) NUMBITS(1) [],
    /// FE
    FE OFFSET(1) NUMBITS(1) [],
    /// PE
    PE OFFSET(0) NUMBITS(1) [],
    /// TXFE
    TXFE OFFSET(23) NUMBITS(1) [],
    /// RXFF
    RXFF OFFSET(24) NUMBITS(1) [],
    /// TCBGT
    TCBGT OFFSET(25) NUMBITS(1) [],
    /// RXFT
    RXFT OFFSET(26) NUMBITS(1) [],
    /// TXFT
    TXFT OFFSET(27) NUMBITS(1) []
],
ISR_disabled [
    /// REACK
    REACK OFFSET(22) NUMBITS(1) [],
    /// TEACK
    TEACK OFFSET(21) NUMBITS(1) [],
    /// RWU
    RWU OFFSET(19) NUMBITS(1) [],
    /// SBKF
    SBKF OFFSET(18) NUMBITS(1) [],
    /// CMF
    CMF OFFSET(17) NUMBITS(1) [],
    /// BUSY
    BUSY OFFSET(16) NUMBITS(1) [],
    /// ABRF
    ABRF OFFSET(15) NUMBITS(1) [],
    /// ABRE
    ABRE OFFSET(14) NUMBITS(1) [],
    /// UDR
    UDR OFFSET(13) NUMBITS(1) [],
    /// EOBF
    EOBF OFFSET(12) NUMBITS(1) [],
    /// RTOF
    RTOF OFFSET(11) NUMBITS(1) [],
    /// CTS
    CTS OFFSET(10) NUMBITS(1) [],
    /// CTSIF
    CTSIF OFFSET(9) NUMBITS(1) [],
    /// LBDF
    LBDF OFFSET(8) NUMBITS(1) [],
    /// TXFNF
    TXFNF OFFSET(7) NUMBITS(1) [],
    /// TC
    TC OFFSET(6) NUMBITS(1) [],
    /// RXFNE
    RXFNE OFFSET(5) NUMBITS(1) [],
    /// IDLE
    IDLE OFFSET(4) NUMBITS(1) [],
    /// ORE
    ORE OFFSET(3) NUMBITS(1) [],
    /// NE
    NE OFFSET(2) NUMBITS(1) [],
    /// FE
    FE OFFSET(1) NUMBITS(1) [],
    /// PE
    PE OFFSET(0) NUMBITS(1) [],
    /// TCBGT
    TCBGT OFFSET(25) NUMBITS(1) []
],
ICR [
    /// Character match clear flag
    CMCF OFFSET(17) NUMBITS(1) [],
    /// End of block clear flag
    EOBCF OFFSET(12) NUMBITS(1) [],
    /// Receiver timeout clear
///               flag
    RTOCF OFFSET(11) NUMBITS(1) [],
    /// CTS clear flag
    CTSCF OFFSET(9) NUMBITS(1) [],
    /// LIN break detection clear
///               flag
    LBDCF OFFSET(8) NUMBITS(1) [],
    /// Transmission complete clear
///               flag
    TCCF OFFSET(6) NUMBITS(1) [],
    /// Idle line detected clear
///               flag
    IDLECF OFFSET(4) NUMBITS(1) [],
    /// Overrun error clear flag
    ORECF OFFSET(3) NUMBITS(1) [],
    /// Noise detected clear flag
    NECF OFFSET(2) NUMBITS(1) [],
    /// Framing error clear flag
    FECF OFFSET(1) NUMBITS(1) [],
    /// Parity error clear flag
    PECF OFFSET(0) NUMBITS(1) [],
    /// TXFECF
    TXFECF OFFSET(5) NUMBITS(1) [],
    /// TCBGTCF
    TCBGTCF OFFSET(7) NUMBITS(1) [],
    /// UDRCF
    UDRCF OFFSET(13) NUMBITS(1) []
],
RDR [
    /// Receive data value
    RDR OFFSET(0) NUMBITS(9) []
],
TDR [
    /// Transmit data value
    TDR OFFSET(0) NUMBITS(9) []
],
PRESC [
    /// PRESCALER
    PRESCALER OFFSET(0) NUMBITS(4) []
],
AUTOCR [
    /// TECLREN
    TECLREN OFFSET(31) NUMBITS(1) [],
    /// IDLEDIS
    IDLEDIS OFFSET(18) NUMBITS(1) [],
    /// TRIGSEL
    TRIGSEL OFFSET(19) NUMBITS(4) [],
    /// TRIGEN
    TRIGEN OFFSET(17) NUMBITS(1) [],
    /// TRIPOL
    TRIGPOL OFFSET(16) NUMBITS(1) [],
    /// TDN
    TDN OFFSET(0) NUMBITS(16) []
]
];

/// Peripheral base addresses (non-secure aliases).
const USART1_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40013800 as *const UsartRegisters) };

const SEC_USART1_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x50013800 as *const UsartRegisters) };

const USART3_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40004800 as *const UsartRegisters) };

const SEC_USART3_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x50004800 as *const UsartRegisters) };

const UART4_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40004C00 as *const UsartRegisters) };

const SEC_UART4_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x50004C00 as *const UsartRegisters) };

const UART5_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40005000 as *const UsartRegisters) };

const SEC_UART5_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x50005000 as *const UsartRegisters) };

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
enum USARTStateTX {
    Idle,
    Transmitting,
    AbortRequested,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
enum USARTStateRX {
    Idle,
    Receiving,
    AbortRequested,
}

pub struct Usart<'a> {
    registers: StaticRef<UsartRegisters>,
    clock: UsartClock<'a>,

    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_position: Cell<usize>,
    tx_status: Cell<USARTStateTX>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_position: Cell<usize>,
    rx_status: Cell<USARTStateRX>,

    deferred_call: DeferredCall,
}

impl<'a> Usart<'a> {
    pub fn new_usart1(clocks: &'a dyn Stm32u5Clocks) -> Self {
        Self::new(
            USART1_BASE,
            UsartClock(phclk::PeripheralClock::new(
                phclk::PeripheralClockType::APB2(phclk::PCLK2::USART1),
                clocks,
            )),
        )
    }

    fn new(base: StaticRef<UsartRegisters>, clock: UsartClock<'a>) -> Self {
        Self {
            registers: base,
            clock,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_position: Cell::new(0),
            tx_status: Cell::new(USARTStateTX::Idle),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_position: Cell::new(0),
            rx_status: Cell::new(USARTStateRX::Idle),
            deferred_call: DeferredCall::new(),
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    /// Interrupt handler — call from the NVIC ISR.
    pub fn handle_interrupt(&self) {
        // Handle TX FIFO not full: push bytes until done.
        if self.tx_status.get() == USARTStateTX::Transmitting
            && self.registers.isr.is_set(ISR::TXFNF)
        {
            let pos = self.tx_position.get();
            let len = self.tx_len.get();
            if pos < len {
                self.tx_buffer.map(|buf| {
                    let byte = buf[pos];
                    self.registers.tdr.set(byte.into());
                });
                self.tx_position.set(pos + 1);

                // When last byte queued, enable TC interrupt to signal completion.
                if pos + 1 == len {
                    self.disable_transmit_interrupt();
                    self.enable_transmit_complete_interrupt();
                }
            }
        }

        // Transmission complete: notify client.
        if self.tx_status.get() == USARTStateTX::Transmitting && self.registers.isr.is_set(ISR::TC)
        {
            self.clear_transmit_complete();
            self.disable_transmit_complete_interrupt();
            self.tx_status.set(USARTStateTX::Idle);

            self.tx_client.map(|client| {
                self.tx_buffer.take().map(|buf| {
                    let len = self.tx_len.get();
                    client.transmitted_buffer(buf, len, Ok(()));
                });
            });
        }

        // RX FIFO not empty: pull bytes into buffer.
        if self.rx_status.get() == USARTStateRX::Receiving && self.registers.isr.is_set(ISR::RXFNE)
        {
            if let Some(buf) = self.rx_buffer.take() {
                let mut buf = buf;
                let pos = self.rx_position.get();
                let len = self.rx_len.get();
                if pos < len {
                    buf[pos] = self.registers.rdr.get() as u8;
                    self.rx_position.set(pos + 1);
                }
                // Put buffer back for continued reception.
                self.rx_buffer.replace(buf);
                if self.rx_position.get() == len {
                    self.disable_receive_interrupt();
                    self.rx_status.set(USARTStateRX::Idle);
                    self.rx_client.map(|client| {
                        self.rx_buffer.take().map(|buf| {
                            client.received_buffer(buf, len, Ok(()), hil::uart::Error::None);
                        });
                    });
                }
            }
        }

        // Overrun handling: clear and report as cancel.
        if self.registers.isr.is_set(ISR::ORE) {
            self.registers.icr.write(ICR::ORECF::SET);
            if self.rx_status.get() == USARTStateRX::Receiving {
                self.disable_receive_interrupt();
                self.rx_status.set(USARTStateRX::Idle);
                self.rx_client.map(|client| {
                    self.rx_buffer.take().map(|buf| {
                        let got = self.rx_position.get();
                        client.received_buffer(
                            buf,
                            got,
                            Err(ErrorCode::CANCEL),
                            hil::uart::Error::OverrunError,
                        );
                    });
                });
            }
        }
    }

    /// For panic paths: blocking single byte send.
    pub fn send_byte(&self, byte: u8) {
        while !self.registers.isr.is_set(ISR::TXFNF) {}
        self.registers.tdr.set(byte.into());
    }

    fn enable_transmit_interrupt(&self) {
        self.registers.cr1.modify(CR1::TXFNFIE::SET);
    }

    fn disable_transmit_interrupt(&self) {
        self.registers.cr1.modify(CR1::TXFNFIE::CLEAR);
    }

    fn enable_transmit_complete_interrupt(&self) {
        self.registers.cr1.modify(CR1::TCIE::SET);
    }

    fn disable_transmit_complete_interrupt(&self) {
        self.registers.cr1.modify(CR1::TCIE::CLEAR);
    }

    fn clear_transmit_complete(&self) {
        self.registers.icr.write(ICR::TCCF::SET);
    }

    fn enable_receive_interrupt(&self) {
        self.registers.cr1.modify(CR1::RXFNEIE::SET);
    }

    fn disable_receive_interrupt(&self) {
        self.registers.cr1.modify(CR1::RXFNEIE::CLEAR);
    }

    fn prescaler_divisor(&self) -> u32 {
        match self.registers.presc.read(PRESC::PRESCALER) {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 6,
            4 => 8,
            5 => 10,
            6 => 12,
            7 => 16,
            8 => 32,
            9 => 64,
            10 => 128,
            11 => 256,
            _ => 1,
        }
    }

    fn set_baud_rate(&self, baud_rate: u32) -> Result<(), ErrorCode> {
        // NOTE: If oversampling is by 8, this will return an error
        // TODO: Implement oversampling by 8
        let kernel_clk = self.clock.0.get_frequency() / self.prescaler_divisor();

        if baud_rate == 0 {
            return Err(ErrorCode::INVAL);
        }

        if (kernel_clk / 16) >= baud_rate {
            // 16x oversampling (OVER8 = 0)
            let div = (kernel_clk + (baud_rate / 2)) / baud_rate;
            self.registers.cr1.modify(CR1::OVER8::CLEAR);
            self.registers.brr.set(div & 0xFFFF);
            Ok(())
        } else {
            Err(ErrorCode::INVAL)
        }
    }

    // Try to disable the USART and return BUSY if a transfer is taking place
    pub fn disable(&self) -> Result<(), ErrorCode> {
        if self.tx_status.get() != USARTStateTX::Idle || self.rx_status.get() != USARTStateRX::Idle
        {
            Err(ErrorCode::BUSY)
        } else {
            self.registers.cr1.modify(CR1::UE::CLEAR);
            Ok(())
        }
    }
}

impl DeferredCallClient for Usart<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        if self.tx_status.get() == USARTStateTX::AbortRequested {
            self.tx_status.set(USARTStateTX::Idle);
            self.tx_client.map(|client| {
                self.tx_buffer.take().map(|buf| {
                    client.transmitted_buffer(buf, self.tx_position.get(), Err(ErrorCode::CANCEL));
                });
            });
        }

        if self.rx_status.get() == USARTStateRX::AbortRequested {
            // Mark idle before notifying the client so a new receive can be issued
            // from within the callback without hitting a BUSY error.
            self.rx_status.set(USARTStateRX::Idle);
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
        }
    }
}

impl<'a> hil::uart::Transmit<'a> for Usart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.tx_status.get() != USARTStateTX::Idle {
            return Err((ErrorCode::BUSY, tx_data));
        }

        if tx_len == 0 || tx_len > tx_data.len() {
            return Err((ErrorCode::SIZE, tx_data));
        }

        self.tx_buffer.replace(tx_data);
        self.tx_len.set(tx_len);
        self.tx_position.set(0);
        self.tx_status.set(USARTStateTX::Transmitting);

        self.enable_transmit_interrupt();
        Ok(())
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if self.tx_status.get() != USARTStateTX::Idle {
            self.disable_transmit_interrupt();
            self.disable_transmit_complete_interrupt();
            self.tx_status.set(USARTStateTX::AbortRequested);
            self.deferred_call.set();
            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }
}

impl hil::uart::Configure for Usart<'_> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        if params.stop_bits != hil::uart::StopBits::One
            || params.parity != hil::uart::Parity::None
            || params.hw_flow_control
            || params.width != hil::uart::Width::Eight
        {
            return Err(ErrorCode::NOSUPPORT);
        }
        // Disable before configuring
        self.registers.cr1.modify(CR1::UE::CLEAR);

        // Configure the word length - 0: 1 Start bit, 8 Data bits, n Stop bits
        self.registers.cr1.modify(CR1::M0::CLEAR);
        self.registers.cr1.modify(CR1::M1::CLEAR);

        // Set the stop bit length - 00: 1 Stop bit
        self.registers.cr2.modify(CR2::STOP.val(0b00_u32));

        // Set no parity
        self.registers.cr1.modify(CR1::PCE::CLEAR);

        // Explicitly select prescaler divide-by-1 to match baud calculation.
        self.registers.presc.modify(PRESC::PRESCALER.val(0));

        self.set_baud_rate(params.baud_rate)?;

        // Clear error flags
        self.registers
            .icr
            .write(ICR::FECF::SET + ICR::NECF::SET + ICR::ORECF::SET + ICR::PECF::SET);

        // Flush RX data
        while self.registers.isr.is_set(ISR::RXFNE) {
            let _ = self.registers.rdr.get();
        }

        // Enable transmit and receive blocks
        self.registers.cr1.modify(CR1::TE::SET);
        self.registers.cr1.modify(CR1::RE::SET);

        // Enable USART
        self.registers.cr1.modify(CR1::UE::SET);

        Ok(())
    }
}

impl<'a> hil::uart::Receive<'a> for Usart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.rx_status.get() != USARTStateRX::Idle {
            return Err((ErrorCode::BUSY, rx_buffer));
        }

        if rx_len == 0 || rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_position.set(0);
        self.rx_status.set(USARTStateRX::Receiving);

        self.enable_receive_interrupt();
        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if self.rx_status.get() != USARTStateRX::Idle {
            self.disable_receive_interrupt();
            self.rx_status.set(USARTStateRX::AbortRequested);
            self.deferred_call.set();
            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }
}

struct UsartClock<'a>(phclk::PeripheralClock<'a>);

impl ClockInterface for UsartClock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}
