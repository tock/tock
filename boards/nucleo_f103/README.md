Platform-Specific Instructions: Nucleo-F103RB
=============================================

The [Nucleo-F103RB Development Board](http://www.st.com/en/evaluation-tools/nucleo-f103rb.html) is a platform based around the STM32F103RB, a microcontroller with an ARM Cortex-M3.

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md).

JTAG is the preferred method to program. The development board has an integrated JTAG debugger, you simply need to [install OpenOCD](../../doc/Getting_Started.md#optional-requirements).

### Programming the kernel

Once you have all software installed, you should be able to simply run `make flash` in this directory to install a fresh kernel.

### Programming user-level applications

You can program an application via JTAG:
```bash
$ cd userland/examples/<app>
$ make TOCK_BOARD=nucleo_f103 flash
```

## Pin Mapping

| GPIO | Name | Function |
|------|------|----------|
|      | D0   | RX       |
|      | D1   | TX       |
| 0    | D2   |          |
| 1    | D3   |          |
| 2    | D4   |          |
| 3    | D5   |          |
| 4    | D6   |          |
| 5    | D7   |          |
| 6    | D8   |          |
| 7    | D9   |          |
| 8    | D10  |          |
| 9    | D11  |          |
| 10   | D12  |          |
|      | D13  | LED      |
| 11   | D14  |          |
| 12   | D15  |          |
| 13   | A0   |          |
| 14   | A1   |          |
| 15   | A2   |          |
| 16   | A3   |          |
| 17   | A4   |          |
| 18   | A5   |          |
