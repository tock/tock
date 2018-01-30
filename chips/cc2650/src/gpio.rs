use core::cell::Cell;
use core::ops::{Index, IndexMut};
use ioc;
use kernel::common::VolatileCell;
use kernel::hil;

const NUM_PINS: usize = 32;

#[repr(C)]
pub struct GPIO {
    _reserved0: [u8; 0x90],
    pub dout_set: VolatileCell<u32>,
    _reserved1: [u8; 0xC],
    pub dout_clr: VolatileCell<u32>,
    _reserved2: [u8; 0xC],
    pub dout_tgl: VolatileCell<u32>,
    _reserved3: [u8; 0xC],
    pub din: VolatileCell<u32>,
    _reserved4: [u8; 0xC],
    pub doe: VolatileCell<u32>,
    _reserved5: [u8; 0xC],
    pub evflags: VolatileCell<u32>,
}

pub const GPIO_BASE: usize = 0x4002_2000;

#[allow(non_snake_case)]
fn GPIO() -> &'static GPIO {
    unsafe { &*(GPIO_BASE as *const GPIO) }
}

pub struct GPIOPin {
    pin: usize,
    pin_mask: u32,
    client_data: Cell<usize>,
    client: Cell<Option<&'static hil::gpio::Client>>,
}

impl GPIOPin {
    const fn new(pin: usize) -> GPIOPin {
        GPIOPin {
            pin: pin,
            pin_mask: 1 << (pin % NUM_PINS),
            client_data: Cell::new(0),
            client: Cell::new(None),
        }
    }

    fn enable_gpio(&self) {
        ioc::IOCFG[self.pin].enable_gpio();
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    pub fn handle_interrupt(&self) {
        self.client.get().map(|client| {
            client.fired(self.client_data.get());
        });
    }
}

impl hil::gpio::PinCtl for GPIOPin {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        ioc::IOCFG[self.pin].set_input_mode(mode);
    }
}

impl hil::gpio::Pin for GPIOPin {
    fn make_output(&self) {
        self.enable_gpio();
        // Disable input in the io configuration
        ioc::IOCFG[self.pin].enable_output();
        // Enable data output
        GPIO().doe.set(GPIO().doe.get() | self.pin_mask);
    }

    fn make_input(&self) {
        self.enable_gpio();
        ioc::IOCFG[self.pin].enable_input();
    }

    fn disable(&self) {
        hil::gpio::PinCtl::set_input_mode(self, hil::gpio::InputMode::PullNone);
    }

    fn set(&self) {
        GPIO().dout_set.set(self.pin_mask);
    }

    fn clear(&self) {
        GPIO().dout_clr.set(self.pin_mask);
    }

    fn toggle(&self) {
        GPIO().dout_tgl.set(self.pin_mask);
    }

    fn read(&self) -> bool {
        GPIO().din.get() & self.pin_mask != 0
    }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        self.client_data.set(client_data);
        ioc::IOCFG[self.pin].enable_interrupt(mode);
    }

    fn disable_interrupt(&self) {
        ioc::IOCFG[self.pin].disable_interrupt();
    }
}

pub struct Port {
    pins: [GPIOPin; NUM_PINS],
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

impl Port {
    pub fn handle_interrupt(&self) {
        let evflags = &GPIO().evflags;
        let mut pin_mask = evflags.get();

        evflags.set(pin_mask); // Clear interrupts

        loop {
            let pin = pin_mask.trailing_zeros() as usize;
            if pin < self.pins.len() {
                pin_mask &= !(1 << pin);
                self.pins[pin].handle_interrupt();
            } else {
                break;
            }
        }
    }
}

pub static mut PORT: Port = Port {
    pins: [
        GPIOPin::new(0),
        GPIOPin::new(1),
        GPIOPin::new(2),
        GPIOPin::new(3),
        GPIOPin::new(4),
        GPIOPin::new(5),
        GPIOPin::new(6),
        GPIOPin::new(7),
        GPIOPin::new(8),
        GPIOPin::new(9),
        GPIOPin::new(10),
        GPIOPin::new(11),
        GPIOPin::new(12),
        GPIOPin::new(13),
        GPIOPin::new(14),
        GPIOPin::new(15),
        GPIOPin::new(16),
        GPIOPin::new(17),
        GPIOPin::new(18),
        GPIOPin::new(19),
        GPIOPin::new(20),
        GPIOPin::new(21),
        GPIOPin::new(22),
        GPIOPin::new(23),
        GPIOPin::new(24),
        GPIOPin::new(25),
        GPIOPin::new(26),
        GPIOPin::new(27),
        GPIOPin::new(28),
        GPIOPin::new(29),
        GPIOPin::new(30),
        GPIOPin::new(31),
    ],
};
