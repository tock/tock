use core::cell::Cell;
use core::mem;
use dma;
use kernel::common::take_cell::TakeCell;
use kernel::common::volatile_cell::VolatileCell;
// other modules
use kernel::hil;
// local modules
use nvic;
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

pub struct USART {
    registers: *mut USARTRegisters,
    clock: pm::Clock,
    nvic: nvic::NvicIdx,

    usart_tx_state: Cell<USARTStateTX>,
    usart_rx_state: Cell<USARTStateRX>,

    rx_dma: TakeCell<&'static dma::DMAChannel>,
    rx_dma_peripheral: dma::DMAPeripheral,
    rx_len: Cell<usize>,
    tx_dma: TakeCell<&'static dma::DMAChannel>,
    tx_dma_peripheral: dma::DMAPeripheral,
    tx_len: Cell<usize>,

    client: TakeCell<&'static hil::uart::Client>,
}

// USART hardware peripherals on SAM4L
pub static mut USART0: USART = USART::new(USART_BASE_ADDRS[0],
                                          pm::PBAClock::USART0,
                                          nvic::NvicIdx::USART0,
                                          dma::DMAPeripheral::USART0_RX,
                                          dma::DMAPeripheral::USART0_TX);
pub static mut USART1: USART = USART::new(USART_BASE_ADDRS[1],
                                          pm::PBAClock::USART1,
                                          nvic::NvicIdx::USART1,
                                          dma::DMAPeripheral::USART1_RX,
                                          dma::DMAPeripheral::USART1_TX);
pub static mut USART2: USART = USART::new(USART_BASE_ADDRS[2],
                                          pm::PBAClock::USART2,
                                          nvic::NvicIdx::USART2,
                                          dma::DMAPeripheral::USART2_RX,
                                          dma::DMAPeripheral::USART2_TX);
pub static mut USART3: USART = USART::new(USART_BASE_ADDRS[3],
                                          pm::PBAClock::USART3,
                                          nvic::NvicIdx::USART3,
                                          dma::DMAPeripheral::USART3_RX,
                                          dma::DMAPeripheral::USART3_TX);

impl USART {
    const fn new(base_addr: *mut USARTRegisters,
                 clock: pm::PBAClock,
                 nvic: nvic::NvicIdx,
                 rx_dma_peripheral: dma::DMAPeripheral,
                 tx_dma_peripheral: dma::DMAPeripheral)
                 -> USART {
        USART {
            registers: base_addr,
            clock: pm::Clock::PBA(clock),
            nvic: nvic,

            usart_rx_state: Cell::new(USARTStateRX::Idle),
            usart_tx_state: Cell::new(USARTStateTX::Idle),

            // these get defined later by `chip.rs`
            rx_dma: TakeCell::empty(),
            rx_dma_peripheral: rx_dma_peripheral,
            rx_len: Cell::new(0),
            tx_dma: TakeCell::empty(),
            tx_dma_peripheral: tx_dma_peripheral,
            tx_len: Cell::new(0),

            // this gets defined later by `main.rs`
            client: TakeCell::empty(),
        }
    }

    pub fn set_dma(&self, rx_dma: &'static dma::DMAChannel, tx_dma: &'static dma::DMAChannel) {
        self.rx_dma.replace(rx_dma);
        self.tx_dma.replace(tx_dma);
    }

    pub fn enable_rx(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let cr_val = 0x00000000 | (1 << 4); // RXEN
        regs.cr.set(cr_val);
    }

    pub fn enable_tx(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let cr_val = 0x00000000 | (1 << 6); // TXEN
        regs.cr.set(cr_val);
    }

    pub fn disable_rx(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let cr_val = 0x00000000 | (1 << 5); // RXDIS
        regs.cr.set(cr_val);

        self.usart_rx_state.set(USARTStateRX::Idle);
    }

    pub fn disable_tx(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let cr_val = 0x00000000 | (1 << 7); // TXDIS
        regs.cr.set(cr_val);

        self.usart_tx_state.set(USARTStateTX::Idle);
    }

    pub fn abort_rx(&self, error: hil::uart::Error) {
        if self.usart_rx_state.get() == USARTStateRX::DMA_Receiving {
            self.disable_rx();
            self.disable_rx_interrupts();
            self.usart_rx_state.set(USARTStateRX::Idle);

            // get buffer
            let mut length = 0;
            let buffer = self.rx_dma.map_or(None, |rx_dma| {
                length = self.rx_len.get() - rx_dma.transfer_counter();
                let buf = rx_dma.abort_xfer();
                rx_dma.disable();
                buf
            });
            self.rx_len.set(0);

            // alert client
            self.client.map(|c| {
                buffer.map(|buf| {
                    c.receive_complete(buf, length, error);
                });
            });
        }
    }

    pub fn abort_tx(&self, error: hil::uart::Error) {
        if self.usart_tx_state.get() == USARTStateTX::DMA_Transmitting {
            self.disable_tx();
            self.disable_tx_interrupts();
            self.usart_tx_state.set(USARTStateTX::Idle);

            // get buffer
            let mut length = 0;
            let buffer = self.tx_dma.map_or(None, |tx_dma| {
                length = self.tx_len.get() - tx_dma.transfer_counter();
                let buf = tx_dma.abort_xfer();
                tx_dma.disable();
                buf
            });
            self.tx_len.set(0);

            // alert client
            self.client.map(|c| {
                buffer.map(|buf| {
                    c.receive_complete(buf, length, error);
                });
            });
        }
    }

    pub fn enable_rx_error_interrupts(&self) {
        self.enable_nvic();
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let ier_val = 0x00000000 |
            (1 <<  7) | // PARE
            (1 <<  6) | // FRAME
            (1 <<  5);  // OVRE
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
            (1 << 1);   // RXRDY
        regs.idr.set(idr_val);

        // XXX: disable nvic if no interrupts are enabled
    }

    pub fn disable_tx_interrupts(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let idr_val = 0x00000000 |
            (1 << 9) | // TXEMPTY
            (1 << 1);  // TXREADY
        regs.idr.set(idr_val);

        // XXX: disable nvic if no interrupts are enabled
    }

    pub fn disable_interrupts(&self) {
        self.disable_nvic();
        self.disable_rx_interrupts();
        self.disable_tx_interrupts();
    }

    pub fn reset(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };

        // reset status bits, transmitter, and receiver
        let cr_val = 0x00000000 |
            (1 << 8) | // RSTSTA
            (1 << 3) | // RSTTX
            (1 <<2);   // RSTRX
        regs.cr.set(cr_val);

        self.abort_rx(hil::uart::Error::ResetError);
        self.abort_tx(hil::uart::Error::ResetError);
    }

    pub fn handle_interrupt(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let status = regs.csr.get();

        if status & (1 << 12) != 0 {
            // DO NOTHING. Why are we here!?

        } else if status & (1 << 8) != 0 {
            // TIMEOUT
            self.disable_rx_timeout();
            self.abort_rx(hil::uart::Error::CommandComplete);

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

        // reset status registers
        regs.cr.set(1 << 8); // RSTSTA
    }

    fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(self.clock);
        }
    }

    fn enable_nvic(&self) {
        unsafe {
            nvic::enable(self.nvic);
        }
    }

    fn disable_nvic(&self) {
        unsafe {
            nvic::disable(self.nvic);
        }
    }

    fn set_mode(&self, mode: u32) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.mr.set(mode);
    }

    // NOTE: dependent on oversampling rate
    // XXX: how do you determine the current clock frequency?
    fn set_baud_rate(&self, baud_rate: u32) {
        let cd = 48000000 / (8 * baud_rate);
        self.set_baud_rate_divider(cd as u16);
    }

    fn set_baud_rate_divider(&self, clock_divider: u16) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let brgr_val: u32 = 0x00000000 | clock_divider as u32;
        regs.brgr.set(brgr_val);
    }

    fn enable_rx_timeout(&self, timeout: u8) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let rtor_val: u32 = 0x00000000 | timeout as u32;
        regs.rtor.set(rtor_val);

        // enable timeout interrupt
        regs.ier.set((1 << 8)); // TIMEOUT
        self.enable_nvic();

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
        // determine if it was an RX or TX transfer
        if pid == self.rx_dma_peripheral {
            // RX transfer was completed

            // disable RX and RX interrupts
            self.disable_rx();
            self.disable_rx_interrupts();
            self.usart_rx_state.set(USARTStateRX::Idle);

            // get buffer
            let buffer = self.rx_dma.map_or(None, |rx_dma| {
                let buf = rx_dma.abort_xfer();
                rx_dma.disable();
                buf
            });

            // alert client
            self.client.map(|c| {
                buffer.map(|buf| {
                    let length = self.rx_len.get();
                    c.receive_complete(buf, length, hil::uart::Error::CommandComplete);
                });
            });
            self.rx_len.set(0);

        } else if pid == self.tx_dma_peripheral {
            // TX transfer was completed

            // note that the DMA has finished but TX cannot be disabled yet
            self.usart_tx_state.set(USARTStateTX::Transfer_Completing);

            // get buffer
            let buffer = self.tx_dma.map_or(None, |tx_dma| {
                let buf = tx_dma.abort_xfer();
                tx_dma.disable();
                buf
            });

            // alert client
            self.client.map(|c| {
                buffer.map(|buf| c.transmit_complete(buf, hil::uart::Error::CommandComplete));
            });
            self.tx_len.set(0);
        }
    }
}

/// Implementation of kernel::hil::UART
impl hil::uart::UART for USART {
    fn set_client(&self, client: &'static hil::uart::Client) {
        self.client.replace(client);
    }

    fn init(&self, params: hil::uart::UARTParams) {
        // enable USART clock
        //  must do this before writing any registers
        self.enable_clock();

        // disable interrupts
        self.disable_interrupts();

        // stop any TX and RX and clear status
        self.reset();

        // set USART mode register
        let mut mode = 0x00000000;
        mode |= 0x1 << 19; // OVER: oversample at 8 times baud rate
        mode |= 0x3 << 6;  // CHRL: 8-bit characters
        mode |= 0x0 << 4;  // USCLKS: select CLK_USART

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
            mode |= 0x2 << 0;  // MODE: hardware handshaking
        } else {
            mode |= 0x0 << 0;  // MODE: normal
        }

        self.set_mode(mode);

        // Set baud rate
        self.set_baud_rate(params.baud_rate);
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        // quit current transmission if any
        self.abort_tx(hil::uart::Error::RepeatCallError);

        // enable TX
        self.enable_tx();
        self.usart_tx_state.set(USARTStateTX::DMA_Transmitting);

        // set up dma transfer and start transmission
        self.tx_dma.map(move |dma| {
            dma.enable();
            dma.do_xfer(self.tx_dma_peripheral, tx_data, tx_len);
            self.tx_len.set(tx_len);
        });
    }

    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
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
        self.rx_dma.map(move |dma| {
            dma.enable();
            dma.do_xfer(self.rx_dma_peripheral, rx_buffer, length);
            self.rx_len.set(rx_len);
        });
    }
}

impl hil::uart::UARTAdvanced for USART {
    fn receive_automatic(&self, rx_buffer: &'static mut [u8], interbyte_timeout: u8) {
        // quit current reception if any
        self.abort_rx(hil::uart::Error::RepeatCallError);

        // enable receive timeout
        self.enable_rx_timeout(interbyte_timeout);

        // enable RX
        self.enable_rx();
        self.enable_rx_error_interrupts();
        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);

        // set up dma transfer and start reception
        self.rx_dma.map(move |dma| {
            dma.enable();
            let length = rx_buffer.len();
            dma.do_xfer(self.rx_dma_peripheral, rx_buffer, length);
            self.rx_len.set(length);
        });
    }

    fn receive_until_terminator(&self, rx_buffer: &'static mut [u8], terminator: u8) {
        // quit current reception if any
        self.abort_rx(hil::uart::Error::RepeatCallError);

        // enable receive terminator
        self.enable_rx_terminator(terminator);

        // enable RX
        self.enable_rx();
        self.enable_rx_error_interrupts();
        self.usart_rx_state.set(USARTStateRX::DMA_Receiving);

        // set up dma transfer and start reception
        self.rx_dma.map(move |dma| {
            dma.enable();
            let length = rx_buffer.len();
            dma.do_xfer(self.rx_dma_peripheral, rx_buffer, length);
            self.rx_len.set(length);
        });
    }
}

// Register interrupt handlers
interrupt_handler!(usart0_handler, USART0);
interrupt_handler!(usart1_handler, USART1);
interrupt_handler!(usart2_handler, USART2);
interrupt_handler!(usart3_handler, USART3);
