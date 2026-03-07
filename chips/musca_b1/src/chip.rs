// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Chip trait setup.

use core::fmt::Write;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::interrupts;
use crate::timer::GPTimer;
use crate::uart::Uart;
use cortexm33::{CortexM33, CortexMVariant};

#[repr(u8)]
pub enum Processor {
    Processor0 = 0,
    Processor1 = 1,
}

pub struct MuscaB1<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService> MuscaB1<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm33::mpu::new(),
            userspace_kernel_boundary: cortexm33::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<I: InterruptService> Chip for MuscaB1<'_, I> {
    type MPU = cortexm33::mpu::MPU<8>;
    type UserspaceKernelBoundary = cortexm33::syscall::SysCall;
    type ThreadIdProvider = cortexm33::thread_id::CortexMThreadIdProvider;

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = cortexm33::nvic::next_pending() {
                let handled = self.interrupt_service.service_interrupt(interrupt);
                assert!(handled, "Unhandled interrupt number {}", interrupt);
                let n = cortexm33::nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm33::nvic::has_pending() }
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm33::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm33::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, writer: &mut dyn Write) {
        CortexM33::print_cortexm_state(writer);
    }
}

pub struct MuscaB1DefaultPeripherals<'a> {
    pub gp_timer: GPTimer<'a>,
    pub uart0: Uart<'a>,
    pub uart1: Uart<'a>,
}

impl MuscaB1DefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            gp_timer: GPTimer::new_sec(),
            uart0: Uart::new_uart0_sec(),
            uart1: Uart::new_uart1_sec(),
        }
    }

    pub fn resolve_dependencies(&'static self) {
        kernel::deferred_call::DeferredCallClient::register(&self.uart0);
        kernel::deferred_call::DeferredCallClient::register(&self.uart1);
    }
}

impl InterruptService for MuscaB1DefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::GP_TIMER_COMBINED => {
                self.gp_timer.handle_interrupt();
                true
            }
            interrupts::GP_TIMER_INT0 => {
                self.gp_timer.handle_interrupt();
                true
            }
            interrupts::UART0_RX
            | interrupts::UART0_TX
            | interrupts::UART0_RT
            | interrupts::UART0_MS
            | interrupts::UART0_E
            | interrupts::UART0_COMBINED => {
                self.uart0.handle_interrupt();
                true
            }
            _ => false,
        }
    }
}
