Application Completion Codes
========================================

**TRD:** 106 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft<br/>
**Author:** Alyssa Haroldsen<br/>
**Draft-Created:** December 6, 2021<br/>
**Draft-Modified:** January 25, 2022<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** devel@lists.tockos.org</br>

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
termination. This distinction is useful so that a Tock kernel can handle
success/failure cases differently, e.g. by printing error messages,
and so that kernel extensions (such as process exit handlers defined by a board)
or external tools (such as a tool designed to parse the output from a kernel
with *trace\_syscalls* enabled) can match on these two cases.
This behavior also matches the convention for Unix exit codes, such that it
likely matches the expectations for users coming from that domain.

A completion code between `1` and `1024` inclusive SHOULD be the
same value as one of the error codes specified in [TRD 104][error-codes].
This requirement is a SHOULD rather than a MUST because it is useful in the
common case (it allows software to infer something about the cause of an
error that led to an exit, and possibly print a useful message) but also
allows a process to do something else if needed (e.g. for compatibility
with some other standard of exit codes).

Accordingly, the core kernel MUST NOT assume any semantic meaning for completion
codes or take actions based on their values besides printing error messages
unless

- there is a specification of a particular application's completion code space
  written in a TRD, and

- the kernel can reliably identify that application and associate it with this
  specification.
While there are common and conventional uses of certain values, applications
are not required to follow these and may assign their own semantic meanings
to values.

| **Completion Code** | **Meaning**                                   |
| ------------------- | --------------------------------------------- |
| 0                   | Success                                       |
| 1-1024              | SHOULD be a [TRD 104 error code][error-codes] |
| 1025-`u32::MAX`     | Not defined                                   |

4 Implementation
===============================
As of writing, libtock [currently implements][termination] this TRD via the
`Termination` trait.

```rust
pub trait Termination {
    fn complete<S: Syscalls>(self) -> !;
}

impl Termination for () {
    fn complete<S: Syscalls>(self) -> ! {
        S::exit_terminate(0)
    }
}

impl Termination for Result<(), ErrorCode> {
    fn complete<S: Syscalls>(self) -> ! {
        let exit_code = match self {
            Ok(()) => 0,
            Err(ec) => ec as u32,
        };
        S::exit_terminate(exit_code);
    }
}
```

5 Author's Address
===============================
```
Alyssa Haroldsen <kupiakos@google.com>
Hudson Ayers <hayers@stanford.edu>
```

[error-codes]: https://github.com/tock/tock/blob/master/doc/reference/trd104-syscalls.md#33-error-codes
[exit-syscall]: https://github.com/tock/tock/blob/master/doc/reference/trd104-syscalls.md#47-exit-class-id-6
[termination]: https://github.com/tock/libtock-rs/blob/030e5450c9480beb8b62674e1d6795f4e1697b19/platform/src/termination.rs
