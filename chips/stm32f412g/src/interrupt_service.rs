use stm32f4xx::chip::Stm32f4xxDefaultPeripherals;

use crate::{stm32f412g_nvic, trng_registers};

pub struct Stm32f412gDefaultPeripherals<'a> {
    pub stm32f4: Stm32f4xxDefaultPeripherals<'a>,
    // Once implemented, place Stm32f412g specific peripherals here
    pub trng: stm32f4xx::trng::Trng<'a>,
}

impl<'a> Stm32f412gDefaultPeripherals<'a> {
    pub unsafe fn new(
        rcc: &'a crate::rcc::Rcc,
        exti: &'a crate::exti::Exti<'a>,
        dma1: &'a crate::dma::Dma1<'a>,
        dma2: &'a crate::dma::Dma2<'a>,
    ) -> Self {
        Self {
            stm32f4: Stm32f4xxDefaultPeripherals::new(rcc, exti, dma1, dma2),
            trng: stm32f4xx::trng::Trng::new(trng_registers::RNG_BASE, rcc),
        }
    }
    // Necessary for setting up circular dependencies & registering deferred calls
    pub fn init(&'static self) {
        self.stm32f4.setup_circular_deps();
    }
}
impl<'a> kernel::platform::chip::InterruptService for Stm32f412gDefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            // put Stm32f412g specific interrupts here
            stm32f412g_nvic::RNG => {
                self.trng.handle_interrupt();
                true
            }
            _ => self.stm32f4.service_interrupt(interrupt),
        }
    }
}
