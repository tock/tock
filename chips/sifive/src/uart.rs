//! UART driver.

use core::cell::Cell;
use kernel::utilities::registers::FieldValue;
use kernel::ErrorCode;

use crate::gpio;
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};

#[repr(C)]
pub struct UartRegisters {
    /// Transmit Data Register
    txdata: ReadWrite<u32, txdata::Register>,
    /// Receive Data Register
    rxdata: ReadWrite<u32, rxdata::Register>,
    /// Transmit Control Register
    txctrl: ReadWrite<u32, txctrl::Register>,
    /// Receive Control Register
    rxctrl: ReadWrite<u32, rxctrl::Register>,
    /// Interrupt Enable Register
    ie: ReadWrite<u32, interrupt::Register>,
    /// Interrupt Pending Register
    ip: ReadOnly<u32, interrupt::Register>,
    /// Baud Rate Divisor Register
    div: ReadWrite<u32, div::Register>,
}

register_bitfields![u32,
    txdata [
        full OFFSET(31) NUMBITS(1) [],
        data OFFSET(0) NUMBITS(8) []
    ],
    rxdata [
        empty OFFSET(31) NUMBITS(1) [],
        data OFFSET(0) NUMBITS(8) []
    ],
    txctrl [
        txcnt OFFSET(16) NUMBITS(3) [],
        nstop OFFSET(1) NUMBITS(1) [
            OneStopBit = 0,
            TwoStopBits = 1
        ],
        txen OFFSET(0) NUMBITS(1) []
    ],
    rxctrl [
        counter OFFSET(16) NUMBITS(3) [],
        enable OFFSET(0) NUMBITS(1) []
    ],
    interrupt [
        rxwm OFFSET(1) NUMBITS(1) [],
        txwm OFFSET(0) NUMBITS(1) []
    ],
    div [
        div OFFSET(0) NUMBITS(16) []
    ]
];

#[derive(Copy, Clone, PartialEq)]
enum UARTStateTX {
    Idle,
    Transmitting,
    AbortRequested,
}

#[derive(Copy, Clone, PartialEq)]
enum UARTStateRX {
    Idle,
    Receiving,
    AbortRequested,
}

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
    clock_frequency: u32,
    stop_bits: Cell<hil::uart::StopBits>,

    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_position: Cell<usize>,
    tx_status: Cell<UARTStateTX>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_position: Cell<usize>,
    rx_status: Cell<UARTStateRX>,

    deferred_call: DeferredCall,
}

#[derive(Copy, Clone)]
pub struct UartParams {
    pub baud_rate: u32,
}

impl<'a> Uart<'a> {
    pub fn new(base: StaticRef<UartRegisters>, clock_frequency: u32) -> Uart<'a> {
        Uart {
            registers: base,
            clock_frequency: clock_frequency,
            stop_bits: Cell::new(hil::uart::StopBits::One),

            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),

            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_position: Cell::new(0),
            tx_status: Cell::new(UARTStateTX::Idle),

            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_position: Cell::new(0),
            rx_status: Cell::new(UARTStateRX::Idle),

            deferred_call: DeferredCall::new(),
        }
    }

    /// Configure GPIO pins for the UART.
    pub fn initialize_gpio_pins(&self, tx: &gpio::GpioPin, rx: &gpio::GpioPin) {
        tx.iof0();
        rx.iof0();
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = self.registers;

        //            f_clk
        // f_baud = ---------
        //           div + 1
        let divisor = (self.clock_frequency / baud_rate) - 1;

        regs.div.write(div::div.val(divisor));
    }

    fn get_stop_bits(&self) -> FieldValue<u32, txctrl::Register> {
        match self.stop_bits.get() {
            hil::uart::StopBits::One => txctrl::nstop::OneStopBit,
            hil::uart::StopBits::Two => txctrl::nstop::TwoStopBits,
        }
    }

    pub fn disable(&self) {
        let regs = self.registers;
        regs.txctrl.modify(txctrl::txen::CLEAR);
        regs.rxctrl.modify(rxctrl::enable::CLEAR);

        self.disable_rx_interrupt();
        self.disable_tx_interrupt();
    }

    fn enable_tx_interrupt(&self) {
        let regs = self.registers;
        regs.ie.modify(interrupt::txwm::SET);
    }

    fn enable_rx_interrupt(&self) {
        let regs = self.registers;
        regs.ie.modify(interrupt::rxwm::SET);
    }

    fn disable_rx_interrupt(&self) {
        let regs = self.registers;
        regs.ie.modify(interrupt::rxwm::CLEAR);
    }

    fn disable_tx_interrupt(&self) {
        let regs = self.registers;
        regs.ie.modify(interrupt::txwm::CLEAR);
    }

    fn uart_is_writable(&self) -> bool {
        !self.registers.txdata.is_set(txdata::full)
    }

    fn tx_progress(&self) {
        while self.uart_is_writable() && self.tx_position.get() < self.tx_len.get() {
            self.tx_buffer.map(|buf| {
                self.registers
                    .txdata
                    .set(buf[self.tx_position.get()].into());
                self.tx_position.replace(self.tx_position.get() + 1);
            });
        }
    }

    fn rx_progress(&self) {
        while self.rx_position.get() < self.rx_len.get() {
            let rxdata_copy = self.registers.rxdata.extract();

            if rxdata_copy.read(rxdata::empty) == 1 {
                break;
            }

            self.rx_buffer.map(|buf| {
                buf[self.rx_position.get()] = rxdata_copy.read(rxdata::data) as u8;
                self.rx_position.replace(self.rx_position.get() + 1);
            });
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;

        // Get a copy so we can check each interrupt flag in the register.
        let pending_interrupts = regs.ip.extract();

        // Determine why an interrupt occurred.
        if self.tx_status.get() == UARTStateTX::Transmitting
            && pending_interrupts.is_set(interrupt::txwm)
        {
            // Got a TX interrupt which means the number of bytes in the FIFO
            // has fallen to zero. If there is more to send do that, otherwise
            // send a callback to the client.

            if self.tx_position.get() == self.tx_len.get() {
                // We are done.
                regs.txctrl.write(txctrl::txen::CLEAR);
                self.disable_tx_interrupt();
                self.tx_status.set(UARTStateTX::Idle);

                // Signal client write is done
                self.tx_client.map(|client| {
                    self.tx_buffer.take().map(|buffer| {
                        client.transmitted_buffer(buffer, self.tx_len.get(), Ok(()));
                    });
                });
            } else {
                self.tx_progress();
            }
        }

        if self.rx_status.get() == UARTStateRX::Receiving
            && pending_interrupts.is_set(interrupt::rxwm)
        {
            self.disable_rx_interrupt();
            // Got a RX interrupt which means the number of bytes in the FIFO
            // is greater than zero. Read them.
            self.rx_progress();

            if self.rx_position.get() == self.rx_len.get() {
                // reception done
                regs.rxctrl.write(rxctrl::enable::CLEAR);
                self.rx_status.replace(UARTStateRX::Idle);

                // Signal client read is done
                self.rx_client.map(|client| {
                    if let Some(buf) = self.rx_buffer.take() {
                        client.received_buffer(
                            buf,
                            self.rx_len.get(),
                            Ok(()),
                            hil::uart::Error::None,
                        );
                    }
                });
            } else {
                self.enable_rx_interrupt();
            }
        }
    }

    pub fn transmit_sync(&self, bytes: &[u8]) {
        let regs = self.registers;

        // Make sure the UART is enabled.
        regs.txctrl
            .write(txctrl::txen::SET + self.get_stop_bits() + txctrl::txcnt.val(1));

        for b in bytes.iter() {
            while regs.txdata.is_set(txdata::full) {}
            regs.txdata.write(txdata::data.val(*b as u32));
        }
    }
}

impl DeferredCallClient for Uart<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self)
    }

    fn handle_deferred_call(&self) {
        if self.tx_status.get() == UARTStateTX::AbortRequested {
            // alert client
            self.tx_client.map(|client| {
                self.tx_buffer.take().map(|buf| {
                    client.transmitted_buffer(buf, self.tx_position.get(), Err(ErrorCode::CANCEL));
                });
            });
            self.tx_status.set(UARTStateTX::Idle);
        }

        if self.rx_status.get() == UARTStateRX::AbortRequested {
            // alert client
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|buf| {
                    client.received_buffer(
                        buf,
                        self.rx_position.get(),
                        Err(ErrorCode::CANCEL),
                        hil::uart::Error::Aborted,
                    );
                });
            });
            self.rx_status.set(UARTStateRX::Idle);
        }
    }
}

impl hil::uart::Configure for Uart<'_> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        // This chip does not support these features.
        if params.parity != hil::uart::Parity::None {
            return Err(ErrorCode::NOSUPPORT);
        }
        if params.hw_flow_control != false {
            return Err(ErrorCode::NOSUPPORT);
        }

        // We can set the baud rate.
        self.set_baud_rate(params.baud_rate);

        // We need to save the stop bits because it is set in the TX register.
        self.stop_bits.set(params.stop_bits);

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
        if self.tx_status.get() != UARTStateTX::Idle {
            Err((ErrorCode::BUSY, tx_buffer))
        } else if tx_len == 0 || tx_len > tx_buffer.len() {
            Err((ErrorCode::SIZE, tx_buffer))
        } else {
            self.tx_status.set(UARTStateTX::Transmitting);

            // Save the buffer so we can keep sending it.
            self.tx_buffer.replace(tx_buffer);
            self.tx_len.set(tx_len);
            self.tx_position.set(0);

            // Enable transmissions and wait until the FIFO is empty before getting
            // an interrupt.
            self.registers
                .txctrl
                .write(txctrl::txen::SET + self.get_stop_bits() + txctrl::txcnt.val(1));

            // Enable the interrupt so we know when we can keep writing.
            self.enable_tx_interrupt();

            Ok(())
        }
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if self.tx_status.get() != UARTStateTX::Idle {
            self.registers.txctrl.write(txctrl::txen::CLEAR);
            self.disable_tx_interrupt();
            self.tx_status.set(UARTStateTX::AbortRequested);

            self.deferred_call.set();

            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
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
        if self.rx_status.get() != UARTStateRX::Idle {
            Err((ErrorCode::BUSY, rx_buffer))
        } else if rx_len > rx_buffer.len() {
            Err((ErrorCode::SIZE, rx_buffer))
        } else {
            self.rx_status.set(UARTStateRX::Receiving);

            self.rx_buffer.put(Some(rx_buffer));
            self.rx_position.set(0);
            self.rx_len.set(rx_len);

            // Enable receptions and wait until the FIFO has at least one byte
            // before getting an interrupt.
            self.registers
                .rxctrl
                .write(rxctrl::enable::SET + rxctrl::counter.val(0));

            self.enable_rx_interrupt();

            Ok(())
        }
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if self.rx_status.get() != UARTStateRX::Idle {
            self.registers.rxctrl.write(rxctrl::enable::CLEAR);
            self.disable_rx_interrupt();
            self.rx_status.set(UARTStateRX::AbortRequested);

            self.deferred_call.set();

            Err(ErrorCode::BUSY)
        } else {
            Ok(())
        }
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}
