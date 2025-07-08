// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Generic support for all Cortex-M platforms.

#![no_std]

use core::fmt::Write;

pub mod dcb;
pub mod dwt;
pub mod mpu;
pub mod nvic;
pub mod scb;
pub mod support;
pub mod syscall;
pub mod systick;

// These constants are defined in the linker script.
extern "C" {
    static _szero: *const u32;
    static _ezero: *const u32;
    static _etext: *const u32;
    static _srelocate: *const u32;
    static _erelocate: *const u32;
}

/// Trait to encapsulate differences in between Cortex-M variants
///
/// This trait contains functions and other associated data (constants) which
/// differs in between different Cortex-M architecture variants (e.g. Cortex-M0,
/// Cortex-M4, etc.). It is not designed to be implemented by an instantiable
/// type and passed around as a runtime-accessible object, but is to be used as
/// a well-defined collection of functions and data to be exposed to
/// architecture-dependent code paths. Hence, implementations can use an `enum`
/// without variants to implement this trait.
///
/// The fact that some functions are proper trait functions, while others are
/// exposed via associated constants is necessitated by the associated const
/// functions being `#\[naked\]`. To wrap these functions in proper trait
/// functions would require these trait functions to also be `#\[naked\]` to
/// avoid generating a function prologue and epilogue, and we'd have to call the
/// respective backing function from within an asm! block. The associated
/// constants allow us to simply reference any function in scope and export it
/// through the provided CortexMVariant trait infrastructure.
// This approach outlined above has some significant benefits over passing
// functions via symbols, as done before this change (tock/tock#3080):
//
// - By using a trait carrying proper first-level Rust functions, the type
//   signatures of the trait and implementing functions are properly validated.
//   Before these changes, some Cortex-M variants previously used incorrect
//   type signatures (e.g. `*mut u8` instead of `*const usize`) for the
//   user_stack argument. It also ensures that all functions are provided by a
//   respective sub-architecture at compile time, instead of throwing linker
//   errors.
//
// - Determining the respective functions at compile time, Rust might be able to
//   perform more aggressive inlining, especially if more device-specific
//   proper Rust functions (non hardware-exposed symbols, i.e. not fault or
//   interrupt handlers) were to be added.
//
// - Most importantly, this avoid ambiguity with respect to a compiler fence
//   being inserted by the compiler around calls to switch_to_user. The asm!
//   macro in that function call will cause Rust to emit a compiler fence given
//   the nomem option is not passed, but the opaque extern "C" function call
//   obscured that code path. While this is probably fine and Rust is obliged
//   to generate a compiler fence when switching to C code, having a traceable
//   code path for Rust to the asm! macro will remove any remaining ambiguity
//   and allow us to argue against requiring volatile accesses to userspace
//   memory(during context switches). See tock/tock#2582 for further discussion
//   of this issue.
pub trait CortexMVariant {
    /// All ISRs not caught by a more specific handler are caught by this
    /// handler. This must ensure the interrupt is disabled (per Tock's
    /// interrupt model) and then as quickly as possible resume the main thread
    /// (i.e. leave the interrupt context). The interrupt will be marked as
    /// pending and handled when the scheduler checks if there are any pending
    /// interrupts.
    ///
    /// If the ISR is called while an app is running, this will switch control
    /// to the kernel.
    const GENERIC_ISR: unsafe extern "C" fn();

    /// The `systick_handler` is called when the systick interrupt occurs,
    /// signaling that an application executed for longer than its timeslice.
    /// This interrupt handler is no longer responsible for signaling to the
    /// kernel thread that an interrupt has occurred, but is slightly more
    /// efficient than the `generic_isr` handler on account of not needing to
    /// mark the interrupt as pending.
    const SYSTICK_HANDLER: unsafe extern "C" fn();

    /// This is called after a `svc` instruction, both when switching to
    /// userspace and when userspace makes a system call.
    const SVC_HANDLER: unsafe extern "C" fn();

    /// Hard fault handler.
    const HARD_FAULT_HANDLER: unsafe extern "C" fn();

    /// Assembly function called from `UserspaceKernelBoundary` to switch to an
    /// an application. This handles storing and restoring application state
    /// before and after the switch.
    unsafe fn switch_to_user(
        user_stack: *const usize,
        process_regs: &mut [usize; 8],
    ) -> *const usize;

    /// Format and display architecture-specific state useful for debugging.
    ///
    /// This is generally used after a `panic!()` to aid debugging.
    unsafe fn print_cortexm_state(writer: &mut dyn Write);
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn unhandled_interrupt() {
    use core::arch::asm;
    let mut interrupt_number: u32;

    // IPSR[8:0] holds the currently active interrupt
    asm!(
        "mrs r0, ipsr",
        out("r0") interrupt_number,
        options(nomem, nostack, preserves_flags)
    );

    interrupt_number &= 0x1ff;

    panic!("Unhandled Interrupt. ISR {} is active.", interrupt_number);
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    /// Assembly function to initialize the .bss and .data sections in RAM.
    ///
    /// We need to (unfortunately) do these operations in assembly because it is
    /// not valid to run Rust code without RAM initialized.
    ///
    /// See <https://github.com/tock/tock/issues/2222> for more information.
    pub fn initialize_ram_jump_to_main();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
"
    .section .initialize_ram_jump_to_main, \"ax\"
    .global initialize_ram_jump_to_main
    .thumb_func
  initialize_ram_jump_to_main:
    // Start by initializing .bss memory. The Tock linker script defines
    // `_szero` and `_ezero` to mark the .bss segment.
    ldr r0, ={sbss}     // r0 = first address of .bss
    ldr r1, ={ebss}     // r1 = first address after .bss

    movs r2, #0         // r2 = 0

  100: // bss_init_loop
    cmp r1, r0          // We increment r0. Check if we have reached r1
                        // (end of .bss), and stop if so.
    beq 101f            // If r0 == r1, we are done.
    stm r0!, {{r2}}     // Write a word to the address in r0, and increment r0.
                        // Since r2 contains zero, this will clear the memory
                        // pointed to by r0. Using `stm` (store multiple) with the
                        // bang allows us to also increment r0 automatically.
    b 100b              // Continue the loop.

  101: // bss_init_done

    // Now initialize .data memory. This involves coping the values right at the
    // end of the .text section (in flash) into the .data section (in RAM).
    ldr r0, ={sdata}    // r0 = first address of data section in RAM
    ldr r1, ={edata}    // r1 = first address after data section in RAM
    ldr r2, ={etext}    // r2 = address of stored data initial values

  200: // data_init_loop
    cmp r1, r0          // We increment r0. Check if we have reached the end
                        // of the data section, and if so we are done.
    beq 201f            // r0 == r1, and we have iterated through the .data section
    ldm r2!, {{r3}}     // r3 = *(r2), r2 += 1. Load the initial value into r3,
                        // and use the bang to increment r2.
    stm r0!, {{r3}}     // *(r0) = r3, r0 += 1. Store the value to memory, and
                        // increment r0.
    b 200b              // Continue the loop.

  201: // data_init_done

    // Now that memory has been initialized, we can jump to main() where the
    // board initialization takes place and Rust code starts.
    bl main
    ",
    sbss = sym _szero,
    ebss = sym _ezero,
    sdata = sym _srelocate,
    edata = sym _erelocate,
    etext = sym _etext,
);

pub unsafe fn print_cortexm_state(writer: &mut dyn Write) {
    let _ccr = syscall::SCB_REGISTERS[0];
    let cfsr = syscall::SCB_REGISTERS[1];
    let hfsr = syscall::SCB_REGISTERS[2];
    let mmfar = syscall::SCB_REGISTERS[3];
    let bfar = syscall::SCB_REGISTERS[4];

    let iaccviol = (cfsr & 0x01) == 0x01;
    let daccviol = (cfsr & 0x02) == 0x02;
    let munstkerr = (cfsr & 0x08) == 0x08;
    let mstkerr = (cfsr & 0x10) == 0x10;
    let mlsperr = (cfsr & 0x20) == 0x20;
    let mmfarvalid = (cfsr & 0x80) == 0x80;

    let ibuserr = ((cfsr >> 8) & 0x01) == 0x01;
    let preciserr = ((cfsr >> 8) & 0x02) == 0x02;
    let impreciserr = ((cfsr >> 8) & 0x04) == 0x04;
    let unstkerr = ((cfsr >> 8) & 0x08) == 0x08;
    let stkerr = ((cfsr >> 8) & 0x10) == 0x10;
    let lsperr = ((cfsr >> 8) & 0x20) == 0x20;
    let bfarvalid = ((cfsr >> 8) & 0x80) == 0x80;

    let undefinstr = ((cfsr >> 16) & 0x01) == 0x01;
    let invstate = ((cfsr >> 16) & 0x02) == 0x02;
    let invpc = ((cfsr >> 16) & 0x04) == 0x04;
    let nocp = ((cfsr >> 16) & 0x08) == 0x08;
    let unaligned = ((cfsr >> 16) & 0x100) == 0x100;
    let divbyzero = ((cfsr >> 16) & 0x200) == 0x200;

    let vecttbl = (hfsr & 0x02) == 0x02;
    let forced = (hfsr & 0x40000000) == 0x40000000;

    let _ = writer.write_fmt(format_args!("\r\n---| Cortex-M Fault Status |---\r\n"));

    if iaccviol {
        let _ = writer.write_fmt(format_args!(
            "Instruction Access Violation:       {}\r\n",
            iaccviol
        ));
    }
    if daccviol {
        let _ = writer.write_fmt(format_args!(
            "Data Access Violation:              {}\r\n",
            daccviol
        ));
    }
    if munstkerr {
        let _ = writer.write_fmt(format_args!(
            "Memory Management Unstacking Fault: {}\r\n",
            munstkerr
        ));
    }
    if mstkerr {
        let _ = writer.write_fmt(format_args!(
            "Memory Management Stacking Fault:   {}\r\n",
            mstkerr
        ));
    }
    if mlsperr {
        let _ = writer.write_fmt(format_args!(
            "Memory Management Lazy FP Fault:    {}\r\n",
            mlsperr
        ));
    }

    if ibuserr {
        let _ = writer.write_fmt(format_args!(
            "Instruction Bus Error:              {}\r\n",
            ibuserr
        ));
    }
    if preciserr {
        let _ = writer.write_fmt(format_args!(
            "Precise Data Bus Error:             {}\r\n",
            preciserr
        ));
    }
    if impreciserr {
        let _ = writer.write_fmt(format_args!(
            "Imprecise Data Bus Error:           {}\r\n",
            impreciserr
        ));
    }
    if unstkerr {
        let _ = writer.write_fmt(format_args!(
            "Bus Unstacking Fault:               {}\r\n",
            unstkerr
        ));
    }
    if stkerr {
        let _ = writer.write_fmt(format_args!(
            "Bus Stacking Fault:                 {}\r\n",
            stkerr
        ));
    }
    if lsperr {
        let _ = writer.write_fmt(format_args!(
            "Bus Lazy FP Fault:                  {}\r\n",
            lsperr
        ));
    }
    if undefinstr {
        let _ = writer.write_fmt(format_args!(
            "Undefined Instruction Usage Fault:  {}\r\n",
            undefinstr
        ));
    }
    if invstate {
        let _ = writer.write_fmt(format_args!(
            "Invalid State Usage Fault:          {}\r\n",
            invstate
        ));
    }
    if invpc {
        let _ = writer.write_fmt(format_args!(
            "Invalid PC Load Usage Fault:        {}\r\n",
            invpc
        ));
    }
    if nocp {
        let _ = writer.write_fmt(format_args!(
            "No Coprocessor Usage Fault:         {}\r\n",
            nocp
        ));
    }
    if unaligned {
        let _ = writer.write_fmt(format_args!(
            "Unaligned Access Usage Fault:       {}\r\n",
            unaligned
        ));
    }
    if divbyzero {
        let _ = writer.write_fmt(format_args!(
            "Divide By Zero:                     {}\r\n",
            divbyzero
        ));
    }

    if vecttbl {
        let _ = writer.write_fmt(format_args!(
            "Bus Fault on Vector Table Read:     {}\r\n",
            vecttbl
        ));
    }
    if forced {
        let _ = writer.write_fmt(format_args!(
            "Forced Hard Fault:                  {}\r\n",
            forced
        ));
    }

    if mmfarvalid {
        let _ = writer.write_fmt(format_args!(
            "Faulting Memory Address:            {:#010X}\r\n",
            mmfar
        ));
    }
    if bfarvalid {
        let _ = writer.write_fmt(format_args!(
            "Bus Fault Address:                  {:#010X}\r\n",
            bfar
        ));
    }

    if cfsr == 0 && hfsr == 0 {
        let _ = writer.write_fmt(format_args!("No Cortex-M faults detected.\r\n"));
    } else {
        let _ = writer.write_fmt(format_args!(
            "Fault Status Register (CFSR):       {:#010X}\r\n",
            cfsr
        ));
        let _ = writer.write_fmt(format_args!(
            "Hard Fault Status Register (HFSR):  {:#010X}\r\n",
            hfsr
        ));
    }
}

///////////////////////////////////////////////////////////////////
// Mock implementations for running tests on CI.
//
// Since tests run on the local architecture, we have to remove any
// ARM assembly since it will not compile.
///////////////////////////////////////////////////////////////////

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe extern "C" fn unhandled_interrupt() {
    unimplemented!()
}

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe extern "C" fn initialize_ram_jump_to_main() {
    unimplemented!()
}
