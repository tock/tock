RISC-V 64-bit ABI
========================================

**TRD:** <br/>
**Working Group:** Core<br/>
**Type:** Documentary<br/>
**Status:** Draft<br/>
**Authors:** Johnathan Van Why, Brad Campbell<br/>
**Extends:** [104](trd104-syscalls.md)<br/>
**Draft-Created:** 2026-06-16<br/>
**Draft-Modified:** 2026-06-16<br/>
**Draft-Version:** 1.0<br/>
**Draft-Discuss:** devel@lists.tockos.org<br/>

Abstract
-------------------------------

This TRD specifies the Tock system call ABI for 64-bit systems.

1 Introduction
====================================================================

[TRD 104](trd104-syscalls.md) defines the base Tock system calls and their
semantics as well as their ABI on 32-bit ARM Cortex-M and 32-bit RISC-V RV32I
platforms. This document extends that TRD to 64-bit platforms.
In general, 64-bit has the same basic set of system calls and semantics,
but some data types in the ABI are different from the 32-bit case.

This TRD attempts to avoid duplicating explanation, details, and design
decisions that are the same as the 32-bit ABI described in
[TRD 104](trd104-syscalls.md).

2 Scope and Compatibility
====================================================================

This specifies the ABI for non-CHERI 64-bit RISC-V systems.

2.1 Compatibility with 32-bit and Tock Capsules
====================================================================

The 64-bit ABI is designed to facilitate the compatibility of Tock capsules
with both 32-bit and 64-bit platforms. This primarily happens by not enabling
more capsule-specific data to be passed between the kernel and userspace by
virtue of the larger register size. For example, a capsule that needs to
receive 16 bytes of data on a Command system call from userspace could, on a
RISC-V 64-bit platform, use the two Command argument registers to design an
interface to pass 128-bits of data using only the Command system call
(and avoiding using an allow-ed buffer). However, the same capsule compiled
on a 32-bit platform would only be able to pass 64-bits of data using the
same mechanism, and the capsule would not be compatible with 32-bit systems.
To avoid this, this 64-bit ABI specifies the same number of usable bits are
passed in commands and upcalls.

3 System Call ABI
====================================================================

This section defines the system call ABI for non-CHERI 64-bit RISC-V systems,
including the sizes of all values.

3.1 Registers
====================================================================

_Modifies TRD 104 [section 3.1](trd104-syscalls.md#31-registers)_

The register mapping remains the same as in TRD 104:

|                       | RISC-V    | Type     |
|-----------------------|-----------|----------|
| Syscall Arguments     |   a0-a3   | _varies_ |
| Syscall Return Values |   a0-a3   | _varies_ |
| Syscall Class ID      |   a4      | `u8`     |
| Upcall Arguments      |   a0-a3   | _varies_ |
| Upcall Return Values  |   None    | None     |

In all instances where a `u32` is passed in a register the `u32` is placed in
the lowest 32-bits of the register with the upper 32-bits being zero.

3.2 Syscall Argument Types
====================================================================

| Type                         | C Analog    | Description                                                                                                    |
|------------------------------|-------------|----------------------------------------------------------------------------------------------------------------|
| `VALUE_32`                   | `uint32_t ` | Any 32-bit value.                                                                                              |
| `VALUE_64`                   | `uint64_t ` | Any 64-bit value.                                                                                              |
| `OPAQUE`                     | `uintptr_t` | Any native machine word sized type.                                                                            |
| `SIZE`                       | `size_t`    | An unsigned, numeric type with a range capable of expressing in bytes the size of any valid contiguous object. |
| `POINTER`                    | `void *`    | A type capable of holding a pointer to a valid memory location.                                                |
| `POINTER_OR_ZERO`            | `void *`    | A type capable of holding a pointer to a valid memory location OR the value `0`.                               |
| `C_FUNCTION_POINTER_OR_ZERO` |             | A type capable of holding a pointer to an executable function, OR the value `0`.                               |


3.3 Return Values
====================================================================

Return values are similar to [TRD 104](trd104-syscalls.md), but with the
following register encoding:

| System call return variant                       | `a0` | `a1`                                          | `a2`                                          | `a3`                        | Notes              |
|--------------------------------------------------|------|-----------------------------------------------|-----------------------------------------------|-----------------------------|--------------------|
| Failure                                          | 0    | `VALUE_32` - Error code                       |                                               |                             |                    |
| Failure with `u32`                               | 1    | `VALUE_32` - Error code                       | `VALUE_32` - Return Value 0                   |                             |                    |
| Failure with 2 `u32`                             | 2    | `VALUE_32` - Error code                       | `VALUE_32` - Return Value 0                   | `VALUE_32` - Return Value 1 |                    |
| Failure with `u64`                               | 3    | `VALUE_32` - Error code                       | `VALUE_64` - Return Value 0                   |                             |                    |
| Failure with upcall pointer and opaque parameter | 4    | `VALUE_32` - Error code                       | `C_FUNCTION_POINTER_OR_ZERO` - Upcall Pointer | `OPAQUE`                    | Only for subscribe |
| Failure with pointer and length                  | 5    | `VALUE_32` - Error code                       | `POINTER_OR_ZERO` - Buffer Pointer            | `SIZE` - Length             | Only for allow     |
| Success                                          | 128  |                                               |                                               |                             |                    |
| Success with `u32`                               | 129  | `VALUE_32` - Return Value 0                   |                                               |                             |                    |
| Success with 2 `u32`                             | 130  | `VALUE_32` - Return Value 0                   | `VALUE_32` - Return Value 1                   |                             |                    |
| Success with `u64`                               | 131  | `VALUE_32` - Return Value 0                   |                                               |                             |                    |
| Success with 3 `u32`                             | 132  | `VALUE_32` - Return Value 0                   | `VALUE_32` - Return Value 1                   | `VALUE_32` - Return Value 2 |                    |
| Success with `u32` and `u64`                     | 133  | `VALUE_32` - Return Value 0                   | `VALUE_64` - Return Value 1                   |                             |                    |
| Success with upcall pointer and opaque parameter | 134  | `C_FUNCTION_POINTER_OR_ZERO` - Upcall Pointer | `OPAQUE`                                      |                             | Only for subscribe |
| Success with pointer and length                  | 135  | `POINTER_OR_ZERO` - Buffer Pointer            | `SIZE` - Length                               |                             | Only for allow     |
| Success with pointer                             | 136  | `POINTER_OR_ZERO` - Memop Pointer             |                                               |                             | Only for memop     |

3.4 Upcalls
====================================================================

| Argument   | Register | Type     |
|------------|----------|----------|
| Argument 0 | a0       | `u32`    |
| Argument 1 | a1       | `u32`    |
| Argument 2 | a2       | `u32`    |
| Argument 3 | a3       | `OPAQUE` |

4 System Call API
=================================

This ABI uses the same system calls as [TRD 104](trd104-syscalls.md).

4.1 Yield (Class ID: 0)
--------------------------------

| Argument               | Register | Type     |
|------------------------|----------|----------|
| Yield number           | a0       | `u32`    |
| yield-param-A          | a1       | _varies_ |
| yield-param-B          | a2       | _varies_ |
| yield-param-C          | a3       | _varies_ |

### 4.1.1 Yield-NoWait

| Argument      | Register | Type           | Value                                            |
|---------------|----------|----------------|--------------------------------------------------|
| Yield number  | a0       | `u32`          | `0`                                              |
| yield-param-A | a1       | `*mut [u8; 1]` | Pointer to one byte of userspace memory or `0x0` |


### 4.1.2 Yield-Wait

| Argument      | Register | Type           | Value |
|---------------|----------|----------------|-------|
| Yield number  | a0       | `u32`          | `1`   |

### 4.1.3 Yield-WaitFor

| Argument      | Register | Type  | Value            |
|---------------|----------|-------|------------------|
| Yield number  | a0       | `u32` | `2`              |
| yield-param-A | a1       | `u32` | Driver number    |
| yield-param-B | a2       | `u32` | Subscribe number |


4.2 Subscribe (Class ID: 1)
--------------------------------

| Argument          | Register | Type                       |
|-------------------|----------|----------------------------|
| Driver number     | a0       | `u32`                      |
| Subscribe number  | a1       | `u32`                      |
| Upcall pointer    | a2       | C_FUNCTION_POINTER_OR_ZERO |
| Application data  | a3       | OPAQUE                     |

Valid return types:

- `Failure with upcall pointer and opaque parameter`
- `Success with upcall pointer and opaque parameter`

For success, the upcall pointer is the pointer passed in the previous call to
Subscribe (the existing upcall) and the opaque parameter is the application
data parameter passed in the previous call to Subscribe(the existing
application data). For failure, the upcall pointer is the passed upcall
pointer and the opaque parameter is the passed application data parameter.
For the first successful call to Subscribe for a given upcall, the upcall
pointer and application data parameter returned MUST be the Null Upcall.

4.3 Command (Class ID: 2)
---------------------------------

| Argument          | Register | Type                         |
|-------------------|----------|------------------------------|
| Driver number     | a0       | `u32`                        |
| Command number    | a1       | `u32`                        |
| Argument 0        | a2       | `{u32, i32, u64_lo, i64_lo}` |
| Argument 1        | a3       | `{u32, i32, u64_hi, i64_hi}` |

Note, although a command needing to pass a `u64` could pass the entire value
in a single register, without some variant indication (like with return
values), there is no way for the kernel to know the argument is a `u64` and
to ignore all contents of Argument 1. Therefore, the `u64` data must still be
split in the lower 32-bits of Argument 0 and Argument 1.

Valid return types:

- `Failure`
- `Failure with u32`
- `Failure with 2 u32`
- `Failure with u64`
- `Success`
- `Success with u32`
- `Success with 2 u32`
- `Success with u64`
- `Success with 3 u32`
- `Success with u32 and u64`

4.4 Read-Write Allow (Class ID: 3)
---------------------------------

| Argument         | Register | Type              | Additional Restrictions                                                                 |
|------------------|----------|-------------------|-----------------------------------------------------------------------------------------|
| Driver number    | a0       | `u32`             |                                                                                         |
| Allow number     | a1       | `u32`             |                                                                                         |
| Address          | a2       | `POINTER_OR_ZERO` | Pointers must refer to a contiguous array of writable userspace memory of length `{a3}` |
| Size             | a3       | `SIZE`            |                                                                                         |

Valid return types:

- `Failure with pointer and length`
- `Success with pointer and length`

In both cases, Argument 0 contains an address and Argument 1 contains a
length.

4.5 Read-Only Allow (Class ID: 4)
---------------------------------

| Argument         | Register | Type              | Additional Restrictions                                                                 |
|------------------|----------|-------------------|-----------------------------------------------------------------------------------------|
| Driver number    | a0       | `u32`             |                                                                                         |
| Allow number     | a1       | `u32`             |                                                                                         |
| Address          | a2       | `POINTER_OR_ZERO` | Pointers must refer to a contiguous array of readable userspace memory of length `{a3}` |
| Size             | a3       | `SIZE`            |                                                                                         |

Return types are the same as Read-Write Allow.

4.6 Memop (Class ID: 5)
---------------------------------

| Argument               | Register | Type     |
|------------------------|----------|----------|
| Operation              | a0       | `u32`    |
| Operation argument     | a1       | _varies_ |

Memop operations:

| Memop Operation | Operation                                               | Argument  | Success              | Failure |
|-----------------|---------------------------------------------------------|-----------|----------------------|---------|
| 0               | Break                                                   | `POINTER` | Success              | Failure |
| 1               | SBreak                                                  | `i64`     | Success with pointer | Failure |
| 2               | Get process RAM start address                           |           | Success with pointer |         |
| 3               | Get address immediately after process RAM allocation    |           | Success with pointer |         |
| 4               | Get process flash start address                         |           | Success with pointer |         |
| 5               | Get address immediately after process flash region      |           | Success with pointer |         |
| 6               | Get lowest address (end) of the grant region            |           | Success with pointer |         |
| 7               | Get number of writeable flash regions in process header |           | Success with u32     |         |
| 8               | Get start address of a writeable flash region           | `u32`     | Success with pointer | Failure |
| 9               | Get end address of a writeable flash region             | `u32`     | Success with pointer | Failure |
| 10              | Set the start of the process stack                      | `POINTER` | Success              |         |
| 11              | Set the start of the process heap                       | `POINTER` | Success              |         |

4.7 Exit (Class ID: 6)
--------------------------------

| Argument         | Register | Type  |
|------------------|----------|-------|
| Exit number      | a0       | `u32` |
| Completion code  | a1       | `u32` |


5 Author's Address
====================================================================
Johnathan Van Why <jrvanwhy@betterbytes.org>

Brad Campbell <bradjc@virginia.edu>
