// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use core::ptr::addr_of;
use kernel::debug;
use kernel::platform::chip::Chip;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use rv32i::csr;
use rv32i::csr::{mcause, mie::mie, mip::mip, CSR};
use rv32i::pmp::{simple::SimplePMP, PMPUserMPU};

use crate::plic::PLIC;
use kernel::hil::time::Freq32KHz;
use kernel::platform::chip::InterruptService;
use sifive::plic::Plic;

pub type E310xClint<'a> = sifive::clint::Clint<'a, Freq32KHz>;

pub struct E310x<'a, I: InterruptService + 'a> {
    userspace_kernel_boundary: rv32i::syscall::SysCall,
    pmp: PMPUserMPU<4, SimplePMP<8>>,
    plic: &'a Plic,
    timer: &'a E310xClint<'a>,
    plic_interrupt_service: &'a I,
}

pub struct E310xDefaultPeripherals<'a> {
    pub uart0: sifive::uart::Uart<'a>,
    pub uart1: sifive::uart::Uart<'a>,
    pub gpio_port: crate::gpio::Port<'a>,
    pub prci: sifive::prci::Prci,
    pub pwm0: sifive::pwm::Pwm,
    pub pwm1: sifive::pwm::Pwm,
    pub pwm2: sifive::pwm::Pwm,
    pub rtc: sifive::rtc::Rtc,
    pub watchdog: sifive::watchdog::Watchdog,
}

impl E310xDefaultPeripherals<'_> {
    pub fn new(clock_frequency: u32) -> Self {
        Self {
            uart0: sifive::uart::Uart::new(crate::uart::UART0_BASE, clock_frequency),
            uart1: sifive::uart::Uart::new(crate::uart::UART1_BASE, clock_frequency),
            gpio_port: crate::gpio::Port::new(),
            prci: sifive::prci::Prci::new(crate::prci::PRCI_BASE),
            pwm0: sifive::pwm::Pwm::new(crate::pwm::PWM0_BASE),
            pwm1: sifive::pwm::Pwm::new(crate::pwm::PWM1_BASE),
            pwm2: sifive::pwm::Pwm::new(crate::pwm::PWM2_BASE),
            rtc: sifive::rtc::Rtc::new(crate::rtc::RTC_BASE),
            watchdog: sifive::watchdog::Watchdog::new(crate::watchdog::WATCHDOG_BASE),
        }
    }

    // Resolve any circular dependencies and register deferred calls
    pub fn init(&'static self) {
        kernel::deferred_call::DeferredCallClient::register(&self.uart0);
        kernel::deferred_call::DeferredCallClient::register(&self.uart1);
    }
}

impl InterruptService for E310xDefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, _interrupt: u32) -> bool {
        false
    }
}

impl<'a, I: InterruptService + 'a> E310x<'a, I> {
    pub unsafe fn new(plic_interrupt_service: &'a I, timer: &'a E310xClint<'a>) -> Self {
        Self {
            userspace_kernel_boundary: rv32i::syscall::SysCall::new(),
            pmp: PMPUserMPU::new(SimplePMP::new().unwrap()),
            plic: &*addr_of!(PLIC),
            timer,
            plic_interrupt_service,
        }
    }

    pub unsafe fn enable_plic_interrupts(&self) {
        /* E31 core manual
         * https://sifive.cdn.prismic.io/sifive/c29f9c69-5254-4f9a-9e18-24ea73f34e81_e31_core_complex_manual_21G2.pdf
         * PLIC Chapter 9.4 p.114: A pending bit in the PLIC core can be cleared
         * by setting the associated enable bit then performing a claim.
         */

        // first disable interrupts globally
        let old_mie = csr::CSR
            .mstatus
            .read_and_clear_field(csr::mstatus::mstatus::mie);

        self.plic.enable_all();
        self.plic.clear_all_pending();

        // restore the old external interrupt enable bit
        csr::CSR
            .mstatus
            .modify(csr::mstatus::mstatus::mie.val(old_mie));
    }

    unsafe fn handle_plic_interrupts(&self) {
        while let Some(interrupt) = self.plic.get_saved_interrupts() {
            if !self.plic_interrupt_service.service_interrupt(interrupt) {
                debug!("Pidx {}", interrupt);
            }
            self.atomic(|| {
                self.plic.complete(interrupt);
            });
        }
    }
}

impl<'a, I: InterruptService + 'a> kernel::platform::chip::Chip for E310x<'a, I> {
    type MPU = PMPUserMPU<4, SimplePMP<8>>;
    type UserspaceKernelBoundary = rv32i::syscall::SysCall;

    fn mpu(&self) -> &Self::MPU {
        &self.pmp
    }

    fn userspace_kernel_boundary(&self) -> &rv32i::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        loop {
            let mip = CSR.mip.extract();

            if mip.is_set(mip::mtimer) {
                self.timer.handle_interrupt();
            }
            if self.plic.get_saved_interrupts().is_some() {
                unsafe {
                    self.handle_plic_interrupts();
                }
            }

            if !mip.any_matching_bits_set(mip::mtimer::SET)
                && self.plic.get_saved_interrupts().is_none()
            {
                break;
            }
        }

        // Re-enable all MIE interrupts that we care about. Since we looped
        // until we handled them all, we can re-enable all of them.
        CSR.mie.modify(mie::mext::SET + mie::mtimer::SET);
    }

    fn has_pending_interrupts(&self) -> bool {
        // First check if the global machine timer interrupt is set.
        // We would also need to check for additional global interrupt bits
        // if there were to be used for anything in the future.
        if CSR.mip.is_set(mip::mtimer) {
            return true;
        }

        // Then we can check the PLIC.
        self.plic.get_saved_interrupts().is_some()
    }

    fn sleep(&self) {
        unsafe {
            rv32i::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv32i::support::atomic(f)
    }

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        rv32i::print_riscv_state(writer);
    }
}

fn handle_exception(exception: mcause::Exception) {
    match exception {
        mcause::Exception::UserEnvCall | mcause::Exception::SupervisorEnvCall => (),

        mcause::Exception::InstructionMisaligned
        | mcause::Exception::InstructionFault
        | mcause::Exception::IllegalInstruction
        | mcause::Exception::Breakpoint
        | mcause::Exception::LoadMisaligned
        | mcause::Exception::LoadFault
        | mcause::Exception::StoreMisaligned
        | mcause::Exception::StoreFault
        | mcause::Exception::MachineEnvCall
        | mcause::Exception::InstructionPageFault
        | mcause::Exception::LoadPageFault
        | mcause::Exception::StorePageFault
        | mcause::Exception::Unknown => {
            panic!("fatal exception");
        }
    }
}

unsafe fn handle_interrupt(intr: mcause::Interrupt) {
    match intr {
        mcause::Interrupt::UserSoft
        | mcause::Interrupt::UserTimer
        | mcause::Interrupt::UserExternal => {
            panic!("unexpected user-mode interrupt");
        }
        mcause::Interrupt::SupervisorExternal
        | mcause::Interrupt::SupervisorTimer
        | mcause::Interrupt::SupervisorSoft => {
            panic!("unexpected supervisor-mode interrupt");
        }

        mcause::Interrupt::MachineSoft => {
            CSR.mie.modify(mie::msoft::CLEAR);
        }
        mcause::Interrupt::MachineTimer => {
            CSR.mie.modify(mie::mtimer::CLEAR);
        }
        mcause::Interrupt::MachineExternal => {
            // We received an interrupt, disable interrupts while we handle them
            CSR.mie.modify(mie::mext::CLEAR);

            // Claim the interrupt, unwrap() as we know an interrupt exists
            // Once claimed this interrupt won't fire until it's completed
            // NOTE: The interrupt is no longer pending in the PLIC
            loop {
                let interrupt = (*addr_of!(PLIC)).next_pending();

                match interrupt {
                    Some(irq) => {
                        // Safe as interrupts are disabled
                        (*addr_of!(PLIC)).save_interrupt(irq);
                    }
                    None => {
                        // Enable generic interrupts
                        CSR.mie.modify(mie::mext::SET);

                        break;
                    }
                }
            }
        }

        mcause::Interrupt::Unknown(_) => {
            panic!("interrupt of unknown cause");
        }
    }
}

/// Trap handler for board/chip specific code.
///
/// For the e310 this gets called when an interrupt occurs while the chip is
/// in kernel mode.
#[export_name = "_start_trap_rust_from_kernel"]
pub unsafe extern "C" fn start_trap_rust() {
    match mcause::Trap::from(CSR.mcause.extract()) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        mcause::Trap::Exception(exception) => {
            handle_exception(exception);
        }
    }
}

/// Function that gets called if an interrupt occurs while an app was running.
///
/// mcause is passed in, and this function should correctly handle disabling the
/// interrupt that fired so that it does not trigger again.
#[export_name = "_disable_interrupt_trap_rust_from_app"]
pub unsafe extern "C" fn disable_interrupt_trap_handler(mcause_val: u32) {
    match mcause::Trap::from(mcause_val as usize) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        _ => {
            panic!("unexpected non-interrupt\n");
        }
    }
}
