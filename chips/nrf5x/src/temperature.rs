//! Temperature sensor driver, nRF5X-family
//!
//! Generates a simple temperature measurement without sampling
//!
//! Authors
//! -------------------
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: March 03, 2017

use core::cell::Cell;
use kernel;
use nvic;
use peripheral_interrupts;
use peripheral_registers;

/// Syscall Number
pub const DRIVER_NUM: usize = 0x80_06_00_01;

const NRF_TEMP_DATARDY_INTR: u32 = 1;
const NRF_TEMP_ENABLE: u32 = 1;
const NRF_TEMP_DISABLE: u32 = 0;


#[deny(no_mangle_const_items)]
#[no_mangle]
pub struct Temperature {
    regs: *const peripheral_registers::TEMP_REGS,
    client: Cell<Option<&'static kernel::hil::sensors::TemperatureClient>>,
}

pub static mut TEMP: Temperature = Temperature::new();

impl Temperature {
    const fn new() -> Temperature {
        Temperature {
            regs: peripheral_registers::TEMP_BASE as *const peripheral_registers::TEMP_REGS,
            client: Cell::new(None),
        }
    }

    // MEASUREMENT DONE
    pub fn handle_interrupt(&self) {
        // disable interrupts
        self.disable_nvic();
        self.disable_interrupts();
        let regs = unsafe { &*self.regs };

        // get temperature
        // Result of temperature measurement in °C, 2's complement format, 0.25 °C
        let temp = (regs.temp.get() / 4) * 100;

        // stop measurement
        regs.task_stop.set(NRF_TEMP_DISABLE);

        // disable interrupts
        self.disable_nvic();
        self.disable_interrupts();

        // trigger callback with temperature
        self.client
            .get()
            .map(|client| client.callback(temp as usize));
        nvic::clear_pending(peripheral_interrupts::NvicIdx::TEMP);
    }

    fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(NRF_TEMP_DATARDY_INTR);
    }

    fn disable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(NRF_TEMP_DATARDY_INTR);
    }

    fn enable_nvic(&self) {
        nvic::enable(peripheral_interrupts::NvicIdx::TEMP);
    }

    fn disable_nvic(&self) {
        nvic::disable(peripheral_interrupts::NvicIdx::TEMP);
    }
}

impl kernel::hil::sensors::TemperatureDriver for Temperature {
    fn read_temperature(&self) -> kernel::ReturnCode {
        let regs = unsafe { &*self.regs };
        self.enable_nvic();
        self.enable_interrupts();
        regs.event_datardy.set(NRF_TEMP_DISABLE);
        regs.task_start.set(NRF_TEMP_ENABLE);
        kernel::ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'static kernel::hil::sensors::TemperatureClient) {
        self.client.set(Some(client));
    }
}
