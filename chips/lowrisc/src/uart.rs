//! UART driver.

use core::cell::Cell;
use kernel::ErrorCode;

use kernel::hil;
use kernel::hil::uart;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;

#[allow(clippy::wildcard_imports)]
use crate::registers::uart_regs::*;

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
    clock_frequency: u32,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UartParams {
    pub baud_rate: u32,
}

impl<'a> Uart<'a> {
    pub const fn new(base: StaticRef<UartRegisters>, clock_frequency: u32) -> Uart<'a> {
        Uart {
            registers: base,
            clock_frequency: clock_frequency,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
        }
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = self.registers;
        let uart_ctrl_nco = ((baud_rate as u64) << 20) / self.clock_frequency as u64;

        regs.ctrl
            .write(CTRL::NCO.val((uart_ctrl_nco & 0xffff) as u32));
        regs.ctrl.modify(CTRL::TX::SET + CTRL::RX::SET);

        regs.fifo_ctrl
            .write(FIFO_CTRL::RXRST::SET + FIFO_CTRL::TXRST::SET);
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
        regs.fifo_ctrl.write(FIFO_CTRL::RXILVL.val(0 as u32));
    }

    fn disable_rx_interrupt(&self) {
        let regs = self.registers;

        // Generate an interrupt if we get any value in the RX buffer
        regs.intr_enable.modify(INTR::RX_WATERMARK::CLEAR);

        // Clear the interrupt bit (by writing 1), if it happens to be set
        regs.intr_state.write(INTR::RX_WATERMARK::SET);
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

            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|rx_buf| {
                    let mut len = 0;
                    let mut return_code = Ok(());

                    for i in 0..self.rx_len.get() {
                        rx_buf[i] = regs.rdata.get() as u8;
                        len = i + 1;

                        if regs.status.is_set(STATUS::RXEMPTY) {
                            /* RX is empty */
                            return_code = Err(ErrorCode::SIZE);
                            break;
                        }
                    }

                    client.received_buffer(rx_buf, len, return_code, uart::Error::None);
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
        self.set_baud_rate(params.baud_rate);

        regs.fifo_ctrl
            .write(FIFO_CTRL::RXRST::SET + FIFO_CTRL::TXRST::SET);

        // Disable all interrupts for now
        regs.intr_enable.set(0 as u32);

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

        Ok(())
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}
