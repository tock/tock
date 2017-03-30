# Application Code
This document explains how application code works in Tock. This is not a guide
to creating your own applications, but rather documentation of the design
thoughts behind how applications function.

<!-- npm i -g markdown-toc; markdown-toc -i Userland.md -->

<!-- toc -->

- [Overview of Applications in Tock](#overview-of-applications-in-tock)
- [System Calls](#system-calls)
- [Callbacks](#callbacks)
- [Inter-Process Communication](#inter-process-communication)
- [Application Entry Point](#application-entry-point)
- [Stack and Heap](#stack-and-heap)
- [Debugging](#debugging)
- [Libraries](#libraries)
  * [Newlib](#newlib)
  * [libtock](#libtock)
- [Related](#related)

<!-- tocstop -->

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
memory to the kernel so it can store data related to the application,
communication with other application code, and many others. In practice,
system calls are made through library code and the application need not
deal with them directly.

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

A more in-depth discussion can be found in the [system call
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

## Application Entry Point

__Warning: Unstable__

Applications should define a `main` method:

    int main(void)

Currently, main receives no arguments and its return value is ignored.
Applications **should** return 0 from `main`.  Applications are not terminated
when `main` returns, rather an implicit `while (1) { yield(); }` follows
`main`, allowing applications to set up a series of event subscriptions in
their `main` method and then return.

## Stack and Heap
Applications can specify their required stack and heap sizes by defining the
make variables `STACK_SIZE` and `APP_HEAP_SIZE`, which default to 2K and 1K
respectively as of this writing.  Note that the Tock kernel treats these as
minimum values, depending on the underlying platform, the stack and heap may be
larger than requested, but will never be smaller.

If there is insufficient memory to load your application, the kernel will fail
during loading and print a message.

If an application exceeds its alloted memory during runtime, the
application will crash (see the [Debugging](#debugging) section for an
example).

## Debugging

If an application crashes, Tock can provide a lot of useful information.
By default, when an application crashes Tock prints a crash dump over the
platform's default console interface.

Note that because an application is relocated when it is loaded, this trace
will print both relocated addresses and the original symbol address where
appropriate.

```
---| Fault Status |---
Data Access Violation:              true
Forced Hard Fault:                  true
Faulting Memory Address:            0x00000000
Fault Status Register (CFSR):       0x00000082
Hard Fault Status Register (HFSR):  0x40000000

---| App Status |---
App: sensors
[Fault]  -  Events Queued: 0  Syscall Count: 24

╔═══════════╤══════════════════════════════════════════╗
║  Address  │ Region Name    Used | Allocated (bytes)  ║
╚0x20006000═╪══════════════════════════════════════════╝
            │ ▼ Grant         356 |   1024          
 0x20005E9C ┼───────────────────────────────────────────
            │ Unused
 0x20004FB4 ┼───────────────────────────────────────────
            │ ▲ Heap         1580 |   1024 EXCEEDED!     S
 0x20004988 ┼─────────────────────────────────────────── R
            │ ▼ Stack          48 |   2048               A
 0x20004958 ┼─────────────────────────────────────────── M
            │ Unused
 0x20004188 ┼───────────────────────────────────────────
            │ Data            392 |    392
 0x20004000 ┴───────────────────────────────────────────
            .....
 0x00034000 ┬───────────────────────────────────────────
            │ Unused
 0x00033BBF ┼─────────────────────────────────────────── F
            │ Data            329                        L
 0x00033A76 ┼─────────────────────────────────────────── A
            │ Text          14842                        S
 0x0003007C ┼─────────────────────────────────────────── H
            │ Header          124
 0x00030000 ┴───────────────────────────────────────────

 R0 : 0x00000000    R6 : 0x00000000
 R1 : 0x00000005    R7 : 0x20004978
 R2 : 0x00000103    R8 : 0x00000000
 R3 : 0x00000000    R10: 0x00000000
 R4 : 0x00000000    R11: 0x00000000
 R5 : 0x00000000    R12: 0x00000000
 R9 : 0x20004000 (Static Base Register)
 SP : 0x20004070 (Process Stack Pointer)
 LR : 0x00031667 [0x800015EA in lst file]
 PC : 0x0003036A [0x800002EE in lst file]
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

