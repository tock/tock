Platform-Specific Instructions: nRF52-DK
===================================

The [nRF52 Development
Kit](https://www.nordicsemi.com/eng/Products/Bluetooth-low-energy/nRF52-DK) is a platform
based around the nRF52832, an SoC with an ARM Cortex-M4 and a BLE
radio. The kit is Arduino shield compatible and includes several
buttons.

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md)

JTAG is the preferred method to program. The development kit has an
integrated JTAG debugger, you simply need to [install JTAG
software](../../doc/Getting_Started.md#optional-requirements).

## Programming the kernel
Once you have all software installed, you should be able to simply run
make flash in this directory to install a fresh kernel.

## Programming user-level applications
You can program an application via JTAG and there are two ways to do so:
 1. via `tockloader`:

    ```bash
    $ cd userland/examples/<app>
    $ make
    $ tockloader install --jtag --board nrf52dk --arch cortex-m4 --app-address 0x20000 --jtag-device nrf52
    ```

 2. Alternatively, via `flash`.
    ```bash
    $ cd userland/examples/<app>
    $ make TOCK_BOARD=nrf52dk flash
    ```

To compile and install compile applications for the nrf52dk, follow the
standard Tock instructions (the "Getting Started" guide).

## Debugging

Because the nRF52DK has integrated JTAG support, you can debug it
directly using gdb. In this setup, gdb connects to a process that
gives access to the device over JTAG. </br>

There already exist prepared scripts to support debugging in [tock/boards/nrf52dk/jtag](https://github.com/helena-project/tock/tree/master/boards/nrf52dk/jtag). </br>
Open two separate terminals go to the jtag directory by: </br>

```bash
$ cd tock/boards/nrf52dk/jtag
```

In the first window start the JLink gdb server by:

```bash
$ ./jdbserver_pca10040.sh
```
Alternatively launch it manually by:
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
