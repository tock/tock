
Long Range modem, or LoRa, by Semtech® is a key modern-day IoT enabler. On the PHY layer, it is based on the robust chirp spread spectrum (CSS) while on the MAC layer, it utilizes the LoRaWAN specification which has gained popularity among various online communities like TTN and Helium.

This capsule specifies an interface for mirco-controlling a LoRa modem (SX127X) using SPI and building a network on top of it. So far, the PHY layer has been implemented and tested on MCCI Catena 4610 (SX1276) using Nordic's nRF52840dk. It contains the following files:

 * radio.rs - PHY layer implementation. Uses SPI to communicate with and control the SX127X modem on a slave node. Based on https://github.com/sandeepmistry/arduino-LoRa.
 * driver.rs - Tock-style driver for boards. Once placed on a board, calls to it can be made through a libtock-c app (See examples libtock-c/examples).
