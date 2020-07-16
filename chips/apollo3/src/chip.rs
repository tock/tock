//! Chip trait setup.

use core::fmt::Write;
use cortexm4;
use kernel::Chip;

use crate::ble;
use crate::gpio;
use crate::iom;
use crate::nvic;
use crate::stimer;
use crate::uart;

pub struct Apollo3 {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    scheduler_timer: cortexm4::systick::SysTick,
}

impl Apollo3 {
    pub unsafe fn new() -> Apollo3 {
        Apollo3 {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            scheduler_timer: cortexm4::systick::SysTick::new_with_calibration(48_000_000),
        }
    }
}

impl Chip for Apollo3 {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    match interrupt {
                        nvic::STIMER..=nvic::STIMER_CMPR7 => stimer::STIMER.handle_interrupt(),
                        nvic::UART0 => uart::UART0.handle_interrupt(),
                        nvic::UART1 => uart::UART1.handle_interrupt(),
                        nvic::GPIO => gpio::PORT.handle_interrupt(),
                        nvic::IOMSTR0 => iom::IOM0.handle_interrupt(),
                        nvic::IOMSTR1 => iom::IOM1.handle_interrupt(),
                        nvic::IOMSTR2 => iom::IOM2.handle_interrupt(),
                        nvic::IOMSTR3 => iom::IOM3.handle_interrupt(),
                        nvic::IOMSTR4 => iom::IOM4.handle_interrupt(),
                        nvic::IOMSTR5 => iom::IOM5.handle_interrupt(),
                        nvic::BLE => ble::BLE.handle_interrupt(),
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
        &()
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm4::scb::unset_sleepdeep();
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
