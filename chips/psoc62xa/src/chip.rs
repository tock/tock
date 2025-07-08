// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::{cpuss, gpio, hsiom, peri, scb, srss, tcpwm};
use cortexm0p::{CortexM0P, CortexMVariant};

pub struct Psoc62xa<'a, I: InterruptService + 'a> {
    mpu: cortexm0p::mpu::MPU,
    userspace_kernel_boundary: cortexm0p::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService> Psoc62xa<'a, I> {
    pub fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: unsafe { cortexm0p::mpu::new() },
            userspace_kernel_boundary: unsafe { cortexm0p::syscall::SysCall::new() },
            interrupt_service,
        }
    }
}

impl<I: InterruptService> Chip for Psoc62xa<'_, I> {
    type MPU = cortexm0p::mpu::MPU;
    type UserspaceKernelBoundary = cortexm0p::syscall::SysCall;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
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

    unsafe fn print_state(&self, writer: &mut dyn core::fmt::Write) {
        CortexM0P::print_cortexm_state(writer);
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm0p::nvic::has_pending() }
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(interrupt) = cortexm0p::nvic::next_pending() {
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
            while let Some(interrupt) = cortexm0p::nvic::next_pending() {
                let nvic = cortexm0p::nvic::Nvic::new(interrupt);
                nvic.clear_pending();
                nvic.enable();
            }
        }
    }
}

pub struct PsoC62xaDefaultPeripherals<'a> {
    pub cpuss: cpuss::Cpuss,
    pub gpio: gpio::PsocPins<'a>,
    pub hsiom: hsiom::Hsiom,
    pub peri: peri::Peri,
    pub scb: scb::Scb<'a>,
    pub srss: srss::Srss,
    pub tcpwm: tcpwm::Tcpwm0<'a>,
}

impl PsoC62xaDefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            cpuss: cpuss::Cpuss::new(),
            gpio: gpio::PsocPins::new(),
            hsiom: hsiom::Hsiom::new(),
            peri: peri::Peri::new(),
            scb: scb::Scb::new(),
            srss: srss::Srss::new(),
            tcpwm: tcpwm::Tcpwm0::new(),
        }
    }
}

impl InterruptService for PsoC62xaDefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            0 => {
                self.scb.handle_interrupt();
                self.tcpwm.handle_interrupt();
            }
            1 => {
                // We use interrupt number 1 for GPIO so we don't
                // check for all of the GPIO ports on every non-GPIO
                // releated interrupt.
                self.gpio.handle_interrupt();
            }
            _ => return false,
        }
        true
    }
}
