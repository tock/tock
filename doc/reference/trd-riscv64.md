RISC-V 64-bit ABI
========================================

**TRD:** <br/>
**Working Group:** Core<br/>
**Type:** Documentary<br/>
**Status:** Draft<br/>
**Authors:** Johnathan Van Why<br/>
**Extends:** [104](trd104-syscalls.md)<br/>
**Draft-Created:** 2026-06-16<br/>
**Draft-Modified:** 2026-06-16<br/>
**Draft-Version:** 1.0<br/>
**Draft-Discuss:** devel@lists.tockos.org<br/>

Abstract
-------------------------------

This TRD specifies the Tock system call ABI for non-CHERI 64-bit RISC-V systems.

1 Introduction
====================================================================

[TRD 104](trd104-syscalls.md) defines the base Tock system calls and their
semantics as well as their ABI on 32-bit ARM Cortex-M and 32-bit RISC-V RV32I
platforms. This document extends that TRD to 64-bit non-CHERI RISC-V platforms.
In general, 64-bit RISC-V has the same basic set of system calls and semantics,
but many data types in the ABI are different from the 32-bit case.

2 System Call ABI
====================================================================

This section defines the system call ABI for non-CHERI 64-bit RISC-V systems,
including the sizes of all values.

2.1 Registers
====================================================================

_Modifies TRD 104 [section 3.1](#31-registers)_

The register mapping remains the same as in TRD 104:

|                       | Registers | Type     |
|-----------------------|-----------|----------|
| Syscall Arguments     |   a0-a3   | _varies_ |
| Syscall Return Values |   a0-a3   | _varies_ |
| Syscall Class ID      |   a4      | `u8`     |
| Upcall Arguments      |   a0-a3   | TODO     |
| Upcall Return Values  |   None    | None     |

The example `command` signature is updated as follows:

```
fn command(&self, minor_num: u32, TODO: TODO, TODO: TODO, caller_id: AppId) -> Result<(), ErrorCode>
```



NUMBER_TODO Implementation
====================================================================

TODO

NUMBER_TODO Author's Address
====================================================================
Johnathan Van Why <jrvanwhy@betterbytes.org>
