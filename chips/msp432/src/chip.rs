use core::fmt::Write;
use cortexm4;
use kernel::Chip;

use crate::gpio;
use crate::nvic;
use crate::uart;
use crate::wdt;

pub struct Msp432 {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    scheduler_timer: cortexm4::systick::SysTick,
}

impl Msp432 {
    pub unsafe fn new() -> Msp432 {
        Msp432 {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            scheduler_timer: cortexm4::systick::SysTick::new_with_calibration(48_000_000),
        }
    }
}

impl Chip for Msp432 {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = wdt::Wdt;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    match interrupt {
                        nvic::USCI_A0 => uart::UART0.handle_interrupt(),
                        nvic::IO_PORT1 => gpio::handle_interrupt(0),
                        nvic::IO_PORT2 => gpio::handle_interrupt(1),
                        nvic::IO_PORT3 => gpio::handle_interrupt(2),
                        nvic::IO_PORT4 => gpio::handle_interrupt(3),
                        nvic::IO_PORT5 => gpio::handle_interrupt(4),
                        nvic::IO_PORT6 => gpio::handle_interrupt(5),
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

    fn scheduler_timer(&self) -> &cortexm4::systick::SysTick {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        unsafe { &wdt::WDT }
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm4::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        cortexm4::print_cortexm4_state(write);
    }
}
