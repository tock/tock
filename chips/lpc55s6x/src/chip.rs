// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Chip trait setup.
use core::fmt::Write;
use core::panic;

use cortexm33::{CortexM33, CortexMVariant};
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::Lpc55ImageHeader;
use crate::clocks::Clock;
use crate::ctimer0::LPCTimer;
use crate::flexcomm::Flexcomm;
use crate::gpio::Pins;
use crate::interrupts;
use crate::rng::RNG;
use crate::uart::Uart;

#[repr(u8)]
pub enum Processor {
    Processor0 = 0,
    Processor1 = 1,
}

pub struct Lpc55s69<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService> Lpc55s69<'a, I> {
    pub fn new(interrupt_service: &'a I) -> Self {
        // SAFETY: All LPC55S6x chips are Cortex-M33 cores with valid MPU hardware.
        let mpu = unsafe { cortexm33::mpu::new() };

        Self {
            mpu,
            userspace_kernel_boundary: cortexm33::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<I: InterruptService> Chip for Lpc55s69<'_, I> {
    type MPU = cortexm33::mpu::MPU<8>;
    type UserspaceKernelBoundary = cortexm33::syscall::SysCall;
    type ThreadIdProvider = cortexm33::thread_id::CortexMThreadIdProvider;

    fn init() {
        cortexm33::nvic::disable_all();
        cortexm33::nvic::clear_all_pending();

        unsafe {
            let lpc55_image_header: *const Lpc55ImageHeader =
                core::ptr::addr_of!(crate::LPC55_IMAGE_HEADER);
            let lpc55_image_header: *const () = lpc55_image_header.cast();
            // The LPC55 image header is a superset of the Cortex-M33 vector
            // table: it places the image header fields that the boot ROM reads
            // in the vector slots that Cortex-M33 reserves (offsets 0x20, 0x24,
            // 0x28, and 0x34), so the header *is* the vector table. Point VTOR
            // at its start.
            cortexm33::scb::set_vector_table_offset(lpc55_image_header);
        }

        cortexm33::nvic::enable_all();
    }

    fn service_pending_interrupts(&self) {
        while let Some(interrupt) = cortexm33::nvic::next_pending() {
            if !self.interrupt_service.service_interrupt(interrupt) {
                panic!("unhandled interrupt {}", interrupt);
            }

            let n = cortexm33::nvic::Nvic::new(interrupt);
            n.clear_pending();
            n.enable();
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        cortexm33::nvic::has_pending()
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        let cap = kernel::create_capability!(kernel::capabilities::CpuStateControlCapability);
        cortexm33::support::wfi(&cap);
    }

    fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm33::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, writer: &mut dyn Write) {
        CortexM33::print_cortexm_state(writer);
    }
}

pub struct Lpc55s69DefaultPeripheral<'a> {
    pub pins: Pins<'a>,
    pub ctimer0: LPCTimer<'a>,
    pub uart: Uart<'a>,
    pub rng: RNG<'a>,
}

impl Lpc55s69DefaultPeripheral<'_> {
    pub fn new(clocks: &'static Clock, flexcomm: &'static Flexcomm) -> Self {
        Self {
            pins: Pins::new(),
            ctimer0: LPCTimer::new(),
            uart: Uart::new_uart0(clocks, flexcomm),
            rng: RNG::new(),
        }
    }

    pub fn init(&'static self) {
        kernel::deferred_call::DeferredCallClient::register(&self.rng);
    }
}

impl InterruptService for Lpc55s69DefaultPeripheral<'_> {
    fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            interrupts::GPIO_INT0_IRQ0 => {
                self.pins.handle_interrupt();
                true
            }
            interrupts::GPIO_INT0_IRQ1 => {
                self.pins.handle_interrupt();
                true
            }

            interrupts::GPIO_INT0_IRQ2 => {
                self.pins.handle_interrupt();
                true
            }
            interrupts::GPIO_INT0_IRQ3 => {
                self.pins.handle_interrupt();
                true
            }
            interrupts::GPIO_INT0_IRQ4 => {
                self.pins.handle_interrupt();
                true
            }
            interrupts::GPIO_INT0_IRQ5 => {
                self.pins.handle_interrupt();
                true
            }
            interrupts::GPIO_INT0_IRQ6 => {
                self.pins.handle_interrupt();
                true
            }
            interrupts::GPIO_INT0_IRQ7 => {
                self.pins.handle_interrupt();
                true
            }

            interrupts::CTIMER0 => {
                self.ctimer0.handle_interrupt();
                true
            }

            interrupts::FLEXCOMM0 => {
                self.uart.handle_interrupt();
                true
            }

            _ => true,
        }
    }
}
