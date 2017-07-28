use driver::Driver;

pub mod mpu;
pub mod systick;

/// Interface for individual boards.
pub trait Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R where F: FnOnce(Option<&Driver>) -> R;
    fn kernel_start_address(&self) -> u32;
    fn kernel_end_address(&self) -> u32;
    fn apps_start_address(&self) -> u32;
    fn apps_end_address(&self) -> u32;
}

/// Interface for individual MCUs.
pub trait Chip {
    type MPU: mpu::MPU;
    type SysTick: systick::SysTick;

    fn service_pending_interrupts(&mut self);
    fn has_pending_interrupts(&self) -> bool;
    fn mpu(&self) -> &Self::MPU;
    fn systick(&self) -> &Self::SysTick;
    fn prepare_for_sleep(&self) {}
}
