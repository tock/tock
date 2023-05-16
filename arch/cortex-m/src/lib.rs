// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Generic support for all Cortex-M platforms.

#![crate_name = "cortexm"]
#![crate_type = "rlib"]
#![feature(naked_functions)]
#![no_std]

use core::fmt::Write;

pub mod mpu;
pub mod nvic;
pub mod scb;
pub mod support;
pub mod syscall;
pub mod systick;

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
//   signatures of the trait and implementing functions are properly
//   validated. Before these changes, some Cortex-M variants previously used
//   incorrect type signatures (e.g. `*mut u8` instead of `*const usize`) for
//   the user_stack argument. It also ensures that all functions are provided by
//   a respective sub-architecture at compile time, instead of throwing linker
//   errors.
//
// - Determining the respective functions at compile time, Rust might be able to
//   perform more aggressive inlining, especially if more device-specific proper
//   Rust functions (non hardware-exposed symbols, i.e. not fault or interrupt
//   handlers) were to be added.
//
// - Most importantly, this avoid ambiguity with respect to a compiler fence
//   being inserted by the compiler around calls to switch_to_user. The asm!
//   macro in that function call will cause Rust to emit a compiler fence given
//   the nomem option is not passed, but the opaque extern "C" function call
//   obscured that code path. While this is probably fine and Rust is obliged to
//   generate a compiler fence when switching to C code, having a traceable code
//   path for Rust to the asm! macro will remove any remaining ambiguity and
//   allow us to argue against requiring volatile accesses to userspace memory
//   (during context switches). See tock/tock#2582 for further discussion of
//   this issue.
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
    /// signaling that an application executed for longer than its
    /// timeslice. This interrupt handler is no longer responsible for signaling
    /// to the kernel thread that an interrupt has occurred, but is slightly
    /// more efficient than the `generic_isr` handler on account of not needing
    /// to mark the interrupt as pending.
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

// These constants are defined in the linker script.
extern "C" {
    static _estack: u32;
    static mut _sstack: u32;
    static mut _szero: u32;
    static mut _ezero: u32;
    static mut _etext: u32;
    static mut _srelocate: u32;
    static mut _erelocate: u32;
}

/// ARMv7-M systick handler function.
///
/// For documentation of this function, please see
/// [`CortexMVariant::SYSTICK_HANDLER`].
#[cfg(all(
    target_arch = "arm",
    target_feature = "v7",
    target_feature = "thumb-mode",
    target_os = "none"
))]
#[naked]
pub unsafe extern "C" fn systick_handler_arm_v7m() {
    use core::arch::asm;
    asm!(
        "
    // Use the CONTROL register to set the thread mode to privileged to switch
    // back to kernel mode.
    //
    // CONTROL[1]: Stack status
    //   0 = Default stack (MSP) is used
    //   1 = Alternate stack is used
    // CONTROL[0]: Mode
    //   0 = Privileged in thread mode
    //   1 = User state in thread mode
    mov r0, #0                        // r0 = 0
    msr CONTROL, r0                   // CONTROL = 0
    // CONTROL writes must be followed by an Instruction Synchronization Barrier
    // (ISB). https://developer.arm.com/documentation/dai0321/latest
    isb                               // synchronization barrier

    // Set the link register to the special EXC_RETURN value of 0xFFFFFFF9 which
    // instructs the CPU to run in thread mode with the main (kernel) stack.
    ldr lr, =0xFFFFFFF9               // LR = 0xFFFFFFF9

    // This will resume in the switch_to_user function where application state
    // is saved and the scheduler can choose what to do next.
    bx lr
    ",
        options(noreturn)
    );
}

/// Handler of `svc` instructions on ARMv7-M.
///
/// For documentation of this function, please see
/// [`CortexMVariant::SVC_HANDLER`].
#[cfg(all(
    target_arch = "arm",
    target_feature = "v7",
    target_feature = "thumb-mode",
    target_os = "none"
))]
#[naked]
pub unsafe extern "C" fn svc_handler_arm_v7m() {
    use core::arch::asm;
    asm!(
        "
    // First check to see which direction we are going in. If the link register
    // is something other than 0xFFFFFFF9, then we are coming from an app which
    // has called a syscall.
    cmp lr, #0xFFFFFFF9               // LR ≟ 0xFFFFFFF9
    bne 100f // to_kernel             // if LR != 0xFFFFFFF9, jump to to_kernel

    // If we get here, then this is a context switch from the kernel to the
    // application. Use the CONTROL register to set the thread mode to
    // unprivileged to run the application.
    //
    // CONTROL[1]: Stack status
    //   0 = Default stack (MSP) is used
    //   1 = Alternate stack is used
    // CONTROL[0]: Mode
    //   0 = Privileged in thread mode
    //   1 = User state in thread mode
    mov r0, #1                        // r0 = 1
    msr CONTROL, r0                   // CONTROL = 1
    // CONTROL writes must be followed by an Instruction Synchronization Barrier
    // (ISB). https://developer.arm.com/documentation/dai0321/latest
    isb

    // Set the link register to the special EXC_RETURN value of 0xFFFFFFFD which
    // instructs the CPU to run in thread mode with the process stack.
    ldr lr, =0xFFFFFFFD               // LR = 0xFFFFFFFD

    // Switch to the app.
    bx lr

  100: // to_kernel
    // An application called a syscall. We mark this in the global variable
    // `SYSCALL_FIRED` which is stored in the syscall file.
    // `UserspaceKernelBoundary` will use this variable to decide why the app
    // stopped executing.
    ldr r0, =SYSCALL_FIRED            // r0 = &SYSCALL_FIRED
    mov r1, #1                        // r1 = 1
    str r1, [r0]                      // *SYSCALL_FIRED = 1

    // Use the CONTROL register to set the thread mode to privileged to switch
    // back to kernel mode.
    //
    // CONTROL[1]: Stack status
    //   0 = Default stack (MSP) is used
    //   1 = Alternate stack is used
    // CONTROL[0]: Mode
    //   0 = Privileged in thread mode
    //   1 = User state in thread mode
    mov r0, #0                        // r0 = 0
    msr CONTROL, r0                   // CONTROL = 0
    // CONTROL writes must be followed by an Instruction Synchronization Barrier
    // (ISB). https://developer.arm.com/documentation/dai0321/latest
    isb

    // Set the link register to the special EXC_RETURN value of 0xFFFFFFF9 which
    // instructs the CPU to run in thread mode with the main (kernel) stack.
    ldr lr, =0xFFFFFFF9               // LR = 0xFFFFFFF9

    // Return to the kernel.
    bx lr
    ",
        options(noreturn)
    );
}

/// Generic interrupt handler for ARMv7-M instruction sets.
///
/// For documentation of this function, see [`CortexMVariant::GENERIC_ISR`].
#[cfg(all(
    target_arch = "arm",
    target_feature = "v7",
    target_feature = "thumb-mode",
    target_os = "none"
))]
#[naked]
pub unsafe extern "C" fn generic_isr_arm_v7m() {
    use core::arch::asm;
    asm!(
        "
    // Use the CONTROL register to set the thread mode to privileged to ensure
    // we are executing as the kernel. This may be redundant if the interrupt
    // happened while the kernel code was executing.
    //
    // CONTROL[1]: Stack status
    //   0 = Default stack (MSP) is used
    //   1 = Alternate stack is used
    // CONTROL[0]: Mode
    //   0 = Privileged in thread mode
    //   1 = User state in thread mode
    mov r0, #0                        // r0 = 0
    msr CONTROL, r0                   // CONTROL = 0
    // CONTROL writes must be followed by an Instruction Synchronization Barrier
    // (ISB). https://developer.arm.com/documentation/dai0321/latest
    isb

    // Set the link register to the special EXC_RETURN value of 0xFFFFFFF9 which
    // instructs the CPU to run in thread mode with the main (kernel) stack.
    ldr lr, =0xFFFFFFF9               // LR = 0xFFFFFFF9

    // Now need to disable the interrupt that fired in the NVIC to ensure it
    // does not trigger again before the scheduler has a chance to handle it. We
    // do this here in assembly for performance.
    //
    // The general idea is:
    // 1. Get the index of the interrupt that occurred.
    // 2. Set the disable bit for that interrupt in the NVIC.

    // Find the ISR number (`index`) by looking at the low byte of the IPSR
    // registers.
    mrs r0, IPSR                      // r0 = Interrupt Program Status Register (IPSR)
    and r0, #0xff                     // r0 = r0 & 0xFF; Get lowest 8 bits
    sub r0, #16                       // r0 = r0 - 16;   ISRs start at 16, so subtract 16 to get zero-indexed.

    // Now disable that interrupt in the NVIC.
    // High level:
    //    r0 = index
    //    NVIC.ICER[r0 / 32] = 1 << (r0 & 31)
    lsrs r2, r0, #5                   // r2 = r0 / 32
    // r0 = 1 << (r0 & 31)
    movs r3, #1                       // r3 = 1
    and r0, r0, #31                   // r0 = r0 & 31
    lsl r0, r3, r0                    // r0 = r3 << r0

    // Load the ICER register address.
    ldr r3, =0xe000e180               // r3 = &NVIC.ICER

    // Here:
    // - `r2` is index / 32
    // - `r3` is &NVIC.ICER
    // - `r0` is 1 << (index & 31)
    str r0, [r3, r2, lsl #2]          // *(r3 + r2 * 4) = r0

    // The pending bit in ISPR might be reset by hardware for pulse interrupts
    // at this point. So set it here again so the interrupt does not get lost in
    // `service_pending_interrupts()`.
    ldr r3, =0xe000e200               // r3 = &NVIC.ISPR
    str r0, [r3, r2, lsl #2]          // *(r3 + r2 * 4) = r0

    // Now we can return from the interrupt context and resume what we were
    // doing. If an app was executing we will switch to the kernel so it can
    // choose whether to service the interrupt.
    bx lr
    ",
        options(noreturn)
    );
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub unsafe extern "C" fn unhandled_interrupt() {
    use core::arch::asm;
    let mut interrupt_number: u32;

    // IPSR[8:0] holds the currently active interrupt
    asm!(
        "mrs r0, ipsr",
        out("r0") interrupt_number,
        options(nomem, nostack, preserves_flags)
    );

    interrupt_number = interrupt_number & 0x1ff;

    panic!("Unhandled Interrupt. ISR {} is active.", interrupt_number);
}

/// Assembly function to initialize the .bss and .data sections in RAM.
///
/// We need to (unfortunately) do these operations in assembly because it is
/// not valid to run Rust code without RAM initialized.
///
/// See https://github.com/tock/tock/issues/2222 for more information.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn initialize_ram_jump_to_main() {
    use core::arch::asm;
    asm!(
        "
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
        options(noreturn)
    );
}

/// Assembly function to switch into userspace and store/restore application
/// state.
///
/// For documentation of this function, please see
/// [`CortexMVariant::switch_to_user`].
#[cfg(all(
    target_arch = "arm",
    target_feature = "v7",
    target_feature = "thumb-mode",
    target_os = "none"
))]
pub unsafe fn switch_to_user_arm_v7m(
    mut user_stack: *const usize,
    process_regs: &mut [usize; 8],
) -> *const usize {
    use core::arch::asm;
    asm!(
    "
    // Rust `asm!()` macro (as of May 2021) will not let us mark r6, r7 and r9
    // as clobbers. r6 and r9 is used internally by LLVM, and r7 is used for
    // the frame pointer. However, in the process of restoring and saving the
    // process's registers, we do in fact clobber r6, r7 and r9. So, we work
    // around this by doing our own manual saving of r6 using r2, r7 using r3,
    // r9 using r12, and then mark those as clobbered.
    mov r2, r6                        // r2 = r6
    mov r3, r7                        // r3 = r7
    mov r12, r9                       // r12 = r8

    // The arguments passed in are:
    // - `r0` is the bottom of the user stack
    // - `r1` is a reference to `CortexMStoredState.regs`

    // Load bottom of stack into Process Stack Pointer.
    msr psp, r0                       // PSP = r0

    // Load non-hardware-stacked registers from the process stored state. Ensure
    // that the address register (right now r1) is stored in a callee saved
    // register.
    ldmia r1, {{r4-r11}}              // r4 = r1[0], r5 = r1[1], ...

    // Generate a SVC exception to handle the context switch from kernel to
    // userspace. It doesn't matter which SVC number we use here as it is not
    // used in the exception handler. Data being returned from a syscall is
    // transferred on the app's stack.
    svc 0xff

    // When execution returns here we have switched back to the kernel from the
    // application.

    // Push non-hardware-stacked registers into the saved state for the
    // application.
    stmia r1, {{r4-r11}}              // r1[0] = r4, r1[1] = r5, ...

    // Update the user stack pointer with the current value after the
    // application has executed.
    mrs r0, PSP                       // r0 = PSP

    // Need to restore r6, r7 and r12 since we clobbered them when switching to
    // and from the app.
    mov r6, r2                        // r6 = r2
    mov r7, r3                        // r7 = r3
    mov r9, r12                       // r9 = r12
    ",
    inout("r0") user_stack,
    in("r1") process_regs,
    out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r8") _, out("r10") _,
    out("r11") _, out("r12") _);

    user_stack
}

#[cfg(all(
    target_arch = "arm",
    target_feature = "v7",
    target_feature = "thumb-mode",
    target_os = "none"
))]
#[inline(never)]
unsafe fn kernel_hardfault_arm_v7m(faulting_stack: *mut u32) -> ! {
    let stacked_r0: u32 = *faulting_stack.offset(0);
    let stacked_r1: u32 = *faulting_stack.offset(1);
    let stacked_r2: u32 = *faulting_stack.offset(2);
    let stacked_r3: u32 = *faulting_stack.offset(3);
    let stacked_r12: u32 = *faulting_stack.offset(4);
    let stacked_lr: u32 = *faulting_stack.offset(5);
    let stacked_pc: u32 = *faulting_stack.offset(6);
    let stacked_xpsr: u32 = *faulting_stack.offset(7);

    let mode_str = "Kernel";

    let shcsr: u32 = core::ptr::read_volatile(0xE000ED24 as *const u32);
    let cfsr: u32 = core::ptr::read_volatile(0xE000ED28 as *const u32);
    let hfsr: u32 = core::ptr::read_volatile(0xE000ED2C as *const u32);
    let mmfar: u32 = core::ptr::read_volatile(0xE000ED34 as *const u32);
    let bfar: u32 = core::ptr::read_volatile(0xE000ED38 as *const u32);

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
    let divbysero = ((cfsr >> 16) & 0x200) == 0x200;

    let vecttbl = (hfsr & 0x02) == 0x02;
    let forced = (hfsr & 0x40000000) == 0x40000000;

    let ici_it = (((stacked_xpsr >> 25) & 0x3) << 6) | ((stacked_xpsr >> 10) & 0x3f);
    let thumb_bit = ((stacked_xpsr >> 24) & 0x1) == 1;
    let exception_number = (stacked_xpsr & 0x1ff) as usize;

    panic!(
        "{} HardFault.\r\n\
         \tKernel version {}\r\n\
         \tr0  0x{:x}\r\n\
         \tr1  0x{:x}\r\n\
         \tr2  0x{:x}\r\n\
         \tr3  0x{:x}\r\n\
         \tr12 0x{:x}\r\n\
         \tlr  0x{:x}\r\n\
         \tpc  0x{:x}\r\n\
         \tprs 0x{:x} [ N {} Z {} C {} V {} Q {} GE {}{}{}{} ; ICI.IT {} T {} ; Exc {}-{} ]\r\n\
         \tsp  0x{:x}\r\n\
         \ttop of stack     0x{:x}\r\n\
         \tbottom of stack  0x{:x}\r\n\
         \tSHCSR 0x{:x}\r\n\
         \tCFSR  0x{:x}\r\n\
         \tHSFR  0x{:x}\r\n\
         \tInstruction Access Violation:       {}\r\n\
         \tData Access Violation:              {}\r\n\
         \tMemory Management Unstacking Fault: {}\r\n\
         \tMemory Management Stacking Fault:   {}\r\n\
         \tMemory Management Lazy FP Fault:    {}\r\n\
         \tInstruction Bus Error:              {}\r\n\
         \tPrecise Data Bus Error:             {}\r\n\
         \tImprecise Data Bus Error:           {}\r\n\
         \tBus Unstacking Fault:               {}\r\n\
         \tBus Stacking Fault:                 {}\r\n\
         \tBus Lazy FP Fault:                  {}\r\n\
         \tUndefined Instruction Usage Fault:  {}\r\n\
         \tInvalid State Usage Fault:          {}\r\n\
         \tInvalid PC Load Usage Fault:        {}\r\n\
         \tNo Coprocessor Usage Fault:         {}\r\n\
         \tUnaligned Access Usage Fault:       {}\r\n\
         \tDivide By Zero:                     {}\r\n\
         \tBus Fault on Vector Table Read:     {}\r\n\
         \tForced Hard Fault:                  {}\r\n\
         \tFaulting Memory Address: (valid: {}) {:#010X}\r\n\
         \tBus Fault Address:       (valid: {}) {:#010X}\r\n\
         ",
        mode_str,
        option_env!("TOCK_KERNEL_VERSION").unwrap_or("unknown"),
        stacked_r0,
        stacked_r1,
        stacked_r2,
        stacked_r3,
        stacked_r12,
        stacked_lr,
        stacked_pc,
        stacked_xpsr,
        (stacked_xpsr >> 31) & 0x1,
        (stacked_xpsr >> 30) & 0x1,
        (stacked_xpsr >> 29) & 0x1,
        (stacked_xpsr >> 28) & 0x1,
        (stacked_xpsr >> 27) & 0x1,
        (stacked_xpsr >> 19) & 0x1,
        (stacked_xpsr >> 18) & 0x1,
        (stacked_xpsr >> 17) & 0x1,
        (stacked_xpsr >> 16) & 0x1,
        ici_it,
        thumb_bit,
        exception_number,
        ipsr_isr_number_to_str(exception_number),
        faulting_stack as u32,
        (&_estack as *const u32) as u32,
        (&_sstack as *const u32) as u32,
        shcsr,
        cfsr,
        hfsr,
        iaccviol,
        daccviol,
        munstkerr,
        mstkerr,
        mlsperr,
        ibuserr,
        preciserr,
        impreciserr,
        unstkerr,
        stkerr,
        lsperr,
        undefinstr,
        invstate,
        invpc,
        nocp,
        unaligned,
        divbysero,
        vecttbl,
        forced,
        mmfarvalid,
        mmfar,
        bfarvalid,
        bfar
    );
}

#[cfg(all(
    target_arch = "arm",
    target_feature = "v7",
    target_feature = "thumb-mode",
    target_os = "none"
))]
/// Continue the hardfault handler. This function is not `#[naked]`, meaning we can mix
/// `asm!()` and Rust. We separate this logic to not have to write the entire fault
/// handler entirely in assembly.
unsafe extern "C" fn hard_fault_handler_arm_v7m_continued(
    faulting_stack: *mut u32,
    kernel_stack: u32,
    stack_overflow: u32,
) {
    use core::arch::asm;
    if kernel_stack != 0 {
        if stack_overflow != 0 {
            // Panic to show the correct error.
            panic!("kernel stack overflow");
        } else {
            // Show the normal kernel hardfault message.
            kernel_hardfault_arm_v7m(faulting_stack);
        }
    } else {
        // Hard fault occurred in an app, not the kernel. The app should be
        // marked as in an error state and handled by the kernel.
        asm!(
            "
        /* Read the relevant SCB registers. */
        ldr r0, =SCB_REGISTERS  /* Global variable address */
        ldr r1, =0xE000ED14     /* SCB CCR register address */
        ldr r2, [r1, #0]        /* CCR */
        str r2, [r0, #0]
        ldr r2, [r1, #20]       /* CFSR */
        str r2, [r0, #4]
        ldr r2, [r1, #24]       /* HFSR */
        str r2, [r0, #8]
        ldr r2, [r1, #32]       /* MMFAR */
        str r2, [r0, #12]
        ldr r2, [r1, #36]       /* BFAR */
        str r2, [r0, #16]

        ldr r0, =APP_HARD_FAULT /* Global variable address */
        mov r1, #1              /* r1 = 1 */
        str r1, [r0, #0]        /* APP_HARD_FAULT = 1 */

        /* Set thread mode to privileged */
        mov r0, #0
        msr CONTROL, r0
        /* CONTROL writes must be followed by ISB */
        /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
        isb

        movw LR, #0xFFF9
        movt LR, #0xFFFF",
            out("r1") _,
            out("r0") _,
            out("r2") _,
            options(nostack),
        );
    }
}

/// ARMv7-M hardfault handler.
///
/// For documentation of this function, please see
/// [`CortexMVariant::HARD_FAULT_HANDLER_HANDLER`].
#[cfg(all(
    target_arch = "arm",
    target_feature = "v7",
    target_feature = "thumb-mode",
    target_os = "none"
))]
#[naked]
pub unsafe extern "C" fn hard_fault_handler_arm_v7m() {
    use core::arch::asm;
    // First need to determine if this a kernel fault or a userspace fault, and store
    // the unmodified stack pointer. Place these values in registers, then call
    // a non-naked function, to allow for use of rust code alongside inline asm.
    // Because calling a function increases the stack pointer, we have to check for a kernel
    // stack overflow and adjust the stack pointer before we branch

    asm!(
        "mov    r1, 0     /* r1 = 0 */",
        "tst    lr, #4    /* bitwise AND link register to 0b100 */",
        "itte   eq        /* if lr==4, run next two instructions, else, run 3rd instruction. */",
        "mrseq  r0, msp   /* r0 = kernel stack pointer */",
        "addeq  r1, 1     /* r1 = 1, kernel was executing */",
        "mrsne  r0, psp   /* r0 = userland stack pointer */",
        // Need to determine if we had a stack overflow before we push anything
        // on to the stack. We check this by looking at the BusFault Status
        // Register's (BFSR) `LSPERR` and `STKERR` bits to see if the hardware
        // had any trouble stacking important registers to the stack during the
        // fault. If so, then we cannot use this stack while handling this fault
        // or we will trigger another fault.
        "ldr   r3, =0xE000ED29  /* SCB BFSR register address */",
        "ldrb  r3, [r3]         /* r3 = BFSR */",
        "tst   r3, #0x30        /* r3 = BFSR & 0b00110000; LSPERR & STKERR bits */",
        "ite   ne               /* check if the result of that bitwise AND was not 0 */",
        "movne r2, #1           /* BFSR & 0b00110000 != 0; r2 = 1 */",
        "moveq r2, #0           /* BFSR & 0b00110000 == 0; r2 = 0 */",
        "and r5, r1, r2         /* bitwise and r2 and r1, store in r5 */ ",
        "cmp  r5, #1            /*  update condition codes to reflect if r2 == 1 && r1 == 1 */",
        "itt  eq                /* if r5==1 run the next 2 instructions, else skip to branch */",
        // if true, The hardware couldn't use the stack, so we have no saved data and
        // we cannot use the kernel stack as is. We just want to report that
        // the kernel's stack overflowed, since that is essential for
        // debugging.
        //
        // To make room for a panic!() handler stack, we just re-use the
        // kernel's original stack. This should in theory leave the bottom
        // of the stack where the problem occurred untouched should one want
        // to further debug.
        "ldreq  r4, ={}       /* load _estack into r4 */",
        "moveq  sp, r4        /* Set the stack pointer to _estack */",
        // finally, branch to non-naked handler
        // per ARM calling convention, faulting stack is passed in r0, kernel_stack in r1,
        // and whether there was a stack overflow in r2
        "b {}", // branch to function
        "bx lr", // if continued function returns, we need to manually branch to link register
        sym _estack, sym hard_fault_handler_arm_v7m_continued,
        options(noreturn), // asm block never returns, so no need to mark clobbers
    );
}

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

// Table 2.5
// http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/CHDBIBGJ.html
pub fn ipsr_isr_number_to_str(isr_number: usize) -> &'static str {
    match isr_number {
        0 => "Thread Mode",
        1 => "Reserved",
        2 => "NMI",
        3 => "HardFault",
        4 => "MemManage",
        5 => "BusFault",
        6 => "UsageFault",
        7..=10 => "Reserved",
        11 => "SVCall",
        12 => "Reserved for Debug",
        13 => "Reserved",
        14 => "PendSV",
        15 => "SysTick",
        16..=255 => "IRQn",
        _ => "(Unknown! Illegal value?)",
    }
}

///////////////////////////////////////////////////////////////////
// Mock implementations for running tests on CI.
//
// Since tests run on the local architecture, we have to remove any
// ARM assembly since it will not compile.
///////////////////////////////////////////////////////////////////

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn systick_handler_arm_v7m() {
    unimplemented!()
}

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn svc_handler_arm_v7m() {
    unimplemented!()
}

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn generic_isr_arm_v7m() {
    unimplemented!()
}

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn unhandled_interrupt() {
    unimplemented!()
}

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn initialize_ram_jump_to_main() {
    unimplemented!()
}

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn switch_to_user_arm_v7m(
    _user_stack: *const u8,
    _process_regs: &mut [usize; 8],
) -> *const usize {
    unimplemented!()
}

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn hard_fault_handler_arm_v7m() {
    unimplemented!()
}
