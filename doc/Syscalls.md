Syscalls
========

This document explains how [system
calls](https://en.wikipedia.org/wiki/System_call) work in Tock with regards to
both the kernel and applications. [TRD104](reference/trd104-syscalls.md)
contains the more formal specification of the system call API and ABI for 32-bit
systems. This document describes the considerations behind the system call
design.

<!-- npm i -g markdown-toc; markdown-toc -i Syscalls.md -->

<!-- toc -->

- [Overview of System Calls in Tock](#overview-of-system-calls-in-tock)
- [Tock System Call Types](#tock-system-call-types)
  * [System Call Descriptions](#system-call-descriptions)
- [Data Movement Between Userspace and Kernel](#data-movement-between-userspace-and-kernel)
  * [Userspace → Kernel](#userspace-%E2%86%92-kernel)
  * [Kernel → Userspace](#kernel-%E2%86%92-userspace)
- [System Call Implementations](#system-call-implementations)
  * [Context Switch Interface](#context-switch-interface)
  * [Cortex-M Architecture Details](#cortex-m-architecture-details)
  * [RISC-V Architecture Details](#risc-v-architecture-details)
- [Upcalls](#upcalls)
  * [Process Startup](#process-startup)
- [How System Calls Connect to Capsules (Drivers)](#how-system-calls-connect-to-capsules-drivers)
- [Identifying Syscalls](#identifying-syscalls)
  * [Syscall Class](#syscall-class)
  * [Driver Numbers](#driver-numbers)
  * [Syscall-Specific Numbers](#syscall-specific-numbers)
- [Identifying Error and Return Types](#identifying-error-and-return-types)
  * [Naming Conventions](#naming-conventions)
  * [Type Descriptions](#type-descriptions)

<!-- tocstop -->

## Overview of System Calls in Tock

System calls are the method used to send information from applications to the
kernel. Rather than directly calling a function in the kernel, applications
trigger a context switch to the kernel. The kernel then uses the values in
registers and the stack at the time of the interrupt call to determine how to
route the system call and which driver function to call with which data values.

Using system calls has three advantages. First, the act of triggering a service
call interrupt can be used to change the processor state. Rather than being in
unprivileged mode (as applications are run) and limited by the Memory Protection
Unit (MPU), after the service call the kernel switches to privileged mode where
it has full control of system resources (more detail on ARM [processor
modes](http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/CHDIGFCA.html)).

Second, context switching to the kernel allows it to do other resource handling
before returning to the application. This could include running other
applications, servicing queued upcalls, or many other activities.

Finally, and most importantly, using system calls allows applications to be
built independently from the kernel. The entire codebase of the kernel could
change, but as long as the system call interface remains identical, applications
do not even need to be recompiled to work on the platform. Applications, when
separated from the kernel, no longer need to be loaded at the same time as the
kernel. They could be uploaded at a later time, modified, and then have a new
version uploaded, all without modifying the kernel running on a platform.

## Tock System Call Types

Tock has 7 general types (i.e. "classes") of system calls:

| Syscall Class    |
|------------------|
| Yield            |
| Subscribe        |
| Command          |
| Read-Write Allow |
| Read-Only Allow  |
| Memop            |
| Exit             |

All communication and interaction between applications and the kernel uses only
these system calls.

Within these system calls, there are two general groups of syscalls:
administrative and capsule-specific.

1. **Administrative Syscalls**: These adjust the execution or resources of the
   running process, and are handled entirely by the core kernel. These calls
   always behave the same way no matter which kernel resources are exposed to
   userspace. This group includes:
   - `Yield`
   - `Memop`
   - `Exit`

2. **Capsule-Specific Syscalls**: These interact with specific capsules (i.e.
   kernel modules). While the general semantics are the same no matter the
   underlying capsule or resource being accessed, the actual behavior of the
   syscall depends on which capsule is being accessed. For example, a command to
   a timer capsule might start a timer, whereas a command to a temperature
   sensor capsule might start a temperature measurement. This group includes:
   - `Subscribe`
   - `Command`
   - `Read-Write Allow`
   - `Read-Only Allow`

All Tock system calls are synchronous, which means they immediately return to
the application. Capsules must not implement long-running operations by blocking
on a command system call, as this prevents other applications or kernel routines
from running – kernel code is never preempted.

### System Call Descriptions

This provides an introduction to each type of Tock system call. These are
described in much more detail in [TRD104](reference/trd104-syscalls.md).

- `Yield`: An application yields its execution back to the kernel. The kernel
  will only trigger an upcall for a process after it has called yield.

- `Memop`: This group of "memory operations" allows a process to adjust its
  memory break (i.e. request more memory be available for the process to use),
  learn about its memory allocations, and provide debug information.

- `Exit`: An application can call exit to inform the kernel it no longer needs
  to execute and its resources can be freed. This also lets the process request
  a restart.

- `Subscribe`: An application can issue a subscribe system call to register
  upcalls, which are functions being invoked in response to certain events.
  These upcalls are similar in concept to UNIX signal handlers. A driver can
  request an application-provided upcall to be invoked. Every system call driver
  can provide multiple "subscribe slots", each of which the application can
  register a upcall to.

- `Command`: Applications can use command-type system calls to signal arbitrary
  events or send requests to the userspace driver. A common use-case for
  command-style systems calls is, for instance, to request that a driver start
  some long-running operation.

- `Read-only Allow`: An application may expose some data for drivers to read.
  Tock provides the read-only allow system call for this purpose: an application
  invokes this system call passing a buffer, the contents of which are then made
  accessible to the requested driver. Every driver can have multiple "allow
  slots", each of which the application can place a buffer in.

- `Read-write Allow`: Works similarly to read-only allow, but enables drivers to
  also mutate the application-provided buffer.

## Data Movement Between Userspace and Kernel

All data movement and communication between userspace and the kernel happens
through syscalls. This section describes the general mechanisms for data
movement that syscalls enable. In this case, we use "data" to be very general
and describe any form of information transfer.

### Userspace → Kernel

Moving data from a userspace application to the kernel happens in two forms.

1. Instruction with simple options. Applications often want to instruct the
   kernel to take some action (e.g. play a sound, turn on an LED, or take a
   sensor reading). Some of these may require small amounts of configuration
   (e.g. which LED, or the resolution of the sensor reading). This data transfer
   is possible with the `Command` syscall.

   There are two important considerations for `Command`. First, the amount of
   data that can be transferred for configuration is on the order of 32 bits.
   Second, `Command` is non-blocking, meaning the `Command` syscall will finish
   before the requested operation completes.

2. Arbitrary buffers of data. Applications often need to pass data to the kernel
   for the kernel to use it for some action (e.g. audio samples to play, data
   packets to transmit, or data buffers to encrypt). This data transfer is
   possible with the "allow" family of syscalls, specifically the `Read-only
   allow`.

   Once an application shares a buffer with the kernel via allow, the process
   should not use that buffer until it has "un-shared" the buffer with the
   kernel.

### Kernel → Userspace

Moving data from the kernel to a userspace application to the kernel happens in
three ways.

1. Small data that is synchronously available. The kernel may have status
   information or fixed values it can send to an application (e.g. how many
   packets have been sent, or the maximum resolution of an ADC). This can be
   shared via the return value to a `Command` syscall. An application must call
   the `Command` syscall, and the return value must be immediately available,
   but the kernel can provide about 12 bytes of data back to the application via
   the return value to the command syscall.

2. Arbitrary buffers of data. The kernel may have more data to send to
   application (e.g. an incoming data packet, or ADC readings). This data can be
   shared with the application by filling in a buffer the application has
   already shared with the kernel via an allow syscall. For the kernel to be
   able to modify the buffer, the application must have called the `Read-write
   allow` syscall.

3. Events with small amounts of data. The kernel may need to notify an
   application about a recent event or provide small amounts of new data (e.g. a
   button was pressed, a sensor reading is newly available, or a incoming packet
   has arrived). This is accomplished by the kernel issuing an "upcall" to the
   application. You can think of an upcall as a callback, where when the process
   resumes running it executes a particular function provided with particular
   arguments.

   For the kernel to be able to trigger an upcall, the process must have first
   called `Subscribe` to pass the address of the function the upcall will
   execute.

   The kernel can pass a few arguments (roughly 12 bytes) with the upcall. This
   is useful for providing small amounts of data, like a reading sensor reading.

## System Call Implementations

All system calls are implemented via context switches. A couple values are
passed along with the context switch to indicate the type and manor of the
syscall. A process invokes a system call by triggering context switch via a
software interrupt that transitions the microcontroller to supervisor/kernel
mode. The exact mechanism for this is architecture-specific.
[TRD104](reference/trd104-syscalls.md) specifies how userspace and the kernel
pass values to each other for Cortex-M and RV32I platforms.

Handling a context switch is one of the few pieces of architecture-specific Tock
code. The code is located in `lib.rs` within the `arch/` folder under the
appropriate architecture. As this code deals with low-level functionality in the
processor it is written in assembly wrapped as Rust function calls.

### Context Switch Interface

The architecture crates (in the `/arch` folder) are responsible for implementing
the `UserspaceKernelBoundary` trait which defines the functions needed to allow
the kernel to correctly switch to userspace. These functions handle the
architecture-specific details of how the context switch occurs, such as which
registers are saved on the stack, where the stack pointer is stored, and how
data is passed for the Tock syscall interface.

### Cortex-M Architecture Details

Starting in the kernel before any application has been run but after the process
has been created, the kernel calls `switch_to_user`. This code sets up registers
for the application, including the PIC base register and the process stack
pointer, then triggers a service call interrupt with a call to `svc`. The `svc`
handler code automatically determines if the system desired a switch to
application or to kernel and sets the processor mode. Finally, the `svc` handler
returns, directing the PC to the entry point of the app.

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

The RISC-V architecture provides very lean support for context switching,
providing significant flexibility in software on how to support context
switches. The hardware guarantees the following will happen during a context
switch: when switching from kernel mode to user mode by calling the `mret`
instruction, the PC is set to the value in the `mepc` CSR, and the privilege
mode is set to the value in the `MPP` bits of the `mstatus` CSR. When switching
from user mode to kernel mode using the `ecall` instruction, the PC of the
`ecall` instruction is saved to the `mepc` CSR, the correct bits are set in the
`mcause` CSR, and the privilege mode is restored to machine mode. The kernel can
store 32 bits of state in the `mscratch` CSR.

Tock handles context switching using the following process. When switching to
userland, all register contents are saved to the kernel's stack. Additionally, a
pointer to a per-process struct of stored process state and the PC of where in
the kernel to resume executing after the process switches back to kernel mode
are stored to the kernel's stack. Then, the PC of the process to start executing
is put into the `mepc` CSR, the kernel stack pointer is saved in `mscratch`, and
the previous contents of the app's registers from the per-process stored state
struct are copied back into the registers. Then `mret` is called to switch to
user mode and begin executing the app.

An application calls a system call with the `ecall` instruction. This causes the
trap handler to execute. The trap handler checks `mscratch`, and if the value is
nonzero then it contains the stack pointer of the kernel and this trap must have
happened while the system was executing an application. Then, the kernel stack
pointer from `mscratch` is used to find the pointer to the stored state struct,
and all process registers are saved. The trap handler also saves the process PC
from the `mepc` CSR and the `mcause` CSR. It then loads the kernel address of
where to resume the context switching code to `mepc` and calls `mret` to exit
the trap handler. Back in the context switching code, the kernel restores its
registers from its stack. Then, using the contents of `mcause` the kernel
decides why the application stopped executing, and if it was a system call which
one it is. Returning the context switch reason ends the context switching
process.

All values for the system call functions are passed in registers `a0-a4`. No
values are stored to the application stack. The return value for system call is
set in a0.  In most system calls the kernel will not clobber any userspace
registers except for this return value register (`a0`). However, the `yield()`
system call results in a upcall executing in the process. This can clobber all
caller saved registers, as well as the return address (`ra`) register.

## Upcalls

The kernel can signal events to userspace via upcalls. Upcalls run a function in
userspace after a context switch. The kernel, as part of the upcall, provides
four 32 bit arguments. The address of the function to run is provided via the
`Subscribe` syscall.

### Process Startup

Upon process initialization, the kernel starts executing a process by running an
upcall to the process's entry point. A single function call task is added to the
process's upcall queue. The function is determined by the ENTRY point in the
process TBF header (typically the `_start` symbol) and is passed the following
arguments in registers `r0` - `r3`:

- `r0`: the base address of the process code
- `r1`: the base address of the processes allocated memory region
- `r2`: the total amount of memory in its region
- `r3`: the current process memory break


## How System Calls Connect to Capsules (Drivers)

After a system call is made, the call is handled and routed by the Tock kernel
in [`sched.rs`](../kernel/src/kernel.rs) through a series of steps.

1. For `Command`, `Subscribe`, `Read-Write Allow`, and `Read-Only Allow` system
   calls, the kernel calls a platform-defined system call filter function. This
   function determines if the kernel should handle the system call or not.
   `Yield`, `Exit`, and `Memop` system calls are not filtered. This filter
   function allows the kernel to impose security policies that limit which
   system calls a process might invoke. The filter function takes the system
   call and which process issued the system call to return a `Result<(),
   ErrorCode>` to signal if the system call should be handled or if an error
   should be returned to the process. If the filter function disallows the
   system call it returns `Err(ErrorCode)` and the `ErrorCode` is provided to
   the process as the return code for the system call. Otherwise, the system
   call proceeds. _The filter interface is unstable and may be changed in the
   future._

2. The kernel scheduler loop handles the `Exit` and `Yield` system calls.

3. To handle `Memop` system calls, the scheduler loop invokes the `memop`
   module, which implements the Memop class.

4. `Command`, `Subscribe`, `Read-Write Allow`, and `Read-Only Allow` follow a more
   complex execution path because are implemented by drivers. To route these
   system calls, the scheduler loop calls a struct that implements the
   `SyscallDriverLookup` trait. This trait has a `with_driver()` function that
   the driver number as an argument and returns either a reference to the
   corresponding driver or `None` if it is not installed. The kernel uses the
   returned reference to call the appropriate system call function on that
   driver with the remaining system call arguments.

   An example board that implements the `SyscallDriverLookup` trait looks
   something like this:

   ```rust
   struct TestBoard {
       console: &'static Console<'static, usart::USART>,
   }

   impl SyscallDriverLookup for TestBoard {
       fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
           where F: FnOnce(Option<&kernel::syscall::SyscallDriver>) -> R
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

## Identifying Syscalls

A series of numbers and conventions identify syscalls as they pass via a context
switch.

### Syscall Class

The first identifier specifies which syscall it is. The values are specified as
in the table and are fixed by convention.

| Syscall Class    | Syscall Class Number |
|------------------|----------------------|
| Yield            |           0          |
| Subscribe        |           1          |
| Command          |           2          |
| Read-Write Allow |           3          |
| Read-Only Allow  |           4          |
| Memop            |           5          |
| Exit             |           6          |

### Driver Numbers

For capsule-specific syscalls, the syscall must be directed to the correct
capsule (driver). The `with_driver()` function takes an argument `driver_num` to
identify the driver.

To enable the kernel and userspace to agree, we maintain a
[list](https://github.com/tock/tock/blob/master/capsules/core/src/driver.rs) of
known driver numbers.

To support custom capsules and driver, a `driver_num` whose highest bit is set
is private and can be used by out-of-tree drivers.

### Syscall-Specific Numbers

For each capsule/driver, the driver can support more than one of each syscall
(e.g. it can support multiple commands). Another number included in the context
switch indicates which of the syscall the call refers to.

For the `Command` syscall, the `command_num` 0 is reserved as an existence
check: userspace can call a command for a driver with `command_num` 0 to check
if the driver is installed on the board. Otherwise, the numbers are entirely
driver-specific.

For `Subscribe`, `Read-only allow`, and `Read-write allow`, the numbers start at
0 and increment for each defined use of the various syscalls. There cannot be a
gap between valid subscribe or allow numbers. The actual meaning of each
subscribe or allow number is driver-specific.

## Identifying Error and Return Types

Tock includes some defined types and conventions for errors and return values
between the kernel and userspace. These allow the kernel to indicate success and
failure to userspace.

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
