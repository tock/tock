Teensy 4.0 Development Board
============================

The `teensy4` board supports the Teensy **4.0** development board.
For more information, visit the
[Teensy 4.0 Development Board](https://www.pjrc.com/store/teensy40.html)
product page.

The board may suffice for the [Teensy **4.1** development board](t41), which
shares common pins and a larger amount of flash memory. However, the board
will not contain features that are only available on the Teensy 4.1 development
board, such as extended flash memory and RAM, an on-board SD card, or ethernet.

[t41]: https://www.pjrc.com/store/teensy41.html

Programming Dependencies
------------------------

Before attempting to program your Teensy 4 with Tock and Tock apps, make sure
that you have either

- a build of [`teensy_loader_cli`](https://github.com/PaulStoffregen/teensy_loader_cli), or
- the [Teensy Loader Application](https://www.pjrc.com/teensy/loader.html)

If you're already familiar with programming the Teensy with Arduino tools,
the Teensy Loader Application is already bundled with the Teensyduino add-ons
that you may already be using.

Programming
-----------

From this directory, build the Tock kernel for the Teensy 4:

```bash
$ make
```

Build Tock apps out of tree. Once you've built an app, use
`arm-none-eabi-objcopy` with `--update-section` to create an ELF image that
includes your app(s). The example below combines a prebuilt `blink` example
with the Teensy 4 Tock kernel.

```bash
$ arm-none-eabi-objcopy \
    --update-section .apps=../../../libtock-c/examples/blink/build/cortex-m7/cortex-m7.tbf \
    ../../target/thumbv7em-none-eabi/release/teensy4.elf \
    ../../target/thumbv7em-none-eabi/release/teensy4-app.elf
```

Once you've created a single ELF image, use `arm-none-eabi-objcopy` to turn
that into HEX:

```bash
$ arm-none-eabi-objcopy -O ihex \
    ../../target/thumbv7em-none-eabi/release/teensy4-app.elf \
    ../../target/thumbv7em-none-eabi/release/teensy4-app.hex
```

Finally, use a Teensy programmer to flash `teensy4-app.hex` to your board!

```bash
$ teensy_loader_cli -w -v --mcu=TEENSY40 target/thumbv7em-none-eabi/release/teensy4-app.hex
```

Use the example `Makefile` below to create a build and flash workflow:

```Makefile
APP=../../../libtock-c/examples/blink/build/cortex-m7/cortex-m7.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/teensy4/release/teensy4.elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/teensy4/release/teensy4-app.elf
KERNEL_WITH_APP_HEX=$(TOCK_ROOT_DIRECTORY)/target/teensy4/release/teensy4-app.hex

.PHONY: program
program: target/thumbv7em-none-eabi/release/teensy4.elf
	arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	arm-none-eabi-objcopy -O ihex $(KENERL_WITH_APP) $(KERNEL_WITH_APP_HEX)
    teensy_loader_cli -w -v --mcu=TEENSY40 $(KERNEL_WITH_APP_HEX)
```

For another example, see [`Makefile`](./Makefile).
