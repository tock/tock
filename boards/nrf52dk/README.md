Platform-Specific Instructions: nRF52
===================================

The [nRF52 Development
Kit](https://www.nordicsemi.com/eng/Products/Bluetooth-low-energy/nRF52-DK) is a platform
based around the nRF52832, an SoC with an ARM Cortex-M4 and a BLE
radio. The kit is Arduino shield compatible and includes several
buttons.  All code for the kit is compatible with the nRF52810 (__not checked!!__) as
well.

## Necessary tools

There are two ways to program the nRF52DK: JTAG or the mbed file
system. If you choose to use JTAG (the recommended approach), this
requires `JLinkExe`, a JTAG programming application.  It can be
downloaded from [Segger](https://www.segger.com/downloads/jlink), you
want the "Software and Documentation Pack".

## Programming the kernel

### Programming with JTAG (recommended)

The nRF52DK, a Segger JTAG chip is included on the board. Connecting
the board to your computer over USB allows you to program (and debug)
Tock with JTAG. To compile and install the Tock kernel on the nrf52dk
using JTAG, follow the standard Tock instructions (the "Getting
Started" guide).

### Programming with mbed file system (currently unsupported)

Not supported yet

## Programming user-level applications

To compile and install compile applications for the nrf51dk, follow the
standard Tock instructions (the "Getting Started" guide).

## Debugging

Because the nRF52DK has integrated JTAG support, you can debug it
directly using gdb. In this setup, gdb connects to a process that
gives access to the device over JTAG. There exist already prepared scripts
to support debugging in [tock/boards/nrf52dk/jtag](https://github.com/helena-project/tock/tree/master/boards/nrf52dk/jtag). Preferable open two separate terminal
windows and change directory to __"tock/boards/nrf52dk/jtag"__.

In the first window start the JLink gdb server:

```bash
$ ./jdbserver_pca10040.sh
```
Alternative launch it manually:
```bash
$ JLinkGDBServer -device nrf52 -speed 1200 -if swd -AutoConnect 1 -port 2331
```
In the second terminal start gdb and tell it to use the `gdbinit`:

```bash
arm-none-eabi-gdb -x gdbinit_pca10040.jlink 
```

The second parameter (`...nrf52dk`) is the binary image of the kernel,
in ELF format. It's what allows gdb to know the addresses of symbols,
so when you break on a function in knows where to break. This doesn't
change what runs on the board itself, it's just a lookup for gdb to
use when it sends JTAG commands.  Note that you need to use nrf52dk,
*not* nrf52dk.hex or other files. The former is an ELF file that
contains symbols for debugging, the latter is a flat binary file.

Finally, type `continue` or `c` to start execution. The device
will break on entry to `reset_handler`.

### Debugging Tricks

When debugging in gdb, we recommend that you use tui:

```gdb
tui enable
layout split
layout reg
```

will give you a 3-window layout, showing the current state of the
main registers, and the current assembly instruction. Note that currently
Rust does not output debugging symbols that allow you to do source-level
debugging. You have to use the generated assembly.

Since Rust heavily optimized and inlines code, it can be difficult to
understand, from the assembly, exactly where you are in source code. Two
tricks can help in this regard: the ``inline`` and ``no_mangle`` attributes. If you label a function

```rust
#[inline(never)]
```

then Rust will not inline it so you can see calls into it and break on
entry. However, since Rust often emits complex symbol names, you also
might want to use

```rust
$[no_mangle]
```

which will keep the function's symbol identical to the function name.
For example, if you do this:

```rust
#[no_mangle]
#[inline(never)]

fn important_func(&self) -> u32 {
   ...
}
```

then `important_func` will not be inlined and you can break on
`important_func` in gdb. The code itself will still be assembly, but
you can usually piece together what's happening by keeping the source
code alongside. Note that it also helps a lot to use the above
attributes on functions that your function calls -- otherwise figuring
out if the instructions are the function or its callees can be
difficult.
