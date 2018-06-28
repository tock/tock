//! State holding of the native "chip"

use std::collections::VecDeque;

use kernel::Chip;

pub struct NativeChip<'a> {
    interrupt_queue: VecDeque<&'a Fn()>,
}

impl NativeChip<'a> {
    pub fn new() -> NativeChip<'a> {
        NativeChip {
            interrupt_queue: VecDeque::new(),
        }
    }
}

impl Chip for NativeChip<'a> {
    type MPU = ();
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &()
    }

    fn systick(&self) -> &Self::SysTick {
        &()
    }

    fn service_pending_interrupts(&mut self) {
        while self.has_pending_interrupts() {
            if let Some(next) = self.interrupt_queue.pop_front() {
                next();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        !self.interrupt_queue.is_empty()
    }

    fn sleep(&self) {
        unimplemented!("sleep");
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // TODO: Think about whether there's a situation where native isn't atomic
        f()
    }
}
