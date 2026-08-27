QEMU ARM MPS2 Family
====================

ARM provides the `MPS + ANXXX` Cortex-M System Design Kit (CMSDK) reference
platforms for hardware bringup. These are a common peripheral suite coupled with
differing cores. This crate is the shared base platform.

Peripherals
-----------

Current support includes:

- One CMSDK APB UART (of the five present on the machine), used as the
  console/debug UART.
- One CMSDK APB Timer, backing the kernel's `Alarm`/`Time` HIL.
- The `FPGAIO` block's `LED0` register, exposing the machine's two
  simulated LEDs. There's no display on this `-nographic` machine to show
  them; observe their state by reading the register directly through the
  QEMU monitor (`C-a c` to switch from the serial console, then `xp/1xw
  0x40028000`).
- One PL022 SPI controller (the "Shield0" instance), run in hardware
  loopback mode — see the note below.
- The CMSDK APB Watchdog, backing the kernel's `WatchDog` resource.

Not (yet) supported:

- **GPIO.** QEMU emulates all four CMSDK AHB GPIO banks as inert stubs: writes
  are discarded and reads always return 0 (see QEMU's `hw/arm/mps2.c`,
  `create_unimplemented_device(..., "cmsdk-ahb-gpio", ...)`). There is no way
  to observe pin state changes under this QEMU machine, so this board does not
  implement a GPIO capsule.
- **I2C** and **LAN9118 Ethernet controller**: Not yet implemented.

### SPI Loopback-Only Note
None of the machine's five PL022 instances have an SSI slave device attached in
QEMU, so a non-loopback transfer just reads back whatever the empty bus's
default is, not meaningful data. The driver therefore always enables `CR1.LBM`
(loopback) — see `chips/qemu_arm_mps2/src/spi.rs`'s module docs. Chip select is
a zero-sized placeholder for the same reason GPIO is unavailable: there's no
functional GPIO pin to toggle for it, and no real device to select in the first
place.
