// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Universal Asynchronous Receiver/Transmitter (UART)

use crate::dma;
use crate::usci::{self, UsciARegisters};
use core::cell::Cell;
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

const DEFAULT_CLOCK_FREQ_HZ: u32 = crate::cs::SMCLK_HZ;

struct BaudFraction {
    frac: f32,
    reg_val: u8,
}

#[rustfmt::skip]
// Table out of the datasheet to correct the baudrate
const BAUD_FRACTIONS: &[BaudFraction; 36] = &[
    BaudFraction { frac: 0.0000, reg_val: 0x00 },
    BaudFraction { frac: 0.0529, reg_val: 0x01 },
    BaudFraction { frac: 0.0715, reg_val: 0x02 },
    BaudFraction { frac: 0.0835, reg_val: 0x04 },
    BaudFraction { frac: 0.1001, reg_val: 0x08 },
    BaudFraction { frac: 0.1252, reg_val: 0x10 },
    BaudFraction { frac: 0.1430, reg_val: 0x20 },
    BaudFraction { frac: 0.1670, reg_val: 0x11 },
    BaudFraction { frac: 0.2147, reg_val: 0x21 },
    BaudFraction { frac: 0.2224, reg_val: 0x22 },
    BaudFraction { frac: 0.2503, reg_val: 0x44 },
    BaudFraction { frac: 0.3000, reg_val: 0x25 },
    BaudFraction { frac: 0.3335, reg_val: 0x49 },
    BaudFraction { frac: 0.3575, reg_val: 0x4A },
    BaudFraction { frac: 0.3753, reg_val: 0x52 },
    BaudFraction { frac: 0.4003, reg_val: 0x92 },
    BaudFraction { frac: 0.4286, reg_val: 0x53 },
    BaudFraction { frac: 0.4378, reg_val: 0x55 },
    BaudFraction { frac: 0.5002, reg_val: 0xAA },
    BaudFraction { frac: 0.5715, reg_val: 0x6B },
    BaudFraction { frac: 0.6003, reg_val: 0xAD },
    BaudFraction { frac: 0.6254, reg_val: 0xB5 },
    BaudFraction { frac: 0.6432, reg_val: 0xB6 },
    BaudFraction { frac: 0.6667, reg_val: 0xD6 },
    BaudFraction { frac: 0.7001, reg_val: 0xB7 },
    BaudFraction { frac: 0.7147, reg_val: 0xBB },
    BaudFraction { frac: 0.7503, reg_val: 0xDD },
    BaudFraction { frac: 0.7861, reg_val: 0xED },
    BaudFraction { frac: 0.8004, reg_val: 0xEE },
    BaudFraction { frac: 0.8333, reg_val: 0xBF },
    BaudFraction { frac: 0.8464, reg_val: 0xDF },
    BaudFraction { frac: 0.8572, reg_val: 0xEF },
    BaudFraction { frac: 0.8751, reg_val: 0xF7 },
    BaudFraction { frac: 0.9004, reg_val: 0xFB },
    BaudFraction { frac: 0.9170, reg_val: 0xFD },
    BaudFraction { frac: 0.9288, reg_val: 0xFE },
];

pub struct Uart<'a> {
    registers: StaticRef<UsciARegisters>,
    clock_frequency: u32,

    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    tx_busy: Cell<bool>,
    tx_dma: OptionalCell<&'a dma::DmaChannel<'a>>,
    pub(crate) tx_dma_chan: usize,
    tx_dma_src: u8,

    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    rx_busy: Cell<bool>,
    rx_dma: OptionalCell<&'a dma::DmaChannel<'a>>,
    pub(crate) rx_dma_chan: usize,
    rx_dma_src: u8,
}

impl<'a> Uart<'a> {
    pub const fn new(
        registers: StaticRef<UsciARegisters>,
        tx_dma_chan: usize,
        rx_dma_chan: usize,
        tx_dma_src: u8,
        rx_dma_src: u8,
    ) -> Self {
        Self {
            registers,
            clock_frequency: DEFAULT_CLOCK_FREQ_HZ,

            tx_client: OptionalCell::empty(),
            tx_dma: OptionalCell::empty(),
            tx_dma_chan,
            tx_dma_src,
            tx_busy: Cell::new(false),

            rx_client: OptionalCell::empty(),
            rx_dma: OptionalCell::empty(),
            rx_dma_chan,
            rx_dma_src,
            rx_busy: Cell::new(false),
        }
    }

    pub fn set_dma(&self, tx_dma: &'a dma::DmaChannel<'a>, rx_dma: &'a dma::DmaChannel<'a>) {
        self.tx_dma.replace(tx_dma);
        self.rx_dma.replace(rx_dma);
    }

    pub fn transmit_sync(&self, data: &[u8]) {
        for b in data.iter() {
            while self.registers.statw.is_set(usci::UCAxSTATW::UCBUSY) {}
            self.registers.txbuf.set(*b as u16);
        }
    }
}

impl dma::DmaClient for Uart<'_> {
    fn transfer_done(
        &self,
        tx_buf: Option<&'static mut [u8]>,
        rx_buf: Option<&'static mut [u8]>,
        transmitted_bytes: usize,
    ) {
        if let Some(rxbuf) = rx_buf {
            // RX-transfer done
            self.rx_busy.set(false);
            self.rx_client.map(|client| {
                client.received_buffer(rxbuf, transmitted_bytes, Ok(()), hil::uart::Error::None)
            });
        } else if let Some(txbuf) = tx_buf {
            // TX-transfer done
            self.tx_busy.set(false);
            self.tx_client
                .map(|client| client.transmitted_buffer(txbuf, transmitted_bytes, Ok(())));
        }
    }
}

impl hil::uart::Configure for Uart<'_> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        // Disable module
        let regs = self.registers;
        regs.ctlw0.modify(usci::UCAxCTLW0::UCSWRST::SET);

        // Setup module to UART mode
        regs.ctlw0.modify(usci::UCAxCTLW0::UCMODE::UARTMode);

        // Setup clock-source to SMCLK
        regs.ctlw0.modify(usci::UCAxCTLW0::UCSSEL::SMCLK);

        // Setup word-length
        match params.width {
            hil::uart::Width::Eight => regs.ctlw0.modify(usci::UCAxCTLW0::UC7BIT::CLEAR),
            hil::uart::Width::Seven => regs.ctlw0.modify(usci::UCAxCTLW0::UC7BIT::SET),
            hil::uart::Width::Six => {
                panic!("UART: width of 6 bit is not supported by this hardware!")
            }
        }

        // Setup stop bits
        if params.stop_bits == hil::uart::StopBits::One {
            regs.ctlw0.modify(usci::UCAxCTLW0::UCSPB::CLEAR);
        } else {
            regs.ctlw0.modify(usci::UCAxCTLW0::UCSPB::SET);
        }

        // Setup parity
        if params.parity == hil::uart::Parity::None {
            regs.ctlw0.modify(usci::UCAxCTLW0::UCPEN::CLEAR);
        } else {
            regs.ctlw0.modify(usci::UCAxCTLW0::UCPEN::SET);
            if params.parity == hil::uart::Parity::Even {
                regs.ctlw0.modify(usci::UCAxCTLW0::UCPAR::SET);
            } else {
                regs.ctlw0.modify(usci::UCAxCTLW0::UCPAR::CLEAR);
            }
        }

        // Setup baudrate, all the calculation from the datasheet p. 915
        // DIVISION: no division by 0 can occur because of the `baud_rate` type.
        let n = (self.clock_frequency / params.baud_rate.get()) as u16;
        // DIVISION: no division by 0 can occur because of the `baud_rate` type.
        let n_float = (self.clock_frequency as f32) / (params.baud_rate.get() as f32);
        let frac_part = n_float - (n as f32);
        if n > 16 {
            // Oversampling is enabled
            regs.brw.set(n >> 4); // equals n / 16
            let ucbrf = (((n_float / 16.0f32) - ((n >> 4) as f32)) * 16.0f32) as u16;
            regs.mctlw
                .modify(usci::UCAxMCTLW::UCBRF.val(ucbrf) + usci::UCAxMCTLW::UCOS16::SET);
        } else {
            // No oversampling
            regs.brw.set(n);
            regs.mctlw.modify(usci::UCAxMCTLW::UCOS16::CLEAR);
        }

        // Look for the closest calibration value
        // According to the datasheet not the closest value should be taken but the next smaller one
        let mut ucbrs = BAUD_FRACTIONS[0].reg_val;
        for val in BAUD_FRACTIONS.iter() {
            if val.frac > frac_part {
                break;
            }
            ucbrs = val.reg_val;
        }
        regs.mctlw.modify(usci::UCAxMCTLW::UCBRS.val(ucbrs as u16));

        // Enable module
        regs.ctlw0.modify(usci::UCAxCTLW0::UCSWRST::CLEAR);

        // Configure the DMA
        let tx_conf = dma::DmaConfig {
            src_chan: self.tx_dma_src,
            mode: dma::DmaMode::Basic,
            width: dma::DmaDataWidth::Width8Bit,
            src_incr: dma::DmaPtrIncrement::Incr8Bit,
            dst_incr: dma::DmaPtrIncrement::NoIncr,
        };

        let rx_conf = dma::DmaConfig {
            src_chan: self.rx_dma_src,
            mode: dma::DmaMode::Basic,
            width: dma::DmaDataWidth::Width8Bit,
            src_incr: dma::DmaPtrIncrement::NoIncr,
            dst_incr: dma::DmaPtrIncrement::Incr8Bit,
        };

        self.tx_dma.map(|dma| dma.initialize(&tx_conf));
        self.rx_dma.map(|dma| dma.initialize(&rx_conf));

        Ok(())
    }
}

impl<'a> hil::uart::Transmit<'a> for Uart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if (tx_len == 0) || (tx_len > tx_buffer.len()) {
            return Err((ErrorCode::SIZE, tx_buffer));
        }
        if self.tx_busy.get() {
            Err((ErrorCode::BUSY, tx_buffer))
        } else {
            self.tx_busy.set(true);
            let tx_reg = core::ptr::addr_of!(self.registers.txbuf).cast::<()>();
            self.tx_dma
                .map(move |dma| dma.transfer_mem_to_periph(tx_reg, tx_buffer, tx_len));
            Ok(())
        }
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if !self.tx_busy.get() {
            return Ok(());
        }

        self.tx_dma.map(|dma| {
            let (nr_bytes, tx1, _rx1, _tx2, _rx2) = dma.stop();

            self.tx_client.map(move |cl| {
                if let Some(tx1_buf) = tx1 {
                    cl.transmitted_buffer(tx1_buf, nr_bytes, Err(ErrorCode::CANCEL));
                }
            });
        });

        Err(ErrorCode::BUSY)
    }
}

impl<'a> hil::uart::Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if (rx_len == 0) || (rx_len > rx_buffer.len()) {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        if self.rx_busy.get() {
            Err((ErrorCode::BUSY, rx_buffer))
        } else {
            self.rx_busy.set(true);
            let rx_reg = core::ptr::addr_of!(self.registers.rxbuf).cast::<()>();
            self.rx_dma
                .map(move |dma| dma.transfer_periph_to_mem(rx_reg, rx_buffer, rx_len));
            Ok(())
        }
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if !self.rx_busy.get() {
            return Ok(());
        }

        self.rx_dma.map(|dma| {
            let (nr_bytes, _tx1, rx1, _tx2, _rx2) = dma.stop();

            self.rx_client.map(move |cl| {
                if let Some(rx1_buf) = rx1 {
                    cl.received_buffer(
                        rx1_buf,
                        nr_bytes,
                        Err(ErrorCode::CANCEL),
                        hil::uart::Error::Aborted,
                    );
                }
            });
        });

        Err(ErrorCode::BUSY)
    }
}
