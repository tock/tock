//! UART driver.

use core::cell::Cell;

use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;

register_structs! {
    pub UartRegisters {
        (0x000 => intr_state: ReadWrite<u32, intr::Register>),
        (0x004 => intr_enable: ReadWrite<u32, intr::Register>),
        (0x008 => intr_test: ReadWrite<u32, intr::Register>),
        /// UART control register
        (0x00c => ctrl: ReadWrite<u32, ctrl::Register>),
        /// UART live status register
        (0x010 => status: ReadOnly<u32, status::Register>),
        /// UART read data)
        (0x014 => rdata: ReadOnly<u32, rdata::Register>),
        /// UART write data
        (0x018 => wdata: WriteOnly<u32, wdata::Register>),
        /// UART FIFO control register")
        (0x01c => fifo_ctrl: ReadWrite<u32, fifo_ctrl::Register>),
        /// UART FIFO status register
        (0x020 => fifo_status: ReadWrite<u32, fifo_status::Register>),
        /// TX pin override control. Gives direct SW control over TX pin state
        (0x024 => ovrd: ReadWrite<u32, ovrd::Register>),
        /// UART oversampled values
        (0x028 => val: ReadWrite<u32, val::Register>),
        /// UART RX timeout control
        (0x02c => timeout_ctrl: ReadWrite<u32, timeout_ctrl::Register>),
        (0x030 => @END),
    }
}

register_bitfields![u32,
    intr [
        tx_watermark OFFSET(0) NUMBITS(1) [],
        rx_watermark OFFSET(1) NUMBITS(1) [],
        tx_empty OFFSET(2) NUMBITS(1) [],
        rx_overflow OFFSET(3) NUMBITS(1) [],
        rx_frame_err OFFSET(4) NUMBITS(1) [],
        rx_break_err OFFSET(5) NUMBITS(1) [],
        rx_timeout OFFSET(6) NUMBITS(1) [],
        rx_parity_err OFFSET(7) NUMBITS(1) []
    ],
    ctrl [
        tx OFFSET(0) NUMBITS(1) [],
        rx OFFSET(1) NUMBITS(1) [],
        nf OFFSET(2) NUMBITS(1) [],
        slpbk OFFSET(4) NUMBITS(1) [],
        llpbk OFFSET(5) NUMBITS(1) [],
        parity_en OFFSET(6) NUMBITS(1) [],
        parity_odd OFFSET(7) NUMBITS(1) [],
        rxblvl OFFSET(8) NUMBITS(2) [],
        nco OFFSET(16) NUMBITS(16) []
    ],
    status [
        txfull OFFSET(0) NUMBITS(1) [],
        rxfull OFFSET(1) NUMBITS(1) [],
        txempty OFFSET(2) NUMBITS(1) [],
        txidle OFFSET(3) NUMBITS(1) [],
        rxidle OFFSET(4) NUMBITS(1) [],
        rxempty OFFSET(5) NUMBITS(1) []
    ],
    rdata [
        data OFFSET(0) NUMBITS(8) []
    ],
    wdata [
        data OFFSET(0) NUMBITS(8) []
    ],
    fifo_ctrl [
        rxrst OFFSET(0) NUMBITS(1) [],
        txrst OFFSET(1) NUMBITS(1) [],
        rxilvl OFFSET(2) NUMBITS(2) [],
        txilvl OFFSET(5) NUMBITS(2) []
    ],
    fifo_status [
        txlvl OFFSET(0) NUMBITS(5) [],
        rxlvl OFFSET(16) NUMBITS(5) []
    ],
    ovrd [
        txen OFFSET(0) NUMBITS(1) [],
        txval OFFSET(1) NUMBITS(1) []
    ],
    val [
        rx OFFSET(0) NUMBITS(16) []
    ],
    timeout_ctrl [
        val OFFSET(0) NUMBITS(23) [],
        en OFFSET(31) NUMBITS(1) []
    ]
];

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
    clock_frequency: u32,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UartParams {
    pub baud_rate: u32,
}

impl Uart<'a> {
    pub const fn new(base: StaticRef<UartRegisters>, clock_frequency: u32) -> Uart<'a> {
        Uart {
            registers: base,
            clock_frequency: clock_frequency,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
        }
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = self.registers;
        let uart_ctrl_nco = ((baud_rate as u64) << 20) / self.clock_frequency as u64;

        regs.ctrl
            .write(ctrl::nco.val((uart_ctrl_nco & 0xffff) as u32));
        regs.ctrl.modify(ctrl::tx::SET + ctrl::rx::SET);

        regs.fifo_ctrl
            .write(fifo_ctrl::rxrst::SET + fifo_ctrl::txrst::SET);
    }

    fn enable_tx_interrupt(&self) {
        let regs = self.registers;

        regs.intr_enable.modify(intr::tx_empty::SET);
    }

    fn disable_tx_interrupt(&self) {
        let regs = self.registers;

        regs.intr_enable.modify(intr::tx_empty::CLEAR);
        // Clear the interrupt bit (by writing 1), if it happens to be set
        regs.intr_state.write(intr::tx_empty::SET);
    }

    fn enable_rx_interrupt(&self) {
        let regs = self.registers;

        // Generate an interrupt if we get any value in the RX buffer
        regs.intr_enable.modify(intr::rx_watermark::SET);
        regs.fifo_ctrl.write(fifo_ctrl::rxilvl.val(0 as u32));
    }

    fn disable_rx_interrupt(&self) {
        let regs = self.registers;

        // Generate an interrupt if we get any value in the RX buffer
        regs.intr_enable.modify(intr::rx_watermark::CLEAR);

        // Clear the interrupt bit (by writing 1), if it happens to be set
        regs.intr_state.write(intr::rx_watermark::SET);
    }

    fn tx_progress(&self) {
        let regs = self.registers;
        let idx = self.tx_index.get();
        let len = self.tx_len.get();

        if idx < len {
            self.tx_buffer.map(|tx_buf| {
                let tx_len = len - idx;

                for i in 0..tx_len {
                    if regs.status.is_set(status::txfull) {
                        break;
                    }
                    let tx_idx = idx + i;
                    regs.wdata.write(wdata::data.val(tx_buf[tx_idx] as u32));
                    self.tx_index.set(tx_idx + 1)
                }
            });

            // If the FIFO started empty above, when the first character was written, it would have
            // been immediately dispatched to the shift register, latching the TX_EMPTY state.
            // That should be cleared before enabling the interrupt, so a subsequent empty state
            // will latch the state transition again.
            regs.intr_state.write(intr::tx_empty::SET);

            // With data queued, (re)enable the TX interrupt if the FIFO is not empty
            if !regs.status.is_set(status::txempty) {
                self.enable_tx_interrupt();
                if !regs.status.is_set(status::txempty) {
                    // Assuming the FIFO stayed non-empty while the interrupt was being enabled,
                    // our work here is done
                    return;
                } else {
                    // Otherwise, turn off the interrupt and handle below
                    self.disable_tx_interrupt();
                }
            }
        }

        if self.tx_index.get() == self.tx_len.get() {
            self.tx_client.map(|client| {
                self.tx_buffer.take().map(|tx_buf| {
                    client.transmitted_buffer(tx_buf, self.tx_len.get(), ReturnCode::SUCCESS);
                });
            });
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let intrs = regs.intr_state.extract();

        if intrs.is_set(intr::tx_empty) {
            self.disable_tx_interrupt();
            self.tx_progress();
        } else if intrs.is_set(intr::rx_watermark) {
            self.disable_rx_interrupt();
            // TODO: real RX processing
        }
    }

    pub fn transmit_sync(&self, bytes: &[u8]) {
        let regs = self.registers;
        for b in bytes.iter() {
            while regs.status.is_set(status::txfull) {}
            regs.wdata.write(wdata::data.val(*b as u32));
        }
    }
}

impl hil::uart::UartData<'a> for Uart<'a> {}
impl hil::uart::Uart<'a> for Uart<'a> {}

impl hil::uart::Configure for Uart<'a> {
    fn configure(&self, params: hil::uart::Parameters) -> ReturnCode {
        let regs = self.registers;
        // We can set the baud rate.
        self.set_baud_rate(params.baud_rate);

        regs.fifo_ctrl
            .write(fifo_ctrl::rxrst::SET + fifo_ctrl::txrst::SET);

        // Disable all interrupts for now
        regs.intr_enable.set(0 as u32);

        ReturnCode::SUCCESS
    }
}

impl hil::uart::Transmit<'a> for Uart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if tx_len == 0 {
            (ReturnCode::ESIZE, Some(tx_data))
        } else if self.tx_buffer.is_some() {
            (ReturnCode::EBUSY, Some(tx_data))
        } else {
            // Save the buffer so we can keep sending it.
            self.tx_buffer.replace(tx_data);
            self.tx_len.set(tx_len);
            self.tx_index.set(0);

            self.tx_progress();
            (ReturnCode::SUCCESS, None)
        }
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }
}

/* UART receive is not implemented yet, mostly due to a lack of tests avaliable */
impl hil::uart::Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        _rx_buffer: &'static mut [u8],
        _rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        self.enable_rx_interrupt();

        (ReturnCode::FAIL, None)
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}
