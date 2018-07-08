use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::common::regs::{ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::uart;
use nrf5x::pinmux::Pinmux;

pub static mut UART0: UART = UART::new();

#[repr(C)]
struct UartRegisters {
    // Tasks
    task_startrx: WriteOnly<u32, Task::Register>, //... 0x000
    task_stoprx: WriteOnly<u32, Task::Register>,  //... 0x004
    task_starttx: WriteOnly<u32, Task::Register>, //... 0x008
    task_stoptx: WriteOnly<u32, Task::Register>,  //... 0x00c
    _reserved1: [u32; 3],
    task_suspend: WriteOnly<u32, Task::Register>, //... 0x01c
    _reserved2: [u32; 56],
    // Events
    event_cts: ReadWrite<u32, Event::Register>, //..... 0x100
    event_ncts: ReadWrite<u32, Event::Register>, //.... 0x104
    event_rxdrdy: ReadWrite<u32, Event::Register>, //.. 0x108
    _reserved3: [u32; 4],
    event_txdrdy: ReadWrite<u32, Event::Register>, //.. 0x11c
    _reserved4: [u32; 1],
    event_error: ReadWrite<u32, Event::Register>, //... 0x124
    _reserved5: [u32; 7],
    event_rxto: ReadWrite<u32, Event::Register>, //.... 0x144
    _reserved6: [u32; 46],
    // Shorts
    _shorts: [u32; 1], //.............................. 0x200
    _reserved7: [u32; 63],
    // Registers
    inten: ReadWrite<u32, Interrupt::Register>, //..... 0x300
    intenset: ReadWrite<u32, Interrupt::Register>, //.. 0x304
    intenclr: ReadWrite<u32, Interrupt::Register>, //.. 0x308
    _reserved8: [u32; 93],
    errorsrc: ReadWrite<u32, Errorsrc::Register>, //... 0x480
    _reserved9: [u32; 31],
    enable: ReadWrite<u32, Enable::Register>, //....... 0x500
    _reserved10: [u32; 1],
    pselrts: ReadWrite<u32, Psel::Register>, //........ 0x508
    pseltxd: ReadWrite<u32, Psel::Register>, //........ 0x50c
    pselcts: ReadWrite<u32, Psel::Register>, //........ 0x510
    pselrxd: ReadWrite<u32, Psel::Register>, //........ 0x514
    rxd: ReadWrite<u32, Rxd::Register>,      //........ 0x518
    txd: ReadWrite<u32, Txd::Register>,      //........ 0x51c
    _reserved11: [u32; 1],
    baudrate: ReadWrite<u32, Baudrate::Register>, //... 0x524
    _reserved12: [u32; 17],
    config: ReadWrite<u32, Config::Register>, //....... 0x56c
}

register_bitfields![u32,
    /// Start task.
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Events.
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Interrupts.
    ///
    /// Writes of 0 to the `set` and `clr` variants have no effect.
    Interrupt [
        CTS OFFSET(0) NUMBITS(1),
        NCTS OFFSET(1) NUMBITS(1),
        RXDRDY OFFSET(2) NUMBITS(1),
        TXDRDY OFFSET(7) NUMBITS(1),
        ERROR OFFSET(9) NUMBITS(1),
        RXTO OFFSET(17) NUMBITS(1)
    ],

    /// Error Source.
    ///
    /// Individual bits are cleared by writing a '1' to the bits that shall
    /// be cleared. Writing a '0' will have no effect.
    Errorsrc [
        OVERRUN OFFSET(0) NUMBITS(1),
        PARITY OFFSET(1) NUMBITS(1),
        FRAMING OFFSET(2) NUMBITS(1),
        BREAK OFFSET(3) NUMBITS(1)
    ],

    /// Enable or disable Uart.
    Enable [
        ENABLE OFFSET(0) NUMBITS(3) [
            ON = 4,
            OFF = 0
        ]
    ],

    /// Pin number configuration for UART RTS/TXD/CTS/RXD signals.
    Psel [
        PIN OFFSET(0) NUMBITS(32)
    ],

    /// RX data received in previous transfers, double buffered.
    Rxd [
        RXD OFFSET(0) NUMBITS(8)
    ],

    /// TX data to be transferred.
    Txd [
        TXD OFFSET(0) NUMBITS(8)
    ],

    /// Baudrate.
    Baudrate [
        BAUDRATE OFFSET(0) NUMBITS(32) [
            Baud1200 = 0x0004F000,      // 1200 baud
            Baud2400 = 0x0009D000,      // 2400 baud
            Baud4800 = 0x0013B000,      // 4800 baud
            Baud9600 = 0x00275000,      // 9600 baud
            Baud14400 = 0x003B0000,     // 14400 baud
            Baud19200 = 0x004EA000,     // 19200 baud
            Baud28800 = 0x0075F000,     // 28800 baud
            Baud38400 = 0x009D5000,     // 38400 baud
            Baud57600 = 0x00EBF000,     // 57600 baud
            Baud76800 = 0x013A9000,     // 76800 baud
            Baud115200 = 0x01D7E000,    // 115200 baud
            Baud230400 = 0x03AFB000,    // 230400 baud
            Baud250000 = 0x04000000,    // 250000 baud
            Baud460800 = 0x075F7000,    // 460800 baud
            Baud921600 = 0x0EBEDFA4,    // 921600 baud
            Baud1M = 0x10000000         // 1Mega baud
        ]
    ],

    /// Configuration.
    Config [
        HWFC OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        PARITY OFFSET(1) NUMBITS(3) [
            ExcludeParity = 0,
            IncludeParity = 7
        ]
    ]
];

const UART_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x40002000 as *const UartRegisters) };

pub struct UART {
    registers: StaticRef<UartRegisters>,
    client: Cell<Option<&'static uart::Client>>,
    buffer: TakeCell<'static, [u8]>,
    len: Cell<usize>,
    index: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32,
}

impl UART {
    pub const fn new() -> UART {
        UART {
            registers: UART_BASE,
            client: Cell::new(None),
            buffer: TakeCell::empty(),
            len: Cell::new(0),
            index: Cell::new(0),
        }
    }

    /// This UART implementation uses pins 8-11:
    ///
    /// * pin  8: RTS
    /// * pin  9: TX
    /// * pin 10: CTS
    /// * pin 11: RX
    pub fn configure(&self, tx: Pinmux, rx: Pinmux, cts: Pinmux, rts: Pinmux) {
        let regs = &*self.registers;

        regs.pseltxd.write(Psel::PIN.val(tx.into()));
        regs.pselrxd.write(Psel::PIN.val(rx.into()));
        regs.pselcts.write(Psel::PIN.val(cts.into()));
        regs.pselrts.write(Psel::PIN.val(rts.into()));
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = &*self.registers;
        match baud_rate {
            1200 => regs.baudrate.write(Baudrate::BAUDRATE::Baud1200),
            2400 => regs.baudrate.write(Baudrate::BAUDRATE::Baud2400),
            4800 => regs.baudrate.write(Baudrate::BAUDRATE::Baud4800),
            9600 => regs.baudrate.write(Baudrate::BAUDRATE::Baud9600),
            14400 => regs.baudrate.write(Baudrate::BAUDRATE::Baud14400),
            19200 => regs.baudrate.write(Baudrate::BAUDRATE::Baud19200),
            28800 => regs.baudrate.write(Baudrate::BAUDRATE::Baud28800),
            38400 => regs.baudrate.write(Baudrate::BAUDRATE::Baud38400),
            57600 => regs.baudrate.write(Baudrate::BAUDRATE::Baud57600),
            76800 => regs.baudrate.write(Baudrate::BAUDRATE::Baud76800),
            115200 => regs.baudrate.write(Baudrate::BAUDRATE::Baud115200),
            230400 => regs.baudrate.write(Baudrate::BAUDRATE::Baud230400),
            250000 => regs.baudrate.write(Baudrate::BAUDRATE::Baud250000),
            460800 => regs.baudrate.write(Baudrate::BAUDRATE::Baud460800),
            1000000 => regs.baudrate.write(Baudrate::BAUDRATE::Baud1M),
            _ => panic!("Illegal baud rate"),
        }
    }

    pub fn enable(&self) {
        let regs = &*self.registers;
        regs.enable.write(Enable::ENABLE::ON);
    }

    pub fn enable_rx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenset.write(Interrupt::RXDRDY::SET);
    }

    pub fn enable_tx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenset.write(Interrupt::TXDRDY::SET);
    }

    pub fn disable_rx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenclr.write(Interrupt::RXDRDY::SET);
    }

    pub fn disable_tx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenclr.write(Interrupt::TXDRDY::SET);
    }

    pub fn handle_interrupt(&mut self) {
        let regs = &*self.registers;
        let tx = regs.event_txdrdy.is_set(Event::READY);

        if tx {
            regs.event_txdrdy.write(Event::READY::CLEAR);

            if self.len.get() == self.index.get() {
                regs.task_stoptx.write(Task::ENABLE::SET);

                // Signal client write done
                self.client.get().map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.transmit_complete(buffer, uart::Error::CommandComplete);
                    });
                });

                return;
            }

            self.buffer.map(|buffer| {
                regs.event_txdrdy.write(Event::READY::CLEAR);
                regs.txd.set(buffer[self.index.get()] as u32);
                let next_index = self.index.get() + 1;
                self.index.set(next_index);
            });
        }
    }

    pub unsafe fn send_byte(&self, byte: u8) {
        let regs = &*self.registers;

        self.index.set(1);
        self.len.set(1);

        regs.event_txdrdy.write(Event::READY::CLEAR);
        self.enable_tx_interrupts();
        regs.task_starttx.set(1);
        regs.txd.set(byte as u32);
    }

    pub fn tx_ready(&self) -> bool {
        let regs = &*self.registers;
        regs.event_txdrdy.is_set(Event::READY)
    }

    fn rx_ready(&self) -> bool {
        let regs = &*self.registers;
        regs.event_rxdrdy.is_set(Event::READY)
    }
}

impl uart::UART for UART {
    fn set_client(&self, client: &'static uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, params: uart::UARTParams) {
        self.enable();
        self.set_baud_rate(params.baud_rate);
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        let regs = &*self.registers;

        if tx_len == 0 {
            return;
        }

        self.index.set(1);
        self.len.set(tx_len);

        regs.event_txdrdy.write(Event::READY::CLEAR);
        self.enable_tx_interrupts();
        regs.task_starttx.set(1);
        regs.txd.set(tx_data[0] as u32);
        self.buffer.replace(tx_data);
    }

    // Blocking implementation
    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        let regs = &*self.registers;
        regs.task_startrx.set(1);
        let mut i = 0;
        while i < rx_len {
            while !self.rx_ready() {}
            rx_buffer[i] = regs.rxd.get() as u8;
            i += 1;
        }
    }

    fn abort_receive(&self) {
        unimplemented!()
    }
}
