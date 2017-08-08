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
- [The System Calls](#the-system-calls)
  * [0: Yield](#0-yield)
  * [1: Subscribe](#1-subscribe)
  * [2: Command](#2-command)
  * [3: Allow](#3-allow)
  * [4: Memop](#4-memop)
- [The Context Switch](#the-context-switch)
- [How System Calls Connect to Drivers](#how-system-calls-connect-to-drivers)
- [Allocated Driver Numbers](#allocated-driver-numbers)
  * [Base](#base)
  * [Kernel](#kernel)
  * [HW Buses](#hw-buses)
  * [Radio](#radio)
  * [Cryptography](#cryptography)
  * [Storage](#storage)
  * [Sensors](#sensors)
  * [Sensor ICs](#sensor-ics)
  * [Other ICs](#other-ics)

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

The Yield syscall takes no arguments.

### 1: Subscribe

Subscribe assigns callback functions to be executed in response to various
events.

The Subscribe syscall takes two arguments:

 - `subscribe_number`: An integer index for which function is being subscribed.
 - `callback`: A pointer to a callback function to be executed when this event
 occurs. All callbacks conform to the C-style function signature:
 `void callback(int arg1, int arg2, int arg3, void* data)`.

Individual drivers define a mapping for `subscribe_number` to the events that
may generate that callback as well as the meaning for each of the `callback`
arguments.

### 2: Command

Command instructs the driver to perform a specific action.

The Command syscall takes two arguments:

 - `command_number`: An integer specifying the requested command.
 - `argument`: A command-specific argument.

The `command_number` tells the driver which command was called from
userspace, and the `argument` is specific to the driver and command number.
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

### 3: Allow

Allow marks a region of memory as shared between the kernel and application.

The Allow syscall takes four arguments:

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

### 4: Memop

Memop expands the memory segment available to the process, allows the process to
retrieve pointers to its allocated memory space, and provides a mechanism for
the process to tell the kernel where its stack and heap start.

The Memop syscall takes two arguments:

 - `op_type`: An integer indicating whether this is a `brk` (0), a `sbrk` (1),
   or another memop call.
 - `argument`: The argument to `brk`, `sbrk`, or other call.

Both `brk` and `sbrk` adjust the current memory segment. The `argument` to `brk`
is a pointer indicating the new requested end of memory segment. The `argument`
to `sbrk` is an integer, indicating the number of bytes to adjust the end of the
memory segment by.

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

In summary, execution is handled so that the application resumes at the next
instruction after a system call is complete and the kernel resumes operation
whenever a system call is made.


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

impl Platform for Hail {
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

### Base

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
| ✓ | 0x0           | Timer            |                                            |
| ✓ | 0x1           | Console          | UART console                               |
| ✓ | 0x2           | LED              |                                            |
| ✓ | 0x3           | Button           |                                            |
| ✓ | 0x4           | GPIO             |                                            |
| ✓ | 0x5           | ADC              |                                            |
| ✓ | 0x6           | DAC              | Digital to analog converter                |

### Kernel

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
| ✓ | 0x10000       | IPC              | Inter-process communication                |

### HW Buses

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
| x | 0x20000       | UART             | UART                                       |
| x | 0x20001       | SPI              | Raw SPI Master interface                   |
| x | 0x20002       | SPI Slave        | Raw SPI slave interface                    |
| x | 0x20003       | I2C Master       | Raw I2C Master interface                   |
| x | 0x20004       | I2C Slave        | Raw I2C Slave interface                    |
| x | 0x20005       | USB              | Universal Serial Bus interface             |

### Radio

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
| x | 0x30000       | BLE              | Bluetooth Low Energy                       |
| x | 0x30001       | 802.15.4         | IEEE 802.15.4                              |

### Cryptography

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
| x | 0x40000       | AES              | AES Symmetric Key Cryptography             |
| x | 0x40001       | RNG              | Random number generator                    |
| x | 0x40002       | CRC              | Cyclic Redundancy Check computation        |

### Storage

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
| x | 0x50000       | App Flash        | Allow apps to write their own flash        |
| x | 0x50001       | Nonvolatile Storage | Generic interface for persistent storage |
| x | 0x50002       | SDCard           | Raw block access to an SD card             |

### Sensors

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
| ✓ | 0x60000       | Ninedof          | Virtualized accelerometer/magnetometer/gyroscope |
| ✓ | 0x60001       | Ambient Temp.    | Ambient temperature (centigrate)           |
| ✓ | 0x60002       | Humidity         | Ambient Light Sensor (lumens)              |
| ✓ | 0x60003       | Luminance        | Humidity Sensor (percent)                  |
| x | 0x60004       | Pressure         | Pressure sensor                            |

### Sensor ICs

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
| x | 0x70000       | TSL2561          | Light sensor                               |
| x | 0x70001       | TMP006           | Temperature sensor                         |
| x | 0x70004       | LPS25HB          | Pressure sensor                            |

### Other ICs

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
| x | 0x80000       | LTC294X          | Battery gauge IC                           |
| x | 0x80001       | MAX17205         | Battery gauge IC                           |
| x | 0x80002       | PCA9544A         | I2C address multiplexing                   |
| x | 0x80003       | GPIO Async       | Asynchronous GPIO pins                     |
| x | 0x80004       | nRF51822         | nRF serialization link to nRF51822 BLE SoC |

