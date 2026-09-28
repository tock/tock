Raspberry Pi Pico 2 W - RP2350
==============================

The [Raspberry Pi Pico 2 W](https://datasheets.raspberrypi.com/picow/pico-2-w-datasheet.pdf)
is the Raspberry Pi Pico 2 with an Infineon CYW43439 radio added, giving it
2.4 GHz 802.11n WiFi and Bluetooth 5.2. The processor and memory are the same RP2350 the
Pico 2 has, so this board is built on `raspberry_pi_pico_2` and differs from it
only where the radio takes something over.

## What this board supports

Everything `raspberry_pi_pico_2` supports, plus the CYW43439 radio: the
console, the alarm, GPIO, IPC and WiFi. Bluetooth is not wired up.

The radio is reached over half duplex SPI, driven by a PIO state machine with
DMA underneath it rather than by an SPI peripheral, because it is wired to four
ordinary pins. Its firmware, NVRAM and CLM blobs are linked into the kernel
image, which is why this board's `layout.ld` gives itself 575K of rom where the
plain Pico 2 takes 255K. Applications load at `0x10090000` here.

Nothing drives the radio at boot. The `wifi` syscall driver starts it when an
application asks, so a kernel with no application loaded never talks to it.

The power pin is a caveat rather than a guarantee. `make_output` enables the
output without writing a value, so GPIO 23 drives whatever `GPIO_OUT` already
held: low out of a cold reset, but not necessarily after a warm one from a
kernel that had powered the radio. `boards/raspberry_pi_pico_w` behaves the
same way.

The GPIO driver exposes fewer pins than it does on a Pico 2. GPIO 23, 24, 25
and 29 are the radio's power, gSPI data, chip select and gSPI clock, so they
are not offered to processes: a process that could drive them could power the
radio up underneath the kernel, or corrupt a transfer once the radio is
running. Userspace gets 2 to 22 and 26 to 28.

There is no LED driver. On a Pico 2 the LED is GPIO 25; on a Pico 2 W that pin
is the radio's chip select, and the LED that does exist is pin 0 of the
CYW43439, which cannot be reached until the radio is running. A kernel panic
therefore prints over the console and halts rather than blinking, which is the
one behaviour that differs from `raspberry_pi_pico_2` at runtime.

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md)

## Installing picotool

The RP2350 uses UF2 files for flashing. Tock compiles to an ELF file.
The `picotool` utility is needed to transform the Tock ELF file into an UF2 file.

To install `picotool`, check the instructions from their GitHub [page](https://github.com/raspberrypi/picotool).

## Flashing the kernel

### Using the bootloader

To enter BOOTSEL mode, press the BOOTSEL button and hold it while you connect
the other end of the micro USB cable to your computer. Then `cd` into
`boards/raspberry_pi_pico_2_w` and run:

```bash
$ make flash
```

> Note: The Makefile provides the BOOTSEL_FOLDER variable that points towards the mount point of
> the Pico 2 W flash drive. By default, this is located in `/run/media/$(USER)/RP2350`. This might
> be different on several systems, make sure to adjust it.

### Using a debug probe

With SWD wired to the board's debug header:

```bash
$ make flash-openocd
```

## Flashing an app

Apps are built out-of-tree. Once an app is built, pass its TBF file:

```bash
$ APP="<path to app's tbf file>" make program
```
