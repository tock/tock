Userspace Readable Allow System Call
========================================

**TRD:** XXX <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Alistair Francis <br/>
**Draft-Created:** June 17, 2021<br/>
**Draft-Modified:** Sep 8, 2021<br/>
**Draft-Version:** 2<br/>
**Draft-Discuss:** devel@lists.tockos.org</br>

Abstract
-------------------------------

This document describes the userspace readable allow system call application binary
interface (ABI) between user space processes and the Tock kernel for 32-bit
ARM Cortex-M and RISC-V RV32I platforms.

This is an extension on the allow calls specified in
[TRD 104](trd104-syscalls.md).

1 Introduction
==============

In normal use, userspace does not access a buffer that
has been userspace readable with the kernel with a Read-Write Allow call. This
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

Instead, capsules that support the userspace readable allow call can communicate with
applications without buffers needing to be re-allowed. For example a capsule
might want to report statistics to a userspace app. It could do this by letting
the app perform a userspace readable allow call to allocate a buffer. Then the capsule can
write statistics to the buffer and at any time the app can read the statistics
from the buffer.

The userspace readable allow system call allows userspace to have read-only access
a buffer that is writeable by the kernel.

2 System Call API
=================================

2.1 Userspace Readable Allow (Class ID: 7)
---------------------------------
The userspace readable allow syscall follows the same expectations and
requirements as described for the Read-Write syscall in
[TRD104 Section 4.4](trd104-syscalls.md#44-read-write-allow-class-id-3), with
the exception that apps are explicitly allowed to read buffers that have
been passed to the kernel.

The register arguments for Userspace Readable Allow system calls are as
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

The return variants for Userspace Readable Allow system calls are `Failure
with 2 u32` and `Success with 2 u32`.  In both cases, `Argument 0`
contains an address and `Argument 1` contains a length. When a driver
implementing the Userspace Readable Allow system call returns a failure
result, it MUST return the same address and length as those that were passed
in the call. When a driver implementing the Userspace Readable Allow system call
returns a success result, the returned address and length MUST be those
that were passed in the previous call, unless this is the first call.
On the first successful invocation of a particular Userspace Readable Allow system
call, a driver implementation MUST return address 0 and size 0.

The syscall class ID is shown below:

| Syscall Class            | Syscall Class Number |
|--------------------------|----------------------|
| Userspace Readable Allow |           7          |

The standard access model for userspace readable allowed buffers is that userspace can read
from a buffer while the kernel can read or write. Synchronisation
methods are required to ensure data consistency but are implementation specific.

Simultaneous accesses to a buffer from both userspace and the kernel can
cause userspace to read inconsistent data if not implemented properly.
For example a userspace app could read partially written data.
This would result in obscure timing bugs that are hard to detect. Due to this
each capsule using the userspace readable allow mechanism MUST document, in a Draft or
Final Documentary TRD, how it ensures userspace always reads consistent data
from a userspace readable buffer.

Finally, because a process conceptually relinquishes write access to a buffer
when it makes a userspace readable allow call with it, a userspace API MUST NOT
assume or rely on a process writing an allowed buffer. If userspace needs to
write to a buffer held by the kernel, it MUST first regain access to it by
calling the corresponding Userspace Readable Allow. A userspace API MAY
allow a process to read an allowed buffer, but if it does, it must document
a consistency mechanism.

One example approach to ensure that userspace reads of a data object are
consistent is to use a monotonic counter. Every time the kernel writes
the data object, it increments the counter. If userspace reads the counter,
reads the data object, then reads the counter again to check that it has not
changed, it can check that the object was not modified mid-read. If the counter
changes, it restarts the read of the data object. This approach is simple, but
does make reading the data object take variable time and is theoretically
vulnerable to starvation.

An example of reading a monotonic counter from userspace would look like this:

```c
  // Reference to the readable-allow'd buffer
  volatile uint32_t* ptr;

  do {
    // Read the current counter value
    counter = ptr[0];

    // Read in the data
    my_data0 = ptr[1];
    my_data1 = ptr[2];

    // Only exit the loop if counter and ptr[0] are the same
  } while (counter != ptr[0]);
```

where the counter is incremented on every context switch to userspace.

3 libtock-c Userspace Library Methods
=====================================

3.1 Userspace Readable Allow
---------------------------------

The userspace readable allow system call class is how a userspace process
shares a buffer with the kernel that the kernel can read and write.

The userspace readable allow system call has this function prototype:

```c
typedef struct {
  bool success;
  void* ptr;
  size_t size;
  tock_error_t error;
} userspace_readable_allow_return_t;

userspace_readable_allow_return_t allow_userspace_readable(uint32_t driver, uint32_t allow, volatile void* ptr, size_t size);
```

The `success` field indicates whether the call succeeded.
If it failed, the error code is stored in `error`. If it succeeded,
the value in `error` is undefined. `ptr` and `size` contain the pointer
and size of the passed buffer.

The register arguments for Userspace Readable Allow system calls are as
follows. The registers r0-r3 correspond to r0-r3 on CortexM and a0-a3
on RISC-V.

| Argument         | Register |
|------------------|----------|
| Driver number    | r0       |
| Buffer number    | r1       |
| Address          | r2       |
| Size             | r3       |

4 Author's Address
=================================
Alistair Francis
alistair.francis@wdc.com
