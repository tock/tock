//! Temperature Sensor Driver for nrf51dk
//!
//! Generates a simple temperature measurement without sampling
//!
//! Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! Author: Fredrik Nilsson <frednils@student.chalmers.se>
//! Date: March 03, 2017

use chip;
use core::cell::Cell;
use kernel;
use nvic;
use peripheral_interrupts;
use peripheral_registers;

const NRF_TEMP_DATARDY_INTR: u32 = 1;
const NRF_TEMP_ENABLE: u32 = 1;
const NRF_TEMP_DISABLE: u32 = 0;


#[deny(no_mangle_const_items)]
#[no_mangle]
pub struct Temperature {
    regs: *const peripheral_registers::TEMP_REGS,
    client: Cell<Option<&'static kernel::hil::sensor::TemperatureClient>>,
}

pub static mut TEMP: Temperature = Temperature::new();

impl Temperature {
    const fn new() -> Temperature {
        Temperature {
            regs: peripheral_registers::TEMP_BASE as *const peripheral_registers::TEMP_REGS,
            client: Cell::new(None),
        }
    }

    pub fn handle_interrupt(&self) {
        // disable interrupts
        self.disable_nvic();
        self.disable_interrupts();
        let regs = unsafe { &*self.regs };

        // get temperature
        let temp = regs.TEMP.get() / 4;

        // stop measurement
        regs.STOP.set(NRF_TEMP_DISABLE);

        // trigger callback with temperature
        self.client
            .get()
            .map(|client| client.callback(temp as usize, 0, kernel::ReturnCode::SUCCESS));
        nvic::clear_pending(peripheral_interrupts::NvicIdx::TEMP);
    }

    fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.INTENSET.set(NRF_TEMP_DATARDY_INTR);
    }

    fn disable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.INTENCLR.set(NRF_TEMP_DATARDY_INTR);
    }

    fn enable_nvic(&self) {
        nvic::enable(peripheral_interrupts::NvicIdx::TEMP);
    }

    fn disable_nvic(&self) {
        nvic::disable(peripheral_interrupts::NvicIdx::TEMP);
    }
}

impl kernel::hil::sensor::TemperatureDriver for Temperature {
    fn read_cpu_temperature(&self) -> kernel::ReturnCode {
        let regs = unsafe { &*self.regs };
        self.enable_nvic();
        self.enable_interrupts();
        regs.DATARDY.set(NRF_TEMP_DISABLE);
        regs.START.set(NRF_TEMP_ENABLE);
        kernel::ReturnCode::SUCCESS
    }

    fn set_client(&self, client: &'static kernel::hil::sensor::TemperatureClient) {
        self.client.set(Some(client));
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn TEMP_Handler() {
    use kernel::common::Queue;
    nvic::disable(peripheral_interrupts::NvicIdx::TEMP);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(peripheral_interrupts::NvicIdx::TEMP);
}
