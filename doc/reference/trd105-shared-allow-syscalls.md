Shared Allow System Call
========================================

**TRD:** 105 <br/>
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

This is an extension on the allow calls specified in TRD 105.

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

This allows capsules that support the shared allow call to communicte with
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
as described for the Read-Write syscall in section TRD104 4.4, with the
exceptionthat apps are explicity allowed to read/write buffers that have
been passed to the kernel.

The standard calling pattern for reading data from the Tock kernel is to
  1. use `subscribe` to register a callback,
  2. make a Read-Write Allow call to share a buffer with the kernel,
  3. call a `command` to start an operation that writes the allowed buffer,
  4. in the upcall that signals the operation completes, make another
  Read-Write Allow call to reclaim the buffer shared with the kernel.

As simultaneous accesses to a buffer from both userspace and the kernel can
cause issues if not implemented properly, each capsule using the Shared Allow
mechanism MUST document, in a Draft or Final Documentary TRD, how consistency
of the data exchanged through this shared memory region is achieved.
Examples for mechanisms to ensure consistent data reads/writes can be driver state
machines, precisely specifying when writes will be issued to the buffer by the
kernel, or monotonically increasing counters along the data for userspace to ensure
a read operation yielded consistent data.

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
