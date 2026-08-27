# RP2xxx

Peripherals shared by the RP2040 and the RP2350. Both chips use the same Arm
PL022 PrimeCell for SPI, so the driver lives here once rather than twice. Each
chip crate supplies what differs: the base addresses and the peripheral clock.
