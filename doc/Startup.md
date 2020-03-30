Tock Startup
============

This document walks through how all of the components of Tock start up.

<!-- npm i -g markdown-toc; markdown-toc -i Startup.md -->

<!-- toc -->

- [Optional Bootloader](#optional-bootloader)
- [Tock first instructions](#tock-first-instructions)
  * [ARM Vector Table and IRQ table](#arm-vector-table-and-irq-table)
  * [RISC-V](#risc-v)
- [Reset Handler](#reset-handler)
  * [Memory Initialization](#memory-initialization)
  * [RISC-V Trap setup](#risc-v-trap-setup)
  * [MCU Setup](#mcu-setup)
  * [Peripheral and Capsule Initialization](#peripheral-and-capsule-initialization)
- [Application Startup](#application-startup)
- [Scheduler Execution](#scheduler-execution)

<!-- tocstop -->

When a microcontroller boots (or resets, or services an interrupt) it loads an
address for a function from a table indexed by interrupt type known as the
_vector table_. The location of the vector table in memory is chip-specific,
thus it is placed in a special section for linking.

Cortex-M microcontrollers expect a vector table to be at address 0x00000000.
This can either be a software bootloader or the Tock kernel itself.

RISC-V gives hardware designers a great deal of design freedom for how
booting works. Typically, after coming out of reset, a RISC-V processor
will start executing out of ROM but this may be configurable. The HiFive1
board, for example, supports booting out ROM, One-Time programmable (OTP)
memory or a QSPI flash controller.

## Optional Bootloader

Many Tock boards (including Hail and imix) use a software bootloader that
executes when the MCU first boots. The bootloader provides a way to talk to the
chip over serial and to load new code, as well as potentially other
administrative tasks. When the bootloader has finished, it tells the MCU that
the vector table has moved (to a known address), and then jumps to a new
address.

## Tock first instructions

### ARM Vector Table and IRQ table

On ARM chips, Tock splits the vector table into two sections, `.vectors` which
hold the first 16 entries, common to all ARM cores, and `.irqs`, which is
appended to the end and holds chip-specific interrupts.

In the source code then, the vector table will appear as an array that is
marked to be placed into the `.vectors` section.

In Rust, a vector table will look something like this:
```rust
#[link_section=".vectors"]
#[used] // Ensures that the symbol is kept until the final binary
pub static BASE_VECTORS: [unsafe extern fn(); 16] = [
    _estack,                        // Initial stack pointer value
    tock_kernel_reset_handler,      // Tock's reset handler function
    /* NMI */ unhandled_interrupt,  // Generic handler function
    ...
```

In C, a vector table will look something like this:

```c
__attribute__ ((section(".vectors")))
interrupt_function_t interrupt_table[] = {
        (interrupt_function_t) (&_estack),
        tock_kernel_reset_handler,
        NMI_Handler,
```

At the time of this writing (November 2018), typical chips (like the `sam4l` and
`nrf52`) use the same handler for all interrupts, and look something like:

```rust
#[link_section = ".vectors"]
#[used] // Ensures that the symbol is kept until the final binary
pub static IRQS: [unsafe extern "C" fn(); 80] = [generic_isr; 80];
```

### RISC-V

All RISC-V boards are linked to run the `_start` function as the first
function that gets run before jumping to `reset_handler`. This is currently
inline assembly as of this writing:

```rust
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[link_section = ".riscv.start"]
#[export_name = "_start"]
#[naked]
pub extern "C" fn _start() {
    unsafe {
        asm! ("

```

## Reset Handler

On boot, the MCU calls the reset handler function defined in vector
table. In Tock, the implementation of the reset handler function is
platform-specific and defined in `boards/<board>/src/main.rs` for each
board.

### Memory Initialization

The first operation the reset handler does is setup the kernel's memory by
copying it from flash. For the SAM4L, this is in the `init()` function in
`chips/sam4l/src/lib.rs`.

### RISC-V Trap setup

The `mtvec` register needs to be set on RISC-V to handle traps. Setting
of the vectors is handled by chip specific functions. The common RISC-V trap
handler is `_start_trap`, defined in `arch/rv32i/src/lib.rs`. 

### MCU Setup

Any normal MCU initialization is typically handled next. This includes
things like enabling the correct clocks or setting up DMA channels.

### Peripheral and Capsule Initialization

After the MCU is set up, `reset_handler` initializes peripherals and
capsules. Peripherals are on-chip subsystems, such as UARTs, ADCs, and
SPI buses; they are chip-specific code that read and write
memory-mapped I/O registers and are found in the corresponding `chips`
directory. While peripherals are chip-specific implementations, they
typically provide hardware-independent traits, called hardware
independent layer (HIL) traits, found in `kernel/src/hil`.

Capsules are software abstractions and services; they are
chip-independent and found in the `capsules` directory. For example,
on the imix and hail platforms, the SAM4L SPI peripheral is
implemented in `chips/sam4l/src/spi.rs`, while the capsule that
virtualizes the SPI so multiple capsules can share it is in
`capsules/src/virtual_spi.rs`.  This virtualizer can be
chip-independent because the chip-specific code implements the SPI HIL
(`kernel/src/hil/spi.rs`). The capsule that implements a system call
API to the SPI for processes is in `capsules/src/spi.rs`.

Boards that initialize many peripherals and capsules use the `Component`
trait to encapsulate this complexity from `reset_handler`. The `Component`
trait (`kernel/src/component.rs`) encapsulates any initialization a
particular peripheral, capsule, or set of capsules need inside a
call to the function `finalize()`. Changing what the build of the kernel
includes involve changing just which Components are initialized, rather
than changing many lines of `reset_handler`. Components are typically
found in the `components` crate in the `/boards` folder, but may also be
board-specifc and found inside a `components` subdirectory of the board
directory, e.g. `boards/imix/src/imix_components`.

## Application Startup

Once the kernel components have been setup and initialized, the applications
must be loaded. This procedure essentially iterates over the processes stored in
flash, extracts and validates their Tock Binary Format header, and adds them to
an internal array of process structs.

An example version of this loop is in `kernel/src/process.rs` as the
`load_processes()` function. After setting up pointers, it tries to create a
process from the starting address in flash and with a given amount of memory
remaining. If the header is validated, it tries to load the process into memory
and initialize all of the bookkeeping in the kernel associated with the process.
This can fail if the process needs more memory than is available on the chip. If
the process is successfully loaded the kernel importantly notes the address of
the application's entry function which is called when the process is started.

The load process loop ends when the kernel runs out of statically allocated
memory to store processes in, available RAM for processes, or there is an
invalid TBF header in flash.

## Scheduler Execution

Tock provides a `Scheduler` trait that serves as an abstraction to allow for
plugging in different scheduling algorithms. Schedulers should be initialized
at the end of the reset handler.
The final thing that the reset handler must do is call `scheduler.kernel_loop()`.
This starts the Tock scheduler and the main operation of the kernel.
