// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Low level support for ARM Cortex-R52 processors (ARMv8-R).

// Only A32 instruction set is supported. This require to adapt
// user mode libraries accordingly.

// The code here will be put in ATCM on the mps3

// IMPORTANT TODO: FP context is not done at all, and I did not
// check 8 bytes alignment stack for LDxEXD, STxEXD, as required by
// ABI spec AAPCS.
// Also, probably not all registers need to be saved during a
// context switch, so a better implementation could save a few
// resources.

#![allow(dead_code)]

use core::arch::naked_asm;

/* ARMv8 processor modes */
const MODE_USR: u16 = 0x10; /* unprivileged mode in which most applications run                           */
const MODE_FIQ: u16 = 0x11; /* entered on an FIQ interrupt exception                                      */
const MODE_IRQ: u16 = 0x12; /* entered on an IRQ interrupt exception                                      */
const MODE_SVC: u16 = 0x13; /* entered on reset or when a Supervisor Call instruction ( SVC ) is executed */
const MODE_MON: u16 = 0x16; /* security extensions                                                        */
const MODE_ABT: u16 = 0x17; /* entered on a memory access exception                                       */
const MODE_HYP: u16 = 0x1a; /* virtualization extensions                                                  */
const MODE_UND: u16 = 0x1b; /* entered when an undefined instruction executed                             */
const MODE_SYS: u16 = 0x1f; /* privileged mode, sharing the register view with User mode                  */

const MODE_MASK: u16 = 0x1f;
const NO_ABORT: u16 = 0x100; /* mask to disable Abort Exception */
const NO_IRQ: u16 = 0x80; /* mask to disable IRQ             */
const NO_FIQ: u16 = 0x40; /* mask to disable FIQ             */
const NO_INT: u16 = NO_IRQ | NO_FIQ; /* mask to disable IRQ and FIQ     */
const THUMB_STATE: u16 = 0x20;

// These constants are defined in the linker script.
extern "C" {
    static _szero: *const u32;
    static _ezero: *const u32;
    static _etext: *const u32;
    static _srelocate: *const u32;
    static _erelocate: *const u32;
}

core::arch::global_asm!(
    ".global initialize_ram_jump_to_main",
    ".set initialize_ram_jump_to_main, EL2_RST_Handler"
);

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn EL2_RST_Handler() {
    naked_asm!(
        "
        // This code is taken from 'Cortex-R52 and Cortex-R52+ Programmer's Guide'

        // If run on a multi-core system, put any secondary cores to sleep
        MRC p15, 0, r0, c0, c0, 5   // Read MPIDR, check which CPU I am
        ANDS r0, r0, #0xF
        BEQ cpu0
loop_wfi:
        DSB SY      // Clear all pending data accesses
        WFI         // Go to sleep
        B loop_wfi

cpu0:   // Here we do the minimum setup for EL2 then switch to EL1 forever

        //No change to HVBAR

        // default HSCTLR OK for now
        //mrc p15,4,<Rt>,c1,c0,0 ; Read HSCTLR into Rt
        //mcr p15,4,<Rt>,c1,c0,0 ; Write Rt to HSCTLR

        // Change EL1 exception base address (same than EL2)
        ldr r0, =vectors
        mcr p15, 0, r0, c12, c0, 0  // Write to VBAR

        // default HACTLR of for now
        //mcr p15, 4, r0, c1, c0, 1 // Write to HACTLR

        // Enable EL1 access to all IMP DEF registers
        //ldr r0, =0x7F81
        //mcr p15, 4, r0, c1, c0, 1 // Write to HACTLR

        // Enable the floating-point and Advanced SIMD.
        //mrc p15, 0, r0, c1, c0, 2   // Read CPACR
        //orr r0, r0, #(0xF << 20)    // Enable access to CP 10 & 11
        //mcr p15, 0, r0, c1, c0, 2   // Write CPACR
        //isb
        //mov r0, #0x40000000
        //vmsr FPEXC, r0              // Write FPEXC register, EN bit set

        // Init System Timer CNTFRQ, should be 50_000_000 (MCLK/ACLK)
        //ldr r0, =0
        //mcr p15, 0, r0, c14, c0, 0  // Write r0 to CNTFRQ

        //Disable the HVC instruction execution at EL1
        mrc p15, 4, r0, c1, c1, 0   //read HCR
        orr r0, r0, #(0x1<<29)
        mcr p15, 4, r0, c1, c1, 0   //write HCR

        // Go to SVC mode
        mrs r0, CPSR
        mov r1, #{MODE_SVC} | {NO_IRQ}
        bfi r0, r1, #0, #9
        msr SPSR, r0
        ldr r0, =EL1_RST_Handler
        msr ELR_HYP, r0
        dsb
        isb
        eret

EL1_RST_Handler:
        mrc p15, 0, r0, c1, c0, 0   // Read System Control Register
        bic r0, r0, #0x05           // Disable MPU (M bit) and data cache (C bit)
        bic r0, r0, #0x1000         // Disable instruction cache (I bit)
                                    // set TE[30] T32 Exception Enable ?
        dsb                         //
        mcr p15, 0, r0, c1, c0, 0   // Write System Control Register
        isb                         //

        // Start by initializing .bss memory. The Tock linker script defines
        // `_szero` and `_ezero` to mark the .bss segment.
        ldr r0, ={sbss}     // r0 = first address of .bss
        ldr r1, ={ebss}     // r1 = first address after .bss

        movs r2, #0         // r2 = 0

100:    // bss_init_loop
        cmp r1, r0          // We increment r0. Check if we have reached r1
                            // (end of .bss), and stop if so.
        beq 101f            // If r0 == r1, we are done.
        stm r0!, {{r2}}     // Write a word to the address in r0, and increment r0.
                            // Since r2 contains zero, this will clear the memory
                            // pointed to by r0. Using `stm` (store multiple) with the
                            // bang allows us to also increment r0 automatically.
        b 100b              // Continue the loop.

101:    // bss_init_done

        // Now initialize .data memory. This involves coping the values right at the
        // end of the .text section (in flash) into the .data section (in RAM).
        ldr r0, ={sdata}    // r0 = first address of data section in RAM
        ldr r1, ={edata}    // r1 = first address after data section in RAM
        ldr r2, ={etext}    // r2 = address of stored data initial values

200:    // data_init_loop
        cmp r1, r0          // We increment r0. Check if we have reached the end
                            // of the data section, and if so we are done.
        beq 201f            // r0 == r1, and we have iterated through the .data section
        ldm r2!, {{r3}}     // r3 = *(r2), r2 += 1. Load the initial value into r3,
                            // and use the bang to increment r2.
        stm r0!, {{r3}}     // *(r0) = r3, r0 += 1. Store the value to memory, and
                            // increment r0.
        b 200b              // Continue the loop.

201:    // data_init_done

        // init stacks
        // HYP: not done
        // msr SP_hyp, r0

        // IRQ/UND/ABT: unused or crash so same stack, size 256
        // SYSTEM mode unused and same SP as USR
        ldr r0, =_estack
        msr SP_irq, r0
        msr SP_und, r0
        msr SP_abt, r0
        sub	r0, r0, #256

        // FIQ size 256
        msr SP_fiq, r0
        sub	r0, r0, #256

        // Cannot use SP_svc because current mode is svc
        // SVC size / TODO: is it enough? check main.rs in boards too
        mov SP, r0
        sub	r0, r0, #2048

        // USR size remaining (0x4000 - 3 x 256)
        msr SP_usr, r0

        // Now that memory has been initialized, we can jump to main() where the
        // board initialization takes place and Rust code starts.
        bl main
        ",
        sbss = sym _szero,
        ebss = sym _ezero,
        sdata = sym _srelocate,
        edata = sym _erelocate,
        etext = sym _etext,
        MODE_SVC = const MODE_SVC,
        NO_IRQ = const NO_IRQ,
    );
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DEF_Handler() {
    naked_asm!("bkpt");
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn UND_Handler() {
    naked_asm!("bkpt");
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(link_section = ".atcm")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SVC_Handler() {
    naked_asm!(
        "
        // should only happen in USR mode

        // use lr as sp to save regs in user stack
        // check switch_to_user_arm_cr52() below for frame layout
        push {{lr}}
        mrs lr, sp_usr
        stmdb lr!, {{r0-r12}}
        mov r0, lr
        mrs r1, lr_usr
        mrs r2, spsr
        // retrieve lr as return address
        pop {{r3}}
        stmdb r0!, {{r1-r3}}

        // SVC from userspace: mark that a syscall occurred.
        ldr r2, =SYSCALL_FIRED
        mov r3, #1
        str r3, [r2]

        // restore SVC context ; lr will hold return address from switch_to_user()
        pop {{r4-r12, lr}}
        bx lr       // r0 is the user stack
        ",
    );
}
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn PFT_Handler() {
    naked_asm!("bkpt");
}
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DAB_Handler() {
    naked_asm!("bkpt");
}

// TODO: FIQ handler should be implemented just after the vecor table to
// avoid a useless jump
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(link_section = ".atcm")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn FIQ_Handler() {
    naked_asm!(
        "
        // remeber that R8-R12 are save by the HW, unlike any other modes
        // First we mask FIQ
        mrs     r8, spsr
        orr     r8, r8, #{NO_FIQ}
        msr     spsr, r8
        // then adjust LR to the correct return address
        sub     lr, lr, 4

        // if previous mode was SVC we just return
        ands    r8, r8, #0xF    // USR is 0x10, SVC is 0x13
        eretne

        // previous mode was USR, we save the context
        // check switch_to_user_arm_cr52() below for frame layout

        // use lr as sp to save regs in user stack
        push {{lr}}
        mrs lr, sp_usr
        stmdb lr!, {{r0-r12}}
        mov r0, lr
        mrs r1, lr_usr
        mrs r2, spsr
        // retrieve lr as return address
        pop {{r3}}
        stmdb r0!, {{r1-r3}}

        // switch to SVC
        mrs r1, CPSR
        mov r2, #{MODE_SVC} | {NO_FIQ}
        bfi r1, r2, #0, #9
        msr CPSR, r1
        isb

        // restore SVC context ; lr will hold return address from switch_to_user()
        pop {{r4-r12, lr}}
        bx lr       // r0 is the user stack
        ",
        //MODE_MASK = const MODE_MASK,
        MODE_SVC = const MODE_SVC,
        NO_FIQ = const NO_FIQ,
    );
}
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn IRQ_Handler() {
    naked_asm!("bkpt",);
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors_atcm"
)]
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vectors() {
    naked_asm!(
        "
        b   EL2_RST_Handler
        b   DEF_Handler     // UND_Handler
        b   SVC_Handler
        b   DEF_Handler     // PFT_Handler
        b   DEF_Handler     // DAB_Handler
        .word 0             // RSV
        b   DEF_Handler     // IRQ_Handler
        b   FIQ_Handler     // TODO: move the handler here
        "
    );
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(link_section = ".atcm")]
#[unsafe(naked)]
pub unsafe extern "C" fn switch_to_user_arm_cr52(user_stack: *const usize) -> *const usize {
    naked_asm!(
        "
        // User stack with saved context
        //  | LR          | <- r0 (user SP) before
        //  | CPSR        |
        //  | ret addr    |
        //  | R0          |
        //  | R1 ... R11  |
        //  | R12         |
        //  |             | <- SP_usr after

        // save SVC context and return address
        push {{r4-r12, lr}}

        // restore lr, cpsr and return address from user stack
        ldmia r0!, {{r1-r3}}
        msr lr_usr, r1
        msr spsr, r2
        mov lr, r3

        // set sp_usr above saved registers
        mov r1, r0
        add r1, #4*13
        msr sp_usr, r1

        // restore user context
        ldmia r0, {{r0-r12}}

        // back to user
        eret    // will use lr_svc and spsr_svc
        ",
    );
}
