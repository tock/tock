use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;
use kernel::ReturnCode;

use crate::dma1;
use crate::rcc;

/// Universal synchronous asynchronous receiver transmitter
#[repr(C)]
struct UsartRegisters {
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

const USART2_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x40004400 as *const UsartRegisters) };

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
enum USARTStateRX {
    Idle,
    DMA_Receiving,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
enum USARTStateTX {
    Idle,
    DMA_Transmitting,
    Transfer_Completing, // DMA finished, but not all bytes sent
}

pub struct Usart<'a> {
    registers: StaticRef<UsartRegisters>,
    clock: UsartClock,

    tx_client: OptionalCell<&'a hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a hil::uart::ReceiveClient>,

    tx_dma: OptionalCell<&'a dma1::Stream<'a>>,
    rx_dma: OptionalCell<&'a dma1::Stream<'a>>,

    tx_len: Cell<usize>,
    rx_len: Cell<usize>,

    usart_tx_state: Cell<USARTStateTX>,
    usart_rx_state: Cell<USARTStateRX>,
}

// for use by `set_dma`
pub struct TxDMA<'a>(pub &'a dma1::Stream<'a>);
pub struct RxDMA<'a>(pub &'a dma1::Stream<'a>);

pub static mut USART2: Usart = Usart::new(
    USART2_BASE,
    UsartClock(rcc::PeripheralClock::APB1(rcc::PCLK1::USART2)),
);

impl Usart<'a> {
    const fn new(base_addr: StaticRef<UsartRegisters>, clock: UsartClock) -> Usart<'a> {
        Usart {
            registers: base_addr,
            clock: clock,

            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),

            tx_dma: OptionalCell::empty(),
            rx_dma: OptionalCell::empty(),

            tx_len: Cell::new(0),
            rx_len: Cell::new(0),

            usart_tx_state: Cell::new(USARTStateTX::Idle),
            usart_rx_state: Cell::new(USARTStateRX::Idle),
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

    pub fn set_dma(&self, tx_dma: TxDMA<'a>, rx_dma: RxDMA<'a>) {
        self.tx_dma.set(tx_dma.0);
        self.rx_dma.set(rx_dma.0);
    }

    // According to section 25.4.13, we need to make sure that USART TC flag is
    // set before disabling the DMA TX on the peripheral side.
    pub fn handle_interrupt(&self) {
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
                    client.transmitted_buffer(buf, len, ReturnCode::SUCCESS);
                });
            });
        }
    }

    // for use by dma1
    pub fn get_address_dr(&self) -> u32 {
        &self.registers.dr as *const ReadWrite<u32> as u32
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

    fn abort_tx(&self, rcode: ReturnCode) {
        self.disable_tx();
        self.usart_tx_state.set(USARTStateTX::Idle);

        // get buffer
        let (mut buffer, len) = self.tx_dma.map_or((None, 0), |tx_dma| {
            // `abort_transfer` also disables the stream
            tx_dma.abort_transfer()
        });

        // The number actually transmitted is the difference between
        // the requested number and the number remaining in DMA transfer.
        let count = self.tx_len.get() - len as usize;
        self.tx_len.set(0);

        // alert client
        self.tx_client.map(|client| {
            buffer.take().map(|buf| {
                client.transmitted_buffer(buf, count as usize, rcode);
            });
        });
    }

    fn abort_rx(&self, rcode: ReturnCode, error: hil::uart::Error) {
        self.disable_rx();
        self.usart_rx_state.set(USARTStateRX::Idle);

        // get buffer
        let (mut buffer, len) = self.rx_dma.map_or((None, 0), |rx_dma| {
            // `abort_transfer` also disables the stream
            rx_dma.abort_transfer()
        });

        // The number actually received is the difference between
        // the requested number and the number remaining in DMA transfer.
        let count = self.rx_len.get() - len as usize;
        self.rx_len.set(0);

        // alert client
        self.rx_client.map(|client| {
            buffer.take().map(|buf| {
                client.received_buffer(buf, count, rcode, error);
            });
        });
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
}

impl hil::uart::Transmit<'a> for Usart<'a> {
    fn set_transmit_client(&self, client: &'a hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        // In virtual_uart.rs, transmit is only called when inflight is None. So
        // if the state machine is working correctly, transmit should never
        // abort.

        if self.usart_tx_state.get() != USARTStateTX::Idle {
            // there is an ongoing transmission, quit it
            return (ReturnCode::EBUSY, Some(tx_data));
        }

        // setup and enable dma stream
        self.tx_dma.map(move |dma| {
            self.tx_len.set(tx_len);
            dma.do_transfer(tx_data, tx_len);
        });

        self.usart_tx_state.set(USARTStateTX::DMA_Transmitting);

        // enable dma tx on peripheral side
        self.enable_tx();
        (ReturnCode::SUCCESS, None)
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        if self.usart_tx_state.get() != USARTStateTX::Idle {
            self.abort_tx(ReturnCode::ECANCEL);
            ReturnCode::EBUSY
        } else {
            ReturnCode::SUCCESS
        }
    }
}

impl hil::uart::Configure for Usart<'a> {
    fn configure(&self, params: hil::uart::Parameters) -> ReturnCode {
        if params.baud_rate != 115200
            || params.stop_bits != hil::uart::StopBits::One
            || params.parity != hil::uart::Parity::None
            || params.hw_flow_control != false
            || params.width != hil::uart::Width::Eight
        {
            panic!(
                "Currently we only support uart setting of 115200bps 8N1, no hardware flow control"
            );
        }

        // Configure the word length - 0: 1 Start bit, 8 Data bits, n Stop bits
        self.registers.cr1.modify(CR1::M::CLEAR);

        // Set the stop bit length - 00: 1 Stop bits
        self.registers.cr2.modify(CR2::STOP.val(0b00 as u32));

        // Set no parity
        self.registers.cr1.modify(CR1::PCE::CLEAR);

        // Set the baud rate. By default OVER8 is 0 (oversampling by 16) and
        // PCLK1 is at 16Mhz. The desired baud rate is 115.2KBps. So according
        // to Table 149 of reference manual, the value for BRR is 8.6875
        // DIV_Fraction = 0.6875 * 16 = 11 = 0xB
        // DIV_Mantissa = 8 = 0x8
        self.registers.brr.modify(BRR::DIV_Fraction.val(0xB as u32));
        self.registers.brr.modify(BRR::DIV_Mantissa.val(0x8 as u32));

        // Enable transmit block
        self.registers.cr1.modify(CR1::TE::SET);

        // Enable receive block
        self.registers.cr1.modify(CR1::RE::SET);

        // Enable USART
        self.registers.cr1.modify(CR1::UE::SET);

        ReturnCode::SUCCESS
    }
}

impl hil::uart::Receive<'a> for Usart<'a> {
    fn set_receive_client(&self, client: &'a hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.usart_rx_state.get() != USARTStateRX::Idle {
            return (ReturnCode::EBUSY, Some(rx_buffer));
        }

        if rx_len > rx_buffer.len() {
            return (ReturnCode::ESIZE, Some(rx_buffer));
        }

        // setup and enable dma stream
        self.rx_dma.map(move |dma| {
            self.rx_len.set(rx_len);
            dma.do_transfer(rx_buffer, rx_len);
        });

        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);

        // enable dma rx on the peripheral side
        self.enable_rx();
        (ReturnCode::SUCCESS, None)
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        self.abort_rx(ReturnCode::ECANCEL, hil::uart::Error::Aborted);
        ReturnCode::EBUSY
    }
}

impl hil::uart::UartData<'a> for Usart<'a> {}
impl hil::uart::Uart<'a> for Usart<'a> {}

impl dma1::StreamClient for Usart<'a> {
    fn transfer_done(&self, pid: dma1::Dma1Peripheral) {
        match pid {
            dma1::Dma1Peripheral::USART2_TX => {
                self.usart_tx_state.set(USARTStateTX::Transfer_Completing);
                self.enable_transmit_complete_interrupt();
            }
            dma1::Dma1Peripheral::USART2_RX => {
                // In case of RX, we can call the client directly without having
                // to trigger an interrupt.
                if self.usart_rx_state.get() == USARTStateRX::DMA_Receiving {
                    self.disable_rx();
                    self.usart_rx_state.set(USARTStateRX::Idle);

                    // get buffer
                    let buffer = self.rx_dma.map_or(None, |rx_dma| rx_dma.return_buffer());

                    let length = self.rx_len.get();
                    self.rx_len.set(0);

                    // alert client
                    self.rx_client.map(|client| {
                        buffer.map(|buf| {
                            client.received_buffer(
                                buf,
                                length,
                                ReturnCode::SUCCESS,
                                hil::uart::Error::None,
                            );
                        });
                    });
                }
            }
        }
    }
}

struct UsartClock(rcc::PeripheralClock);

impl ClockInterface for UsartClock {
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
