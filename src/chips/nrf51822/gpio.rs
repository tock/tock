use core::mem;
use core::cell::Cell;
use core::ops::{Index, IndexMut};
use hil;

use common::take_cell::TakeCell;

use peripheral_registers::{GPIO_BASE, GPIO};

struct GpioteRegisters {
    out0: u32, // 0x0
    out1: u32, // 0x4
    out2: u32, // 0x8
    out3: u32, // 0xC
    reserved0: [u32; 0xF0],
    in0:  u32, // 0x100
    in1:  u32, // 0x104
    in2:  u32, // 0x108
    in3:  u32, // 0x10C
    reserved1: [u32; 0x70],
    port: u32, // 0x17C,
    reserved2: [u32; 0x180],
    inten:    u32, // 0x300
    intenset: u32, // 0x304
    intenclr: u32, // 0x308
    reserved3: [u32; 0x204],
    config0:  u32, // 0x510
    config1:  u32, // 0x514
    config2:  u32, // 0x518
    config3:  u32, // 0x51C
}

const GPIOTE_BASE: u32 = 0x40006000;

#[allow(non_snake_case)]
fn GPIO() -> &'static GPIO {
    unsafe { mem::transmute(GPIO_BASE as usize) }
}

pub struct GPIOPin {
    pin: u8,
    client_data: Cell<usize>,
    client: TakeCell<&'static hil::gpio::Client>,
}

impl GPIOPin {
    const fn new(pin: u8) -> GPIOPin {
        GPIOPin {
            pin: pin,
            client_data: Cell::new(0),
            client: TakeCell::empty(),
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.replace(client);
    }

}

impl hil::gpio::GPIOPin for GPIOPin {
    fn enable_output(&self) {
        // bit 0: set as output
        // bit 1: disconnect input buffer
        // bit 2-3: no pullup/down
        // bit 8-10: drive configruation
        // bit 16-17: sensing
        GPIO().pin_cnf[self.pin as usize].set((1 << 0) | (1 << 1) | (0 << 2) | (0 << 8) | (0 << 16));
    }

    // Configuration constants stolen from 
    // mynewt/hw/mcu/nordic/nrf51xxx/include/mcu/nrf51_bitfields.h
    fn enable_input(&self, _mode: hil::gpio::InputMode) {
        let conf = match _mode {
            hil::gpio::InputMode::PullUp   => 0x3 << 2,
            hil::gpio::InputMode::PullDown => 0x1 << 2,
            hil::gpio::InputMode::PullNone => 0,
        };
        GPIO().pin_cnf[self.pin as usize].set(conf);
    }

    // Not clk
    fn disable(&self) {
        self.enable_input(hil::gpio::InputMode::PullNone);
    }

    fn set(&self) {
        GPIO().outset.set(1 << self.pin);
    }

    fn clear(&self) {
        GPIO().outclr.set(1 << self.pin);
    }

    fn toggle(&self) {
        // TODO: check need for a atomic XOR operator
        GPIO().out.set((1 << self.pin) ^ GPIO().out.get());
    }

    fn read(&self) -> bool {
        GPIO().in_.get() & (1 << self.pin) != 0
    }

    fn enable_interrupt(&self, _client_data: usize, _mode: hil::gpio::InterruptMode) {
       self.client_data.set(_client_data);
       let mode_bits = match _mode {
           hil::gpio::InterruptMode::Change      => 0,
           hil::gpio::InterruptMode::RisingEdge  => 0,
           hil::gpio::InterruptMode::FallingEdge => 0,
       };
    }

    fn disable_interrupt(&self) {
        unimplemented!();
    }
}

pub struct Port {
    pins: [GPIOPin; 32]
}

impl Index<usize> for Port {
    type Output = GPIOPin;

    fn index(&self, index: usize) -> &GPIOPin {
        &self.pins[index]
    }
}

impl IndexMut<usize> for Port {
    fn index_mut(&mut self, index: usize) -> &mut GPIOPin {
        &mut self.pins[index]
    }
}

pub static mut PORT : Port = Port {
    pins: [
        GPIOPin::new(0), GPIOPin::new(1), GPIOPin::new(2), GPIOPin::new(3),
        GPIOPin::new(4), GPIOPin::new(5), GPIOPin::new(6), GPIOPin::new(7),
        GPIOPin::new(8), GPIOPin::new(9), GPIOPin::new(10), GPIOPin::new(11),
        GPIOPin::new(12), GPIOPin::new(13), GPIOPin::new(14), GPIOPin::new(15),
        GPIOPin::new(16), GPIOPin::new(17), GPIOPin::new(18), GPIOPin::new(19),
        GPIOPin::new(20), GPIOPin::new(21), GPIOPin::new(22), GPIOPin::new(23),
        GPIOPin::new(24), GPIOPin::new(25), GPIOPin::new(26), GPIOPin::new(27),
        GPIOPin::new(28), GPIOPin::new(29), GPIOPin::new(30), GPIOPin::new(31),
    ],
};
