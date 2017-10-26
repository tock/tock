---
title: Tock Embedded OS Training
date: SOSP 2017
header-includes:
  - \beamertemplatenavigationsymbolsempty
  - \usepackage{pifont}
  - \newcommand{\cmark}{\color{green}\ding{51}}
  - \newcommand{\xmark}{\color{red}\ding{55}}
---

> Please make sure you have completed all of the tutorial pre-requisites.  If
> you prefer, you can download a virtual machine image with all the
> pre-requisites already installed.

<https://github.com/helena-project/tock/tree/master/doc/courses/sosp/README.md>  

> aka  

<tt><https://goo.gl/s17fy8></tt>

## Tock

A secure operating system for microcontrollers

  * Kernel components in Rust

  * Type-safe API for safe driver development

  * Hardware isolated processes for application code

## Use cases

  * Security applications (e.g. authentication keys)

  * Sensor networks

  * Programmable wearables

## Operating System

Tock itself provides services to components in the system:

  * Scheduling

  * Communication

  * Hardware multiplexing

## Low Resource

  * 10's of µA average power draw

  * 10's of kBs of RAM

  * Moderate clock speeds

## Microcontrollers

System-on-a-chip with integrated flash, SRAM, CPU and a bunch of hardware
controllers.

Typically:

  * Communication: UART, SPI, I2C, USB, CAN...

  * External I/O: GPIO, external interrupt, ADC, DAC

  * Timers: RTC, countdown timers

Maybe...

  * Radio (Bluetooth, 15.4)

  * Cryptographic accelerators

  * Other specialized hardware...

## Two types of components: capsules and processes

![](architecture.pdf)

## Two types of scheduling: cooperative and preemptive

![](execution.pdf)

## Agenda Today

  1. Intro to hardware, tools and development environment

  2. Write an end-to-end Bluetooth Low Energy environment sensing application

  3. Add functionality to the Tock kernel

# Part 1: Hardware, tools and development environment

## Hail

![](hail.png)

## Binaries on-board

### Bootloader

### Kernel

### Processes

## Tools

  * `make` (just instrumenting `xargo`)

  * Rust (nightly for `asm!`, compiling `core`, etc)

  * `xargo` to automate compiling base libraries

  * `arm-none-eabi` GCC/LD to link binaries

  * `tockloader` to interact with Hail and the bootloader

## Tools: `tockloader`

Write a binary to a particular address in flash

```bash
$ tockloader flash --address 0x1000 \
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

  1. Compile and flash the kernel

  2. (Optional) Familiarize yourself with `tockloader` commands

    * `uninstall`

    * `list`

    * `erase-apps`

  3. (Optional) Add some other apps from the repo, like `blink` and `sensors`

# Part 2: User space

## System calls

| **Call**  | **Target** | **Description**                  |
|:----------|:----------:|----------------------------------|
| command   | Capsule    | Invoke an operation on a capsule |
| allow     | Capsule    | Share memory with a capsule      |
| subscribe | Capsule    | Register an upcall               |
| memop     | Core       | Modify memory break              |
| yield     | Core       | Bloc until next upcall is ready  |

## C System Calls: `command` & `allow`

```c
int command(uint32_t driver, uint32_t command,
              int arg1, int arg2);

int allow(uint32_t driver, uint32_t allow, void* ptr,
              size_t size);
```

## C System Calls: `subscribe`

```c
typedef void (subscribe_cb)(int, int, int,void*);

int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata);
```

## C System Calls: `yieldk` & `yieldk_for`

```c
void yield(void);

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
  putstr_data_t* data;
  data->buf = str; data->done = false;

  allow(DRIVER_NUM_CONSOLE, 1, str, len);
  subscribe(DRIVER_NUM_CONSOLE, 1, cb, userdata);
  command(DRIVER_NUM_CONSOLE, 1, len, 0);
  yield_for(&data->done);
  return ret;
}
```

## Inter Process Communication (IPC)

![](ipc.pdf)

## Inter Process Communication API

TODO

## Check your understanding

1. How does a process perform a blocking operation? Can you draw the flow of
   operations when a process calls `delay_ms(1000)`?

2. What is a Grant? How do processes interact with grants? Hint: Think about
   memory exhaustion.

## Hands-on: Write a BLE environment sensing application

  1. Get an application running on Hail

  2. [Periodically sample on-board
     sensors](https://gist.github.com/alevy/73d0a1e5c8784df066c86dc5da9d3107)

  3. [Extend your app to report through the `ble-env-sense`
     service](https://gist.github.com/alevy/a274981a29ffc00230aa16101ee0b89f)

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

## The mutable aliases problem

```rust
enum NumOrPointer {
  Num(u32),
  Pointer(&mut u32)
}

// n.b. will not compile
let external : &mut NumOrPointer;
match external {
  Pointer(internal) => {
    // This would violate safety and
    // write to memory at 0xdeadbeef
    *external = Num(0xdeadbeef);
    *internal = 12345;  // Kaboom
  },
  ...
}
```

## Interior mutability to the rescue

| Type           | Copy-only | Mutual exclusion | Opt.      | Mem Opt. |
|----------------|:---------:|:----------------:|:---------:|:--------:|
| `Cell`         | \cmark{}  | \xmark{}         | \cmark{}  | \cmark{} |
| `VolatileCell` | \cmark{}  | \xmark{}         | \xmark{}  | \cmark{} |
| `TakeCell`     | \xmark{}  | \cmark{}         | \xmark{}  | \cmark{} |
| `MapCell`      | \xmark{}  | \cmark{}         | \cmark{}  | \xmark{} |


* * *

```rust
pub struct Fxos8700cq<`a> {
  i2c: &`a I2CDevice,
  state: Cell<State>,
  buffer: TakeCell<`static, [u8]>,
  callback:
    Cell<Option<&`a hil::ninedof::NineDofClient>>,
}

impl<`a> I2CClient for Fxos8700cq<`a> {
  fn cmd_complete(&self, buf: &`static mut [u8]) { ... }
}

impl<`a> hil::ninedof::NineDof for Fxos8700cq<`a> {
  fn read_accelerometer(&self) -> ReturnCode { ... }
}

pub trait NineDofClient {
  fn callback(&self, x: usize, y: usize, z: usize);
}
```

## Check your understanding

  1. What is a `VolatileCell`? Can you find some uses of `VolatileCell`, and do
     you understand why they are needed? Hint: look inside `chips/sam4l/src`.

  2. What is a `TakeCell`? When is a `TakeCell` preferable to a standard
     `Cell`?

## Hands-on: Write and add a capsule to the kernel

  1. Read the Hail boot sequence in `boards/hail/src/main.rs`

  2. [Write a new capsule that prints "Hello World" to the debug
     console.](https://gist.github.com/alevy/56b0566e2d1a6ba582b7d4c09968ddc9)

  3. [Extend your capsule to print "Hello World" every second](https://gist.github.com/alevy/798d11dbfa5409e0aa56d870b4b7afcf)

  4. [Extend your capsule to read and report the accelerometer](https://gist.github.com/alevy/73fca7b0dddcb5449088cebcbfc035f1#file-boot_sequence.rs)

  5. Extra Credit: Write a 9dof virtualization capsule.

