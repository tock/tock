
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

use peripheral_registers::{UART_BASE, UART}; 


const SIZE: usize = 0x4000;
const BASE_ADDRESS: usize = 0x40002000;

#[derive(Copy,Clone)]
pub enum Location {
    UART0
}

pub struct UART {
    regs: *mut Registers,
    client: Option<&'static uart::Client>,
    clock: Clock,
    nvic: nvic::NvicIdx,
}

pub struct UARTParams {
    //pub client: &'static Shared<uart::Client>,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
    pub mode: Mode,
}

fn UART() -> &'static UART {
    unsafe { mem::transmute(UART_BASE as usize) }
}





impl hil::uart::UART for UART{
	fn init(&mut self, params: UARTParams);
    
    fn send_byte(&self, byte: u8){
    	while !self.tx_ready() {}
    	UART().txd.set (byte);
    	UART().txdrdy.set(0);
    }
    
    fn send_bytes(&self, bytes: &'static mut [u8], len: usize);
    fn read_byte(&self) -> u8;
    fn rx_ready(&self) -> bool;
    fn tx_ready(&self) -> bool {
    	let regs : &Registers = unsafe { mem::transmute(self.regs) };
        volatile_load(&regs.csr) & 0b10 != 0
    }
    fn enable_rx(&self);
    fn disable_rx(&mut self);
    fn enable_tx(&self);
    fn disable_tx(&mut self);
}







impl UART {
	
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
		UART().intenset.set(2);
	}

	pub fn enable_tx_interrupts(&mut self) {
        self.enable_nvic();
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.ier, 2 as u32); // set to what??
    }
}


impl Controller for UART {
    type Config = UARTParams;

    fn configure(&self, params: UARTParams) {
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
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.ttgr, 4);
        //self.enable_rx_interrupts();
    }
}

pub static mut UART0 : UART =
    UART::new(Location::UART0, PBAClock::UART0, nvic::NvicIdx::UART0);

impl uart::UART for UART{
	fn init(&mut self, params: UARTParams){
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
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.ttgr, 4);
	}

    fn send_byte(&self, byte: u8){
    	while !self.tx_ready() {}
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.TXD, byte as u32);
    }
    
    fn tx_ready(&self) -> bool {
    	let regs : &Registers = unsafe { mem::transmute(self.regs) };
        volatile_load(&regs.TXDRDY) & 0b10 != 0
    }

    fn enable_tx(&self){
    	let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.ENABLE, 1 << 6);
    }

    fn disable_tx(&mut self){
    	let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        volatile_store(&mut regs.ENABLE, 1 << 7);
    }
}