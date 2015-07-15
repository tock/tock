use core::prelude::*;
use core::intrinsics;
use hil;

#[repr(C, packed)]
struct Register {
    val: u32,
    set: u32,
    clear: u32,
    toggle: u32
}

#[repr(C, packed)]
struct RegisterRO {
    val: u32,
    reserved: [u32; 3]
}

#[repr(C, packed)]
struct RegisterRC {
    val: u32,
    reserved0: u32,
    clear: u32,
    reserved1: u32
}

#[repr(C, packed)]
struct GPIOPortRegisters {
    gper: Register,
    pmr0: Register,
    pmr1: Register,
    pmr2: Register,
    oder: Register,
    ovr: Register,
    pvr: RegisterRO,
    puer: Register,
    pder: Register,
    ier: Register,
    imr0: Register,
    imr1: Register,
    gfer: Register,
    ifr: RegisterRC,
    reserved0: [u32; 8],
    ocdr0: Register,
    ocdr1: Register,
    reserved1: [u32; 4],
    osrr0: Register,
    reserved2: [u32; 8],
    ster: Register,
    reserved3: [u32; 4],
    ever: Register,
    reserved4: [u32; 26],
    parameter: u32,
    version: u32,
}

#[derive(Copy,Clone)]
pub enum PeripheralFunction {
    A, B, C, D, E, F, G, H
}


const BASE_ADDRESS: usize = 0x400E1000;
const SIZE: usize = 0x200;

#[derive(Copy,Clone)]
pub enum Pin {
    PA00, PA01, PA02, PA03, PA04, PA05, PA06, PA07,
    PA08, PA09, PA10, PA11, PA12, PA13, PA14, PA15,
    PA16, PA17, PA18, PA19, PA20, PA21, PA22, PA23,
    PA24, PA25, PA26, PA27, PA28, PA29, PA30, PA31,

    PB00, PB01, PB02, PB03, PB04, PB05, PB06, PB07,
    PB08, PB09, PB10, PB11, PB12, PB13, PB14, PB15,
    PB16, PB17, PB18, PB19, PB20, PB21, PB22, PB23,
    PB24, PB25, PB26, PB27, PB28, PB29, PB30, PB31,

    PC00, PC01, PC02, PC03, PC04, PC05, PC06, PC07,
    PC08, PC09, PC10, PC11, PC12, PC13, PC14, PC15,
    PC16, PC17, PC18, PC19, PC20, PC21, PC22, PC23,
    PC24, PC25, PC26, PC27, PC28, PC29, PC30, PC31,
}

pub struct GPIOPin {
    port: &'static mut GPIOPortRegisters,
    pin_mask: u32
}

impl GPIOPin {
    pub fn new(pin: Pin) -> GPIOPin {
        let address = BASE_ADDRESS + ((pin as usize) / 32) * SIZE;
        let pin_number = ((pin as usize) % 32) as u8;

        GPIOPin {
            port: unsafe { intrinsics::transmute(address) },
            pin_mask: 1 << (pin_number as u32)
        }
    }

    pub fn select_peripheral(&mut self, function: PeripheralFunction) {
        let f = function as u32;
        let (bit0, bit1, bit2) = (f & 0b1, (f & 0b10) >> 1, (f & 0b100) >> 2);

        // clear GPIO enable for pin
        volatile!(self.port.gper.clear = self.pin_mask);

        // Set PMR0-2 according to passed in peripheral

        // bradjc: This code doesn't look great, but actually works.
        if bit0 == 0 {
            volatile!(self.port.pmr0.clear = self.pin_mask);
        } else {
            volatile!(self.port.pmr0.set = self.pin_mask);
        }
        if bit1 == 0 {
            volatile!(self.port.pmr1.clear = self.pin_mask);
        } else {
            volatile!(self.port.pmr1.set = self.pin_mask);
        }
        if bit2 == 0 {
            volatile!(self.port.pmr2.clear = self.pin_mask);
        } else {
            volatile!(self.port.pmr2.set = self.pin_mask);
        }
        // bradjc: These register assigns erase previous settings and don't
        //         work.
        // volatile!(self.port.pmr0.val = bit0 << self.pin_mask);
        // volatile!(self.port.pmr1.val = bit1 << self.pin_mask);
        // volatile!(self.port.pmr2.val = bit2 << self.pin_mask);
    }

    pub fn set_ster(&mut self) {
        volatile!(self.port.ster.set = self.pin_mask);
    }
}

impl hil::Controller for GPIOPin {
    type Config = Option<PeripheralFunction>;


    fn configure(&mut self, config: Option<PeripheralFunction>) {
        config.map(|c| {
            self.select_peripheral(c);
        });
    }
}

impl hil::gpio::GPIOPin for GPIOPin {
    fn enable_output(&mut self) {
        volatile!(self.port.gper.set = self.pin_mask);
        volatile!(self.port.oder.set = self.pin_mask);
        volatile!(self.port.ster.clear = self.pin_mask);
    }

    fn read(&self) -> bool {
        (volatile!(self.port.pvr.val) & self.pin_mask) > 0
    }

    fn toggle(&mut self) {
        volatile!(self.port.ovr.toggle = self.pin_mask);
    }

    fn set(&mut self) {
        volatile!(self.port.ovr.set = self.pin_mask);
    }

    fn clear(&mut self) {
        volatile!(self.port.ovr.clear = self.pin_mask);
    }
}
