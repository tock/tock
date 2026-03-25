// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use crate::scb_registers::*;
use core::cell::Cell;
use core::num::NonZeroUsize;
use kernel::errorcode::ErrorCode;
use kernel::hil::uart::{self, Configure, Receive, ReceiveClient, Transmit, TransmitClient};
use kernel::utilities::StaticRef;
use kernel::utilities::{
    cells::{OptionalCell, TakeCell},
    registers::interfaces::{ReadWriteable, Readable, Writeable},
};

pub struct Scb<'a> {
    registers: StaticRef<ScbRegisters>,

    tx_client: OptionalCell<&'a dyn TransmitClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_length: OptionalCell<NonZeroUsize>,
    tx_position: Cell<usize>,

    rx_client: OptionalCell<&'a dyn ReceiveClient>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_length: OptionalCell<NonZeroUsize>,
    rx_position: Cell<usize>,
}

impl Scb<'_> {
    pub const fn new() -> Self {
        Self {
            registers: SCB3_BASE,

            tx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_length: OptionalCell::empty(),
            tx_position: Cell::new(0),

            rx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            rx_length: OptionalCell::empty(),
            rx_position: Cell::new(0),
        }
    }

    pub fn enable_tx_interrupts(&self) {
        self.registers
            .intr_tx_mask
            .modify(INTR_TX_MASK::UART_DONE::SET);
    }

    pub fn disable_tx_interrupts(&self) {
        self.registers
            .intr_tx_mask
            .modify(INTR_TX_MASK::UART_DONE::CLEAR);
    }

    pub fn enable_rx_interrupts(&self) {
        self.registers
            .intr_rx_mask
            .modify(INTR_RX_MASK::NOT_EMPTY::SET);
    }

    pub fn disable_rx_interrupts(&self) {
        self.registers
            .intr_rx_mask
            .modify(INTR_RX_MASK::NOT_EMPTY::CLEAR);
    }

    pub(crate) fn handle_interrupt(&self) {
        if self.registers.intr_tx.is_set(INTR_TX::UART_DONE) {
            self.disable_tx_interrupts();
            self.registers.intr_tx.modify(INTR_TX::UART_DONE::SET);
            // SAFETY: When a transmit is started, length is set to a non-zero value.
            if self.tx_length.get().is_none() {
                return;
            }
            let tx_length = self.tx_length.get().unwrap().get();
            if tx_length == self.tx_position.get() + 1 {
                self.tx_length.clear();
                // SAFETY: When a transmit is started, a buffer is passed.
                self.tx_client.map(|client| {
                    client.transmitted_buffer(self.tx_buffer.take().unwrap(), tx_length, Ok(()))
                });
            } else {
                let current_position = self.tx_position.get();
                // SAFETY: Because of the if condition, current_position + 1 < buffer.len().
                self.tx_buffer.map(|buffer| {
                    self.registers.tx_fifo_wr.write(
                        TX_FIFO_WR::DATA.val(*buffer.get(current_position + 1).unwrap() as u32),
                    )
                });
                self.tx_position.set(current_position + 1);
                self.enable_tx_interrupts();
            }
        }
        if self.registers.intr_rx.is_set(INTR_RX::NOT_EMPTY) {
            let byte = self.registers.rx_fifo_rd.read(RX_FIFO_RD::DATA) as u8;
            // The caller must ensure that the FIFO buffer is empty before clearing the interrupt.
            self.registers.intr_rx.modify(INTR_RX::NOT_EMPTY::SET);
            // If no rx_buffer is set, then no reception is pending. Simply discard the received
            // byte.
            if let Some(rx_buffer) = self.rx_buffer.take() {
                let mut current_position = self.rx_position.get();
                rx_buffer[current_position] = byte;
                current_position += 1;
                // SAFETY: When a read is started, rx_length is set to a non-zero value.
                let rx_length = self.rx_length.get().unwrap().get();
                if current_position == rx_length {
                    self.rx_length.clear();
                    self.rx_client.map(|client| {
                        client.received_buffer(
                            rx_buffer,
                            rx_length,
                            Ok(()),
                            kernel::hil::uart::Error::None,
                        )
                    });
                } else {
                    self.rx_position.set(current_position);
                    self.rx_buffer.replace(rx_buffer);
                }
            }
        }
    }

    pub fn set_standard_uart_mode(&self) {
        self.registers
            .ctrl
            .modify(CTRL::MODE::UniversalAsynchronousReceiverTransmitterUARTMode);
        self.registers
            .ctrl
            .modify(CTRL::OVS.val(14) + CTRL::EC_AM_MODE.val(0) + CTRL::EC_OP_MODE.val(0));
        self.registers
            .uart_ctrl
            .modify(UART_CTRL::MODE::StandardUARTSubmode);
        self.registers
            .uart_rx_ctrl
            .modify(UART_RX_CTRL::MP_MODE::CLEAR + UART_RX_CTRL::LIN_MODE::CLEAR);

        self.set_uart_sync();
    }

    pub fn enable_scb(&self) {
        self.registers.ctrl.modify(CTRL::ENABLED::SET);
    }

    pub fn disable_scb(&self) {
        self.registers.ctrl.modify(CTRL::ENABLED::CLEAR);
    }

    fn set_uart_sync(&self) {
        self.registers.ctrl.modify(CTRL::BYTE_MODE::SET);
        self.registers
            .tx_ctrl
            .modify(TX_CTRL::DATA_WIDTH.val(7) + TX_CTRL::MSB_FIRST::CLEAR);

        self.registers
            .rx_ctrl
            .modify(RX_CTRL::DATA_WIDTH.val(7) + RX_CTRL::MSB_FIRST::CLEAR);

        self.registers.tx_fifo_wr.write(TX_FIFO_WR::DATA.val(0));

        self.registers
            .tx_fifo_ctrl
            .modify(TX_FIFO_CTRL::TRIGGER_LEVEL.val(1));
        self.registers.tx_fifo_ctrl.modify(TX_FIFO_CTRL::CLEAR::SET);
        while !self.uart_is_transmitter_done() {}
        self.registers
            .tx_fifo_ctrl
            .modify(TX_FIFO_CTRL::CLEAR::CLEAR);

        self.registers
            .rx_fifo_ctrl
            .modify(RX_FIFO_CTRL::TRIGGER_LEVEL.val(1));
        self.registers.rx_fifo_ctrl.modify(RX_FIFO_CTRL::CLEAR::SET);
        self.registers
            .rx_fifo_ctrl
            .modify(RX_FIFO_CTRL::CLEAR::CLEAR);

        self.registers
            .uart_tx_ctrl
            .modify(UART_TX_CTRL::PARITY::CLEAR);
        self.registers
            .uart_tx_ctrl
            .modify(UART_TX_CTRL::STOP_BITS.val(1));

        self.registers
            .uart_rx_ctrl
            .modify(UART_RX_CTRL::PARITY::CLEAR);
        self.registers
            .uart_rx_ctrl
            .modify(UART_RX_CTRL::STOP_BITS.val(1));

        self.registers
            .uart_flow_ctrl
            .modify(UART_FLOW_CTRL::CTS_ENABLED::CLEAR);
    }

    fn uart_is_transmitter_done(&self) -> bool {
        self.registers.tx_fifo_status.read(TX_FIFO_STATUS::SR_VALID) == 0
    }

    pub fn transmit_uart_sync(&self, buffer: &[u8]) {
        for byte in buffer {
            self.registers
                .tx_fifo_wr
                .write(TX_FIFO_WR::DATA.val(*byte as u32));

            while !self.uart_is_transmitter_done() {}
        }
    }

    pub fn transmit_uart_async(
        &self,
        buffer: &'static mut [u8],
        buffer_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.tx_length.is_some() {
            Err((ErrorCode::BUSY, buffer))
        } else if buffer.len() < buffer_len || buffer_len == 0 {
            Err((ErrorCode::SIZE, buffer))
        } else {
            match NonZeroUsize::new(buffer_len) {
                Some(tx_length) => {
                    self.registers
                        .tx_fifo_wr
                        .write(TX_FIFO_WR::DATA.val(*buffer.get(0).unwrap() as u32));
                    self.tx_buffer.put(Some(buffer));
                    self.tx_length.set(tx_length);
                    self.tx_position.set(0);
                    self.enable_tx_interrupts();
                    Ok(())
                }
                None => Err((ErrorCode::SIZE, buffer)),
            }
        }
    }

    pub fn receive_uart_async(
        &self,
        buffer: &'static mut [u8],
        buffer_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.rx_length.is_some() {
            Err((ErrorCode::BUSY, buffer))
        } else if buffer.len() < buffer_len || buffer_len == 0 {
            Err((ErrorCode::SIZE, buffer))
        } else {
            match NonZeroUsize::new(buffer_len) {
                Some(rx_length) => {
                    self.enable_rx_interrupts();
                    self.rx_buffer.put(Some(buffer));
                    self.rx_length.set(rx_length);
                    self.rx_position.set(0);
                    Ok(())
                }
                None => Err((ErrorCode::SIZE, buffer)),
            }
        }
    }
}

impl<'a> Transmit<'a> for Scb<'a> {
    fn set_transmit_client(&self, client: &'a dyn TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (kernel::ErrorCode, &'static mut [u8])> {
        self.transmit_uart_async(tx_buffer, tx_len)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl<'a> Receive<'a> for Scb<'a> {
    fn set_receive_client(&self, client: &'a dyn ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.receive_uart_async(rx_buffer, rx_len)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }
}

impl Configure for Scb<'_> {
    fn configure(&self, params: kernel::hil::uart::Parameters) -> Result<(), ErrorCode> {
        if params.baud_rate != 115200 || params.hw_flow_control {
            Err(ErrorCode::NOSUPPORT)
        } else {
            // Modification of the SCB parameters require it to be disabled.
            if self.registers.ctrl.is_set(CTRL::ENABLED) {
                return Err(ErrorCode::BUSY);
            }
            match params.stop_bits {
                uart::StopBits::One => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::STOP_BITS.val(1));
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::STOP_BITS.val(1));
                }
                uart::StopBits::Two => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::STOP_BITS.val(3));
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::STOP_BITS.val(3));
                }
            }
            match params.parity {
                uart::Parity::None => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::PARITY_ENABLED::CLEAR);
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::PARITY_ENABLED::CLEAR);
                }
                uart::Parity::Odd => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::PARITY_ENABLED::SET + UART_TX_CTRL::PARITY::SET);
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::PARITY_ENABLED::SET + UART_RX_CTRL::PARITY::SET);
                }
                uart::Parity::Even => {
                    self.registers
                        .uart_tx_ctrl
                        .modify(UART_TX_CTRL::PARITY_ENABLED::SET + UART_TX_CTRL::PARITY::CLEAR);
                    self.registers
                        .uart_rx_ctrl
                        .modify(UART_RX_CTRL::PARITY_ENABLED::SET + UART_RX_CTRL::PARITY::CLEAR);
                }
            }
            match params.width {
                uart::Width::Six => {
                    self.registers.tx_ctrl.modify(TX_CTRL::DATA_WIDTH.val(5));
                }
                uart::Width::Seven => {
                    self.registers.tx_ctrl.modify(TX_CTRL::DATA_WIDTH.val(6));
                }
                uart::Width::Eight => {
                    self.registers.tx_ctrl.modify(TX_CTRL::DATA_WIDTH.val(7));
                }
            }
            Ok(())
        }
    }
}
