//! Chip trait setup.

use core::fmt::Write;
use cortexm4;
use kernel::Chip;
use kernel::InterruptService;

pub struct Apollo3<I: InterruptService<()> + 'static> {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    scheduler_timer: cortexm4::systick::SysTick,
    interrupt_service: &'static I,
}

impl<I: InterruptService<()> + 'static> Apollo3<I> {
    pub unsafe fn new(interrupt_service: &'static I) -> Self {
        Self {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            scheduler_timer: cortexm4::systick::SysTick::new_with_calibration(48_000_000),
            interrupt_service,
        }
    }
}

/// This struct, when initialized, instantiates all peripheral drivers for the apollo3.
/// If a board wishes to use only a subset of these peripherals, this
/// should not be used or imported, and a modified version should be
/// constructed manually in main.rs.
pub struct Apollo3DefaultPeripherals {
    pub stimer: crate::stimer::STimer<'static>,
    pub uart0: crate::uart::Uart<'static>,
    pub uart1: crate::uart::Uart<'static>,
    pub gpio_port: crate::gpio::Port<'static>,
    pub iom0: crate::iom::Iom<'static>,
    pub iom1: crate::iom::Iom<'static>,
    pub iom2: crate::iom::Iom<'static>,
    pub iom3: crate::iom::Iom<'static>,
    pub iom4: crate::iom::Iom<'static>,
    pub iom5: crate::iom::Iom<'static>,
    pub ble: crate::ble::Ble<'static>,
}

impl Apollo3DefaultPeripherals {
    pub fn new() -> Self {
        Self {
            stimer: crate::stimer::STimer::new(),
            uart0: crate::uart::Uart::new_uart_0(),
            uart1: crate::uart::Uart::new_uart_1(),
            gpio_port: crate::gpio::Port::new(),
            iom0: crate::iom::Iom::new0(),
            iom1: crate::iom::Iom::new1(),
            iom2: crate::iom::Iom::new2(),
            iom3: crate::iom::Iom::new3(),
            iom4: crate::iom::Iom::new4(),
            iom5: crate::iom::Iom::new5(),
            ble: crate::ble::Ble::new(),
        }
    }
}

impl kernel::InterruptService<()> for Apollo3DefaultPeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        use crate::nvic;
        match interrupt {
            nvic::STIMER..=nvic::STIMER_CMPR7 => self.stimer.handle_interrupt(),
            nvic::UART0 => self.uart0.handle_interrupt(),
            nvic::UART1 => self.uart1.handle_interrupt(),
            nvic::GPIO => self.gpio_port.handle_interrupt(),
            nvic::IOMSTR0 => self.iom0.handle_interrupt(),
            nvic::IOMSTR1 => self.iom1.handle_interrupt(),
            nvic::IOMSTR2 => self.iom2.handle_interrupt(),
            nvic::IOMSTR3 => self.iom3.handle_interrupt(),
            nvic::IOMSTR4 => self.iom4.handle_interrupt(),
            nvic::IOMSTR5 => self.iom5.handle_interrupt(),
            nvic::BLE => self.ble.handle_interrupt(),
            _ => return false,
        }
        true
    }
    unsafe fn service_deferred_call(&self, _: ()) -> bool {
        false
    }
}

impl<I: InterruptService<()> + 'static> Chip for Apollo3<I> {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    if !self.interrupt_service.service_interrupt(interrupt) {
                        panic!("unhandled interrupt, {}", interrupt);
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
