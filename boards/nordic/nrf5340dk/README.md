Platform-Specific Instructions: nRF5340-DK
==========================================

The [nRF5340 Development Kit](https://www.nordicsemi.com/Products/Development-hardware/nRF5340-DK)
has an nRF5340 SoC with two Cortex-M33 cores. This kernel runs on the
application core in secure state. At boot it hands P1.00 and P1.01 to the
network core and releases it from reset, so the network core can run its own
kernel from [`../nrf5340dk-net`](../nrf5340dk-net/README.md).

## Supported Features

| Feature                | Status |
|------------------------|--------|
| Console (UARTE0)       | Yes    |
| LEDs (4)               | Yes    |
| Buttons (4)            | Yes    |
| Alarm (RTC1)           | Yes    |
| ADC (SAADC, A0 to A5)  | Yes    |
| External flash, KV     | Yes    |
| Process console        | Yes    |
| Network core release   | Yes    |
| USB CDC-ACM console    | Yes    |
| BLE advertising        | Yes    |
| TrustZone (non-secure) | Yes    |

The console is on the J-Link virtual COM port connected to P0.20 (TX) and
P0.22 (RX), at 115200 baud without flow control. Both cores run their LFCLK
from the DK's 32.768 kHz crystal.

BLE advertising uses the radio on the network core, so it needs the
[`../nrf5340dk-net`](../nrf5340dk-net/README.md) kernel there. The two kernels
exchange requests over IPC and a mailbox in the last 8 KB of RAM, which this
kernel leaves out of its own RAM and lets the network core access.

The MX25R6435F external flash is on the QSPI pins, driven by SPIM1. A TicKV
key-value store on its first 32 sectors backs the KV system call driver.

A second console runs over USB CDC-ACM on the nRF5340 USB connector. It is
registered as console instance 1 (driver number `0x1000001`). The cable must be
plugged in when the kernel boots.

The kernel also runs in TrustZone non-secure state, behind a small secure
stage; see the
[`nrf5340dk-ns`](../../configurations/nrf5340dk/nrf5340dk-ns/README.md)
configuration.

## Building and Flashing the Kernel

Install [probe-rs](https://probe.rs), connect the DK's J-Link USB port, then
run:

```bash
$ make install
```

This builds the kernel and writes it to flash at address 0x00000000 with
`probe-rs download --chip nRF5340_xxAA_APPONLY`.

## Memory Layout

| Region      | Address      | Size   |
|-------------|--------------|--------|
| Kernel      | 0x00000000   | 256 KB |
| Apps        | 0x00040000   | 768 KB |
| RAM         | 0x20000000   | 504 KB |
| BLE mailbox | 0x2007E000   | 8 KB   |
