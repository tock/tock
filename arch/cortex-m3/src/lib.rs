#![feature(asm, const_fn, naked_functions)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, register_bitfields, register_bitmasks)]
extern crate kernel;
extern crate cortexm;

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m3.
pub use cortexm::support;

pub use cortexm::nvic;
pub use cortexm::scb;
pub use cortexm::syscall;
pub use cortexm::systick;

#[no_mangle]
#[naked]
pub unsafe extern "C" fn systick_handler() {
    asm!(
        "
        /* Mark that the systick handler was called meaning that the process */
        /* stopped executing because it has exceeded its timeslice. */
        ldr r0, =SYSTICK_EXPIRED
        mov r1, #1
        str r1, [r0, #0]

        /* Set thread mode to privileged */
        mov r0, #0
        msr CONTROL, r0

        movw LR, #0xFFF9
        movt LR, #0xFFFF
         "
    );
}

#[no_mangle]
#[naked]
/// All ISRs are caught by this handler which indirects to a custom handler by
/// indexing into `INTERRUPT_TABLE` based on the ISR number.
pub unsafe extern "C" fn generic_isr() {
    asm!(
        "
    /* Skip saving process state if not coming from user-space */
    cmp lr, #0xfffffffd
    bne _ggeneric_isr_no_stacking

    /* We need the most recent kernel's version of r1, which points */
    /* to the Process struct's stored registers field. The kernel's r1 */
    /* lives in the second word of the hardware stacked registers on MSP */
    mov r1, sp
    ldr r1, [r1, #4]
    stmia r1, {r4-r11}

    /* Set thread mode to privileged */
    mov r0, #0
    msr CONTROL, r0

    movw LR, #0xFFF9
    movt LR, #0xFFFF
_ggeneric_isr_no_stacking:
    /* Find the ISR number by looking at the low byte of the IPSR registers */
    mrs r0, IPSR
    and r0, #0xff
    /* ISRs start at 16, so substract 16 to get zero-indexed */
    sub r0, #16

    /*
     * High level:
     *    NVIC.ICER[r0 / 32] = 1 << (r0 & 31)
     * */
	lsrs	r2, r0, #5 /* r2 = r0 / 32 */

    /* r0 = 1 << (r0 & 31) */
	movs r3, #1        /* r3 = 1 */
	and	r0, r0, #31    /* r0 = r0 & 31 */
	lsl	r0, r3, r0     /* r0 = r3 << r0 */

    /* r3 = &NVIC.ICER */
    mov r3, #0xe180
    movt r3, #0xe000

    /* here:
     *
     *  `r2` is r0 / 32
     *  `r3` is &NVIC.ICER
     *  `r0` is 1 << (r0 & 31)
     *
     * So we just do:
     *
     *  `*(r3 + r2 * 4) = r0`
     *
     *  */
	str	r0, [r3, r2, lsl #2]"
    );
}

#[no_mangle]
#[naked]
#[allow(non_snake_case)]
pub unsafe extern "C" fn SVC_Handler() {
    asm!(
        "
  cmp lr, #0xfffffff9
  bne to_kernel

  /* Set thread mode to unprivileged */
  mov r0, #1
  msr CONTROL, r0

  movw lr, #0xfffd
  movt lr, #0xffff
  bx lr
to_kernel:
  ldr r0, =SYSCALL_FIRED
  mov r1, #1
  str r1, [r0, #0]

  /* Set thread mode to privileged */
  mov r0, #0
  msr CONTROL, r0

  movw LR, #0xFFF9
  movt LR, #0xFFFF"
    );
}

#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn switch_to_user(user_stack: *const u8, process_got: *const u8) -> *mut u8 {
    user_stack as *mut u8
}

#[cfg(target_os = "none")]
#[no_mangle]
/// r0 is top of user stack, r1 Process GOT
pub unsafe extern "C" fn switch_to_user(
    mut user_stack: *const u8,
    process_regs: &mut [usize; 8],
) -> *mut u8 {
    asm!("
    /* Load bottom of stack into Process Stack Pointer */
    msr psp, $0

    /* Load non-hardware-stacked registers from Process stack */
    /* Ensure that $2 is stored in a callee saved register */
    ldmia $2, {r4-r11}

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    /* Push non-hardware-stacked registers into Process struct's */
    /* regs field */
    stmia $2, {r4-r11}

    mrs $0, PSP /* PSP into r0 */"
    : "={r0}"(user_stack)
    : "{r0}"(user_stack), "{r1}"(process_regs)
    : "r4","r5","r6","r7","r8","r9","r10","r11");
    user_stack as *mut u8
}
