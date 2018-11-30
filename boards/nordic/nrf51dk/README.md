Platform-Specific Instructions: nRF51-DK
===================================

**Not recommended for new projects.** The nRF51 is based on the Cortex-M0 and
does not have full MPU support. To best use Tock features, please use the
nRF52 based boards instead.

The [nRF51 Development
Kit](https://www.nordicsemi.com/eng/Products/nRF51-DK) is a platform
based around the nRF51422, an SoC with an ARM Cortex-M0 and a BLE
radio. The kit is Arduino shield compatible and includes several
buttons.  All code for the kit is compatible with the nRF51822 as
well.

## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

JTAG is the preferred method to program. The development kit has an
integrated JTAG debugger, you simply need to [install JTAG
software](../../doc/Getting_Started.md#optional-requirements).

### Programming the kernel

Once you have all software installed, you should be able to simply run
`make flash` in this directory to install a fresh kernel.

### Programming user-level applications

You can program an application via JTAG and there are two ways to do so:
 1. via `tockloader`:

    ```bash
    $ cd libtock-c/examples/<app>
    $ make
    $ tockloader install --jtag --board nrf51dk --arch cortex-m0 --app-address 0x20000 --jtag-device nrf51
    ```

 2. Alternatively, via `flash`:
    ```bash
    $ cd libtock-c/examples/<app>
    $ make TOCK_BOARD=nrf51dk flash
    ```

If you run this in the application folder, `tockloader` will automatically
find the tab to flash, otherwise you need to specify the path.

### Debugging

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
arm-none-eabi-gdb -x .gdbinit boards/nrf51dk/target/thumbv6m-none-eabi/release/nrf51dk
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

#### Debugging Tricks

When debugging in gdb, we recommend that you use tui:

```gdb
tui enable
layout split
layout reg
```

will give you a 3-window layout, showing the current state of the
main registers, and the current assembly instruction.
Note that Rust supports debugging symbols but there is too little memory available on nrf51dk to enable that.
You have to use the generated assembly.

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
#[no_mangle]
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
