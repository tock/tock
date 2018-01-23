use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::hil;
use peripheral_registers::{GPIO, GPIO_BASE,
                           IOC, IOC_BASE, IOC_IE, IOC_PULL_CTL,
                           IOC_EDGE_DET, IOC_EDGE_IRQ_EN};

const NUM_PINS: usize = 32;

#[allow(non_snake_case)]
fn IOC() -> &'static IOC { unsafe { &*(IOC_BASE as *const IOC) } }

#[allow(non_snake_case)]
fn GPIO() -> &'static GPIO { unsafe { &*(GPIO_BASE as *const GPIO) } }

pub struct GPIOPin {
    pin: u8,
    pin_mask: u32,
    client_data: Cell<usize>,
    client: Cell<Option<&'static hil::gpio::Client>>,
}

impl GPIOPin {
    const fn new(pin: u8) -> GPIOPin {
        GPIOPin {
            pin,
            pin_mask: 1 << ((pin as usize) % NUM_PINS),
            client_data: Cell::new(0),
            client: Cell::new(None),
        }
    }

    fn enable_gpio(&self) {
        let pin_cnf = &IOC().iocfg[self.pin as usize];
        pin_cnf.set(pin_cnf.get() & !0x3F); // Clear lower 6 bits
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }
}

impl hil::gpio::PinCtl for GPIOPin {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        let conf = match mode {
            hil::gpio::InputMode::PullUp => 2,
            hil::gpio::InputMode::PullDown => 1,
            hil::gpio::InputMode::PullNone => 3,
        };
        let pin_cnf = &IOC().iocfg[self.pin as usize];
        pin_cnf.set(pin_cnf.get() & !(0b11 << IOC_PULL_CTL) | (conf << IOC_PULL_CTL));
    }
}

impl hil::gpio::Pin for GPIOPin {
    fn make_output(&self) {
        self.enable_gpio();
        // Disable input
        let pin_cnf = &IOC().iocfg[self.pin as usize];
        pin_cnf.set(pin_cnf.get() & !(1 << IOC_IE));
        // Enable data output
        GPIO().doe.set(GPIO().doe.get() | self.pin_mask);
    }

    fn make_input(&self) {
        self.enable_gpio();
        let pin_cnf = &IOC().iocfg[self.pin as usize];
        pin_cnf.set(pin_cnf.get() | 1 << IOC_IE);
    }

    fn disable(&self) {
        hil::gpio::PinCtl::set_input_mode(self, hil::gpio::InputMode::PullNone);
    }

    fn set(&self) { GPIO().dout_set.set(self.pin_mask); }

    fn clear(&self) { GPIO().dout_clr.set(self.pin_mask); }

    fn toggle(&self) { GPIO().dout_tgl.set(self.pin_mask); }

    fn read(&self) -> bool { GPIO().din.get() & self.pin_mask != 0 }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        self.client_data.set(client_data);
        let pin_cnf = &IOC().iocfg[self.pin as usize];

        let mode_bits = match mode {
            hil::gpio::InterruptMode::EitherEdge => 3 << IOC_EDGE_DET,
            hil::gpio::InterruptMode::RisingEdge => 2 << IOC_EDGE_DET,
            hil::gpio::InterruptMode::FallingEdge => 1 << IOC_EDGE_DET,
        };

        pin_cnf.set(pin_cnf.get() & !(0b11 << IOC_EDGE_DET) | mode_bits | 1 << IOC_EDGE_IRQ_EN);
    }

    fn disable_interrupt(&self) {
        let pin_cnf = &IOC().iocfg[self.pin as usize];
        pin_cnf.set(pin_cnf.get() & !(1 << IOC_EDGE_IRQ_EN));
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
