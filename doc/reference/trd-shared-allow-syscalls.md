Shared Allow System Call
========================================

**TRD:** XXX <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Alistair Francis <br/>
**Draft-Created:** June 17, 2021<br/>
**Draft-Modified:** June 17, 2021<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes the shared allow system call application binary
interface (ABI) between user space processes and the Tock kernel for 32-bit
ARM Cortex-M and RISC-V RV32I platforms.

This is an extension on the allow calls specified in
[TRD 104](trd104-syscalls.md).

1 Introduction
==============

In normal use, userspace does not access a buffer that
has been shared with the kernel with a Read-Write Allow call. This
reading restriction is because the contents of the buffer may be in an
intermediate state and so not consistent with expected data
models. Ensuring every system call driver maintains consistency in the
presence of arbitrary userspace reads is too great a programming
burden for an unintended use case.

However, there can be cases when it is necessary for userspace to be
able to read a buffer without first revoking it from the kernel with a
Read-Write Allow. These cases are situations when the cost of a
Read-Write Allow system call is an unacceptable overhead for
accessing the data.

Instead, capsules that support the shared allow call can communicate with
applications without buffers needing to be re-allowed. For example a capsule
might want to report statistics to a userspace app. It could do this by letting
the app perform a shared allow call to allocate a buffer. Then the capsule can
write statistics to the buffer and at any time the app can read the statistics
from the buffer.

2 System Call API
=================================

2.1 Shared Allow (Class ID: 7)
---------------------------------
The shared allow syscall follows the same expectations and requirements
as described for the Read-Write syscall in
[TRD104 Section 4.4](trd104-syscalls.md#44-read-write-allow-class-id-3), with
the exception that apps are explicitly allowed to read buffers that have
been passed to the kernel.

The register arguments for Read-Write Allow system calls are as
follows. The registers r0-r3 correspond to r0-r3 on CortexM and a0-a3
on RISC-V.

| Argument         | Register |
|------------------|----------|
| Driver number    | r0       |
| Buffer number    | r1       |
| Address          | r2       |
| Size             | r3       |

The Tock kernel MUST check that the passed buffer is contained within
the calling process's writeable address space. Every byte of a passed
buffer must be readable and writeable by the process. Zero-length
buffers may therefore have arbitrary addresses. If the passed buffer is
not complete within the calling process's writeable address space, the
kernel MUST return a failure result with an error code of `INVALID`.
The buffer number specifies which buffer this is. A driver may
support multiple allowed buffers.

The return variants for Read-Write Allow system calls are `Failure
with 2 u32` and `Success with 2 u32`.  In both cases, `Argument 0`
contains an address and `Argument 1` contains a length. When a driver
implementing the Read-Write Allow system call returns a failure
result, it MUST return the same address and length as those that were passed
in the call. When a driver implementing the Read-Write Allow system call
returns a success result, the returned address and length MUST be those
that were passed in the previous call, unless this is the first call.
On the first successful invocation of a particular Read-Write Allow system
call, an driver implementation MUST return address 0 and size 0.

The syscall class ID is shown below:

| Syscall Class    | Syscall Class Number |
|------------------|----------------------|
| Read-Write Allow |           7          |

The standard access model for shared allowed buffers is that userspace can read
from a buffer while the kernel can read or write. Syncronisation
methods are required to ensure data consistency but are implementation specifc.

Simultaneous accesses to a buffer from both userspace and the kernel can
cause issues if not implemented properly. For example a userspace app could
read partially written data or the kernel could act on partially written data.
This would result in obscure timing bugs that are hard to detect. Due to this
each capsule using the Shared Allow mechanism MUST document, in a Draft or
Final Documentary TRD, how consistency of the data exchanged through this
shared memory region is achieved.

An examples to ensure consistent data reads/writes can be a monotonically
increasing counters along with the data for userspace to ensure a read
operation yielded consistent data.

An example of reading a monotomic counter from userspace would look like this:

```c
  volatile uint32_t high, low;

  do {
    // Set the high bytes the value in memory
    high = ptr[3];
    // Read the low bytes
    low = ptr[2];
    // If the high bytes don't match what is still in memory re-try
    // the load
  } while (high != ptr[3]);
```

where the counter is incremented on every context switch to userspace.

3 libtock-c Userspace Library Methods
=====================================

3.1 Shared Allow
---------------------------------

The read-write allow system call has this function prototype:

```c
typedef struct {
  bool success;
  void* ptr;
  size_t size;
  tock_error_t error;
} allow_shared_return_t;

allow_shared_return_t allow_shared(uint32_t driver, uint32_t allow, void* ptr, size_t size);
```

The `success` field indicates whether the call succeeded.
If it failed, the error code is stored in `error`. If it succeeded,
the value in `error` is undefined. `ptr` and `size` contain the pointer
and size of the passed buffer.

4 Authors' Address
=================================
Alistair Francis
alistair.francis@wdc.com
