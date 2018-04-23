//! Universal asynchronous receiver/transmitter with EasyDMA (UARTE)
//!
//! The driver provides only tranmission functionlity
//!
//! Author
//! -------------------
//!
//! * Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Date: March 10 2018

use core;
use core::cell::Cell;
use kernel;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use nrf5x::pinmux;

const UARTE_BASE: u32 = 0x40002000;
static mut BYTE: u8 = 0;

#[repr(C)]
struct UarteRegisters {
    pub task_startrx: WriteOnly<u32, Task::Register>, // 0x000
    pub task_stoprx: WriteOnly<u32, Task::Register>,  // 0x004
    pub task_starttx: WriteOnly<u32, Task::Register>, // 0x008
    pub task_stoptx: WriteOnly<u32, Task::Register>,  // 0x00c
    _reserved1: [u32; 7],                             // 0x010-0x02c
    pub task_flush_rx: WriteOnly<u32, Task::Register>, // 0x02c
    _reserved2: [u32; 52],                            // 0x030-0x100
    pub event_cts: ReadWrite<u32, Event::Register>,   // 0x100-0x104
    pub event_ncts: ReadWrite<u32, Event::Register>,  // 0x104-0x108
    _reserved3: [u32; 2],                             // 0x108-0x110
    pub event_endrx: ReadWrite<u32, Event::Register>, // 0x110-0x114
    _reserved4: [u32; 3],                             // 0x114-0x120
    pub event_endtx: ReadWrite<u32, Event::Register>, // 0x120-0x124
    pub event_error: ReadWrite<u32, Event::Register>, // 0x124-0x128
    _reserved6: [u32; 7],                             // 0x128-0x144
    pub event_rxto: ReadWrite<u32, Event::Register>,  // 0x144-0x148
    _reserved7: [u32; 1],                             // 0x148-0x14C
    pub event_rxstarted: ReadWrite<u32, Event::Register>, // 0x14C-0x150
    pub event_txstarted: ReadWrite<u32, Event::Register>, // 0x150-0x154
    _reserved8: [u32; 1],                             // 0x154-0x158
    pub event_txstopped: ReadWrite<u32, Event::Register>, // 0x158-0x15c
    _reserved9: [u32; 41],                            // 0x15c-0x200
    pub shorts: ReadWrite<u32, Shorts::Register>,     // 0x200-0x204
    _reserved10: [u32; 64],                           // 0x204-0x304
    pub intenset: ReadWrite<u32, Interrupt::Register>, // 0x304-0x308
    pub intenclr: ReadWrite<u32, Interrupt::Register>, // 0x308-0x30C
    _reserved11: [u32; 93],                           // 0x30C-0x480
    pub errorsrc: ReadWrite<u32, ErrorSrc::Register>, // 0x480-0x484
    _reserved12: [u32; 31],                           // 0x484-0x500
    pub enable: ReadWrite<u32, Uart::Register>,       // 0x500-0x504
    _reserved13: [u32; 1],                            // 0x504-0x508
    pub pselrts: ReadWrite<u32, Psel::Register>,      // 0x508-0x50c
    pub pseltxd: ReadWrite<u32, Psel::Register>,      // 0x50c-0x510
    pub pselcts: ReadWrite<u32, Psel::Register>,      // 0x510-0x514
    pub pselrxd: ReadWrite<u32, Psel::Register>,      // 0x514-0x518
    _reserved14: [u32; 3],                            // 0x518-0x524
    pub baudrate: ReadWrite<u32, Baudrate::Register>, // 0x524-0x528
    _reserved15: [u32; 3],                            // 0x528-0x534
    pub rxd_ptr: ReadWrite<u32, Pointer::Register>,   // 0x534-0x538
    pub rxd_maxcnt: ReadWrite<u32, Counter::Register>, // 0x538-0x53c
    pub rxd_amount: ReadOnly<u32, Counter::Register>, // 0x53c-0x540
    _reserved16: [u32; 1],                            // 0x540-0x544
    pub txd_ptr: ReadWrite<u32, Pointer::Register>,   // 0x544-0x548
    pub txd_maxcnt: ReadWrite<u32, Counter::Register>, // 0x548-0x54c
    pub txd_amount: ReadOnly<u32, Counter::Register>, // 0x54c-0x550
    _reserved17: [u32; 7],                            // 0x550-0x56C
    pub config: ReadWrite<u32, Config::Register>,     // 0x56C-0x570
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
    regs: *const UarteRegisters,
    client: Cell<Option<&'static kernel::hil::uart::Client>>,
    tx_buffer: kernel::common::take_cell::TakeCell<'static, [u8]>,
    tx_remaining_bytes: Cell<usize>,
    rx_buffer: kernel::common::take_cell::TakeCell<'static, [u8]>,
    rx_remaining_bytes: Cell<usize>,
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
            regs: UARTE_BASE as *const UarteRegisters,
            client: Cell::new(None),
            tx_buffer: kernel::common::take_cell::TakeCell::empty(),
            tx_remaining_bytes: Cell::new(0),
            rx_buffer: kernel::common::take_cell::TakeCell::empty(),
            rx_remaining_bytes: Cell::new(0),
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
        let regs = unsafe { &*self.regs };
        regs.pseltxd.write(Psel::PIN.val(txd.into()));
        regs.pselrxd.write(Psel::PIN.val(rxd.into()));
        regs.pselcts.write(Psel::PIN.val(cts.into()));
        regs.pselrts.write(Psel::PIN.val(rts.into()));
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = unsafe { &*self.regs };
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
        let regs = unsafe { &*self.regs };
        regs.enable.write(Uart::ENABLE::ON);
    }

    #[allow(dead_code)]
    fn disable_uart(&self) {
        let regs = unsafe { &*self.regs };
        regs.enable.write(Uart::ENABLE::OFF);
    }

    fn enable_rx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.write(Interrupt::ENDRX::SET);
    }

    fn enable_tx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.write(Interrupt::ENDTX::SET);
    }

    fn disable_rx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.write(Interrupt::ENDRX::SET);
    }

    fn disable_tx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.write(Interrupt::ENDTX::SET);
    }

    /// UART interrupt handler that listens for both tx_end and rx_end events
    #[inline(never)]
    pub fn handle_interrupt(&mut self) {
        let regs = unsafe { &*self.regs };

        if self.tx_ready() {
            self.disable_tx_interrupts();
            // disable interrupts
            regs.event_endtx.write(Event::READY::CLEAR);
            let tx_bytes = regs.txd_amount.get() as usize;
            let rem = self.tx_remaining_bytes.get();

            // More bytes transmitted than requested `return silently`
            // Cause probably a hardware fault
            // FIXME: Progate error to the capsule
            if tx_bytes > rem {
                debug!("error more bytes than requested\r\n");
                return;
            }

            self.tx_remaining_bytes.set(rem - tx_bytes);
            self.offset.set(tx_bytes);

            if self.tx_remaining_bytes.get() == 0 {
                // Signal client write done
                self.client.get().map(|client| {
                    self.tx_buffer.take().map(|tx_buffer| {
                        client.transmit_complete(
                            tx_buffer,
                            kernel::hil::uart::Error::CommandComplete,
                        );
                    });
                });
            }
            // Not all bytes have been transmitted then update offset and continue transmitting
            else {
                self.set_tx_dma_pointer_to_buffer();
                regs.task_starttx.write(Task::ENABLE::SET);
                self.enable_tx_interrupts();
            }
        }

        if self.rx_ready() {
            self.disable_rx_interrupts();
            regs.event_endrx.write(Event::READY::CLEAR);
            //Get the number of bytes in the buffer
            let rx_bytes = regs.rxd_amount.get() as usize;
            let rem = self.rx_remaining_bytes.get();
            //should check if rx_bytes > 0 (because we'll flush the FIFO and it will flag
            //ENDRX even if it's empty)
            if rx_bytes > 0 {
                self.rx_remaining_bytes.set(rem.saturating_sub(rx_bytes));
                //check if we're waiting for more (i.e. are we done listening?)
                if self.rx_remaining_bytes.get() == 0 {
                    // Signal client that the read is done
                    self.client.get().map(|client| {
                        self.rx_buffer.take().map(|rx_buffer| {
                            client.receive_complete(
                                rx_buffer,
                                rx_bytes,
                                kernel::hil::uart::Error::CommandComplete,
                            );
                        });
                    });
                } else {
                    self.set_rx_dma_pointer_to_buffer();
                    //Flush the fifo, as per the datasheet recommendations
                    regs.task_flush_rx.write(Task::ENABLE::SET);
                    regs.task_startrx.write(Task::ENABLE::SET);
                    self.enable_rx_interrupts();
                }
            }
        }
    }

    /// Transmit one byte at the time and the client is resposible for polling
    /// This is used by the panic handler
    pub unsafe fn send_byte(&self, byte: u8) {
        let regs = &*self.regs;

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
        let regs = unsafe { &*self.regs };
        regs.event_endtx.is_set(Event::READY)
    }

    /// Check if either the rx_buffer is full or the UART has timed out
    pub fn rx_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.event_endrx.is_set(Event::READY)
    }

    fn set_tx_dma_pointer_to_buffer(&self) {
        let regs = unsafe { &*self.regs };
        self.tx_buffer.map(|tx_buffer| {
            regs.txd_ptr
                .set(tx_buffer[self.offset.get()..].as_ptr() as u32);
        });
    }

    fn set_rx_dma_pointer_to_buffer(&self) {
        let regs = unsafe { &*self.regs };
        self.rx_buffer.map(|rx_buffer| {
            regs.rxd_ptr
                .set(rx_buffer[self.offset.get()..].as_ptr() as u32);
        });
    }
}

impl kernel::hil::uart::UART for Uarte {
    fn set_client(&self, client: &'static kernel::hil::uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, params: kernel::hil::uart::UARTParams) {
        self.enable_uart();
        self.set_baud_rate(params.baud_rate);
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        let regs = unsafe { &*self.regs };

        if tx_len == 0 {
            return;
        }

        self.tx_remaining_bytes.set(tx_len);
        self.offset.set(0);
        self.tx_buffer.replace(tx_data);
        self.set_tx_dma_pointer_to_buffer();

        regs.txd_maxcnt.write(Counter::COUNTER.val(tx_len as u32));
        regs.task_stoptx.write(Task::ENABLE::SET);
        regs.task_starttx.write(Task::ENABLE::SET);

        self.enable_tx_interrupts();
    }

    fn receive(&self, rx_buf: &'static mut [u8], rx_len: usize) {
        let regs = unsafe { &*self.regs };

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
}
