Platform-Specific Instructions: nRF5340-DK
==========================================

The [nRF5340 Development Kit](https://www.nordicsemi.com/Products/Development-hardware/nRF5340-DK)
has an nRF5340 SoC with two Cortex-M33 cores. Tock runs on the application core
in secure state. The network core is held in reset.

## Supported Features

| Feature                | Status |
|------------------------|--------|
| Console (UARTE0)       | Yes    |
| LEDs (4)               | Yes    |
| Buttons (4)            | Yes    |
| Alarm (RTC1)           | Yes    |
| Process console        | Yes    |
| Network core, BLE      | No     |
| TrustZone (non-secure) | No     |

The console is on the J-Link virtual COM port connected to P0.20 (TX) and
P0.22 (RX), at 115200 baud without flow control. The LFCLK runs from the
internal RC oscillator.

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
| RAM         | 0x20000000   | 512 KB |
