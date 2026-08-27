// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Semihosting support for ARM Cortex M Architectures.

/// Issue an ARM semihosting call.
///
/// `operation` is the semihosting operation number (e.g. `0x18` for
/// `SYS_EXIT`) and `parameter` is its operation-specific argument.
///
/// Not exposed outside this module: it's a general, unrestricted semihosting
/// interface. External callers should use specific, narrow operations (e.g.
/// [`semihost_terminate`]) that encode specific commands.
///
/// # Safety
///
/// Only meaningful when running under a semihosting host (e.g. QEMU started
/// with `-semihosting`, or an attached debug probe); otherwise the `bkpt`
/// instruction traps with no host to service it, so **the caller must not
/// assume this call takes effect**.
///
/// The exact safety requirements depend on `operation`. This method should
/// not be called directly with raw parameters. Instead, this module wraps
/// calls with fixed `operation`s that specify their safety requirements.
// Reference documentation for Cortex-M series arches semihosting:
// https://support.arm.com/documentation/dui0471/e/semihosting/the-semihosting-interface
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[inline(always)]
#[doc(hidden)]
unsafe fn semihost_command(operation: u32, parameter: u32) -> u32 {
    use core::arch::asm;
    let result;

    // SAFETY: r0/r1 are set to `operation`/`parameter`, per the ABI ARM
    // semihosting defines (ARM's "Semihosting for AArch32 and AArch64"
    // specification); the caller is responsible for those being valid for
    // the chosen `operation`, per this function's own `# Safety` doc above.
    // - OUTPUTS: r0 is overwritten with the semihosting call's result.
    // - Options set:
    //   - nostack: This does not use the stack.
    // - Options not set:
    //   - nomem: not guaranteed in general -- some operations (e.g.
    //     `SYS_WRITEC`) dereference `parameter` as a pointer.
    //   - pure, readonly: not applicable, as above.
    //   - preserves_flags: not documented by the semihosting spec.
    //   - noreturn: we do fall through (there may be no host to service
    //     this call at all, e.g. real hardware with no debugger attached).
    //   - att_syntax: not on arm.
    //   - raw: not required.
    unsafe {
        asm!(
            "bkpt #0xAB",
            inout("r0") operation => result,
            in("r1") parameter,
            options(nostack),
        );
    }
    result
}

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
unsafe fn semihost_command(_operation: u32, _parameter: u32) -> u32 {
    unimplemented!()
}

/// Reason codes for SysExit under semihosting.
///
/// See ARM documentation for details:
/// https://support.arm.com/documentation/dui0471/e/semihosting/angel-swireason-reportexception--0x18-?lang=en
#[repr(u32)]
#[allow(non_camel_case_types)] // Match ARM Specs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysexitReason {
    // Hardware events
    ADP_Stopped_BranchThroughZero = 0x20000,
    ADP_Stopped_UndefinedInstr = 0x20001,
    ADP_Stopped_SoftwareInterrupt = 0x20002,
    ADP_Stopped_PrefetchAbort = 0x20003,
    ADP_Stopped_DataAbort = 0x20004,
    ADP_Stopped_AddressException = 0x20005,
    ADP_Stopped_IRQ = 0x20006,
    ADP_Stopped_FIQ = 0x20007,
    // Software events
    ADP_Stopped_BreakPoint = 0x20020,
    ADP_Stopped_WatchPoint = 0x20021,
    ADP_Stopped_StepComplete = 0x20022,
    ADP_Stopped_RunTimeErrorUnknown = 0x20023,
    ADP_Stopped_InternalError = 0x20024,
    ADP_Stopped_UserInterruption = 0x20025,
    ADP_Stopped_ApplicationExit = 0x20026,
    ADP_Stopped_StackOverflow = 0x20027,
    ADP_Stopped_DivisionByZero = 0x20028,
    ADP_Stopped_OSSpecific = 0x20029,
}

/// Ask a semihosting host to terminate execution.
///
/// Issues ARM semihosting's `SYS_EXIT` (`0x18`) [aka,
/// `angel_SWIreason_ReportException (0x18)` on older cortex-m arches,
/// but they have the same semantics].
///
/// # Safety
///
/// This nominally halts execution, thus the caller should have the authority to
/// halt execution. This *should* only be called on under semihosting (e.g. on
/// a QEMU board); on other targets the `BPKT` will escalate to a `HardFault`.
///
/// This **may not actually halt execution**. A debugger *can* tell semihosting
/// to resume the target. Callers must assume this can fall through.
#[inline(always)]
pub unsafe fn terminate(reason: SysexitReason) {
    const SYS_EXIT: u32 = 0x18;
    // SAFETY: SYS_EXIT is a valid `operation` for semihosting. With SYS_EXIT,
    // `parameter` is interpreted as a plain integer, which are constrained to
    // valid values for SYS_EXIT by the enum type parameter.
    unsafe {
        semihost_command(SYS_EXIT, reason as u32);
    }
}
