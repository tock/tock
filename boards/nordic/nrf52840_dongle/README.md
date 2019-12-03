Platform-Specific Instructions: nRF52840-DK
===================================

The [nRF52840 Dongle](https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52840-DK)
is a platform based around the nRF52840, an SoC with an ARM Cortex-M4 and a BLE radio.
The kit is uses a USB key form factor and includes 1 button, 1 red LED and 1 RGB LED.

## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

JTAG is the preferred method to program. The development kit has the JTAG pins exposed either
through the half-moons pads or, below the PCB, on a Tag-Connect TC2050 connector footprint.
You need to [install JTAG software](../../../doc/Getting_Started.md#optional-requirements).

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
the nRF52840 Dongle.
