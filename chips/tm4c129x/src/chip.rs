//! Interrupt mapping

use cortexm4;
use gpt;
use kernel::Chip;
use uart;

pub struct Tm4c129x {
    pub mpu: cortexm4::mpu::MPU,
    pub systick: cortexm4::systick::SysTick,
}

impl Tm4c129x {
    pub unsafe fn new() -> Tm4c129x {
        Tm4c129x {
            mpu: cortexm4::mpu::MPU::new(),
            systick: cortexm4::systick::SysTick::new(),
        }
    }
}

impl Chip for Tm4c129x {
    type MPU = cortexm4::mpu::MPU;
    type SysTick = cortexm4::systick::SysTick;

    fn service_pending_interrupts(&self) {
        use nvic;

        unsafe {
            loop {
                if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    match interrupt {
                        nvic::UART0 => uart::UART0.handle_interrupt(),
                        nvic::TIMER0A => gpt::TIMER0.handle_interrupt(),
                        _ => {
                            panic!("unhandled interrupt {}", interrupt);
                        }
                    }
                    let n = cortexm4::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm4::nvic::has_pending() }
    }

    fn mpu(&self) -> &cortexm4::mpu::MPU {
        &self.mpu
    }

    fn systick(&self) -> &cortexm4::systick::SysTick {
        &self.systick
    }

    fn sleep(&self) {
        /*if pm::deep_sleep_ready() {
            unsafe {
                cortexm4::scb::set_sleepdeep();
            }
        } else {
            unsafe {
                cortexm4::scb::unset_sleepdeep();
            }
        }*/
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }
}
