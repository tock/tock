# Nordic Semiconductor nRF5340 SoC

Support for the nRF5340 application core, running in secure state. It reuses the
`nrf5x` GPIO, RTC and UARTE drivers, whose registers match the nRF52 versions.
It can hand pins to the network core and release it from reset.
