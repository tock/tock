# Application Code
This document explains how application code works in Tock. This is not a guide
to creating your own applications, but rather documentation of the design
thoughts behind how applications function.


## Overview of Applications in Tock
Applications in Tock are the user-level code meant to accomplish some type of
task for the end user. Applications are distinguished from kernel code which
handles device drivers, chip-specific details, and general operating system
tasks. Unlike many existing embedded operating systems, in Tock applications
are not built as one with the kernel. Instead they are entirely separate code
that interact with the kernel and each other through [system
calls](https://en.wikipedia.org/wiki/System_call).

Since applications are not a part of the kernel, they may be written in any
language that can be compiled into code capable of running on ARM Cortex-M
processors. While the Tock kernel is written in Rust, applications are commonly
written in C. Additionally, Tock supports running multiple applications
concurrently. Co-operatively multiprogramming is the default, but applications
may also be time sliced. Applications may talk to each other via Inter-Process
Communication (IPC) through system calls.

Applications do not have compile-time knowledge of the address at which they
will be installed and loaded. In the current design of Tock, applications must
be compiled as [position independent
code](https://en.wikipedia.org/wiki/Position-independent_code) (PIC). This
allows them to be run from any address they happen to be loaded into. The use
of PIC for Tock apps is not a fundamental choice, future versions of the system
may support run-time relocatable code.

Applications are unprivileged code. They may not access all portions of memory
and may, in fact, fault if they attempt to access memory outside of their
boundaries (similarly to segmentation faults in Linux code). In order to
interact with hardware, applications must make calls to the kernel.


## System Calls
System calls (aka syscalls) are used to send commands to the kernel. These
could include commands to drivers, subscriptions to callbacks, granting of
memory to the kernel to use to store received data, communication with other
application code, and many others. In practice, the system call is taken care
of by library code and the application need not deal with them directly.

For example the following is the system call handling the `gpio_set` command
from [gpio.c](../userland/libtock/gpio.c):

```c
int gpio_set(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 2, pin);
}
```

The command system call itself is implemented as the ARM assembly instruction
`svc` (service call) in [tock.c](../userland/libtock/tock.c):

```c
int __attribute__((naked))
command(uint32_t driver, uint32_t command, int data) {
  asm volatile("svc 2\nbx lr" ::: "memory", "r0");
}
```

A more in-depth discussion of can be found in the [system call
documentation](./Syscalls.md).


## Callbacks
Tock is designed to support embedded applications, which often handle
asynchronous events through the use of [callback
functions](https://en.wikipedia.org/wiki/Callback_(computer_programming)). For
example, in order to receive timer callbacks, you first call `timer_subscribe`
with a function pointer to your own function that you want called when the
timer fires. Specific state that you want the callback to act upon can be
passed as the pointer `userdata`. After the application has started the timer,
calls `yield`, and the timer fires, the callback function will be called.

It is important to note that `yield` must be called in order for events to be
serviced in the current implementation of Tock. Callbacks to the application
will be queued when they occur but the application will not receive them until
it yields. This is not fundamental to Tock, and future version may service
callbacks on any system call or when performing application time slicing. After
receiving and running the callback, application code will continue after the
`yield`. Tock automatically calls `yield` continuously for applications that
return from execution (for example, an application that returns from `main`).


## Inter-Process Communication
 * **TODO:** how does this work?

## Debugging

If an application crashes, Tock can provide a lot of useful information.
By default, when an application crashes Tock prints a crash dump over the
platform's default console interface.

Note that because an applicaiton is relocated when it is loaded, this trace
will print both relocated addresses and the original symbol address where
appropriate.

```
---| Fault Status |---
Data Access Violation:              true
Forced Hard Fault:                  true
Faulting Memory Address:            0x00000006
Fault Status Register (CFSR):       0x00000082
Hard Fault Status Register (HFSR):  0x40000000

---| App Status |---
App: sensors
[Fault]  -  Events Queued: 0  Syscall Count: 57

╔═══════════╤══════════════════════════════════════════╗
║  Address  │ Region Name    Used | Allocated (bytes)  ║
╚0x20006000═╪══════════════════════════════════════════╝
            │ ▼ Grant         340 |   1024          
 0x20005EAC ┼───────────────────────────────────────────
            │ Unused
 0x20004FE8 ┼───────────────────────────────────────────
            │ ▲ Heap         1608 |   1024 EXCEEDED!     S
 0x200049A0 ┼─────────────────────────────────────────── R
            │ ▼ Stack         104 |   2048               A
 0x20004938 ┼─────────────────────────────────────────── M
            │ Unused
 0x200041A0 ┼───────────────────────────────────────────
            │ Data            416 |    416
 0x20004000 ┴───────────────────────────────────────────
            .....                                        F
 0x00034000 ┬─────────────────────────────────────────── l
            │ Text              ? |  16384               a
 0x00030000 ┴─────────────────────────────────────────── s
                                                         h
 R0 : 0x0000000A
 R1 : 0x00000003
 R2 : 0x20004000
 R3 : 0x00000006
 R4 : 0x00000000
 R5 : 0x00000000
 R6 : 0x00000000
 R7 : 0x2000495C
 R8 : 0x00000000
 R9 : 0x20004000
 R10: 0x00000000
 R11: 0x00000000
 R12: 0x00000006
 PC : 0x00030234 [0x800001B8 in lst file]
 LR : 0x00031CF7 [0x80001C7A in lst file]
```

## Libraries
Application code does not need to stand alone, libraries are available that can
be utilized!


### Newlib
Application code written in C has access to most of the [C standard
library](https://en.wikipedia.org/wiki/C_standard_library) which is implemented
by [Newlib](https://en.wikipedia.org/wiki/Newlib). Newlib is focused on
providing capabilities for embedded systems. It provides interfaces such as
`printf`, `malloc`, and `memcpy`. Most, but not all features of the standard
library are available to applications. The built configuration of Newlib is
specified in [build.sh](../userland/newlib/build.sh).


### libtock
In order to interact with the Tock kernel, application code can use the
`libtock` library. The majority of `libtock` are wrappers for interacting
with Tock drivers through system calls. They provide the user a meaningful
function name and arguments and then internally translate these into a
`command`, `subscribe`, etc. Where it makes sense, the libraries also provide
a synchronous interface to a driver using an internal callback and `yield_for`
(example:
[`tmp006_read_sync`](https://github.com/helena-project/tock/blob/master/userland/libtock/tmp006.c#L20))

`libtock` also provides the startup code for applications
([`crt1.c`](https://github.com/helena-project/tock/blob/master/userland/libtock/crt1.c)),
an implementation for the system calls
([`tock.c`](https://github.com/helena-project/tock/blob/master/userland/libtock/tock.c)),
and pin definitions for platforms.


## Related

 * For general information on Tock: [Overview](./Overview.md)
 * For more information on system calls: [Syscalls](./Syscalls.md)
 * For more information on compiling app and app binary format: [Compilation](./Compilation.md)

