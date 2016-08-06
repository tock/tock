#![feature(core_intrinsics)]

use core::mem;

//use hil::uart;
//use nvic;
//use chip;
use core::intrinsics;

struct Registers {
    startrx: u32,
    stoprx: u32,
    starttx: u32,
    stoptx: u32,
    reserved0: [u32; 62],
    rxdrdy: u32,    
    reserved1: [u32; 4],
    txdrdy: u32,
    error: u32,
    reserved2: [u32; 119],
    inten: u32,
    intenset: u32,
    intenclr: u32,
    reserved3: [u32; 93],
    errorsrc: u32,
    reserved4: [u32; 31],
    enable: u32,
    reserved5: u32,
    pselrts: u32,
    pseltxd: u32,
    pselcts: u32,
    pselrxd: u32,
    rxd: u32,
    txd: u32,
    reserved6: u32,
    baudrate: u32,
    reserved7: [u32; 71],
    config: u32
}

const UART_BASE: u32 = 0x40002000;
const UART_SIZE: u32 = 0x4;

pub struct UART {
    regs: *mut Registers
}





#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32,
}

impl UART {

    pub const fn new() -> UART {
        UART {
            regs: UART_BASE as *mut Registers
        }
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        match baud_rate {
            1200 => { unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x0004F000);}}
            2400 => { unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x0009D000);}}
            4800 => { unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x0013B000);}}
            9600 => { unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x00275000);}}
            14400 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x003B0000);}}
            19200 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x004EA000);}}
            28800 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x0075F000);}}
            38400 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x009D5000);}}
            57600 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x00EBF000);}}
            76800 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x013A9000);}}
            115200 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x01D7E000);}}
            230400 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x03AFB000);}}
            250000 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x04000000);}}
            460800 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x075F7000);}}
            1000000 =>{ unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x10000000);}}
            _ => { unsafe{intrinsics::volatile_store(&mut regs.baudrate, 0x01D7E000);}} //setting default to 115200
        }
    }
   
    pub fn init(&mut self, baud_rate: u32) {
		let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
		unsafe{intrinsics::volatile_store(&mut regs.enable, 100);}
        self.set_baud_rate(baud_rate);
        unsafe{intrinsics::volatile_store(&mut regs.pselrts, 8);}
        unsafe{intrinsics::volatile_store(&mut regs.pseltxd, 9);}
        unsafe{intrinsics::volatile_store(&mut regs.pselcts, 10);}
        unsafe{intrinsics::volatile_store(&mut regs.pselrxd, 11);}
	}

    fn rx_ready(&self) -> bool {
        let regs : &Registers = unsafe { mem::transmute(self.regs) };
        unsafe{intrinsics::volatile_load(&regs.rxdrdy) & 0b1 != 0 }
    }

    fn tx_ready(&self) -> bool {
        let regs : &Registers = unsafe { mem::transmute(self.regs) };
        unsafe{intrinsics::volatile_load(&regs.txdrdy) & 0b1 != 0}
    }

    pub fn send_byte(&self, byte: u8) {
        while !self.tx_ready() {}
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        unsafe{intrinsics::volatile_store(&mut regs.starttx, 1 as u32);}
        unsafe{intrinsics::volatile_store(&mut regs.txd, byte as u32);}
    }

    fn send_bytes(&self, bytes: &'static mut [u8], len: usize) {
        unimplemented!();
    }

    fn read_byte(&self) -> u8 {
        while !self.rx_ready() {}
        let regs : &Registers = unsafe { mem::transmute(self.regs) };
        //intrinsics::volatile_store(&mut regs.startrx, 1 as u32);
        unsafe{intrinsics::volatile_load(&regs.rxd) as u8}
    }

    fn enable_rx(&self) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        unsafe{intrinsics::volatile_store(&mut regs.startrx, 1);}
    }

    fn disable_rx(&mut self) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        unsafe{intrinsics::volatile_store(&mut regs.stoprx, 1);}
    }

    fn enable_tx(&self) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        unsafe{intrinsics::volatile_store(&mut regs.starttx, 1);}
    }

    fn disable_tx(&mut self) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };
        unsafe{intrinsics::volatile_store(&mut regs.stoptx, 1);}
    }

}
