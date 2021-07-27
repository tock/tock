use core::cell::Cell;
use kernel::hil;
use kernel::hil::uart::ReceiveClient;
use kernel::hil::uart::{
    Configure, Parameters, Parity, Receive, StopBits, Transmit, TransmitClient, Width,
};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    /// controls serial port
    UartRegisters {
        (0x000 => uartdr: ReadWrite<u32, UARTDR::Register>),

        (0x004 => uartrsr: ReadWrite<u32, UARTRSR::Register>),
        (0x008 => _reserved0),

        (0x018 => uartfr: ReadOnly<u32, UARTFR::Register>),
        (0x01c => _reserved1),

        (0x020 => uartilpr: ReadWrite<u32, UARTILPR::Register>),

        (0x024 => uartibrd: ReadWrite<u32, UARTIBRD::Register>),

        (0x028 => uartfbrd: ReadWrite<u32, UARTFBRD::Register>),

        (0x02c => uartlcr_h: ReadWrite<u32, UARTLCR_H::Register>),

        (0x030 => uartcr: ReadWrite<u32, UARTCR::Register>),

        (0x034 => uartifls: ReadWrite<u32, UARTIFLS::Register>),

        (0x038 => uartimsc: ReadWrite<u32, UARTIMSC::Register>),

        (0x03c => uartris: ReadOnly<u32, UARTRIS::Register>),

        (0x040 => uartmis: ReadOnly<u32, UARTMIS::Register>),

        (0x044 => uarticr: ReadWrite<u32, UARTICR::Register>),

        (0x048 => uartdmacr: ReadWrite<u32, UARTDMACR::Register>),
        (0x04c => _reserved2),

        (0xfe0 => uartperiphid0: ReadOnly<u32, UARTPERIPHID0::Register>),

        (0xfe4 => uartperiphid1: ReadOnly<u32, UARTPERIPHID1::Register>),

        (0xfe8 => uartperiphid2: ReadOnly<u32, UARTPERIPHID2::Register>),

        (0xfec => uartperiphid3: ReadOnly<u32, UARTPERIPHID3::Register>),

        (0xff0 => uartpcellid0: ReadOnly<u32, UARTPCELLID0::Register>),

        (0xff4 => uartpcellid1: ReadOnly<u32, UARTPCELLID1::Register>),

        (0xff8 => uartpcellid2: ReadOnly<u32, UARTPCELLID2::Register>),

        (0xffc => uartpcellid3: ReadOnly<u32, UARTPCELLID3::Register>),

        (0x1000 => @END),
    }
}

register_bitfields! [u32,
    /// data register
    UARTDR [
        /// data bytes
        DATA OFFSET(0) NUMBITS(8) [],
        /// framing error
        FE OFFSET(8) NUMBITS(1) [],
        /// parity error
        PE OFFSET(9) NUMBITS(1) [],
        /// break error
        BE OFFSET(10) NUMBITS(1) [],
        /// overrun error
        OE OFFSET(11) NUMBITS(1) []
    ],
    /// receive status register/ error clear register
    UARTRSR [
        /// framing error
        FE OFFSET(0) NUMBITS(1) [],
        /// parity error
        PE OFFSET(1) NUMBITS(1) [],
        /// break error
        BE OFFSET(2) NUMBITS(1) [],
        /// overrun error
        OE OFFSET(3) NUMBITS(1) []
    ],
    /// flag register
    UARTFR  [
        /// clear to send
        CTS OFFSET(0) NUMBITS(1) [],
        /// data set ready
        DSR OFFSET(1) NUMBITS(1) [],
        /// data carrier detect
        DCD OFFSET(2) NUMBITS(1) [],
        /// busy
        BUSY OFFSET(3) NUMBITS(1) [],
        /// receive FIFO empty
        RXFE OFFSET(4) NUMBITS(1) [],
        /// transmit FIFO full
        TXFF OFFSET(5) NUMBITS(1) [],
        /// receive FIFO full
        RXFF OFFSET(6) NUMBITS(1) [],
        /// transmit FIFO empty
        TXFE OFFSET(7) NUMBITS(1) [],
        /// ring indicator
        RI OFFSET(8) NUMBITS(1) []
    ],
    /// IrDA low-power counter register
    UARTILPR [
        /// 8-bit low-power divisor value
        ILPDVSR OFFSET(0) NUMBITS(8) []
    ],
    /// integer baud rate register
    UARTIBRD [
        /// the integer baud rate divisor
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],
    /// fractional baud rate register
    UARTFBRD [
        /// the fractional baud rate divisor
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],
    /// line control register
    UARTLCR_H [
        /// send break
        BRK OFFSET(0) NUMBITS(1) [],
        /// parity enable
        PEN OFFSET(1) NUMBITS(1) [],
        /// even parity select
        EPS OFFSET(2) NUMBITS(1) [],
        /// two stop bits select
        STP2 OFFSET(3) NUMBITS(1) [],
        /// enable FIFOs
        FEN OFFSET(4) NUMBITS(1) [],
        /// word length
        WLEN OFFSET(5) NUMBITS(2) [
            BITS_8 = 0b11,
            BITS_7 = 0b10,
            BITS_6 = 0b01,
            BITS_5 = 0b00
        ],
        /// stick parity select
        SPS OFFSET(7) NUMBITS(1) []
    ],
    /// control register
    UARTCR [
        /// UART enable
        UARTEN OFFSET(0) NUMBITS(1) [],
        /// SIR enable
        SIREN OFFSET(1) NUMBITS(1) [],
        /// SIR low-power IrDA mode
        SIRLP OFFSET(2) NUMBITS(1) [],

        //RESERVED OFFSET(3) NUMBITS(3) [],
        /// loopback enable
        LBE OFFSET(7) NUMBITS(1) [],
        /// transmit enable
        TXE OFFSET(8) NUMBITS(1) [],
        /// receive enable
        RXE OFFSET(9) NUMBITS(1) [],
        /// data transmit ready
        DTR OFFSET(10) NUMBITS(1) [],
        /// request to send
        RTS OFFSET(11) NUMBITS(1) [],
        /// the complement of the UART Out1 (nUARTOut1) modem status output
        OUT1 OFFSET(12) NUMBITS(1) [],
        /// the complement of the UART Out2 (nUARTOut2) modem status output
        OUT2 OFFSET(13) NUMBITS(1) [],
        /// RTS hardware flow control enable
        RTSEN OFFSET(14) NUMBITS(1) [],
        /// CTS hardware flow control enable
        CTSEN OFFSET(15) NUMBITS(1) []
    ],
    /// interrupt FIFO level select register
    UARTIFLS [
        /// transmit interrupt FIFO level select
        TXIFLSEL OFFSET(0) NUMBITS(3) [
            FIFO_1_8 = 0b000,
            FIFO_1_4 = 0b001,
            FIFO_1_2 = 0b010,
            FIFO_3_4 = 0b011,
            FIFO_7_8 = 0b100,
        ],
        /// receive interrupt FIFO level select
        RXIFLSEL OFFSET(3) NUMBITS(3) [
            FIFO_1_8 = 0b000,
            FIFO_1_4 = 0b001,
            FIFO_1_2 = 0b010,
            FIFO_3_4 = 0b011,
            FIFO_7_8 = 0b100,
        ]
    ],

    /// interrupt mask set/clear register
    UARTIMSC [
        /// nUARTRI modem interrupt mask
        RIMIM OFFSET(0) NUMBITS(1) [],
        /// nUARTCTS modem interrupt mask
        CTSMIM OFFSET(1) NUMBITS(1) [],
        /// nUARTDCD modem interrupt mask
        DCDMIM OFFSET(2) NUMBITS(1) [],
        /// nUARTDSR modem interrupt mask
        DSRMIM OFFSET(3) NUMBITS(1) [],
        /// receive interrupt mask
        RXIM OFFSET(4) NUMBITS(1) [],
        /// transmit interrupt mask
        TXIM OFFSET(5) NUMBITS(1) [],
        /// receive timeout interrupt mask
        RTIM OFFSET(6) NUMBITS(1) [],
        /// framing error interrupt mask
        FEIM OFFSET(7) NUMBITS(1) [],
        /// parity error interrupt mask
        PEIM OFFSET(8) NUMBITS(1) [],
        /// break error interrupt mask
        BEIM OFFSET(9) NUMBITS(1) [],
        /// overrun error interrupt mask
        OEIM OFFSET(10) NUMBITS(1) []
    ],
    /// raw interrupt status register
    UARTRIS [
        /// nUARTRI modem interrupt status
        RIRMIS OFFSET(0) NUMBITS(1) [],
        /// nUARTCTS modem interrupt status
        CTSRMIS OFFSET(1) NUMBITS(1) [],
        /// nUARTDCD modem interrupt status
        DCDRMIS OFFSET(2) NUMBITS(1) [],
        /// nUARTDSR modem interrupt status
        DSRRMIS OFFSET(3) NUMBITS(1) [],
        /// receive interrupt status
        RXRIS OFFSET(4) NUMBITS(1) [],
        /// transmit interrupt status
        TXRIS OFFSET(5) NUMBITS(1) [],
        /// receive timeout interrupt status
        RTRIS OFFSET(6) NUMBITS(1) [],
        /// framing error interrupt status
        FERIS OFFSET(7) NUMBITS(1) [],
        /// parity error interrupt status
        PERIS OFFSET(8) NUMBITS(1) [],
        /// break error interrupt status
        BERIS OFFSET(9) NUMBITS(1) [],
        /// overrun error interrupt status
        OERIS OFFSET(10) NUMBITS(1) []
    ],
    /// masked interrupt status register
    UARTMIS [
        /// nUARTRI modem masked interrupt status
        RIMMIS OFFSET(0) NUMBITS(1) [],
        /// nUARTCTS modem masked interrupt status
        CTSMMIS OFFSET(1) NUMBITS(1) [],
        /// nUARTDCD modem masked interrupt status
        DCDMMIS OFFSET(2) NUMBITS(1) [],
        /// nUARTDSR modem masked interrupt status
        DSRMMIS OFFSET(3) NUMBITS(1) [],
        /// receive masked interrupt status
        RXMIS OFFSET(4) NUMBITS(1) [],
        /// transmit masked interrupt status
        TXMIS OFFSET(5) NUMBITS(1) [],
        /// receive timeout masked interrupt status
        RTMIS OFFSET(6) NUMBITS(1) [],
        /// framing error masked interrupt status
        FEMIS OFFSET(7) NUMBITS(1) [],
        /// parity error masked interrupt status
        PEMIS OFFSET(8) NUMBITS(1) [],
        /// break error masked interrupt status
        BEMIS OFFSET(9) NUMBITS(1) [],
        /// overrun error masked interrupt status
        OEMIS OFFSET(10) NUMBITS(1) []
    ],
    /// interrupt clear register
    UARTICR [
        /// nUARTRI modem interrupt clear
        RIMIC OFFSET(0) NUMBITS(1) [],
        /// nUARTCTS modem interrupt clear
        CTSMIC OFFSET(1) NUMBITS(1) [],
        /// nUARTDCD modem interrupt clear
        DCDMIC OFFSET(2) NUMBITS(1) [],
        /// nUARTDSR modem interrupt clear
        DSRMIC OFFSET(3) NUMBITS(1) [],
        /// receive interrupt clear
        RXIC OFFSET(4) NUMBITS(1) [],
        /// transmit interrupt clear
        TXIC OFFSET(5) NUMBITS(1) [],
        /// receive timeout interrupt clear
        RTIC OFFSET(6) NUMBITS(1) [],
        /// framing error interrupt clear
        FEIC OFFSET(7) NUMBITS(1) [],
        /// parity error interrupt clear
        PEIC OFFSET(8) NUMBITS(1) [],
        /// break error interrupt clear
        BEIC OFFSET(9) NUMBITS(1) [],
        /// overrun error interrupt clear
        OEIC OFFSET(10) NUMBITS(1) []
    ],
    /// DMA control register
    UARTDMACR [
        /// Receive DMA enable
        RXDMAE OFFSET(0) NUMBITS(1) [],
        /// transmit DMA enable
        TXDMAE OFFSET(1) NUMBITS(1) [],
        /// DMA on error
        DMAONERR OFFSET(2) NUMBITS(1) []
    ],
    /// UARTPeriphID0 register
    UARTPERIPHID0 [
        /// these bits read back as 0x11
        PARTNUMBER0 OFFSET(0) NUMBITS(8) []
    ],
    /// UARTPeriphID1 register
    UARTPERIPHID1 [
        /// these bits read back as 0x0
        PARTNUMBER1 OFFSET(0) NUMBITS(4) [],
        /// these bits read back as 0x1
        DESIGNER0 OFFSET(4) NUMBITS(4) []
    ],
    /// UARTPeriphID2 register
    UARTPERIPHID2 [
        /// these bits read back as 0x4
        DESIGNER1 OFFSET(0) NUMBITS(4) [],
        /// this field depends on the revision of the UART: r1p0 0x0 r1p1 0x1 r1p3 0x2 r1p4 0x2 r1p5 0x3
        REVISION OFFSET(4) NUMBITS(4) []
    ],
    /// UARTPeriphID3 register
    UARTPERIPHID3 [
        /// these bits read back as 0x00
        CONFIGURATION OFFSET(0) NUMBITS(8) []
    ],
    /// UARTPCellID0 register
    UARTPCELLID0 [
        /// these bits read back as 0x0D
        UARTPCELLID0 OFFSET(0) NUMBITS(8) []
    ],
    /// UARTPCellID1 register
    UARTPCELLID1 [
        /// these bits read back as 0xF0
        UARTPCELLID1 OFFSET(0) NUMBITS(8) []
    ],
    /// UARTPCellID2 register
    UARTPCELLID2 [
        /// these bits read back as 0x05
        UARTPCELLID2 OFFSET(0) NUMBITS(8) []
    ],
    /// UARTPCellID3 register
    UARTPCELLID3 [
        /// these bits read back as 0xB1
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
    unsafe { StaticRef::new(0x40034000 as *const UartRegisters) };

const UART1_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x40038000 as *const UartRegisters) };

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
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
    pub const fn new_uart0() -> Self {
        Self {
            registers: UART0_BASE,
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
    pub const fn new_uart1() -> Self {
        Self {
            registers: UART1_BASE,
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
                } else if self.tx_status.get() == UARTStateTX::AbortRequested {
                    self.tx_status.replace(UARTStateTX::Idle);
                    self.tx_client.map(|client| {
                        if let Some(buf) = self.tx_buffer.take() {
                            client.transmitted_buffer(
                                buf,
                                self.tx_position.get(),
                                Err(ErrorCode::CANCEL),
                            );
                        }
                    });
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
            } else if self.rx_status.get() == UARTStateRX::AbortRequested {
                self.rx_status.replace(UARTStateRX::Idle);
                self.rx_client.map(|client| {
                    if let Some(buf) = self.rx_buffer.take() {
                        client.received_buffer(
                            buf,
                            self.rx_position.get(),
                            Err(ErrorCode::CANCEL),
                            hil::uart::Error::Aborted,
                        );
                    }
                });
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
        if self.registers.uartcr.is_set(UARTCR::UARTEN)
            && (self.registers.uartcr.is_set(UARTCR::RXE)
                || self.registers.uartcr.is_set(UARTCR::TXE))
        {
            true
        } else {
            false
        }
    }
}

impl Configure for Uart<'_> {
    fn configure(&self, params: Parameters) -> Result<(), ErrorCode> {
        self.disable();
        self.registers.uartlcr_h.modify(UARTLCR_H::FEN::CLEAR);

        let clk = 125_000_000;

        // Calculate baud rate
        let baud_rate_div = 8 * clk / params.baud_rate;
        let mut baud_ibrd = baud_rate_div >> 7;
        let mut baud_fbrd = ((baud_rate_div & 0x7f) + 1) / 2;

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
        Err(ErrorCode::FAIL)
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
            self.rx_status.set(UARTStateRX::AbortRequested);
            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }
}
