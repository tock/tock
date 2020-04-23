Platforms Supported by Tock
===========================

The `/boards` directory contains the physical hardware platforms
that Tock supports.

| Board                                                         | Architecture    | MCU            | Interface  | App deployment | Where to Get It                          |
|---------------------------------------------------------------|-----------------|----------------|------------|----------------|------------------------------------------|
| [Hail](hail/README.md)                                        | ARM Cortex-M4   | SAM4LC8BA      | Bootloader | tockloader     | [Lab11llc][tockhw]                       |
| [Imix](imix/README.md)                                        | ARM Cortex-M4   | SAM4LC8CA      | Bootloader | tockloader     | [Lab11llc][tockhw]                       |
| [Nordic nRF52-DK](nordic/nrf52dk/README.md)                   | ARM Cortex-M4   | nRF52832       | jLink      | tockloader     | [Nordic distributors][nrf52dk-hw]        |
| [Nordic nRF52840-DK](nordic/nrf52840dk/README.md)             | ARM Cortex-M4   | nRF52840       | jLink      | tockloader     | [Nordic distributors][nrf52840dk-hw]     |
| [Nordic nRF52840-Dongle](nordic/nrf52840_dongle/README.md)    | ARM Cortex-M4   | nRF52840       | jLink      | tockloader     | [Nordic distributors][nrf52840dongle-hw] |
| [ACD52832](acd52832/README.md)                                | ARM Cortex-M4   | nRF52832       | jLink      | tockloader     | [Aconno][aconno]                         |
| [TI LAUNCHXL-CC26x2](launchxl/README.md)                      | ARM Cortex-M4   | CC2652R        | openocd    | tockloader     | [TI or distributors][launchxl-hw]        |
| [ST Nucleo F446RE](nucleo_f446re/README.md)                   | ARM Cortex-M4   | STM32F446      | openocd    | custom         | [ST or distributors][f446re-hw]          |
| [ST Nucleo F429ZI](nucleo_f429zi/README.md)                   | ARM Cortex-M4   | STM32F429      | openocd    | custom         | [ST distributors][f429zi-hw]             |
| [STM32F3Discovery kit](stm32f3discovery/README.md)            | ARM Cortex-M4   | STM32F303VCT6  | openocd    | custom         | [ST distributors][discovery-hw]          |
| [SiFive HiFive1](hifive1/README.md)                           | RISC-V          | FE310-G000     | openocd    | tockloader     | [Crowdsupply][hifive1-revB-hw]*          |
| [Digilent Arty A-7 100T](arty-e21/README.md)                  | RISC-V RV32IMAC | SiFive E21     | openocd    | tockloader     | [Digilent or distributors][arty-hw]      |
| [Nexys Video OpenTitan](opentitan/README.md)                  | RISC-V RV32IMC  | Ibex           | custom     | custom         | _See board README_                       |

*Note: The RevA board is no longer avaiable, this now links to the RevB board.

[tockhw]: https://www.tockos.org/hardware/
[nrf52dk-hw]: https://www.nordicsemi.com/About-us/BuyOnline?search_token=nRF52-DK&series_token=nRF52832
[nrf52840dk-hw]: https://www.nordicsemi.com/About-us/BuyOnline?search_token=nrf52840-DK&series_token=nRF52840
[nrf52840dongle-hw]: https://www.nordicsemi.com/About-us/BuyOnline?search_token=nRF52840DONGLE&series_token=nRF52840
[aconno]: https://aconno.de/products/acd52832/
[launchxl-hw]: http://www.ti.com/tool/LAUNCHXL-CC26X2R1#buy
[f446re-hw]: https://www.st.com/en/evaluation-tools/nucleo-f446re.html#sample-and-buy
[f429zi-hw]: https://www.st.com/en/evaluation-tools/nucleo-f429zi.html#sample-and-buy
[discovery-hw]: https://www.st.com/en/evaluation-tools/stm32f3discovery.html#sample-and-buy
[hifive1-revB-hw]: https://www.crowdsupply.com/sifive/hifive1-rev-b
[arty-hw]: https://store.digilentinc.com/arty-a7-artix-7-fpga-development-board-for-makers-and-hobbyists/
