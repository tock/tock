# Nordic Semiconductor nRF5340 SoC

Support for the nRF5340 application core, running in secure state. It reuses the
`nrf5x` GPIO, RTC, UARTE and USBD drivers, whose registers match the nRF52
versions, and the `nrf5x` IPC driver. Its USB regulator driver reports VBUS to
the USBD, as POWER does on nRF52. It can hand pins and a RAM region to the
network core and release it from reset.
