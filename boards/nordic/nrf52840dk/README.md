Platform-Specific Instructions: nRF52840-DK
===================================

The [nRF52840 Development
Kit](https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52840-DK) is a platform
based around the nRF52840, an SoC with an ARM Cortex-M4 and a BLE
radio. The kit is Arduino shield compatible and includes several
buttons.

## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

JTAG is the preferred method to program. The development kit has an
integrated JTAG debugger, you simply need to [install JTAG
software](../../../doc/Getting_Started.md#loading-the-kernel-onto-a-board).

## Programming the kernel
Once you have all software installed, you should be able to simply run
make flash in this directory to install a fresh kernel.

## Programming user-level applications
You can program an application over USB using the integrated JTAG and `tockloader`:

```bash
$ cd libtock-c/examples/<app>
$ make
$ tockloader install --jlink --board nrf52dk
```

The same options (`--jlink --board nrf52dk`) must be passed for other tockloader commands
such as `erase-apps` or `list`.

Viewing console output on the nrf52840dk is slightly different from other boards. You must use
```bash
$ tockloader listen
```
**followed by a press of the reset button** in order to view console output starting from the boot
sequence. Notably, you should not
pass the `--jlink` option to `tockloader listen`.

## Console output

This board supports two methods for writing messages to a console interface
(console driver for applications as well as debug statements in the kernel).

By default, messages are written to a UART interface over the GPIO pins `P0.05`
to `P0.08` (see the [main.rs](src/main.rs) file).

If you don't have any UART cables or want to use a different interface, there is
also a console over the Segger RTT protocol. This only requires a micro-USB
cable on the USB debugging port (the same used to flash Tock on the board), and
is enabled by setting the `USB_DEBUGGING` constant to `true` in the
[main.rs](src/main.rs) file.
This disables the UART interface.

For instructions about how to receive RTT messages on the host, see the
[corresponding capsule](../../../capsules/src/segger_rtt.rs).

## Debugging

See the [nrf52dk README](../nrf52dk/README.md) for information about debugging
the nRF52840dk.
