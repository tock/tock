//! Implementation of the SAM4L USART peripheral.
//!
//! Supports UART and SPI master modes.

use core::cell::Cell;
use core::cmp;
use core::mem;
use dma;
use kernel::ReturnCode;
use kernel::common::VolatileCell;
// other modules
use kernel::hil;
// local modules
use pm;

// Register map for SAM4L USART
#[repr(C, packed)]
struct USARTRegisters {
    cr: VolatileCell<u32>, // 0x00
    mr: VolatileCell<u32>,
    ier: VolatileCell<u32>,
    idr: VolatileCell<u32>,
    imr: VolatileCell<u32>,
    csr: VolatileCell<u32>,
    rhr: VolatileCell<u32>,
    thr: VolatileCell<u32>,
    brgr: VolatileCell<u32>,
    rtor: VolatileCell<u32>,
    ttgr: VolatileCell<u32>, // 0x28
    _reserved0: [VolatileCell<u32>; 5],
    fidi: VolatileCell<u32>, // 0x40
    ner: VolatileCell<u32>,
    _reserved1: VolatileCell<u32>,
    ifr: VolatileCell<u32>,
    man: VolatileCell<u32>,
    linmr: VolatileCell<u32>,
    linir: VolatileCell<u32>,
    linbrr: VolatileCell<u32>, // 0x5C
    _reserved2: [VolatileCell<u32>; 33],
    wpmr: VolatileCell<u32>, // 0xE4
    wpsr: VolatileCell<u32>,
    _reserved3: [VolatileCell<u32>; 4],
    version: VolatileCell<u32>, // 0xFC
}

const USART_BASE_ADDRS: [*mut USARTRegisters; 4] = [0x40024000 as *mut USARTRegisters,
                                                    0x40028000 as *mut USARTRegisters,
                                                    0x4002C000 as *mut USARTRegisters,
                                                    0x40030000 as *mut USARTRegisters];

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

#[derive(Copy,Clone)]
enum UsartMode {
    Uart,
    Spi,
    Unused,
}

#[derive(Copy,Clone)]
enum UsartClient<'a> {
    Uart(&'a hil::uart::Client),
    SpiMaster(&'a hil::spi::SpiMasterClient),
}

pub struct USART {
    registers: *mut USARTRegisters,
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
pub static mut USART0: USART = USART::new(USART_BASE_ADDRS[0],
                                          pm::PBAClock::USART0,
                                          dma::DMAPeripheral::USART0_RX,
                                          dma::DMAPeripheral::USART0_TX);
pub static mut USART1: USART = USART::new(USART_BASE_ADDRS[1],
                                          pm::PBAClock::USART1,
                                          dma::DMAPeripheral::USART1_RX,
                                          dma::DMAPeripheral::USART1_TX);
pub static mut USART2: USART = USART::new(USART_BASE_ADDRS[2],
                                          pm::PBAClock::USART2,
                                          dma::DMAPeripheral::USART2_RX,
                                          dma::DMAPeripheral::USART2_TX);
pub static mut USART3: USART = USART::new(USART_BASE_ADDRS[3],
                                          pm::PBAClock::USART3,
                                          dma::DMAPeripheral::USART3_RX,
                                          dma::DMAPeripheral::USART3_TX);

impl USART {
    const fn new(base_addr: *mut USARTRegisters,
                 clock: pm::PBAClock,
                 rx_dma_peripheral: dma::DMAPeripheral,
                 tx_dma_peripheral: dma::DMAPeripheral)
                 -> USART {
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
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let cr_val = 0x00000000 | (1 << 4); // RXEN
        regs.cr.set(cr_val);
    }

    pub fn enable_tx(&self) {
        self.enable_clock();
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let cr_val = 0x00000000 | (1 << 6); // TXEN
        regs.cr.set(cr_val);
    }

    pub fn disable_rx(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let cr_val = 0x00000000 | (1 << 5); // RXDIS
        regs.cr.set(cr_val);

        self.usart_rx_state.set(USARTStateRX::Idle);
        if self.usart_tx_state.get() == USARTStateTX::Idle {
            // TX disabled too
            self.disable_clock();
        }
    }

    pub fn disable_tx(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let cr_val = 0x00000000 | (1 << 7); // TXDIS
        regs.cr.set(cr_val);

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
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.ier.set(1 << 9);
    }

    pub fn disable_tx_empty_interrupt(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.idr.set(1 << 9);
    }

    pub fn enable_rx_error_interrupts(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let ier_val = 0x00000000 |
            (1 <<  7) | // PARE
            (1 <<  6) | // FRAME
            (1 <<  5); //. OVRE
        regs.ier.set(ier_val);
    }

    pub fn disable_rx_interrupts(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let idr_val = 0x00000000 |
            (1 << 12) | // RXBUFF
            (1 <<  8) | // TIMEOUT
            (1 <<  7) | // PARE
            (1 <<  6) | // FRAME
            (1 <<  5) | // OVRE
            (1 << 1); //.. RXRDY
        regs.idr.set(idr_val);
    }

    pub fn disable_tx_interrupts(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let idr_val = 0x00000000 |
            (1 << 9) | // TXEMPTY
            (1 << 1); //. TXREADY
        regs.idr.set(idr_val);
    }

    pub fn disable_interrupts(&self) {
        self.disable_rx_interrupts();
        self.disable_tx_interrupts();
    }

    pub fn reset(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };

        // reset status bits, transmitter, and receiver
        let cr_val = 0x00000000 |
            (1 << 8) | // RSTSTA
            (1 << 3) | // RSTTX
            (1 <<2); //.. RSTRX
        regs.cr.set(cr_val);

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

            let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
            let status = regs.csr.get();
            let mask = regs.imr.get();

            // Reset status registers. We need to do this first because some
            // interrupts signal us to turn off our clock.
            regs.cr.set(1 << 8); // RSTSTA

            if status & (1 << 8) != 0 && mask & (1 << 8) != 0 {
                // TIMEOUT
                self.disable_rx_timeout();
                self.abort_rx(hil::uart::Error::CommandComplete);
            } else if status & (1 << 9) != 0 && mask & (1 << 9) != 0 {
                self.disable_tx_empty_interrupt();
                self.disable_tx();
                self.usart_tx_state.set(USARTStateTX::Idle);
            } else if status & (1 << 7) != 0 {
                // PARE
                self.abort_rx(hil::uart::Error::ParityError);

            } else if status & (1 << 6) != 0 {
                // FRAME
                self.abort_rx(hil::uart::Error::FramingError);

            } else if status & (1 << 5) != 0 {
                // OVRE
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

    fn set_mode(&self, mode: u32) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.mr.set(mode);
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };

        let system_frequency = pm::get_system_frequency();

        // The clock divisor is calculated differently in UART and SPI modes.
        let cd = match self.usart_mode.get() {
            UsartMode::Uart => system_frequency / (8 * baud_rate),
            UsartMode::Spi => system_frequency / baud_rate,
            _ => 0,
        };

        regs.brgr.set(cd);
    }

    /// In non-SPI mode, this drives RTS low.
    /// In SPI mode, this asserts (drives low) the chip select line.
    fn rts_enable_spi_assert_cs(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.cr.set(1 << 18);
    }

    /// In non-SPI mode, this drives RTS high.
    /// In SPI mode, this de-asserts (drives high) the chip select line.
    fn rts_disable_spi_deassert_cs(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.cr.set(1 << 19);
    }

    fn enable_rx_timeout(&self, timeout: u8) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let rtor_val: u32 = 0x00000000 | timeout as u32;
        regs.rtor.set(rtor_val);

        // enable timeout interrupt
        regs.ier.set((1 << 8)); // TIMEOUT

        // start timeout
        regs.cr.set((1 << 11)); // STTTO
    }

    fn disable_rx_timeout(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.rtor.set(0);

        // enable timeout interrupt
        regs.idr.set((1 << 8)); // TIMEOUT
    }

    fn enable_rx_terminator(&self, _terminator: u8) {
        // XXX: implement me
        panic!("didn't write terminator stuff yet");
    }

    // for use by panic in io.rs
    pub fn send_byte(&self, byte: u8) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let thr_val: u32 = 0x00000000 | byte as u32;
        regs.thr.set(thr_val);
    }

    // for use by panic in io.rs
    pub fn tx_ready(&self) -> bool {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let csr_val: u32 = regs.csr.get();
        let mut ret_val = false;
        if (csr_val & (1 << 1)) == (1 << 1) {
            // tx is ready
            ret_val = true;
        }
        ret_val
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
                                    client.receive_complete(buf,
                                                            length,
                                                            hil::uart::Error::CommandComplete);
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
                if (self.usart_rx_state.get() == USARTStateRX::Idle &&
                    pid == self.tx_dma_peripheral) ||
                   pid == self.rx_dma_peripheral {
                    // SPI transfer was completed

                    self.spi_chip_select.get().map_or_else(|| {
                        // Do "else" case first. Thanks, rust.
                        self.rts_disable_spi_deassert_cs();
                    }, |cs| {
                        cs.set();
                    });

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
        let mut mode = 0x00000000;
        mode |= 0x1 << 19; // OVER: oversample at 8 times baud rate
        mode |= 0x3 << 6; // CHRL: 8-bit characters
        mode |= 0x0 << 4; // USCLKS: select CLK_USART

        match params.stop_bits {
            hil::uart::StopBits::One => mode |= 0x0 << 12, // NBSTOP: 1 stop bit
            hil::uart::StopBits::Two => mode |= 0x2 << 12, // NBSTOP: 2 stop bits
        };

        match params.parity {
            hil::uart::Parity::None => mode |= 0x4 << 9,   // PAR: no parity
            hil::uart::Parity::Odd => mode |= 0x1 << 9,    // PAR: odd parity
            hil::uart::Parity::Even => mode |= 0x0 << 9,   // PAR: even parity
        };

        if params.hw_flow_control {
            mode |= 0x2 << 0; // MODE: hardware handshaking
        } else {
            mode |= 0x0 << 0; // MODE: normal
        }

        self.set_mode(mode);

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
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };

        self.usart_mode.set(UsartMode::Spi);
        self.enable_clock();

        // Set baud rate, default to 2 MHz.
        self.set_baud_rate(2000000);

        let mode =
            0xe << 0 /* SPI Master mode */
            | 0 << 4 /* USCLKS*/
            | 0x3 << 6 /* Character Length 8 bits */
            | 0x4 << 9 /* No Parity */
            | 1 << 18 /* USART drives the clock pin */;
        self.set_mode(mode);

        // Disable transmitter timeguard
        regs.ttgr.set(4);

        self.disable_clock();
    }


    fn set_client(&self, client: &'static hil::spi::SpiMasterClient) {
        let c = UsartClient::SpiMaster(client);
        self.client.set(Some(c));
    }

    fn is_busy(&self) -> bool {
        return false;
    }

    fn read_write_bytes(&self,
                        mut write_buffer: &'static mut [u8],
                        read_buffer: Option<&'static mut [u8]>,
                        len: usize)
                        -> ReturnCode {

        self.enable_tx();
        self.enable_rx();

        // Calculate the correct length for the transmission
        let buflen = read_buffer.as_ref().map_or(write_buffer.len(),
                                                 |rbuf| cmp::min(rbuf.len(), write_buffer.len()));
        let count = cmp::min(buflen, len);

        self.tx_len.set(count);

        // Set !CS low
        self.spi_chip_select.get().map_or_else(|| {
            // Do the "else" case first. If a CS pin was provided as the
            // CS line, we use the HW RTS pin as the CS line instead.
            self.rts_enable_spi_assert_cs();
        }, |cs| {
            cs.clear();
        });

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
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.enable_clock();
        regs.cr.set((1 << 4) | (1 << 6));

        regs.thr.set(val as u32);
    }

    fn read_byte(&self) -> u8 {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.enable_clock();
        regs.rhr.get() as u8
    }

    fn read_write_byte(&self, val: u8) -> u8 {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.enable_clock();
        regs.cr.set((1 << 4) | (1 << 6));

        regs.thr.set(val as u32);
        while regs.csr.get() & (1 << 0) == 0 {}
        regs.rhr.get() as u8
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
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.enable_clock();
        let system_frequency = pm::get_system_frequency();
        let cd = regs.brgr.get() & 0xFFFF;
        system_frequency / cd
    }

    fn set_clock(&self, polarity: hil::spi::ClockPolarity) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.enable_clock();
        let mode = regs.mr.get();

        match polarity {
            hil::spi::ClockPolarity::IdleLow => {
                regs.mr.set(mode & !(1 << 16));
            }
            hil::spi::ClockPolarity::IdleHigh => {
                regs.mr.set(mode | (1 << 16));
            }
        }
    }

    fn get_clock(&self) -> hil::spi::ClockPolarity {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.enable_clock();
        let mode = regs.mr.get();

        match mode & (1 << 16) {
            0 => hil::spi::ClockPolarity::IdleLow,
            _ => hil::spi::ClockPolarity::IdleHigh,
        }
    }

    fn set_phase(&self, phase: hil::spi::ClockPhase) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.enable_clock();
        let mode = regs.mr.get();

        match phase {
            hil::spi::ClockPhase::SampleLeading => {
                regs.mr.set(mode | (1 << 8));
            }
            hil::spi::ClockPhase::SampleTrailing => {
                regs.mr.set(mode & !(1 << 8));
            }
        }
    }

    fn get_phase(&self) -> hil::spi::ClockPhase {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.enable_clock();
        let mode = regs.mr.get();

        match mode & (1 << 8) {
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
