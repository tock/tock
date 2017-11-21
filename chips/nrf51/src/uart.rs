use core::cell::Cell;
use kernel::common::VolatileCell;
use kernel::common::take_cell::TakeCell;
use kernel::hil::uart;
use nrf5x::pinmux::Pinmux;

#[repr(C, packed)]
pub struct Registers {
    pub task_startrx: VolatileCell<u32>,
    pub task_stoprx: VolatileCell<u32>,
    pub task_starttx: VolatileCell<u32>,
    pub task_stoptx: VolatileCell<u32>,
    _reserved1: [u32; 3],
    pub task_suspend: VolatileCell<u32>,
    _reserved2: [u32; 56],
    pub event_cts: VolatileCell<u32>,
    pub event_ncts: VolatileCell<u32>,
    pub event_rxdrdy: VolatileCell<u32>,
    _reserved3: [u32; 4],
    pub event_txdrdy: VolatileCell<u32>,
    _reserved4: [u32; 1],
    pub event_error: VolatileCell<u32>,
    _reserved5: [u32; 7],
    pub event_rxto: VolatileCell<u32>,
    _reserved6: [u32; 46],
    pub shorts: VolatileCell<u32>,
    _reserved7: [u32; 64],
    pub intenset: VolatileCell<u32>,
    pub intenclr: VolatileCell<u32>,
    _reserved8: [u32; 93],
    pub errorsrc: VolatileCell<u32>,
    _reserved9: [u32; 31],
    pub enable: VolatileCell<u32>,
    _reserved10: [u32; 1],
    pub pselrts: VolatileCell<Pinmux>,
    pub pseltxd: VolatileCell<Pinmux>,
    pub pselcts: VolatileCell<Pinmux>,
    pub pselrxd: VolatileCell<Pinmux>,
    pub rxd: VolatileCell<u32>,
    pub txd: VolatileCell<u32>,
    _reserved11: [u32; 1],
    pub baudrate: VolatileCell<u32>,
    _reserved12: [u32; 17],
    pub config: VolatileCell<u32>,
    _reserved13: [u32; 675],
    pub power: VolatileCell<u32>,
}

const UART_BASE: u32 = 0x40002000;

pub struct UART {
    regs: *const Registers,
    client: Cell<Option<&'static uart::Client>>,
    buffer: TakeCell<'static, [u8]>,
    len: Cell<usize>,
    index: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32,
}

pub static mut UART0: UART = UART::new();

// This UART implementation uses pins 8-11:
//   pin  8: RTS
//   pin  9: TX
//   pin 10: CTS
//   pin 11: RX
impl UART {
    pub const fn new() -> UART {
        UART {
            regs: UART_BASE as *mut Registers,
            client: Cell::new(None),
            buffer: TakeCell::empty(),
            len: Cell::new(0),
            index: Cell::new(0),
        }
    }

    pub fn configure(&self, tx: Pinmux, rx: Pinmux, cts: Pinmux, rts: Pinmux) {
        let regs = unsafe { &*self.regs };

        regs.pseltxd.set(tx);
        regs.pselrxd.set(rx);
        regs.pselcts.set(cts);
        regs.pselrts.set(rts);
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = unsafe { &*self.regs };
        match baud_rate {
            1200 => regs.baudrate.set(0x0004F000),
            2400 => regs.baudrate.set(0x0009D000),
            4800 => regs.baudrate.set(0x0013B000),
            9600 => regs.baudrate.set(0x00275000),
            14400 => regs.baudrate.set(0x003B0000),
            19200 => regs.baudrate.set(0x004EA000),
            28800 => regs.baudrate.set(0x0075F000),
            38400 => regs.baudrate.set(0x009D5000),
            57600 => regs.baudrate.set(0x00EBF000),
            76800 => regs.baudrate.set(0x013A9000),
            115200 => regs.baudrate.set(0x01D7E000),
            230400 => regs.baudrate.set(0x03AFB000),
            250000 => regs.baudrate.set(0x04000000),
            460800 => regs.baudrate.set(0x075F7000),
            1000000 => regs.baudrate.set(0x10000000),
            _ => regs.baudrate.set(0x01D7E000), //setting default to 115200
        }
    }

    pub fn enable(&self) {
        let regs = unsafe { &*self.regs };
        regs.enable.set(0b100);
    }

    pub fn enable_rx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(1 << 3 as u32);
    }

    pub fn enable_tx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(1 << 7 as u32);
    }

    pub fn disable_rx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(1 << 3 as u32);
    }

    pub fn disable_tx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(1 << 7 as u32);
    }

    pub fn handle_interrupt(&mut self) {
        let regs = unsafe { &*self.regs };
        // let rx = regs.event_rxdrdy.get() != 0;
        let tx = regs.event_txdrdy.get() != 0;

        // if rx {
        //     let val = regs.rxd.get();
        //     self.client.map(|client| {
        //         client.read_done(val as u8);
        //     });
        // }
        if tx {
            regs.event_txdrdy.set(0 as u32);

            if self.len.get() == self.index.get() {
                regs.task_stoptx.set(1 as u32);

                // Signal client write done
                self.client.get().map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.transmit_complete(buffer, uart::Error::CommandComplete);
                    });
                });

                return;
            }

            self.buffer.map(|buffer| {
                regs.event_txdrdy.set(0 as u32);
                regs.txd.set(buffer[self.index.get()] as u32);
                let next_index = self.index.get() + 1;
                self.index.set(next_index);
            });
        }
    }

    pub unsafe fn send_byte(&self, byte: u8) {
        let regs = &*self.regs;

        self.index.set(1);
        self.len.set(1);

        regs.event_txdrdy.set(0);
        self.enable_tx_interrupts();
        regs.task_starttx.set(1);
        regs.txd.set(byte as u32);
    }

    pub fn tx_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.event_txdrdy.get() & 0b1 != 0
    }

    fn rx_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.event_rxdrdy.get() & 0b1 != 0
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
        let regs = unsafe { &*self.regs };

        if tx_len == 0 {
            return;
        }

        self.index.set(1);
        self.len.set(tx_len);

        regs.event_txdrdy.set(0);
        self.enable_tx_interrupts();
        regs.task_starttx.set(1);
        regs.txd.set(tx_data[0] as u32);
        self.buffer.replace(tx_data);
    }

    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        let regs = unsafe { &*self.regs };
        regs.task_startrx.set(1);
        let mut i = 0;
        while i < rx_len {
            while !self.rx_ready() {}
            rx_buffer[i] = regs.rxd.get() as u8;
            i += 1;
        }
    }
}
