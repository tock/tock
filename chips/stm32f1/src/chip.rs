use cortexm3;
use kernel;
use stm32;
use stm32::nvic::NvicIdx;

pub struct STM32F1 {
    mpu: (),
    systick: &'static cortexm3::systick::SysTick,
}

impl STM32F1 {
    pub unsafe fn new() -> STM32F1 {
        stm32::chip::init();
        STM32F1 {
            mpu: (),
            systick: cortexm3::systick::SysTick::new(),
        }
    }
}

impl kernel::Chip for STM32F1 {
    type MPU = ();
    type SysTick = cortexm3::systick::SysTick;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        self.systick
    }

    fn service_pending_interrupts(&mut self) {
        unsafe {
            stm32::chip::dequeue_interrupt().map(|interrupt| {
                match interrupt {
                    NvicIdx::TIM2 => stm32::timer::TIMER2.handle_interrupt(),
                    NvicIdx::USART1 => stm32::usart::USART1.handle_interrupt(),
                    _ => {}
                }
                stm32::nvic::enable(interrupt);
            });
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { stm32::chip::has_pending_interrupts() }
    }
}
