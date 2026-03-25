// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Chip trait setup.

use core::fmt::Write;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::interrupts;
use crate::peri_clk::PeriPClk;
use crate::scb::Scb;
use crate::srss::Srss;
use crate::tcpwm::Tcpwm0;
use cortexm33::{CortexM33, CortexMVariant};

pub struct Psc3<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService> Psc3<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm33::mpu::new(),
            userspace_kernel_boundary: cortexm33::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<I: InterruptService> Chip for Psc3<'_, I> {
    type MPU = cortexm33::mpu::MPU<8>;
    type UserspaceKernelBoundary = cortexm33::syscall::SysCall;
    type ThreadIdProvider = cortexm33::thread_id::CortexMThreadIdProvider;

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = cortexm33::nvic::next_pending() {
                // ignore PROC1_IRQ_CTI as it is intended for processor 1
                // not able to unset its pending status
                // probably only processor 1 can unset the pending by reading the fifo
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

pub struct Psc3DefaultPeripherals<'a> {
    // pub cpuss: cpuss::Cpuss,
    // pub gpio: gpio::PsocPins<'a>,
    // pub hsiom: hsiom::Hsiom,
    pub peri_clk: PeriPClk,
    pub scb3: Scb<'a>,
    pub srss: Srss,
    pub tcpwm: Tcpwm0<'a>,
}

impl Psc3DefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            scb3: Scb::new(),
            peri_clk: PeriPClk::new(),
            srss: Srss::new(),
            tcpwm: Tcpwm0::new(),
        }
    }

    pub fn init(&self) {
        self.srss.init_clock();
        self.peri_clk.init_clocks();
        self.peri_clk.init_peripherals();
    }
}

impl InterruptService for Psc3DefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::TCPWM_0_INTERRUPTS_0 => {
                self.tcpwm.handle_interrupt();
                true
            }
            interrupts::SCB_5_INTERRUPT => {
                self.scb3.handle_interrupt();
                true
            }
            _ => false,
        }
    }
}
