use cortexm4::nvic;
use kernel;

pub struct Cc2650(());

impl Cc2650 {
    pub unsafe fn new() -> Cc2650 { Cc2650(()) }
}

impl kernel::Chip for Cc2650 {
    type MPU = ();
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &self.0
    }

    fn systick(&self) -> &Self::SysTick {
        &self.0
    }

    fn service_pending_interrupts(&mut self) {
        unsafe {
            while let Some(interrupt) = nvic::next_pending() {
                let n = nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe {
            nvic::has_pending()
        }
    }
}

