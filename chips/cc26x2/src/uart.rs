//! UART driver, cc26x2 family
use kernel;
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::uart;
use kernel::ReturnCode;
use prcm;

use core::cmp;

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

pub static mut UART0: UART = UART::new(&UART0_BASE);
pub static mut UART1: UART = UART::new(&UART1_BASE);

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

const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x40001000 as *const UartRegisters) };

const UART1_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4000B000 as *const UartRegisters) };

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

pub struct UART {
    registers: &'static StaticRef<UartRegisters>,
    tx_client: OptionalCell<&'static uart::Client>,
    rx_client: OptionalCell<&'static uart::Client>,
    tx: MapCell<Transaction>,
    rx: MapCell<Transaction>,
}

impl UART {
    const fn new(registers: &'static StaticRef<UartRegisters>) -> UART {
        UART {
            registers,

            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),

            tx: MapCell::empty(),
            rx: MapCell::empty(),
        }
    }

    /// Initialize the UART hardware.
    ///
    /// This function needs to be run before the UART module is used.
    pub fn initialize(&self) {
        self.power_and_clock();
        self.enable_interrupts();
    }

    pub fn configure(&self, params: kernel::hil::uart::UARTParameters) -> ReturnCode {
        // These could probably be implemented, but are currently ignored, so
        // throw an error.
        if params.stop_bits != kernel::hil::uart::StopBits::One {
            return ReturnCode::ENOSUPPORT;
        }
        if params.parity != kernel::hil::uart::Parity::None {
            return ReturnCode::ENOSUPPORT;
        }
        if params.hw_flow_control != false {
            return ReturnCode::ENOSUPPORT;
        }

        // Disable the UART before configuring
        self.disable();

        self.set_baud_rate(params.baud_rate);

        // Set word length
        self.registers.lcrh.write(LineControl::WORD_LENGTH::Len8);

        self.fifo_enable();

        self.enable_interrupts();

        // Enable UART, RX and TX
        self.registers
            .ctl
            .write(Control::UART_ENABLE::SET + Control::RX_ENABLE::SET + Control::TX_ENABLE::SET);

        ReturnCode::SUCCESS
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

        self.rx.take().map(|mut rx| {
            while self.rx_fifo_not_empty() && rx.index < rx.length {
                let byte = self.read_byte();
                rx.buffer[rx.index] = byte;
                rx.index += 1;
            }

            if rx.index == rx.length {
                self.rx_client.map(move |client| {
                    client.receive_complete(
                        rx.buffer,
                        rx.index,
                        kernel::hil::uart::Error::CommandComplete,
                    );
                });
            } else {
                self.rx.put(rx);
            }
        });
        // if there is no client, empty the buffer into the void
        if self.rx_fifo_not_empty() {
            self.read_byte();
        }

        self.tx.take().map(|mut tx| {
            // if a big buffer was given, this could be a very long call
            if self.tx_fifo_not_full() && tx.index < tx.length {
                self.send_byte(tx.buffer[tx.index]);
                tx.index += 1;
            }
            if tx.index == tx.length {
                self.tx_client.map(move |client| {
                    client.transmit_complete(tx.buffer, kernel::hil::uart::Error::CommandComplete);
                });
            } else {
                self.tx.put(tx);
            }
        });
    }

    // Pushes a byte into the TX FIFO.
    #[inline]
    pub fn send_byte(&self, c: u8) {
        // Put byte in data register
        self.registers.dr.set(c as u32);
    }

    // Pulls a byte out of the RX FIFO.
    #[inline]
    pub fn read_byte(&self) -> u8 {
        self.registers.dr.get() as u8
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

    pub fn set_tx_client(&self, client: &'static kernel::hil::uart::Client) {
        self.tx_client.set(client);
    }

    pub fn set_rx_client(&self, client: &'static kernel::hil::uart::Client) {
        self.rx_client.set(client);
    }
}

impl kernel::hil::uart::UART for UART {
    fn set_client(&self, client: &'static kernel::hil::uart::Client) {
        self.rx_client.set(client);
        self.tx_client.set(client);
    }

    fn configure(&self, params: kernel::hil::uart::UARTParameters) -> ReturnCode {
        self.configure(params)
    }

    fn transmit(&self, buffer: &'static mut [u8], len: usize) {
        // if there is a weird input, don't try to do any transfers
        if len == 0 {
            self.tx_client.map(move |client| {
                client.transmit_complete(buffer, kernel::hil::uart::Error::CommandComplete);
            });
        } else {
            // if client set len too big, we will receive what we can
            let tx_len = cmp::min(len, buffer.len());

            // we will send one byte, causing EOT interrupt
            if self.tx_fifo_not_full() {
                self.send_byte(buffer[0]);
            }

            // Transaction will be continued in interrupt handler
            self.tx.put(Transaction {
                buffer: buffer,
                length: tx_len,
                index: 1,
            });
        }
    }

    fn receive(&self, buffer: &'static mut [u8], len: usize) {
        if len == 0 {
            self.rx_client.map(move |client| {
                client.receive_complete(buffer, len, kernel::hil::uart::Error::CommandComplete);
            });
        } else {
            // if client set len too big, we will receive what we can
            let rx_len = cmp::min(len, buffer.len());

            self.rx.put(Transaction {
                buffer: buffer,
                length: rx_len,
                index: 0,
            });
        }
    }

    fn abort_receive(&self) {
        self.rx.take().map(|rx| {
            self.rx_client.map(move |client| {
                client.receive_complete(
                    rx.buffer,
                    rx.index,
                    kernel::hil::uart::Error::CommandComplete,
                );
            });
        });
    }
}
