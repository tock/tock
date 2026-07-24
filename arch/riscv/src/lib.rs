// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Shared support for RISC-V architectures.

#![no_std]

use core::fmt::Write;

use kernel::utilities::registers::interfaces::{Readable, Writeable};

pub mod clic;
pub mod csr;
pub mod dma_fence;
pub mod pmp;
pub mod pseudo_instructions;
pub mod support;
pub mod syscall;
pub mod thread_id;

/// `XLEN` is the width of an integer register in bits (either 32 or 64).
pub const XLEN: usize = 1 << XLEN_LOG2;

/// `XLEN_LOG2` is the log base 2 of XLEN.
#[cfg(target_arch = "riscv32")]
pub const XLEN_LOG2: usize = 5;
#[cfg(target_arch = "riscv64")]
pub const XLEN_LOG2: usize = 6;
// Default to 32 bit if no architecture is specified of if this is being
// compiled for docs or testing on a different architecture.
#[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
pub const XLEN_LOG2: usize = 5;

extern "C" {
    // Where the end of the stack region is (and hence where the stack should
    // start), and the start of the stack region.
    static _estack: usize;
    static _sstack: usize;

    // Boundaries of the .bss section.
    static mut _szero: usize;
    static mut _ezero: usize;

    // Where the .data section is stored in flash.
    static mut _etext: usize;

    // Boundaries of the .data section.
    static mut _srelocate: usize;
    static mut _erelocate: usize;

    // The global pointer, value set in the linker script
    #[link_name = "__global_pointer$"]
    static __global_pointer: usize;
}

/// Entry point of all programs
///
/// This assembly does three functions:
///
/// 1. It initializes the stack pointer, the frame pointer (needed for closures
///    to work in start_rust) and the global pointer.
/// 2. It initializes the .bss and .data RAM segments. This must be done before
///    any Rust code runs. See <https://github.com/tock/tock/issues/2222> for more
///    information.
/// 3. Finally it calls `main()`, the main entry point for Tock boards.
#[cfg(any(doc, all(target_arch = "riscv32", target_os = "none")))]
#[link_section = ".riscv.start"]
#[unsafe(naked)]
// We don't want the function name symbol to be mangled in order to be able to refer to
// it the linker script. It is not currently being used in the provided linker script
// for the supported boards, however it might be needed in the future or for downstream developers
// providing their own linker scripts, where this symbol might be used to point to the entry point
// (i.e. ENTRY(_start), or verify the placement via asserts or perform memory layout calculations).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn initialize_ram_jump_to_main() {
    use core::arch::naked_asm;
    naked_asm!(
        "
    // Set the global pointer register using the variable defined in the
    // linker script. This register is only set once. The global pointer
    // is a method for sharing state between the linker and the CPU so
    // that the linker can emit code with offsets that are relative to
    // the gp register, and the CPU can successfully execute them.
    //
    // https://gnu-mcu-eclipse.github.io/arch/riscv/programmer/#the-gp-global-pointer-register
    // https://groups.google.com/a/groups.riscv.org/forum/#!msg/sw-dev/60IdaZj27dY/5MydPLnHAQAJ
    // https://www.sifive.com/blog/2017/08/28/all-aboard-part-3-linker-relaxation-in-riscv-toolchain/
    //
    // Disable linker relaxation for code that sets up GP so that this doesn't
    // get turned into `mv gp, gp`.
    .option push
    .option norelax

    la gp, {gp}                 // Set the global pointer from linker script.

    // Re-enable linker relaxations.
    .option pop

    // Initialize the stack pointer register. This comes directly from
    // the linker script.
    la sp, {estack}             // Set the initial stack pointer.

    // Set s0 (the frame pointer) to the start of the stack.
    add  s0, sp, zero           // s0 = sp

    // Initialize mscratch to 0 so that we know that we are currently
    // in the kernel. This is used for the check in the trap handler.
    csrw 0x340, zero            // CSR=0x340=mscratch

    // INITIALIZE MEMORY

    // Start by initializing .bss memory. The Tock linker script defines
    // `_szero` and `_ezero` to mark the .bss segment.
    la a0, {sbss}               // a0 = first address of .bss
    la a1, {ebss}               // a1 = first address after .bss

100: // bss_init_loop
    beq  a0, a1, 101f           // If a0 == a1, we are done.
    sw   zero, 0(a0)            // *a0 = 0. Write 0 to the memory location in a0.
    addi a0, a0, 4              // a0 = a0 + 4. Increment pointer to next word.
    j 100b                      // Continue the loop.

101: // bss_init_done

    // Now initialize .data memory. This involves coping the values right at the
    // end of the .text section (in flash) into the .data section (in RAM).
    la a0, {sdata}              // a0 = first address of data section in RAM
    la a1, {edata}              // a1 = first address after data section in RAM
    la a2, {etext}              // a2 = address of stored data initial values

200: // data_init_loop
    beq  a0, a1, 201f           // If we have reached the end of the .data
                                // section then we are done.
    lw   a3, 0(a2)              // a3 = *a2. Load value from initial values into a3.
    sw   a3, 0(a0)              // *a0 = a3. Store initial value into
                                // next place in .data.
    addi a0, a0, 4              // a0 = a0 + 4. Increment to next word in memory.
    addi a2, a2, 4              // a2 = a2 + 4. Increment to next word in flash.
    j 200b                      // Continue the loop.

201: // data_init_done

    // With that initial setup out of the way, we now branch to the main
    // code, likely defined in a board's main.rs.
    j main
        ",
        gp = sym __global_pointer,
        estack = sym _estack,
        sbss = sym _szero,
        ebss = sym _ezero,
        sdata = sym _srelocate,
        edata = sym _erelocate,
        etext = sym _etext,
    );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(doc, all(target_arch = "riscv32", target_os = "none"))))]
pub unsafe extern "C" fn initialize_ram_jump_to_main() {
    unimplemented!()
}

/// The various privilege levels in RISC-V.
pub enum PermissionMode {
    User = 0x0,
    Supervisor = 0x1,
    Reserved = 0x2,
    Machine = 0x3,
}

/// Tell the MCU what address the trap handler is located at, and initialize
/// `mscratch` to zero, indicating kernel execution.
///
/// This is a generic implementation. There may be board specific versions as
/// some platforms have added more bits to the `mtvec` register.
///
/// The trap handler is called on exceptions and for interrupts.
pub unsafe fn configure_trap_handler() {
    // Indicate to the trap handler that we are executing kernel code.
    csr::CSR.mscratch.set(0);

    // Set the machine-mode trap handler. By not configuring an S-mode or U-mode
    // trap handler, this should ensure that all traps are handled by the M-mode
    // handler.
    csr::CSR.mtvec.write(
        csr::mtvec::mtvec::trap_addr.val(_start_trap as extern "C" fn() -> ! as usize >> 2)
            + csr::mtvec::mtvec::mode::CLEAR,
    );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(
    doc,
    all(target_arch = "riscv32", target_os = "none"),
    all(target_arch = "riscv64", target_os = "none")
)))]
pub extern "C" fn _start_trap() -> ! {
    unimplemented!()
}

/// This is the trap handler function. This code is called on all traps,
/// including interrupts, exceptions, and system calls from applications.
///
/// Tock uses only the single trap handler, and does not use any vectored
/// interrupts or other exception handling. The trap handler has to
/// determine why the trap handler was called, and respond
/// accordingly. Generally, there are two reasons the trap handler gets
/// called: an interrupt occurred or an application called a syscall.
///
/// In the case of an interrupt while the kernel was executing we only need
/// to save the kernel registers and then run whatever interrupt handling
/// code we need to. If the trap happens while an application was executing,
/// we have to save the application state and then resume the `switch_to()`
/// function to correctly return back to the kernel.
///
/// We implement this distinction through a branch on the value of the
/// `mscratch` CSR. If, at the time the trap was taken, it contains `0`, we
/// assume that the hart is currently executing kernel code.
///
/// If it contains any other value, we interpret it to be a memory address
/// pointing to a particular data structure (`custom_trap_handler`):
///
/// ```text
/// custom_trap_handler: [usize; 2];
///
/// mscratch = &custom_trap_handler;
/// ```
///
/// or in memory:
///
/// ```text
/// mscratch
///  \->|--------------------------------------------------------------|
///     | scratch word, overwritten with s1 register contents          |
///     |--------------------------------------------------------------|
///     | trap handler address, continue trap handler execution here   |
///     |--------------------------------------------------------------|
/// ```
///
/// Thus, process implementations can define their own strategy for how
/// traps should be handled when they occur during process execution. This
/// global trap handler behavior is well defined. It will:
///
/// 1. atomically swap s0 and the mscratch CSR,
///
/// 2. execute the default kernel trap handler if s0 now contains `0` (meaning
///    that the mscratch CSR contained `0` before entering this trap handler),
///
/// 3. otherwise, save s1 to `0*(XLEN/8)(s0)`, and finally
///
/// 4. load the address at `1*(XLEN/8)(s0)` into s1, and jump to it.
///
/// No registers other than s0, s1 and the mscratch CSR are to be clobbered
/// before continuing execution at the address loaded into the mscratch CSR
/// or the _start_kernel_trap kernel trap handler.  Execution with these
/// second-stage trap handlers must continue in the same trap handler
/// context as originally invoked by the trap (e.g., the global trap handler
/// will not execute an mret instruction). It will not modify CSRs that
/// contain information on the trap source or the system state prior to
/// entering the trap handler.
///
/// Before a trap handler executes any Rust code, or code that can
/// transititively call any Rust code, it must indicate that a trap handler is
/// currently active by writing an MXLEN-sized (usize), non-zero value to the
/// address at `_trap_handler_active + (MXLEN / 8) * mhartid`. Before leaving
/// the trap handler using `mret`, but after any Rust code has run, it must
/// reset the value at this address back to zero. This is used to accurately
/// determine the running thread ID, taking trap handlers into account as a
/// separate thread.
///
/// We deliberately clobber callee-saved instead of caller-saved registers,
/// as this makes it easier to call other functions as part of the trap
/// handler (for example to to disable interrupts from within Rust
/// code). This global trap handler saves the previous values of these
/// clobbered registers ensuring that they can be restored later.  It places
/// new values into these clobbered registers (such as the previous `s0`
/// register contents) that are required to be retained for correctly
/// returning from the trap handler, and as such need to be saved across
/// C-ABI function calls. Loading them into saved registers avoids the need
/// to manually save them across such calls.
///
/// When a custom trap handler stack is registered in `mscratch`, the custom
/// handler is responsible for restoring the kernel trap handler (by setting
/// mscratch=0) before returning to kernel execution from the trap handler
/// context.
///
/// If a board or chip must, for whichever reason, use a different global
/// trap handler, it should abide to the above contract and emulate its
/// behavior for all traps and interrupts that are required to be handled by
/// the respective kernel or other trap handler as registered in mscratch.
///
/// For instance, a chip that does not support non-vectored trap handlers
/// can register a vectored trap handler that routes each trap source to
/// this global trap handler.
///
/// Alternatively, a board can be allowed to ignore certain traps or
/// interrupts, some or all of the time, provided they are not vital to
/// Tock's execution. These boards may choose to register an alternative
/// handler for some or all trap sources. When this alternative handler is
/// invoked, it may, for instance, choose to ignore a certain trap, access
/// global state (subject to synchronization), etc. It must still abide to
/// the contract as stated above.
#[cfg(any(
    doc,
    all(target_arch = "riscv32", target_os = "none"),
    all(target_arch = "riscv64", target_os = "none")
))]
#[link_section = ".riscv.trap"]
// We need the `_start_trap` function to be 256 byte aligned. The linker script
// includes a check for whether a symbol named `_start_trap` exists. If it does,
// it makes sure to align the `.riscv.trap` section on a 256 byte
// boundary. Thus, ensure that this function is exported under this stable
// symbol name.
#[export_name = "_start_trap"]
#[unsafe(naked)]
pub extern "C" fn _start_trap() -> ! {
    use core::arch::naked_asm;
    naked_asm!(
        xlen_macros!(),
        "
    // This is the global trap handler. By default, Tock expects this
    // trap handler to be registered at all times, and that all traps
    // and interrupts occurring in all modes of execution (M-, S-, and
    // U-mode) will cause this trap handler to be executed.
    //
    // For documentation of its behavior, and how process
    // implementations can hook their own trap handler code, see the
    // comment on the `_start_trap` function.

    // Atomically swap s0 and mscratch. This puts `&custom_trap_handler`
    // in s0.
    csrrw s0, mscratch, s0        // s0 = mscratch; mscratch = s0

    // If mscratch contained 0, invoke the kernel trap handler.
    beq   s0, x0, 100f            // if s0==x0: goto 100

    // Else, save the current value of s1 to `custom_trap_handler[0]`,
    // load `custom_trap_handler[1]` into s1 and jump to it (invoking
    // a custom trap handler).
    sx    s1, 0*({XLEN}/8)(s0)    // custom_trap_handler[0] = s1
    lx    s1, 1*({XLEN}/8)(s0)    // s1 = custom_trap_handler[1]
    jr    s1                      // goto s1

  100: // _start_kernel_trap

    // The global trap handler has swapped s0 into mscratch. We can thus
    // freely clobber s0 without losing any information.
    //
    // Since we want to use the stack to save kernel registers, we
    // first need to make sure that the trap wasn't the result of a
    // stack overflow, in which case we can't use the current stack
    // pointer. Use s0 as a scratch register:

    // Load the address of the bottom of the stack (`_sstack`) into our
    // newly freed-up s0 register.
    la s0, {sstack}               // s0 = _sstack

    // Compare the kernel stack pointer to the bottom of the stack. If
    // the stack pointer is above the bottom of the stack, then continue
    // handling the fault as normal.
    bgtu sp, s0, 200f             // branch if sp > s0

    // If we get here, then we did encounter a stack overflow. We are
    // going to panic at this point, but for that to work we need a
    // valid stack to run the panic code. We do this by just starting
    // over with the kernel stack and placing the stack pointer at the
    // top of the original stack.
    la sp, {estack}               // sp = _estack

200: // _start_kernel_trap_continue

    // Restore s0. We reset mscratch to 0 (kernel trap handler mode)
    csrrw s0, mscratch, zero      // s0 = mscratch; mscratch = 0

    // Make room for the caller saved registers we need to restore after running
    // any trap handler code.
    addi sp, sp, -20*({XLEN}/8)   // riscv32: sp = sp - (20*4), riscv64: sp = sp - (20*8)

    // Save all of the caller saved registers.
    sx    ra,  0*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (0*4)) = ra, riscv64: *(stackptr +  (0*8)) = ra
    sx    t0,  1*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (1*4)) = t0, riscv64: *(stackptr +  (1*8)) = t0
    sx    t1,  2*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (2*4)) = t1, riscv64: *(stackptr +  (2*8)) = t1
    sx    t2,  3*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (3*4)) = t2, riscv64: *(stackptr +  (3*8)) = t2
    sx    t3,  4*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (4*4)) = t3, riscv64: *(stackptr +  (4*8)) = t3
    sx    t4,  5*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (5*4)) = t4, riscv64: *(stackptr +  (5*8)) = t4
    sx    t5,  6*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (6*4)) = t5, riscv64: *(stackptr +  (6*8)) = t5
    sx    t6,  7*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (7*4)) = t6, riscv64: *(stackptr +  (7*8)) = t6
    sx    a0,  8*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (8*4)) = a0, riscv64: *(stackptr +  (8*8)) = a0
    sx    a1,  9*({XLEN}/8)(sp)   // riscv32: *(stackptr +  (9*4)) = a1, riscv64: *(stackptr +  (9*8)) = a1
    sx    a2, 10*({XLEN}/8)(sp)   // riscv32: *(stackptr + (10*4)) = a2, riscv64: *(stackptr + (10*8)) = a2
    sx    a3, 11*({XLEN}/8)(sp)   // riscv32: *(stackptr + (11*4)) = a3, riscv64: *(stackptr + (11*8)) = a3
    sx    a4, 12*({XLEN}/8)(sp)   // riscv32: *(stackptr + (12*4)) = a4, riscv64: *(stackptr + (12*8)) = a4
    sx    a5, 13*({XLEN}/8)(sp)   // riscv32: *(stackptr + (13*4)) = a5, riscv64: *(stackptr + (13*8)) = a5
    sx    a6, 14*({XLEN}/8)(sp)   // riscv32: *(stackptr + (14*4)) = a6, riscv64: *(stackptr + (14*8)) = a6
    sx    a7, 15*({XLEN}/8)(sp)   // riscv32: *(stackptr + (15*4)) = a7, riscv64: *(stackptr + (15*8)) = a7

    // Save one callee-saved register (s0), which we place the address of
    // the hart-specific 'are we in a trap handler' flag in:
    sx    s0, 16*({XLEN}/8)(sp)   // riscv32: *(stackptr + (16*4)) = s0, riscv64: *(stackptr + (16*8)) = s0

    // Determine the address of the hart-specific 'are we in a trap handler'
    // flag as an offset to the _trap_handler_active symbol. The chip crate
    // is responsible for defining this symbol, and ensuring it is large
    // enough to fit `max(mhartid) * MXLEN` bytes.
    la   s0, _trap_handler_active // s0 = addr(_trap_handler_active)
    csrr t0, mhartid              // t0 = hartid
    slli t0, t0, ({XLEN_LOG2}-3)  // t0 = t0 * sizeof(usize)
    add  s0, s0, t0               // s0 = addr(_trap_handler_active[hartid])

    // Indicate that we are in a trap handler on this hart:
    li   t0, 1                    // t0 = 1
    sw   t0, 0(s0)                // _trap_handler_active[hartid] = 1

    // Jump to board-specific trap handler code. Likely this was an
    // interrupt and we want to disable a particular interrupt, but each
    // board/chip can customize this as needed.
    jal ra, _start_trap_rust_from_kernel

    // Indicate that we are no longer going to be in a trap handler on this
    // hart:
    sw   x0, 0(s0)                // _trap_handler_active[hartid] = 0

    // Restore the caller saved registers from the stack.
    lx    ra,  0*({XLEN}/8)(sp)   // riscv32: ra = *(stackptr +  (0*4)), riscv64: ra = *(stackptr +  (0*8))
    lx    t0,  1*({XLEN}/8)(sp)   // riscv32: t0 = *(stackptr +  (1*4)), riscv64: t0 = *(stackptr +  (1*8))
    lx    t1,  2*({XLEN}/8)(sp)   // riscv32: t1 = *(stackptr +  (2*4)), riscv64: t1 = *(stackptr +  (2*8))
    lx    t2,  3*({XLEN}/8)(sp)   // riscv32: t2 = *(stackptr +  (3*4)), riscv64: t2 = *(stackptr +  (3*8))
    lx    t3,  4*({XLEN}/8)(sp)   // riscv32: t3 = *(stackptr +  (4*4)), riscv64: t3 = *(stackptr +  (4*8))
    lx    t4,  5*({XLEN}/8)(sp)   // riscv32: t4 = *(stackptr +  (5*4)), riscv64: t4 = *(stackptr +  (5*8))
    lx    t5,  6*({XLEN}/8)(sp)   // riscv32: t5 = *(stackptr +  (6*4)), riscv64: t5 = *(stackptr +  (6*8))
    lx    t6,  7*({XLEN}/8)(sp)   // riscv32: t6 = *(stackptr +  (7*4)), riscv64: t6 = *(stackptr +  (7*8))
    lx    a0,  8*({XLEN}/8)(sp)   // riscv32: a0 = *(stackptr +  (8*4)), riscv64: a0 = *(stackptr +  (8*8))
    lx    a1,  9*({XLEN}/8)(sp)   // riscv32: a1 = *(stackptr +  (9*4)), riscv64: a1 = *(stackptr +  (9*8))
    lx    a2, 10*({XLEN}/8)(sp)   // riscv32: a2 = *(stackptr + (10*4)), riscv64: a2 = *(stackptr + (10*8))
    lx    a3, 11*({XLEN}/8)(sp)   // riscv32: a3 = *(stackptr + (11*4)), riscv64: a3 = *(stackptr + (11*8))
    lx    a4, 12*({XLEN}/8)(sp)   // riscv32: a4 = *(stackptr + (12*4)), riscv64: a4 = *(stackptr + (12*8))
    lx    a5, 13*({XLEN}/8)(sp)   // riscv32: a5 = *(stackptr + (13*4)), riscv64: a5 = *(stackptr + (13*8))
    lx    a6, 14*({XLEN}/8)(sp)   // riscv32: a6 = *(stackptr + (14*4)), riscv64: a6 = *(stackptr + (14*8))
    lx    a7, 15*({XLEN}/8)(sp)   // riscv32: a7 = *(stackptr + (15*4)), riscv64: a7 = *(stackptr + (15*8))

    // Restore the one callee-saved register (s0), which used to hold the
    // address of the hart-specific 'are we in a trap handler flag':
    lx    s0, 16*({XLEN}/8)(sp)   // riscv32: s0 = *(stackptr + (16*4)), riscv64: s0 = *(stackptr + (16*8))

    // Reset the stack pointer.
    addi sp, sp, 20*({XLEN}/8)    // riscv32: sp = sp + (20*4), riscv32: sp = sp + (20*8)

    // mret returns from the trap handler. The PC is set to what is in
    // mepc and execution proceeds from there. Since we did not modify
    // mepc we will return to where the exception occurred.
    mret
        ",
        estack = sym _estack,
        sstack = sym _sstack,
        XLEN = const XLEN,
        XLEN_LOG2 = const XLEN_LOG2,
    );
}

/// RISC-V semihosting needs three exact instructions in uncompressed form.
///
/// See <https://github.com/riscv/riscv-semihosting-spec/blob/main/riscv-semihosting-spec.adoc#11-semihosting-trap-instruction-sequence>
/// for more details on the three instructions.
///
/// In order to work with semihosting we include the assembly here
/// where we are able to disable compressed instruction support. This
/// follows the example used in the Linux kernel:
/// <https://elixir.bootlin.com/linux/v5.12.10/source/arch/riscv/include/asm/jump_label.h#L21>
/// as suggested by the RISC-V developers:
/// <https://groups.google.com/a/groups.riscv.org/g/isa-dev/c/XKkYacERM04/m/CdpOcqtRAgAJ>
#[cfg(any(doc, all(target_arch = "riscv32", target_os = "none")))]
pub unsafe fn semihost_command(command: usize, arg0: usize, arg1: usize) -> usize {
    use core::arch::asm;
    let res;
    asm!(
        "
    .balign 16                    // ensure 16 byte alignment
    .option push                  // enable the following options:
    .option norelax               // - norelax: do not replace these instructions
    .option norvc                 // - norvc: force full 32 bit instructions
    slli x0, x0, 0x1f             // useless instruction (writes to x0), but serves as sentinel for semihosting
    ebreak                        // trap to debugger
    srai x0, x0, 7                // useless instruction (writes to x0), but serves as second sentinel
    .option pop
        ",
        in("a0") command,         // a0 holds command (and return code)
        in("a1") arg0,            // a1 holds first argument
        in("a2") arg1,            // a2 holds second argument
        lateout("a0") res,        // semihosting replaces a0 with return code
    );
    res
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(doc, all(target_arch = "riscv32", target_os = "none"))))]
pub unsafe fn semihost_command(_command: usize, _arg0: usize, _arg1: usize) -> usize {
    unimplemented!()
}

/// Print a readable string for an mcause reason.
pub unsafe fn print_mcause(mcval: csr::mcause::Trap, writer: &mut dyn Write) {
    let s = match mcval {
        csr::mcause::Trap::Interrupt(interrupt) => match interrupt {
            csr::mcause::Interrupt::UserSoft => "User software interrupt",
            csr::mcause::Interrupt::SupervisorSoft => "Supervisor software interrupt",
            csr::mcause::Interrupt::MachineSoft => "Machine software interrupt",
            csr::mcause::Interrupt::UserTimer => "User timer interrupt",
            csr::mcause::Interrupt::SupervisorTimer => "Supervisor timer interrupt",
            csr::mcause::Interrupt::MachineTimer => "Machine timer interrupt",
            csr::mcause::Interrupt::UserExternal => "User external interrupt",
            csr::mcause::Interrupt::SupervisorExternal => "Supervisor external interrupt",
            csr::mcause::Interrupt::MachineExternal => "Machine external interrupt",
            csr::mcause::Interrupt::Unknown(_) => "Reserved/Unknown",
        },
        csr::mcause::Trap::Exception(exception) => match exception {
            csr::mcause::Exception::InstructionMisaligned => "Instruction access misaligned",
            csr::mcause::Exception::InstructionFault => "Instruction access fault",
            csr::mcause::Exception::IllegalInstruction => "Illegal instruction",
            csr::mcause::Exception::Breakpoint => "Breakpoint",
            csr::mcause::Exception::LoadMisaligned => "Load address misaligned",
            csr::mcause::Exception::LoadFault => "Load access fault",
            csr::mcause::Exception::StoreMisaligned => "Store/AMO address misaligned",
            csr::mcause::Exception::StoreFault => "Store/AMO access fault",
            csr::mcause::Exception::UserEnvCall => "Environment call from U-mode",
            csr::mcause::Exception::SupervisorEnvCall => "Environment call from S-mode",
            csr::mcause::Exception::MachineEnvCall => "Environment call from M-mode",
            csr::mcause::Exception::InstructionPageFault => "Instruction page fault",
            csr::mcause::Exception::LoadPageFault => "Load page fault",
            csr::mcause::Exception::StorePageFault => "Store/AMO page fault",
            csr::mcause::Exception::Unknown => "Reserved",
        },
    };
    let _ = writer.write_str(s);
}

/// Prints out RISCV machine state, including basic system registers
/// (mcause, mstatus, mtvec, mepc, mtval, interrupt status).
pub unsafe fn print_riscv_state(writer: &mut dyn Write) {
    let mcval: csr::mcause::Trap = core::convert::From::from(csr::CSR.mcause.extract());
    let _ = writer.write_fmt(format_args!("\r\n---| RISC-V Machine State |---\r\n"));
    let _ = writer.write_fmt(format_args!("Last cause (mcause): "));
    print_mcause(mcval, writer);
    let interrupt = csr::CSR.mcause.read(csr::mcause::mcause::is_interrupt);
    let code = csr::CSR.mcause.read(csr::mcause::mcause::reason);
    let _ = writer.write_fmt(format_args!(
        " (interrupt={}, exception code={:#0width$X})",
        interrupt,
        code,
        width = (XLEN / 4) + 2,
    ));
    let _ = writer.write_fmt(format_args!(
        "\r\nLast value (mtval):  {:#0width$X}\
         \r\n\
         \r\nSystem register dump:\
         \r\n mepc:    {:#0width$X}    mstatus:     {:#0width$X}\
         \r\n mcycle:  {:#0width$X}    minstret:    {:#0width$X}\
         \r\n mtvec:   {:#0width$X}",
        csr::CSR.mtval.get(),
        csr::CSR.mepc.get(),
        csr::CSR.mstatus.get(),
        csr::CSR.mcycle.get(),
        csr::CSR.minstret.get(),
        csr::CSR.mtvec.get(),
        width = (XLEN / 4) + 2,
    ));
    let mstatus = csr::CSR.mstatus.extract();
    let uie = mstatus.is_set(csr::mstatus::mstatus::uie);
    let sie = mstatus.is_set(csr::mstatus::mstatus::sie);
    let mie = mstatus.is_set(csr::mstatus::mstatus::mie);
    let upie = mstatus.is_set(csr::mstatus::mstatus::upie);
    let spie = mstatus.is_set(csr::mstatus::mstatus::spie);
    let mpie = mstatus.is_set(csr::mstatus::mstatus::mpie);
    let spp = mstatus.is_set(csr::mstatus::mstatus::spp);
    let _ = writer.write_fmt(format_args!(
        "\r\n mstatus: {:#0width$X}\
         \r\n  uie:    {:5}    upie:   {:5}\
         \r\n  sie:    {:5}    spie:   {:5}\
         \r\n  mie:    {:5}    mpie:   {:5}\
         \r\n  spp:    {:5}",
        mstatus.get(),
        uie,
        upie,
        sie,
        spie,
        mie,
        mpie,
        spp,
        width = (XLEN / 4) + 2,
    ));
    let e_usoft = csr::CSR.mie.is_set(csr::mie::mie::usoft);
    let e_ssoft = csr::CSR.mie.is_set(csr::mie::mie::ssoft);
    let e_msoft = csr::CSR.mie.is_set(csr::mie::mie::msoft);
    let e_utimer = csr::CSR.mie.is_set(csr::mie::mie::utimer);
    let e_stimer = csr::CSR.mie.is_set(csr::mie::mie::stimer);
    let e_mtimer = csr::CSR.mie.is_set(csr::mie::mie::mtimer);
    let e_uext = csr::CSR.mie.is_set(csr::mie::mie::uext);
    let e_sext = csr::CSR.mie.is_set(csr::mie::mie::sext);
    let e_mext = csr::CSR.mie.is_set(csr::mie::mie::mext);

    let p_usoft = csr::CSR.mip.is_set(csr::mip::mip::usoft);
    let p_ssoft = csr::CSR.mip.is_set(csr::mip::mip::ssoft);
    let p_msoft = csr::CSR.mip.is_set(csr::mip::mip::msoft);
    let p_utimer = csr::CSR.mip.is_set(csr::mip::mip::utimer);
    let p_stimer = csr::CSR.mip.is_set(csr::mip::mip::stimer);
    let p_mtimer = csr::CSR.mip.is_set(csr::mip::mip::mtimer);
    let p_uext = csr::CSR.mip.is_set(csr::mip::mip::uext);
    let p_sext = csr::CSR.mip.is_set(csr::mip::mip::sext);
    let p_mext = csr::CSR.mip.is_set(csr::mip::mip::mext);
    let _ = writer.write_fmt(format_args!(
        "\r\n mie:     {:#0width$X}    mip:         {:#0width$X}\
         \r\n  usoft:  {:5}        {space:>swidth$} {:5}\
         \r\n  ssoft:  {:5}        {space:>swidth$} {:5}\
         \r\n  msoft:  {:5}        {space:>swidth$} {:5}\
         \r\n  utimer: {:5}        {space:>swidth$} {:5}\
         \r\n  stimer: {:5}        {space:>swidth$} {:5}\
         \r\n  mtimer: {:5}        {space:>swidth$} {:5}\
         \r\n  uext:   {:5}        {space:>swidth$} {:5}\
         \r\n  sext:   {:5}        {space:>swidth$} {:5}\
         \r\n  mext:   {:5}        {space:>swidth$} {:5}\r\n",
        csr::CSR.mie.get(),
        csr::CSR.mip.get(),
        e_usoft,
        p_usoft,
        e_ssoft,
        p_ssoft,
        e_msoft,
        p_msoft,
        e_utimer,
        p_utimer,
        e_stimer,
        p_stimer,
        e_mtimer,
        p_mtimer,
        e_uext,
        p_uext,
        e_sext,
        p_sext,
        e_mext,
        p_mext,
        space = "",
        width = (XLEN / 4) + 2,
        swidth = (XLEN / 4) - 8 + 13,
    ));
}
