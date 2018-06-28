use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::common::StaticRef;
use kernel::hil::uart;
use kernel::ReturnCode;
use nrf5x::pinmux::Pinmux;

pub static mut UART0: UART = UART::new();

#[repr(C)]
struct UartRegisters {
    task_startrx: VolatileCell<u32>,
    task_stoprx: VolatileCell<u32>,
    task_starttx: VolatileCell<u32>,
    task_stoptx: VolatileCell<u32>,
    _reserved1: [u32; 3],
    task_suspend: VolatileCell<u32>,
    _reserved2: [u32; 56],
    event_cts: VolatileCell<u32>,
    event_ncts: VolatileCell<u32>,
    event_rxdrdy: VolatileCell<u32>,
    _reserved3: [u32; 4],
    event_txdrdy: VolatileCell<u32>,
    _reserved4: [u32; 1],
    event_error: VolatileCell<u32>,
    _reserved5: [u32; 7],
    event_rxto: VolatileCell<u32>,
    _reserved6: [u32; 46],
    shorts: VolatileCell<u32>,
    _reserved7: [u32; 64],
    intenset: VolatileCell<u32>,
    intenclr: VolatileCell<u32>,
    _reserved8: [u32; 93],
    errorsrc: VolatileCell<u32>,
    _reserved9: [u32; 31],
    enable: VolatileCell<u32>,
    _reserved10: [u32; 1],
    pselrts: VolatileCell<Pinmux>,
    pseltxd: VolatileCell<Pinmux>,
    pselcts: VolatileCell<Pinmux>,
    pselrxd: VolatileCell<Pinmux>,
    rxd: VolatileCell<u32>,
    txd: VolatileCell<u32>,
    _reserved11: [u32; 1],
    baudrate: VolatileCell<u32>,
    _reserved12: [u32; 17],
    config: VolatileCell<u32>,
    _reserved13: [u32; 675],
    power: VolatileCell<u32>,
}

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
    pub fn initialize(&self, tx: Pinmux, rx: Pinmux, cts: Pinmux, rts: Pinmux) {
        let regs = &*self.registers;

        regs.pseltxd.set(tx);
        regs.pselrxd.set(rx);
        regs.pselcts.set(cts);
        regs.pselrts.set(rts);

        self.enable();
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = &*self.registers;
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
        let regs = &*self.registers;
        regs.enable.set(0b100);
    }

    pub fn enable_rx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenset.set(1 << 3 as u32);
    }

    pub fn enable_tx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenset.set(1 << 7 as u32);
    }

    pub fn disable_rx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenclr.set(1 << 3 as u32);
    }

    pub fn disable_tx_interrupts(&self) {
        let regs = &*self.registers;
        regs.intenclr.set(1 << 7 as u32);
    }

    pub fn handle_interrupt(&mut self) {
        let regs = &*self.registers;
        let tx = regs.event_txdrdy.get() != 0;

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
        let regs = &*self.registers;

        self.index.set(1);
        self.len.set(1);

        regs.event_txdrdy.set(0);
        self.enable_tx_interrupts();
        regs.task_starttx.set(1);
        regs.txd.set(byte as u32);
    }

    pub fn tx_ready(&self) -> bool {
        let regs = &*self.registers;
        regs.event_txdrdy.get() & 0b1 != 0
    }

    fn rx_ready(&self) -> bool {
        let regs = &*self.registers;
        regs.event_rxdrdy.get() & 0b1 != 0
    }
}

impl uart::UART for UART {
    fn set_client(&self, client: &'static uart::Client) {
        self.client.set(Some(client));
    }

    fn configure(&self, params: uart::UARTParameters) -> ReturnCode {
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

        self.set_baud_rate(params.baud_rate);

        ReturnCode::SUCCESS
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        let regs = &*self.registers;

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

    fn abort_receive(&self) -> ReturnCode {
        unimplemented!()
    }
}
