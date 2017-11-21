//! Temperature Sensor Driver for nrf51dk
//!
//! Generates a simple temperature measurement without sampling
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: March 03, 2017

use chip;
use core::cell::Cell;
use kernel::hil::temperature::{TemperatureDriver, Client};
use peripheral_registers::{TEMP_REGS, TEMP_BASE};

/// Syscall Number
pub const DRIVER_NUM: usize = 0x80_06_00_01;

#[deny(no_mangle_const_items)]
#[no_mangle]
pub struct Temperature {
    regs: *const TEMP_REGS,
    client: Cell<Option<&'static Client>>,
}

pub static mut TEMP: Temperature = Temperature::new();

impl Temperature {
    const fn new() -> Temperature {
        Temperature {
            regs: TEMP_BASE as *mut TEMP_REGS,
            client: Cell::new(None),
        }
    }

    fn measure(&self) {
        let regs = unsafe { &*self.regs };

        self.enable_interrupts();

        regs.DATARDY.set(0);
        regs.START.set(1);
    }

    // MEASUREMENT DONE
    pub fn handle_interrupt(&self) {
        // ONLY DATARDY CAN TRIGGER THIS INTERRUPT
        let regs = unsafe { &*self.regs };

        // get temperature
        let temp = regs.TEMP.get() / 4;

        // stop measurement
        regs.STOP.set(1);

        // disable interrupts
        self.disable_interrupts();

        // trigger callback with temperature
        self.client.get().map(|client| client.measurement_done(temp as usize));
    }

    fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        // enable interrupts on DATARDY events
        regs.INTEN.set(1);
        regs.INTENSET.set(1);
    }

    fn disable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        // disable interrupts on DATARDY events
        regs.INTENCLR.set(1);
    }

    pub fn set_client<C: Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }
}
// Methods of RadioDummy Trait/Interface and are shared between Capsules and Chips
impl TemperatureDriver for Temperature {
    fn take_measurement(&self) {
        self.measure()
    }
}
