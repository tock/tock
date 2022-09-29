# BBC HiFive Inventor - FE310-G003 RISC-V Board

<img src="https://www.hifiveinventor.com/image/hifive/support/gs-2-overview.png" width="35%">

The [BBC HiFive Inventor](https://www.hifiveinventor.com/) is a
board based on the SiFive FE310-G003 chip built around the
[E31 Core](https://www.sifive.com/cores/e31). It includes the following
peripherals:

- 6x8 RGB LED Matrix
- Light Sensor
- LSM303AGR compass and accelerometer
- Bluetooth and Wi-Fi connectivity co-processor

**At present, the peripherals are not set up.** We are waiting for the schematic.

## Programming

When using `tockloader` use settings for board `hifive1b` since they are the
same (same debugger, same kernel and program memory).

### Using J-Link

Running `make flash-jlink` should load the kernel onto the board. It requires
you install [J-Link](https://www.segger.com/downloads/jlink#J-LinkSoftwareAndDocumentationPack).
Make sure that the `JLinkExe` executable is accessible starting from your
`PATH` variable.

If need, use `gdb` to debug the kernel. Start a custom gdb server with
`JLinkGDBServerExe`, or use the following configuration:

```bash
$ JLinkGDBServerCLExe -select USB -device FE310 -endian little -if JTAG -speed 1200 -noir -noLocalhostOnly
```

### Other tools

I would also like to note that `openocd` support is in developement.
[A update](https://review.openocd.org/c/openocd/+/7135) for adding the flash
ISSI IS25LQ040 chip is on it's way.

Running in QEMU has not been tested, yet.
