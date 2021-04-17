//! Chip trait setup.

use core::fmt::Write;
use cortexm7;
use kernel::debug;
use kernel::{Chip, InterruptService};

use crate::nvic;

pub struct Imxrt10xx<I: InterruptService<()> + 'static> {
    mpu: cortexm7::mpu::MPU,
    userspace_kernel_boundary: cortexm7::syscall::SysCall,
    scheduler_timer: cortexm7::systick::SysTick,
    interrupt_service: &'static I,
}

impl<I: InterruptService<()> + 'static> Imxrt10xx<I> {
    pub unsafe fn new(interrupt_service: &'static I) -> Self {
        Imxrt10xx {
            mpu: cortexm7::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm7::syscall::SysCall::new(),
            scheduler_timer: cortexm7::systick::SysTick::new_with_calibration(792_000_000),
            interrupt_service,
        }
    }
}

pub struct Imxrt10xxDefaultPeripherals {
    pub iomuxc: crate::iomuxc::Iomuxc,
    pub iomuxc_snvs: crate::iomuxc_snvs::IomuxcSnvs,
    pub ccm: &'static crate::ccm::Ccm,
    pub dcdc: crate::dcdc::Dcdc<'static>,
    pub ccm_analog: crate::ccm_analog::CcmAnalog,
    pub ports: crate::gpio::Ports<'static>,
    pub lpi2c1: crate::lpi2c::Lpi2c<'static>,
    pub lpuart1: crate::lpuart::Lpuart<'static>,
    pub lpuart2: crate::lpuart::Lpuart<'static>,
    pub gpt1: crate::gpt::Gpt1<'static>,
    pub gpt2: crate::gpt::Gpt2<'static>,
}

impl Imxrt10xxDefaultPeripherals {
    pub const fn new(ccm: &'static crate::ccm::Ccm) -> Self {
        Self {
            iomuxc: crate::iomuxc::Iomuxc::new(),
            iomuxc_snvs: crate::iomuxc_snvs::IomuxcSnvs::new(),
            ccm,
            dcdc: crate::dcdc::Dcdc::new(ccm),
            ccm_analog: crate::ccm_analog::CcmAnalog::new(),
            ports: crate::gpio::Ports::new(ccm),
            lpi2c1: crate::lpi2c::Lpi2c::new_lpi2c1(ccm),
            lpuart1: crate::lpuart::Lpuart::new_lpuart1(ccm),
            lpuart2: crate::lpuart::Lpuart::new_lpuart2(ccm),
            gpt1: crate::gpt::Gpt1::new_gpt1(ccm),
            gpt2: crate::gpt::Gpt2::new_gpt2(ccm),
        }
    }
}

impl InterruptService<()> for Imxrt10xxDefaultPeripherals {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nvic::LPUART1 => self.lpuart1.handle_interrupt(),
            nvic::LPUART2 => self.lpuart2.handle_interrupt(),
            nvic::LPI2C1 => self.lpi2c1.handle_event(),
            nvic::GPT1 => self.gpt1.handle_interrupt(),
            nvic::GPT2 => self.gpt2.handle_interrupt(),
            nvic::GPIO1_1 => self.ports.gpio1.handle_interrupt(),
            nvic::GPIO1_2 => self.ports.gpio1.handle_interrupt(),
            nvic::GPIO2_1 => self.ports.gpio2.handle_interrupt(),
            nvic::GPIO2_2 => self.ports.gpio2.handle_interrupt(),
            nvic::GPIO3_1 => self.ports.gpio3.handle_interrupt(),
            nvic::GPIO3_2 => self.ports.gpio3.handle_interrupt(),
            nvic::GPIO4_1 => self.ports.gpio4.handle_interrupt(),
            nvic::GPIO4_2 => self.ports.gpio4.handle_interrupt(),
            nvic::GPIO5_1 => self.ports.gpio5.handle_interrupt(),
            nvic::GPIO5_2 => self.ports.gpio5.handle_interrupt(),
            nvic::SNVS_LP_WRAPPER => debug!("Interrupt: SNVS_LP_WRAPPER"),
            _ => {
                return false;
            }
        }
        true
    }

    unsafe fn service_deferred_call(&self, _: ()) -> bool {
        false
    }
}

impl<I: InterruptService<()> + 'static> Chip for Imxrt10xx<I> {
    type MPU = cortexm7::mpu::MPU;
    type UserspaceKernelBoundary = cortexm7::syscall::SysCall;
    type SchedulerTimer = cortexm7::systick::SysTick;
    type WatchDog = ();
    type Core = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(interrupt) = cortexm7::nvic::next_pending() {
                    let handled = self.interrupt_service.service_interrupt(interrupt);
                    assert!(handled, "Unhandled interrupt number {}", interrupt);
                    let n = cortexm7::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm7::nvic::has_pending() }
    }

    fn mpu(&self) -> &cortexm7::mpu::MPU {
        &self.mpu
    }

    fn scheduler_timer(&self) -> &cortexm7::systick::SysTick {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &cortexm7::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm7::scb::unset_sleepdeep();
            cortexm7::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm7::support::atomic(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        cortexm7::print_cortexm7_state(write);
    }

    fn current_core(&self) -> &Self::Core {
        &()
    }
}
