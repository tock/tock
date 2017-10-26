Userland
========

This document explains how application code works in Tock. This is not a guide
to creating your own applications, but rather documentation of the design
thoughts behind how applications function.

<!-- npm i -g markdown-toc; markdown-toc -i Userland.md -->

<!-- toc -->

- [Overview of Applications in Tock](#overview-of-applications-in-tock)
- [System Calls](#system-calls)
- [Callbacks](#callbacks)
- [Inter-Process Communication](#inter-process-communication)
  * [Services](#services)
  * [Clients](#clients)
- [Application Entry Point](#application-entry-point)
- [Stack and Heap](#stack-and-heap)
- [Debugging](#debugging)
- [C Applications](#c-applications)
  * [Entry Point](#entry-point)
  * [Stack and Heap](#stack-and-heap-1)
  * [Libraries](#libraries)
    + [Newlib](#newlib)
    + [libtock](#libtock)
    + [libc++](#libc)
    + [libnrfserialization](#libnrfserialization)
    + [lua53](#lua53)
  * [Style & Format](#style--format)
- [Rust Applications](#rust-applications)

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

IPC allows for multiple applications to communicate directly through shared
buffers. IPC in Tock is implemented with a service-client model. Each app can
support one service and the service is identified by the `PACKAGE_NAME` variable
set in its Makefile. An app can communicate with multiple services and will get
a unique handle for each discovered service. Clients and services communicate
through shared buffers. Each client can share some of its own application memory
with the service and then notify the service to instruct it to parse the shared
buffer.

### Services

Services are named by the `PACKAGE_NAME` variable in the application Makefile.
To register a service, an app can call `ipc_register_svc()` to setup a callback.
This callback will be called whenever a client calls notify on that service.

### Clients

Clients must first discover services they wish to use with the function
`ipc_discover()`. They can then share a buffer with the service by calling
`ipc_share()`. To instruct the service to do something with the buffer, the
client can call `ipc_notify_svc()`. If the app wants to get notifications from
the service, it must call `ipc_register_client_cb()` to receive events from when
the service when the service calls `ipc_notify_client()`.

See `ipc.h` in `libtock` for more information on these functions.

## Application Entry Point

An application specifies the first function the kernel should call by setting
the variable `init_fn_offset` in its TBF header. This function should have the
following signature:

```c
void _start(void* text_start, void* mem_start, void* memory_len, void* app_heap_break);
```

## Stack and Heap

Applications can memory requirements by setting the `minimum_ram_size` variable
in their TBF headers. Note that the Tock kernel treats this as a minimum,
depending on the underlying platform, the amount of memory may be larger than
requested, but will never be smaller.

If there is insufficient memory to load your application, the kernel will fail
during loading and print a message.

If an application exceeds its alloted memory during runtime, the application
will crash (see the [Debugging](#debugging) section for an example).

## Debugging

If an application crashes, Tock can provide a lot of useful information.
By default, when an application crashes Tock prints a crash dump over the
platform's default console interface.

Note that because an application is relocated when it is loaded, the binaries
and debugging .lst files generated when the app was originally compiled will not
match the actual executing application on the board. To generate matching files
(and in particular a matching .lst file), you can use the `make debug` target
app directory to create an appropriate .lst file that matches how the
application was actually executed. See the end of the debug print out for an
example command invocation.

```
---| Fault Status |---
Data Access Violation:              true
Forced Hard Fault:                  true
Faulting Memory Address:            0x00000000
Fault Status Register (CFSR):       0x00000082
Hard Fault Status Register (HFSR):  0x40000000

---| App Status |---
App: printf_long   -   [Yielded]
 Events Queued: 0   Syscall Count: 12   Last Syscall: YIELD

 ╔═══════════╤══════════════════════════════════════════╗
 ║  Address  │ Region Name    Used | Allocated (bytes)  ║
 ╚0x20006000═╪══════════════════════════════════════════╝
             │ ▼ Grant         332 |    332
  0x20005EB4 ┼───────────────────────────────────────────
             │ Unused
  0x2000506C ┼───────────────────────────────────────────
             │ ▲ Heap         1596 |   5252               S
  0x20004A30 ┼─────────────────────────────────────────── R
             │ Data              0 |      0               A
  0x20004800 ┼─────────────────────────────────────────── M
             │ ▼ Stack         880 |   2048
  0x200046C0 ┼───────────────────────────────────────────
             │ Unused
  0x20004000 ┴───────────────────────────────────────────
             .....
  0x00031000 ┬─────────────────────────────────────────── F
             │ App Flash      4000                        L
  0x00030060 ┼─────────────────────────────────────────── A
             │ Protected        96                        S
  0x00030000 ┴─────────────────────────────────────────── H

  R0 : 0x20004800    R6 : 0x200048CC
  R1 : 0x00000000    R7 : 0x00000000
  R2 : 0x00000000    R8 : 0x00000000
  R3 : 0x00000000    R10: 0x00000000
  R4 : 0x00000000    R11: 0x00000000
  R5 : 0x00000000    R12: 0x00000000
  R9 : 0x20004800 (Static Base Register)
  SP : 0x200047E0 (Process Stack Pointer)
  LR : 0x00030093
  PC : 0x00000000
 YPC : 0x0003010C

 APSR: N 0 Z 0 C 0 V 0 Q 0
       GE 0 0 1 1
 IPSR: Exception Type - IRQn
 EPSR: ICI.IT 0x00
       ThumbBit false !!ERROR - Cortex M Thumb only!
 To debug, run `make debug RAM_START=0x20004000 FLASH_INIT=0x30089`
 in the app's folder.
```

## C Applications

The bulk of Tock applications are written in C.

### Entry Point

Applications written in C that compile against libtock should define a `main`
method with the following signature:

```c
int main(void);
```

Applications **should** return 0 from `main`, but `main` is called from `_start`
and includes an implicit `while()` loop:

```c
void _start(void* text_start, void* mem_start, void* memory_len, void* app_heap_break) {
  main();
  while (1) {
    yield();
  }
}
```

Applications should set up a series of event subscriptions in their `main`
method and then return.

### Stack and Heap

Applications can specify their required stack and heap sizes by defining the
make variables `STACK_SIZE` and `APP_HEAP_SIZE`, which default to 2K and 1K
respectively as of this writing.

### Libraries

Application code does not need to stand alone, libraries are available that can
be utilized!

#### Newlib
Application code written in C has access to most of the [C standard
library](https://en.wikipedia.org/wiki/C_standard_library) which is implemented
by [Newlib](https://en.wikipedia.org/wiki/Newlib). Newlib is focused on
providing capabilities for embedded systems. It provides interfaces such as
`printf`, `malloc`, and `memcpy`. Most, but not all features of the standard
library are available to applications. The built configuration of Newlib is
specified in [build.sh](../userland/newlib/build.sh).

#### libtock
In order to interact with the Tock kernel, application code can use the
`libtock` library. The majority of `libtock` are wrappers for interacting
with Tock drivers through system calls. They provide the user a meaningful
function name and arguments and then internally translate these into a
`command`, `subscribe`, etc. Where it makes sense, the libraries also provide
a synchronous interface to a driver using an internal callback and `yield_for`
(example:
[`tmp006_read_sync`](https://github.com/helena-project/tock/blob/master/userland/libtock/tmp006.c#L19))

`libtock` also provides the startup code for applications
([`crt1.c`](https://github.com/helena-project/tock/blob/master/userland/libtock/crt1.c)),
an implementation for the system calls
([`tock.c`](https://github.com/helena-project/tock/blob/master/userland/libtock/tock.c)),
and pin definitions for platforms.

#### libc++
Provides support for C++ apps. See `examples/cxx_hello`.

#### libnrfserialization
Provides a pre-compiled library for using the Nordic nRF serialization library
for writing BLE apps.

#### lua53
Provides support for running a lua runtime as a Tock app. See
`examples/lua-hello`.

### Style & Format

We try to keep a consistent style in mainline userland code. For C/C++, we use
[uncrustify](https://github.com/uncrustify/uncrustify). High level:

  - Two space character indents.
  - Braces on the same line.
  - Spaces around most operators.

For details, see the [configuration](../userland/tools/uncrustify).

Travis will automatically check formatting. You can format code locally using
`make format`, or check the whole codebase with
[format_all.sh](../userland/examples/format_all.sh). Formatting will overwrite
files when it runs.


## Rust Applications

See the [libtock-rs](https://github.com/helena-project/libtock-rs) repo for more
information on writing userland rust apps.
