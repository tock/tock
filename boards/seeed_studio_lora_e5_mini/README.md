Platform-Specific Instructions: Seeed Studio LoRa E5 Mini
=======================================================

The [Seeed Studio LoRa E5 Mini](https://wiki.seeedstudio.com/LoRa_E5_mini/) is a
development board based around the [STM32WLE5JC](https://www.st.com/en/microcontrollers-microprocessors/stm32wle5jc.html), an SoC with an ARM Cortex-M4
and LoRa SubGhz radio. This kit contains a UART to USB-C adaptor.

## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

OpenOCD is the preferred method to program the board. The development kit does
not have an integrated debugger. An external ST-Link is recommended. This can be
connected to the board's SWD pins for programming.

## Programming the kernel
Once you have all software installed, you should be able to simply run
`make flash` in this directory to install a fresh kernel.

## Programming user-level applications
You can program an application over USB using `tockloader`:

```bash
$ cd libtock-c/examples/<app>
$ make
$ tockloader install
```

## Console output
To view the console output on the Seeed Studio LoRa E5 HF Mini:

```bash
$ tockloader listen
```

