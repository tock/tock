Platforms Supported by Tock
===========================

The `/boards` directory contains the physical hardware platforms
that Tock supports.

| Board                                                                | Architecture    | MCU            | Interface  | App deployment | QEMU Support? |
|----------------------------------------------------------------------|-----------------|----------------|------------|----------------|---------------|
| [Hail](hail/README.md)                                               | ARM Cortex-M4   | SAM4LC8BA      | Bootloader | tockloader     | No            |
| [Imix](imix/README.md)                                               | ARM Cortex-M4   | SAM4LC8CA      | Bootloader | tockloader     | No            |
| [Nordic nRF52-DK](nordic/nrf52dk/README.md)                          | ARM Cortex-M4   | nRF52832       | jLink      | tockloader     | No            |
| [Nordic nRF52840-DK](nordic/nrf52840dk/README.md)                    | ARM Cortex-M4   | nRF52840       | jLink      | tockloader     | No            |
| [Nordic nRF52840-Dongle](nordic/nrf52840_dongle/README.md)           | ARM Cortex-M4   | nRF52840       | jLink      | tockloader     | No            |
| [ACD52832](acd52832/README.md)                                       | ARM Cortex-M4   | nRF52832       | jLink      | tockloader     | No            |
| [TI LAUNCHXL-CC26x2](launchxl/README.md)                             | ARM Cortex-M4   | CC2652R        | openocd    | tockloader     | No            |
| [ST Nucleo F446RE](nucleo_f446re/README.md)                          | ARM Cortex-M4   | STM32F446      | openocd    | custom         | #1827         |
| [ST Nucleo F429ZI](nucleo_f429zi/README.md)                          | ARM Cortex-M4   | STM32F429      | openocd    | custom         | #1827         |
| [STM32F3Discovery kit](stm32f3discovery/README.md)                   | ARM Cortex-M4   | STM32F303VCT6  | openocd    | custom         | #1827         |
| [SparkFun RedBoard Artemis Nano](redboard_artemis_nano/README.md)    | ARM Cortex-M4   | Apollo3        | custom     | custom         | No            |
| [SiFive HiFive1](hifive1/README.md)                                  | RISC-V          | FE310-G000     | openocd    | tockloader     | Tock fork     |
| [Digilent Arty A-7 100T](arty_e21/README.md)                         | RISC-V RV32IMAC | SiFive E21     | openocd    | tockloader     | No            |
| [Nexys Video OpenTitan](opentitan/README.md)                         | RISC-V RV32IMC  | Ibex           | custom     | custom         | Tock fork     |
