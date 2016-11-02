Platform-Specific Instructions: nRF
===================================

The [nRF51 Development Kit](https://www.nordicsemi.com/eng/Products/nRF51-DK) is
a platform based around the nRF51422, an SoC with an ARM Cortex-M0 and a BLE
radio. The kit is Arduino shield compatible and includes several buttons.
All code for the kit is compatible with the nRF51822 as well.

## Necessary tools

There are two ways to program the nRF51DK: JTAG or the mbed file system. If
you choose to use JTAG (the recommended approach), this requires `JLinkExe`, 
a JTAG programming application.
It can be downloaded from [Segger](https://www.segger.com/downloads/jlink),
you want the "Software and Documentation Pack".


## Programming the kernel

### Programming with JTAG (recommended)

The nRF51DK, a Segger JTAG chip is included on the board. A USB
connection to a computer allows code to be uploaded to the board.

To program the Tock kernel onto the nRF51 DK, run:

```bash
$ make TOCK_BOARD=nrf51dk program
```

This will build `boards/nrf51dk/target/nrf51/release/nrf51dk` and 
program it on the board using `JLinkExe`.

### Programming with mbed file system

The nRF51DK supports ARM mbed development. This means that under Mac OS and 
Windows, plugging the nRF51DK in over USB causes it to appear as a file
system (a storage device). Copying an executable in the ihex format  
named 'firmware.hex' to the device causes it to reprogram. When this
occurs successfully, the nRF51DK will remove itself and re-mount itself.
It does this because it isn't actually a storage device: firmware.hex
doesn't persist, and the only way to make sure the OS doesn't think it's
still there is to disconnect and reconnect.

To program with the mbed file system, run

```bash
$ make TOCK_BOARD=nrf51dk hex
```

This will build `boards/nrf51dk/target/nrf51/release/nrf51dk.hex`. Next,
copy this file to your mbed device, renaming it to `firmware.hex`. 

## Programming user-level processes

**XXX: TODO**

## Console support

**XXX: Do we even have console support?**

## Debugging

Because the nRF51DK has integrated JTAG support, you can
program it directly using gdb. In this setup, gdb connects
to a process that gives access to the device over JTAG.
First, create a `.gdbinit` file in the directory you are
debugging from to tell gdb to connect to the
process, load the binary, and reset the device:

```target remote localhost:2331
load
mon reset
break main
```

Second start the JLink gdb server:

```bash
JLinkGDBServer -device nrf51422 -speed 1200 -if swd -AutoConnect 1 -port 2331
```

Third, start gdb in a new terminal:

```bash
arm-none-eabi-gdb boards/nrf51dk/target/nrf51/release/nrf51dk
```

Note that you need to use nrf51dk, *not* nrf51dk.hex. The former
is an ELF file that contains symbols for debugging, the latter
is a flat binary file.

Finally, type `continue` or `c` to start execution. The device
will break on entry to main.

