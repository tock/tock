use helpers::*;
use core::mem;
use hil::{uart, Controller};
use hil::uart::{Parity, Mode};
use nvic;
use chip;

#[repr(C, packed)]
struct Registers {
    startrx: u32,
    stoprx: u32,
    starttx: u32,
    stoptx: u32,
    rxdrdy: u32,
    txdrdy: u32,
    error: u32,
    inten: u32,
    intenset: u32,
    intenclr: u32,
    errorsrc: u32,
    enable: u32,
    pselrts: u32,
    pseltxd: u32,
    pselcts: u32,
    pselrxd: u32,
    rxd: u32,
    txd: u32,
    baudrate: u32,
    config: u32
}

const SIZE: usize = 0x4000;
const BASE_ADDRESS: usize = 0x40002000;

#[derive(Copy,Clone)]
pub enum Location {
    UART0
}

pub struct UART {
    regs: *mut Registers,
    client: Option<&'static uart::Client>,
    //clock: Clock,
    nvic: nvic::NvicIdx,
}

pub struct UARTParams {
    //pub client: &'static Shared<uart::Client>,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
}

impl Controller for UART {
    type Config = UARTParams;

    fn configure(&self, params: UARTParams) {
        self.set_baud_rate(params.baud_rate);         
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        
        self.enable_rx_interrupts();
    }
}

pub static mut UART0 : UART =
    UART::new(Location::UART0, nvic::NvicIdx::UART0); //clock??


impl UART {
    const fn new(location: Location, nvic: nvic::NvicIdx) //clock??
            -> UART {
        UART {
            regs: (BASE_ADDRESS + (location as usize) * SIZE)
                as *mut Registers,
            //clock: Clock::PBA(clock),
            nvic: nvic,
            client: None,
        }
    }

    pub fn set_client<C: uart::Client>(&mut self, client: &'static C) {
        self.client = Some(client);
    }

    //removed set_dma


    fn set_baud_rate(&self, baud_rate: u32) {
        //let cd = 48000000 / (8 * baud_rate);
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.baudrate, 0x00275000);
    }

    //removed set_mode

    /*fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(self.clock);  //what clock to use??
        }
    }*/

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
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.inten, 1 as u32);
    }

    pub fn enable_tx_interrupts(&mut self) {
        self.enable_nvic();
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.inten, 1 as u32); 
    }

    pub fn disable_rx_interrupts(&mut self) {
        self.disable_nvic();
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.inten, 0 as u32);
    }

    pub fn handle_interrupt(&mut self) {
        use hil::uart::UART;
        if self.rx_ready() {
            let regs : &Registers = unsafe { mem::transmute(self.regs) };
            let c = volatile_load(&regs.rxd) as u8;
            match self.client {
                Some(ref client) => {client.read_done(c)},
                None => {}
            }
        }
    }

  /*  pub fn reset_rx(&mut self) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.cr, 1 << 2); //to reset receiver, no analogue in nrf??
    }*/
}

//removed DMAclient

impl uart::UART for UART {
    fn init(&mut self, params: uart::UARTParams) {
        self.set_baud_rate(params.baud_rate);
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.pselrts, 8);
        volatile_store(&mut regs.pseltxd, 9);
        volatile_store(&mut regs.pselcts, 10);
        volatile_store(&mut regs.pselrxd, 11);
    }

    fn send_byte(&self, byte: u8) {
        while !self.tx_ready() {}
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.txd, byte as u32);
    }

    fn send_bytes(&self, bytes: &'static mut [u8], len: usize) {
        unimplemented!();
    }

    fn rx_ready(&self) -> bool {
        let regs : &Registers = unsafe { mem::transmute(self.regs) };
        volatile_load(&regs.rxdrdy) & 0b1 != 0
    }

    fn tx_ready(&self) -> bool {
        let regs : &Registers = unsafe { mem::transmute(self.regs) };
        volatile_load(&regs.txdrdy) & 0b1 != 0
    }


    fn read_byte(&self) -> u8 {
        while !self.rx_ready() {}
        let regs : &Registers = unsafe { mem::transmute(self.regs) };
        volatile_load(&regs.rxd) as u8
    }

    fn enable_rx(&self) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.startrx, 1);
    }

    fn disable_rx(&mut self) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.stoprx, 1);
    }

    fn enable_tx(&self) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.starttx, 1);
    }

    fn disable_tx(&mut self) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.stoptx, 1);
    }

}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn UART0_Handler() {
    use common::Queue;

    nvic::disable(nvic::NvicIdx::UART0);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nvic::NvicIdx::UART0);
}
