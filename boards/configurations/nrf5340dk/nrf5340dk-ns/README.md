nRF5340-DK in Non-Secure State
==============================

This configuration runs the [`nrf5340dk`](../../../nordic/nrf5340dk/README.md)
kernel in the non-secure state of the application core's TrustZone, behind a
small secure stage in [`secure/`](secure/src/main.rs). The kernel is the same
board code; it detects the security state at boot and uses the non-secure
peripheral addresses.

## Secure Stage

The application core starts in the secure stage, at address 0x0. It:

1. Applies the nRF5340 startup errata, which need secure state.
2. Hands P1.00 and P1.01 to the network core and releases it.
3. Makes the kernel's flash, all RAM, all GPIO pins, and the peripherals the
   kernel uses non-secure in the SPU, and sends their interrupts to
   non-secure state.
4. Sends BusFault, HardFault and NMI to non-secure state, so kernel faults
   reach the kernel's fault handler.
5. Jumps to the kernel at 0x4000 in non-secure state.

No secure code runs after that. The SPU, the CryptoCell, and the NVMC with the
KMU stay secure, so the kernel cannot change the secure stage or write flash.
A fault in the secure stage stops in its `fault_handler`.

## Building and Flashing

Install [probe-rs](https://probe.rs), connect the DK's J-Link USB port, then
run:

```bash
$ make install
```

This builds both images, writes the secure stage at 0x0 and the kernel at
0x4000, and resets the chip. Applications install as for `nrf5340dk`. The
network core runs the [`nrf5340dk-net`](../../../nordic/nrf5340dk-net/README.md)
kernel, as with the secure kernel.

## Memory Layout

| Region       | Address      | Size   | Security   |
|--------------|--------------|--------|------------|
| Secure stage | 0x00000000   | 16 KB  | Secure     |
| Kernel       | 0x00004000   | 240 KB | Non-secure |
| Apps         | 0x00040000   | 768 KB | Non-secure |
| RAM          | 0x20000000   | 504 KB | Non-secure |
| BLE mailbox  | 0x2007E000   | 8 KB   | Non-secure |
