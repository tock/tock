// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::uart::{self};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

use crate::dma::Dma;

register_structs! {
    pub UsartRegisters {
        /// Control register 1
        (0x000 => pub cr1: ReadWrite<u32, CR1::Register>),
        /// Control register 2
        (0x004 => pub cr2: ReadWrite<u32, CR2::Register>),
        /// Control register 3
        (0x008 => pub cr3: ReadWrite<u32, CR3::Register>),
        /// Baud rate register
        (0x00C => pub brr: ReadWrite<u32>),
        /// Guard time and prescaler register
        (0x010 => pub gtpr: ReadWrite<u32>),
        /// Receiver timeout register
        (0x014 => pub rtor: ReadWrite<u32>),
        /// Request register
        (0x018 => pub rqr: ReadWrite<u32>),
        /// Interrupt and status register
        (0x01C => pub isr: ReadOnly<u32, ISR::Register>),
        /// Interrupt flag clear register
        (0x020 => pub icr: ReadWrite<u32>),
        /// Receive data register
        (0x024 => pub rdr: ReadOnly<u32>),
        /// Transmit data register
        (0x028 => pub tdr: ReadWrite<u32>),
        /// Prescaler register
        (0x02C => pub presc: ReadWrite<u32>),
        (0x030 => @END),
    }
}

register_bitfields![u32,
    pub CR1 [
        /// USART enable
        UE      OFFSET(0)   NUMBITS(1) [],
        /// Receiver enable
        RE      OFFSET(2)   NUMBITS(1) [],
        /// Transmitter enable
        TE      OFFSET(3)   NUMBITS(1) [],
        /// RXNE interrupt enable
        RXNEIE  OFFSET(5)   NUMBITS(1) [],
        /// Transmission complete interrupt enable
        TCIE    OFFSET(6)   NUMBITS(1) [],
        /// TXE interrupt enable
        TXEIE   OFFSET(7)   NUMBITS(1) []
    ],
    pub CR2 [
        /// STOP bits
        STOP    OFFSET(12)  NUMBITS(2) []
    ],
    pub CR3 [
        /// Error interrupt enable
        EIE     OFFSET(0)   NUMBITS(1) [],
        /// DMA enable transmitter
        DMAT    OFFSET(7)   NUMBITS(1) [],
        /// DMA enable receiver
        DMAR    OFFSET(6)   NUMBITS(1) []
    ],
    pub ISR [
        /// Read data register not empty
        RXNE OFFSET(5) NUMBITS(1) [],
        /// Transmit data register empty
        TXE  OFFSET(7) NUMBITS(1) []
    ]
];

pub struct Usart<'a> {
    pub registers: StaticRef<UsartRegisters>,
    dma: OptionalCell<&'a Dma>,
    dma_channel_tx: Cell<usize>,
    dma_channel_rx: Cell<usize>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    rx_buffer: TakeCell<'static, [u8]>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    deferred_call: DeferredCall,
    fifo: Cell<[u8; 32]>,
    fifo_write: Cell<usize>,
    fifo_read: Cell<usize>,
}

impl<'a> Usart<'a> {
    pub fn new(base: StaticRef<UsartRegisters>) -> Usart<'a> {
        Usart {
            registers: base,
            dma: OptionalCell::empty(),
            dma_channel_tx: Cell::new(0),
            dma_channel_rx: Cell::new(0),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            deferred_call: DeferredCall::new(),
            fifo: Cell::new([0; 32]),
            fifo_write: Cell::new(0),
            fifo_read: Cell::new(0),
        }
    }

    pub fn set_dma(&self, dma: &'a Dma, tx_channel: usize, rx_channel: usize) {
        self.dma.set(dma);
        self.dma_channel_tx.set(tx_channel);
        // Unnused but kept for testing
        self.dma_channel_rx.set(rx_channel);
    }

    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;
        let isr = regs.isr.get();

        if (isr & 0x0F) != 0 {
            regs.icr.set(0x0F);
        }

        // Receive Logic (IRQ Based - Drain hardware into FIFO)
        let mut data_received = false;
        while regs.isr.is_set(ISR::RXNE) {
            let byte = regs.rdr.get() as u8;
            let mut f = self.fifo.get();
            let w = self.fifo_write.get();
            f[w % 32] = byte;
            self.fifo.set(f);
            self.fifo_write.set(w + 1);
            data_received = true;
        }

        if data_received {
            self.deferred_call.set();
        }
    }

    pub fn handle_dma_interrupt(&self, is_tx: bool) {
        if is_tx {
            self.dma.map(|dma| {
                dma.clear_interrupt(self.dma_channel_tx.get());
            });
            self.registers.cr3.modify(CR3::DMAT::CLEAR);
            self.deferred_call.set();
        }
    }

    fn try_receive_from_fifo(&self) {
        let r = self.fifo_read.get();
        let w = self.fifo_write.get();
        if r < w {
            if let Some(buf) = self.rx_buffer.take() {
                let f = self.fifo.get();
                buf[0] = f[r % 32];
                self.fifo_read.set(r + 1);
                self.rx_client.map(|client| {
                    client.received_buffer(buf, 1, Ok(()), uart::Error::None);
                });
            }
        }
    }
}

impl<'a> DeferredCallClient for Usart<'a> {
    fn handle_deferred_call(&self) {
        // 1. Transmit Completion (DMA)
        if let Some(buf) = self.tx_buffer.take() {
            let len = self.tx_len.get();
            self.tx_client.map(move |client| {
                client.transmitted_buffer(buf, len, Ok(()));
            });
        }

        // 2. Receive Completion (FIFO)
        self.try_receive_from_fifo();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<'a> uart::Transmit<'a> for Usart<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        if self.tx_buffer.is_some() {
            return Err((kernel::ErrorCode::BUSY, tx_buffer));
        }

        self.tx_buffer.replace(tx_buffer);
        self.tx_len.set(tx_len);

        self.dma.map(|dma| {
            self.tx_buffer.map(|buf| {
                dma.setup_usart1_tx(
                    self.dma_channel_tx.get(),
                    buf.as_ptr() as u32,
                    tx_len as u32,
                );
                self.registers.cr3.modify(CR3::DMAT::SET);
            });
        });

        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.cr3.modify(CR3::DMAT::CLEAR);
        if let Some(buf) = self.tx_buffer.take() {
            self.tx_client.map(move |client| {
                client.transmitted_buffer(buf, 0, Err(kernel::ErrorCode::CANCEL));
            });
        }
        Ok(())
    }

    fn transmit_word(&self, _word: u32) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }
}

impl<'a> uart::Configure for Usart<'a> {
    fn configure(&self, _params: uart::Parameters) -> Result<(), kernel::ErrorCode> {
        let regs = &*self.registers;
        regs.cr1.modify(CR1::UE::CLEAR);
        regs.presc.set(0);
        regs.brr.set(35);
        regs.icr.set(0x3F);
        // Hybrid: RXNEIE is ENABLED for typing interrupts
        regs.cr1
            .write(CR1::TE::SET + CR1::RE::SET + CR1::UE::SET + CR1::RXNEIE::SET);
        Ok(())
    }
}

impl<'a> uart::Receive<'a> for Usart<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        _rx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        self.rx_buffer.replace(rx_buffer);
        self.try_receive_from_fifo();
        Ok(())
    }

    fn receive_abort(&self) -> Result<(), kernel::ErrorCode> {
        if let Some(buf) = self.rx_buffer.take() {
            self.rx_client.map(move |client| {
                client.received_buffer(
                    buf,
                    0,
                    Err(kernel::ErrorCode::CANCEL),
                    uart::Error::Aborted,
                );
            });
        }
        Ok(())
    }

    fn receive_word(&self) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }
}
