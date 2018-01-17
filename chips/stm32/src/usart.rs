use core::cell::Cell;
use core::mem;
use gpio;
use kernel;
use kernel::common::VolatileCell;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::hil::Controller;
use nvic;
use rcc;

#[repr(C, packed)]
struct USARTRegisters {
    sr: VolatileCell<u32>,
    dr: VolatileCell<u32>,
    brr: VolatileCell<u32>,
    cr1: VolatileCell<u32>,
    cr2: VolatileCell<u32>,
    cr3: VolatileCell<u32>,
    gtpr: VolatileCell<u32>,
}

const USART_BASE_ADDRS: [*mut USARTRegisters; 5] = [0x40013800 as *mut USARTRegisters,
                                                    0x40004400 as *mut USARTRegisters,
                                                    0x40004800 as *mut USARTRegisters,
                                                    0x40004c00 as *mut USARTRegisters,
                                                    0x40005000 as *mut USARTRegisters];

pub struct USART {
    registers: *mut USARTRegisters,
    clock: rcc::Clock,
    nvic: nvic::NvicIdx,
    rx: Cell<Option<&'static gpio::GPIOPin>>,
    tx: Cell<Option<&'static gpio::GPIOPin>>,
    client: Cell<Option<&'static kernel::hil::uart::Client>>,
    buffer: TakeCell<'static, [u8]>,
    remaining: Cell<usize>,
    offset: Cell<usize>,
}

pub static mut USART1: USART = USART::new(USART_BASE_ADDRS[0],
                                          rcc::Clock::APB2(rcc::APB2Clock::USART1),
                                          nvic::NvicIdx::USART1);

pub static mut USART2: USART = USART::new(USART_BASE_ADDRS[1],
                                          rcc::Clock::APB1(rcc::APB1Clock::USART2),
                                          nvic::NvicIdx::USART2);

pub static mut USART3: USART = USART::new(USART_BASE_ADDRS[2],
                                          rcc::Clock::APB1(rcc::APB1Clock::USART3),
                                          nvic::NvicIdx::USART3);

impl USART {
    const fn new(base_addr: *mut USARTRegisters, clock: rcc::Clock, nvic: nvic::NvicIdx) -> USART {
        USART {
            registers: base_addr,
            clock: clock,
            nvic: nvic,
            rx: Cell::new(None),
            tx: Cell::new(None),
            client: Cell::new(None),
            buffer: TakeCell::empty(),
            remaining: Cell::new(0),
            offset: Cell::new(0),
        }
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        let clk = rcc::get_frequency(self.clock) / baud_rate;
        let mantissa = clk / 16;
        let fraction = clk - (mantissa * 16);
        regs.brr.set(((mantissa & 0x0fff) << 4) | (fraction & 0x0f));
    }

    pub fn specify_pins(&self, rx: &'static gpio::GPIOPin, tx: &'static gpio::GPIOPin) {
        self.rx.set(Some(rx));
        self.tx.set(Some(tx));
    }

    fn enable(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        unsafe {
            rcc::enable_clock(self.clock);
            nvic::enable(self.nvic);
        }
        regs.cr1.set(regs.cr1.get() | (1 << 13)); // UE
    }

    fn enable_tx_interrupts(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.cr1.set(regs.cr1.get() | (1 << 6)); // TCIE
    }

    fn disable_tx_interrupts(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.cr1.set(regs.cr1.get() & !(1 << 6)); // TCIE
    }

    pub fn enable_tx(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.tx
            .get()
            .unwrap()
            .configure(gpio::Mode::Output2MHz(gpio::OutputMode::AlternatePushPull));
        self.enable();
        regs.cr1.set(regs.cr1.get() | (1 << 3)); // TE
    }

    pub fn enable_rx(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        self.rx.get().unwrap().configure(gpio::Mode::Input(gpio::InputMode::Floating));
        self.enable();
        regs.cr1.set(regs.cr1.get() | (1 << 2)); // RE
    }

    pub fn send_byte(&self, byte: u8) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        while regs.sr.get() & (1 << 7) == 0 {} // TXE
        regs.dr.set(byte as u32);
    }

    pub fn tx_ready(&self) -> bool {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        regs.sr.get() & (1 << 6) != 0 // TC
    }

    fn send_next(&self) {
        self.buffer.map(|buffer| { self.send_byte(buffer[self.offset.get()]); });
    }

    pub fn handle_interrupt(&self) {
        let regs: &mut USARTRegisters = unsafe { mem::transmute(self.registers) };
        // check if caused by TC
        if regs.sr.get() & (1 << 6) != 0 {
            self.remaining.set(self.remaining.get() - 1);
            self.offset.set(self.offset.get() + 1);
            if self.remaining.get() > 0 {
                self.send_next();
            } else {
                self.disable_tx_interrupts();
                self.client.get().map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.transmit_complete(buffer, kernel::hil::uart::Error::CommandComplete);
                    });
                });
            }
        }
    }
}

impl hil::uart::UART for USART {
    fn set_client(&self, client: &'static hil::uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, params: hil::uart::UARTParams) {
        self.enable();
        self.set_baud_rate(params.baud_rate)
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        self.buffer.replace(tx_data);
        self.offset.set(0);
        self.remaining.set(tx_len);
        self.enable_tx();
        self.enable_tx_interrupts();
        self.send_next();
    }

    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        unimplemented!()
    }
}

interrupt_handler!(usart1_handler, USART1);
interrupt_handler!(usart2_handler, USART2);
interrupt_handler!(usart3_handler, USART3);
