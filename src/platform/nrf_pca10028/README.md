Platform-Specific Instructions: nRF
===================================

The [nRF51 Development Kit](https://www.nordicsemi.com/eng/Products/nRF51-DK) is
a platform based around the nRF51422, an SoC with an ARM Cortex-M0 and a BLE
radio. The kit is Arduino shield compatible and includes several buttons.
All code for the kit is compatible with the nRF51822 as well.

## Necessary tools

Programming the nRF51 DK requires `JLinkExe`, a JTAG programming application.
It can be downloaded from [Segger](https://www.segger.com/downloads/jlink),
you want the "Software and Documentation Pack".


## Programming the kernel

To program the nRF51 DK, a Segger JTAG chip is included on the board. A USB
connection to a computer allows code to be uploaded to the board.

To program the Tock kernel onto the nRF51 DK, run:

```bash
$ make TOCK_PLATFORM=nrf_pca10001 program
```

This will build `build/nrf_pca10001/kernel.elf` and program it on the board
using `JLinkExe`.

## Programming user-level processes

**XXX: TODO**

## Console support

**XXX: Do we even have console support?**

## Debugging

To manually connect to the board with a jlink programmer:

```bash
JLinkExe -device nrf51422 -if swd -speed 1000 -AutoConnect 1
```

To debug with GDB:

```bash
JLinkGDBServer -device nrf51422 -speed 1200 -if swd -AutoConnect 1 -port 2331

(open a new terminal)

arm-none-eabi-gdb <ELF_FILE>
```

You also need a `.gdbinit` file:

```bash
target remote localhost:2331
load
mon reset
break main
```

