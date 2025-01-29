// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::platform::chip::ClockInterface;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::clocks::{phclk, Stm32f4Clocks};
use crate::dma;

/// Universal synchronous asynchronous receiver transmitter
#[repr(C)]
pub struct UsartRegisters {
    /// Status register
    sr: ReadWrite<u32, SR::Register>,
    /// Data register
    dr: ReadWrite<u32>,
    /// Baud rate register
    brr: ReadWrite<u32, BRR::Register>,
    /// Control register 1
    cr1: ReadWrite<u32, CR1::Register>,
    /// Control register 2
    cr2: ReadWrite<u32, CR2::Register>,
    /// Control register 3
    cr3: ReadWrite<u32, CR3::Register>,
    /// Guard time and prescaler register
    gtpr: ReadWrite<u32, GTPR::Register>,
}

register_bitfields![u32,
    SR [
        /// CTS flag
        CTS OFFSET(9) NUMBITS(1) [],
        /// LIN break detection flag
        LBD OFFSET(8) NUMBITS(1) [],
        /// Transmit data register empty
        TXE OFFSET(7) NUMBITS(1) [],
        /// Transmission complete
        TC OFFSET(6) NUMBITS(1) [],
        /// Read data register not empty
        RXNE OFFSET(5) NUMBITS(1) [],
        /// IDLE line detected
        IDLE OFFSET(4) NUMBITS(1) [],
        /// Overrun error
        ORE OFFSET(3) NUMBITS(1) [],
        /// Noise detected flag
        NF OFFSET(2) NUMBITS(1) [],
        /// Framing error
        FE OFFSET(1) NUMBITS(1) [],
        /// Parity error
        PE OFFSET(0) NUMBITS(1) []
    ],
    BRR [
        /// mantissa of USARTDIV
        DIV_Mantissa OFFSET(4) NUMBITS(12) [],
        /// fraction of USARTDIV
        DIV_Fraction OFFSET(0) NUMBITS(4) []
    ],
    CR1 [
        /// Oversampling mode
        OVER8 OFFSET(15) NUMBITS(1) [],
        /// USART enable
        UE OFFSET(13) NUMBITS(1) [],
        /// Word length
        M OFFSET(12) NUMBITS(1) [],
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
        /// Receiver wakeup
        RWU OFFSET(1) NUMBITS(1) [],
        /// Send break
        SBK OFFSET(0) NUMBITS(1) []
    ],
    CR2 [
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
        /// Address of the USART node
        ADD OFFSET(0) NUMBITS(4) []
    ],
    CR3 [
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
    GTPR [
        /// Guard time value
        GT OFFSET(8) NUMBITS(8) [],
        /// Prescaler value
        PSC OFFSET(0) NUMBITS(8) []
    ]
];

// See Table 13. STM32F427xx and STM32F429xx register boundary addresses
// of the STM32F429zi datasheet
pub const USART1_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40011000 as *const UsartRegisters) };
pub const USART2_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40004400 as *const UsartRegisters) };
pub const USART3_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40004800 as *const UsartRegisters) };

// for use by dma1
pub(crate) fn get_address_dr(regs: StaticRef<UsartRegisters>) -> u32 {
    core::ptr::addr_of!(regs.dr) as u32
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
enum USARTStateRX {
    Idle,
    DMA_Receiving,
    Aborted(Result<(), ErrorCode>, hil::uart::Error),
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
enum USARTStateTX {
    Idle,
    DMA_Transmitting,
    Aborted(Result<(), ErrorCode>),
    Transfer_Completing, // DMA finished, but not all bytes sent
}

pub struct Usart<'a, DMA: dma::StreamServer<'a>> {
    registers: StaticRef<UsartRegisters>,
    clock: UsartClock<'a>,

    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_dma: OptionalCell<&'a dma::Stream<'a, DMA>>,
    tx_dma_pid: DMA::Peripheral,
    rx_dma: OptionalCell<&'a dma::Stream<'a, DMA>>,
    rx_dma_pid: DMA::Peripheral,

    tx_len: Cell<usize>,
    rx_len: Cell<usize>,

    usart_tx_state: Cell<USARTStateTX>,
    usart_rx_state: Cell<USARTStateRX>,

    partial_tx_buffer: TakeCell<'static, [u8]>,
    partial_tx_len: Cell<usize>,

    partial_rx_buffer: TakeCell<'static, [u8]>,
    partial_rx_len: Cell<usize>,

    deferred_call: DeferredCall,
}

// for use by `set_dma`
pub struct TxDMA<'a, DMA: dma::StreamServer<'a>>(pub &'a dma::Stream<'a, DMA>);
pub struct RxDMA<'a, DMA: dma::StreamServer<'a>>(pub &'a dma::Stream<'a, DMA>);

impl<'a> Usart<'a, dma::Dma1<'a>> {
    pub fn new_usart2(clocks: &'a dyn Stm32f4Clocks) -> Self {
        Self::new(
            USART2_BASE,
            UsartClock(phclk::PeripheralClock::new(
                phclk::PeripheralClockType::APB1(phclk::PCLK1::USART2),
                clocks,
            )),
            dma::Dma1Peripheral::USART2_TX,
            dma::Dma1Peripheral::USART2_RX,
        )
    }

    pub fn new_usart3(clocks: &'a dyn Stm32f4Clocks) -> Self {
        Self::new(
            USART3_BASE,
            UsartClock(phclk::PeripheralClock::new(
                phclk::PeripheralClockType::APB1(phclk::PCLK1::USART3),
                clocks,
            )),
            dma::Dma1Peripheral::USART3_TX,
            dma::Dma1Peripheral::USART3_RX,
        )
    }
}

impl<'a> Usart<'a, dma::Dma2<'a>> {
    pub fn new_usart1(clocks: &'a dyn Stm32f4Clocks) -> Self {
        Self::new(
            USART1_BASE,
            UsartClock(phclk::PeripheralClock::new(
                phclk::PeripheralClockType::APB2(phclk::PCLK2::USART1),
                clocks,
            )),
            dma::Dma2Peripheral::USART1_TX,
            dma::Dma2Peripheral::USART1_RX,
        )
    }
}

impl<'a, DMA: dma::StreamServer<'a>> Usart<'a, DMA> {
    fn new(
        base_addr: StaticRef<UsartRegisters>,
        clock: UsartClock<'a>,
        tx_dma_pid: DMA::Peripheral,
        rx_dma_pid: DMA::Peripheral,
    ) -> Usart<'a, DMA> {
        Usart {
            registers: base_addr,
            clock,

            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),

            tx_dma: OptionalCell::empty(),
            tx_dma_pid,
            rx_dma: OptionalCell::empty(),
            rx_dma_pid,

            tx_len: Cell::new(0),
            rx_len: Cell::new(0),

            usart_tx_state: Cell::new(USARTStateTX::Idle),
            usart_rx_state: Cell::new(USARTStateRX::Idle),

            partial_tx_buffer: TakeCell::empty(),
            partial_tx_len: Cell::new(0),

            partial_rx_buffer: TakeCell::empty(),
            partial_rx_len: Cell::new(0),

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

    pub fn set_dma(&self, tx_dma: TxDMA<'a, DMA>, rx_dma: RxDMA<'a, DMA>) {
        self.tx_dma.set(tx_dma.0);
        self.rx_dma.set(rx_dma.0);
    }

    // According to section 25.4.13, we need to make sure that USART TC flag is
    // set before disabling the DMA TX on the peripheral side.
    pub fn handle_interrupt(&self) {
        if self.registers.sr.is_set(SR::TC) {
            self.clear_transmit_complete();
            self.disable_transmit_complete_interrupt();

            // Ignore if USARTStateTX is in some other state other than
            // Transfer_Completing.
            if self.usart_tx_state.get() == USARTStateTX::Transfer_Completing {
                self.disable_tx();
                self.usart_tx_state.set(USARTStateTX::Idle);

                // get buffer
                let buffer = self.tx_dma.map_or(None, |tx_dma| tx_dma.return_buffer());
                let len = self.tx_len.get();
                self.tx_len.set(0);

                // alert client
                self.tx_client.map(|client| {
                    buffer.map(|buf| {
                        let buf = buf.take();
                        client.transmitted_buffer(buf, len, Ok(()));
                    });
                });
            }
        }

        if self.is_enabled_error_interrupt() && self.registers.sr.is_set(SR::ORE) {
            let _ = self.registers.dr.get(); // clear overrun error
            if self.usart_rx_state.get() == USARTStateRX::DMA_Receiving {
                self.usart_rx_state.set(USARTStateRX::Idle);

                self.disable_rx();
                self.disable_error_interrupt();

                // get buffer
                let (buffer, len) = self.rx_dma.map_or((None, 0), |rx_dma| {
                    // `abort_transfer` also disables the stream
                    rx_dma.abort_transfer()
                });

                // The number actually received is the difference between
                // the requested number and the number remaining in DMA transfer.
                let count = self.rx_len.get() - len as usize;
                self.rx_len.set(0);

                // alert client
                self.rx_client.map(|client| {
                    buffer.map(|buf| {
                        let buf = buf.take();
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

    // for use by panic in io.rs
    pub fn send_byte(&self, byte: u8) {
        // loop till TXE (Transmit data register empty) becomes 1
        while !self.registers.sr.is_set(SR::TXE) {}

        self.registers.dr.set(byte.into());
    }

    // enable DMA TX from the peripheral side
    fn enable_tx(&self) {
        self.registers.cr3.modify(CR3::DMAT::SET);
    }

    // disable DMA TX from the peripheral side
    fn disable_tx(&self) {
        self.registers.cr3.modify(CR3::DMAT::CLEAR);
    }

    // enable DMA RX from the peripheral side
    fn enable_rx(&self) {
        self.registers.cr3.modify(CR3::DMAR::SET);
    }

    // disable DMA RX from the peripheral side
    fn disable_rx(&self) {
        self.registers.cr3.modify(CR3::DMAR::CLEAR);
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

    fn abort_tx(&self, rcode: Result<(), ErrorCode>) {
        if matches!(self.usart_tx_state.get(), USARTStateTX::Aborted(_)) {
            return;
        }

        self.disable_tx();

        // get buffer
        let (mut buffer, len) = self.tx_dma.map_or((None, 0), |tx_dma| {
            // `abort_transfer` also disables the stream
            tx_dma.abort_transfer()
        });

        // The number actually transmitted is the difference between
        // the requested number and the number remaining in DMA transfer.
        let count = self.tx_len.get() - len as usize;
        self.tx_len.set(0);

        if let Some(buf) = buffer.take() {
            let buf = buf.take();
            self.partial_tx_buffer.replace(buf);
            self.partial_tx_len.set(count);

            self.usart_tx_state.set(USARTStateTX::Aborted(rcode));

            self.deferred_call.set();
        } else {
            self.usart_tx_state.set(USARTStateTX::Idle);
        }
    }

    fn abort_rx(&self, rcode: Result<(), ErrorCode>, error: hil::uart::Error) {
        if matches!(self.usart_rx_state.get(), USARTStateRX::Aborted(_, _)) {
            return;
        }

        self.disable_rx();
        self.disable_error_interrupt();

        // get buffer
        let (mut buffer, len) = self.rx_dma.map_or((None, 0), |rx_dma| {
            // `abort_transfer` also disables the stream
            rx_dma.abort_transfer()
        });

        // The number actually received is the difference between
        // the requested number and the number remaining in DMA transfer.
        let count = self.rx_len.get() - len as usize;
        self.rx_len.set(0);

        if let Some(buf) = buffer.take() {
            let buf = buf.take();
            self.partial_rx_buffer.replace(buf);
            self.partial_rx_len.set(count);

            self.usart_rx_state.set(USARTStateRX::Aborted(rcode, error));

            self.deferred_call.set();
        } else {
            self.usart_rx_state.set(USARTStateRX::Idle);
        }
    }

    fn enable_transmit_complete_interrupt(&self) {
        self.registers.cr1.modify(CR1::TCIE::SET);
    }

    fn disable_transmit_complete_interrupt(&self) {
        self.registers.cr1.modify(CR1::TCIE::CLEAR);
    }

    fn clear_transmit_complete(&self) {
        self.registers.sr.modify(SR::TC::CLEAR);
    }

    fn transfer_done(&self, pid: DMA::Peripheral) {
        if pid == self.tx_dma_pid {
            self.usart_tx_state.set(USARTStateTX::Transfer_Completing);
            self.enable_transmit_complete_interrupt();
        } else if pid == self.rx_dma_pid {
            // In case of RX, we can call the client directly without having
            // to trigger an interrupt.
            if self.usart_rx_state.get() == USARTStateRX::DMA_Receiving {
                self.disable_rx();
                self.disable_error_interrupt();
                self.usart_rx_state.set(USARTStateRX::Idle);

                // get buffer
                let buffer = self.rx_dma.map_or(None, |rx_dma| rx_dma.return_buffer());

                let length = self.rx_len.get();
                self.rx_len.set(0);

                // alert client
                self.rx_client.map(|client| {
                    buffer.map(|buf| {
                        let buf = buf.take();
                        client.received_buffer(buf, length, Ok(()), hil::uart::Error::None);
                    });
                });
            }
        }
    }

    fn set_baud_rate(&self, baud_rate: u32) -> Result<(), ErrorCode> {
        // USARTDIV calculation based on stm32-rs stm32f4xx-hal:
        // https://github.com/stm32-rs/stm32f4xx-hal/blob/v0.20.0/src/serial/uart_impls.rs#L145
        //
        // The equation to calculate USARTDIV is this:
        //
        // (Taken from STM32F411xC/E Reference Manual, Section 19.3.4, Equation 1)
        //
        // 16 bit oversample: OVER8 = 0
        // 8 bit oversample:  OVER8 = 1
        //
        // USARTDIV =          (pclk)
        //            ------------------------
        //            8 x (2 - OVER8) x (baud)
        //
        // BUT, the USARTDIV has 4 "fractional" bits, which effectively means that we need to
        // "correct" the equation as follows:
        //
        // USARTDIV =      (pclk) * 16
        //            ------------------------
        //            8 x (2 - OVER8) x (baud)
        //
        // When OVER8 is enabled, we can only use the lowest three fractional bits, so we'll need
        // to shift those last four bits right one bit

        let pclk_freq = self.clock.0.get_frequency();

        let (mantissa, fraction) = if (pclk_freq / 16) >= baud_rate {
            // We have the ability to oversample to 16 bits, take advantage of it.
            //
            // We also add `baud / 2` to the `pclk_freq` to ensure rounding of values to the
            // closest scale, rather than the floored behavior of normal integer division.
            let div = (pclk_freq + (baud_rate / 2)) / baud_rate;

            self.registers.cr1.modify(CR1::OVER8::CLEAR);

            (div >> 4, div & 0x0F)
        } else if (pclk_freq / 8) >= baud_rate {
            // We are close enough to pclk where we can only
            // oversample 8.

            // See note above regarding `baud` and rounding.
            let div = ((pclk_freq * 2) + (baud_rate / 2)) / baud_rate;

            self.registers.cr1.modify(CR1::OVER8::SET);

            // Ensure the the fractional bits (only 3) are right-aligned.
            (div >> 4, (div & 0x0F) >> 1)
        } else {
            return Err(ErrorCode::INVAL);
        };

        self.registers.brr.modify(BRR::DIV_Mantissa.val(mantissa));
        self.registers.brr.modify(BRR::DIV_Fraction.val(fraction));
        Ok(())
    }

    // try to disable the USART and return BUSY if a transfer is taking place
    pub fn disable(&self) -> Result<(), ErrorCode> {
        if self.usart_tx_state.get() == USARTStateTX::DMA_Transmitting
            || self.usart_tx_state.get() == USARTStateTX::Transfer_Completing
            || self.usart_rx_state.get() == USARTStateRX::DMA_Receiving
        {
            Err(ErrorCode::BUSY)
        } else {
            self.registers.cr1.modify(CR1::UE::CLEAR);
            Ok(())
        }
    }
}

impl<'a, DMA: dma::StreamServer<'a>> DeferredCallClient for Usart<'a, DMA> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        if let USARTStateTX::Aborted(rcode) = self.usart_tx_state.get() {
            // alert client
            self.tx_client.map(|client| {
                self.partial_tx_buffer.take().map(|buf| {
                    client.transmitted_buffer(buf, self.partial_tx_len.get(), rcode);
                });
            });
            self.usart_tx_state.set(USARTStateTX::Idle);
        }

        if let USARTStateRX::Aborted(rcode, error) = self.usart_rx_state.get() {
            // alert client
            self.rx_client.map(|client| {
                self.partial_rx_buffer.take().map(|buf| {
                    client.received_buffer(buf, self.partial_rx_len.get(), rcode, error);
                });
            });
            self.usart_rx_state.set(USARTStateRX::Idle);
        }
    }
}

impl<'a, DMA: dma::StreamServer<'a>> hil::uart::Transmit<'a> for Usart<'a, DMA> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // In virtual_uart.rs, transmit is only called when inflight is None. So
        // if the state machine is working correctly, transmit should never
        // abort.

        if self.usart_tx_state.get() != USARTStateTX::Idle {
            // there is an ongoing transmission, quit it
            return Err((ErrorCode::BUSY, tx_data));
        }

        // setup and enable dma stream
        self.tx_dma.map(move |dma| {
            self.tx_len.set(tx_len);
            let mut tx_data: SubSliceMut<u8> = tx_data.into();
            tx_data.slice(..tx_len);
            dma.do_transfer(tx_data);
        });

        self.usart_tx_state.set(USARTStateTX::DMA_Transmitting);

        // enable dma tx on peripheral side
        self.enable_tx();
        Ok(())
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if self.usart_tx_state.get() != USARTStateTX::Idle {
            self.abort_tx(Err(ErrorCode::CANCEL));
            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }
}

impl<'a, DMA: dma::StreamServer<'a>> hil::uart::Configure for Usart<'a, DMA> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        if params.stop_bits != hil::uart::StopBits::One
            || params.parity != hil::uart::Parity::None
            || params.hw_flow_control
            || params.width != hil::uart::Width::Eight
        {
            panic!("Currently we only support uart setting of 8N1, no hardware flow control");
        }

        // Configure the word length - 0: 1 Start bit, 8 Data bits, n Stop bits
        self.registers.cr1.modify(CR1::M::CLEAR);

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

impl<'a, DMA: dma::StreamServer<'a>> hil::uart::Receive<'a> for Usart<'a, DMA> {
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

        // setup and enable dma stream
        self.rx_dma.map(move |dma| {
            self.rx_len.set(rx_len);
            let mut rx_buffer: SubSliceMut<u8> = rx_buffer.into();
            rx_buffer.slice(..rx_len);
            dma.do_transfer(rx_buffer);
        });

        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);

        self.enable_error_interrupt();

        // enable dma rx on the peripheral side
        self.enable_rx();
        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if self.usart_rx_state.get() != USARTStateRX::Idle {
            self.abort_rx(Err(ErrorCode::CANCEL), hil::uart::Error::Aborted);
            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }
}

impl<'a> dma::StreamClient<'a, dma::Dma1<'a>> for Usart<'a, dma::Dma1<'a>> {
    fn transfer_done(&self, pid: dma::Dma1Peripheral) {
        self.transfer_done(pid);
    }
}

impl<'a> dma::StreamClient<'a, dma::Dma2<'a>> for Usart<'a, dma::Dma2<'a>> {
    fn transfer_done(&self, pid: dma::Dma2Peripheral) {
        self.transfer_done(pid);
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
