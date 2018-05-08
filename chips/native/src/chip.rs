//! State holding of the native "chip"

use kernel::Chip;

pub struct NativeChip( () );

impl NativeChip {
    pub fn new() -> NativeChip {
        NativeChip( () )
    }
}

impl Chip for NativeChip {
    type MPU = ();
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &self.0
    }

    fn systick(&self) -> &Self::SysTick {
        &self.0
    }

    fn service_pending_interrupts(&mut self) {
        unimplemented!("service_pending_interrupts");
    }

    fn has_pending_interrupts(&self) -> bool {
        unimplemented!("has_pending_interrupts");
    }

    fn sleep(&self) {
        unimplemented!("sleep");
    }

    unsafe fn atomic<F, R>(&self, _f: F) -> R
    where
        F: FnOnce() -> R,
    {
        unimplemented!("atomic operation");
    }
}
