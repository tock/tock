Platform-Specific Instructions: nRF52840-DK
===================================

The [nRF52840 Development
Kit](https://www.nordicsemi.com/eng/Products/nRF52840-DK) is a platform
based around the nRF52840, an SoC with an ARM Cortex-M4 and a BLE
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
You can program an application via JTAG using `tockloader`:

    ```bash
    $ cd libtock-c/examples/<app>
    $ make
    $ tockloader install --jlink --board nrf52dk
    ```

## Debugging

See the [nrf52dk README](../nrf52dk/README.md) for information about debugging
the nRF52840dk.
