// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::fmt::Write;
use cortexm33::{CortexM33, CortexMVariant, nvic};
use kernel::platform::chip::InterruptService;
use kernel::utilities::StaticRef;

//
// Peripheral Registers Instantiations (secure aliases)
//

const CLOCK_BASE: StaticRef<crate::clock::ClockRegisters> =
    unsafe { StaticRef::new(0x50005000 as *const crate::clock::ClockRegisters) };

const RESET_BASE: StaticRef<crate::network::ResetRegisters> =
    unsafe { StaticRef::new(0x50005000 as *const crate::network::ResetRegisters) };

const PIN_CNF_PORT0: StaticRef<crate::network::PinCnfRegisters> =
    unsafe { StaticRef::new(0x50842000 as *const crate::network::PinCnfRegisters) };

const PIN_CNF_PORT1: StaticRef<crate::network::PinCnfRegisters> =
    unsafe { StaticRef::new(0x50842300 as *const crate::network::PinCnfRegisters) };

const RTC1_BASE: StaticRef<crate::rtc::RtcRegisters> =
    unsafe { StaticRef::new(0x50015000 as *const crate::rtc::RtcRegisters) };

pub const UARTE0_BASE: StaticRef<crate::uarte::UarteRegisters> =
    unsafe { StaticRef::new(0x50008000 as *const crate::uarte::UarteRegisters) };

const USBD_BASE: StaticRef<crate::usbd::UsbdRegisters<'static>> =
    unsafe { StaticRef::new(0x50036000 as *const crate::usbd::UsbdRegisters<'static>) };

const USBREG_BASE: StaticRef<crate::usbreg::UsbRegRegisters> =
    unsafe { StaticRef::new(0x50037000 as *const crate::usbreg::UsbRegRegisters) };

pub struct Nrf5340<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService + 'a> Nrf5340<'a, I> {
    pub fn new(interrupt_service: &'a I) -> Self {
        // SAFETY: The nRF5340 application core is a Cortex-M33 with an MPU.
        let mpu = unsafe { cortexm33::mpu::new::<8>() };

        Self {
            mpu,
            userspace_kernel_boundary: cortexm33::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

/// This struct, when initialized, instantiates all peripheral drivers for the
/// nRF5340 application core.
pub struct Nrf5340DefaultPeripherals<'a> {
    pub clock: crate::clock::Clock,
    pub gpio_port: crate::gpio::Port<'a, { crate::gpio::NUM_PINS }>,
    pub network: crate::network::NetworkCore,
    pub rtc: crate::rtc::Rtc<'a>,
    pub uarte0: crate::uarte::Uarte<'a>,
    pub usbd: crate::usbd::Usbd<'a>,
    pub usbreg: crate::usbreg::UsbReg<'a>,
}

impl Nrf5340DefaultPeripherals<'_> {
    /// Create default peripherals for the nRF5340 application core.
    ///
    /// # Safety
    ///
    /// Some of these peripherals use DMA. As such, the default peripherals must
    /// be unique. This requires:
    ///
    /// - This constructor must be called at most once.
    /// - There must not be additional instances of the DMA-enabled peripheral
    ///   drivers.
    /// - There must not be any other code that accesses the DMA buffer and
    ///   length registers of the DMA-enabled peripherals.
    pub unsafe fn new() -> Self {
        // SAFETY: See function-level doc.
        let uarte0_registers = unsafe { crate::uarte::UarteRegistersManager::new(UARTE0_BASE) };

        Self {
            clock: crate::clock::Clock::new(CLOCK_BASE),
            gpio_port: crate::gpio::nrf5340_gpio_create(),
            network: crate::network::NetworkCore::new(RESET_BASE, [PIN_CNF_PORT0, PIN_CNF_PORT1]),
            rtc: crate::rtc::Rtc::new(RTC1_BASE),
            uarte0: crate::uarte::Uarte::new(uarte0_registers),
            usbd: crate::usbd::Usbd::new(USBD_BASE, None),
            usbreg: crate::usbreg::UsbReg::new(USBREG_BASE),
        }
    }

    // Necessary for setting up circular dependencies
    pub fn init(&'static self) {
        self.usbreg.set_usbd(&self.usbd);
        self.usbd.set_power_ref(&self.usbreg);
    }
}

impl InterruptService for Nrf5340DefaultPeripherals<'_> {
    fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            crate::peripheral_interrupts::GPIOTE0 => self.gpio_port.handle_interrupt(),
            crate::peripheral_interrupts::RTC1 => self.rtc.handle_interrupt(),
            crate::peripheral_interrupts::SERIAL0 => self.uarte0.handle_interrupt(),
            crate::peripheral_interrupts::USBD => self.usbd.handle_interrupt(),
            crate::peripheral_interrupts::USBREGULATOR => self.usbreg.handle_interrupt(),
            _ => return false,
        }
        true
    }
}

impl<'a, I: InterruptService + 'a> kernel::platform::chip::Chip for Nrf5340<'a, I> {
    type MPU = cortexm33::mpu::MPU<8>;
    type UserspaceKernelBoundary = cortexm33::syscall::SysCall;
    type ThreadIdProvider = cortexm33::thread_id::CortexMThreadIdProvider;

    fn init() {
        // # Safety
        //
        // We are setting up the nRF5340 application core in secure state, so
        // these memory locations are valid and this is OK to call.
        unsafe {
            crate::crt1::fix_errata();
        }
        crate::crt1::initialize_vector_table();

        nvic::enable_all();
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        while let Some(interrupt) = nvic::next_pending() {
            if !self.interrupt_service.service_interrupt(interrupt) {
                panic!("unhandled interrupt {}", interrupt);
            }
            let n = nvic::Nvic::new(interrupt);
            n.clear_pending();
            n.enable();
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        nvic::has_pending()
    }

    fn sleep(&self) {
        unsafe {
            cortexm33::support::wfi();
        }
    }

    fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm33::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, write: &mut dyn Write) {
        CortexM33::print_cortexm_state(write);
    }
}
