// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Chip trait setup.

use core::fmt::Write;
use cortexm4f::{CortexM4F, CortexMVariant};
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

pub struct Apollo3<I: InterruptService + 'static> {
    mpu: cortexm4f::mpu::MPU,
    userspace_kernel_boundary: cortexm4f::syscall::SysCall,
    interrupt_service: &'static I,
}

impl<I: InterruptService + 'static> Apollo3<I> {
    pub unsafe fn new(interrupt_service: &'static I) -> Self {
        Self {
            mpu: cortexm4f::mpu::new(),
            userspace_kernel_boundary: cortexm4f::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

/// This struct, when initialized, instantiates all peripheral drivers for the apollo3.
///
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
    pub ios: crate::ios::Ios<'static>,
    pub ble: crate::ble::Ble<'static>,
    pub flash_ctrl: crate::flashctrl::FlashCtrl<'static>,
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
            ios: crate::ios::Ios::new(),
            ble: crate::ble::Ble::new(),
            flash_ctrl: crate::flashctrl::FlashCtrl::new(),
        }
    }

    pub fn init(&'static self) {
        kernel::deferred_call::DeferredCallClient::register(&self.flash_ctrl);
    }
}

impl kernel::platform::chip::InterruptService for Apollo3DefaultPeripherals {
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
            nvic::IOSLAVE | nvic::IOSLAVEACC => self.ios.handle_interrupt(),
            nvic::BLE => self.ble.handle_interrupt(),
            _ => return false,
        }
        true
    }
}

impl<I: InterruptService + 'static> Chip for Apollo3<I> {
    type MPU = cortexm4f::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4f::syscall::SysCall;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(interrupt) = cortexm4f::nvic::next_pending() {
                    if !self.interrupt_service.service_interrupt(interrupt) {
                        panic!("unhandled interrupt, {}", interrupt);
                    }

                    let n = cortexm4f::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm4f::nvic::has_pending() }
    }

    fn mpu(&self) -> &cortexm4f::mpu::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4f::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm4f::scb::set_sleepdeep();
            cortexm4f::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4f::support::atomic(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        CortexM4F::print_cortexm_state(write);
    }
}
