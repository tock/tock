// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use kernel::hil::uart::{self, Configure, Receive, Transmit};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
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
        EIE     OFFSET(0)   NUMBITS(1) []
    ],
    pub ISR [
        /// Read data register not empty
        RXNE OFFSET(5) NUMBITS(1) [],
        /// Transmit data register empty
        TXE  OFFSET(7) NUMBITS(1) []
    ]
];

pub struct Usart<'a> {
    registers: StaticRef<UsartRegisters>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
}

impl<'a> Usart<'a> {
    pub const fn new(base: StaticRef<UsartRegisters>) -> Usart<'a> {
        Usart {
            registers: base,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }

    pub fn transmit_byte(&self, byte: u8) {
        // Wait until TXE (Transmit data register empty) is set
        while !self.registers.isr.is_set(ISR::TXE) {}
        // Write the byte to the TDR register
        self.registers.tdr.set(byte as u32);
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
        // For now, we use our working synchronous loop
        for i in 0..tx_len {
            self.transmit_byte(tx_buffer[i]);
        }

        // Use a separate scope to prevent borrowing issues
        if let Some(client) = self.tx_client.take() {
            client.transmitted_buffer(tx_buffer, tx_len, Ok(()));
            self.tx_client.set(client);
        }
        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }
}

// Implement Configure (Satisfies the compiler)
impl<'a> uart::Configure for Usart<'a> {
    fn configure(&self, _params: uart::Parameters) -> Result<(), kernel::ErrorCode> {
        // We already configured it in main.rs for now.
        // In a full driver, you'd move that logic here.
        Ok(())
    }
}

// Implement Receive (Stub for now)
impl<'a> uart::Receive<'a> for Usart<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        _rx_buffer: &'static mut [u8],
        _rx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        Err((kernel::ErrorCode::NOSUPPORT, _rx_buffer))
    }

    fn receive_abort(&self) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }

    fn receive_word(&self) -> Result<(), kernel::ErrorCode> {
        Err(kernel::ErrorCode::NOSUPPORT)
    }
}
