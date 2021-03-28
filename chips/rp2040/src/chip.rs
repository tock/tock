//! Chip trait setup.

use core::fmt::Write;
use kernel::common::deferred_call;
use kernel::Chip;
use kernel::InterruptService;

use crate::clocks::Clocks;
use crate::deferred_call_tasks::DeferredCallTask;
use crate::gpio::SIO;
use crate::interrupts;
use crate::resets::Resets;
use crate::xosc::Xosc;

#[repr(u8)]
pub enum Processor {
    Processor0 = 0,
    Processor1 = 1,
}

pub struct Rp2040<'a, I: InterruptService<DeferredCallTask> + 'a> {
    mpu: cortexm0p::mpu::MPU,
    userspace_kernel_boundary: cortexm0p::syscall::SysCall,
    scheduler_timer: cortexm0p::systick::SysTick,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService<DeferredCallTask>> Rp2040<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm0p::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm0p::syscall::SysCall::new(),
            scheduler_timer: cortexm0p::systick::SysTick::new(),
            interrupt_service,
        }
    }
}

impl<'a, I: InterruptService<DeferredCallTask>> Chip for Rp2040<'a, I> {
    type MPU = cortexm0p::mpu::MPU;
    type UserspaceKernelBoundary = cortexm0p::syscall::SysCall;
    type SchedulerTimer = cortexm0p::systick::SysTick;
    type WatchDog = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    if !self.interrupt_service.service_deferred_call(task) {
                        panic!("unhandled deferred call");
                    }
                } else if let Some(interrupt) = cortexm0p::nvic::next_pending() {
                    // ignore SIO_IRQ_PROC1 as it is intended for processor 1
                    // not able to unset its pending status
                    // probably only processor 1 can unset the pending by reading the fifo
                    if interrupt == interrupts::SIO_IRQ_PROC1 {
                        break;
                    }
                    if !self.interrupt_service.service_interrupt(interrupt) {
                        panic!("unhandled interrupt {}", interrupt);
                    }
                    let n = cortexm0p::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        // ignore SIO_IRQ_PROC1 as it is intended for processor 1
        // not able to unset its pending status
        // probably only processor 1 can unset the pending by reading the fifo
        unsafe {
            (cortexm0p::nvic::has_pending()
                && cortexm0p::nvic::next_pending() != Some(interrupts::SIO_IRQ_PROC1))
                || deferred_call::has_tasks()
        }
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm0p::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm0p::support::atomic(f)
    }

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        cortexm0p::print_cortexm0_state(writer);
    }
}

pub struct Rp2040DefaultPeripherals {
    pub resets: Resets,
    pub sio: SIO,
    pub clocks: Clocks,
    pub xosc: Xosc,
}

impl Rp2040DefaultPeripherals {
    pub const fn new() -> Self {
        Self {
            resets: Resets::new(),
            sio: SIO::new(),
            clocks: Clocks::new(),
            xosc: Xosc::new(),
        }
    }
}

impl InterruptService<DeferredCallTask> for Rp2040DefaultPeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::SIO_IRQ_PROC0 => {
                self.sio.handle_proc_interrupt(Processor::Processor0);
                true
            }
            interrupts::SIO_IRQ_PROC1 => {
                self.sio.handle_proc_interrupt(Processor::Processor1);
                true
            }
            _ => false,
        }
        // true
    }

    unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
        match task {
            // DeferredCallTask::Flash => self.flash.handle_interrupt(),
            _ => false,
        }
        // true
    }
}
