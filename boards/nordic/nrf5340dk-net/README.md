Platform-Specific Instructions: nRF5340-DK Network Core
=======================================================

This board runs Tock on the network core (Cortex-M33, 64 MHz) of the
[nRF5340 Development Kit](https://www.nordicsemi.com/Products/Development-hardware/nRF5340-DK).
It is a separate kernel from the application core kernel in
[`../nrf5340dk`](../nrf5340dk/README.md).

The network core only starts once the application core releases it from reset.
The `nrf5340dk` kernel does this at boot, after handing P1.00 and P1.01 to the
network core. Flash that kernel first.

## Supported Features

| Feature           | Status |
|-------------------|--------|
| Console (UARTE0)  | Yes    |
| Alarm (RTC1)      | Yes    |
| Process console   | Yes    |
| BLE radio server  | Yes    |
| RNG               | Yes    |
| Temperature       | Yes    |

The console is on P1.01 (TX) and P1.00 (RX), at 115200 baud without flow
control. These pins are connected to one of the DK's J-Link virtual COM ports;
the application core console is on the other one.

The kernel drives the radio for the application core kernel, whose processes
use the BLE advertising driver. The two kernels exchange requests over IPC and
a mailbox in the last 8 KB of application RAM (0x2007E000), which the
application core shares with this core.

## Building and Flashing the Kernel

Install [probe-rs](https://probe.rs), connect the DK's J-Link USB port, then
run:

```bash
$ make install
```

This writes the kernel to the network core's flash with
`probe-rs download --chip nRF5340_xxAA`, then resets the chip through the
application core. The network core's debug port only answers while it is
released from reset, so the application core kernel must already be running.

## Memory Layout

| Region      | Address      | Size   |
|-------------|--------------|--------|
| Kernel      | 0x01000000   | 128 KB |
| Apps        | 0x01020000   | 128 KB |
| RAM         | 0x21000000   | 64 KB  |
