Secure Bootloader for TockOS
============================

A secure bootloader that verifies the kernel.
Once the bootloader verifies the signature and the version
of the kernel, the board boots into the kernel.

## Building
```bash
cd bootloader-secure/boards/nrf52840dk
make