# Raspberry Pi Pico W

The [Raspberry Pi Pico W](https://datasheets.raspberrypi.com/picow/pico-w-product-brief.pdf) is a
board developed by the Raspberry Pi Foundation and is based on the RP2040 chip.

In order to get started, you can follow the [Raspberry Pi Pico guide](../raspberry_pi_pico/README.md).
This board is almost identical to the main one, with the key difference being Wi-Fi capabilities
provided by the on-board Infineon CYW43439 chip.

## External dependencies

This crate uses the external [`tock-firmware-cyw43` crate](https://github.com/tock/firmware) for the WiFi chip firmware on the Pico W.

`cargo-tree` for the `tock-firmware-cyw43` crate outputs:

```
├── tock-firmware-cyw43 v0.1.0 (https://github.com/tock/firmware.git#86ad8b00) # has no sub-dependencies
├── ...
```
