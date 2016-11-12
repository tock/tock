# Syscalls

This document explains how [system
calls](https://en.wikipedia.org/wiki/System_call) work in Tock both with
regards to the kernel and applications. This is documentation of the design
thoughts behind the current implementation of syscalls rather than a tutorial
on how to use them in drivers or applications.


## Overview of System Calls in Tock

System calls are the method used to send information from applications to the
kernel. Rather than directly calling a function in the kernel, applications
trigger a service call (`svc`) interrupt which causes a context switch to the
kernel. The kernel then uses the values in registers and the stack at the time
of the interrupt call to determine how to route the system call and which
driver function to call with which data values.

Using system calls has three advantages. First, the act of triggering a service
call interrupt can be used to change the processor state. Rather than being in
unprivileged mode (as applications are run) and limited by the Memory
Protection Unit (MPU), after the service call the kernel switches to privileged
mode where it has full control of system resources (more detail on ARM
[processor
modes](http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/CHDIGFCA.html)).
Second, context switching to the kernel allows it to do other resource handling
before returning to the application. This could include running other
applications, servicing queued callbacks, or many other activities. Finally,
and most importantly, using system call allows applications to be built
independently from the kernel. The entire codebase of the kernel could change,
but as long as the system call interface remains identical, applications do not
even need to be recompiled to work on the platform. Applications, when
separated from the kernel, no longer need to be loaded at the same time as the
kernel. They could be uploaded at a later time, modified, and then have a new
version uploaded, all without modifying the kernel running on a platform.


## The System Calls

### Command

### Subscribe

### Allow

### Yield

### Memop?!


## The Context Switch

Handling a context switch is one of the few pieces of Tock code that is
actually architecture dependent and not just chip-specific. The code is located
in `lib.rs` within the `arch/` folder under the appropriate architecture. As
this code deals with low-level functionality in the processor it is written in
assembly wrapped as Rust function calls.

Starting in the kernel before any application has been run but after the
process has been created, the kernel calls `switch_to_user`. This code sets up
registers for the application, including the PIC base register and the process
stack pointer, then triggers a service call interrupt with a call to `svc`.
The `svc` handler code automatically determines if the system desired a switch
to application or to kernel and sets the processor mode. Finally, the `svc`
handler returns, directing the PC to the entry point of the app.

The application runs in unprivileged mode performing whatever its true purpose
is until it decides to make a call to the kernel. It calls `svc`. The `svc`
handler determines that it should switch to the kernel from an app, sets the
processor mode to privileged, and returns. Since the stack has changed to the
kernel's stack pointer (rather than the process stack pointer), execution
returns to `switch_to_user` immediately after the `svc` that led to the
application starting. `switch_to_user` saves registers and returns to the
kernel so the system call can be processed.

On the next `switch_to_user` call, the application will resume execution based
on the process stack pointer, which points to the instruction after the system
call that switched execution to the kernel.

The long and short of it is that execution is handled so that the application
resumes at the next instruction after a system call is complete and the kernel
resumes operation whenever a system call is made.


## How System Calls Connect to Drivers

After a system call is made, Tock routes the call to the appropriate driver.

First, in [`sched.rs`](../kernel/src/sched.rs) the number of the `svc` is
matched against the valid syscall types. `yield` and `memop` have special
functionality that is handled by the kernel. `command`, `subscribe`, and
`allow` are routed to drivers for handling.





## Allocated Driver Numbers

 * Point to a separate document

