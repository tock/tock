use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;
use kernel::ReturnCode;

use crate::ccm;

/// LP Universal asynchronous receiver transmitter
#[repr(C)]
struct LpuartRegisters {
    ///  Version ID Register
    verid: ReadOnly<u32, VERID::Register>,
    /// Parameter Register
    param: ReadOnly<u32, PARAM::Register>,
    /// LPUART Global Register
    global: ReadWrite<u32, GLOBAL::Register>,
    /// LPUART Pin Configuration Register
    pincfg: ReadWrite<u32, PINCFG::Register>,
    /// LPUART Baud Rate Register
    baud: ReadWrite<u32, BAUD::Register>,
    /// LPUART Status Register
    stat: ReadWrite<u32, STAT::Register>,
    /// LPUART Control Register
    ctrl: ReadWrite<u32, CTRL::Register>,
    /// LPUART Data Register
    data: ReadWrite<u32, DATA::Register>,
    /// LPUART Match Address Register
    r#match: ReadWrite<u32, MATCH::Register>,
    /// LPUART Modem IrDA Register
    modir: ReadWrite<u32, MODIR::Register>,
    /// LPUART FIFO Register
    fifo: ReadWrite<u32, FIFO::Register>,
    /// LPUART Watemark Register
    water: ReadWrite<u32, WATER::Register>,
}

register_bitfields![u32,
    VERID [
        /// Major Version Number
        MAJOR OFFSET(24) NUMBITS(8) [],
        /// Minor Version Number
        MINOR OFFSET(16) NUMBITS(8) [],
        /// Feature Identification Number
        FEATURE OFFSET(0) NUMBITS(16) []
    ],

    PARAM [
        /// Receive FIFO Size
        RXFIFO OFFSET(8) NUMBITS(8) [],
        /// Transmit FIFO Size
        TXFIFO OFFSET(0) NUMBITS(8) []
    ],

    GLOBAL [
        /// Software reset
        RST OFFSET(1) NUMBITS(1) []
    ],

    PINCFG [
        /// Trigger Select for input trigger usage
        TRGSEL OFFSET(0) NUMBITS(2) []
    ],

    BAUD [
        /// Match Address Mode Enable 1
        MAEN1 OFFSET(31) NUMBITS(1) [],
        /// Match Address Mode Enable 2
        MAEN2 OFFSET(30) NUMBITS(1) [],
        /// 10-bit Mode select
        M10 OFFSET(29) NUMBITS(1) [],
        /// Oversampling Ratio
        OSR OFFSET(24) NUMBITS(5) [],
        /// Transmitter DMA Enable
        TDMAE OFFSET(23) NUMBITS(1) [],
        /// Receiver Full DMA Enable
        RDMAE OFFSET(21) NUMBITS(1) [],
        /// Receiver Idle DMA Enable
        RIDMAE OFFSET(20) NUMBITS(1) [],
        /// Match Configuration
        MATCFG OFFSET(18) NUMBITS(2) [],
        /// Both Edge Sampling
        BOTHEDGE OFFSET(17) NUMBITS(1) [],
        /// Resynchronization Disable
        RESYNCDIS OFFSET(16) NUMBITS(1) [],
        /// LIN Break Detect Interrupt Enable
        LBKDIE OFFSET(15) NUMBITS(1) [],
        /// RX Input Active Edge Interrupt Enable
        RXEDGIE OFFSET(14) NUMBITS(1) [],
        /// Stop Bit Number Select
        SBNS OFFSET(13) NUMBITS(1) [],
        /// Baud Rate Modulo Divisor
        SBR OFFSET(0) NUMBITS(13) []
    ],

    STAT [
        /// LIN Break Detect Interrupt Flag
        LBKDIF OFFSET(31) NUMBITS(1) [],
        /// RXD Pin Active Edge Interrupt Flag
        RXEDGIF OFFSET(30) NUMBITS(1) [],
        /// MSB First
        MSBF OFFSET(29) NUMBITS(1) [],
        /// Receive Data Inversion
        RXINV OFFSET(28) NUMBITS(1) [],
        /// Receive Wake Up Idle Detect
        RWUID OFFSET(27) NUMBITS(1) [],
        /// Break Character Generation Length
        BRK13 OFFSET(26) NUMBITS(1) [],
        /// LIN Break Detection Enable
        LBKDE OFFSET(25) NUMBITS(1) [],
        /// Receiver Active Flag
        RAF OFFSET(24) NUMBITS(1) [],
        /// Transmit Data Register Empty Flag
        TDRE OFFSET(23) NUMBITS(1) [],
        /// Transmission Complete Flag
        TC OFFSET(22) NUMBITS(1) [],
        /// Receive Data Register Full Flag
        RDRF OFFSET(21) NUMBITS(1) [],
        /// Idle Line Flag
        IDLE OFFSET(20) NUMBITS(1) [],
        /// Receiver Overrun Flag
        OR OFFSET(19) NUMBITS(1) [],
        /// Noise Flag
        NF OFFSET(18) NUMBITS(1) [],
        /// Framing Error Flag
        FE OFFSET(17) NUMBITS(1) [],
        /// Parity Error Flag
        PF OFFSET(16) NUMBITS(1) [],
        /// Match 1 Flag
        MA1F OFFSET(15) NUMBITS(1) [],
        /// Match 2 Flag
        MA2F OFFSET(14) NUMBITS(1) []
    ],

    CTRL [
        /// Receive Bit 8 / Transmit Bit 9
        R8T9 OFFSET(31) NUMBITS(1) [],
        /// Receive Bit 9 / Transmit Bit 8
        R9T8 OFFSET(30) NUMBITS(1) [],
        /// TXD Pin Direction in Single-Wire Mode
        TXDIR OFFSET(29) NUMBITS(1) [],
        /// Transmit Data Inversion
        TXINV OFFSET(28) NUMBITS(1) [],
        /// Overrun Interrupt Enable
        ORIE OFFSET(27) NUMBITS(1) [],
        /// Noise Error Interrupt Enable
        NEIE OFFSET(26) NUMBITS(1) [],
        /// Framing Error Interrupt Enable
        FEIE OFFSET(25) NUMBITS(1) [],
        /// Parity Error Interrupt Enable
        PEIE OFFSET(24) NUMBITS(1) [],
        /// Transmit Interrupt Enable
        TIE OFFSET(23) NUMBITS(1) [],
        /// Transmission Complete Interrupt Enable for
        TCIE OFFSET(22) NUMBITS(1) [],
        /// Receiver Interrupt Enable
        RIE OFFSET(21) NUMBITS(1) [],
        /// Idle Line Interrupt Enable
        ILIE OFFSET(20) NUMBITS(1) [],
        /// Transmitter Enable
        TE OFFSET(19) NUMBITS(1) [],
        /// Receiver Enable
        RE OFFSET(18) NUMBITS(1) [],
        /// Receiver Wakeup Control
        RWU OFFSET(17) NUMBITS(1) [],
        /// Send Break
        SBK OFFSET(16) NUMBITS(1) [],
        /// Match 1 Interrupt Enable
        MA1IE OFFSET(15) NUMBITS(1) [],
        /// Match 2 Interrupt Enable
        MA2IE OFFSET(14) NUMBITS(1) [],
        /// 7-Bit Mode Select
        M7 OFFSET(11) NUMBITS(1) [],
        /// Idle Configuration
        IDLECFG OFFSET(8) NUMBITS(3) [],
        /// Loop Mode Select
        LOOPS OFFSET(7) NUMBITS(1) [],
        /// Doze Enable
        DOZEEN OFFSET(6) NUMBITS(1) [],
        /// Receiver Source Select
        RSRC OFFSET(5) NUMBITS(1) [],
        /// 9-Bit or 8-Bit Mode Select
        M OFFSET(4) NUMBITS(1) [],
        /// Receiver Wakeup Method Select
        WAKE OFFSET(3) NUMBITS(1) [],
        /// Idle Line Type Select
        ILT OFFSET(2) NUMBITS(1) [],
        /// Parity Enable
        PE OFFSET(1) NUMBITS(1) [],
        /// Parity Type
        PT OFFSET(0) NUMBITS(1) []
    ],

    DATA [
        /// NOISY
        NOISY OFFSET(15) NUMBITS(8) [],
        /// PARITYE
        PARITYE OFFSET(14) NUMBITS(8) [],
        /// Frame Error / Transmit Special Character
        FRETSC OFFSET(13) NUMBITS(8) [],
        /// Receive Buffer Empty
        RXEMPT OFFSET(12) NUMBITS(8) [],
        /// Idle Line
        IDLINE OFFSET(11) NUMBITS(8) [],
        /// R9T9
        R9T9 OFFSET(9) NUMBITS(8) [],
        /// R8T8
        R8T8 OFFSET(8) NUMBITS(8) [],
        /// R7T7
        R7T7 OFFSET(7) NUMBITS(8) [],
        /// R6T6
        R6T6 OFFSET(6) NUMBITS(8) [],
        /// R5T5
        R5T5 OFFSET(5) NUMBITS(8) [],
        /// R4T4
        R4T4 OFFSET(4) NUMBITS(8) [],
        /// R3T3
        R3T3 OFFSET(3) NUMBITS(8) [],
        /// R2T2
        R2T2 OFFSET(2) NUMBITS(8) [],
        /// R1T1
        R1T1 OFFSET(1) NUMBITS(8) [],
        /// R0T0
        R0T0 OFFSET(0) NUMBITS(8) []
    ],

    MATCH [
        /// Match Address 2
        MA2 OFFSET(16) NUMBITS(10) [],
        /// Match Address 1
        MA1 OFFSET(0) NUMBITS(10) []
    ],

    MODIR [
        /// Infrared enable
        IREN OFFSET(18) NUMBITS(1) [],
        /// Transmitter narrow pulse
        TNP OFFSET(16) NUMBITS(2) [],
        /// Receive RTS Configuration
        RTSWATER OFFSET(8) NUMBITS(2) [],
        /// Transmit CTS Source
        TXCTSSRC OFFSET(5) NUMBITS(1) [],
        /// Transmit CTS Configuration
        TXCTSC OFFSET(4) NUMBITS(1) [],
        /// Receiver request-to-send enable
        RXRTSE OFFSET(3) NUMBITS(1) [],
        /// Transmitter request-to-send polarity
        TXRTSPOL OFFSET(2) NUMBITS(1) [],
        /// Transmitter request-to-send enable
        TXRTSE OFFSET(1) NUMBITS(1) [],
        /// Transmitter clear-to-send enable
        TXCTSE OFFSET(0) NUMBITS(1) []
    ],

    FIFO [
        /// Transmit Buffer/FIFO Empty
        TXEMPT OFFSET(23) NUMBITS(1) [],
        /// Receive Buffer/FIFO Empty
        RXEMPT OFFSET(22) NUMBITS(1) [],
        /// Transmitter Buffer Overflow Flag
        TXOF OFFSET(17) NUMBITS(1) [],
        /// Receiver Buffer Underflow Flag
        RXUF OFFSET(16) NUMBITS(1) [],
        /// Transmit FIFO/Buffer Flush
        TXFLUSH OFFSET(15) NUMBITS(1) [],
        /// Receive FIFO/Buffer Flush
        RXFLUSH OFFSET(14) NUMBITS(1) [],
        /// Receiver Idle Empty Enable
        RXIDEN OFFSET(10) NUMBITS(2) [],
        /// Transmit FIFO Overflow Interrupt Enable
        TXOFE OFFSET(9) NUMBITS(1) [],
        /// Receive FIFO Underflow Interrupt Enable
        RXUFE OFFSET(8) NUMBITS(1) [],
        /// Transmit FIFO Enable
        TXFE OFFSET(7) NUMBITS(1) [],
        /// Transmit FIFO Buffer Depth
        TXFIFOSIZE OFFSET(4) NUMBITS(3) [],
        /// Receive FIFO Enable
        RXFE OFFSET(3) NUMBITS(1) [],
        /// Receive FIFO Buffer Depth
        RXFIFOSIZE OFFSET(0) NUMBITS(3) []
    ],

    WATER [
        /// Receive Counter
        RXCOUNT OFFSET(24) NUMBITS(3) [],
        /// Receive Watermark
        RXWATER OFFSET(16) NUMBITS(2) [],
        /// Transmit Counter
        TXCOUNT OFFSET(8) NUMBITS(3) [],
        /// Transmit Watermark
        TXWATER OFFSET(0) NUMBITS(2) []
    ]
];

const LPUART1_BASE: StaticRef<LpuartRegisters> =
    unsafe { StaticRef::new(0x40184000 as *const LpuartRegisters) };
const LPUART2_BASE: StaticRef<LpuartRegisters> =
    unsafe { StaticRef::new(0x4018_8000 as *const LpuartRegisters) };

#[derive(Copy, Clone, PartialEq)]
enum LPUARTStateTX {
    Idle,
    Transmitting,
    AbortRequested,
}

#[derive(Copy, Clone, PartialEq)]
enum USARTStateRX {
    Idle,
    Receiving,
    AbortRequested,
}

pub struct Lpuart<'a> {
    registers: StaticRef<LpuartRegisters>,
    clock: LpuartClock,

    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_position: Cell<usize>,
    tx_len: Cell<usize>,
    tx_status: Cell<LPUARTStateTX>, // rx_len: Cell<usize>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_position: Cell<usize>,
    rx_len: Cell<usize>,
    rx_status: Cell<USARTStateRX>,
}

pub static mut LPUART1: Lpuart = Lpuart::new(
    LPUART1_BASE,
    LpuartClock(ccm::PeripheralClock::CCGR5(ccm::HCLK5::LPUART1)),
);

pub static mut LPUART2: Lpuart = Lpuart::new(
    LPUART2_BASE,
    LpuartClock(ccm::PeripheralClock::CCGR0(ccm::HCLK0::LPUART2)),
);

impl<'a> Lpuart<'a> {
    const fn new(base_addr: StaticRef<LpuartRegisters>, clock: LpuartClock) -> Lpuart<'a> {
        Lpuart {
            registers: base_addr,
            clock: clock,

            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),

            tx_buffer: TakeCell::empty(),
            tx_position: Cell::new(0),
            tx_len: Cell::new(0),
            tx_status: Cell::new(LPUARTStateTX::Idle),

            rx_buffer: TakeCell::empty(),
            rx_position: Cell::new(0),
            rx_len: Cell::new(0),
            rx_status: Cell::new(USARTStateRX::Idle),
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

    pub fn set_baud(&self) {
        // Set the Baud Rate Modulo Divisor
        self.registers.baud.modify(BAUD::SBR.val(139 as u32));
    }

    // for use by panic in io.rs
    pub fn send_byte(&self, byte: u8) {
        // loop until TDRE (Transmit data register empty) becomes 1
        while !self.registers.stat.is_set(STAT::TDRE) {}

        self.registers.data.set(byte.into());

        while !self.registers.stat.is_set(STAT::TC) {}
    }

    fn enable_transmit_complete_interrupt(&self) {
        self.registers.ctrl.modify(CTRL::TIE::SET);
    }

    fn disable_transmit_complete_interrupt(&self) {
        self.registers.ctrl.modify(CTRL::TIE::CLEAR);
    }

    fn clear_transmit_complete(&self) {
        self.registers.stat.modify(STAT::TDRE::CLEAR);
    }

    fn enable_receive_interrupt(&self) {
        self.registers.ctrl.modify(CTRL::RIE::SET);
    }

    fn disable_receive_interrupt(&self) {
        self.registers.ctrl.modify(CTRL::RIE::CLEAR);
    }

    fn clear_overrun(&self) {
        self.registers.ctrl.modify(CTRL::ORIE::CLEAR);
    }

    pub fn handle_interrupt(&self) {
        if self.registers.stat.is_set(STAT::TDRE) {
            self.clear_transmit_complete();
            self.disable_transmit_complete_interrupt();

            // ignore IRQ if not transmitting
            if self.tx_status.get() == LPUARTStateTX::Transmitting {
                let position = self.tx_position.get();
                if position < self.tx_len.get() {
                    self.tx_buffer.map(|buf| {
                        self.registers.data.set(buf[position].into());
                        self.tx_position.replace(self.tx_position.get() + 1);
                        self.enable_transmit_complete_interrupt();
                    });
                } else {
                    // transmission done
                    self.tx_status.replace(LPUARTStateTX::Idle);
                }
                // notify client if transfer is done
                if self.tx_status.get() == LPUARTStateTX::Idle {
                    self.tx_client.map(|client| {
                        if let Some(buf) = self.tx_buffer.take() {
                            client.transmitted_buffer(buf, self.tx_len.get(), ReturnCode::SUCCESS);
                        }
                    });
                }
            } else if self.tx_status.get() == LPUARTStateTX::AbortRequested {
                self.tx_status.replace(LPUARTStateTX::Idle);
                self.tx_client.map(|client| {
                    if let Some(buf) = self.tx_buffer.take() {
                        client.transmitted_buffer(buf, self.tx_position.get(), ReturnCode::ECANCEL);
                    }
                });
            }
        }

        if self.registers.stat.is_set(STAT::RDRF) {
            let byte = self.registers.data.get() as u8;

            self.disable_receive_interrupt();

            // ignore IRQ if not receiving
            if self.rx_status.get() == USARTStateRX::Receiving {
                if self.rx_position.get() < self.rx_len.get() {
                    self.rx_buffer.map(|buf| {
                        buf[self.rx_position.get()] = byte;
                        self.rx_position.replace(self.rx_position.get() + 1);
                    });
                }
                if self.rx_position.get() == self.rx_len.get() {
                    // reception done
                    self.rx_status.replace(USARTStateRX::Idle);
                } else {
                    self.enable_receive_interrupt();
                }
                // notify client if transfer is done
                if self.rx_status.get() == USARTStateRX::Idle {
                    self.rx_client.map(|client| {
                        if let Some(buf) = self.rx_buffer.take() {
                            client.received_buffer(
                                buf,
                                self.rx_len.get(),
                                ReturnCode::SUCCESS,
                                hil::uart::Error::None,
                            );
                        }
                    });
                }
            } else if self.rx_status.get() == USARTStateRX::AbortRequested {
                self.rx_status.replace(USARTStateRX::Idle);
                self.rx_client.map(|client| {
                    if let Some(buf) = self.rx_buffer.take() {
                        client.received_buffer(
                            buf,
                            self.rx_position.get(),
                            ReturnCode::ECANCEL,
                            hil::uart::Error::Aborted,
                        );
                    }
                });
            }
        }

        if self.registers.stat.is_set(STAT::OR) {
            self.clear_overrun();
            self.rx_status.replace(USARTStateRX::Idle);
            self.rx_client.map(|client| {
                if let Some(buf) = self.rx_buffer.take() {
                    client.received_buffer(
                        buf,
                        self.rx_position.get(),
                        ReturnCode::ECANCEL,
                        hil::uart::Error::OverrunError,
                    );
                }
            });
        }
    }
}

impl<'a> hil::uart::Transmit<'a> for Lpuart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.tx_status.get() == LPUARTStateTX::Idle {
            if tx_len <= tx_data.len() {
                self.tx_buffer.put(Some(tx_data));
                self.tx_position.set(0);
                self.tx_len.set(tx_len);
                self.tx_status.set(LPUARTStateTX::Transmitting);
                self.enable_transmit_complete_interrupt();
                (ReturnCode::SUCCESS, None)
            } else {
                (ReturnCode::ESIZE, Some(tx_data))
            }
        } else {
            (ReturnCode::EBUSY, Some(tx_data))
        }
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        if self.tx_status.get() != LPUARTStateTX::Idle {
            self.tx_status.set(LPUARTStateTX::AbortRequested);
            ReturnCode::EBUSY
        } else {
            ReturnCode::SUCCESS
        }
    }
}

impl<'a> hil::uart::Configure for Lpuart<'a> {
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

        unsafe {
            self.disable_clock();
            ccm::CCM.disable_uart_clock_mux();
            ccm::CCM.disable_uart_clock_podf();
            self.enable_clock();
        }
        // Reset the LPUART using software
        self.registers.global.modify(GLOBAL::RST::SET);
        self.registers.global.modify(GLOBAL::RST::CLEAR);

        // Enable Bothedge sampling
        self.registers.baud.modify(BAUD::BOTHEDGE::SET);

        // Set Oversampling Ratio to 5 (the value written is -1)
        self.registers.baud.modify(BAUD::OSR.val(0b100 as u32));

        // Set the Baud Rate Modulo Divisor
        self.registers.baud.modify(BAUD::SBR.val(139 as u32));

        // Set bit count and parity mode
        self.registers.baud.modify(BAUD::M10::CLEAR);

        self.registers.ctrl.modify(CTRL::PE::CLEAR);
        self.registers.ctrl.modify(CTRL::PT::CLEAR);
        self.registers.ctrl.modify(CTRL::M::CLEAR);
        self.registers.ctrl.modify(CTRL::ILT::CLEAR);
        self.registers.ctrl.modify(CTRL::IDLECFG::CLEAR);

        // Set 1 stop bit
        self.registers.baud.modify(BAUD::SBNS::CLEAR);

        // Clear RX and TX watermarks
        self.registers.water.modify(WATER::RXWATER::CLEAR);
        self.registers.water.modify(WATER::TXWATER::CLEAR);

        // Enable TX and RX FIFO
        self.registers.fifo.modify(FIFO::TXFE::CLEAR);
        self.registers.fifo.modify(FIFO::RXFE::CLEAR);

        // Flush RX FIFO and TX FIFO
        self.registers.fifo.modify(FIFO::TXFLUSH::CLEAR);
        self.registers.fifo.modify(FIFO::RXFLUSH::CLEAR);

        // Clear all status registers
        self.registers.stat.modify(STAT::RXEDGIF::SET);
        self.registers.stat.modify(STAT::IDLE::SET);
        self.registers.stat.modify(STAT::OR::SET);
        self.registers.stat.modify(STAT::NF::SET);
        self.registers.stat.modify(STAT::FE::SET);
        self.registers.stat.modify(STAT::PF::SET);
        self.registers.stat.modify(STAT::MA1F::SET);
        self.registers.stat.modify(STAT::MA2F::SET);

        // Set the CTS configuration/TX CTS source.
        self.registers.modir.modify(MODIR::TXCTSC::CLEAR);
        self.registers.modir.modify(MODIR::TXCTSSRC::CLEAR);

        // Set as LSB
        self.registers.stat.modify(STAT::MSBF::CLEAR);

        // Enable TX and RX over LPUART
        self.registers.ctrl.modify(CTRL::TE::SET);
        self.registers.ctrl.modify(CTRL::RE::SET);

        ReturnCode::SUCCESS
    }
}

impl<'a> hil::uart::Receive<'a> for Lpuart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.rx_status.get() == USARTStateRX::Idle {
            if rx_len <= rx_buffer.len() {
                self.rx_buffer.put(Some(rx_buffer));
                self.rx_position.set(0);
                self.rx_len.set(rx_len);
                self.rx_status.set(USARTStateRX::Receiving);
                self.enable_receive_interrupt();
                (ReturnCode::SUCCESS, None)
            } else {
                (ReturnCode::ESIZE, Some(rx_buffer))
            }
        } else {
            (ReturnCode::EBUSY, Some(rx_buffer))
        }
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        if self.rx_status.get() != USARTStateRX::Idle {
            self.rx_status.set(USARTStateRX::AbortRequested);
            ReturnCode::EBUSY
        } else {
            ReturnCode::SUCCESS
        }
    }
}

impl<'a> hil::uart::UartData<'a> for Lpuart<'a> {}
impl<'a> hil::uart::Uart<'a> for Lpuart<'a> {}
struct LpuartClock(ccm::PeripheralClock);

impl ClockInterface for LpuartClock {
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
