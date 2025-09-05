// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Chip trait setup.

use core::fmt::Write;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::clocks::Clocks;
use crate::gpio::{RPPins, SIO};
use crate::interrupts;
use crate::resets::Resets;
use crate::ticks::Ticks;
use crate::timer::RPTimer;
use crate::uart::Uart;
use crate::xosc::Xosc;
use cortexm33::{interrupt_mask, CortexM33, CortexMVariant};

#[repr(u8)]
pub enum Processor {
    Processor0 = 0,
    Processor1 = 1,
}

pub struct Rp2350<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
    sio: &'a SIO,
    processor0_interrupt_mask: (u128, u128),
    processor1_interrupt_mask: (u128, u128),
}

impl<'a, I: InterruptService> Rp2350<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I, sio: &'a SIO) -> Self {
        Self {
            mpu: cortexm33::mpu::new(),
            userspace_kernel_boundary: cortexm33::syscall::SysCall::new(),
            interrupt_service,
            sio,
            processor0_interrupt_mask: interrupt_mask!(interrupts::PROC1_IRQ_CTI),
            processor1_interrupt_mask: interrupt_mask!(interrupts::PROC0_IRQ_CTI),
        }
    }
}

impl<I: InterruptService> Chip for Rp2350<'_, I> {
    type MPU = cortexm33::mpu::MPU<8>;
    type UserspaceKernelBoundary = cortexm33::syscall::SysCall;
    type ThreadIdProvider = cortexm33::thread_id::CortexMThreadIdProvider;

    fn service_pending_interrupts(&self) {
        unsafe {
            let mask = match self.sio.get_processor() {
                Processor::Processor0 => self.processor0_interrupt_mask,
                Processor::Processor1 => self.processor1_interrupt_mask,
            };
            while let Some(interrupt) = cortexm33::nvic::next_pending_with_mask(mask) {
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
        let mask = match self.sio.get_processor() {
            Processor::Processor0 => self.processor0_interrupt_mask,
            Processor::Processor1 => self.processor1_interrupt_mask,
        };
        unsafe { cortexm33::nvic::has_pending_with_mask(mask) }
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

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        CortexM33::print_cortexm_state(writer);
    }
}

pub struct Rp2350DefaultPeripherals<'a> {
    pub clocks: Clocks,
    pub pins: RPPins<'a>,
    pub resets: Resets,
    pub sio: SIO,
    pub ticks: Ticks,
    pub timer0: RPTimer<'a>,
    pub uart0: Uart<'a>,
    pub uart1: Uart<'a>,
    pub xosc: Xosc,
}

impl Rp2350DefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            clocks: Clocks::new(),
            pins: RPPins::new(),
            resets: Resets::new(),
            sio: SIO::new(),
            ticks: Ticks::new(),
            timer0: RPTimer::new_timer0(),
            uart0: Uart::new_uart0(),
            uart1: Uart::new_uart1(),
            xosc: Xosc::new(),
        }
    }

    pub fn resolve_dependencies(&'static self) {
        self.uart0.set_clocks(&self.clocks);
        self.ticks.set_timer0_generator();
        self.ticks.set_timer1_generator();
        kernel::deferred_call::DeferredCallClient::register(&self.uart0);
        kernel::deferred_call::DeferredCallClient::register(&self.uart1);
    }
}

impl InterruptService for Rp2350DefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::TIMER0_IRQ_0 => {
                self.timer0.handle_interrupt();
                true
            }
            interrupts::SIO_IRQ_FIFO => {
                self.sio.handle_proc_interrupt(self.sio.get_processor());
                true
            }
            interrupts::UART0_IRQ => {
                self.uart0.handle_interrupt();
                true
            }
            _ => false,
        }
    }
}
