Syscalls
========

This document explains how [system
calls](https://en.wikipedia.org/wiki/System_call) work in Tock with regards
to both the kernel and applications. [TRD104](reference/trd104-syscalls.md) contains
the more formal specification
of the system call API and ABI for 32-bit systems.
This document describes the considerations behind the system call design.

<!-- toc -->

- [Overview of System Calls in Tock](#overview-of-system-calls-in-tock)
- [Process State](#process-state)
- [Startup](#startup)
- [System Call Invocation](#system-call-invocation)
- [The Context Switch](#the-context-switch)
  * [Context Switch Interface](#context-switch-interface)
  * [Cortex-M Architecture Details](#cortex-m-architecture-details)
  * [RISC-V Architecture Details](#risc-v-architecture-details)
- [How System Calls Connect to Drivers](#how-system-calls-connect-to-drivers)
- [Error and Return Types](#error-and-return-types)
  * [Naming Conventions](#naming-conventions)
  * [Type Descriptions](#type-descriptions)
- [Allocated Driver Numbers](#allocated-driver-numbers)

<!-- tocstop -->

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
applications, servicing queued upcalls, or many other activities.

Finally,
and most importantly, using system calls allows applications to be built
independently from the kernel. The entire codebase of the kernel could change,
but as long as the system call interface remains identical, applications do not
even need to be recompiled to work on the platform. Applications, when
separated from the kernel, no longer need to be loaded at the same time as the
kernel. They could be uploaded at a later time, modified, and then have a new
version uploaded, all without modifying the kernel running on a platform.

## Process State

In Tock, a process can be in one of seven states:

- **Running**: Normal operation. A Running process is eligible to be scheduled
  for execution, although is subject to being paused by Tock to allow interrupt
  handlers or other processes to run. During normal operation, a process remains
  in the Running state until it explicitly yields. Upcalls from other kernel
  operations are not delivered to Running processes (i.e. upcalls do not
  interrupt processes), rather they are enqueued until the process yields.
- **Yielded**: Suspended operation. A Yielded process will not be scheduled by
  Tock. Processes often yield while they are waiting for I/O or other operations
  to complete and have no immediately useful work to do. Whenever the kernel
  issues an upcall to a Yielded process, the process is transitioned to the
  Running state.
- **Fault**: Erroneous operation. A Fault-ed process will not be scheduled by
  Tock. Processes enter the Fault state by performing an illegal operation, such
  as accessing memory outside of their address space.
- **Terminated** The process ended itself by calling the `Exit` system call and
  the kernel has not restarted it.
- **Unstarted** The process has not yet started; this state is typically very
  short-lived, between process loading and it started. However, in cases when
  processes might be loaded for a long time without running, this state might be
  long-lived.
- **StoppedRunning**, **StoppedYielded** These states correspond to a process
  that was in either the Running or Yielded state but was then explicitly
  stopped by the kernel (e.g., by the process console). A process in these
  states will not be made runnable until it is restarted, at which point it will
  continue execution where it was stopped.

## Startup

Upon process initialization, a single function call task is added to its
upcall queue. The function is determined by the ENTRY point in the process
TBF header (typically the `_start` symbol) and is passed the following
arguments in registers `r0` - `r3`:

  * r0: the base address of the process code
  * r1: the base address of the processes allocated memory region
  * r2: the total amount of memory in its region
  * r3: the current process memory break

## System Call Invocation

A process invokes a system call by triggering a software interrupt that
transitions the microcontroller to supervisor/kernel mode. The exact
mechanism for this is architecture-specific. [TRD104](reference/trd104-syscalls.md)
specifies how userspace and the kernel pass values to each other for
CortexM and RISCV32I platforms.

## The Context Switch

Handling a context switch is one of the few pieces of architecture-specific
Tock code. The code is located
in `lib.rs` within the `arch/` folder under the appropriate architecture. As
this code deals with low-level functionality in the processor it is written in
assembly wrapped as Rust function calls.

### Context Switch Interface

The architecture crates (in the `/arch` folder) are responsible for implementing
the `UserspaceKernelBoundary` trait which defines the functions needed to allow the
kernel to correctly switch to userspace. These functions handle the
architecture-specific details of how the context switch occurs, such as which
registers are saved on the stack, where the stack pointer is stored, and how
data is passed for the Tock syscall interface.

### Cortex-M Architecture Details

Starting in the kernel before any application has been run but after the
process has been created, the kernel calls `switch_to_user`. This code sets up
registers for the application, including the PIC base register and the process
stack pointer, then triggers a service call interrupt with a call to `svc`.
The `svc` handler code automatically determines if the system desired a switch
to application or to kernel and sets the processor mode. Finally, the `svc`
handler returns, directing the PC to the entry point of the app.

The application runs in unprivileged mode while executing. When it needs to use
a kernel resource it issues a syscall by running `svc` instruction. The
`svc_handler` determines that it should switch to the kernel from an app, sets
the processor mode to privileged, and returns. Since the stack has changed to
the kernel's stack pointer (rather than the process stack pointer), execution
returns to `switch_to_user` immediately after the `svc` that led to the
application starting. `switch_to_user` saves registers and returns to the kernel
so the system call can be processed.

On the next `switch_to_user` call, the application will resume execution based
on the process stack pointer, which points to the instruction after the system
call that switched execution to the kernel.

Syscalls may clobber userspace memory, as the kernel may write to buffers
previously given to it using Allow. The kernel will not clobber any userspace
registers except for the return value register (`r0`). However, Yield must be
treated as clobbering more registers, as it can call an upcall in userspace
before returning. This upcall can clobber r0-r3, r12, and lr. See [this
comment](https://github.com/tock/libtock-c/blob/f5004277ec88c2afe8f473a06b74aa2faba70d68/libtock/tock.c#L49)
in the libtock-c syscall code for more information about Yield.

### RISC-V Architecture Details

Tock assumes that a RISC-V platform that supports context switching has two
privilege modes: machine mode and user mode.

The RISC-V architecture provides very lean support for context
switching, providing significant flexibility in software on how to
support context switches. The hardware guarantees the following will
happen during a context switch: when switching from kernel mode to
user mode by calling the `mret` instruction, the PC is set to the
value in the `mepc` CSR, and the privilege mode is set to the value in
the `MPP` bits of the `mstatus` CSR. When switching from user mode to
kernel mode using the `ecall` instruction, the PC of the `ecall`
instruction is saved to the `mepc` CSR, the correct bits are set in
the `mcause` CSR, and the privilege mode is restored to machine
mode. The kernel can store 32 bits of state in the `mscratch` CSR.

Tock handles context switching using the following process. When
switching to userland, all register contents are saved to the kernel's
stack. Additionally, a pointer to a per-process struct of stored
process state and the PC of where in the kernel to resume executing
after the process switches back to kernel mode are stored to the
kernel's stack. Then, the PC of the process to start executing is put
into the `mepc` CSR, the kernel stack pointer is saved in `mscratch`,
and the previous contents of the app's registers from the per-process
stored state struct are copied back into the registers. Then `mret` is
called to switch to user mode and begin executing the app.

An application calls a system call with the `ecall` instruction. This
causes the trap handler to execute. The trap handler checks
`mscratch`, and if the value is nonzero then it contains the stack
pointer of the kernel and this trap must have happened while the
system was executing an application. Then, the kernel stack pointer
from `mscratch` is used to find the pointer to the stored state
struct, and all process registers are saved. The trap handler also
saves the process PC from the `mepc` CSR and the `mcause` CSR. It then
loads the kernel address of where to resume the context switching code
to `mepc` and calls `mret` to exit the trap handler. Back in the
context switching code, the kernel restores its registers from its
stack. Then, using the contents of `mcause` the kernel decides why the
application stopped executing, and if it was a system call which one it is.
Returning the context switch reason ends the
context switching process.

All values for the system call functions are passed in registers
`a0-a4`. No values are stored to the application stack. The return
value for system call is set in a0.  In most system calls the kernel will not
clobber any userspace registers except for this return value register
(`a0`). However, the `yield()` system call results in a upcall executing
in the process. This can clobber all caller saved registers, as well
as the return address (`ra`) register.

## How System Calls Connect to Drivers

After a system call is made, the call is handled and routed by the Tock kernel
in [`sched.rs`](../kernel/src/kernel.rs) through a series of steps.

1. For Command, Subscribe, Allow Read-Write and Allow Read-Only system
call classes, the kernel calls a platform-defined system call filter
function. This function determines if the kernel should handle the
system call or not. Yield, Exit, and Memop system calls are not
filtered. This filter function allows the kernel to impose security
policies that limit which system calls a process might invoke. The
filter function takes the system call and which process issued the system call
to return a `Result((), ErrorCode)` to signal if the system call should be
handled or if an error should be returned to the process. If the
filter function disallows the system call it returns `Err(ErrorCode)` and
the `ErrorCode` is provided to the process as the return code for the
system call. Otherwise, the system call proceeds. _The filter interface is
unstable and may be changed in the future._

2. The kernel scheduler loop handles the Exit and Yield system call classes.

3. To handle Memop system calls, the scheduler loop invokes the `memop` module,
which implements the Memop class.

4. Allow Read-Write, Allow Read-Only, Subscribe, and Command follow a
more complex execution path because are implemented by drivers.  To
route these system calls, the scheduler loop calls a struct that
implements the `Platform` trait. This trait has a `with_driver()`
function that the driver number as an argument and returns either a
reference to the corresponding driver or `None` if it is not
installed. The kernel uses the returned reference to call the
appropriate system call function on that driver with the remaining
system call arguments.

An example board that implements the `Platform` trait looks something like
this:

   ```rust
   struct TestBoard {
       console: &'static Console<'static, usart::USART>,
   }

   impl Platform for TestBoard {
       fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
           where F: FnOnce(Option<&kernel::Driver>) -> R
       {

           match driver_num {
               0 => f(Some(self.console)), // use capsules::console::DRIVER_NUM rather than 0 in real code
               _ => f(None),
           }
       }
   }
   ```

`TestBoard` then supports one driver, the UART console, and maps it to driver
number 0. Any `command`, `subscribe`, and `allow` sycalls to driver number 0
will get routed to the console, and all other driver numbers will return
`Err(ErrorCode::NODEVICE)`.

## Error and Return Types

Tock includes some defined types and conventions for errors and return values
between the kernel and userspace.

### Naming Conventions

- `*Code` (e.g. `ErrorCode`, `StatusCode`): These types are mappings between
  numeric values and semantic meanings. These can always be encoded in a
  `usize`.
- `*Return` (e.g. `SyscallReturn`): These are more complex return types that can
  include arbitrary values, errors, or `*Code` types.

### Type Descriptions

- `*Code` Types:
  - `ErrorCode`: A standard set of errors and their numeric representations in
    Tock.  This is used to represent errors for syscalls, and elsewhere in the
    kernel.
  - `StatusCode`: All errors in `ErrorCode` plus a Success value (represented by
    0). This is used to pass a success/error status between the kernel and
    userspace.

    `StatusCode` is a pseudotype that is not actually defined as a concrete Rust
    type. Instead, it is always encoded as a `usize`. Even though it is not a
    concrete type, it is useful to be able to return to it conceptually, so we
    give it the name `StatusCode`.

    The intended use of `StatusCode` is to convey success/failure to userspace
    in upcalls. To try to keep things simple, we use the same numeric
    representations in `StatusCode` as we do with `ErrorCode`.

- `*Return` Types:
  - `SyscallReturn`: The return type for a syscall. Includes whether the syscall
    succeeded or failed, optionally additional data values, and in the case of
    failure an `ErrorCode`.

## Allocated Driver Numbers

All documented drivers are in the [doc/syscalls](syscalls/README.md) folder.

The `with_driver()` function takes an argument `driver_num` to identify the
driver. `driver_num` whose highest bit is set is private and can be used by
out-of-tree drivers.
