# Infineon CYW4343* Wi-Fi chip driver

This driver provides support for interacting with Infineon CYW4343* FullMAC Wi-Fi chips. The implementation is split into two layers: the bus/interface layer and the driver layer.

Datasheet can be found [here](https://www.mouser.com/datasheet/2/196/Infineon_CYW43439_DataSheet_v03_00_EN-3074791.pdf).

## Bus

The CYW43439 datasheet specifies two possible interfaces for communicating with the chip: SDIO and generic SPI (Section 4: WLAN System interfaces). There are 3 functions supported by the SDIO interface
and the gSPI interface (in the gSPI implementation the functions are encoded as 2 bits in the command word as seen in Figure 12):

- Function 0 (F0) implies that the host will read/write to a specific SDIO or SPI register (e.g. enabling interrupts)
- Function 1 (F1) implies that the host will read/write to the backplane address space (e.g. loading the firmware, resetting/disable blocks such as the WLAN application core)
- Function 2 (F2) implies the the host will read/write a WLAN packet

The bus abstraction hides these functions from the driver.

## Driver

The driver is only "aware" of the WLAN packets (Ethernet frames or IOCTLs), which are encapsulated using the Broadcom SDPCM protocol. The driver handles the SDPCM headers and uses the `write_bytes`/`read_bytes` methods from the `Bus` interface to transmit/receive the data as F2 packets.
