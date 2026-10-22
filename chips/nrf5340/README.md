# Nordic Semiconductor nRF5340 SoC

Support for the nRF5340 application core, in secure or non-secure state. In
non-secure state, a secure stage must first apply the errata in `crt1` and make
the peripherals, interrupts and pins non-secure; `chip::bases` then picks the
non-secure peripheral aliases at runtime. It reuses the `nrf5x` GPIO, RTC,
SAADC, SPIM, UARTE and USBD drivers, whose registers match the nRF52 versions,
and the `nrf5x` IPC driver. Its USB regulator driver reports VBUS to the USBD,
as POWER does on nRF52. Its SPU driver sets the security of memory, peripherals
and pins, which also controls what the network core can reach. It can hand pins
to the network core and release it from reset.
