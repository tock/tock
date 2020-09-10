Platforms Supported by Tock
===========================

The `/boards` directory contains the physical hardware platforms
that Tock supports.

| Board                                                                | Architecture    | MCU            | Interface  | App deployment | QEMU Support?     |
|----------------------------------------------------------------------|-----------------|----------------|------------|----------------|-------------------|
| [Hail](hail/README.md)                                               | ARM Cortex-M4   | SAM4LC8BA      | Bootloader | tockloader     | No                |
| [Imix](imix/README.md)                                               | ARM Cortex-M4   | SAM4LC8CA      | Bootloader | tockloader     | No                |
| [Nordic nRF52-DK](nordic/nrf52dk/README.md)                          | ARM Cortex-M4   | nRF52832       | jLink      | tockloader     | No                |
| [Nordic nRF52840-DK](nordic/nrf52840dk/README.md)                    | ARM Cortex-M4   | nRF52840       | jLink      | tockloader     | No                |
| [Nordic nRF52840-Dongle](nordic/nrf52840_dongle/README.md)           | ARM Cortex-M4   | nRF52840       | jLink      | tockloader     | No                |
| [ACD52832](acd52832/README.md)                                       | ARM Cortex-M4   | nRF52832       | jLink      | tockloader     | No                |
| [Nano 33 BLE](nano33ble/README.md)                                   | ARM Cortex-M4   | nRF52840       | BOSSA      | bossac         | No                |
| [ST Nucleo F446RE](nucleo_f446re/README.md)                          | ARM Cortex-M4   | STM32F446      | openocd    | custom         | #1827             |
| [ST Nucleo F429ZI](nucleo_f429zi/README.md)                          | ARM Cortex-M4   | STM32F429      | openocd    | custom         | #1827             |
| [STM32F3Discovery kit](stm32f3discovery/README.md)                   | ARM Cortex-M4   | STM32F303VCT6  | openocd    | custom         | #1827             |
| [STM32F412G Discovery kit](stm32f412gdiscovery/README.md)            | ARM Cortex-M4   | STM32F412G     | openocd    | custom         | #1827             |
| [SparkFun RedBoard Artemis Nano](redboard_artemis_nano/README.md)    | ARM Cortex-M4   | Apollo3        | custom     | custom         | No                |
| [SiFive HiFive1](hifive1/README.md)                                  | RISC-V          | FE310-G000     | openocd    | tockloader     | [Yes (5.1)][qemu] |
| [Digilent Arty A-7 100T](arty_e21/README.md)                         | RISC-V RV32IMAC | SiFive E21     | openocd    | tockloader     | No                |
| [Nexys Video OpenTitan](opentitan/README.md)                         | RISC-V RV32IMC  | EarlGrey       | custom     | custom         | [Yes (5.1)][qemu] |

# Out of Tree Boards

Some projects that use Tock maintain their own board definitions outside the
Tock repository.

| Project                                                  | Boards                       | Architecture   | MCU      | Build System  |
|----------------------------------------------------------|------------------------------|----------------|----------|---------------|
| [OpenSK](https://github.com/google/opensk)               | nRF52840-DK, nRF52840-Dongle | ARM Cortex-M4  | nRF52840 | Python script |
| [OpenTitan](https://github.com/lowrisc/opentitan)        | OpenTitan                    | RISC-V RV32IMC | EarlGrey | Meson         |
| [Tock-on-Titan](https://github.com/google/tock-on-titan) | golf2, papa                  | ARM Cortex-M3  | H1       | Makefiles     |
