---
title: Tock Embedded OS Tutorial
date: SenSys 2017
header-includes:
  - \beamertemplatenavigationsymbolsempty
  - \usepackage{pifont}
  - \newcommand{\cmark}{\color{green}\ding{51}}
  - \newcommand{\xmark}{\color{red}\ding{55}}
---

## Welcome to the Tock OS Training!

> Please make sure you have completed all of the tutorial pre-requisites.  If
> you prefer, you can download a virtual machine image with all the
> pre-requisites already installed.

<https://github.com/helena-project/tock/tree/master/doc/courses/sensys/README.md>

> aka

<tt><http://bit.do/tock></tt>

## Tock

A secure operating system for microcontrollers

  * Kernel components in Rust

  * Type-safe API for safe driver development

  * Hardware isolated processes for application code

## Use cases

  * Security applications (e.g. authentication keys)

  * Sensor networks

  * Programmable wearables

  * PC/phone peripherals

  * Home/industrial automation

  * Flight control

## TockOS Stack

![](stack.pdf)

## Two types of scheduling: cooperative and preemptive

![](scheduling-crop.pdf)

## Agenda Today

  1. Intro to hardware, tools and development environment

  2. Write an end-to-end Bluetooth Low Energy environment sensing application

  3. Add functionality to the Tock kernel

# Part 1: Hardware, tools, and development environment

## Hail

![](hail.png)

## We need the Hails back at the end of the tutorial

But you can take one home with you! Purchase here:

<https://tockos.org/hardware>

Put in "SENSYS17" for $5 off, and "XXX" as the address for local pickup.

## Binaries on-board in flash

  - `0x00000`: **Bootloader**: Interact with Tockloader; load code

  - `0x10000`: **Kernel**

  - `0x30000`: **Processes**: Packed back-to-back

## Tools

  * `make`

  * Rust/Cargo/Xargo (Rust code → LLVM)

  * `arm-none-eabi` (LLVM → Cortex-M)

  * `tockloader` to interact with Hail and the bootloader

## Tools: `tockloader`

Write a binary to a particular address in flash

```bash
$ tockloader flash --address 0x10000 \
    target/thumbv7em-none-eabi/release/hail.bin
```

Program a process in Tock Binary Format[^footnote]:

```bash
$ tockloader install myapp.tab
```

Restart the board and connect to the debug console:

```bash
$ tockloader listen
```

[^footnote]: TBFs are relocatable process binaries prefixed with headers like
  the package name. `.tab` is a tarball of TBFs for different architectures as
  well as a metadata file for `tockloader`.

## Check your understanding

  1. What kinds of binaries exist on a Tock board? Hint: There are three, and
     only two can be programmed using `tockloader`.

  2. Can you point to the chip on the Hail that runs the Tock kernel? How about
     the processes?

  3. What steps would you follow to program a processes onto Hail? What about
     to replace the kernel?

## Hands-on: Set-up development environment

  3. Compile and flash the kernel

  4. (Optional) Familiarize yourself with `tockloader` commands

    * `uninstall`

    * `list`

    * `erase-apps`

  5. (Optional) Add some other apps from the repo, like `blink` and `sensors`


 - Head to <http://bit.do/tock2> to get started!
 - \tiny ([https://github.com/helena-project/tock/blob/master/doc/courses/sensys/environment.md](https://github.com/helena-project/tock/blob/master/doc/courses/sensys/environment.md))

# Part 2: User space

## System calls

| **Call**  | **Target** | **Description**                  |
|:----------|:----------:|----------------------------------|
| command   | Capsule    | Invoke an operation on a capsule |
| allow     | Capsule    | Share memory with a capsule      |
| subscribe | Capsule    | Register an upcall               |
| memop     | Core       | Modify memory break              |
| yield     | Core       | Block until next upcall is ready |

## C System Calls: `command` & `allow`


```c
// Start an operation
int command(u32 driver, u32 command, int arg1, int arg2);

// Share memory with the kernel
int allow(u32 driver, u32 allow, void* ptr, size_t size);
```

## C System Calls: `subscribe`

```c
// Callback function type
typedef void (sub_cb)(int, int, int, void* userdata);

// Register a callback with the kernel
int subscribe(u32 driver,
              u32 subscribe,
              sub_cb cb,
              void* userdata);
```

## C System Calls: `yield` & `yield_for`

```c
// Block until next callback
void yield(void);

// Block until specific callback
void yield_for(bool *cond) {
  while (!*cond) {
    yield();
  }
}
```

## Example: printing to the debug console

```c
static void putstr_cb(int _x, int _y, int _z, void* ud) {
  putstr_data_t* data = (putstr_data_t*)ud;
  data->done = true;
}

int putnstr(const char *str, size_t len) {
  putstr_data_t data;
  data.buf = str;
  data.done = false;

  allow(DRIVER_NUM_CONSOLE, 1, str, len);
  subscribe(DRIVER_NUM_CONSOLE, 1, putstr_cb, &data);
  command(DRIVER_NUM_CONSOLE, 1, len, 0);
  yield_for(&data.done);
  return ret;
}
```

## Inter Process Communication (IPC)

![](ipc.pdf)

## Tock Inter Process Communication Overview

*Servers*

 * Register as an IPC service
 * Call `notify` to trigger callback in connected client
 * Receive a callback when a client calls `notify`

*Clients*

 * Discover IPC services by application name
 * Able to share a buffer with a connected service
 * Call `notify` to trigger callback in connected service
 * Receive a callback when service calls `notify`

## Inter Process Communication API

```c
// discover IPC service by name
// returns error code or PID for service
int ipc_discover(const char* pkg_name);

// shares memory slice at address with IPC service
int ipc_share(int pid, void* base, int len);

// register for callback on server `notify`
int ipc_register_client_cb(int pid, subscribe_cb cb,
                           void* userdata);

// trigger callback in service
int ipc_notify_svc(int pid);

// trigger callback in a client
int ipc_notify_client(int pid);
```

## Check your understanding

1. How does a process perform a blocking operation? Can you draw the flow of
   operations when a process calls `delay_ms(1000)`?

2. How would you write an IPC service to print to the console? Which functions
   would the client need to call?

## Hands-on: Write a BLE environment sensing application

  3. Get an application running on Hail

  4. [Print "Hello World" every second](https://github.com/helena-project/tock/bloc/master/doc/courses/sensys/exercises/app/solutions/repeat-hello.c)

  5. [Extend your app to sample on-board sensors](https://github.com/helena-project/tock/bloc/master/doc/courses/sensys/exercises/app/solutions/sensors.c)

  6. [Extend your app to report through the `ble-env-sense` service](https://github.com/helena-project/tock/bloc/master/doc/courses/sensys/exercises/app/solutions/ble-ess.c)


 - Head to <http://bit.do/tock3> to get started!
 - \tiny ([https://github.com/helena-project/tock/blob/master/doc/courses/sensys/application.md](https://github.com/helena-project/tock/blob/master/doc/courses/sensys/application.md#2-check-your-understanding))

# Part 3: The kernel

## Trusted Computing Base (`unsafe` allowed)

  * Hardware Abstraction Layer

  * Board configuration

  * Event & Process scheduler

  * Rust `core` library

  * Core Tock primitives

```
kernel/
chips/
```

## Capsules (`unsafe` not allowed)

  * Virtualization

  * Peripheral drivers

  * Communication protocols (IP, USB, etc)

  * Application logic

```
capsules/
```

## Constraints

### Small isolation units

Breaking a monolithic component into smaller ones should have low/no cost

### Avoid memory exhaustion in the kernel

No heap. Everything is allocated statically.

### Low communication overhead

Communicating between components as cheap as an internal function call. Ideally
inlined.

## Event-driven execution model

```rust
pub fn main<P, C>(platform: &P, chip: &mut C,
                  processes: &mut [Process]) {
    loop {
        chip.service_pending_interrupts();
        for (i, p) in processes.iter_mut().enumerate() {
            sched::do_process(platform, chip, process);
        }

        if !chip.has_pending_interrupts() {
            chip.prepare_for_sleep();
            support::wfi();
        }
    }
}
```

## Event-driven execution model

```rust
fn service_pending_interrupts(&mut self) {
    while let Some(interrupt) = get_interrupt() {
        match interrupt {
            ASTALARM => ast::AST.handle_interrupt(),
            USART0 => usart::USART0.handle_interrupt(),
            USART1 => usart::USART1.handle_interrupt(),
            USART2 => usart::USART2.handle_interrupt(),
            ...
        }
    }
}
```

## Event-driven execution model

```rust
impl Ast {
    pub fn handle_interrupt(&self) {
        self.clear_alarm();
        self.callback.get().map(|cb| { cb.fired(); });
    }
}
impl time::Client for MuxAlarm {
    fn fired(&self) {
        for cur in self.virtual_alarms.iter() {
            if cur.should_fire() {
                cur.armed.set(false);
                self.enabled.set(self.enabled.get() - 1);
                cur.fired();
            }
        }
    }
}
```

* * *

![Capsules reference each other directly, assisting inlining](rng.pdf)

## Check your understanding

## Hands-on: Write and add a capsule to the kernel

  4. Read the Hail boot sequence in `boards/hail/src/main.rs`

  5. Write a new capsule that prints "Hello World" to the debug
     console.

  6. Extend your capsule to print "Hello World" every second

  7. Extend your capsule to print light readings every second

  8. Extra credit


 - Head to <http://bit.do/tock4> to get started!
 - \tiny ([https://github.com/helena-project/tock/blob/master/doc/courses/sensys/capsule.md](https://github.com/helena-project/tock/blob/master/doc/courses/sensys/capsule.md#2-check-your-understanding))

## We need the Hails back!

But you can take one home with you! Purchase here:


<https://tockos.org/hardware>


Put in "SENSYS17" for $5 off, and "XXX" as the address for local pickup.

## Stay in touch!

<https://www.tockos.org>

<https://github.com/helena-project/tock>

<tock-dev@googlegroups.com>

\#tock on Freenode

### Quick Survey!

- <https://goo.gl/???>
