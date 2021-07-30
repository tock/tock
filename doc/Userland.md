Userland
========

This document explains how application code works in Tock. This is not a guide
to writing applications, but rather documentation of the overall design
of how applications function.

<!-- npm i -g markdown-toc; markdown-toc -i Userland.md -->

<!-- toc -->

- [Overview of Processes in Tock](#overview-of-processes-in-tock)
- [System Calls](#system-calls)
- [Upcalls and Termination](#upcalls-and-termination)
- [Inter-Process Communication](#inter-process-communication)
  * [Services](#services)
  * [Clients](#clients)
- [Application Entry Point](#application-entry-point)
- [Stack and Heap](#stack-and-heap)
- [Debugging](#debugging)
- [Applications](#applications)

<!-- tocstop -->

## Overview of Processes in Tock

Processes in Tock run application code meant to accomplish some type
of task for the end user. Processes run in user mode. Unlike kernel
code, which runs in supervisor mode and handles device drivers,
chip-specific details, as well as general operating system tasks,
appliction code running in processes is independent of the details of
the underlying hardware (except the instruction set
architecture). Unlike many existing embedded operating systems, in
Tock processes are not compiled with the kernel. Instead they are
entirely separate code that interact with the kernel and each other
through [system calls](https://en.wikipedia.org/wiki/System_call).

Since processes are not a part of the kernel, application code running
in a process may be written in any language that can be compiled into
code capable of running on a microcontroller.  Tock supports running
multiple processes concurrently. Co-operatively multiprogramming is
the default, but processes may also be time sliced.  Processes may
share data with each other via Inter-Process Communication (IPC)
through system calls.

Processes run code in unprivileged mode (e.g., user mode on CortexM or
RV32I microcontrollers). The Tock kernel uses hardware memory
protection (an MPU on CortexM and a PMP on RV32I) to restrict which
addresses application code running in a process can access. A process
makes system calls to access hardware peripherals or modify what
memory is accessible to it.

Tock supports dynamically loading and unloading independently compiled
applications. In this setting, applications not not know at compile
time what address they will be installed at and loaded from. To be 
dynamically loadable, application code must be compiled as [position
independent
code](https://en.wikipedia.org/wiki/Position-independent_code)
(PIC). This allows them to be run from any address they happen to be
loaded into. 

In some cases, applications may know their location at compile-time. This
happens, for example, in cases where the kernel and applications are combined
into a single cryptographically signed binary that is accepted by 
a secure bootloader. In these cases, compiling an application with
explicit addresses works.

Tock supports running multiple processes at the same time. The maximum
number of processes supported by the kernel is typically a compile-time
constant in the range of 2-4, but is limited only by the available RAM
and Flash resources of the chip. Tock scheduling generally assumes that
it is a small number (e.g., uses O(n) scheduling algorithms).


## System Calls

System calls are how processes and the kernel share data and interact. These
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

A detailed description of Tock's system call API and ABI can be found in
[TRD104](reference/trd104-syscalls.md). The [system call
documentation](./Syscalls.md) describes how the are implemented in the
kernel.


## Upcalls and Termination

The Tock kernel is completely non-blocking, and it pushes this 
asynchronous behavior to userspace code. This means that system calls
(with one exception) do not block. Instead, they always return very quickly.
Long-running operations (e.g., sending data over a bus, sampling a sensor)
signal their completion to userspace through upcalls. An upcall is a function 
call the kernel makes on userspace code.

Yield system calls are the exception to this non-blocking rule. The yield-wait 
system call blocks until the kernel invokes an upcall on the process. 
The kernel only invokes upcalls when a process issues the yield system call:
it does not invoke upcalls at arbitrary points in the program.

For example, consider the case of when a process wants to sleep for 100 
milliseconds. The timer library might break this into three operations:

1. It registers an upcall for the timer system call driver with a Subscribe
system call.
2. It tells the timer system call driver to issue an upcall in 100 
milliseconds by invoking a Command system call.
3. It calls the yield-wait system call. This causes the process to block
until the timer upcall executes. The kernel pushes a stack frame onto
the process to execute the upcall; this function call returns to the
instruction after yield was invoked.

When a process registers an upcall with a call to a Subscribe system call,
it may pass a pointer `userdata`. The kernel does not access or use this
data: it simply passes it back on each invocation of the upcall. This
allows a process to register the same function as multiple upcalls, and
distinguish them by the data passed in the argument.

It is important to note that upcalls are not executed until a process
calls `yield`. The kernel will enqueue upcalls as events occur within
the kernel, but the application will not handle them until it yields.

Applications which are "finished"
should call an Exit system call. There are two variants of Exit:
exit-terminate and exit-restart. They differ in what they signal to
the kernel: does the application wish to stop running, or be rebooted?

## Inter-Process Communication

Inter-process communication (IPC) allows for separate processes to
communicate directly through shared buffers. IPC in Tock is
implemented with a service-client model. Each process can support one
service. The service is identified by the name of the application running
in the process, which is
included in the Tock Binary Format Header for the application. A process can
communicate with multiple services and will get a unique handle for
each discovered service.  Clients and services communicate through
shared buffers. Each client can share some of its own application
memory with the service and then notify the service to instruct it to
parse the shared buffer.

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
[move their program break](/doc/syscalls/memop.md#operation-type-0-brk) to accommodate
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

If an application crashes, Tock provides a very detailed stack dump.
By default, when an application crashes Tock prints a crash dump over the
platform's default console interface. When your application crashes,
we recommend looking at this output very carefully: often we have spent 
hours trying to track down a bug which in retrospect was quite obviously
indicated in the dump, if we had just looked at the right fields.

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
