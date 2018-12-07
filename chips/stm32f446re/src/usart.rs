use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;
use kernel::ReturnCode;

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

pub struct Usart<'a> {
    registers: StaticRef<UsartRegisters>,
    clock: UsartClock,
    client: OptionalCell<&'a hil::uart::Client>,
}

pub static mut USART2: Usart = Usart::new(
    USART2_BASE,
    UsartClock(rcc::PeripheralClock::APB1(rcc::PCLK1::USART2)),
);

impl Usart<'a> {
    const fn new(base_addr: StaticRef<UsartRegisters>, clock: UsartClock) -> Usart<'a> {
        Usart {
            registers: base_addr,
            clock: clock,
            client: OptionalCell::empty(),
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

    // for use by panic in io.rs
    pub fn send_byte(&self, byte: u8) {
        // loop till TXE (Transmit data register empty) becomes 1
        while !self.registers.sr.is_set(SR::TXE) {}

        self.registers.dr.set(byte.into());
    }
}

impl hil::uart::UART for Usart<'a> {
    fn set_client(&self, client: &'static hil::uart::Client) {
        self.client.set(client);
    }

    fn configure(&self, params: hil::uart::UARTParameters) -> ReturnCode {
        if params.baud_rate != 115200
            || params.stop_bits != hil::uart::StopBits::One
            || params.parity != hil::uart::Parity::None
            || params.hw_flow_control != false
        {
            panic!(
                "Currently we only support uart setting of 115200bps 8N1, no hardware flow control"
            );
        }

        // Right now in order to get the debug console working, only configure
        // the transmit block

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

        // Enable USART
        self.registers.cr1.modify(CR1::UE::SET);

        ReturnCode::SUCCESS
    }

    // Blocking transmit for now
    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        for i in 0..tx_len {
            // loop till TXE (Transmit data register empty) becomes 1
            while !self.registers.sr.is_set(SR::TXE) {}

            self.registers.dr.set(tx_data[i].into());
        }
    }

    fn receive(&self, _rx_buffer: &'static mut [u8], _rx_len: usize) {
        unimplemented!();
    }

    fn abort_receive(&self) {
        unimplemented!();
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
