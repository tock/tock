use crate::deferred_call_tasks::DeferredCallTask;
use nrf52::chip::Nrf52DefaultPeripherals;

/// This struct, when initialized, instantiates all peripheral drivers for the nrf52840.
/// If a board wishes to use only a subset of these peripherals, this
/// should not be used or imported, and a modified version should be
/// constructed manually in main.rs.
//create all base nrf52 peripherals
pub struct Nrf52840DefaultPeripherals<'a> {
    pub nrf52: Nrf52DefaultPeripherals<'a>,
    pub usbd: crate::usbd::Usbd<'a>,
    pub gpio_port: crate::gpio::Port<'a, { crate::gpio::NUM_PINS }>,
}

impl<'a> Nrf52840DefaultPeripherals<'a> {
    pub unsafe fn new() -> Self {
        Self {
            nrf52: Nrf52DefaultPeripherals::new(),
            usbd: crate::usbd::Usbd::new(),
            gpio_port: crate::gpio::nrf52840_gpio_create(),
        }
    }
    // Necessary for setting up circular dependencies
    pub fn init(&'a self) {
        self.nrf52.pwr_clk.set_usb_client(&self.usbd);
        self.usbd.set_power_ref(&self.nrf52.pwr_clk);
        self.nrf52.init();
    }
}
impl<'a> kernel::platform::chip::InterruptService<DeferredCallTask>
    for Nrf52840DefaultPeripherals<'a>
{
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            crate::peripheral_interrupts::USBD => self.usbd.handle_interrupt(),
            nrf52::peripheral_interrupts::GPIOTE => self.gpio_port.handle_interrupt(),
            _ => return self.nrf52.service_interrupt(interrupt),
        }
        true
    }
    unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
        self.nrf52.service_deferred_call(task)
    }
}
