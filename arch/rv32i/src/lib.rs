//! Support for the 32-bit RISC-V architecture.

#![crate_name = "rv32i"]
#![crate_type = "rlib"]
#![feature(
    asm,
    const_fn,
    lang_items,
    global_asm,
    crate_visibility_modifier,
    naked_functions,
    in_band_lifetimes
)]
#![no_std]

pub mod clic;
pub mod csr;
pub mod machine_timer;
pub mod plic;
pub mod pmp;
pub mod support;
pub mod syscall;
extern crate tock_registers;

extern "C" {
    // Where the end of the stack region is (and hence where the stack should
    // start).
    static _estack: u32;

    // Boundaries of the .bss section.
    static mut _szero: u32;
    static mut _ezero: u32;

    // Where the .data section is stored in flash.
    static mut _etext: u32;

    // Boundaries of the .data section.
    static mut _srelocate: u32;
    static mut _erelocate: u32;
}

/// Entry point of all programs (`_start`).
///
/// It initializes the stack pointer, the frame pointer (needed for closures to
/// work in start_rust) and the global pointer. Then it calls `reset_handler()`,
/// the main entry point for Tock boards.
#[cfg(target_os = "none")]
#[link_section = ".riscv.start"]
#[export_name = "_start"]
#[naked]
pub extern "C" fn _start() {
    unsafe {
        asm! ("
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
            lui  gp, %hi(__global_pointer$$)     // Set the global pointer.
            addi gp, gp, %lo(__global_pointer$$) // Value set in linker script.

            // Initialize the stack pointer register. This comes directly from
            // the linker script.
            lui  sp, %hi(_estack)     // Set the initial stack pointer.
            addi sp, sp, %lo(_estack) // Value from the linker script.

            // Set s0 (the frame pointer) to the start of the stack.
            add  s0, sp, zero

            // Initialize mscratch to 0 so that we know that we are currently
            // in the kernel. This is used for the check in the trap handler.
            csrw 0x340, zero  // CSR=0x340=mscratch

            // With that initial setup out of the way, we now branch to the main
            // code, likely defined in a board's main.rs.
            j    reset_handler
        "
        :
        :
        :
        : "volatile");
    }
}

/// Setup memory for the kernel.
///
/// This moves the data segment from flash to RAM and zeros out the BSS section.
pub unsafe fn init_memory() {
    tock_rt0::init_data(&mut _etext, &mut _srelocate, &mut _erelocate);
    tock_rt0::zero_bss(&mut _szero, &mut _ezero);
}

/// The various privilege levels in RISC-V.
pub enum PermissionMode {
    User = 0x0,
    Supervisor = 0x1,
    Reserved = 0x2,
    Machine = 0x3,
}

/// Tell the MCU what address the trap handler is located at.
///
/// This is a generic implementation. There may be board specific versions as
/// some platforms have added more bits to the `mtvec` register.
///
/// The trap handler is called on exceptions and for interrupts.
pub unsafe fn configure_trap_handler(mode: PermissionMode) {
    match mode {
        PermissionMode::Machine => csr::CSR.mtvec.write(
            csr::mtvec::mtvec::trap_addr.val(_start_trap as u32 >> 2)
                + csr::mtvec::mtvec::mode::CLEAR,
        ),
        PermissionMode::Supervisor => csr::CSR.stvec.write(
            csr::stvec::stvec::trap_addr.val(_start_trap as u32 >> 2)
                + csr::stvec::stvec::mode::CLEAR,
        ),
        PermissionMode::User => csr::CSR.utvec.write(
            csr::utvec::utvec::trap_addr.val(_start_trap as u32 >> 2)
                + csr::utvec::utvec::mode::CLEAR,
        ),
        PermissionMode::Reserved => (
            // TODO some sort of error handling?
            ),
    }
}

/// This is the trap handler function. This code is called on all traps,
/// including interrupts, exceptions, and system calls from applications.
///
/// Tock uses only the single trap handler, and does not use any vectored
/// interrupts or other exception handling. The trap handler has to determine
/// why the trap handler was called, and respond accordingly. Generally, there
/// are two reasons the trap handler gets called: an interrupt occurred or an
/// application called a syscall.
///
/// In the case of an interrupt while the kernel was executing we only need to
/// save the kernel registers and then run whatever interrupt handling code we
/// need to. If the trap happens while and application was executing, we have to
/// save the application state and then resume the `switch_to()` function to
/// correctly return back to the kernel.
#[cfg(not(target_os = "none"))]
fn _start_trap() {
    // Mock implementation for tests.
}

#[cfg(target_os = "none")]
#[link_section = ".riscv.trap"]
#[export_name = "_start_trap"]
#[naked]
pub extern "C" fn _start_trap() {
    unsafe {
        asm! ("
            // The first thing we have to do is determine if we came from user
            // mode or kernel mode, as we need to save state and proceed
            // differently. We cannot, however, use any registers because we do
            // not want to lose their contents. So, we rely on `mscratch`. If
            // mscratch is 0, then we came from the kernel. If it is >0, then it
            // contains the kernel's stack pointer and we came from an app.
            //
            // We use the csrrw instruction to save the current stack pointer
            // so we can retrieve it if necessary.
            csrrw sp, 0x340, sp // CSR=0x340=mscratch
            bnez  sp, _from_app // If sp != 0 then we must have come from an app.


        _from_kernel:
            // Read back the stack pointer that we temporarily stored in
            // mscratch.
            csrr sp, 0x340    // CSR=0x340=mscratch

            // Make room for the caller saved registers we need to restore after
            // running any trap handler code.
            addi sp, sp, -16*4

            // Save all of the caller saved registers.
            sw   ra, 0*4(sp)
            sw   t0, 1*4(sp)
            sw   t1, 2*4(sp)
            sw   t2, 3*4(sp)
            sw   t3, 4*4(sp)
            sw   t4, 5*4(sp)
            sw   t5, 6*4(sp)
            sw   t6, 7*4(sp)
            sw   a0, 8*4(sp)
            sw   a1, 9*4(sp)
            sw   a2, 10*4(sp)
            sw   a3, 11*4(sp)
            sw   a4, 12*4(sp)
            sw   a5, 13*4(sp)
            sw   a6, 14*4(sp)
            sw   a7, 15*4(sp)

            // Jump to board-specific trap handler code. Likely this was and
            // interrupt and we want to disable a particular interrupt, but each
            // board/chip can customize this as needed.
            jal ra, _start_trap_rust

            // set mstatus how we expect
            li   t0, 0x1808
            csrw 0x300, t0

            // Restore the registers from the stack.
            lw   ra, 0*4(sp)
            lw   t0, 1*4(sp)
            lw   t1, 2*4(sp)
            lw   t2, 3*4(sp)
            lw   t3, 4*4(sp)
            lw   t4, 5*4(sp)
            lw   t5, 6*4(sp)
            lw   t6, 7*4(sp)
            lw   a0, 8*4(sp)
            lw   a1, 9*4(sp)
            lw   a2, 10*4(sp)
            lw   a3, 11*4(sp)
            lw   a4, 12*4(sp)
            lw   a5, 13*4(sp)
            lw   a6, 14*4(sp)
            lw   a7, 15*4(sp)

            // Reset the stack pointer.
            addi sp, sp, 16*4

            // mret returns from the trap handler. The PC is set to what is in
            // mepc and execution proceeds from there. Since we did not modify
            // mepc we will return to where the exception occurred.
            mret



            // Handle entering the trap handler from an app differently.
        _from_app:

            // At this point all we know is that we entered the trap handler
            // from an app. We don't know _why_ we got a trap, it could be from
            // an interrupt, syscall, or fault (or maybe something else).
            // Therefore we have to be very careful not to overwrite any
            // registers before we have saved them.
            //
            // We ideally want to save registers in the per-process stored state
            // struct. However, we don't have a pointer to that yet, and we need
            // to use a temporary register to get that address. So, we save s0
            // to the kernel stack before we can it to the proper spot.
            sw   s0, 0*4(sp)

            // Ideally it would be better to save all of the app registers once
            // we return back to the `switch_to_process()` code. However, we
            // also potentially need to disable an interrupt in case the app was
            // interrupted, so it is safer to just immediately save all of the
            // app registers.
            //
            // We do this by retrieving the stored state pointer from the kernel
            // stack and storing the necessary values in it.
            lw   s0,  1*4(sp)  // Load the stored state pointer into s0.
            sw   x1,  0*4(s0)  // ra
            sw   x3,  2*4(s0)  // gp
            sw   x4,  3*4(s0)  // tp
            sw   x5,  4*4(s0)  // t0
            sw   x6,  5*4(s0)  // t1
            sw   x7,  6*4(s0)  // t2
            sw   x9,  8*4(s0)  // s1
            sw   x10, 9*4(s0)  // a0
            sw   x11, 10*4(s0) // a1
            sw   x12, 11*4(s0) // a2
            sw   x13, 12*4(s0) // a3
            sw   x14, 13*4(s0) // a4
            sw   x15, 14*4(s0) // a5
            sw   x16, 15*4(s0) // a6
            sw   x17, 16*4(s0) // a7
            sw   x18, 17*4(s0) // s2
            sw   x19, 18*4(s0) // s3
            sw   x20, 19*4(s0) // s4
            sw   x21, 20*4(s0) // s5
            sw   x22, 21*4(s0) // s6
            sw   x23, 22*4(s0) // s7
            sw   x24, 23*4(s0) // s8
            sw   x25, 24*4(s0) // s9
            sw   x26, 25*4(s0) // s10
            sw   x27, 26*4(s0) // s11
            sw   x28, 27*4(s0) // t3
            sw   x29, 28*4(s0) // t4
            sw   x30, 29*4(s0) // t5
            sw   x31, 30*4(s0) // t6
            // Now retrieve the original value of s0 and save that as well.
            lw   t0,  0*4(sp)
            sw   t0,  7*4(s0)  // s0,fp

            // We also need to store the app stack pointer, mcause, and mepc. We
            // need to store mcause because we use that to determine why the app
            // stopped executing and returned to the kernel. We store mepc
            // because it is where we need to return to in the app at some
            // point.
            csrr t0, 0x340    // CSR=0x340=mscratch
            sw   t0, 1*4(s0)  // Save the app sp to the stored state struct
            csrr t0, 0x341    // CSR=0x341=mepc
            sw   t0, 31*4(s0) // Save the PC to the stored state struct
            csrr t0, 0x342    // CSR=0x342=mcause
            sw   t0, 32*4(s0) // Save mcause to the stored state struct

            // Now we need to check if this was an interrupt, and if it was,
            // then we need to disable the interrupt before returning from this
            // trap handler so that it does not fire again. If mcause is greater
            // than or equal to zero this was not an interrupt (i.e. the most
            // significant bit is not 1).
            bge  t0, zero, _from_app_continue
            // Copy mcause into a0 and then call the interrupt disable function.
            mv   a0, t0
            jal  ra, _disable_interrupt_trap_handler

        _from_app_continue:
            // Now determine the address of _return_to_kernel and resume the
            // context switching code. We need to load _return_to_kernel into
            // mepc so we can use it to return to the context switch code.
            lw   t0, 2*4(sp)  // Load _return_to_kernel into t0.
            csrw 0x341, t0    // CSR=0x341=mepc

            // Ensure that mscratch is 0. This makes sure that we know that on
            // a future trap that we came from the kernel.
            csrw 0x340, zero  // CSR=0x340=mscratch

            // Need to set mstatus.MPP to 0b11 so that we stay in machine mode.
            csrr t0, 0x300    // CSR=0x300=mstatus
            li   t1, 0x1808   // Load 0b11 to the MPP bits location in t1
            or   t0, t0, t1   // Set the MPP bits to one
            csrw 0x300, t0    // CSR=0x300=mstatus

            // Use mret to exit the trap handler and return to the context
            // switching code.
            mret
        "
        :
        :
        :
        : "volatile");
    }
}

/// Ensure an abort symbol exists.
#[cfg(target_os = "none")]
#[link_section = ".init"]
#[export_name = "abort"]
pub extern "C" fn abort() {
    unsafe {
        asm! ("
            // Simply go back to the start as if we had just booted.
            j    _start
        "
        :
        :
        :
        : "volatile");
    }
}
