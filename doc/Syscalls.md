Syscalls
========

This document explains how [system
calls](https://en.wikipedia.org/wiki/System_call) work in Tock with regards
to both the kernel and applications. This is a description of the design
considerations behind the current implementation of syscalls, rather than a
tutorial on how to use them in drivers or applications.

<!-- toc -->

- [Overview of System Calls in Tock](#overview-of-system-calls-in-tock)
- [Process State](#process-state)
- [Startup](#startup)
- [The System Calls](#the-system-calls)
  * [0: Yield](#0-yield)
    + [Arguments](#arguments)
    + [Return](#return)
  * [1: Subscribe](#1-subscribe)
    + [Arguments](#arguments-1)
    + [Return](#return-1)
  * [2: Command](#2-command)
    + [Arguments](#arguments-2)
    + [Return](#return-2)
  * [3: Allow](#3-allow)
    + [Arguments](#arguments-3)
    + [Return](#return-3)
  * [4: Memop](#4-memop)
    + [Arguments](#arguments-4)
    + [Return](#return-4)
- [The Context Switch](#the-context-switch)
  * [Context Switch Interface](#context-switch-interface)
  * [Cortex-M Architecture Details](#cortex-m-architecture-details)
- [How System Calls Connect to Drivers](#how-system-calls-connect-to-drivers)
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
applications, servicing queued callbacks, or many other activities. Finally,
and most importantly, using system calls allows applications to be built
independently from the kernel. The entire codebase of the kernel could change,
but as long as the system call interface remains identical, applications do not
even need to be recompiled to work on the platform. Applications, when
separated from the kernel, no longer need to be loaded at the same time as the
kernel. They could be uploaded at a later time, modified, and then have a new
version uploaded, all without modifying the kernel running on a platform.

## Process State

In Tock, a process can be in one of three states:

 - **Running**: Normal operation. A Running process is eligible to be scheduled
 for execution, although is subject to being paused by Tock to allow interrupt
 handlers or other processes to run. During normal operation, a process remains
 in the Running state until it explicitly yields. Callbacks from other kernel
 operations are not delivered to Running processes (i.e. callbacks do not
 interrupt processes), rather they are enqueued until the process yields.
 - **Yielded**: Suspended operation. A Yielded process will not be scheduled by
 Tock. Processes often yield while they are waiting for I/O or other operations
 to complete and have no immediately useful work to do. Whenever the kernel issues
 a callback to a Yielded process, the process is transitioned to the Running state.
 - **Fault**: Erroneous operation. A Fault-ed process will not be scheduled by
 Tock. Processes enter the Fault state by performing an illegal operation, such
 as accessing memory outside of their address space.

## Startup

Upon process initialization, a single function call task is added to it's
callback queue. The function is determined by the ENTRY point in the process
TBF header (typically the `_start` symbol) and is passed the following
arguments in registers `r0` - `r3`:

  * r0: the base address of the process code
  * r1: the base address of the processes allocated memory region
  * r2: the total amount of memory in its region
  * r3: the current process memory break

## The System Calls

All system calls except Yield (which cannot fail) return an integer return code
value to userspace. Negative return codes indicate an error. Values greater
than or equal to zero indicate success. Sometimes syscall return values encode
useful data, for example in the `gpio` driver, the command for reading the
value of a pin returns 0 or 1 based on the status of the pin.

Currently, the following return codes are defined, also available as `#defines`
in C from the `tock.h` header (prepended with `TOCK_`):

```rust
pub enum ReturnCode {
    SuccessWithValue { value: usize }, // Success value must be >= 0
    SUCCESS,
    FAIL, //.......... Generic failure condition
    EBUSY, //......... Underlying system is busy; retry
    EALREADY, //...... The state requested is already set
    EOFF, //.......... The component is powered down
    ERESERVE, //...... Reservation required before use
    EINVAL, //........ An invalid parameter was passed
    ESIZE, //......... Parameter passed was too large
    ECANCEL, //....... Operation cancelled by a call
    ENOMEM, //........ Memory required not available
    ENOSUPPORT, //.... Operation or command is unsupported
    ENODEVICE, //..... Device does not exist
    EUNINSTALLED, //.. Device is not physically installed
    ENOACK, //........ Packet transmission not acknowledged
}
```

### 0: Yield

Yield transitions the current process from the Running to the Yielded state, and
the process will not execute again until another callback re-schedules the
process.

If a process has enqueued callbacks waiting to execute when Yield is called, the
process immediately re-enters the Running state and the first callback runs.

```rust
yield()
```

#### Arguments

None.

#### Return

None.


### 1: Subscribe

Subscribe assigns callback functions to be executed in response to various
events. A null pointer to a callback disables a previously set callback.

```rust
subscribe(driver: u32, subscribe_number: u32, callback: u32, userdata: u32) -> ReturnCode as u32
```

#### Arguments

 - `driver`: An integer specifying which driver to call.
 - `subscribe_number`: An integer index for which function is being subscribed.
 - `callback`: A pointer to a callback function to be executed when this event
 occurs. All callbacks conform to the C-style function signature:
 `void callback(int arg1, int arg2, int arg3, void* data)`.
 - `userdata`: A pointer to a value of any type that will be passed back by the
   kernel as the last argument to `callback`.

Individual drivers define a mapping for `subscribe_number` to the events that
may generate that callback as well as the meaning for each of the `callback`
arguments.

#### Return

 - `EINVAL` if the callback pointer is NULL.
 - `ENODEVICE` if `driver` does not refer to a valid kernel driver.
 - `ENOSUPPORT` if the driver exists but doesn't support the `subscribe_number`.
 - Other return codes based on the specific driver.


### 2: Command

Command instructs the driver to perform a specific action.

```rust
command(driver: u32, command_number: u32, argument1: u32, argument2: u32) -> ReturnCode as u32
```

#### Arguments

 - `driver`: An integer specifying which driver to call.
 - `command_number`: An integer specifying the requested command.
 - `argument1`: A command-specific argument.
 - `argument2`: A command-specific argument.

The `command_number` tells the driver which command was called from
userspace, and the `argument`s are specific to the driver and command number.
One example of the argument being used is in the `led` driver, where the
command to turn on an LED uses the argument to specify which LED.

One Tock convention with the Command syscall is that command number 0 will
always return a value of 0 or greater if the driver is supported by the running
kernel. This means that any application can call command number 0 on any driver
number to determine if the driver is present and the related functionality is
supported. In most cases this command number will return 0, indicating that the
driver is present. In other cases, however, the return value can have an
additional meaning such as the number of devices present, as is the case in the
`led` driver to indicate how many LEDs are present on the board.

#### Return

 - `ENODEVICE` if `driver` does not refer to a valid kernel driver.
 - `ENOSUPPORT` if the driver exists but doesn't support the `command_number`.
 - Other return codes based on the specific driver.


### 3: Allow

Allow marks a region of memory as shared between the kernel and application.
A null pointer revokes sharing a region.

```rust
allow(driver: u32, allow_number: u32, pointer: usize, size: u32) -> ReturnCode as u32
```

#### Arguments

 - `driver`: An integer specifying which driver should be granted access.
 - `allow_number`: A driver-specific integer specifying the purpose of this
   buffer.
 - `pointer`: A pointer to the start of the buffer in the process memory space.
 - `size`: An integer number of bytes specifying the length of the buffer.

Many driver commands require that buffers are Allow-ed before they can execute.
A buffer that has been Allow-ed does not need to be Allow-ed to be used again.

As of this writing, most Tock drivers do not provide multiple virtual devices to
each application. If one application needs multiple users of a driver (i.e. two
libraries on top of I2C), each library will need to re-Allow its buffers before
beginning operations.

#### Return

 - `ENODEVICE` if `driver` does not refer to a valid kernel driver.
 - `ENOSUPPORT` if the driver exists but doesn't support the `allow_number`.
 - `EINVAL` the buffer referred to by `pointer` and `size` lies completely or
partially outside of the processes addressable RAM.
 - Other return codes based on the specific driver.


### 4: Memop

Memop expands the memory segment available to the process, allows the process to
retrieve pointers to its allocated memory space, provides a mechanism for
the process to tell the kernel where its stack and heap start, and other
operations involving process memory.

```rust
memop(op_type: u32, argument: u32) -> [[ VARIES ]] as u32
```

#### Arguments

 - `op_type`: An integer indicating whether this is a `brk` (0), a `sbrk` (1),
   or another memop call.
 - `argument`: The argument to `brk`, `sbrk`, or other call.

Each memop operation is specific and details of each call can be found in
the [memop syscall documentation](syscalls/memop.md).

#### Return

- Dependent on the particular memop call.


## The Context Switch

Handling a context switch is one of the few pieces of Tock code that is
actually architecture dependent and not just chip-specific. The code is located
in `lib.rs` within the `arch/` folder under the appropriate architecture. As
this code deals with low-level functionality in the processor it is written in
assembly wrapped as Rust function calls.

### Context Switch Interface

The architecture crates (in the `/arch` folder) are responsible for implementing
the `SyscallInterface` trait which defines the functions needed to allow the
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


## How System Calls Connect to Drivers

After a system call is made, Tock routes the call to the appropriate driver.

First, in [`sched.rs`](../kernel/src/sched.rs) the number of the `svc` is
matched against the valid syscall types. `yield` and `memop` have special
functionality that is handled by the kernel. `command`, `subscribe`, and
`allow` are routed to drivers for handling.

To route the `command`, `subscribe`, and `allow` syscalls, each board creates a
struct that implements the `Platform` trait. Implementing that trait only
requires implementing a `with_driver()` function that takes one argument, the
driver number, and returns a reference to the correct driver if it is supported
or `None` otherwise. The kernel then calls the appropriate syscall function on
that driver with the remaining syscall arguments.

An example board that implements the `Platform` trait looks something like this:

```rust
struct TestBoard {
    console: &'static Console<'static, usart::USART>,
}

impl Platform for TestBoard {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
        where F: FnOnce(Option<&kernel::Driver>) -> R
    {

        match driver_num {
            0 => f(Some(self.console)),
            _ => f(None),
        }
    }
}
```

`TestBoard` then supports one driver, the UART console, and maps it to driver
number 0. Any `command`, `subscribe`, and `allow` sycalls to driver number 0
will get routed to the console, and all other driver numbers will return
`ReturnCode::ENODEVICE`.



## Allocated Driver Numbers

All documented drivers are in the [doc/syscalls](syscalls/README.md) folder.
