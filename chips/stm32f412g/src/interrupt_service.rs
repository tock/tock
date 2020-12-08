use crate::stm32f412g_nvic;
use stm32f4xx::chip::Stm32f4xxDefaultPeripherals;
use stm32f4xx::deferred_calls::DeferredCallTask;

pub struct Stm32f412gDefaultPeripherals<'a> {
    pub stm32f4: Stm32f4xxDefaultPeripherals<'a>,
    // Once implemented, place Stm32f412g specific peripherals here
    pub trng: stm32f4xx::trng::Trng<'a>,
}

impl<'a> Stm32f412gDefaultPeripherals<'a> {
    pub unsafe fn new(
        rcc: &'a crate::rcc::Rcc,
        exti: &'a crate::exti::Exti<'a>,
        dma: &'a crate::dma1::Dma1<'a>,
    ) -> Self {
        Self {
            stm32f4: Stm32f4xxDefaultPeripherals::new(rcc, exti, dma),
            trng: stm32f4xx::trng::Trng::new(rcc),
        }
    }
    // Necessary for setting up circular dependencies
    pub fn init(&'a self) {
        self.stm32f4.setup_circular_deps();
    }
}
impl<'a> kernel::InterruptService<DeferredCallTask> for Stm32f412gDefaultPeripherals<'a> {
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
    unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
        self.stm32f4.service_deferred_call(task)
    }
}
