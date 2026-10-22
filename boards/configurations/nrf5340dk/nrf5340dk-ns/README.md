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
2. Connects the 32.768 kHz crystal, whose pins only secure code can select.
3. Hands P1.00 and P1.01 to the network core and releases it.
4. Makes the kernel's flash and RAM, all GPIO pins, and the peripherals the
   kernel uses non-secure in the SPU, and sends their interrupts to
   non-secure state. The first 8 KB of RAM, its stack, stays secure.
5. Makes its last 32 bytes of flash, the CryptoCell gateway, non-secure
   callable.
6. Sends BusFault, HardFault and NMI to non-secure state, so kernel faults
   reach the kernel's fault handler.
7. Jumps to the kernel at 0x4000 in non-secure state.

After that, secure code runs only when the kernel calls the gateway. The SPU,
the CryptoCell, and the NVMC with the KMU stay secure, so the kernel cannot
change the secure stage or write flash. A fault in the secure stage stops in
its `fault_handler`.

## CryptoCell Gateway

The CryptoCell-312 accepts only secure accesses, so the secure stage runs it
for the kernel. The gateway has four entry points, 8 bytes apart from
0x3FE0: AES-128 (ECB, CBC or CTR), SHA-256 over whole blocks, the final
SHA-256 block, and 192 bits from the TRNG. Each takes pointers, which the
secure stage checks are in the kernel's RAM, or for data it only reads, the
kernel's or apps' flash. It returns 0 on success and 1 for an invalid
argument. `nrf5340::cryptocell::gateway` defines the calls.

The kernel's RNG and SHA-256 system call drivers use the gateway. Each call
runs to completion while the kernel waits; kernel interrupts still preempt
it. A TRNG read takes two samples, as the first after a start is
dropped.

## Limits

The SPU gives each bus master other than the application core one security
attribute, and the network core is non-secure. It can therefore reach all of
the kernel's RAM, not only the BLE mailbox. Isolating the kernel from it would
need the kernel itself in secure state.

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
| Gateway      | 0x00003FE0   | 32 B   | NSC        |
| Kernel       | 0x00004000   | 240 KB | Non-secure |
| Apps         | 0x00040000   | 768 KB | Non-secure |
| Secure stack | 0x20000000   | 8 KB   | Secure     |
| Kernel RAM   | 0x20002000   | 496 KB | Non-secure |
| BLE mailbox  | 0x2007E000   | 8 KB   | Non-secure |

NSC is non-secure callable: secure, but non-secure code may call the gateway's
entry points in it.
