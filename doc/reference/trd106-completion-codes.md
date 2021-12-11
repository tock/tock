Application Completion Codes
========================================

**TRD:** 106 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft<br/>
**Author:** Alyssa Haroldsen<br/>
**Draft-Created:** December 6, 2021<br/>
**Draft-Modified:** December 7, 2021<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------
This advisory document describes the expected behavior of application completion
codes when terminating via the `exit` syscall, as described in
[TRD 104][exit-syscall].

1 Introduction
===============================
When an application exits via the [`exit` syscall][exit-syscall], it can specify
a **completion code**, an unsigned 32-bit number which indicates status. This
information can be stored in the kernel and used in management or policy
decisions.

This number is called an "exit status", "exit code", or "result code" on other
platforms.

2 Design Considerations
===============================
When possible, Tock applications should follow existing conventions and
terminology from other major platforms. This assists in helping the project be
more understandable to newcomers by following the principle of least
astonishment.

This advisory document provides guidance for the ecosystem of Tock applications
using the `exit` syscall, and does not define the behavior of the syscall
itself.

3 Design
===============================
A completion code of `0` passed to the `exit` syscall MUST indicate normal app
termination. A non-zero completion code SHOULD be used to indicate abnormal
termination. A completion code between `1` and `1024` inclusive SHOULD be the
same value as one of the error codes specified in [TRD 104][error-codes].

The kernel MAY treat zero and non-zero completion codes differently.

| **Completion Code** | **Meaning**                                   |
| ------------------- | --------------------------------------------- |
| 0                   | Success                                       |
| 1-1024              | SHOULD be a [TRD 104 error code][error-codes] |
| 1025-`u32::MAX`     | Not defined                                   |

4 Implementation
===============================
As of writing, libtock [currently implements][termination] this TRD via the
`Termination` trait.

5 Author's Address
===============================
```
Alyssa Haroldsen <kupiakos@google.com>
```

[error-codes]: https://github.com/tock/tock/blob/master/doc/reference/trd104-syscalls.md#33-error-codes
[exit-syscall]: https://github.com/tock/tock/blob/master/doc/reference/trd104-syscalls.md#47-exit-class-id-6
[termination]: https://github.com/tock/libtock-rs/blob/030e5450c9480beb8b62674e1d6795f4e1697b19/platform/src/termination.rs
