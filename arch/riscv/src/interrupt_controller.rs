// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! RISC-V support for configuring and decoding traps per interrupt controller.

use kernel::utilities::registers::LocalRegisterCopy;
use kernel::utilities::registers::interfaces::Writeable;

use crate::csr;
use crate::csr::mcause::mcause;
use crate::csr::mtvec::mtvec;

/// Interrupt controller
pub trait InterruptController {
    /// Returns the interrupt or exception code contained in `mcause`.
    fn exccode(mcause: usize) -> usize {
        LocalRegisterCopy::<usize, mcause::Register>::new(mcause).read(mcause::reason)
    }

    /// Tell the MCU what address the trap handler is located at, and initialize
    /// `mscratch` to zero, indicating kernel execution.
    ///
    /// The trap handler is called on exceptions and for interrupts.
    unsafe fn configure_trap_handler() {
        // Indicate to the trap handler that we are executing kernel code.
        csr::CSR.mscratch.set(0);

        // Set the machine-mode trap handler. By not configuring an S-mode or U-mode
        // trap handler, this should ensure that all traps are handled by the M-mode
        // handler.
        csr::CSR.mtvec.write(
            mtvec::trap_addr.val(crate::start_trap as extern "C" fn() -> ! as usize >> 2)
                + mtvec::mode::CLEAR,
        );
    }
}

/// CLINT mode
pub enum Clint {}

impl InterruptController for Clint {}
