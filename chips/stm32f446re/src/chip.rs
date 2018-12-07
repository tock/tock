//! Chip trait setup.

use cortexm4;
use kernel::Chip;

// There is a MPU, SysCall and SysTick impl for `()`. Use that till we are ready
// to add `cortexm4::mpu::MPU`, `cortexm4::syscall::SysCall` and
// `cortexm4::systick::SysTick`
pub struct Stm32f446re {
    mpu: (),
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: (),
}

impl Stm32f446re {
    pub unsafe fn new() -> Stm32f446re {
        Stm32f446re {
            mpu: (),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            systick: (),
        }
    }
}

impl Chip for Stm32f446re {
    type MPU = ();
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SysTick = ();

    fn service_pending_interrupts(&self) {
        // TODO
    }

    fn has_pending_interrupts(&self) -> bool {
        // TODO
        false
    }

    fn mpu(&self) -> &() {
        &self.mpu
    }

    fn systick(&self) -> &() {
        &self.systick
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        // TODO
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }
}
