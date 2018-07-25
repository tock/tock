#![feature(asm, const_fn, naked_functions)]
#![no_std]

extern crate cortexm;
extern crate kernel;

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m0.
pub use cortexm::support;

pub use cortexm::nvic;

#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn generic_isr() {}

#[cfg(target_os = "none")]
#[naked]
/// All ISRs are caught by this handler which disables the NVIC and switches to the kernel.
pub unsafe extern "C" fn generic_isr() {
    asm!(
        "
    /* Skip saving process state if not coming from user-space */
    ldr r0, MEXC_RETURN_PSP
    cmp lr, r0
    bne _ggeneric_isr_no_stacking

    /* We need the most recent kernel's version of r1, which points */
    /* to the Process struct's stored registers field. The kernel's r1 */
    /* lives in the second word of the hardware stacked registers on MSP */
    mov r1, sp
    ldr r1, [r1, #4]
    str r4, [r1, #16]
    str r5, [r1, #20]
    str r6, [r1, #24]
    str r7, [r1, #28]

    push {r4-r7}
    mov  r4, r8
    mov  r5, r9
    mov  r6, r10
    mov  r7, r11
    str r4, [r1, #0]
    str r5, [r1, #4]
    str r6, [r1, #8]
    str r7, [r1, #12]
    pop {r4-r7}

    ldr r0, MEXC_RETURN_MSP
_ggeneric_isr_no_stacking:
    /* Find the ISR number by looking at the low byte of the IPSR registers */
    mrs r0, IPSR
    movs r1, #0xff
    ands r0, r1
    /* ISRs start at 16, so substract 16 to get zero-indexed */
    subs r0, r0, #16

    /*
     * High level:
     *    NVIC.ICER[r0 / 32] = 1 << (r0 & 31)
     * */
    /* r3 = &NVIC.ICER[r0 / 32] */
	ldr	r2, NVICICER     /* r2 = &NVIC.ICER */
	lsrs	r3, r0, #5   /* r3 = r0 / 32 */
	lsls	r3, r3, #2   /* ICER is word-sized, so multiply offset by 4 */
	adds	r3, r3, r2   /* r3 = r2 + r3 */

    /* r2 = 1 << (r0 & 31) */
	movs	r2, #31      /* r2 = 31 */
	ands	r0, r2       /* r0 = r0 & r2 */
	subs	r2, r2, #30  /* r2 = r2 - 30 i.e. r2 = 1 */
	lsls	r2, r2, r0   /* r2 = 1 << r0 */

    /* *r3 = r2 */
	str	r2, [r3]
    bx lr /* return here since we have extra words in the assembly */

.align 2
NVICICER:
  .word 0xE000E180
MEXC_RETURN_MSP:
  .word 0xFFFFFFF9
MEXC_RETURN_PSP:
  .word 0xFFFFFFFD"
    );
}

#[naked]
#[allow(non_snake_case)]
pub unsafe extern "C" fn SVC_Handler() {
    asm!(
        "
  ldr r0, EXC_RETURN_MSP
  cmp lr, r0
  bne to_kernel
  ldr r1, EXC_RETURN_PSP
  bx r1

to_kernel:
  ldr r0, =SYSCALL_FIRED
  movs r1, #1
  str r1, [r0, #0]
  ldr r1, EXC_RETURN_MSP
  bx r1

EXC_RETURN_MSP:
  .word 0xFFFFFFF9
EXC_RETURN_PSP:
  .word 0xFFFFFFFD
  "
    );
}

#[no_mangle]
pub unsafe extern "C" fn switch_to_user(
    mut user_stack: *const u8,
    process_regs: &mut [usize; 8],
) -> *mut u8 {
    asm!("
    /* Load non-hardware-stacked registers from Process stack */
    ldmia $2!, {r4-r7}
    mov r11, r7
    mov r10, r6
    mov r9,  r5
    mov r8,  r4
    ldmia $2!, {r4-r7}
    subs $2, 32 /* Restore pointer to process_regs
                /* ldmia! added a 32-byte offset */

    /* Load bottom of stack into Process Stack Pointer */
    msr psp, $0

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    /* Store non-hardware-stacked registers in process_regs */
    /* $2 still points to process_regs because we are clobbering all */
    /* non-hardware-stacked registers */
    str r4, [$2, #16]
    str r5, [$2, #20]
    str r6, [$2, #24]
    str r7, [$2, #28]

    mov  r4, r8
    mov  r5, r9
    mov  r6, r10
    mov  r7, r11

    str r4, [$2, #0]
    str r5, [$2, #4]
    str r6, [$2, #8]
    str r7, [$2, #12]

    mrs $0, PSP /* PSP into user_stack */

    "
    : "={r0}"(user_stack)
    : "{r0}"(user_stack), "{r1}"(process_regs)
    : "r4","r5","r6","r7","r8","r9","r10","r11");
    user_stack as *mut u8
}
