nRF52840-DK USB CDC Console Test Board
===================================

This is a test kernel for the nRF52840DK that, in addition to the standard
UART console provided by `nrf52840dk-test-base`, sets up the nRF52840's USB
controller and the CDC-ACM stack to expose a second, independent serial
console to userspace over USB.

The second console is at driver number `CONSOLE2_DRIVER_NUM | 0x0100_0000`.

This is for simpler testing of the CDC over USB stack.
