use cortexm4::nvic;
use kernel;

pub struct cc2650(());

impl cc2650 {
    pub unsafe fn new() -> cc2650 { cc2650(()) }
}

impl kernel::Chip for cc2650 {
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

