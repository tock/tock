use crate::chip_specs::Stm32u545Specs;
use kernel::platform::chip::InterruptService;
use stm32u5xx::chip::Stm32u5xxDefaultPeripherals;
pub struct Stm32u545DefaultPeripherals<'a> {
    pub stm32u545: Stm32u5xxDefaultPeripherals<'a, Stm32u545Specs>,
}

impl<'a> Stm32u545DefaultPeripherals<'a> {
    pub unsafe fn new(clocks: &'a crate::clocks::Clocks<'a, Stm32u545Specs>) -> Self {
        Self {
            stm32u545: Stm32u5xxDefaultPeripherals::new(clocks),
        }
    }

    pub fn init(&'static self) {
        self.stm32u545.setup_circular_deps()
    }
}

impl InterruptService for Stm32u545DefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        self.stm32u545.service_interrupt(interrupt)
    }
}
