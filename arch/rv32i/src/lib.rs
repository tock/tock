// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Support for the 32-bit RISC-V architecture.

#![crate_name = "rv32i"]
#![crate_type = "rlib"]
#![no_std]

use core::fmt::Write;

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
use core::arch::global_asm;

use kernel::utilities::registers::interfaces::{Readable, Writeable};

pub mod clic;
pub mod machine_timer;
pub mod pmp;
pub mod support;
pub mod syscall;

// Re-export the shared CSR library so that dependent crates do not have to have
// both rv32i and riscv as dependencies.
pub use riscv::csr;

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

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
extern "C" {
    // Entry point of all programs (`_start`).
    ///
    /// This assembly does three functions:
    ///
    /// 1. It initializes the stack pointer, the frame pointer (needed for closures
    ///    to work in start_rust) and the global pointer.
    /// 2. It initializes the .bss and .data RAM segments. This must be done before
    ///    any Rust code runs. See https://github.com/tock/tock/issues/2222 for more
    ///    information.
    /// 3. Finally it calls `main()`, the main entry point for Tock boards.
    pub fn _start();
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
global_asm! ("
            .section .riscv.start, \"ax\"
            .globl _start
          _start:

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
pub unsafe fn configure_trap_handler() {
    csr::CSR.mtvec.write(
        csr::mtvec::mtvec::trap_addr.val(_start_trap as usize >> 2)
            + csr::mtvec::mtvec::mode::CLEAR,
    );
    csr::CSR.mscratch.set(0);
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
pub extern "C" fn _start_trap() {
    unimplemented!()
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
extern "C" {
    /// TODO
    pub fn _start_trap();
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
global_asm!(
    "
            .section .riscv.trap, \"ax\"
            .globl _start_trap
            .globl _start_kernel_trap
          _start_trap:
            // This is the global trap handler. By default, Tock expects this
            // trap handler to be registered at all times, and that all traps
            // and interrupts occurring in all modes of execution (M-, S-, and
            // U-mode) will cause this trap handler to be executed.
            //
            // Its behavior is well defined. It will:
            //
            // 1. atomically swap t0 and the mscratch CSR,
            // 2. execute the default kernel trap handler if t0 now contains `0`
            //    (meaning that the mscratch CSR contained `0` before entering
            //    this trap handler),
            // 3. otherwise, jump to the address loaded into t0.
            //
            // No registers other than t0 and the mscratch CSR are to be
            // clobberd before contiuing execution at the address loaded into
            // the mscratch CSR or the _start_kernel_trap kernel trap handler.
            // Execution with these second-stage trap handlers must continue
            // in the same trap handler context as originally invoked by the
            // trap (e.g., the global trap handler will not execute an mret
            // instruction). It will not modify CSRs that contain information
            // on the trap source or the system state prior to entering the
            // trap handler.
            //
            // When a custom trap handler is registered in `mscratch`, the
            // custom handler is responsible for restoring the kernel trap
            // handler (by setting mscratch=0) before returning to kernel
            // execution from the trap handler context.
            //
            // If a board or chip must, for whichever reason, use a different
            // global trap handler, it should abide to the above contract and
            // emulate its behavior for all traps and interrupts that are
            // required to be handled by the respective kernel or other trap
            // handler as registered in mscratch.
            //
            // For instance, a chip that does not support non-vectored trap
            // handlers can register a vectored trap handler that routes each
            // trap source to this global trap handler.
            //
            // Alternatively, a board can be allowed to ignore certain traps or
            // interrupts, some or all of the time, provided they are not vital
            // to Tock's execution. These boards may choose to register an
            // alternative handler for some or all trap sources. When this
            // alternative handler is invoked, it may, for instance, choose to
            // ignore a certain trap, access global state (subject to
            // synchronization), etc. It must still abide to the contract as
            // stated above.

            // Atomically swap t0 and mscratch:
            csrrw t0, mscratch, t0

            // If mscratch contained 0, invoke the kernel trap handler.
            beq   t0, x0, _start_kernel_trap

            // Else, invoke the trap handler at the address that was loaded in
            // the mscratch CSR.
            jr    t0

          _start_kernel_trap:

            // The global trap handler has swapped t0 into mscratch. We can thus
            // freely clobber t0 without losing any information.
            //
            // Since we want to use the stack to save kernel registers, we
            // first need to make sure that the trap wasn't the result of a
            // stack overflow, in which case we can't use the current stack
            // pointer. Use t0 as a scratch register:

            // Load the address of the bottom of the stack (`_sstack`) into our
            // newly freed-up t0 register.
            la t0, {sstack}                     // t0 = _sstack

            // Compare the kernel stack pointer to the bottom of the stack. If
            // the stack pointer is above the bottom of the stack, then continue
            // handling the fault as normal.
            bgtu sp, t0, 200f                   // branch if sp > t0

            // If we get here, then we did encounter a stack overflow. We are
            // going to panic at this point, but for that to work we need a
            // valid stack to run the panic code. We do this by just starting
            // over with the kernel stack and placing the stack pointer at the
            // top of the original stack.
            la sp, {estack}                     // sp = _estack

        200: // _start_kernel_trap_continue

            // Restore t0. We reset mscratch to 0 (kernel trap handler mode)
            csrrw t0, mscratch, 0

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

            // Jump to board-specific trap handler code. Likely this was an
            // interrupt and we want to disable a particular interrupt, but each
            // board/chip can customize this as needed.
            jal ra, _start_trap_rust_from_kernel

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
    ",
    estack = sym _estack,
    sstack = sym _sstack,
);

/// RISC-V semihosting needs three exact instructions in uncompressed form.
///
/// See https://github.com/riscv/riscv-semihosting-spec/blob/main/riscv-semihosting-spec.adoc#11-semihosting-trap-instruction-sequence
/// for more details on the three insturctions.
///
/// In order to work with semihosting we include the assembly here
/// where we are able to disable compressed instruction support. This
/// follows the example used in the Linux kernel:
/// https://elixir.bootlin.com/linux/v5.12.10/source/arch/riscv/include/asm/jump_label.h#L21
/// as suggested by the RISC-V developers:
/// https://groups.google.com/a/groups.riscv.org/g/isa-dev/c/XKkYacERM04/m/CdpOcqtRAgAJ
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
pub unsafe fn semihost_command(command: usize, arg0: usize, arg1: usize) -> usize {
    use core::arch::asm;
    let res;
    asm!(
    "
      .balign 16
      .option push
      .option norelax
      .option norvc
      slli x0, x0, 0x1f
      ebreak
      srai x0, x0, 7
      .option pop
      ",
    in("a0") command,
    in("a1") arg0,
    in("a2") arg1,
    lateout("a0") res,
    );
    res
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
pub unsafe fn semihost_command(_command: usize, _arg0: usize, _arg1: usize) -> usize {
    unimplemented!()
}

/// Print a readable string for an mcause reason.
pub unsafe fn print_mcause(mcval: csr::mcause::Trap, writer: &mut dyn Write) {
    match mcval {
        csr::mcause::Trap::Interrupt(interrupt) => match interrupt {
            csr::mcause::Interrupt::UserSoft => {
                let _ = writer.write_fmt(format_args!("User software interrupt"));
            }
            csr::mcause::Interrupt::SupervisorSoft => {
                let _ = writer.write_fmt(format_args!("Supervisor software interrupt"));
            }
            csr::mcause::Interrupt::MachineSoft => {
                let _ = writer.write_fmt(format_args!("Machine software interrupt"));
            }
            csr::mcause::Interrupt::UserTimer => {
                let _ = writer.write_fmt(format_args!("User timer interrupt"));
            }
            csr::mcause::Interrupt::SupervisorTimer => {
                let _ = writer.write_fmt(format_args!("Supervisor timer interrupt"));
            }
            csr::mcause::Interrupt::MachineTimer => {
                let _ = writer.write_fmt(format_args!("Machine timer interrupt"));
            }
            csr::mcause::Interrupt::UserExternal => {
                let _ = writer.write_fmt(format_args!("User external interrupt"));
            }
            csr::mcause::Interrupt::SupervisorExternal => {
                let _ = writer.write_fmt(format_args!("Supervisor external interrupt"));
            }
            csr::mcause::Interrupt::MachineExternal => {
                let _ = writer.write_fmt(format_args!("Machine external interrupt"));
            }
            csr::mcause::Interrupt::Unknown => {
                let _ = writer.write_fmt(format_args!("Reserved/Unknown"));
            }
        },
        csr::mcause::Trap::Exception(exception) => match exception {
            csr::mcause::Exception::InstructionMisaligned => {
                let _ = writer.write_fmt(format_args!("Instruction access misaligned"));
            }
            csr::mcause::Exception::InstructionFault => {
                let _ = writer.write_fmt(format_args!("Instruction access fault"));
            }
            csr::mcause::Exception::IllegalInstruction => {
                let _ = writer.write_fmt(format_args!("Illegal instruction"));
            }
            csr::mcause::Exception::Breakpoint => {
                let _ = writer.write_fmt(format_args!("Breakpoint"));
            }
            csr::mcause::Exception::LoadMisaligned => {
                let _ = writer.write_fmt(format_args!("Load address misaligned"));
            }
            csr::mcause::Exception::LoadFault => {
                let _ = writer.write_fmt(format_args!("Load access fault"));
            }
            csr::mcause::Exception::StoreMisaligned => {
                let _ = writer.write_fmt(format_args!("Store/AMO address misaligned"));
            }
            csr::mcause::Exception::StoreFault => {
                let _ = writer.write_fmt(format_args!("Store/AMO access fault"));
            }
            csr::mcause::Exception::UserEnvCall => {
                let _ = writer.write_fmt(format_args!("Environment call from U-mode"));
            }
            csr::mcause::Exception::SupervisorEnvCall => {
                let _ = writer.write_fmt(format_args!("Environment call from S-mode"));
            }
            csr::mcause::Exception::MachineEnvCall => {
                let _ = writer.write_fmt(format_args!("Environment call from M-mode"));
            }
            csr::mcause::Exception::InstructionPageFault => {
                let _ = writer.write_fmt(format_args!("Instruction page fault"));
            }
            csr::mcause::Exception::LoadPageFault => {
                let _ = writer.write_fmt(format_args!("Load page fault"));
            }
            csr::mcause::Exception::StorePageFault => {
                let _ = writer.write_fmt(format_args!("Store/AMO page fault"));
            }
            csr::mcause::Exception::Unknown => {
                let _ = writer.write_fmt(format_args!("Reserved"));
            }
        },
    }
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
        " (interrupt={}, exception code={:#010X})",
        interrupt, code
    ));
    let _ = writer.write_fmt(format_args!(
        "\r\nLast value (mtval):  {:#010X}\
         \r\n\
         \r\nSystem register dump:\
         \r\n mepc:    {:#010X}    mstatus:     {:#010X}\
         \r\n mcycle:  {:#010X}    minstret:    {:#010X}\
         \r\n mtvec:   {:#010X}",
        csr::CSR.mtval.get(),
        csr::CSR.mepc.get(),
        csr::CSR.mstatus.get(),
        csr::CSR.mcycle.get(),
        csr::CSR.minstret.get(),
        csr::CSR.mtvec.get()
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
        "\r\n mstatus: {:#010X}\
         \r\n  uie:    {:5}  upie:   {}\
         \r\n  sie:    {:5}  spie:   {}\
         \r\n  mie:    {:5}  mpie:   {}\
         \r\n  spp:    {}",
        mstatus.get(),
        uie,
        upie,
        sie,
        spie,
        mie,
        mpie,
        spp
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
        "\r\n mie:   {:#010X}   mip:   {:#010X}\
         \r\n  usoft:  {:6}              {:6}\
         \r\n  ssoft:  {:6}              {:6}\
         \r\n  msoft:  {:6}              {:6}\
         \r\n  utimer: {:6}              {:6}\
         \r\n  stimer: {:6}              {:6}\
         \r\n  mtimer: {:6}              {:6}\
         \r\n  uext:   {:6}              {:6}\
         \r\n  sext:   {:6}              {:6}\
         \r\n  mext:   {:6}              {:6}\r\n",
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
        p_mext
    ));
}
