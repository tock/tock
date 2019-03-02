//! UART driver, cc26x2 family
use crate::prcm;
use core::cell::Cell;
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::uart;
use kernel::ReturnCode;

const MCU_CLOCK: u32 = 48_000_000;

#[repr(C)]
struct UartRegisters {
    dr: ReadWrite<u32>,
    rsr_ecr: ReadWrite<u32>,
    _reserved0: [u32; 0x4],
    fr: ReadOnly<u32, Flags::Register>,
    _reserved1: [u32; 0x2],
    ibrd: ReadWrite<u32, IntDivisor::Register>,
    fbrd: ReadWrite<u32, FracDivisor::Register>,
    lcrh: ReadWrite<u32, LineControl::Register>,
    ctl: ReadWrite<u32, Control::Register>,
    ifls: ReadWrite<u32>,
    imsc: ReadWrite<u32, Interrupts::Register>,
    ris: ReadOnly<u32, Interrupts::Register>,
    mis: ReadOnly<u32, Interrupts::Register>,
    icr: WriteOnly<u32, Interrupts::Register>,
    dmactl: ReadWrite<u32>,
}

pub static mut UART0: UART = UART::new(&UART0_REG);
pub static mut UART1: UART = UART::new(&UART1_REG);

register_bitfields![
    u32,
    Control [
        UART_ENABLE OFFSET(0) NUMBITS(1) [],
        LB_ENABLE OFFSET(7) NUMBITS(1) [],
        TX_ENABLE OFFSET(8) NUMBITS(1) [],
        RX_ENABLE OFFSET(9) NUMBITS(1) []
    ],
    LineControl [
        FIFO_ENABLE OFFSET(4) NUMBITS(1) [],
        WORD_LENGTH OFFSET(5) NUMBITS(2) [
            Len5 = 0x0,
            Len6 = 0x1,
            Len7 = 0x2,
            Len8 = 0x3
        ]
    ],
    IntDivisor [
        DIVISOR OFFSET(0) NUMBITS(16) []
    ],
    FracDivisor [
        DIVISOR OFFSET(0) NUMBITS(6) []
    ],
    Flags [
        CTS OFFSET(0) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
        RX_FIFO_EMPTY OFFSET(4) NUMBITS(1) [],
        TX_FIFO_FULL OFFSET(5) NUMBITS(1) [],
        RX_FIFO_FULL OFFSET(6) NUMBITS(1) [],
        TX_FIFO_EMPTY OFFSET(7) NUMBITS(1) []
    ],
    Interrupts [
         ALL_INTERRUPTS OFFSET(0) NUMBITS(12) [
            // sets all interrupts without writing 1's to reg with undefined behavior
            Set =  0b111111110010,
            // you are allowed to write 0 to everyone
            Clear = 0x000000
        ],
        CTSIMM OFFSET(1) NUMBITS(1) [],              // clear to send interrupt mask
        RX OFFSET(4) NUMBITS(1) [],                  // receive interrupt mask
        TX OFFSET(5) NUMBITS(1) [],                  // transmit interrupt mask
        RX_TIMEOUT OFFSET(6) NUMBITS(1) [],          // receive timeout interrupt mask
        FE OFFSET(7) NUMBITS(1) [],                  // framing error interrupt mask
        PE OFFSET(8) NUMBITS(1) [],                  // parity error interrupt mask
        BE OFFSET(9) NUMBITS(1) [],                  // break error interrupt mask
        OE OFFSET(10) NUMBITS(1) [],                 // overrun error interrupt mask
        END_OF_TRANSMISSION OFFSET(11) NUMBITS(1) [] // end of transmission interrupt mask
    ]
];

use crate::memory_map::{UART0_BASE, UART1_BASE};

const UART0_REG: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(UART0_BASE as *const UartRegisters) };

const UART1_REG: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(UART1_BASE as *const UartRegisters) };

/// Stores an ongoing TX transaction
struct Transaction {
    /// The buffer containing the bytes to transmit as it should be returned to
    /// the client
    buffer: &'static mut [u8],
    /// The total amount to transmit
    length: usize,
    /// The index of the byte currently being sent
    index: usize,
}

pub struct UART<'a> {
    registers: &'static StaticRef<UartRegisters>,
    tx_client: OptionalCell<&'a uart::TransmitClient>,
    rx_client: OptionalCell<&'a uart::ReceiveClient>,
    tx: MapCell<Transaction>,
    rx: MapCell<Transaction>,
    receiving_word: Cell<bool>,
}

impl<'a> UART<'a> {
    const fn new(registers: &'static StaticRef<UartRegisters>) -> UART {
        UART {
            registers,

            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),

            tx: MapCell::empty(),
            rx: MapCell::empty(),

            receiving_word: Cell::new(false),
        }
    }

    /// Initialize the UART hardware.
    ///
    /// This function needs to be run before the UART module is used.
    pub fn initialize(&self) {
        self.power_and_clock();
        self.enable_interrupts();
    }

    fn power_and_clock(&self) {
        prcm::Power::enable_domain(prcm::PowerDomain::Serial);
        while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}
        prcm::Clock::enable_uarts();
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        // Fractional baud rate divider
        let div = (((MCU_CLOCK * 8) / baud_rate) + 1) / 2;
        // Set the baud rate
        self.registers.ibrd.write(IntDivisor::DIVISOR.val(div / 64));
        self.registers
            .fbrd
            .write(FracDivisor::DIVISOR.val(div % 64));
    }

    fn fifo_enable(&self) {
        self.registers.lcrh.modify(LineControl::FIFO_ENABLE::SET);
    }

    fn fifo_disable(&self) {
        self.registers.lcrh.modify(LineControl::FIFO_ENABLE::CLEAR);
    }

    fn disable(&self) {
        // disable interrupts
        self.registers.imsc.write(Interrupts::ALL_INTERRUPTS::CLEAR);
        self.fifo_disable();
        self.registers.ctl.modify(
            Control::UART_ENABLE::CLEAR + Control::TX_ENABLE::CLEAR + Control::RX_ENABLE::CLEAR,
        );
    }

    fn enable_interrupts(&self) {
        // set only interrupts used
        self.registers.imsc.modify(
            Interrupts::RX::SET
                + Interrupts::RX_TIMEOUT::SET
                + Interrupts::END_OF_TRANSMISSION::SET,
        );
    }

    /// Clears all interrupts related to UART.
    pub fn handle_interrupt(&self) {
        // Clear interrupts
        self.registers.icr.write(Interrupts::ALL_INTERRUPTS::SET);

        // Hardware RX FIFO is not empty
        while self.rx_fifo_not_empty() {
            // word read request was made
            if self.receiving_word.get() {
                let word = self.read();
                self.receiving_word.set(false);
                self.rx_client.map(move |client| {
                    client.received_word(word, ReturnCode::SUCCESS, uart::Error::None);
                });
            }
            // buffer read request was made
            else if self.rx.is_some() {
                self.rx.take().map(|mut rx| {
                    // read in a byte
                    if rx.index < rx.length {
                        let byte = self.read() as u8;
                        rx.buffer[rx.index] = byte;
                        rx.index += 1;
                    }

                    if rx.index == rx.length {
                        self.rx_client.map(move |client| {
                            client.received_buffer(
                                rx.buffer,
                                rx.index,
                                ReturnCode::SUCCESS,
                                uart::Error::None,
                            );
                        });
                    } else {
                        self.rx.put(rx);
                    }
                });
            }
            // no current read request
            else {
                // read bytes into the void to avoid hardware RX buffer overflow
                self.read();
            }
        }

        self.tx.take().map(|mut tx| {
            // send out one byte at a time, IRQ when TX FIFO empty will bring us back
            if self.tx_fifo_not_full() && tx.index < tx.length {
                self.write(tx.buffer[tx.index] as u32);
                tx.index += 1;
            }
            // request is done
            if tx.index == tx.length {
                self.tx_client.map(move |client| {
                    client.transmitted_buffer(tx.buffer, tx.length, ReturnCode::SUCCESS);
                });
            } else {
                // keep TX buffer as there is more left in request
                self.tx.put(tx);
            }
        });
    }

    pub fn write(&self, c: u32) {
        // Put byte in data register
        self.registers.dr.set(c);
    }

    // Pulls a byte out of the RX FIFO.
    #[inline]
    pub fn read(&self) -> u32 {
        self.registers.dr.get()
    }

    /// Checks if there is space in the transmit fifo queue.
    #[inline]
    pub fn rx_fifo_not_empty(&self) -> bool {
        !self.registers.fr.is_set(Flags::RX_FIFO_EMPTY)
    }

    /// Checks if there is space in the transmit fifo queue.
    #[inline]
    pub fn tx_fifo_not_full(&self) -> bool {
        !self.registers.fr.is_set(Flags::TX_FIFO_FULL)
    }
}

impl<'a> uart::Uart<'a> for UART<'a> {}
impl<'a> uart::UartData<'a> for UART<'a> {}

impl<'a> uart::Configure for UART<'a> {
    fn configure(&self, params: uart::Parameters) -> ReturnCode {
        // These could probably be implemented, but are currently ignored, so
        // throw an error.
        if params.stop_bits != uart::StopBits::One {
            return ReturnCode::ENOSUPPORT;
        }
        if params.parity != uart::Parity::None {
            return ReturnCode::ENOSUPPORT;
        }
        if params.hw_flow_control != false {
            return ReturnCode::ENOSUPPORT;
        }

        // Disable the UART before configuring
        self.disable();

        self.set_baud_rate(params.baud_rate);

        // Set word length
        let word_width = match params.width {
            uart::Width::Six => LineControl::WORD_LENGTH::Len6,
            uart::Width::Seven => LineControl::WORD_LENGTH::Len7,
            uart::Width::Eight => LineControl::WORD_LENGTH::Len8,
        };
        self.registers.lcrh.write(word_width);

        self.fifo_enable();

        self.enable_interrupts();

        // Enable UART, RX and TX
        self.registers
            .ctl
            .write(Control::UART_ENABLE::SET + Control::RX_ENABLE::SET + Control::TX_ENABLE::SET);

        ReturnCode::SUCCESS
    }
}

impl<'a> uart::Transmit<'a> for UART<'a> {
    fn set_transmit_client(&self, client: &'a uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        // if there is a weird input, don't try to do any transfers
        if len == 0 || len > buffer.len() {
            (ReturnCode::ESIZE, Some(buffer))
        } else if self.tx.is_some() {
            (ReturnCode::EBUSY, Some(buffer))
        } else {
            // we will send one byte, causing EOT interrupt
            if self.tx_fifo_not_full() {
                self.write(buffer[0] as u32);
            }
            // Transaction will be continued in interrupt bottom half
            self.tx.put(Transaction {
                buffer: buffer,
                length: len,
                index: 1,
            });
            (ReturnCode::SUCCESS, None)
        }
    }

    fn transmit_word(&self, word: u32) -> ReturnCode {
        // if there's room in outgoing FIFO and no buffer transaction
        if self.tx_fifo_not_full() && self.tx.is_none() {
            self.write(word);
            return ReturnCode::SUCCESS;
        }
        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}

impl<'a> uart::Receive<'a> for UART<'a> {
    fn set_receive_client(&self, client: &'a uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if len == 0 || len > buffer.len() {
            (ReturnCode::ESIZE, Some(buffer))
        } else if self.rx.is_some() || self.receiving_word.get() {
            (ReturnCode::EBUSY, Some(buffer))
        } else {
            self.rx.put(Transaction {
                buffer: buffer,
                length: len,
                index: 0,
            });

            (ReturnCode::SUCCESS, None)
        }
    }

    fn receive_word(&self) -> ReturnCode {
        if self.rx.is_some() || self.receiving_word.get() {
            ReturnCode::EBUSY
        } else {
            self.receiving_word.set(true);
            ReturnCode::SUCCESS
        }
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}
