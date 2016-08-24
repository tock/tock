
use core::mem;
use dma::{DMAChannel, DMAClient, DMAPeripheral};
use helpers::*;
use hil::{uart, Controller};
use hil::uart::{Parity, Mode};
use nvic;
use pm::{self, Clock, PBAClock};

#[repr(C, packed)]
struct Registers {
    cr: u32,
    mr: u32,
    ier: u32,
    idr: u32,
    imr: u32,
    csr: u32,
    rhr: u32,
    thr: u32,
    brgr: u32, // 0x20
    rtor: u32,
    ttgr: u32,
    reserved0: [u32; 5],
    fidi: u32, // 0x40
    ner: u32,
    reserved1: u32,
    ifr: u32,
    man: u32,
    linmr: u32,
    linir: u32,
    linbrr: u32,
    wpmr: u32,
    wpsr: u32,
    version: u32,
}

const SIZE: usize = 0x4000;
const BASE_ADDRESS: usize = 0x40024000;

#[derive(Copy,Clone)]
pub enum Location {
    USART0,
    USART1,
    USART2,
    USART3,
}

pub struct USART {
    regs: *mut Registers,
    client: Option<&'static uart::Client>,
    clock: Clock,
    nvic: nvic::NvicIdx,
    dma_peripheral: DMAPeripheral,
    dma: Option<&'static mut DMAChannel>,
}

pub struct USARTParams {
    // pub client: &'static Shared<uart::Client>,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
    pub mode: Mode,
}

impl Controller for USART {
    type Config = USARTParams;

    fn configure(&self, params: USARTParams) {
        //   self.client = Some(params.client.borrow_mut());
        let chrl = ((params.data_bits - 1) & 0x3) as u32;
        let mode =
            (params.mode as u32) /* mode */
            | 0 << 4 /*USCLKS*/
            | chrl << 6 /* Character Length */
            | (params.parity as u32) << 9 /* Parity */
            | 0 << 12 /* Number of stop bits = 1 */
            | 1 << 19 /* Oversample at 8 times baud rate */;

        self.enable_clock();
        self.set_baud_rate(params.baud_rate);
        self.set_mode(mode);
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.ttgr, 4);
        self.enable_rx_interrupts();
    }
}

pub static mut USART0: USART =
    USART::new(Location::USART0, PBAClock::USART0, nvic::NvicIdx::USART0);
pub static mut USART1: USART =
    USART::new(Location::USART1, PBAClock::USART1, nvic::NvicIdx::USART1);
pub static mut USART2: USART =
    USART::new(Location::USART2, PBAClock::USART2, nvic::NvicIdx::USART2);
pub static mut USART3: USART =
    USART::new(Location::USART3, PBAClock::USART3, nvic::NvicIdx::USART3);

impl USART {
    const fn new(location: Location, clock: PBAClock, nvic: nvic::NvicIdx) -> USART {
        USART {
            regs: (BASE_ADDRESS + (location as usize) * SIZE) as *mut Registers,
            clock: Clock::PBA(clock),
            nvic: nvic,
            dma: None,
            dma_peripheral: DMAPeripheral::USART0_RX, // Set to some default.
            // This is updated when a
            // real DMA is configured.
            client: None,
        }
    }

    pub fn set_client<C: uart::Client>(&mut self, client: &'static C) {
        self.client = Some(client);
    }

    pub fn set_dma(&mut self, dma: &'static mut DMAChannel, dma_peripheral: DMAPeripheral) {
        self.dma = Some(dma);
        self.dma_peripheral = dma_peripheral;
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let cd = 48000000 / (8 * baud_rate);
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.brgr, cd);
    }

    fn set_mode(&self, mode: u32) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.mr, mode);
    }

    fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(self.clock);
        }
    }

    fn enable_nvic(&self) {
        unsafe {
            nvic::enable(self.nvic);
        }
    }

    fn disable_nvic(&self) {
        unsafe {
            nvic::disable(self.nvic);
        }
    }

    pub fn enable_rx_interrupts(&self) {
        self.enable_nvic();
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.ier, 1 as u32);
    }

    pub fn enable_tx_interrupts(&mut self) {
        self.enable_nvic();
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.ier, 2 as u32);
    }

    pub fn disable_rx_interrupts(&mut self) {
        self.disable_nvic();
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.idr, 1 as u32);
    }

    pub fn handle_interrupt(&mut self) {
        use hil::uart::UART;
        if self.rx_ready() {
            let regs: &Registers = unsafe { mem::transmute(self.regs) };
            let c = volatile_load(&regs.rhr) as u8;
            match self.client {
                Some(ref client) => client.read_done(c),
                None => {}
            }
        }
    }

    pub fn reset_rx(&mut self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.cr, 1 << 2);
    }
}

impl DMAClient for USART {
    fn xfer_done(&mut self, _pid: usize) {
        let buffer = match self.dma.as_mut() {
            Some(dma) => {
                let buf = dma.abort_xfer();
                dma.disable();
                buf
            }
            None => None,
        };
        self.client.as_ref().map(move |c| {
            buffer.map(|buf| c.write_done(buf));
        });
    }
}

impl uart::UART for USART {
    fn init(&mut self, params: uart::UARTParams) {
        let chrl = ((params.data_bits - 1) & 0x3) as u32;
        let mode =
            (params.mode as u32) /* mode */
            | 0 << 4 /*USCLKS*/
            | chrl << 6 /* Character Length */
            | (params.parity as u32) << 9 /* Parity */
            | 0 << 12 /* Number of stop bits = 1 */
            | 1 << 19 /* Oversample at 8 times baud rate */;

        self.enable_clock();
        self.set_baud_rate(params.baud_rate);
        self.set_mode(mode);
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.ttgr, 4);
    }

    fn send_byte(&self, byte: u8) {
        while !self.tx_ready() {}
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.thr, byte as u32);
    }

    fn send_bytes(&self, bytes: &'static mut [u8], len: usize) {
        self.dma.as_ref().map(move |dma| {
            dma.enable();
            dma.do_xfer(self.dma_peripheral, bytes, len);
        });
    }

    fn rx_ready(&self) -> bool {
        let regs: &Registers = unsafe { mem::transmute(self.regs) };
        volatile_load(&regs.csr) & 0b1 != 0
    }

    fn tx_ready(&self) -> bool {
        let regs: &Registers = unsafe { mem::transmute(self.regs) };
        volatile_load(&regs.csr) & 0b10 != 0
    }


    fn read_byte(&self) -> u8 {
        while !self.rx_ready() {}
        let regs: &Registers = unsafe { mem::transmute(self.regs) };
        volatile_load(&regs.rhr) as u8
    }

    fn enable_rx(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.cr, 1 << 4);
    }

    fn disable_rx(&mut self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.cr, 1 << 5);
    }

    fn enable_tx(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.cr, 1 << 6);
    }

    fn disable_tx(&mut self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.cr, 1 << 7);
    }
}

interrupt_handler!(usart0_handler, USART0);
interrupt_handler!(usart1_handler, USART1);
interrupt_handler!(usart2_handler, USART2);
interrupt_handler!(usart3_handler, USART3);
