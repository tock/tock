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
        tx_overflow OFFSET(2) NUMBITS(1) [],
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
        data OFFSET(0) NUMBITS(7) []
    ],
    wdata [
        data OFFSET(0) NUMBITS(7) []
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
    buffer: TakeCell<'static, [u8]>,
    len: Cell<usize>,
    index: Cell<usize>,
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
            buffer: TakeCell::empty(),
            len: Cell::new(0),
            index: Cell::new(0),
        }
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = self.registers;
        let uart_ctrl_nco = ((baud_rate as u64) << 20) / self.clock_frequency as u64;

        regs.ctrl
            .write(ctrl::nco.val((uart_ctrl_nco & 0xffff) as u32));
        regs.ctrl.modify(ctrl::tx.val(1 as u32));
        regs.ctrl.modify(ctrl::rx.val(1 as u32));

        regs.fifo_ctrl
            .write(fifo_ctrl::rxrst.val(1 as u32) + fifo_ctrl::txrst.val(1 as u32));
    }

    fn enable_tx_interrupt(&self) {
        let regs = self.registers;

        // Set watermark to 1 char
        regs.fifo_ctrl.write(fifo_ctrl::txilvl.val(0 as u32));

        regs.intr_enable
            .modify(intr::tx_watermark.val(1 as u32) + intr::tx_overflow.val(1 as u32));
    }

    fn disable_tx_interrupt(&self) {
        let regs = self.registers;

        regs.intr_enable
            .modify(intr::tx_watermark.val(0 as u32) + intr::tx_overflow.val(0 as u32));
    }

    fn enable_rx_interrupt(&self) {
        let regs = self.registers;

        // Generate an interrupt if we get any value in the RX buffer
        regs.fifo_ctrl.write(fifo_ctrl::rxilvl.val(0 as u32));

        regs.intr_enable
            .modify(intr::rx_watermark.val(1 as u32) + intr::rx_overflow.val(1 as u32));
    }

    fn disable_rx_interrupt(&self) {
        let regs = self.registers;

        regs.intr_enable
            .modify(intr::rx_watermark.val(0 as u32) + intr::rx_overflow.val(0 as u32));
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;

        // Get a copy so we can check each interrupt flag in the register.
        let pending_interrupts = regs.intr_state.extract();

        // Determine why an interrupt occurred.
        if pending_interrupts.is_set(intr::tx_watermark) {
            // Got a TX interrupt which means the number of bytes in the FIFO
            // has fallen to zero. If there is more to send do that, otherwise
            // send a callback to the client.
            if self.len.get() == self.index.get() {
                // We are done.
                self.disable_tx_interrupt();

                // Signal client write done
                self.tx_client.map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.transmitted_buffer(buffer, self.len.get(), ReturnCode::SUCCESS);
                    });
                });
            } else {
                // More to send. Fill the buffer until it is full.
                self.buffer.map(|buffer| {
                    for i in self.index.get()..self.len.get() {
                        // Write the byte from the array to the tx register.
                        regs.wdata.write(wdata::data.val(buffer[i] as u32));
                        self.index.set(i + 1);
                        // Check if the buffer is full
                        if regs.status.is_set(status::txfull) {
                            // If it is, break and wait for the TX interrupt.
                            break;
                        }
                    }
                });
            }
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

        regs.fifo_ctrl.write(fifo_ctrl::rxrst.val(1 as u32));
        regs.fifo_ctrl.modify(fifo_ctrl::txrst.val(1 as u32));

        self.disable_tx_interrupt();
        self.enable_rx_interrupt();

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
        let regs = self.registers;

        if tx_len == 0 {
            return (ReturnCode::ESIZE, Some(tx_data));
        }

        // Fill the TX buffer until it reports full.
        for i in 0..tx_len {
            // Write the byte from the array to the tx register.
            regs.wdata.write(wdata::data.val(tx_data[i] as u32));
            self.index.set(i + 1);

            // Check if the buffer is full and wait until it isn't
            if regs.status.is_set(status::txfull) {
                break;
            }
        }

        // Save the buffer so we can keep sending it.
        self.buffer.replace(tx_data);
        self.len.set(tx_len);

        self.enable_tx_interrupt();

        (ReturnCode::SUCCESS, None)
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
        self.disable_rx_interrupt();

        (ReturnCode::FAIL, None)
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}
