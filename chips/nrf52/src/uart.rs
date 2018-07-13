//! Universal asynchronous receiver/transmitter with EasyDMA (UARTE)
//!
//! Author
//! -------------------
//!
//! * Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Date: March 10 2018

use core;
use core::cell::Cell;
use core::cmp::min;
use kernel;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use nrf5x::pinmux;

const UARTE_MAX_BUFFER_SIZE: u32 = 0xff;

static mut BYTE: u8 = 0;

const UARTE_BASE: StaticRef<UarteRegisters> =
    unsafe { StaticRef::new(0x40002000 as *const UarteRegisters) };

#[repr(C)]
struct UarteRegisters {
    task_startrx: WriteOnly<u32, Task::Register>,
    task_stoprx: WriteOnly<u32, Task::Register>,
    task_starttx: WriteOnly<u32, Task::Register>,
    task_stoptx: WriteOnly<u32, Task::Register>,
    _reserved1: [u32; 7],
    task_flush_rx: WriteOnly<u32, Task::Register>,
    _reserved2: [u32; 52],
    event_cts: ReadWrite<u32, Event::Register>,
    event_ncts: ReadWrite<u32, Event::Register>,
    _reserved3: [u32; 2],
    event_endrx: ReadWrite<u32, Event::Register>,
    _reserved4: [u32; 3],
    event_endtx: ReadWrite<u32, Event::Register>,
    event_error: ReadWrite<u32, Event::Register>,
    _reserved6: [u32; 7],
    event_rxto: ReadWrite<u32, Event::Register>,
    _reserved7: [u32; 1],
    event_rxstarted: ReadWrite<u32, Event::Register>,
    event_txstarted: ReadWrite<u32, Event::Register>,
    _reserved8: [u32; 1],
    event_txstopped: ReadWrite<u32, Event::Register>,
    _reserved9: [u32; 41],
    shorts: ReadWrite<u32, Shorts::Register>,
    _reserved10: [u32; 64],
    intenset: ReadWrite<u32, Interrupt::Register>,
    intenclr: ReadWrite<u32, Interrupt::Register>,
    _reserved11: [u32; 93],
    errorsrc: ReadWrite<u32, ErrorSrc::Register>,
    _reserved12: [u32; 31],
    enable: ReadWrite<u32, Uart::Register>,
    _reserved13: [u32; 1],
    pselrts: ReadWrite<u32, Psel::Register>,
    pseltxd: ReadWrite<u32, Psel::Register>,
    pselcts: ReadWrite<u32, Psel::Register>,
    pselrxd: ReadWrite<u32, Psel::Register>,
    _reserved14: [u32; 3],
    baudrate: ReadWrite<u32, Baudrate::Register>,
    _reserved15: [u32; 3],
    rxd_ptr: ReadWrite<u32, Pointer::Register>,
    rxd_maxcnt: ReadWrite<u32, Counter::Register>,
    rxd_amount: ReadOnly<u32, Counter::Register>,
    _reserved16: [u32; 1],
    txd_ptr: ReadWrite<u32, Pointer::Register>,
    txd_maxcnt: ReadWrite<u32, Counter::Register>,
    txd_amount: ReadOnly<u32, Counter::Register>,
    _reserved17: [u32; 7],
    config: ReadWrite<u32, Config::Register>,
}

register_bitfields! [u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Shortcuts
    Shorts [
        // Shortcut between ENDRX and STARTRX
        ENDRX_STARTRX OFFSET(5) NUMBITS(1),
        // Shortcut between ENDRX and STOPRX
        ENDRX_STOPRX OFFSET(6) NUMBITS(1)
    ],

    /// UART Interrupts
    Interrupt [
        CTS OFFSET(0) NUMBITS(1),
        NCTS OFFSET(1) NUMBITS(1),
        ENDRX OFFSET(4) NUMBITS(1),
        ENDTX OFFSET(8) NUMBITS(1),
        ERROR OFFSET(9) NUMBITS(1),
        RXTO OFFSET(17) NUMBITS(1),
        RXSTARTED OFFSET(19) NUMBITS(1),
        TXSTARTED OFFSET(20) NUMBITS(1),
        TXSTOPPED OFFSET(22) NUMBITS(1)
    ],

    /// UART Errors
    ErrorSrc [
        OVERRUN OFFSET(0) NUMBITS(1),
        PARITY OFFSET(1) NUMBITS(1),
        FRAMING OFFSET(2) NUMBITS(1),
        BREAK OFFSET(3) NUMBITS(1)
    ],

    /// Enable UART
    Uart [
        ENABLE OFFSET(0) NUMBITS(4) [
            ON = 8,
            OFF = 0
        ]
    ],

    /// Pin select
    Psel [
        // Pin number
        PIN OFFSET(0) NUMBITS(5),
        // Connect/Disconnect
        CONNECT OFFSET(31) NUMBITS(1)
    ],

    /// Baudrate
    Baudrate [
        BAUDRAUTE OFFSET(0) NUMBITS(32)
    ],

    /// DMA pointer
    Pointer [
        POINTER OFFSET(0) NUMBITS(32)
    ],

    /// Counter value
    Counter [
        COUNTER OFFSET(0) NUMBITS(8)
    ],

    /// Configuration of parity and flow control
    Config [
        HWFC OFFSET(0) NUMBITS(1),
        PARITY OFFSET(1) NUMBITS(3)
    ]
];

/// UARTE
// It should never be instanced outside this module but because a static mutable reference to it
// is exported outside this module it must be `pub`
pub struct Uarte {
    registers: StaticRef<UarteRegisters>,
    client: OptionalCell<&'static kernel::hil::uart::Client>,
    tx_buffer: kernel::common::cells::TakeCell<'static, [u8]>,
    tx_remaining_bytes: Cell<usize>,
    rx_buffer: kernel::common::cells::TakeCell<'static, [u8]>,
    rx_remaining_bytes: Cell<usize>,
    rx_abort_in_progress: Cell<bool>,
    offset: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32,
}

/// UARTE0 handle
// This should only be accessed by the reset_handler on startup
pub static mut UARTE0: Uarte = Uarte::new();

impl Uarte {
    /// Constructor
    pub const fn new() -> Uarte {
        Uarte {
            registers: UARTE_BASE,
            client: OptionalCell::empty(),
            tx_buffer: kernel::common::cells::TakeCell::empty(),
            tx_remaining_bytes: Cell::new(0),
            rx_buffer: kernel::common::cells::TakeCell::empty(),
            rx_remaining_bytes: Cell::new(0),
            rx_abort_in_progress: Cell::new(false),
            offset: Cell::new(0),
        }
    }

    /// Configure which pins the UART should use for txd, rxd, cts and rts
    pub fn configure(
        &self,
        txd: pinmux::Pinmux,
        rxd: pinmux::Pinmux,
        cts: pinmux::Pinmux,
        rts: pinmux::Pinmux,
    ) {
        let regs = &*self.registers;
        regs.pseltxd.write(Psel::PIN.val(txd.into()));
        regs.pselrxd.write(Psel::PIN.val(rxd.into()));
        regs.pselcts.write(Psel::PIN.val(cts.into()));
        regs.pselrts.write(Psel::PIN.val(rts.into()));
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = &*self.registers;
        match baud_rate {
            1200 => regs.baudrate.set(0x0004F000),
            2400 => regs.baudrate.set(0x0009D000),
            4800 => regs.baudrate.set(0x0013B000),
            9600 => regs.baudrate.set(0x00275000),
            14400 => regs.baudrate.set(0x003AF000),
            19200 => regs.baudrate.set(0x004EA000),
            28800 => regs.baudrate.set(0x0075C000),
            38400 => regs.baudrate.set(0x009D0000),
            57600 => regs.baudrate.set(0x00EB0000),
            76800 => regs.baudrate.set(0x013A9000),
            115200 => regs.baudrate.set(0x01D60000),
            230400 => regs.baudrate.set(0x03B00000),
            250000 => regs.baudrate.set(0x04000000),
            460800 => regs.baudrate.set(0x07400000),
            921600 => regs.baudrate.set(0x0F000000),
            1000000 => regs.baudrate.set(0x10000000),
            _ => regs.baudrate.set(0x01D60000), //setting default to 115200
        }
    }

    // Enable UART peripheral, this need to disabled for low power applications
    fn enable_uart(&self) {
        let regs = &*self.registers;
        regs.enable.write(Uart::ENABLE::ON);
    }

    #[allow(dead_code)]
    fn disable_uart(&self) {
        let regs = &*self.registers;
        regs.enable.write(Uart::ENABLE::OFF);
    }

    fn enable_rx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenset.write(Interrupt::ENDRX::SET);
    }

    fn enable_tx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenset.write(Interrupt::ENDTX::SET);
    }

    fn disable_rx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenclr.write(Interrupt::ENDRX::SET);
    }

    fn disable_tx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenclr.write(Interrupt::ENDTX::SET);
    }

    /// UART interrupt handler that listens for both tx_end and rx_end events
    #[inline(never)]
    pub fn handle_interrupt(&mut self) {
        let regs = &*self.registers;

        if self.tx_ready() {
            self.disable_tx_interrupts();
            let regs = &*self.registers;
            regs.event_endtx.write(Event::READY::CLEAR);
            let tx_bytes = regs.txd_amount.get() as usize;

            let rem = match self.tx_remaining_bytes.get().checked_sub(tx_bytes) {
                None => {
                    debug!(
                        "Error more bytes transmitted than requested\n \
                         remaining: {} \t transmitted: {}",
                        self.tx_remaining_bytes.get(),
                        tx_bytes
                    );
                    return;
                }
                Some(r) => r,
            };

            // All bytes have been transmitted
            if rem == 0 {
                // Signal client write done
                self.client.map(|client| {
                    self.tx_buffer.take().map(|tx_buffer| {
                        client.transmit_complete(
                            tx_buffer,
                            kernel::hil::uart::Error::CommandComplete,
                        );
                    });
                });
            } else {
                // Not all bytes have been transmitted then update offset and continue transmitting
                self.offset.set(self.offset.get() + tx_bytes);
                self.tx_remaining_bytes.set(rem);
                self.set_tx_dma_pointer_to_buffer();
                regs.txd_maxcnt
                    .write(Counter::COUNTER.val(min(rem as u32, UARTE_MAX_BUFFER_SIZE)));
                regs.task_starttx.write(Task::ENABLE::SET);
                self.enable_tx_interrupts();
            }
        }

        if self.rx_ready() {
            self.disable_rx_interrupts();

            // Clear the ENDRX event
            regs.event_endrx.write(Event::READY::CLEAR);

            // Get the number of bytes in the buffer that was received this time
            let rx_bytes = regs.rxd_amount.get() as usize;

            // Check if this ENDRX is due to an abort. If so, we want to
            // do the receive callback immediately.
            if self.rx_abort_in_progress.get() {
                self.rx_abort_in_progress.set(false);
                self.client.map(|client| {
                    self.rx_buffer.take().map(|rx_buffer| {
                        client.receive_complete(
                            rx_buffer,
                            self.offset.get() + rx_bytes,
                            kernel::hil::uart::Error::CommandComplete,
                        );
                    });
                });
            } else {
                // In the normal case, we need to either pass call the callback
                // or do another read to get more bytes.

                // Update how many bytes we still need to receive and
                // where we are storing in the buffer.
                self.rx_remaining_bytes
                    .set(self.rx_remaining_bytes.get().saturating_sub(rx_bytes));
                self.offset.set(self.offset.get() + rx_bytes);

                let rem = self.rx_remaining_bytes.get();
                if rem == 0 {
                    // Signal client that the read is done
                    self.client.map(|client| {
                        self.rx_buffer.take().map(|rx_buffer| {
                            client.receive_complete(
                                rx_buffer,
                                self.offset.get(),
                                kernel::hil::uart::Error::CommandComplete,
                            );
                        });
                    });
                } else {
                    // Setup how much we can read. We already made sure that
                    // this will fit in the buffer.
                    let to_read = core::cmp::min(rem, 255);
                    regs.rxd_maxcnt.write(Counter::COUNTER.val(to_read as u32));

                    // Actually do the receive.
                    self.set_rx_dma_pointer_to_buffer();
                    regs.task_startrx.write(Task::ENABLE::SET);
                    self.enable_rx_interrupts();
                }
            }
        }
    }

    /// Transmit one byte at the time and the client is responsible for polling
    /// This is used by the panic handler
    pub unsafe fn send_byte(&self, byte: u8) {
        let regs = &*self.registers;

        self.tx_remaining_bytes.set(1);
        regs.event_endtx.write(Event::READY::CLEAR);
        // precaution: copy value into variable with static lifetime
        BYTE = byte;
        regs.txd_ptr.set((&BYTE as *const u8) as u32);
        regs.txd_maxcnt.write(Counter::COUNTER.val(1));
        regs.task_starttx.write(Task::ENABLE::SET);
    }

    /// Check if the UART transmission is done
    pub fn tx_ready(&self) -> bool {
        let regs = &*self.registers;
        regs.event_endtx.is_set(Event::READY)
    }

    /// Check if either the rx_buffer is full or the UART has timed out
    pub fn rx_ready(&self) -> bool {
        let regs = &*self.registers;
        regs.event_endrx.is_set(Event::READY)
    }

    fn set_tx_dma_pointer_to_buffer(&self) {
        let regs = &*self.registers;
        self.tx_buffer.map(|tx_buffer| {
            regs.txd_ptr
                .set(tx_buffer[self.offset.get()..].as_ptr() as u32);
        });
    }

    fn set_rx_dma_pointer_to_buffer(&self) {
        let regs = &*self.registers;
        self.rx_buffer.map(|rx_buffer| {
            regs.rxd_ptr
                .set(rx_buffer[self.offset.get()..].as_ptr() as u32);
        });
    }
}

impl kernel::hil::uart::UART for Uarte {
    fn set_client(&self, client: &'static kernel::hil::uart::Client) {
        self.client.set(client);
    }

    fn init(&self, params: kernel::hil::uart::UARTParams) {
        self.enable_uart();
        self.set_baud_rate(params.baud_rate);
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        let truncated_len = min(tx_data.len(), tx_len);

        if truncated_len == 0 {
            return;
        }

        self.tx_remaining_bytes.set(tx_len);
        self.offset.set(0);
        self.tx_buffer.replace(tx_data);
        self.set_tx_dma_pointer_to_buffer();

        let regs = &*self.registers;
        regs.txd_maxcnt
            .write(Counter::COUNTER.val(min(tx_len as u32, UARTE_MAX_BUFFER_SIZE)));
        regs.task_starttx.write(Task::ENABLE::SET);

        self.enable_tx_interrupts();
    }

    fn receive(&self, rx_buf: &'static mut [u8], rx_len: usize) {
        let regs = &*self.registers;

        // truncate rx_len if necessary
        let truncated_length = core::cmp::min(rx_len, rx_buf.len());

        self.rx_remaining_bytes.set(truncated_length);
        self.offset.set(0);
        self.rx_buffer.replace(rx_buf);
        self.set_rx_dma_pointer_to_buffer();

        let truncated_uart_max_length = core::cmp::min(truncated_length, 255);

        regs.rxd_maxcnt
            .write(Counter::COUNTER.val(truncated_uart_max_length as u32));
        regs.task_stoprx.write(Task::ENABLE::SET);
        regs.task_startrx.write(Task::ENABLE::SET);

        self.enable_rx_interrupts();
    }

    fn abort_receive(&self) {
        // Trigger the STOPRX event to cancel the current receive call.
        let regs = &*self.registers;
        self.rx_abort_in_progress.set(true);
        regs.task_stoprx.write(Task::ENABLE::SET);
    }
}
