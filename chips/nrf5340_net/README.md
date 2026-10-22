# Nordic Semiconductor nRF5340 SoC, Network Core

Support for the nRF5340 network core. It reuses the `nrf5x` BLE radio, GPIO,
RTC and UARTE drivers. The application core must release the network core from
reset, and hand it any pins it uses, before this code runs.
