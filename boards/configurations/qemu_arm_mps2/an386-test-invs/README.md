QEMU MPS2 AN386 Isolated Nonvolatile Storage Test Board
========================================================

This is a minimal kernel for testing isolated nonvolatile storage on the
QEMU ARM MPS2 AN386 (Cortex-M4) machine.

The AN386 has no real flash controller, so this configuration uses a
RAM-backed stand-in for nonvolatile storage
(`capsules_extra::ram_nonvolatile_storage`): contents behave like
nonvolatile storage from an app's perspective, but do not survive a reset.

Apps are granted storage permissions individually (each app can only access
its own storage region), assigned by a name-based AppID and a null
credentials checker -- see the [family README](../../../qemu_arm_mps2/mps2_base/README.md)
for how to run apps under QEMU.

This configuration also exposes a second `Console`, at driver number
`0x01000001`, over UART1 -- `mps2_base` only wires up UART0, so this board
creates a separate `UartMux`/`Console` for UART1 itself. `make run`/`make
run-app` bridge it to a host pseudo-terminal (see the `Makefile`'s
`QEMU_BASE_CMDLINE` comment); QEMU prints its path on startup, e.g.:

```
char device redirected to /dev/ttys004 (label serial1)
```

Connect to it from another terminal with, e.g., `screen /dev/ttys004
115200`.
