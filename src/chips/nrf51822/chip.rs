use main;

pub struct NRF51822(());

impl NRF51822 {
    pub unsafe fn new() -> NRF51822 {
        NRF51822(())
    }
}

impl main::Chip for NRF51822 {
    type MPU = ();
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &self.0
    }

    fn systick(&self) -> &Self::SysTick {
        &self.0
    }

    fn service_pending_interrupts(&mut self) {}

    fn has_pending_interrupts(&self) -> bool { false }
}

