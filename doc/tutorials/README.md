Tock Tutorials
==============

These tutorials walk through how to use the various features of Tock

1. **[Blink an LED](01_running_blink.md)**: Get your first Tock app running.
1. **[Button to Printf()](02_button_print.md)**: Print to terminal in response to button presses.
1. **[BLE Advertisement Scanning](03_ble_scan.md)**: Sense nearby BLE packets.
1. **[Sample Sensors and Use Drivers](04_sensors_and_drivers.md)**: Use syscalls to interact with kernel drivers.
1. **[Inter-process Communication](05_ipc.md)**: Tock's IPC mechanism.

### Board compatiblity matrix

| Tutorial #    | Supported boards                |
|---------------|---------------------------------|
| 1             | [hail](../../boards/hail), [imix](../../boards/imix), [nRF51-DK](../../boards/nrf51dk) & [nRF52-DK](../../boards/nrf52dk) |
| 2             | [hail](../../boards/hail), [imix](../../boards/imix), [nRF51-DK](../../boards/nrf51dk) & [nRF52-DK](../../boards/nrf52dk) |
| 3             | [hail](../../boards/hail) & [imix](../../boards/imix)|
| 4             | [hail](../../boards/hail) & [imix](../../boards/imix)|
| 5             | [hail](../../boards/hail) & [imix](../../boards/imix)|

### Planned

#### Apps
4. App that implements a BLE device.
5. App that uses 15.4.


#### Kernel
1. Capsule for turning on an LED.
2. Using JTAG/GDB.
3. Write a driver that turns on LED and prints a string. Explain driver interface.
4. Loopback SPI.
5. Add a new platform.
