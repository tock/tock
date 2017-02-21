Platform-Specific Instructions: nRF
===================================

The [nRF51 Development
Kit](https://www.nordicsemi.com/eng/Products/nRF51-DK) is a platform
based around the nRF51422, an SoC with an ARM Cortex-M0 and a BLE
radio. The kit is Arduino shield compatible and includes several
buttons.  All code for the kit is compatible with the nRF51822 as
well.

## Necessary tools

There are two ways to program the nRF51DK: JTAG or the mbed file
system. If you choose to use JTAG (the recommended approach), this
requires `JLinkExe`, a JTAG programming application.  It can be
downloaded from [Segger](https://www.segger.com/downloads/jlink), you
want the "Software and Documentation Pack".

## Programming the kernel

### Programming with JTAG (recommended)

The nRF51DK, a Segger JTAG chip is included on the board. Connecting
the board to your computer over USB allows you to program (and debug)
Tock with JTAG. To compile and install the Tock kernel on the nrf51dk
using JTAG, follow the standard Tock instructions (the "Getting
Started" guide).

### Programming with mbed file system (currently unsupported)

The nRF51DK supports ARM mbed development. This means that under Mac OS and 
Windows, plugging the nRF51DK in over USB causes it to appear as a file
system (a storage device). Copying an executable in the ihex format  
named 'firmware.hex' to the device causes it to reprogram. When this
occurs successfully, the nRF51DK will remove itself and re-mount itself.
It does this because it isn't actually a storage device: firmware.hex
doesn't persist, and the only way to make sure the OS doesn't think it's
still there is to disconnect and reconnect.

To program with the mbed file system, run

```bash
$ make TOCK_BOARD=nrf51dk hex
```

This will build `boards/nrf51dk/target/nrf51/release/nrf51dk.hex`. Next,
copy this file to your mbed device, renaming it to `firmware.hex`. 

## Programming user-level applications

To compile and install compile applications for the nrf51dk, follow the
standard Tock instructions (the "Getting Started" guide).

## Debugging

Because the nRF51DK has integrated JTAG support, you can debug it
directly using gdb. In this setup, gdb connects to a process that
gives access to the device over JTAG.  First, create a `.gdbinit` file
in the directory you are debugging from to tell gdb to connect to the
process, load the binary, reset the device, and break on
`reset_handler`, which is the function called on reset/boot. Your
`.gdbinit` file should be as follows:

```gdb
target remote localhost:2331
load
mon reset
break reset_handler
```

Second start the JLink gdb server:

```bash
JLinkGDBServer -device nrf51422 -speed 1200 -if swd -AutoConnect 1 -port 2331
```

Third, start gdb in a new terminal, telling it to use the `.gdbinit`:

```bash
arm-none-eabi-gdb -x .gdbinit boards/nrf51dk/target/nrf51/release/nrf51dk
```

The second parameter (`...nrf51dk`) is the binary image of the kernel,
in ELF format. It's what allows gdb to know the addresses of symbols,
so when you break on a function in knows where to break. This doesn't
change what runs on the board itself, it's just a lookup for gdb to
use when it sends JTAG commands.  Note that you need to use nrf51dk,
*not* nrf51dk.hex or other files. The former is an ELF file that
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
