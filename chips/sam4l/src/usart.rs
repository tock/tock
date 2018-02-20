//! Implementation of the SAM4L USART peripheral.
//!
//! Supports UART and SPI master modes.

use core::cell::Cell;
use core::cmp;
use dma;
use kernel::ReturnCode;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
// other modules
use kernel::hil;
// local modules
use pm;

// Register map for SAM4L USART
#[repr(C)]
struct UsartRegisters {
    cr: WriteOnly<u32, Control::Register>,       // 0x00
    mr: ReadWrite<u32, Mode::Register>,          // 0x04
    ier: WriteOnly<u32, Interrupt::Register>,    // 0x08
    idr: WriteOnly<u32, Interrupt::Register>,    // 0x0C
    imr: ReadOnly<u32, Interrupt::Register>,     // 0x10
    csr: ReadOnly<u32, ChannelStatus::Register>, // 0x14
    rhr: ReadOnly<u32, ReceiverHold::Register>,  // 0x18
    thr: WriteOnly<u32, TransmitHold::Register>, // 0x1C
    brgr: ReadWrite<u32, BaudRate::Register>,    // 0x20
    rtor: ReadWrite<u32, RxTimeout::Register>,   // 0x24
    ttgr: ReadWrite<u32, TxTimeGuard::Register>, // 0x28
    _reserved0: [ReadOnly<u32>; 5],
    fidi: ReadWrite<u32, FidiRatio::Register>, // 0x40
    ner: ReadOnly<u32, NumErrors::Register>,   // 0x44
    _reserved1: ReadOnly<u32>,
    ifr: ReadWrite<u32, IrdaFilter::Register>, // 0x4C
    man: ReadWrite<u32, Manchester::Register>, // 0x50
    linmr: ReadWrite<u32, LinMode::Register>,  // 0x54
    linir: ReadWrite<u32, LinID::Register>,    // 0x58
    linbr: ReadOnly<u32, LinBaud::Register>,   // 0x5C
    _reserved2: [ReadOnly<u32>; 33],
    wpmr: ReadWrite<u32, ProtectMode::Register>,  // 0xE4
    wpsr: ReadOnly<u32, ProtectStatus::Register>, // 0xE8
    _reserved3: [ReadOnly<u32>; 4],
    version: ReadOnly<u32, Version::Register>, // 0xFC
}

register_bitfields![u32,
    Control [
        LINWKUP 21,
        LINABT  20,
        RTSDIS  19,
        RTSEN   18,
        DTRDIS  17,
        DTREN   16,
        RETTO   15,
        RSTNACK 14,
        RSTIT   13,
        SENDA   12,
        STTTO   11,
        STPBRK  10,
        STTBRK   9,
        RSTSTA   8,
        TXDIS    7,
        TXEN     6,
        RXDIS    5,
        RXEN     4,
        RSTTX    3,
        RSTRX    2
    ],
    Mode [
        ONEBIT        OFFSET(31)  NUMBITS(1) [],
        MODSYNC       OFFSET(30)  NUMBITS(1) [],
        MAN           OFFSET(29)  NUMBITS(1) [],
        FILTER        OFFSET(28)  NUMBITS(1) [],
        MAX_ITERATION OFFSET(24)  NUMBITS(3) [],
        INVDATA       OFFSET(23)  NUMBITS(1) [],
        VAR_SYNC      OFFSET(22)  NUMBITS(1) [],
        DSNACK        OFFSET(21)  NUMBITS(1) [],
        INACK         OFFSET(20)  NUMBITS(1) [],
        OVER          OFFSET(19)  NUMBITS(1) [],
        CLKO          OFFSET(18)  NUMBITS(1) [],
        MODE9         OFFSET(17)  NUMBITS(1) [],
        MSBF          OFFSET(16)  NUMBITS(1) [],
        CHMODE        OFFSET(14)  NUMBITS(2) [
            NORMAL    = 0b00,
            ECHO      = 0b01,
            LOOPBACK  = 0xb10,
            RLOOPBACK = 0b11
        ],
        NBSTOP        OFFSET(12)  NUMBITS(2) [
            BITS_1_1  = 0b00,
            BITS_15_R = 0b01,
            BITS_2_2  = 0b10,
            BITS_R_R  = 0b11
        ],
        PAR           OFFSET(9)   NUMBITS(3) [
            EVEN    = 0b000,
            ODD     = 0b001,
            SPACE   = 0b010,
            MARK    = 0b011,
            NONE    = 0b100,
            MULTID  = 0b110
        ],
        SYNC          OFFSET(8)   NUMBITS(1) [],
        CHRL          OFFSET(6)   NUMBITS(2) [
            BITS5  = 0b00,
            BITS6  = 0b01,
            BITS7  = 0b10,
            BITS8  = 0b11
        ],
        USCLKS        OFFSET(4)   NUMBITS(2) [
            CLK_USART     = 0b00,
            CLK_USART_DIV = 0b01,
            RES           = 0b10,
            CLK           = 0b11
        ],
        MODE          OFFSET(0)   NUMBITS(4) [
            NORMAL        = 0b0000,
            RS485         = 0b0001,
            HARD_HAND     = 0b0010,
            MODEM         = 0b0011,
            ISO7816_T0    = 0b0100,
            ISO7816_T1    = 0b0110,
            IRDA          = 0b1000,
            LIN_MASTER    = 0b1010,
            LIN_SLAVE     = 0b1011,
            SPI_MASTER    = 0b1110,
            SPI_SLAVE     = 0b1111
        ]
    ],
    Interrupt [
        LINHTE  31,
        LINSTE  30,
        LINSNRE 29,
        LINCE   28,
        LINIPE  27,
        LINISFE 26,
        LINBE   25,
        MANEA   24,
        MANE    20,
        CTSIC   19,
        DCDIC   18,
        DSRIC   17,
        RIIC    16,
        LINTC   15,
        LINID   14,
        NACK    13,
        RXBUFF  12,
        ITER    10,
        TXEMPTY  9,
        TIMEOUT  8,
        PARE     7,
        FRAME    6,
        OVRE     5,
        RXBRK    2,
        TXRDY    1,
        RXRDY    0
    ],
    ChannelStatus [
        LINHTE  31,
        LINSTE  30,
        LINSNRE 29,
        LINCE   28,
        LINIPE  27,
        LINISFE 26,
        LINBE   25,
        MANERR  24,
        CTS     23,
        DCD     22,
        DSR     21,
        RI      20,
        CTSIC   19,
        DCDIC   18,
        DSRIC   17,
        RIIC    16,
        LINTC   15,
        LINID   14,
        NACK    13,
        RXBUFF  12,
        ITER    10,
        TXEMPTY  9,
        TIMEOUT  8,
        PARE     7,
        FRAME    6,
        OVRE     5,
        RXBRK    2,
        TXRDY    1,
        RXRDY    0
    ],
    ReceiverHold [
        RXSYNH   OFFSET(15)  NUMBITS(1) [],
        RXCHR    OFFSET(0)   NUMBITS(9) []
    ],
    TransmitHold [
        TXSYNH   OFFSET(15)  NUMBITS(1) [],
        TXCHR    OFFSET(0)   NUMBITS(9) []
    ],
    BaudRate [
        FP       OFFSET(16)  NUMBITS(3)  [],
        CD       OFFSET(0)   NUMBITS(16) []
    ],
    RxTimeout [
        TO       OFFSET(0)  NUMBITS(17)  []
    ],
    TxTimeGuard [
        TG       OFFSET(0)  NUMBITS(8)   []
    ],
    FidiRatio [
        RATIO    OFFSET(0)  NUMBITS(11)  []
    ],
    NumErrors [
        NB_ERRORS  OFFSET(0)  NUMBITS(8)  []
    ],
    IrdaFilter [
        FILTER     OFFSET(0)  NUMBITS(8)  []
    ],
    Manchester [
        DRIFT      OFFSET(30) NUMBITS(1)  [],
        RX_MPOL    OFFSET(28) NUMBITS(1)  [],
        RX_PP      OFFSET(24) NUMBITS(2)  [
            ALL_ONE = 0b00,
            ALL_ZERO = 0b01,
            ZERO_ONE = 0b10,
            ONE_ZERO = 0b11
        ],
        RX_PL      OFFSET(16) NUMBITS(4)  [],
        TX_MPOL    OFFSET(12) NUMBITS(1)  [],
        TX_PP      OFFSET(8)  NUMBITS(2)  [
            ALL_ONE = 0b00,
            ALL_ZERO = 0b01,
            ZERO_ONE = 0b10,
            ONE_ZERO = 0b11
        ],
        TX_PL      OFFSET(0)  NUMBITS(4)  []
    ],
    LinMode [
        SYNCDIS   OFFSET(17)  NUMBITS(1) [],
        PDCM      OFFSET(16)  NUMBITS(1) [],
        DLC       OFFSET(8)   NUMBITS(8) [],
        WKUPTYP   OFFSET(7)   NUMBITS(1) [],
        FSDIS     OFFSET(6)   NUMBITS(1) [],
        DLM       OFFSET(5)   NUMBITS(1) [],
        CHKTYP    OFFSET(4)   NUMBITS(1) [],
        CHKDIS    OFFSET(3)   NUMBITS(1) [],
        PARDIS    OFFSET(2)   NUMBITS(1) [],
        NACT      OFFSET(0)   NUMBITS(2) [
            PUBLISH    = 0b00,
            SUBSCRIBE  = 0b01,
            IGNORE     = 0b10,
            RESERVED   = 0b11
        ]
    ],
    LinID [
        IDCHR   OFFSET(0)  NUMBITS(8) []
    ],
    LinBaud [
        LINFP   OFFSET(16) NUMBITS(3)  [],
        LINCD   OFFSET(0)  NUMBITS(16) []
    ],
    ProtectMode [
        WPKEY   OFFSET(8)  NUMBITS(24) [],
        WPEN    OFFSET(0)  NUMBITS(1)  []
    ],
    ProtectStatus [
        WPVSRC  OFFSET(8)  NUMBITS(16) [],
        WPVS    OFFSET(0)  NUMBITS(1)  []
    ],
    Version [
        MFN     OFFSET(16)  NUMBITS(3)  [],
        VERSION OFFSET(0)   NUMBITS(11) []
    ]
];

const USART_BASE_ADDRS: [*mut UsartRegisters; 4] = [
    0x40024000 as *mut UsartRegisters,
    0x40028000 as *mut UsartRegisters,
    0x4002C000 as *mut UsartRegisters,
    0x40030000 as *mut UsartRegisters,
];

#[derive(Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum USARTStateRX {
    Idle,
    DMA_Receiving,
}

#[derive(Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum USARTStateTX {
    Idle,
    DMA_Transmitting,
    Transfer_Completing, // DMA finished, but not all bytes sent
}

#[derive(Copy, Clone)]
enum UsartMode {
    Uart,
    Spi,
    Unused,
}

#[derive(Copy, Clone)]
enum UsartClient<'a> {
    Uart(&'a hil::uart::Client),
    SpiMaster(&'a hil::spi::SpiMasterClient),
}

pub struct USART {
    registers: *mut UsartRegisters,
    clock: pm::Clock,

    usart_mode: Cell<UsartMode>,

    usart_tx_state: Cell<USARTStateTX>,
    usart_rx_state: Cell<USARTStateRX>,

    rx_dma: Cell<Option<&'static dma::DMAChannel>>,
    rx_dma_peripheral: dma::DMAPeripheral,
    rx_len: Cell<usize>,
    tx_dma: Cell<Option<&'static dma::DMAChannel>>,
    tx_dma_peripheral: dma::DMAPeripheral,
    tx_len: Cell<usize>,

    client: Cell<Option<UsartClient<'static>>>,

    spi_chip_select: Cell<Option<&'static hil::gpio::Pin>>,
}

// USART hardware peripherals on SAM4L
pub static mut USART0: USART = USART::new(
    USART_BASE_ADDRS[0],
    pm::PBAClock::USART0,
    dma::DMAPeripheral::USART0_RX,
    dma::DMAPeripheral::USART0_TX,
);
pub static mut USART1: USART = USART::new(
    USART_BASE_ADDRS[1],
    pm::PBAClock::USART1,
    dma::DMAPeripheral::USART1_RX,
    dma::DMAPeripheral::USART1_TX,
);
pub static mut USART2: USART = USART::new(
    USART_BASE_ADDRS[2],
    pm::PBAClock::USART2,
    dma::DMAPeripheral::USART2_RX,
    dma::DMAPeripheral::USART2_TX,
);
pub static mut USART3: USART = USART::new(
    USART_BASE_ADDRS[3],
    pm::PBAClock::USART3,
    dma::DMAPeripheral::USART3_RX,
    dma::DMAPeripheral::USART3_TX,
);

impl USART {
    const fn new(
        base_addr: *mut UsartRegisters,
        clock: pm::PBAClock,
        rx_dma_peripheral: dma::DMAPeripheral,
        tx_dma_peripheral: dma::DMAPeripheral,
    ) -> USART {
        USART {
            registers: base_addr,
            clock: pm::Clock::PBA(clock),

            usart_mode: Cell::new(UsartMode::Unused),

            usart_rx_state: Cell::new(USARTStateRX::Idle),
            usart_tx_state: Cell::new(USARTStateTX::Idle),

            // these get defined later by `chip.rs`
            rx_dma: Cell::new(None),
            rx_dma_peripheral: rx_dma_peripheral,
            rx_len: Cell::new(0),
            tx_dma: Cell::new(None),
            tx_dma_peripheral: tx_dma_peripheral,
            tx_len: Cell::new(0),

            // this gets defined later by `main.rs`
            client: Cell::new(None),

            // This is only used if the USART is in SPI mode.
            spi_chip_select: Cell::new(None),
        }
    }

    pub fn set_dma(&self, rx_dma: &'static dma::DMAChannel, tx_dma: &'static dma::DMAChannel) {
        self.rx_dma.set(Some(rx_dma));
        self.tx_dma.set(Some(tx_dma));
    }

    pub fn enable_rx(&self) {
        self.enable_clock();
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.cr.write(Control::RXEN::SET);
    }

    pub fn enable_tx(&self) {
        self.enable_clock();
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.cr.write(Control::TXEN::SET);
    }

    pub fn disable_rx(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.cr.write(Control::RXDIS::SET);

        self.usart_rx_state.set(USARTStateRX::Idle);
        if self.usart_tx_state.get() == USARTStateTX::Idle {
            // TX disabled too
            self.disable_clock();
        }
    }

    pub fn disable_tx(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.cr.write(Control::TXDIS::SET);

        self.usart_tx_state.set(USARTStateTX::Idle);
        if self.usart_rx_state.get() == USARTStateRX::Idle {
            // RX disabled too
            self.disable_clock();
        }
    }

    pub fn abort_rx(&self, error: hil::uart::Error) {
        if self.usart_rx_state.get() == USARTStateRX::DMA_Receiving {
            self.disable_rx_interrupts();
            self.disable_rx();
            self.usart_rx_state.set(USARTStateRX::Idle);

            // get buffer
            let mut length = 0;
            let buffer = self.rx_dma.get().map_or(None, |rx_dma| {
                length = self.rx_len.get() - rx_dma.transfer_counter();
                let buf = rx_dma.abort_xfer();
                rx_dma.disable();
                buf
            });
            self.rx_len.set(0);

            // alert client
            self.client.get().map(|usartclient| {
                buffer.map(|buf| match usartclient {
                    UsartClient::Uart(client) => {
                        client.receive_complete(buf, length, error);
                    }
                    UsartClient::SpiMaster(_) => {}
                });
            });
        }
    }

    pub fn abort_tx(&self, error: hil::uart::Error) {
        if self.usart_tx_state.get() == USARTStateTX::DMA_Transmitting {
            self.disable_tx_interrupts();
            self.disable_tx();
            self.usart_tx_state.set(USARTStateTX::Idle);

            // get buffer
            let mut length = 0;
            let buffer = self.tx_dma.get().map_or(None, |tx_dma| {
                length = self.tx_len.get() - tx_dma.transfer_counter();
                let buf = tx_dma.abort_xfer();
                tx_dma.disable();
                buf
            });
            self.tx_len.set(0);

            // alert client
            self.client.get().map(|usartclient| {
                buffer.map(|buf| match usartclient {
                    UsartClient::Uart(client) => {
                        client.receive_complete(buf, length, error);
                    }
                    UsartClient::SpiMaster(_) => {}
                });
            });
        }
    }

    pub fn enable_tx_empty_interrupt(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.ier.write(Interrupt::TXEMPTY::SET);
    }

    pub fn disable_tx_empty_interrupt(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.idr.write(Interrupt::TXEMPTY::SET);
    }

    pub fn enable_rx_error_interrupts(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.ier
            .write(Interrupt::PARE::SET + Interrupt::FRAME::SET + Interrupt::OVRE::SET);
    }

    pub fn disable_rx_interrupts(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.idr.write(
            Interrupt::RXBUFF::SET + Interrupt::TIMEOUT::SET + Interrupt::PARE::SET
                + Interrupt::FRAME::SET + Interrupt::OVRE::SET + Interrupt::RXRDY::SET,
        );
    }

    pub fn disable_tx_interrupts(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.idr
            .write(Interrupt::TXEMPTY::SET + Interrupt::TXRDY::SET);
    }

    pub fn disable_interrupts(&self) {
        self.disable_rx_interrupts();
        self.disable_tx_interrupts();
    }

    pub fn reset(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };

        regs.cr
            .write(Control::RSTSTA::SET + Control::RSTTX::SET + Control::RSTRX::SET);

        self.abort_rx(hil::uart::Error::ResetError);
        self.enable_clock(); // in case abort_rx turned them off
        self.abort_tx(hil::uart::Error::ResetError);
    }

    pub fn handle_interrupt(&self) {
        // only handle interrupts if the clock is enabled for this peripheral.
        // Now, why are we occasionally getting interrupts with the clock
        // disabled? That is a good question that I don't have the answer to.
        // They don't even seem to be causing a problem, but seemed bad, so I
        // stopped it from occurring just in case it caused issues in the
        // future.
        if self.is_clock_enabled() {
            let regs: &UsartRegisters = unsafe { &*self.registers };

            if regs.csr.is_set(ChannelStatus::TIMEOUT) && regs.imr.is_set(Interrupt::TIMEOUT) {
                // Reset status registers. We need to do this first because some
                // interrupts signal us to turn off our clock.
                regs.cr.write(Control::RSTSTA::SET);
                self.disable_rx_timeout();
                self.abort_rx(hil::uart::Error::CommandComplete);
            } else if regs.csr.is_set(ChannelStatus::TXEMPTY) && regs.imr.is_set(Interrupt::TXEMPTY)
            {
                regs.cr.write(Control::RSTSTA::SET);
                self.disable_tx_empty_interrupt();
                self.disable_tx();
                self.usart_tx_state.set(USARTStateTX::Idle);
            } else if regs.csr.is_set(ChannelStatus::PARE) {
                regs.cr.write(Control::RSTSTA::SET);
                self.abort_rx(hil::uart::Error::ParityError);
            } else if regs.csr.is_set(ChannelStatus::FRAME) {
                regs.cr.write(Control::RSTSTA::SET);
                self.abort_rx(hil::uart::Error::FramingError);
            } else if regs.csr.is_set(ChannelStatus::OVRE) {
                regs.cr.write(Control::RSTSTA::SET);
                self.abort_rx(hil::uart::Error::OverrunError);
            }
        }
    }

    fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(self.clock);
        }
    }

    fn disable_clock(&self) {
        unsafe {
            pm::disable_clock(self.clock);
        }
    }

    fn is_clock_enabled(&self) -> bool {
        unsafe { pm::is_clock_enabled(self.clock) }
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs: &UsartRegisters = unsafe { &*self.registers };

        let system_frequency = pm::get_system_frequency();

        // The clock divisor is calculated differently in UART and SPI modes.
        let cd = match self.usart_mode.get() {
            UsartMode::Uart => system_frequency / (8 * baud_rate),
            UsartMode::Spi => system_frequency / baud_rate,
            _ => 0,
        };

        regs.brgr.write(BaudRate::CD.val(cd));
    }

    /// In non-SPI mode, this drives RTS low.
    /// In SPI mode, this asserts (drives low) the chip select line.
    fn rts_enable_spi_assert_cs(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.cr.write(Control::RTSEN::SET);
    }

    /// In non-SPI mode, this drives RTS high.
    /// In SPI mode, this de-asserts (drives high) the chip select line.
    fn rts_disable_spi_deassert_cs(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.cr.write(Control::RTSDIS::SET);
    }

    fn enable_rx_timeout(&self, timeout: u8) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.rtor.write(RxTimeout::TO.val(timeout as u32));

        // enable timeout interrupt
        regs.ier.write(Interrupt::TIMEOUT::SET);

        // start timeout
        regs.cr.write(Control::STTTO::SET);
    }

    fn disable_rx_timeout(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.rtor.write(RxTimeout::TO.val(0));

        // enable timeout interrupt
        regs.idr.write(Interrupt::TIMEOUT::SET);
    }

    fn enable_rx_terminator(&self, _terminator: u8) {
        // XXX: implement me
        panic!("didn't write terminator stuff yet");
    }

    // for use by panic in io.rs
    pub fn send_byte(&self, byte: u8) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.thr.write(TransmitHold::TXCHR.val(byte as u32));
    }

    // for use by panic in io.rs
    pub fn tx_ready(&self) -> bool {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.csr.is_set(ChannelStatus::TXRDY)
    }
}

impl dma::DMAClient for USART {
    fn xfer_done(&self, pid: dma::DMAPeripheral) {
        match self.usart_mode.get() {
            UsartMode::Uart => {
                // determine if it was an RX or TX transfer
                if pid == self.rx_dma_peripheral {
                    // RX transfer was completed

                    // disable RX and RX interrupts
                    self.disable_rx_interrupts();
                    self.disable_rx();
                    self.usart_rx_state.set(USARTStateRX::Idle);

                    // get buffer
                    let buffer = self.rx_dma.get().map_or(None, |rx_dma| {
                        let buf = rx_dma.abort_xfer();
                        rx_dma.disable();
                        buf
                    });

                    // alert client
                    self.client.get().map(|usartclient| {
                        buffer.map(|buf| {
                            let length = self.rx_len.get();
                            match usartclient {
                                UsartClient::Uart(client) => {
                                    client.receive_complete(
                                        buf,
                                        length,
                                        hil::uart::Error::CommandComplete,
                                    );
                                }
                                UsartClient::SpiMaster(_) => {}
                            }
                        });
                    });
                    self.rx_len.set(0);
                } else if pid == self.tx_dma_peripheral {
                    // TX transfer was completed

                    // note that the DMA has finished but TX cannot yet be disabled yet because
                    // there may still be bytes left in the TX buffer.
                    self.usart_tx_state.set(USARTStateTX::Transfer_Completing);
                    self.enable_tx_empty_interrupt();

                    // get buffer
                    let buffer = self.tx_dma.get().map_or(None, |tx_dma| {
                        let buf = tx_dma.abort_xfer();
                        tx_dma.disable();
                        buf
                    });

                    // alert client
                    self.client.get().map(|usartclient| {
                        buffer.map(|buf| match usartclient {
                            UsartClient::Uart(client) => {
                                client.transmit_complete(buf, hil::uart::Error::CommandComplete);
                            }
                            UsartClient::SpiMaster(_) => {}
                        });
                    });
                    self.tx_len.set(0);
                }
            }

            UsartMode::Spi => {
                if (self.usart_rx_state.get() == USARTStateRX::Idle
                    && pid == self.tx_dma_peripheral)
                    || pid == self.rx_dma_peripheral
                {
                    // SPI transfer was completed

                    self.spi_chip_select.get().map_or_else(
                        || {
                            // Do "else" case first. Thanks, rust.
                            self.rts_disable_spi_deassert_cs();
                        },
                        |cs| {
                            cs.set();
                        },
                    );

                    // note that the DMA has finished but TX cannot be disabled yet
                    self.usart_tx_state.set(USARTStateTX::Transfer_Completing);
                    self.enable_tx_empty_interrupt();

                    self.usart_rx_state.set(USARTStateRX::Idle);
                    self.disable_rx();

                    // get buffer
                    let txbuf = self.tx_dma.get().map_or(None, |dma| {
                        let buf = dma.abort_xfer();
                        dma.disable();
                        buf
                    });

                    let rxbuf = self.rx_dma.get().map_or(None, |dma| {
                        let buf = dma.abort_xfer();
                        dma.disable();
                        buf
                    });

                    let len = self.tx_len.get();

                    // alert client
                    self.client.get().map(|usartclient| {
                        txbuf.map(|tbuf| match usartclient {
                            UsartClient::Uart(_) => {}
                            UsartClient::SpiMaster(client) => {
                                client.read_write_done(tbuf, rxbuf, len);
                            }
                        });
                    });
                    self.tx_len.set(0);
                }
            }

            _ => {}
        }
    }
}

/// Implementation of kernel::hil::UART
impl hil::uart::UART for USART {
    fn set_client(&self, client: &'static hil::uart::Client) {
        let c = UsartClient::Uart(client);
        self.client.set(Some(c));
    }

    fn init(&self, params: hil::uart::UARTParams) {
        self.usart_mode.set(UsartMode::Uart);

        // enable USART clock
        //  must do this before writing any registers
        self.enable_clock();

        // disable interrupts
        self.disable_interrupts();

        // stop any TX and RX and clear status
        self.reset();
        self.enable_clock();

        // set USART mode register
        let mut mode = Mode::OVER::SET; // OVER: oversample at 8x

        mode += Mode::CHRL::BITS8; // CHRL: 8-bit characters
        mode += Mode::USCLKS::CLK_USART; // USCLKS: select CLK_USART

        mode += match params.stop_bits {
            hil::uart::StopBits::One => Mode::NBSTOP::BITS_1_1,
            hil::uart::StopBits::Two => Mode::NBSTOP::BITS_2_2,
        };

        mode += match params.parity {
            hil::uart::Parity::None => Mode::PAR::NONE, // no parity
            hil::uart::Parity::Odd => Mode::PAR::ODD,   // odd parity
            hil::uart::Parity::Even => Mode::PAR::EVEN, // even parity
        };

        mode += match params.hw_flow_control {
            true => Mode::MODE::HARD_HAND,
            false => Mode::MODE::NORMAL,
        };

        let regs: &UsartRegisters = unsafe { &*self.registers };
        regs.mr.write(mode);

        // Set baud rate
        self.set_baud_rate(params.baud_rate);

        self.disable_clock();
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        // enable USART clock
        //  must do this before writing any registers
        self.enable_clock();

        // quit current transmission if any
        self.abort_tx(hil::uart::Error::RepeatCallError);

        // enable TX
        self.enable_tx();
        self.usart_tx_state.set(USARTStateTX::DMA_Transmitting);

        // set up dma transfer and start transmission
        self.tx_dma.get().map(move |dma| {
            dma.enable();
            dma.do_xfer(self.tx_dma_peripheral, tx_data, tx_len);
            self.tx_len.set(tx_len);
        });
    }

    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        // enable USART clock
        //  must do this before writing any registers
        self.enable_clock();

        // quit current reception if any
        self.abort_rx(hil::uart::Error::RepeatCallError);

        // truncate rx_len if necessary
        let mut length = rx_len;
        if rx_len > rx_buffer.len() {
            length = rx_buffer.len();
        }

        // enable RX
        self.enable_rx();
        self.enable_rx_error_interrupts();
        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);

        // set up dma transfer and start reception
        self.rx_dma.get().map(move |dma| {
            dma.enable();
            dma.do_xfer(self.rx_dma_peripheral, rx_buffer, length);
            self.rx_len.set(rx_len);
        });
    }
}

impl hil::uart::UARTAdvanced for USART {
    fn receive_automatic(&self, rx_buffer: &'static mut [u8], interbyte_timeout: u8) {
        // enable USART clock
        //  must do this before writing any registers
        self.enable_clock();

        // quit current reception if any
        self.abort_rx(hil::uart::Error::RepeatCallError);

        // enable RX
        self.enable_rx();
        self.enable_rx_error_interrupts();
        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);

        // enable receive timeout
        self.enable_rx_timeout(interbyte_timeout);

        // set up dma transfer and start reception
        self.rx_dma.get().map(move |dma| {
            dma.enable();
            let length = rx_buffer.len();
            dma.do_xfer(self.rx_dma_peripheral, rx_buffer, length);
            self.rx_len.set(length);
        });
    }

    fn receive_until_terminator(&self, rx_buffer: &'static mut [u8], terminator: u8) {
        // enable USART clock
        //  must do this before writing any registers
        self.enable_clock();

        // quit current reception if any
        self.abort_rx(hil::uart::Error::RepeatCallError);

        // enable RX
        self.enable_rx();
        self.enable_rx_error_interrupts();
        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);

        // enable receive terminator
        self.enable_rx_terminator(terminator);

        // set up dma transfer and start reception
        self.rx_dma.get().map(move |dma| {
            dma.enable();
            let length = rx_buffer.len();
            dma.do_xfer(self.rx_dma_peripheral, rx_buffer, length);
            self.rx_len.set(length);
        });
    }
}

/// SPI
impl hil::spi::SpiMaster for USART {
    type ChipSelect = Option<&'static hil::gpio::Pin>;

    fn init(&self) {
        let regs: &UsartRegisters = unsafe { &*self.registers };

        self.usart_mode.set(UsartMode::Spi);
        self.enable_clock();

        // Set baud rate, default to 2 MHz.
        self.set_baud_rate(2000000);

        regs.mr.write(
            Mode::MODE::SPI_MASTER + Mode::USCLKS::CLK_USART + Mode::CHRL::BITS8 + Mode::PAR::NONE
                + Mode::CLKO::SET,
        );

        // Set four bit periods of guard time before RTS/CTS toggle after a
        // message.
        regs.ttgr.write(TxTimeGuard::TG.val(4));

        self.disable_clock();
    }

    fn set_client(&self, client: &'static hil::spi::SpiMasterClient) {
        let c = UsartClient::SpiMaster(client);
        self.client.set(Some(c));
    }

    fn is_busy(&self) -> bool {
        return false;
    }

    fn read_write_bytes(
        &self,
        mut write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> ReturnCode {
        self.enable_tx();
        self.enable_rx();

        // Calculate the correct length for the transmission
        let buflen = read_buffer.as_ref().map_or(write_buffer.len(), |rbuf| {
            cmp::min(rbuf.len(), write_buffer.len())
        });
        let count = cmp::min(buflen, len);

        self.tx_len.set(count);

        // Set !CS low
        self.spi_chip_select.get().map_or_else(
            || {
                // Do the "else" case first. If a CS pin was provided as the
                // CS line, we use the HW RTS pin as the CS line instead.
                self.rts_enable_spi_assert_cs();
            },
            |cs| {
                cs.clear();
            },
        );

        // Check if we should read and write or just write.
        if read_buffer.is_some() {
            // We are reading and writing.
            read_buffer.map(|rbuf| {
                self.tx_dma.get().map(move |dma| {
                    self.rx_dma.get().map(move |read| {
                        // Do all the maps before starting anything in case
                        // they take too much time.

                        // Start the write transaction.
                        self.usart_tx_state.set(USARTStateTX::DMA_Transmitting);
                        self.usart_rx_state.set(USARTStateRX::Idle);
                        dma.enable();
                        dma.do_xfer(self.tx_dma_peripheral, write_buffer, count);

                        // Start the read transaction.
                        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);
                        read.enable();
                        read.do_xfer(self.rx_dma_peripheral, rbuf, count);
                    });
                });
            });
        } else {
            // We are just writing.
            self.tx_dma.get().map(move |dma| {
                self.usart_tx_state.set(USARTStateTX::DMA_Transmitting);
                self.usart_rx_state.set(USARTStateRX::Idle);
                dma.enable();
                dma.do_xfer(self.tx_dma_peripheral, write_buffer, count);
            });
        }

        ReturnCode::SUCCESS
    }

    fn write_byte(&self, val: u8) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        self.enable_clock();
        regs.cr.write(Control::RXEN::SET + Control::TXEN::SET);
        regs.thr.write(TransmitHold::TXCHR.val(val as u32));
    }

    fn read_byte(&self) -> u8 {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        self.enable_clock();
        regs.rhr.read(ReceiverHold::RXCHR) as u8
    }

    fn read_write_byte(&self, val: u8) -> u8 {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        self.enable_clock();
        regs.cr.write(Control::RXEN::SET + Control::TXEN::SET);

        regs.thr.write(TransmitHold::TXCHR.val(val as u32));
        while !regs.csr.is_set(ChannelStatus::RXRDY) {}
        regs.rhr.read(ReceiverHold::RXCHR) as u8
    }

    /// Pass in a None to use the HW chip select pin on the USART (RTS).
    fn specify_chip_select(&self, cs: Self::ChipSelect) {
        self.spi_chip_select.set(cs);
    }

    /// Returns the actual rate set
    fn set_rate(&self, rate: u32) -> u32 {
        self.enable_clock();
        self.set_baud_rate(rate);

        // Calculate what rate will actually be
        let system_frequency = pm::get_system_frequency();
        let cd = system_frequency / rate;
        system_frequency / cd
    }

    fn get_rate(&self) -> u32 {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        self.enable_clock();
        let system_frequency = pm::get_system_frequency();
        let cd = regs.brgr.read(BaudRate::CD);
        system_frequency / cd
    }

    fn set_clock(&self, polarity: hil::spi::ClockPolarity) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        self.enable_clock();
        // Note that in SPI mode MSBF bit is clock polarity (CPOL)
        match polarity {
            hil::spi::ClockPolarity::IdleLow => {
                regs.mr.modify(Mode::MSBF::CLEAR);
            }
            hil::spi::ClockPolarity::IdleHigh => {
                regs.mr.modify(Mode::MSBF::SET);
            }
        }
    }

    fn get_clock(&self) -> hil::spi::ClockPolarity {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        self.enable_clock();

        // Note that in SPI mode MSBF bit is clock polarity (CPOL)
        let idle = regs.mr.read(Mode::MSBF);
        match idle {
            0 => hil::spi::ClockPolarity::IdleLow,
            _ => hil::spi::ClockPolarity::IdleHigh,
        }
    }

    fn set_phase(&self, phase: hil::spi::ClockPhase) {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        self.enable_clock();

        // Note that in SPI mode SYNC bit is clock phase
        match phase {
            hil::spi::ClockPhase::SampleLeading => {
                regs.mr.modify(Mode::SYNC::SET);
            }
            hil::spi::ClockPhase::SampleTrailing => {
                regs.mr.modify(Mode::SYNC::CLEAR);
            }
        }
    }

    fn get_phase(&self) -> hil::spi::ClockPhase {
        let regs: &UsartRegisters = unsafe { &*self.registers };
        self.enable_clock();
        let phase = regs.mr.read(Mode::SYNC);

        // Note that in SPI mode SYNC bit is clock phase
        match phase {
            0 => hil::spi::ClockPhase::SampleLeading,
            _ => hil::spi::ClockPhase::SampleTrailing,
        }
    }

    // These two functions determine what happens to the chip
    // select line between transfers. If hold_low() is called,
    // then the chip select line is held low after transfers
    // complete. If release_low() is called, then the chip select
    // line is brought high after a transfer completes. A "transfer"
    // is any of the read/read_write calls. These functions
    // allow an application to manually control when the
    // CS line is high or low, such that it can issue multi-byte
    // requests with single byte operations.
    fn hold_low(&self) {
        unimplemented!("USART: SPI: Use `read_write_bytes()` instead.");
    }

    fn release_low(&self) {
        unimplemented!("USART: SPI: Use `read_write_bytes()` instead.");
    }
}
