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
- [Applications](#applications)

<!-- tocstop -->

## Overview of Applications in Tock

Applications in Tock are the user-level code meant to accomplish some type of
task for the end user. Applications are distinguished from kernel code which
handles device drivers, chip-specific details, and general operating system
tasks. Unlike many existing embedded operating systems, in Tock applications
are not compiled with the kernel. Instead they are entirely separate code
that interact with the kernel and each other through [system
calls](https://en.wikipedia.org/wiki/System_call).

Since applications are not a part of the kernel, they may be written in any
language that can be compiled into code capable of running on a microcontroller.
Tock supports running multiple applications concurrently. Co-operatively
multiprogramming is the default, but applications may also be time sliced.
Applications may talk to each other via Inter-Process Communication (IPC)
through system calls.

Applications do not have compile-time knowledge of the address at which they
will be installed and loaded. In the current design of Tock, applications must
be compiled as [position independent
code](https://en.wikipedia.org/wiki/Position-independent_code) (PIC). This
allows them to be run from any address they happen to be loaded into. The use
of PIC for Tock apps is not a fundamental choice, future versions of the system
may support run-time relocatable code.

Applications are unprivileged code. They may not access all portions of memory
and will fault if they attempt to access memory outside of their boundaries
(similarly to segmentation faults in Linux code). To interact with hardware,
applications must make calls to the kernel.


## System Calls

System calls (aka syscalls) are used to send commands to the kernel. These
could include commands to drivers, subscriptions to callbacks, granting of
memory to the kernel so it can store data related to the application,
communication with other application code, and many others. In practice,
system calls are made through library code and the application need not
deal with them directly.

For example, consider the following system call that sets a GPIO pin high:

```c
int gpio_set(GPIO_Pin_t pin) {
  return command(GPIO_DRIVER_NUM, 2, pin);
}
```

The command system call itself is implemented as the ARM assembly instruction
`svc` (service call):

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

It is important to note that `yield` must be called for events to be serviced in
the current implementation of Tock. Callbacks to the application will be queued
when they occur but the application will not receive them until it yields. This
is not fundamental to Tock, and future version may service callbacks on any
system call or when performing application time slicing. After receiving and
running the callback, application code will continue after the `yield`.
Applications which are "finished" (i.e. have returned from `main()`) should call
`yield` in a loop to avoid being scheduled by the kernel.


## Inter-Process Communication

IPC allows for multiple applications to communicate directly through shared
buffers. IPC in Tock is implemented with a service-client model. Each app can
support one service and the service is identified by its package name which is
included in the Tock Binary Format Header for the app. An app can communicate
with multiple services and will get a unique handle for each discovered service.
Clients and services communicate through shared buffers. Each client can share
some of its own application memory with the service and then notify the service
to instruct it to parse the shared buffer.

### Services

Services are named by the package name included in the app's TBF header.
To register a service, an app can call `ipc_register_svc()` to setup a callback.
This callback will be called whenever a client calls notify on that service.

### Clients

Clients must first discover services they wish to use with the function
`ipc_discover()`. They can then share a buffer with the service by calling
`ipc_share()`. To instruct the service to do something with the buffer, the
client can call `ipc_notify_svc()`. If the app wants to get notifications from
the service, it must call `ipc_register_client_cb()` to receive events from when
the service when the service calls `ipc_notify_client()`.

See `ipc.h` in `libtock-c` for more information on these functions.

## Application Entry Point

An application specifies the first function the kernel should call by setting
the variable `init_fn_offset` in its TBF header. This function should have the
following signature:

```c
void _start(void* text_start, void* mem_start, void* memory_len, void* app_heap_break);
```

The Tock kernel tries to impart no restrictions on the stack and heap layout of
application processes. As such, a process starts in a very minimal environment,
with an initial stack sufficient to support a syscall, but not much more.
Application startup routines should first
[move their program break](/doc/syscalls/memop.md#operation-type-0-brk) to accomodate
their desired layout, and then setup local stack and heap tracking in accordance
with their runtime.


## Stack and Heap

Applications can specify memory requirements by setting the `minimum_ram_size` variable
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
App: crash_dummy   -   [Fault]
 Events Queued: 0   Syscall Count: 0   Dropped Callback Count: 0
 Restart Count: 0
 Last Syscall: None

 ╔═══════════╤══════════════════════════════════════════╗
 ║  Address  │ Region Name    Used | Allocated (bytes)  ║
 ╚0x20006000═╪══════════════════════════════════════════╝
             │ ▼ Grant         948 |    948
  0x20005C4C ┼───────────────────────────────────────────
             │ Unused
  0x200049F0 ┼───────────────────────────────────────────
             │ ▲ Heap            0 |   4700               S
  0x200049F0 ┼─────────────────────────────────────────── R
             │ Data            496 |    496               A
  0x20004800 ┼─────────────────────────────────────────── M
             │ ▼ Stack          72 |   2048
  0x200047B8 ┼───────────────────────────────────────────
             │ Unused
  0x20004000 ┴───────────────────────────────────────────
             .....
  0x00030400 ┬─────────────────────────────────────────── F
             │ App Flash       976                        L
  0x00030030 ┼─────────────────────────────────────────── A
             │ Protected        48                        S
  0x00030000 ┴─────────────────────────────────────────── H

  R0 : 0x00000000    R6 : 0x20004894
  R1 : 0x00000001    R7 : 0x20004000
  R2 : 0x00000000    R8 : 0x00000000
  R3 : 0x00000000    R10: 0x00000000
  R4 : 0x00000000    R11: 0x00000000
  R5 : 0x20004800    R12: 0x12E36C82
  R9 : 0x20004800 (Static Base Register)
  SP : 0x200047B8 (Process Stack Pointer)
  LR : 0x000301B7
  PC : 0x000300AA
 YPC : 0x000301B6

 APSR: N 0 Z 1 C 1 V 0 Q 0
       GE 0 0 0 0
 EPSR: ICI.IT 0x00
       ThumbBit true

 Cortex-M MPU
  Region 0: base: 0x20004000, length: 8192 bytes; ReadWrite (0x3)
  Region 1: base:    0x30000, length: 1024 bytes; ReadOnly (0x6)
  Region 2: Unused
  Region 3: Unused
  Region 4: Unused
  Region 5: Unused
  Region 6: Unused
  Region 7: Unused

To debug, run `make debug RAM_START=0x20004000 FLASH_INIT=0x30059`
in the app's folder and open the .lst file.
```

## Applications

For example applications, see the language specific userland repos:

- [libtock-c](https://github.com/tock/libtock-c): C and C++ apps.
- [libtock-rs](https://github.com/tock/libtock-rs): Rust apps.
