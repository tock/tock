use PinCnf;
use core::mem;

use peripheral_registers::{UART0_BASE, UART0 as Registers};

pub struct UART {
    regs: *mut Registers
}

impl UART {
    pub const unsafe fn new() -> UART {
        UART {
            regs: UART0_BASE as *mut Registers
        }
    }

    pub fn set_baudrate(&self, baudrate: u32) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };

        let reg_value = match baudrate {
            1200    => 0x0004F000,
            2400    => 0x0009D000,
            4800    => 0x0013B000,
            9600    => 0x00275000,
            14400   => 0x003B0000,
            19200   => 0x004EA000,
            28800   => 0x0075F000,
            38400   => 0x009D5000,
            57600   => 0x00EBF000,
            76800   => 0x013A9000,
            115200  => 0x01D7E000,
            230400  => 0x03AFB000,
            250000  => 0x04000000,
            460800  => 0x075F7000,
            1000000 => 0x10000000,
            _       => 0x01D7E000 // Default to 115200
        };

        regs.baudrate.set(reg_value);
    }

    pub fn init(&mut self, txd: PinCnf, rxd: PinCnf, rts: PinCnf, cts: PinCnf) {
		let regs : &mut Registers = unsafe { mem::transmute(self.regs) };

        regs.pseltxd.set(txd);
        regs.pselrxd.set(rxd);
        regs.pselrts.set(rts);
        regs.pselcts.set(cts);
	}

    pub fn rx_ready(&self) -> bool {
        let regs : &Registers = unsafe { mem::transmute(self.regs) };
        regs.events_rxdrdy.get() & 0b1 != 0
    }

    fn tx_ready(&self) -> bool {
        let regs : &Registers = unsafe { mem::transmute(self.regs) };
        regs.events_txdrdy.get() == 1
    }

    pub fn send_bytes(&self, bytes: &[u8]) {
        let regs : &mut Registers = unsafe { mem::transmute(self.regs) };

		regs.enable.set(4);

        regs.tasks_starttx.set(1);

        for c in bytes.iter() {
            regs.events_txdrdy.set(0);
            regs.txd.set(*c as u32);
            while !self.tx_ready() {}
        }
        regs.tasks_stoptx.set(1);

		regs.enable.set(0);
    }
}

