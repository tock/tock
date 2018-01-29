//! GPIO and GPIOTE (task and events), nRF5x-family
//!
//! ### Author
//! * Philip Levis <pal@cs.stanford.edu>
//! * Date: August 18, 2016

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::common::VolatileCell;
use kernel::hil;
use peripheral_registers::{GPIO, GPIO_BASE};

#[cfg(feature = "nrf51")]
const NUM_GPIOTE: usize = 4;
#[cfg(feature = "nrf52")]
const NUM_GPIOTE: usize = 8;

/// The nRF5x doesn't automatically provide GPIO interrupts. Instead, to receive
/// interrupts from a GPIO line, you must allocate a GPIOTE (GPIO Task and
/// Event) channel, and bind the channel to the desired pin. There are 4
/// channels for the nrf51 and 8 channels for the nrf52. This means that
/// requesting an interrupt can fail, if they are all already allocated.
#[repr(C)]
struct GpioteRegisters {
    out: [VolatileCell<u32>; NUM_GPIOTE], // 0x0
    _reserved0: [u8; 0x100 - (0x0 + NUM_GPIOTE * 4)],
    event_in: [VolatileCell<u32>; NUM_GPIOTE], // 0x100
    _reserved1: [u8; 0x17C - (0x100 + NUM_GPIOTE * 4)],
    port: VolatileCell<u32>, // 0x17C,
    _reserved2: [u8; 0x180],
    inten: VolatileCell<u32>,    // 0x300
    intenset: VolatileCell<u32>, // 0x304
    intenclr: VolatileCell<u32>, // 0x308
    _reserved3: [u8; 0x204],
    config: [VolatileCell<u32>; NUM_GPIOTE], // 0x510
}

const GPIOTE_BASE: usize = 0x40006000;

#[allow(non_snake_case)]
fn GPIO() -> &'static GPIO {
    unsafe { &*(GPIO_BASE as *const GPIO) }
}

// Access to the GPIO Task and Event (GPIOTE) registers, for setting
// up interrupts through the nRF51822 task/event system, in chapter 10
// of the reference manual (v3.0).
#[allow(non_snake_case)]
fn GPIOTE() -> &'static GpioteRegisters {
    unsafe { &*(GPIOTE_BASE as *const GpioteRegisters) }
}

/// Allocate a GPIOTE channel
fn allocate_channel() -> i8 {
    for (i, ch) in GPIOTE().config.iter().enumerate() {
        if ch.get() & 1 == 0 {
            return i as i8;
        }
    }
    return -1;
}

/// Return which channel is allocated to a pin, or -1 if none.
fn find_channel(pin: u8) -> i8 {
    for (i, ch) in GPIOTE().config.iter().enumerate() {
        if (ch.get() >> 8) & 0x1F == pin as u32 {
            return i as i8;
        }
    }
    return -1;
}

pub struct GPIOPin {
    pin: u8,
    client_data: Cell<usize>,
    client: Cell<Option<&'static hil::gpio::Client>>,
}

impl GPIOPin {
    const fn new(pin: u8) -> GPIOPin {
        GPIOPin {
            pin: pin,
            client_data: Cell::new(0),
            client: Cell::new(None),
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }
}

impl hil::gpio::PinCtl for GPIOPin {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        let conf = match mode {
            hil::gpio::InputMode::PullUp => 3,
            hil::gpio::InputMode::PullDown => 1,
            hil::gpio::InputMode::PullNone => 0,
        };
        let pin_cnf = &GPIO().pin_cnf[self.pin as usize];
        pin_cnf.set((pin_cnf.get() & !(0b11 << 2)) | (conf << 2));
    }
}

impl hil::gpio::Pin for GPIOPin {
    fn make_output(&self) {
        GPIO().dirset.set(1 << self.pin);
    }

    // Configuration constants stolen from
    // mynewt/hw/mcu/nordic/nrf51xxx/include/mcu/nrf51_bitfields.h
    fn make_input(&self) {
        GPIO().dirclr.set(1 << self.pin);
    }

    // Not clk
    fn disable(&self) {
        hil::gpio::PinCtl::set_input_mode(self, hil::gpio::InputMode::PullNone);
    }

    fn set(&self) {
        GPIO().outset.set(1 << self.pin);
    }

    fn clear(&self) {
        GPIO().outclr.set(1 << self.pin);
    }

    fn toggle(&self) {
        GPIO().out.set((1 << self.pin) ^ GPIO().out.get());
    }

    fn read(&self) -> bool {
        GPIO().in_.get() & (1 << self.pin) != 0
    }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        self.client_data.set(client_data);
        let mut mode_bits: u32 = 1; // Event
        mode_bits |= match mode {
            hil::gpio::InterruptMode::EitherEdge => 3 << 16,
            hil::gpio::InterruptMode::RisingEdge => 1 << 16,
            hil::gpio::InterruptMode::FallingEdge => 2 << 16,
        };
        let pin = (self.pin & 0b11111) as u32;
        mode_bits |= pin << 8;
        let channel = allocate_channel();
        if channel >= 0 {
            GPIOTE().config[channel as usize].set(mode_bits);
            GPIOTE().intenset.set(1 << channel);
        } else {
            panic!("No available GPIOTE interrupt channels");
        }
    }

    fn disable_interrupt(&self) {
        let channel = find_channel(self.pin);
        if channel >= 0 {
            GPIOTE().config[channel as usize].set(0);
            GPIOTE().intenclr.set(1 << channel);
        }
    }
}

impl GPIOPin {
    pub fn handle_interrupt(&self) {
        self.client.get().map(|client| {
            client.fired(self.client_data.get());
        });
    }
}

pub struct Port {
    pins: [GPIOPin; 32],
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
    /// GPIOTE interrupt: check each of 4 GPIOTE channels, if any has
    /// fired then trigger its corresponding pin's interrupt handler.
    pub fn handle_interrupt(&self) {
        for (i, ev) in GPIOTE().event_in.iter().enumerate() {
            if ev.get() != 0 {
                ev.set(0);
                let pin = (GPIOTE().config[i].get() >> 8 & 0x1F) as usize;
                self.pins[pin].handle_interrupt();
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
