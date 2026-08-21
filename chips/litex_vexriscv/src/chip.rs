// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! High-level setup and interrupt mapping for the chip.

use core::fmt::Write;
use core::ptr::addr_of;
use kernel::context_tokens::InterruptsDisabledContext;
use kernel::debug;
use kernel::platform::chip::{Chip, InterruptService};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use rv32i::csr::{CSR, mcause, mie::mie};
use rv32i::pmp::{PMPUserMPU, kernel_protection::KernelProtectionPMP};
use rv32i::syscall::SysCall;

use crate::interrupt_controller::VexRiscvInterruptController;

/// Global static variable for the InterruptController, as it must be
/// accessible to the raw interrupt handler functions
static mut INTERRUPT_CONTROLLER: VexRiscvInterruptController = VexRiscvInterruptController::new();

// The VexRiscv "Secure" variant of
// [pythondata-cpu-vexriscv](https://github.com/litex-hub/pythondata-cpu-vexriscv)
// has 16 PMP slots
pub struct LiteXVexRiscv<I: 'static + InterruptService> {
    soc_identifier: &'static str,
    userspace_kernel_boundary: SysCall,
    interrupt_controller: &'static VexRiscvInterruptController,
    pmp_mpu: PMPUserMPU<4, KernelProtectionPMP<16>>,
    interrupt_service: &'static I,
}

impl<I: 'static + InterruptService> LiteXVexRiscv<I> {
    pub unsafe fn new(
        soc_identifier: &'static str,
        interrupt_service: &'static I,
        pmp: KernelProtectionPMP<16>,
    ) -> Self {
        Self {
            soc_identifier,
            userspace_kernel_boundary: SysCall::new(),
            interrupt_controller: &*addr_of!(INTERRUPT_CONTROLLER),
            pmp_mpu: PMPUserMPU::new(pmp),
            interrupt_service,
        }
    }

    pub unsafe fn unmask_interrupts(&self) {
        VexRiscvInterruptController::unmask_all_interrupts();
    }

    fn handle_interrupts(&self) {
        while let Some(interrupt) = self.interrupt_controller.next_saved() {
            if !self.interrupt_service.service_interrupt(interrupt as u32) {
                debug!("Unknown interrupt: {}", interrupt);
            }
            self.with_interrupts_disabled(|interrupts_disabled| {
                self.interrupt_controller
                    .complete_saved(interrupt, interrupts_disabled);
            });
        }
    }
}

impl<I: 'static + InterruptService> kernel::platform::chip::Chip for LiteXVexRiscv<I> {
    type MPU = PMPUserMPU<4, KernelProtectionPMP<16>>;
    type UserspaceKernelBoundary = SysCall;
    type ThreadIdProvider = rv32i::thread_id::RiscvThreadIdProvider;

    fn init() {}

    fn mpu(&self) -> &Self::MPU {
        &self.pmp_mpu
    }

    fn userspace_kernel_boundary(&self) -> &SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        while self.interrupt_controller.next_saved().is_some() {
            self.handle_interrupts();
        }

        // Re-enable all MIE interrupts that we care about. Since we
        // looped until we handled them all, we can re-enable all of
        // them.
        CSR.mie.modify(mie::mext::SET);
    }

    fn has_pending_interrupts(&self) -> bool {
        self.interrupt_controller.next_saved().is_some()
    }

    fn sleep(&self) {
        unsafe {
            rv32i::support::wfi();
        }
    }

    fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&InterruptsDisabledContext) -> R,
    {
        rv32i::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(this: Option<&Self>, writer: &mut dyn Write) {
        let _ = writer.write_fmt(format_args!(
            "\r\n---| LiteX configuration for {} |---",
            this.map_or("unknown board (in trap handler thread)", |t| t
                .soc_identifier),
        ));
        rv32i::print_riscv_state(writer);
        if let Some(t) = this {
            let _ = writer.write_fmt(format_args!("{}", t.pmp_mpu.pmp));
        }
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

fn handle_interrupt(intr: mcause::Interrupt, interrupts_disabled: &InterruptsDisabledContext) {
    match intr {
        mcause::Interrupt::UserSoft
        | mcause::Interrupt::UserTimer
        | mcause::Interrupt::UserExternal => {
            debug!("unexpected user-mode interrupt");
        }
        mcause::Interrupt::SupervisorExternal
        | mcause::Interrupt::SupervisorTimer
        | mcause::Interrupt::SupervisorSoft => {
            debug!("unexpected supervisor-mode interrupt");
        }

        mcause::Interrupt::MachineSoft => {
            CSR.mie.modify(mie::msoft::CLEAR);
        }
        mcause::Interrupt::MachineTimer => {
            CSR.mie.modify(mie::mtimer::CLEAR);
        }
        mcause::Interrupt::MachineExternal => {
            // Mask the external interrupt source so this (level-triggered)
            // interrupt cannot re-trap before the bottom half drains and
            // completes it.
            CSR.mie.modify(mie::mext::CLEAR);

            // Safety: interrupts are disabled (we have an InterruptsDisabledContext
            // token), so this cannot race a concurrent access to the
            // interrupt controller.
            let interrupt_controller_ref = unsafe { &*addr_of!(INTERRUPT_CONTROLLER) };

            // Save the interrupts and check whether at least one
            // interrupt is to be handled
            //
            // If no interrupt was saved, unmask the external interrupt
            // source immediately
            if !interrupt_controller_ref.save_pending(interrupts_disabled) {
                CSR.mie.modify(mie::mext::SET);
            }
        }

        mcause::Interrupt::Unknown(_) => {
            debug!("interrupt of unknown cause");
        }
    }
}

/// Trap handler for board/chip specific code.
///
/// This gets called when an interrupt occurs while the chip is in
/// kernel mode.
#[export_name = "_start_trap_rust_from_kernel"]
pub unsafe extern "C" fn start_trap_rust() {
    // we were just called via a trap, and RISC-V hardware clears
    // `mstatus.MIE` on trap entry before any Rust code runs.
    let interrupts_disabled = kernel::mint_interrupts_disabled_context!();
    match mcause::Trap::from(CSR.mcause.extract()) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt, &interrupts_disabled);
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
    // we were just called via a trap, and RISC-V hardware clears
    // `mstatus.MIE` on trap entry before any Rust code runs.
    let interrupts_disabled = kernel::mint_interrupts_disabled_context!();
    match mcause::Trap::from(mcause_val as usize) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt, &interrupts_disabled);
        }
        _ => {
            panic!("unexpected non-interrupt\n");
        }
    }
}

/// Array used to track the "trap handler active" state per hart.
///
/// The `riscv` crate requires chip crates to allocate an array to
/// track whether any given hart is currently in a trap handler. The
/// array must be zero-initialized.
#[export_name = "_trap_handler_active"]
static mut TRAP_HANDLER_ACTIVE: [usize; 1] = [0; 1];
