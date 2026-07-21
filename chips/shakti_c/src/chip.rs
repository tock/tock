// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! High-level setup and interrupt mapping for the SHAKTI C-Class test SoC.
//!
//! The simulation SoC has no external interrupt controller (the PLIC inputs are
//! tied to zero), so the only interrupt source is the CLINT (machine timer and
//! machine software). The trap handler therefore only ever has to service those.

use core::fmt::Write;

use kernel::platform::chip::Chip;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};

use rv64i::csr::{mcause, mie::mie, mip::mip, CSR};
use rv64i::pmp::{simple::SimplePMP, PMPUserMPU};

use crate::clint;

/// CLINT machine timer, clocked at the SoC's 10 MHz `timebase-frequency`.
///
/// This is the SHAKTI-specific 64-bit-access CLINT driver, not `sifive::clint`:
/// the SHAKTI `mkclint_axi4` mis-handles 32-bit reads of the 64-bit `mtime`
/// register (see [`crate::clint`]).
pub type ShaktiCClint<'a> = crate::clint::Clint<'a>;

/// The default peripherals available on the SHAKTI C-Class test SoC.
pub struct ShaktiCDefaultPeripherals<'a> {
    pub uart0: crate::uart::Uart<'a>,
    pub timer: ShaktiCClint<'a>,
}

impl Default for ShaktiCDefaultPeripherals<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaktiCDefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            uart0: crate::uart::Uart::new(crate::uart::UART0_BASE),
            timer: ShaktiCClint::new(&clint::CLINT_BASE),
        }
    }

    /// Register deferred-call clients. Must be called once the peripherals have
    /// a `'static` lifetime.
    pub fn init(&'static self) {
        kernel::deferred_call::DeferredCallClient::register(&self.uart0);
    }
}

pub struct ShaktiC<'a> {
    userspace_kernel_boundary: rv64i::syscall::SysCall,
    pmp: PMPUserMPU<2, SimplePMP<4>>,
    timer: &'a ShaktiCClint<'a>,
}

impl<'a> ShaktiC<'a> {
    /// # Safety
    ///
    /// Reads the number of implemented PMP entries from the hardware; must be
    /// called on the SHAKTI C-Class (which implements 4 PMP entries).
    pub unsafe fn new(timer: &'a ShaktiCClint<'a>) -> Self {
        Self {
            userspace_kernel_boundary: rv64i::syscall::SysCall::new(),
            pmp: PMPUserMPU::new(SimplePMP::new().unwrap()),
            timer,
        }
    }
}

impl Chip for ShaktiC<'_> {
    type MPU = PMPUserMPU<2, SimplePMP<4>>;
    type UserspaceKernelBoundary = rv64i::syscall::SysCall;
    type ThreadIdProvider = rv64i::thread_id::RiscvThreadIdProvider;

    fn mpu(&self) -> &Self::MPU {
        &self.pmp
    }

    fn userspace_kernel_boundary(&self) -> &rv64i::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        loop {
            let mip = CSR.mip.extract();

            if mip.is_set(mip::mtimer) {
                self.timer.handle_interrupt();
            }

            if !mip.any_matching_bits_set(mip::mtimer::SET) {
                break;
            }
        }

        // Re-enable the machine timer interrupt now that it has been serviced.
        CSR.mie.modify(mie::mtimer::SET);
    }

    fn has_pending_interrupts(&self) -> bool {
        CSR.mip.is_set(mip::mtimer)
    }

    fn sleep(&self) {
        unsafe {
            rv64i::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv64i::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, writer: &mut dyn Write) {
        rv64i::print_riscv_state(writer);
    }
}

fn handle_exception(exception: mcause::Exception) {
    match exception {
        mcause::Exception::UserEnvCall | mcause::Exception::SupervisorEnvCall => (),
        _ => panic!("fatal exception"),
    }
}

unsafe fn handle_interrupt(intr: mcause::Interrupt) {
    match intr {
        // Disable the source that fired; the kernel re-enables it after
        // servicing in `service_pending_interrupts`.
        mcause::Interrupt::MachineSoft => CSR.mie.modify(mie::msoft::CLEAR),
        mcause::Interrupt::MachineTimer => CSR.mie.modify(mie::mtimer::CLEAR),
        // No PLIC in this SoC, so no machine-external interrupts are expected;
        // ignore any other (e.g. spurious lower-privilege) interrupt.
        _ => {}
    }
}

/// Trap handler for chip-specific code, called from the shared `riscv` trap
/// assembly when an interrupt/exception occurs while the chip is in kernel mode.
#[export_name = "_start_trap_rust_from_kernel"]
pub unsafe extern "C" fn start_trap_rust() {
    match mcause::Trap::from(CSR.mcause.extract()) {
        mcause::Trap::Interrupt(interrupt) => handle_interrupt(interrupt),
        mcause::Trap::Exception(exception) => handle_exception(exception),
    }
}

/// Called if an interrupt occurs while an app was running. `mcause` is passed
/// in; this disables the interrupt that fired so it does not immediately refire.
#[export_name = "_disable_interrupt_trap_rust_from_app"]
pub unsafe extern "C" fn disable_interrupt_trap_handler(mcause_val: usize) {
    match mcause::Trap::from(mcause_val) {
        mcause::Trap::Interrupt(interrupt) => handle_interrupt(interrupt),
        _ => panic!("unexpected non-interrupt"),
    }
}

/// Per-hart "trap handler active" tracking array required by the `riscv` crate.
/// The SHAKTI C-Class test SoC runs a single hart (id 0).
#[export_name = "_trap_handler_active"]
static mut TRAP_HANDLER_ACTIVE: [usize; 1] = [0; 1];
