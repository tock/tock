// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use crate::dma::{ChannelId, Dma, DmaPeripheral};
use core::cell::Cell;
use kernel::hil::uart::{self};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

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

/// Base address for USART1 in Secure Alias mode.
pub const USART1_BASE: StaticRef<UsartRegisters> =
    unsafe { StaticRef::new(0x50013800 as *const UsartRegisters) };

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
        /// Transmission complete
        TC   OFFSET(6) NUMBITS(1) [],
        /// Read data register not empty
        RXNE OFFSET(5) NUMBITS(1) [],
        /// Transmit data register empty
        TXE  OFFSET(7) NUMBITS(1) []
    ]
];

/// USART driver implementation for the STM32U5 series.
///
/// This driver uses the GPDMA (General Purpose Direct Memory Access) controller
/// for both transmitting and receiving data, which provides high efficiency and
/// ensures that fast data bursts (such as arrow key escape sequences) are
/// captured correctly.
pub struct Usart<'a> {
    pub registers: StaticRef<UsartRegisters>,
    dma: OptionalCell<&'a Dma>,
    dma_channel_tx: Cell<Option<ChannelId>>,
    dma_channel_rx: Cell<Option<ChannelId>>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    rx_buffer: TakeCell<'static, [u8]>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    rx_len: Cell<usize>,
}

impl<'a> Usart<'a> {
    pub const fn new(base: StaticRef<UsartRegisters>) -> Self {
        Self {
            registers: base,
            dma: OptionalCell::empty(),
            dma_channel_tx: Cell::new(None),
            dma_channel_rx: Cell::new(None),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            rx_len: Cell::new(0),
        }
    }

    /// Associates a DMA controller and channels with the USART driver.
    pub fn set_dma(
        usart: &'static Self,
        dma: &'a Dma,
        tx_channel: ChannelId,
        rx_channel: ChannelId,
    ) {
        usart.dma.set(dma);
        usart.dma_channel_tx.set(Some(tx_channel));
        usart.dma_channel_rx.set(Some(rx_channel));
        dma.set_client(tx_channel, usart);
        dma.set_client(rx_channel, usart);
    }

    /// Hardware interrupt handler for the USART.
    ///
    /// This handles non-DMA events like errors. DMA-specific completions
    /// are handled in `handle_dma_interrupt`.
    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;
        let isr = regs.isr.get();

        // Clear any error flags (Parity, Framing, Noise, Overrun).
        if (isr & 0x0F) != 0 {
            regs.icr.set(0x0F);
        }
    }

    /// Handles completion interrupts from the GPDMA controller.
    ///
    /// This is called when a DMA transfer for either Transmit or Receive
    /// has finished.
    pub fn handle_dma_interrupt(&self, is_tx: bool) {
        if is_tx {
            self.dma.map(|dma| {
                if let Some(ch) = self.dma_channel_tx.get() {
                    dma.clear_interrupt(ch);
                }
            });
            self.registers.cr3.modify(CR3::DMAT::CLEAR);
            if let Some(buf) = self.tx_buffer.take() {
                let len = self.tx_len.get();
                self.tx_client.map(move |client| {
                    client.transmitted_buffer(buf, len, Ok(()));
                });
            }
        } else {
            self.dma.map(|dma| {
                if let Some(ch) = self.dma_channel_rx.get() {
                    dma.clear_interrupt(ch);
                }
            });
            self.registers.cr3.modify(CR3::DMAR::CLEAR);
            if let Some(buf) = self.rx_buffer.take() {
                let len = self.rx_len.get();
                self.rx_client.map(move |client| {
                    client.received_buffer(buf, len, Ok(()), uart::Error::None);
                });
            }
        }
    }

    /// Synchronous (Blocking) send.
    /// ONLY for use in the Panic handler when DMA is unavailable.
    pub fn transmit_byte(&self, byte: u8) {
        let regs = &*self.registers;

        // 1. Disable DMA transmitter temporarily if it was enabled
        // so that manual writes to TDR are processed correctly.
        let dmat_enabled = regs.cr3.is_set(CR3::DMAT);
        if dmat_enabled {
            regs.cr3.modify(CR3::DMAT::CLEAR);
        }

        // 2. Wait until TXE (Transmit Data Register Empty) is set
        while !regs.isr.is_set(ISR::TXE) {}
        regs.tdr.set(byte as u32);

        // 3. Wait for Transmission Complete before potentially re-enabling DMA
        while !regs.isr.is_set(ISR::TC) {}

        // 4. Restore DMA state
        if dmat_enabled {
            regs.cr3.modify(CR3::DMAT::SET);
        }
    }
}

impl crate::dma::DmaClient for Usart<'_> {
    fn transfer_done(&self, channel: ChannelId) {
        if let Some(tx_ch) = self.dma_channel_tx.get() {
            if channel == tx_ch {
                self.handle_dma_interrupt(true);
                return;
            }
        }
        if let Some(rx_ch) = self.dma_channel_rx.get() {
            if channel == rx_ch {
                self.handle_dma_interrupt(false);
            }
        }
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

        let Some(dma) = self.dma.get() else {
            return Err((kernel::ErrorCode::OFF, tx_buffer));
        };

        self.tx_buffer.replace(tx_buffer);
        self.tx_len.set(tx_len);

        self.tx_buffer.map(|buf| {
            if let Some(ch) = self.dma_channel_tx.get() {
                dma.setup(
                    ch,
                    DmaPeripheral::Usart1Tx,
                    buf.as_ptr() as u32,
                    tx_len as u32,
                );
                self.registers.cr3.modify(CR3::DMAT::SET);
            }
        });

        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.cr3.modify(CR3::DMAT::CLEAR);
        if let Some(buf) = self.tx_buffer.take() {
            self.tx_client.map(move |client| {
                client.transmitted_buffer(buf, 0, Err(kernel::ErrorCode::CANCEL));
            });
            Ok(())
        } else {
            Err(kernel::ErrorCode::OFF)
        }
    }

    fn transmit_word(&self, _word: u32) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }
}

impl uart::Configure for Usart<'_> {
    fn configure(&self, _params: uart::Parameters) -> Result<(), kernel::ErrorCode> {
        let regs = &*self.registers;
        regs.cr1.modify(CR1::UE::CLEAR);
        regs.presc.set(0);
        regs.brr.set(35);
        regs.icr.set(0x3F);

        // Enable transmitter, receiver, and USART
        regs.cr1.write(CR1::TE::SET + CR1::RE::SET + CR1::UE::SET);

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
        rx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        if self.rx_buffer.is_some() {
            return Err((kernel::ErrorCode::BUSY, rx_buffer));
        }

        let Some(dma) = self.dma.get() else {
            return Err((kernel::ErrorCode::OFF, rx_buffer));
        };

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);

        self.rx_buffer.map(|buf| {
            if let Some(ch) = self.dma_channel_rx.get() {
                dma.setup(
                    ch,
                    DmaPeripheral::Usart1Rx,
                    buf.as_ptr() as u32,
                    rx_len as u32,
                );
                self.registers.cr3.modify(CR3::DMAR::SET);
            }
        });

        Ok(())
    }

    fn receive_abort(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.cr3.modify(CR3::DMAR::CLEAR);
        if let Some(buf) = self.rx_buffer.take() {
            self.rx_client.map(move |client| {
                client.received_buffer(
                    buf,
                    0,
                    Err(kernel::ErrorCode::CANCEL),
                    uart::Error::Aborted,
                );
            });
            Ok(())
        } else {
            Err(kernel::ErrorCode::OFF)
        }
    }

    fn receive_word(&self) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }
}
