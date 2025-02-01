// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! UART driver.

use core::cell::Cell;
use kernel::ErrorCode;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::hil::uart;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;

use crate::registers::uart_regs::UartRegisters;
use crate::registers::uart_regs::{CTRL, FIFO_CTRL, INTR, STATUS, TIMEOUT_CTRL, WDATA};

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
    clock_frequency: u32,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    rx_deferred_call: DeferredCall,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>,
    rx_timeout: Cell<u8>,
}

#[derive(Copy, Clone)]
pub struct UartParams {
    pub baud_rate: u32,
    pub parity: uart::Parity,
}

/// Compute a / b, rounding such that the result is within 2.5% error of the floating point result.
fn div_round_bounded(a: u64, b: u64) -> Result<u64, ErrorCode> {
    let q = a / b;

    if 39 * a <= 40 * b * q {
        Ok(q)
    } else if 40 * b * (q + 1) <= 41 * a {
        Ok(q + 1)
    } else {
        Err(ErrorCode::INVAL)
    }
}

impl<'a> Uart<'a> {
    pub fn new(base: StaticRef<UartRegisters>, clock_frequency: u32) -> Uart<'a> {
        Uart {
            registers: base,
            clock_frequency,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            rx_deferred_call: DeferredCall::new(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
            rx_timeout: Cell::new(0),
        }
    }

    fn set_baud_rate(&self, baud_rate: u32) -> Result<(), ErrorCode> {
        const NCO_BITS: u32 = u32::count_ones(CTRL::NCO.mask);

        let regs = self.registers;
        let baud_adj = (baud_rate as u64) << (NCO_BITS + 4);
        let freq_clk = self.clock_frequency as u64;
        let uart_ctrl_nco = div_round_bounded(baud_adj, freq_clk)?;

        regs.ctrl
            .write(CTRL::NCO.val((uart_ctrl_nco & 0xffff) as u32));
        regs.ctrl.modify(CTRL::TX::SET + CTRL::RX::SET);

        regs.fifo_ctrl
            .write(FIFO_CTRL::RXRST::SET + FIFO_CTRL::TXRST::SET);

        Ok(())
    }

    fn enable_tx_interrupt(&self) {
        let regs = self.registers;

        regs.intr_enable.modify(INTR::TX_EMPTY::SET);
    }

    fn disable_tx_interrupt(&self) {
        let regs = self.registers;

        regs.intr_enable.modify(INTR::TX_EMPTY::CLEAR);
        // Clear the interrupt bit (by writing 1), if it happens to be set
        regs.intr_state.write(INTR::TX_EMPTY::SET);
    }

    fn enable_rx_interrupt(&self) {
        let regs = self.registers;

        // Generate an interrupt if we get any value in the RX buffer
        regs.intr_enable.modify(INTR::RX_WATERMARK::SET);
        regs.fifo_ctrl.write(FIFO_CTRL::RXILVL.val(0_u32));

        // In cases where the RX FIFO isn't empty the edge-triggered watermark will never trigger.
        // If there is already data pending, set a deferred call to read the data instead.
        if !regs.status.is_set(STATUS::RXEMPTY) {
            self.rx_deferred_call.set();
            self.disable_rx_interrupt();
        }
    }

    fn disable_rx_interrupt(&self) {
        let regs = self.registers;

        // Generate an interrupt if we get any value in the RX buffer
        regs.intr_enable.modify(INTR::RX_WATERMARK::CLEAR);

        // Clear the interrupt bit (by writing 1), if it happens to be set
        regs.intr_state.write(INTR::RX_WATERMARK::SET);
    }

    fn enable_rx_timeout(&self, interbyte_timeout: u8) {
        let regs = self.registers;

        // Program the timeout value
        regs.timeout_ctrl
            .write(TIMEOUT_CTRL::VAL.val(interbyte_timeout as u32));

        // Enable RX timeout feature
        regs.timeout_ctrl.write(TIMEOUT_CTRL::EN::SET);

        // Enable RX timeout interrupt
        regs.intr_enable.write(INTR::RX_TIMEOUT::SET);
    }

    fn disable_rx_timeout(&self) {
        let regs = self.registers;

        // Disable RX timeout feature
        regs.timeout_ctrl.modify(TIMEOUT_CTRL::EN::CLEAR);

        // Disable RX timeout interrupt
        regs.intr_enable.modify(INTR::RX_TIMEOUT::CLEAR);

        // Clear the interrupt bit (by writing 1), if it happens to be set
        regs.intr_state.write(INTR::RX_TIMEOUT::SET);
    }

    fn tx_progress(&self) {
        let regs = self.registers;
        let idx = self.tx_index.get();
        let len = self.tx_len.get();

        if idx < len {
            // If we are going to transmit anything, we first need to enable the
            // TX interrupt. This ensures that we will get an interrupt, where
            // we can either call the callback from, or continue transmitting
            // bytes.
            self.enable_tx_interrupt();

            // Read from the transmit buffer and send bytes to the UART hardware
            // until either the buffer is empty or the UART hardware is full.
            self.tx_buffer.map(|tx_buf| {
                let tx_len = len - idx;

                for i in 0..tx_len {
                    if regs.status.is_set(STATUS::TXFULL) {
                        break;
                    }
                    let tx_idx = idx + i;
                    regs.wdata.write(WDATA::WDATA.val(tx_buf[tx_idx] as u32));
                    self.tx_index.set(tx_idx + 1)
                }
            });
        }
    }

    fn consume_rx(&self) {
        let regs = self.registers;

        self.rx_client.map(|client| {
            self.rx_buffer.take().map(|rx_buf| {
                let mut len = 0;
                let mut return_code = Ok(());

                for i in self.rx_index.get()..self.rx_len.get() {
                    if regs.status.is_set(STATUS::RXEMPTY) {
                        /* RX is empty */

                        // If this was kicked off by `receive_automatic()` then we can reenable
                        // interupts and wait for either the rest of the data or for the timeout.
                        let rx_timeout = self.rx_timeout.get();
                        if rx_timeout > 0 {
                            self.rx_index.set(i);
                            self.enable_rx_timeout(rx_timeout);
                            self.enable_rx_interrupt();
                            return;
                        } else {
                            return_code = Err(ErrorCode::SIZE);
                            break;
                        }
                    }

                    rx_buf[i] = regs.rdata.get() as u8;
                    len = i + 1;
                }

                client.received_buffer(rx_buf, len, return_code, uart::Error::None);
            });
        });
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let intrs = regs.intr_state.extract();

        if intrs.is_set(INTR::TX_EMPTY) {
            self.disable_tx_interrupt();

            if self.tx_index.get() == self.tx_len.get() {
                // We sent everything to the UART hardware, now from an
                // interrupt callback we can issue the callback.
                self.tx_client.map(|client| {
                    self.tx_buffer.take().map(|tx_buf| {
                        client.transmitted_buffer(tx_buf, self.tx_len.get(), Ok(()));
                    });
                });
            } else {
                // We have more to transmit, so continue in tx_progress().
                self.tx_progress();
            }
        } else if intrs.is_set(INTR::RX_WATERMARK) {
            self.disable_rx_interrupt();
            self.consume_rx();
        } else if intrs.is_set(INTR::RX_TIMEOUT) {
            self.disable_rx_interrupt();
            self.disable_rx_timeout();

            // On timeout return whatever is in the buffer to the client.
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|rx_buf| {
                    client.received_buffer(
                        rx_buf,
                        self.rx_index.get() + 1,
                        Err(kernel::ErrorCode::SIZE),
                        uart::Error::None,
                    );
                })
            });
        } else if intrs.is_set(INTR::TX_WATERMARK) {
            // TODO: Additional logic or notification related to the watermark.
        } else if intrs.is_set(INTR::RX_OVERFLOW) {
            self.disable_rx_interrupt();
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|rx_buf| {
                    client.received_buffer(
                        rx_buf,
                        self.rx_index.get(),
                        Err(kernel::ErrorCode::FAIL),
                        uart::Error::OverrunError,
                    );
                });
            });
        } else if intrs.is_set(INTR::RX_FRAME_ERR) {
            self.disable_rx_interrupt();
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|rx_buf| {
                    client.received_buffer(
                        rx_buf,
                        self.rx_index.get(),
                        Err(kernel::ErrorCode::FAIL),
                        uart::Error::FramingError,
                    );
                });
            });
        } else if intrs.is_set(INTR::RX_BREAK_ERR) {
            self.disable_rx_interrupt();
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|rx_buf| {
                    client.received_buffer(
                        rx_buf,
                        self.rx_index.get(),
                        Err(kernel::ErrorCode::FAIL),
                        uart::Error::BreakError,
                    );
                });
            });
        } else if intrs.is_set(INTR::RX_PARITY_ERR) {
            self.disable_rx_interrupt();
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|rx_buf| {
                    client.received_buffer(
                        rx_buf,
                        self.rx_index.get(),
                        Err(kernel::ErrorCode::FAIL),
                        uart::Error::ParityError,
                    );
                });
            });
        }
    }

    pub fn transmit_sync(&self, bytes: &[u8]) {
        let regs = self.registers;
        for b in bytes.iter() {
            while regs.status.is_set(STATUS::TXFULL) {}
            regs.wdata.write(WDATA::WDATA.val(*b as u32));
        }
    }
}

impl hil::uart::Configure for Uart<'_> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        let regs = self.registers;
        // We can set the baud rate.
        self.set_baud_rate(params.baud_rate)?;

        match params.parity {
            uart::Parity::Even => regs
                .ctrl
                .modify(CTRL::PARITY_EN::SET + CTRL::PARITY_ODD::CLEAR),
            uart::Parity::Odd => regs
                .ctrl
                .modify(CTRL::PARITY_EN::SET + CTRL::PARITY_ODD::SET),
            uart::Parity::None => regs
                .ctrl
                .modify(CTRL::PARITY_EN::CLEAR + CTRL::PARITY_ODD::CLEAR),
        }

        regs.fifo_ctrl
            .write(FIFO_CTRL::RXRST::SET + FIFO_CTRL::TXRST::SET);

        // Disable all interrupts for now
        regs.intr_enable.set(0_u32);

        Ok(())
    }
}

impl<'a> hil::uart::Transmit<'a> for Uart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len == 0 || tx_len > tx_data.len() {
            Err((ErrorCode::SIZE, tx_data))
        } else if self.tx_buffer.is_some() {
            Err((ErrorCode::BUSY, tx_data))
        } else {
            // Save the buffer so we can keep sending it.
            self.tx_buffer.replace(tx_data);
            self.tx_len.set(tx_len);
            self.tx_index.set(0);

            self.tx_progress();
            Ok(())
        }
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

/* UART receive is not implemented yet, mostly due to a lack of tests avaliable */
impl<'a> hil::uart::Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if rx_len == 0 || rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        self.enable_rx_interrupt();

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_timeout.set(0);
        self.rx_index.set(0);

        Ok(())
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> hil::uart::ReceiveAdvanced<'a> for Uart<'a> {
    fn receive_automatic(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        interbyte_timeout: u8,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if rx_len == 0 || rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_timeout.set(interbyte_timeout);
        self.rx_index.set(0);

        Ok(())
    }
}

impl DeferredCallClient for Uart<'_> {
    fn handle_deferred_call(&self) {
        self.consume_rx();
    }

    fn register(&'static self) {
        self.rx_deferred_call.register(self);
    }
}

#[cfg(test)]
mod tests {
    use super::div_round_bounded;
    use kernel::ErrorCode;

    #[test]
    fn test_bounded_division() {
        const TEST_VECTORS: [(u64, u64, Result<u64, ErrorCode>); 10] = [
            (100, 4, Ok(25)),
            (41, 40, Ok(1)),
            (83, 40, Err(ErrorCode::INVAL)),
            (105, 40, Err(ErrorCode::INVAL)),
            (120, 40, Ok(3)),
            (121, 40, Ok(3)),
            (158, 40, Ok(4)),
            (159, 40, Ok(4)),
            (10, 3, Err(ErrorCode::INVAL)),
            (120_795_955_200, 6_000_000, Ok(20132)),
        ];
        for (a, b, expected) in &TEST_VECTORS {
            assert_eq!(div_round_bounded(*a, *b), *expected);
        }
    }
}
