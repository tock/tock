Platform-Specific Instructions: ACD52832
========================================

![acd52832](http://aconno.de/wp-content/uploads/2016/03/img_modul-08.png)

The [ACD52832](http://aconno.de/acd52832/) is a development board based on the
nRF52832. It includes:

- E-ink display
- LEDs
- Buttons
- Buzzer
- Temperature sensor
- IR LED
- Joystick
- Potentiometer
- 2x 60V DC Relays
- 9DOF sensor
- Light sensor

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md)

JTAG is the preferred method to program. The development kit has an
integrated JTAG debugger, you simply need to [install JTAG
software](../../doc/Getting_Started.md#optional-requirements).

## Programming the kernel
Once you have all software installed, you should be able to simply run
make flash in this directory to install a fresh kernel.

## Programming user-level applications
You can program an application via JTAG using Tockloader:

```bash
$ tockloader install --jlink --board nrf52dk
```

## TODO

- Implement panic! print in io.rs.


