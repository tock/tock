// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors
//
// Minimal STM32U5xx chip trait setup for Tock.

use core::fmt::Write;

use crate::chip_specifics::chip_specs::ChipSpecs as ChipSpecsTrait;
use crate::nvic;
use cortexm33::{CortexM33, CortexMVariant};
use kernel::platform::chip::{Chip, InterruptService};

// Pull in whatever you actually implement for these:
use crate::clocks;
//use crate::flash;
use crate::gpio;

/// Top-level chip object used by the kernel.
///
/// This is the Cortex-M33 + STM32U5 integration. It owns:
/// - the MPU instance
/// - the userspace/kernel boundary abstraction
/// - a reference to the interrupt service object
///
pub struct Stm32u5xx<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<16>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

/// “Default peripherals” bundle for STM32U5.
///
/// For a minimal bring-up we only keep:
/// - clocks
/// - flash (for latency config)
/// - GPIO ports
///
/// Add UART, timers, DMA, etc. later as you implement those drivers.
pub struct Stm32u5xxDefaultPeripherals<'a, ChipSpecs> {
    pub clocks: &'a clocks::Clocks<'a, ChipSpecs>,
    //pub flash: flash::Flash<ChipSpecs>,
    pub gpio_ports: gpio::GpioPorts<'a>,
    pub usart1: crate::usart::Usart<'a>,
    pub tim2: crate::tim::Tim2<'a>,
}

impl<'a, ChipSpecs: ChipSpecsTrait> Stm32u5xxDefaultPeripherals<'a, ChipSpecs> {
    pub fn new(clocks: &'a clocks::Clocks<'a, ChipSpecs>) -> Self {
        Self {
            clocks,
            //flash: flash::Flash::new(),
            gpio_ports: gpio::GpioPorts::new(clocks),
            usart1: crate::usart::Usart::new_usart1(clocks),
            tim2: crate::tim::Tim2::new(clocks),
        }
    }

    /// Setup circular dependencies and register deferred calls.
    ///
    /// Extend this as you add more peripherals (UART, timers, etc.).
    pub fn setup_circular_deps(&'static self) {
        // Allow clock driver to adjust flash latency when switching SYSCLK.
        //self.clocks.set_flash(&self.flash);

        // GPIO may need clocks / EXTI references wired inside.
        self.gpio_ports.setup_circular_deps();

        // Example: once you have USART/TIM/etc, register them here:
        kernel::deferred_call::DeferredCallClient::register(&self.usart1);
    }
}

/// Interrupt dispatch for STM32U5.
///
/// For now this is a stub: no interrupts are wired.
/// Once you implement NVIC numbers and drivers, handle them here.
impl<ChipSpecs: ChipSpecsTrait> InterruptService for Stm32u5xxDefaultPeripherals<'_, ChipSpecs> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            // TODO: map STM32U5 NVIC numbers to driver handlers, e.g.:
            // nvic::EXTI0 => self.exti.handle_interrupt(),
            nvic::USART1 => self.usart1.handle_interrupt(),
            nvic::TIM2 => self.tim2.handle_interrupt(),
            _ => return false,
        }
        true
    }
}

impl<'a, I: InterruptService + 'a> Stm32u5xx<'a, I> {
    /// Create the chip object.
    ///
    /// Call this *after* early clock / power init (HSI16 as SYSCLK, etc.).
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm33::mpu::new(),
            userspace_kernel_boundary: cortexm33::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<'a, I: InterruptService + 'a> Chip for Stm32u5xx<'a, I> {
    type MPU = cortexm33::mpu::MPU<16>;
    type UserspaceKernelBoundary = cortexm33::syscall::SysCall;
    type ThreadIdProvider = cortexm33::thread_id::CortexMThreadIdProvider;

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = cortexm33::nvic::next_pending() {
                if !self.interrupt_service.service_interrupt(interrupt) {
                    panic!("unhandled interrupt {}", interrupt);
                }

                let n = cortexm33::nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm33::nvic::has_pending() }
    }

    fn userspace_kernel_boundary(&self) -> &cortexm33::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm33::scb::unset_sleepdeep();
            cortexm33::support::wfi();
        }
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm33::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, write: &mut dyn Write) {
        CortexM33::print_cortexm_state(write);
    }
}
