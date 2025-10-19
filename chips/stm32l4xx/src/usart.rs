// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::clocks::{phclk, Stm32l4Clocks};

/// Universal synchronous asynchronous receiver transmitter
/// Register layout per RM0351: CR1, CR2, CR3, BRR, GTPR, RTOR, RQR, ISR, ICR, RDR, TDR, PRESC
#[repr(C)]
pub struct UsartRegisters {
    /// Control register 1
    cr1: ReadWrite<u32, CR1::Register>, // 0x00
    /// Control register 2
    cr2: ReadWrite<u32, CR2::Register>, // 0x04
    /// Control register 3
    cr3: ReadWrite<u32, CR3::Register>, // 0x08
    /// Baud rate register
    brr: ReadWrite<u32, BRR::Register>, // 0x0C
    /// Guard time and prescaler register (smartcard/IrDA)
    gtpr: ReadWrite<u32, GTPR::Register>, // 0x10
    /// Receiver timeout register
    rtor: ReadWrite<u32, RTOR::Register>, // 0x14
    /// Request register
    rqr: ReadWrite<u32, RQR::Register>, // 0x18
    /// Interrupt & status register
    isr: ReadOnly<u32, ISR::Register>, // 0x1C
    /// Interrupt flag clear register
    icr: ReadWrite<u32, ICR::Register>, // 0x20 (write 1 to clear)
    /// Receive data register
    rdr: ReadOnly<u32, RDR::Register>, // 0x24
    /// Transmit data register
    tdr: ReadWrite<u32, TDR::Register>, // 0x28
    /// Prescaler register
    presc: ReadWrite<u32, PRESC::Register>, // 0x2C
}

register_bitfields![u32,
    CR1 [
        /// FIFO mode enable
        FIFOEN OFFSET(29) NUMBITS(1) [],
        /// Word length bit 1
        M1 OFFSET(28) NUMBITS(1) [],
        /// End of Block interrupt enable
        EOBIE OFFSET(27) NUMBITS(1) [],
        /// Receiver timeout interrupt enable
        RTOIE OFFSET(26) NUMBITS(1) [],
        /// Driver Enable assertion time
        DEAT OFFSET(21) NUMBITS(5) [],
        /// Driver Enable de-assertion time
        DEDT OFFSET(16) NUMBITS(5) [],
        /// Oversampling mode
        OVER8 OFFSET(15) NUMBITS(1) [],
        /// Character match interrupt enable
        CMIE OFFSET(14) NUMBITS(1) [],
        /// Mute mode enable
        MME OFFSET(13) NUMBITS(1) [],
        /// Word length bit 0
        M0 OFFSET(12) NUMBITS(1) [],
        /// Wakeup method
        WAKE OFFSET(11) NUMBITS(1) [],
        /// Parity control enable
        PCE OFFSET(10) NUMBITS(1) [],
        /// Parity selection
        PS OFFSET(9) NUMBITS(1) [],
        /// PE interrupt enable
        PEIE OFFSET(8) NUMBITS(1) [],
        /// TXE interrupt enable
        TXEIE OFFSET(7) NUMBITS(1) [],
        /// Transmission complete interrupt enable
        TCIE OFFSET(6) NUMBITS(1) [],
        /// RXNE interrupt enable
        RXNEIE OFFSET(5) NUMBITS(1) [],
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
    ],
    CR2 [
        /// Address of the USART node (4 bits MSB)
        ADD4_7 OFFSET(28) NUMBITS(4) [],
        /// Address of the USART node (4 bits LSB)
        ADD0_3 OFFSET(24) NUMBITS(4) [],
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
        /// TX pin active level inversion
        TXINV OFFSET(17) NUMBITS(1) [],
        /// RX pin active level inversion
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
        /// LIN break detection interrupt enable
        LBDIE OFFSET(6) NUMBITS(1) [],
        /// lin break detection length
        LBDL OFFSET(5) NUMBITS(1) [],
        /// 7-bit Address Detection/4-bit Address Detection
        ADDM7 OFFSET(4) NUMBITS(1) []
    ],
    CR3 [
        /// Wakeup from Stop mode interrupt flag clear
        WUFIE OFFSET(22) NUMBITS(1) [],
        /// Wakeup from Stop mode interrupt flag selection
        WUS OFFSET(20) NUMBITS(2) [],
        /// Smartcard auto-retry count
        SCARCNT OFFSET(17) NUMBITS(3) [],
        /// Driver enable polarity selection
        DEP OFFSET(15) NUMBITS(1) [],
        /// Driver enable mode
        DEM OFFSET(14) NUMBITS(1) [],
        /// DMA Disable on Reception Error
        DDRE OFFSET(13) NUMBITS(1) [],
        /// Overrun Disable
        OVRDIS OFFSET(12) NUMBITS(1) [],
        /// One sample bit method enable
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
        /// IrDA low-power
        IRLP OFFSET(2) NUMBITS(1) [],
        /// IrDA mode enable
        IREN OFFSET(1) NUMBITS(1) [],
        /// Error interrupt enable
        EIE OFFSET(0) NUMBITS(1) []
    ],
    BRR [
        /// Baud rate register (write full value)
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
        /// Transmit data flush request
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
        /// Parity error
        PE OFFSET(0) NUMBITS(1) [],
        /// Framing error
        FE OFFSET(1) NUMBITS(1) [],
        /// Noise error
        NE OFFSET(2) NUMBITS(1) [],
        /// Overrun error
        ORE OFFSET(3) NUMBITS(1) [],
        /// IDLE line detected
        IDLE OFFSET(4) NUMBITS(1) [],
        /// Read data register not empty
        RXNE OFFSET(5) NUMBITS(1) [],
        /// Transmission complete
        TC OFFSET(6) NUMBITS(1) [],
        /// Transmit data register empty
        TXE OFFSET(7) NUMBITS(1) [],
        /// LIN break detection flag
        LBDF OFFSET(8) NUMBITS(1) [],
        /// CTS interrupt flag
        CTSIF OFFSET(9) NUMBITS(1) [],
        /// CTS flag
        CTS OFFSET(10) NUMBITS(1) [],
        /// Receiver timeout
        RTOF OFFSET(11) NUMBITS(1) [],
        /// End of block flag
        EOBF OFFSET(12) NUMBITS(1) [],
        /// Auto baud rate error
        ABRE OFFSET(14) NUMBITS(1) [],
        /// Auto baud rate flag
        ABRF OFFSET(15) NUMBITS(1) [],
        /// Busy flag
        BUSY OFFSET(16) NUMBITS(1) [],
        /// Character match flag
        CMF OFFSET(17) NUMBITS(1) [],
        /// Send break flag
        SBKF OFFSET(18) NUMBITS(1) [],
        /// Receiver wakeup from Mute mode
        RWU OFFSET(19) NUMBITS(1) [],
        /// Wakeup from Stop mode flag
        WUF OFFSET(20) NUMBITS(1) [],
        /// Transmit enable acknowledge flag
        TEACK OFFSET(21) NUMBITS(1) [],
        /// Receive enable acknowledge flag
        REACK OFFSET(22) NUMBITS(1) []
    ],
    ICR [
        /// Parity error clear flag
        PECF OFFSET(0) NUMBITS(1) [],
        /// Framing error clear flag
        FECF OFFSET(1) NUMBITS(1) [],
        /// Noise error clear flag
        NECF OFFSET(2) NUMBITS(1) [],
        /// Overrun error clear flag
        ORECF OFFSET(3) NUMBITS(1) [],
        /// IDLE line detected clear flag
        IDLECF OFFSET(4) NUMBITS(1) [],
        /// Transmission complete clear flag
        TCCF OFFSET(6) NUMBITS(1) [],
        /// LIN break detection clear flag
        LBDCF OFFSET(8) NUMBITS(1) [],
        /// CTS clear flag
        CTSCF OFFSET(9) NUMBITS(1) [],
        /// Receiver timeout clear flag
        RTOCF OFFSET(11) NUMBITS(1) [],
        /// End of block clear flag
        EOBCF OFFSET(12) NUMBITS(1) [],
        /// Character match clear flag
        CMCF OFFSET(17) NUMBITS(1) [],
        /// Wakeup from Stop mode clear flag
        WUCF OFFSET(20) NUMBITS(1) []
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
        /// Prescaler value
        PRESCALER OFFSET(0) NUMBITS(4) []
    ]
];

// STM32L476 register boundary addresses
pub const USART1_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40013800 as *const UsartRegisters) };
pub const USART2_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40004400 as *const UsartRegisters) };
pub const USART3_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40004800 as *const UsartRegisters) };

macro_rules! usart_new {
    ($base:expr, $clock_type:expr, $clocks:expr) => {
        Usart {
            registers: $base,
            clock: UsartClock(phclk::PeripheralClock::new($clock_type, $clocks)),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
            usart_tx_state: Cell::new(USARTStateTX::Idle),
            usart_rx_state: Cell::new(USARTStateRX::Idle),
            deferred_call: DeferredCall::new(),
        }
    };
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
enum USARTStateRX {
    Idle,
    Receiving,
    Aborted(Result<(), ErrorCode>, hil::uart::Error),
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
enum USARTStateTX {
    Idle,
    Transmitting,
    Aborted(Result<(), ErrorCode>),
}

pub struct Usart<'a> {
    registers: StaticRef<UsartRegisters>,
    clock: UsartClock<'a>,

    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>,

    usart_tx_state: Cell<USARTStateTX>,
    usart_rx_state: Cell<USARTStateRX>,

    deferred_call: DeferredCall,
}

impl<'a> Usart<'a> {
    pub fn new_usart1(clocks: &'a dyn Stm32l4Clocks) -> Self {
        usart_new!(
            USART1_BASE,
            phclk::PeripheralClockType::APB2(phclk::PCLK2::USART1),
            clocks
        )
    }

    pub fn new_usart2(clocks: &'a dyn Stm32l4Clocks) -> Self {
        usart_new!(
            USART2_BASE,
            phclk::PeripheralClockType::APB1(phclk::PCLK1::USART2),
            clocks
        )
    }

    pub fn new_usart3(clocks: &'a dyn Stm32l4Clocks) -> Self {
        usart_new!(
            USART3_BASE,
            phclk::PeripheralClockType::APB1(phclk::PCLK1::USART3),
            clocks
        )
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

    pub fn handle_interrupt(&self) {
        // TXE interrupt - transmit data register empty
        if self.registers.isr.is_set(ISR::TXE)
            && self.usart_tx_state.get() == USARTStateTX::Transmitting
        {
            let index = self.tx_index.get();
            let len = self.tx_len.get();

            if index < len {
                self.tx_buffer.map(|buf| {
                    self.registers.tdr.set(buf[index] as u32);
                });
                self.tx_index.set(index + 1);

                // If last byte, disable TXE and enable TC interrupt
                if index + 1 >= len {
                    self.registers.cr1.modify(CR1::TXEIE::CLEAR);
                    self.enable_transmit_complete_interrupt();
                }
            }
        }

        // TC interrupt - transmission complete
        if self.registers.isr.is_set(ISR::TC)
            && self.usart_tx_state.get() == USARTStateTX::Transmitting
        {
            self.clear_transmit_complete();
            self.disable_transmit_complete_interrupt();
            self.usart_tx_state.set(USARTStateTX::Idle);

            let len = self.tx_len.get();
            self.tx_len.set(0);
            self.tx_index.set(0);

            self.tx_client.map(|client| {
                self.tx_buffer.take().map(|buf| {
                    client.transmitted_buffer(buf, len, Ok(()));
                });
            });
        }

        // RXNE interrupt - receive data register not empty
        if self.registers.isr.is_set(ISR::RXNE)
            && self.usart_rx_state.get() == USARTStateRX::Receiving
        {
            let index = self.rx_index.get();
            let len = self.rx_len.get();

            if index < len {
                let data = self.registers.rdr.get() as u8;
                self.rx_buffer.map(|buf| {
                    buf[index] = data;
                });
                self.rx_index.set(index + 1);

                // If buffer full, complete reception
                if index + 1 >= len {
                    self.registers.cr1.modify(CR1::RXNEIE::CLEAR);
                    self.disable_error_interrupt();
                    self.usart_rx_state.set(USARTStateRX::Idle);

                    self.rx_len.set(0);
                    self.rx_index.set(0);

                    self.rx_client.map(|client| {
                        self.rx_buffer.take().map(|buf| {
                            client.received_buffer(buf, len, Ok(()), hil::uart::Error::None);
                        });
                    });
                }
            }
        }

        // Error handling - must come before RXNE to clear errors first
        if self.is_enabled_error_interrupt() {
            let isr = self.registers.isr.get();

            // Check for framing error
            if (isr & (1 << 1)) != 0 {
                self.registers.icr.write(ICR::FECF::SET);
            }

            // Check for noise error
            if (isr & (1 << 2)) != 0 {
                self.registers.icr.write(ICR::NECF::SET);
            }

            // Check for overrun error
            if (isr & (1 << 3)) != 0 {
                self.registers.icr.write(ICR::ORECF::SET);

                if self.usart_rx_state.get() == USARTStateRX::Receiving {
                    self.registers.cr1.modify(CR1::RXNEIE::CLEAR);
                    self.disable_error_interrupt();
                    self.usart_rx_state.set(USARTStateRX::Idle);

                    let count = self.rx_index.get();
                    self.rx_len.set(0);
                    self.rx_index.set(0);

                    self.rx_client.map(|client| {
                        self.rx_buffer.take().map(|buf| {
                            client.received_buffer(
                                buf,
                                count,
                                Err(ErrorCode::CANCEL),
                                hil::uart::Error::OverrunError,
                            );
                        })
                    });
                }
            }
        }
    }

    // for use by panic in io.rs
    pub fn send_byte(&self, byte: u8) {
        // Clear TC flag before starting
        self.registers.icr.write(ICR::TCCF::SET);

        // Wait for TXE with timeout to avoid infinite loop
        let mut timeout = 100000;
        while !self.registers.isr.is_set(ISR::TXE) {
            timeout -= 1;
            if timeout == 0 {
                return; // Timeout - skip this byte
            }
        }

        self.registers.tdr.set(byte.into());

        // Wait for TC (transmission complete) to ensure byte is fully sent
        timeout = 100000;
        while !self.registers.isr.is_set(ISR::TC) {
            timeout -= 1;
            if timeout == 0 {
                break;
            }
        }
    }

    // enable interrupts for framing, overrun and noise errors
    fn enable_error_interrupt(&self) {
        self.registers.cr3.modify(CR3::EIE::SET);
    }

    // disable interrupts for framing, overrun and noise errors
    fn disable_error_interrupt(&self) {
        self.registers.cr3.modify(CR3::EIE::CLEAR);
    }

    // check if interrupts for framing, overrun and noise errors are enbaled
    fn is_enabled_error_interrupt(&self) -> bool {
        self.registers.cr3.is_set(CR3::EIE)
    }

    fn enable_transmit_complete_interrupt(&self) {
        self.registers.cr1.modify(CR1::TCIE::SET);
    }

    fn disable_transmit_complete_interrupt(&self) {
        self.registers.cr1.modify(CR1::TCIE::CLEAR);
    }

    fn clear_transmit_complete(&self) {
        // On L4, clear by writing 1 to ICR.TCCF
        self.registers.icr.write(ICR::TCCF::SET);
    }

    fn set_baud_rate(&self, baud_rate: u32) -> Result<(), ErrorCode> {
        // USARTDIV calculation per RM0351 Section 40.5.4 (STM32L476)
        //
        // Baud rate calculation equation:
        //
        // OVER8 = 0 (16x oversampling):
        //   Tx/Rx baud = f_CK / USARTDIV
        //   BRR contains USARTDIV (16-bit value with 4-bit fraction [3:0], 12-bit mantissa [15:4])
        //
        // OVER8 = 1 (8x oversampling):
        //   Tx/Rx baud = 2 * f_CK / USARTDIV
        //   BRR[2:0] = USARTDIV[3:0] >> 1 (only 3 fraction bits)
        //   BRR[3] must be 0
        //   BRR[15:4] = USARTDIV[15:4]

        let pclk_freq = self.clock.0.get_frequency();

        let brr_val = if (pclk_freq / 16) >= baud_rate {
            // Use 16x oversampling (OVER8 = 0)
            self.registers.cr1.modify(CR1::OVER8::CLEAR);

            // Calculate USARTDIV with 4-bit fraction: (f_CK * 16) / baud_rate
            // Add baud_rate/2 for rounding
            (pclk_freq + (baud_rate / 2)) / baud_rate
        } else if (pclk_freq / 8) >= baud_rate {
            // Use 8x oversampling (OVER8 = 1)
            self.registers.cr1.modify(CR1::OVER8::SET);

            // Calculate USARTDIV with 4-bit fraction: (f_CK * 32) / baud_rate
            let div = ((pclk_freq * 32) + (baud_rate / 2)) / baud_rate;

            // BRR encoding for OVER8=1:
            // Mantissa [15:4] unchanged, fraction [3:0] >> 1 to [2:0], bit [3] = 0
            let mantissa = div & 0xFFF0;
            let fraction = (div & 0x000F) >> 1;
            mantissa | fraction
        } else {
            return Err(ErrorCode::INVAL);
        };

        self.registers.brr.modify(BRR::BRR.val(brr_val));
        Ok(())
    }

    // try to disable the USART and return BUSY if a transfer is taking place
    pub fn disable(&self) -> Result<(), ErrorCode> {
        if self.usart_tx_state.get() == USARTStateTX::Transmitting
            || self.usart_rx_state.get() == USARTStateRX::Receiving
        {
            Err(ErrorCode::BUSY)
        } else {
            self.registers.cr1.modify(CR1::UE::CLEAR);
            Ok(())
        }
    }
}

impl<'a> DeferredCallClient for Usart<'a> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        if let USARTStateTX::Aborted(rcode) = self.usart_tx_state.get() {
            self.tx_client.map(|client| {
                self.tx_buffer.take().map(|buf| {
                    client.transmitted_buffer(buf, self.tx_index.get(), rcode);
                });
            });
            self.usart_tx_state.set(USARTStateTX::Idle);
        }

        if let USARTStateRX::Aborted(rcode, error) = self.usart_rx_state.get() {
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|buf| {
                    client.received_buffer(buf, self.rx_index.get(), rcode, error);
                });
            });
            self.usart_rx_state.set(USARTStateRX::Idle);
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
        if self.usart_tx_state.get() != USARTStateTX::Idle {
            return Err((ErrorCode::BUSY, tx_data));
        }

        self.tx_buffer.replace(tx_data);
        self.tx_len.set(tx_len);
        self.tx_index.set(0);

        self.usart_tx_state.set(USARTStateTX::Transmitting);

        // Enable TXE interrupt
        self.registers.cr1.modify(CR1::TXEIE::SET);
        Ok(())
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if self.usart_tx_state.get() != USARTStateTX::Idle {
            self.registers.cr1.modify(CR1::TXEIE::CLEAR);
            self.usart_tx_state
                .set(USARTStateTX::Aborted(Err(ErrorCode::CANCEL)));
            self.deferred_call.set();
            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }
}

impl<'a> hil::uart::Configure for Usart<'a> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        if params.stop_bits != hil::uart::StopBits::One
            || params.parity != hil::uart::Parity::None
            || params.hw_flow_control
            || params.width != hil::uart::Width::Eight
        {
            panic!("Currently we only support uart setting of 8N1, no hardware flow control");
        }

        // Configure the word length - 0: 1 Start bit, 8 Data bits, n Stop bits
        self.registers.cr1.modify(CR1::M0::CLEAR);
        self.registers.cr1.modify(CR1::M1::CLEAR);

        // Set the stop bit length - 00: 1 Stop bits
        self.registers.cr2.modify(CR2::STOP.val(0b00_u32));

        // Set no parity
        self.registers.cr1.modify(CR1::PCE::CLEAR);

        self.set_baud_rate(params.baud_rate)?;

        // Enable transmit block
        self.registers.cr1.modify(CR1::TE::SET);

        // Enable receive block
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
        if self.usart_rx_state.get() != USARTStateRX::Idle {
            return Err((ErrorCode::BUSY, rx_buffer));
        }

        if rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_index.set(0);

        self.usart_rx_state.set(USARTStateRX::Receiving);

        self.enable_error_interrupt();

        // Enable RXNE interrupt
        self.registers.cr1.modify(CR1::RXNEIE::SET);
        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if self.usart_rx_state.get() != USARTStateRX::Idle {
            self.registers.cr1.modify(CR1::RXNEIE::CLEAR);
            self.disable_error_interrupt();
            self.usart_rx_state.set(USARTStateRX::Aborted(
                Err(ErrorCode::CANCEL),
                hil::uart::Error::Aborted,
            ));
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
