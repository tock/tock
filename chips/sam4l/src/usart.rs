// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of the SAM4L USART peripheral.
//!
//! Supports UART and SPI master modes.

use core::cell::Cell;
use core::cmp;
use core::num::NonZero;
use core::sync::atomic::{AtomicBool, Ordering};
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::spi;
use kernel::hil::spi::cs::ChipSelectPolar;
use kernel::hil::uart;
use kernel::non_zero;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::dma;
use crate::pm;

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

const USART_BASE_ADDRS: [StaticRef<UsartRegisters>; 4] = unsafe {
    [
        StaticRef::new(0x40024000 as *const UsartRegisters),
        StaticRef::new(0x40028000 as *const UsartRegisters),
        StaticRef::new(0x4002C000 as *const UsartRegisters),
        StaticRef::new(0x40030000 as *const UsartRegisters),
    ]
};

pub struct USARTRegManager<'a> {
    registers: &'a UsartRegisters,
    clock: pm::Clock,
    rx_dma: Option<&'a dma::DMAChannel>,
    tx_dma: Option<&'a dma::DMAChannel>,
}

static IS_PANICING: AtomicBool = AtomicBool::new(false);

impl<'a> USARTRegManager<'a> {
    fn real_new(usart: &'a USART) -> USARTRegManager<'a> {
        if !pm::is_clock_enabled(usart.clock) {
            pm::enable_clock(usart.clock);
        }
        let regs: &UsartRegisters = &usart.registers;
        USARTRegManager {
            registers: regs,
            clock: usart.clock,
            rx_dma: usart.rx_dma.get(),
            tx_dma: usart.tx_dma.get(),
        }
    }

    fn new(usart: &'a USART) -> USARTRegManager<'a> {
        USARTRegManager::real_new(usart)
    }

    pub fn panic_new(usart: &'a USART) -> USARTRegManager<'a> {
        IS_PANICING.store(true, Ordering::Relaxed);
        USARTRegManager::real_new(usart)
    }
}

impl Drop for USARTRegManager<'_> {
    fn drop(&mut self) {
        // Anything listening for RX or TX interrupts?
        let ints_active = self.registers.imr.any_matching_bits_set(
            Interrupt::RXBUFF::SET
                + Interrupt::TXEMPTY::SET
                + Interrupt::TIMEOUT::SET
                + Interrupt::PARE::SET
                + Interrupt::FRAME::SET
                + Interrupt::OVRE::SET
                + Interrupt::TXRDY::SET
                + Interrupt::RXRDY::SET,
        );

        let rx_active = self.rx_dma.is_some_and(|rx_dma| rx_dma.is_enabled());
        let tx_active = self.tx_dma.is_some_and(|tx_dma| tx_dma.is_enabled());

        // Special-case panic here as panic does not actually use the
        // USART driver code in this file, rather it writes the registers
        // directly and we can't safely reason about what the custom panic
        // USART driver is doing / expects.
        let is_panic = IS_PANICING.load(Ordering::Relaxed);
        if !(rx_active || tx_active || ints_active || is_panic) {
            pm::disable_clock(self.clock);
        }
    }
}

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

#[derive(Copy, Clone, PartialEq)]
pub enum UsartMode {
    Uart,
    Spi,
    Unused,
}

#[derive(Copy, Clone)]
enum UsartClient<'a> {
    Uart(
        Option<&'a dyn uart::ReceiveClient>,
        Option<&'a dyn uart::TransmitClient>,
    ),
    SpiMaster(&'a dyn spi::SpiMasterClient),
}

// State that needs to be stored for deferred calls
struct DeferredCallState {
    abort_rx_buf: Option<&'static mut [u8]>,
    abort_rx_len: usize,
    abort_rx_rcode: Result<(), ErrorCode>,
    abort_rx_error: uart::Error,
}

pub struct USART<'a> {
    registers: StaticRef<UsartRegisters>,
    clock: pm::Clock,

    usart_mode: Cell<UsartMode>,

    usart_tx_state: Cell<USARTStateTX>,
    usart_rx_state: Cell<USARTStateRX>,

    rx_dma: Cell<Option<&'a dma::DMAChannel>>,
    rx_dma_peripheral: dma::DMAPeripheral,
    rx_len: Cell<usize>,
    tx_dma: Cell<Option<&'a dma::DMAChannel>>,
    tx_dma_peripheral: dma::DMAPeripheral,
    tx_len: Cell<usize>,

    client: OptionalCell<UsartClient<'a>>,

    spi_chip_select: OptionalCell<ChipSelectPolar<'a, crate::gpio::GPIOPin<'a>>>,
    pm: &'a pm::PowerManager,
    dc_state: OptionalCell<DeferredCallState>,
    deferred_call: DeferredCall,
}

impl<'a> USART<'a> {
    fn new(
        base_addr: StaticRef<UsartRegisters>,
        clock: pm::PBAClock,
        rx_dma_peripheral: dma::DMAPeripheral,
        tx_dma_peripheral: dma::DMAPeripheral,
        pm: &'a pm::PowerManager,
    ) -> Self {
        Self {
            registers: base_addr,
            clock: pm::Clock::PBA(clock),

            usart_mode: Cell::new(UsartMode::Unused),

            usart_rx_state: Cell::new(USARTStateRX::Idle),
            usart_tx_state: Cell::new(USARTStateTX::Idle),

            // these get defined later by `chip.rs`
            rx_dma: Cell::new(None),
            rx_dma_peripheral,
            rx_len: Cell::new(0),
            tx_dma: Cell::new(None),
            tx_dma_peripheral,
            tx_len: Cell::new(0),

            // this gets defined later by `main.rs`
            client: OptionalCell::empty(),

            // This is only used if the USART is in SPI mode.
            spi_chip_select: OptionalCell::empty(),
            pm,
            dc_state: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
        }
    }

    pub fn new_usart0(pm: &'a pm::PowerManager) -> Self {
        USART::new(
            USART_BASE_ADDRS[0],
            pm::PBAClock::USART0,
            dma::DMAPeripheral::USART0_RX,
            dma::DMAPeripheral::USART0_TX,
            pm,
        )
    }

    pub fn new_usart1(pm: &'a pm::PowerManager) -> Self {
        USART::new(
            USART_BASE_ADDRS[1],
            pm::PBAClock::USART1,
            dma::DMAPeripheral::USART1_RX,
            dma::DMAPeripheral::USART1_TX,
            pm,
        )
    }

    pub fn new_usart2(pm: &'a pm::PowerManager) -> Self {
        USART::new(
            USART_BASE_ADDRS[2],
            pm::PBAClock::USART2,
            dma::DMAPeripheral::USART2_RX,
            dma::DMAPeripheral::USART2_TX,
            pm,
        )
    }

    pub fn new_usart3(pm: &'a pm::PowerManager) -> Self {
        USART::new(
            USART_BASE_ADDRS[3],
            pm::PBAClock::USART3,
            dma::DMAPeripheral::USART3_RX,
            dma::DMAPeripheral::USART3_TX,
            pm,
        )
    }

    pub fn set_dma(&self, rx_dma: &'a dma::DMAChannel, tx_dma: &'a dma::DMAChannel) {
        self.rx_dma.set(Some(rx_dma));
        self.tx_dma.set(Some(tx_dma));
    }

    pub fn set_mode(&self, mode: UsartMode) {
        if self.usart_mode.get() != UsartMode::Unused {
            // n.b. This may actually "just work", particularly if we reset the
            // whole peripheral here. But we really should check other
            // conditions, such as whether there's an outstanding transaction
            // in progress (will a USART reset cancel the DMA? will we get an
            // unexpected interrupt?), before letting this happen.
            unimplemented!("Dynamically changing USART mode");
        }

        self.usart_mode.set(mode);

        let usart = &USARTRegManager::new(self);

        // disable interrupts
        self.disable_interrupts(usart);

        // stop any TX and RX and clear status
        self.reset(usart);
    }

    fn enable_rx(&self, usart: &USARTRegManager) {
        usart.registers.cr.write(Control::RXEN::SET);
    }

    pub fn enable_tx(&self, usart: &USARTRegManager) {
        usart.registers.cr.write(Control::TXEN::SET);
    }

    fn disable_rx(&self, usart: &USARTRegManager) {
        usart.registers.cr.write(Control::RXDIS::SET);
        self.usart_rx_state.set(USARTStateRX::Idle);
    }

    fn disable_tx(&self, usart: &USARTRegManager) {
        usart.registers.cr.write(Control::TXDIS::SET);
        self.usart_tx_state.set(USARTStateTX::Idle);
    }

    fn abort_rx(&self, usart: &USARTRegManager, rcode: Result<(), ErrorCode>, error: uart::Error) {
        if self.usart_rx_state.get() == USARTStateRX::DMA_Receiving {
            self.disable_rx_interrupts(usart);
            self.disable_rx(usart);
            self.usart_rx_state.set(USARTStateRX::Idle);

            // get buffer
            let mut length = 0;
            let mut buffer = self.rx_dma.get().and_then(|rx_dma| {
                length = self.rx_len.get() - rx_dma.transfer_counter();
                let buf = rx_dma.abort_transfer();
                rx_dma.disable();
                buf
            });
            self.rx_len.set(0);
            // Save state for deferred call
            let dc_state = DeferredCallState {
                abort_rx_buf: buffer.take(),
                abort_rx_len: length,
                abort_rx_rcode: rcode,
                abort_rx_error: error,
            };
            self.dc_state.set(dc_state);

            // schedule a deferred call to alert the client of this particular UART
            self.deferred_call.set();
        }
    }

    fn abort_tx(&self, usart: &USARTRegManager, rcode: Result<(), ErrorCode>) {
        if self.usart_tx_state.get() == USARTStateTX::DMA_Transmitting {
            self.disable_tx_interrupts(usart);
            self.disable_tx(usart);
            self.usart_tx_state.set(USARTStateTX::Idle);

            // get buffer
            let mut length = 0;
            let mut buffer = self.tx_dma.get().and_then(|tx_dma| {
                length = self.tx_len.get() - tx_dma.transfer_counter();
                let buf = tx_dma.abort_transfer();
                tx_dma.disable();
                buf
            });
            self.tx_len.set(0);

            // alert client
            self.client.map(|usartclient| {
                if let UsartClient::Uart(_rx, Some(tx)) = usartclient {
                    buffer
                        .take()
                        .map(|buf| tx.transmitted_buffer(buf, length, rcode));
                }
            });
        }
    }

    fn enable_tx_empty_interrupt(&self, usart: &USARTRegManager) {
        usart.registers.ier.write(Interrupt::TXEMPTY::SET);
    }

    fn disable_tx_empty_interrupt(&self, usart: &USARTRegManager) {
        usart.registers.idr.write(Interrupt::TXEMPTY::SET);
    }

    fn enable_rx_error_interrupts(&self, usart: &USARTRegManager) {
        usart
            .registers
            .ier
            .write(Interrupt::PARE::SET + Interrupt::FRAME::SET + Interrupt::OVRE::SET);
    }

    fn disable_rx_interrupts(&self, usart: &USARTRegManager) {
        usart.registers.idr.write(
            Interrupt::RXBUFF::SET
                + Interrupt::TIMEOUT::SET
                + Interrupt::PARE::SET
                + Interrupt::FRAME::SET
                + Interrupt::OVRE::SET
                + Interrupt::RXRDY::SET,
        );
    }

    fn disable_tx_interrupts(&self, usart: &USARTRegManager) {
        usart
            .registers
            .idr
            .write(Interrupt::TXEMPTY::SET + Interrupt::TXRDY::SET);
    }

    fn disable_interrupts(&self, usart: &USARTRegManager) {
        self.disable_rx_interrupts(usart);
        self.disable_tx_interrupts(usart);
    }

    fn reset(&self, usart: &USARTRegManager) {
        usart
            .registers
            .cr
            .write(Control::RSTSTA::SET + Control::RSTTX::SET + Control::RSTRX::SET);

        self.abort_rx(usart, Err(ErrorCode::FAIL), uart::Error::ResetError);
        self.abort_tx(usart, Err(ErrorCode::FAIL));
    }

    pub fn handle_interrupt(&self) {
        let usart = &USARTRegManager::new(self);

        let status = usart.registers.csr.extract();
        let mask = usart.registers.imr.extract();

        if status.is_set(ChannelStatus::TIMEOUT) && mask.is_set(Interrupt::TIMEOUT) {
            self.disable_rx_timeout(usart);
            self.abort_rx(usart, Ok(()), uart::Error::Aborted);
        } else if status.is_set(ChannelStatus::TXEMPTY) && mask.is_set(Interrupt::TXEMPTY) {
            self.disable_tx_empty_interrupt(usart);
            self.disable_tx(usart);
            self.usart_tx_state.set(USARTStateTX::Idle);

            // Now that we know the TX transaction is finished we can get the
            // buffer back from DMA and pass it back to the client. If we don't
            // wait until we are completely finished, then the
            // `transmit_complete` callback is in a "bad" part of the USART
            // state machine, and clients cannot issue other USART calls from
            // the callback.
            let txbuffer = self.tx_dma.get().map_or(None, |tx_dma| {
                let buf = tx_dma.abort_transfer();
                tx_dma.disable();
                buf
            });

            // alert client
            self.client.map(|usartclient| {
                match usartclient {
                    UsartClient::Uart(_rx, tx) => {
                        if txbuffer.is_some() {
                            let len = self.tx_len.get();
                            self.tx_len.set(0);
                            tx.map(|client| {
                                client.transmitted_buffer(txbuffer.unwrap(), len, Ok(()))
                            });
                        }
                    }
                    UsartClient::SpiMaster(client) => {
                        // For the SPI case it is a little more complicated.

                        // First, it is now a valid time to de-assert the CS
                        // line because we know the write and/or read is done.
                        self.spi_chip_select.map_or_else(
                            || {
                                // Do "else" case first. Thanks, rust.
                                self.rts_disable_spi_deassert_cs(usart);
                            },
                            |cs| {
                                cs.deactivate();
                            },
                        );

                        // Get the RX buffer, and it is ok if we didn't use one,
                        // we can just return None.
                        let rxbuf = self.rx_dma.get().map_or(None, |dma| {
                            let buf = dma.abort_transfer();
                            dma.disable();
                            buf
                        });

                        // And now it is safe to notify the client because TX is
                        // in its Idle state rather than its transfer completing
                        // state.
                        let len = self.tx_len.get();
                        self.tx_len.set(0);
                        client.read_write_done(
                            txbuffer.unwrap().into(),
                            rxbuf.map(|b| b.into()),
                            Ok(len),
                        );
                    }
                }
            });
        } else if status.is_set(ChannelStatus::PARE) {
            self.abort_rx(usart, Err(ErrorCode::FAIL), uart::Error::ParityError);
        } else if status.is_set(ChannelStatus::FRAME) {
            self.abort_rx(usart, Err(ErrorCode::FAIL), uart::Error::FramingError);
        } else if status.is_set(ChannelStatus::OVRE) {
            self.abort_rx(usart, Err(ErrorCode::FAIL), uart::Error::OverrunError);
        }

        // Reset status registers.
        usart.registers.cr.write(Control::RSTSTA::SET);
    }

    fn set_baud_rate(&self, usart: &USARTRegManager, baud_rate: NonZero<u32>) {
        let baud_rate = baud_rate.get();
        let system_frequency = self.pm.get_system_frequency();

        // The clock divisor is calculated differently in UART and SPI modes.
        match self.usart_mode.get() {
            UsartMode::Uart => {
                let uart_baud_rate = 8 * baud_rate;
                let cd = system_frequency / uart_baud_rate;
                //Generate fractional part
                let fp = (system_frequency + baud_rate / 2) / baud_rate - 8 * cd;
                usart
                    .registers
                    .brgr
                    .write(BaudRate::FP.val(fp) + BaudRate::CD.val(cd));
            }
            UsartMode::Spi => {
                let cd = system_frequency / baud_rate;
                usart.registers.brgr.write(BaudRate::CD.val(cd));
            }
            UsartMode::Unused => {}
        }
    }

    /// In non-SPI mode, this drives RTS low.
    /// In SPI mode, this asserts (drives low) the chip select line.
    fn rts_enable_spi_assert_cs(&self, usart: &USARTRegManager) {
        usart.registers.cr.write(Control::RTSEN::SET);
    }

    /// In non-SPI mode, this drives RTS high.
    /// In SPI mode, this de-asserts (drives high) the chip select line.
    fn rts_disable_spi_deassert_cs(&self, usart: &USARTRegManager) {
        usart.registers.cr.write(Control::RTSDIS::SET);
    }

    fn enable_rx_timeout(&self, usart: &USARTRegManager, timeout: u8) {
        usart
            .registers
            .rtor
            .write(RxTimeout::TO.val(timeout as u32));

        // Start the timeout, and we must do this before enabling the interrupt.
        // This ordering ensures that the interrupt does not fire prematurely.
        usart.registers.cr.write(Control::STTTO::SET);

        // enable timeout interrupt
        usart.registers.ier.write(Interrupt::TIMEOUT::SET);
    }

    fn disable_rx_timeout(&self, usart: &USARTRegManager) {
        usart.registers.rtor.write(RxTimeout::TO.val(0));

        // enable timeout interrupt
        usart.registers.idr.write(Interrupt::TIMEOUT::SET);
    }

    // for use by panic in io.rs
    pub fn send_byte(&self, usart: &USARTRegManager, byte: u8) {
        usart
            .registers
            .thr
            .write(TransmitHold::TXCHR.val(byte as u32));
    }

    // for use by panic in io.rs
    pub fn tx_ready(&self, usart: &USARTRegManager) -> bool {
        usart.registers.csr.is_set(ChannelStatus::TXRDY)
    }
}

impl DeferredCallClient for USART<'_> {
    fn handle_deferred_call(&self) {
        self.dc_state.take().map(|mut dc_state| {
            self.client.map(|usartclient| {
                if let UsartClient::Uart(Some(rx), _tx) = usartclient {
                    dc_state.abort_rx_buf.take().map(|buf| {
                        rx.received_buffer(
                            buf,
                            dc_state.abort_rx_len,
                            dc_state.abort_rx_rcode,
                            dc_state.abort_rx_error,
                        )
                    });
                }
            });
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl dma::DMAClient for USART<'_> {
    fn transfer_done(&self, pid: dma::DMAPeripheral) {
        let usart = &USARTRegManager::new(self);
        match self.usart_mode.get() {
            UsartMode::Uart => {
                // determine if it was an RX or TX transfer
                if pid == self.rx_dma_peripheral {
                    // RX transfer was completed

                    // disable RX and RX interrupts
                    self.disable_rx_interrupts(usart);
                    self.disable_rx(usart);
                    self.usart_rx_state.set(USARTStateRX::Idle);

                    // get buffer
                    let buffer = self.rx_dma.get().and_then(|rx_dma| {
                        let buf = rx_dma.abort_transfer();
                        rx_dma.disable();
                        buf
                    });

                    // alert client
                    self.client.map(|usartclient| {
                        if let UsartClient::Uart(Some(rx), _tx) = usartclient {
                            buffer.map(|buf| {
                                let length = self.rx_len.get();
                                self.rx_len.set(0);
                                rx.received_buffer(buf, length, Ok(()), uart::Error::None);
                            });
                        }
                    });
                } else if pid == self.tx_dma_peripheral {
                    // TX transfer was completed

                    // note that the DMA has finished but TX cannot yet be disabled yet because
                    // there may still be bytes left in the TX buffer.
                    self.usart_tx_state.set(USARTStateTX::Transfer_Completing);
                    self.enable_tx_empty_interrupt(usart);
                }
            }

            UsartMode::Spi => {
                if (self.usart_rx_state.get() == USARTStateRX::Idle
                    && pid == self.tx_dma_peripheral)
                    || pid == self.rx_dma_peripheral
                {
                    // SPI transfer was completed. Either we didn't do a read,
                    // so the only event we expect is a TX DMA done, OR, we did
                    // a read so we ignore the TX DMA done event and wait for
                    // the RX DMA done event.

                    // Note that the DMA has finished but TX cannot be disabled
                    // yet.
                    self.usart_tx_state.set(USARTStateTX::Transfer_Completing);
                    self.enable_tx_empty_interrupt(usart);

                    // The RX is either already idle and disabled (we didn't
                    // do a read) or it is now safe to do this.
                    self.usart_rx_state.set(USARTStateRX::Idle);
                    self.disable_rx(usart);
                }
            }

            _ => {}
        }
    }
}

/// Implementation of kernel::uart
impl<'a> uart::Receive<'a> for USART<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        if let Some(UsartClient::Uart(_rx, Some(tx))) = self.client.take() {
            self.client.set(UsartClient::Uart(Some(client), Some(tx)));
        } else {
            self.client.set(UsartClient::Uart(Some(client), None));
        }
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }
        let usart = &USARTRegManager::new(self);

        // enable RX
        self.enable_rx(usart);
        self.enable_rx_error_interrupts(usart);
        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);
        // set up dma transfer and start reception
        if let Some(dma) = self.rx_dma.get() {
            dma.enable();
            self.rx_len.set(rx_len);
            dma.do_transfer(self.rx_dma_peripheral, rx_buffer, rx_len);
            Ok(())
        } else {
            Err((ErrorCode::OFF, rx_buffer))
        }
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        let usart = &USARTRegManager::new(self);
        self.disable_rx_timeout(usart);
        self.abort_rx(usart, Err(ErrorCode::CANCEL), uart::Error::Aborted);
        Err(ErrorCode::BUSY)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> uart::Transmit<'a> for USART<'a> {
    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.usart_tx_state.get() != USARTStateTX::Idle {
            Err((ErrorCode::BUSY, tx_buffer))
        } else {
            if tx_len > tx_buffer.len() {
                return Err((ErrorCode::SIZE, tx_buffer));
            }
            let usart = &USARTRegManager::new(self);
            // enable TX
            self.enable_tx(usart);
            self.usart_tx_state.set(USARTStateTX::DMA_Transmitting);

            // set up dma transfer and start transmission
            if self.tx_dma.get().is_some() {
                self.tx_dma.get().map(move |dma| {
                    dma.enable();
                    self.tx_len.set(tx_len);
                    dma.do_transfer(self.tx_dma_peripheral, tx_buffer, tx_len);
                });
                Ok(())
            } else {
                Err((ErrorCode::OFF, tx_buffer))
            }
        }
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if self.usart_tx_state.get() != USARTStateTX::Idle {
            let usart = &USARTRegManager::new(self);
            self.abort_tx(usart, Err(ErrorCode::CANCEL));
            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }

    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        if let Some(UsartClient::Uart(Some(rx), _tx)) = self.client.take() {
            self.client.set(UsartClient::Uart(Some(rx), Some(client)));
        } else {
            self.client.set(UsartClient::Uart(None, Some(client)));
        }
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl uart::Configure for USART<'_> {
    fn configure(&self, parameters: uart::Parameters) -> Result<(), ErrorCode> {
        if self.usart_mode.get() != UsartMode::Uart {
            return Err(ErrorCode::OFF);
        }

        let usart = &USARTRegManager::new(self);

        // set USART mode register
        let mut mode = Mode::OVER::SET; // OVER: oversample at 8x

        mode += Mode::CHRL::BITS8; // CHRL: 8-bit characters
        mode += Mode::USCLKS::CLK_USART; // USCLKS: select CLK_USART

        mode += match parameters.stop_bits {
            uart::StopBits::One => Mode::NBSTOP::BITS_1_1,
            uart::StopBits::Two => Mode::NBSTOP::BITS_2_2,
        };

        mode += match parameters.parity {
            uart::Parity::None => Mode::PAR::NONE, // no parity
            uart::Parity::Odd => Mode::PAR::ODD,   // odd parity
            uart::Parity::Even => Mode::PAR::EVEN, // even parity
        };

        mode += match parameters.hw_flow_control {
            true => Mode::MODE::HARD_HAND,
            false => Mode::MODE::NORMAL,
        };
        usart.registers.mr.write(mode);
        // Set baud rate
        self.set_baud_rate(usart, parameters.baud_rate);

        Ok(())
    }
}

impl<'a> uart::ReceiveAdvanced<'a> for USART<'a> {
    fn receive_automatic(
        &self,
        rx_buffer: &'static mut [u8],
        len: usize,
        interbyte_timeout: u8,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.usart_rx_state.get() != USARTStateRX::Idle {
            Err((ErrorCode::BUSY, rx_buffer))
        } else {
            let usart = &USARTRegManager::new(self);
            let length = cmp::min(len, rx_buffer.len());

            // enable RX
            self.enable_rx(usart);
            self.enable_rx_error_interrupts(usart);
            self.usart_rx_state.set(USARTStateRX::DMA_Receiving);

            // enable receive timeout
            self.enable_rx_timeout(usart, interbyte_timeout);

            // set up dma transfer and start reception
            self.rx_dma.get().map(move |dma| {
                dma.enable();
                dma.do_transfer(self.rx_dma_peripheral, rx_buffer, length);
                self.rx_len.set(length);
            });
            Ok(())
        }
    }
}

/// SPI
impl<'a> spi::SpiMaster<'a> for USART<'a> {
    type ChipSelect = ChipSelectPolar<'a, crate::gpio::GPIOPin<'a>>;

    fn init(&self) -> Result<(), ErrorCode> {
        let usart = &USARTRegManager::new(self);

        self.usart_mode.set(UsartMode::Spi);

        // Set baud rate, default to 2 MHz.
        self.set_baud_rate(usart, non_zero!(2_000_000));

        usart.registers.mr.write(
            Mode::MODE::SPI_MASTER
                + Mode::USCLKS::CLK_USART
                + Mode::CHRL::BITS8
                + Mode::PAR::NONE
                + Mode::CLKO::SET,
        );

        // Set four bit periods of guard time before RTS/CTS toggle after a
        // message.
        usart.registers.ttgr.write(TxTimeGuard::TG.val(4));
        Ok(())
    }

    fn set_client(&self, client: &'a dyn spi::SpiMasterClient) {
        let c = UsartClient::SpiMaster(client);
        self.client.set(c);
    }

    fn is_busy(&self) -> bool {
        false
    }

    fn read_write_bytes(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
    ) -> Result<
        (),
        (
            ErrorCode,
            SubSliceMut<'static, u8>,
            Option<SubSliceMut<'static, u8>>,
        ),
    > {
        let usart = &USARTRegManager::new(self);

        self.enable_tx(usart);
        self.enable_rx(usart);

        // Calculate the correct length for the transmission
        let count = read_buffer.as_ref().map_or(write_buffer.len(), |rbuf| {
            cmp::min(rbuf.len(), write_buffer.len())
        });

        self.tx_len.set(count);

        // Set !CS low
        self.spi_chip_select.map_or_else(
            || {
                // Do the "else" case first. If a CS pin was provided as the
                // CS line, we use the HW RTS pin as the CS line instead.
                self.rts_enable_spi_assert_cs(usart);
            },
            |cs| {
                cs.activate();
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
                        dma.do_transfer(self.tx_dma_peripheral, write_buffer.take(), count);

                        // Start the read transaction.
                        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);
                        read.enable();
                        read.do_transfer(self.rx_dma_peripheral, rbuf.take(), count);
                    });
                });
            });
        } else {
            // We are just writing.
            self.tx_dma.get().map(move |dma| {
                self.usart_tx_state.set(USARTStateTX::DMA_Transmitting);
                self.usart_rx_state.set(USARTStateRX::Idle);
                dma.enable();
                dma.do_transfer(self.tx_dma_peripheral, write_buffer.take(), count);
            });
        }

        Ok(())
    }

    fn write_byte(&self, val: u8) -> Result<(), ErrorCode> {
        let usart = &USARTRegManager::new(self);
        usart
            .registers
            .cr
            .write(Control::RXEN::SET + Control::TXEN::SET);
        usart
            .registers
            .thr
            .write(TransmitHold::TXCHR.val(val as u32));
        Ok(())
    }

    fn read_byte(&self) -> Result<u8, ErrorCode> {
        let usart = &USARTRegManager::new(self);
        Ok(usart.registers.rhr.read(ReceiverHold::RXCHR) as u8)
    }

    fn read_write_byte(&self, val: u8) -> Result<u8, ErrorCode> {
        let usart = &USARTRegManager::new(self);
        usart
            .registers
            .cr
            .write(Control::RXEN::SET + Control::TXEN::SET);

        usart
            .registers
            .thr
            .write(TransmitHold::TXCHR.val(val as u32));
        while !usart.registers.csr.is_set(ChannelStatus::RXRDY) {}
        Ok(usart.registers.rhr.read(ReceiverHold::RXCHR) as u8)
    }

    /// Pass in a None to use the HW chip select pin on the USART (RTS).
    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        self.spi_chip_select.set(cs);
        Ok(())
    }

    /// Returns the actual rate set
    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        let non_zero_rate = match NonZero::new(rate) {
            Some(non_zero_rate) => non_zero_rate,
            None => return Err(ErrorCode::INVAL),
        };
        let usart = &USARTRegManager::new(self);
        self.set_baud_rate(usart, non_zero_rate);

        // Calculate what rate will actually be
        let system_frequency = self.pm.get_system_frequency();
        // DIVISION: No division by 0 can occur because of the `non_zero_rate` type
        let cd = system_frequency / non_zero_rate.get();
        Ok(system_frequency / cd)
    }

    fn get_rate(&self) -> u32 {
        let usart = &USARTRegManager::new(self);
        let system_frequency = self.pm.get_system_frequency();
        let cd = usart.registers.brgr.read(BaudRate::CD);
        system_frequency / cd
    }

    fn set_polarity(&self, polarity: spi::ClockPolarity) -> Result<(), ErrorCode> {
        let usart = &USARTRegManager::new(self);
        // Note that in SPI mode MSBF bit is clock polarity (CPOL)
        match polarity {
            spi::ClockPolarity::IdleLow => {
                usart.registers.mr.modify(Mode::MSBF::CLEAR);
            }
            spi::ClockPolarity::IdleHigh => {
                usart.registers.mr.modify(Mode::MSBF::SET);
            }
        }
        Ok(())
    }

    fn get_polarity(&self) -> spi::ClockPolarity {
        let usart = &USARTRegManager::new(self);

        // Note that in SPI mode MSBF bit is clock polarity (CPOL)
        let idle = usart.registers.mr.read(Mode::MSBF);
        match idle {
            0 => spi::ClockPolarity::IdleLow,
            _ => spi::ClockPolarity::IdleHigh,
        }
    }

    fn set_phase(&self, phase: spi::ClockPhase) -> Result<(), ErrorCode> {
        let usart = &USARTRegManager::new(self);

        // Note that in SPI mode SYNC bit is clock phase
        match phase {
            spi::ClockPhase::SampleLeading => {
                usart.registers.mr.modify(Mode::SYNC::SET);
            }
            spi::ClockPhase::SampleTrailing => {
                usart.registers.mr.modify(Mode::SYNC::CLEAR);
            }
        }
        Ok(())
    }

    fn get_phase(&self) -> spi::ClockPhase {
        let usart = &USARTRegManager::new(self);
        let phase = usart.registers.mr.read(Mode::SYNC);

        // Note that in SPI mode SYNC bit is clock phase
        match phase {
            0 => spi::ClockPhase::SampleLeading,
            _ => spi::ClockPhase::SampleTrailing,
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
