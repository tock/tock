Platform-Specific Instructions: nRF52840-Dongle
===================================

The [nRF52840 Dongle](https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52840-Dongle)
is a platform based around the nRF52840, an SoC with an ARM Cortex-M4 and a BLE radio.
The kit is uses a USB key form factor and includes 1 button, 1 red LED and 1 RGB LED.

## Getting Started

To program the nRF52840 Dongle with Tock, you will need a JLink JTAG device and the
appropriate cables. An example setup is:

- [JLink JTAG Device](https://www.digikey.com/product-detail/en/segger-microcontroller-systems/8.08.90-J-LINK-EDU/899-1008-ND/2263130)
- [ARM to TagConnect Adapter](https://www.digikey.com/product-detail/en/tag-connect-llc/TC2050-ARM2010/TC2050-ARM2010-ND/3528170)
- [10pin TagConnect Cable](https://www.digikey.com/product-detail/en/tag-connect-llc/TC2050-IDC-NL/TC2050-IDC-NL-ND/2605367)

Then, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

JTAG is the preferred method to program. The development kit has the JTAG pins exposed either
through the half-moons pads or, below the PCB, on a Tag-Connect TC2050 connector footprint.
You need to [install JTAG software](../../../doc/Getting_Started.md#optional-requirements).

## Programming the kernel
Once you have all software installed, you should be able to simply run
make flash in this directory to install a fresh kernel.

## Programming user-level applications
You can program an application via JTAG using `tockloader`:

```shell
$ cd libtock-c/examples/<app>
$ make
$ tockloader install --jlink --board nrf52dk
```

## Debugging

See the [nrf52dk README](../nrf52dk/README.md) for information about debugging
the nRF52840 Dongle.
