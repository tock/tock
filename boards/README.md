Platforms Supported by Tock
===========================

The `/boards` directory contains the physical hardware platforms
that Tock supports.

Tock divides boards into three approximate 'tiers' of support.
These tiers are newly defined and are a bit informal as a result,
but the approximate definitions:

 - **Tier 1:** The most feature-complete and thoroughly tested boards. These
               are boards used most regularly by core team members or other
               highly engaged contributors. They are used as examples in the
               [Tock Book](https://book.tockos.org).
 - **Tier 2:** Platforms seeing reasonably regular use. These generally
               have broader, but still incomplete, peripheral support.
               They may also be 'relatives' of Tier 1 boards (e.g. a
               less-used varient in the nrf52 family) – likely in good
               shape, but not heavily tested. Some Tier 2 boards may
               have known issues, which are documented in release notes
               during release testing.
 - **Tier 3:** New or highly experimental. These should support the minimum
               platform requirements laid out in [the Porting
               documentation](https://book.tockos.org/development/porting), but
               make no promises beyond that.

 - **Other:** See each board for specific details.

---

> #### RISC-V?
>
> Tock has solid support for the RISC-V architecture, but no tier 1 or 2 support
> for any single RISC-V board. If you are interested in running Tock on RISC-V
> there are a few options:
>
> 1. If you would like a cheap RISC-V development board you can use the
>    [ESP32-C3-DevKitM-1](esp32-c3-devkitM-1/README.md). This board is
>    under active development to move to Tier 2 support.
> 1. For a fully virtual platform on QEMU you can use the
>    [QEMU RISC-V 32 bit `virt` platform](qemu_rv32_virt/README.md) board. This
>    can be quickly started and run on a host computer.
> 1. For a simulation environment you can use Verilator with
>    [OpenTitan Earlgrey on CW310](opentitan/earlgrey-cw310/README.md) or
>    [Verilated LiteX Simulation](litex/sim/README.md).
> 1. For an FPGA setup you can use
>    [OpenTitan Earlgrey on CW310](opentitan/earlgrey-cw310/README.md) or
>    [LiteX on Digilent Arty A-7](litex/arty/README.md).

---

### Tier 1

| Board                                                             | Architecture     | MCU            | Interface  | App deployment              | QEMU Support? |
|-------------------------------------------------------------------|------------------|----------------|------------|-----------------------------|---------------|
| [Hail](hail/README.md)                                            | ARM Cortex-M4    | SAM4LC8BA      | Bootloader | tockloader                  | No            |
| [Imix](imix/README.md)                                            | ARM Cortex-M4    | SAM4LC8CA      | Bootloader | tockloader                  | No            |
| [Nordic nRF52840-DK](nordic/nrf52840dk/README.md)                 | ARM Cortex-M4    | nRF52840       | jLink      | tockloader                  | No            |
| [Nano 33 BLE](nano33ble/README.md)                                | ARM Cortex-M4    | nRF52840       | Bootloader | tockloader                  | No            |
| [Nano 33 BLE Rev2](nano33ble_rev2/README.md)                      | ARM Cortex-M4    | nRF52840       | Bootloader | tockloader                  | No            |
| [BBC Micro:bit v2](microbit_v2/README.md)                         | ARM Cortex-M4    | nRF52833       | openocd    | tockloader                  | No            |
| [Clue nRF52840](clue_nrf52840/README.md)                          | ARM Cortex-M4    | nRF52840       | Bootloader | tockloader                  | No            |

### Tier 2

| Board                                                             | Architecture     | MCU            | Interface  | App deployment              | QEMU Support? |
|-------------------------------------------------------------------|------------------|----------------|------------|-----------------------------|---------------|
| [Nordic nRF52-DK](nordic/nrf52dk/README.md)                       | ARM Cortex-M4    | nRF52832       | jLink      | tockloader                  | No            |
| [Nordic nRF52840-Dongle](nordic/nrf52840_dongle/README.md)        | ARM Cortex-M4    | nRF52840       | jLink      | tockloader                  | No            |
| [Particle Boron](particle_boron/README.md)                        | ARM Cortex-M4    | nRF52840       | jLink      | tockloader                  | No            |
| [MakePython nRF52840dk](makepython-nrf52840/README.md)            | ARM Cortex-M4    | nRF52840       | Bootloader | tockloader                  | No            |
| [ST Nucleo F446RE](nucleo_f446re/README.md)                       | ARM Cortex-M4    | STM32F446      | openocd    | custom                      | https://github.com/tock/tock/issues/1827 |
| [ST Nucleo F429ZI](nucleo_f429zi/README.md)                       | ARM Cortex-M4    | STM32F429      | openocd    | custom                      | https://github.com/tock/tock/issues/1827 |
| [STM32F3Discovery kit](stm32f3discovery/README.md)                | ARM Cortex-M4    | STM32F303VCT6  | openocd    | custom                      | https://github.com/tock/tock/issues/1827 |
| [STM32F412G Discovery kit](stm32f412gdiscovery/README.md)         | ARM Cortex-M4    | STM32F412G     | openocd    | custom                      | https://github.com/tock/tock/issues/1827 |
| [STM32F429I Discovery kit](stm32f429idiscovery/README.md)         | ARM Cortex-M4    | STM32F429I     | openocd    | custom                      | https://github.com/tock/tock/issues/1827 |
| [Pico Explorer Base](pico_explorer_base/README.md)                | ARM Cortex-M0+   | RP2040         | openocd    | openocd                     | No            |
| [Nano RP2040 Connect](nano_rp2040_connect/README.md)              | ARM Cortex-M0+   | RP2040         | custom     | custom                      | No            |
| [Raspberry Pi Pico](raspberry_pi_pico/README.md)                  | ARM Cortex-M0+   | RP2040         | openocd    | openocd                     | No            |
| [SparkFun RedBoard Artemis Nano](apollo3/redboard_artemis_nano/README.md) | ARM Cortex-M4 | Apollo3   | custom     | custom                      | No            |
| [SparkFun LoRa Thing Plus - expLoRaBLE](apollo3/lora_things_plus/README.md) | ARM Cortex-M4 | Apollo3 | custom     | custom                      | No            |
| [SparkFun RedBoard Artemis ATP](apollo3/redboard_artemis_atp/README.md) | ARM Cortex-M4 | Apollo3     | custom     | custom                      | No            |
| [SMA Q3](sma_q3/README.md)                                        | ARM Cortex-M4    | nRF52840       | openocd    | tockloader                  | No            |
| [Wio WM1110 Development Board](wm1110dev/README.md)               | ARM Cortex-M4    | nRF52840       | Bootloader | tockloader                  | No            |

### Tier 3

| Board                                                             | Architecture     | MCU            | Interface  | App deployment              | QEMU Support? |
|-------------------------------------------------------------------|------------------|----------------|------------|-----------------------------|---------------|
| [WeAct F401CCU6 Core Board](weact_f401ccu6/README.md)             | ARM Cortex-M4    | STM32F401CCU6  | openocd    | custom                      | No            |
| [SparkFun RedBoard Red-V](redboard_redv/README.md)                | RISC-V           | FE310-G002     | openocd    | tockloader                  | Yes (5.1)     |
| [SiFive HiFive1 Rev B](hifive1/README.md)                         | RISC-V           | FE310-G002     | openocd    | tockloader                  | Yes (5.1)     |
| [BBC HiFive Inventor](hifive_inventor/README.md)                  | RISC-V           | FE310-G003     | tockloader | tockloader                  | No            |
| [ESP32-C3-DevKitM-1](esp32-c3-devkitM-1/README.md)                | RISC-V-ish RV32I | ESP32-C3       | custom     | custom                      | No            |
| [i.MX RT 1052 Evaluation Kit](imxrt1050-evkb/README.md)           | ARM Cortex-M7    | i.MX RT 1052   | custom     | custom                      | No            |
| [Teensy 4.0](teensy40/README.md)                                  | ARM Cortex-M7    | i.MX RT 1062   | custom     | custom                      | No            |
| [Digilent Arty A-7 100T](arty_e21/README.md)                      | RISC-V RV32IMAC  | SiFive E21     | openocd    | tockloader                  | No            |
| [MSP432 Evaluation kit MSP432P401R](msp_exp432p401r/README.md)    | ARM Cortex-M4    | MSP432P401R    | openocd    | custom                      | No            |
| [CY8CPROTO-062-4343W](cy8cproto_62_4343_w/README.md)              | ARM Cortex-M0+   | PSoC62         | openocd    | custom                      | No            |


### Other

An FPGA and Verilator implementation that is well supported and is regularly tested as part of CI.

| Board                                                             | Architecture     | MCU            | Interface  | App deployment              | QEMU Support? |
|-------------------------------------------------------------------|------------------|----------------|------------|-----------------------------|---------------|
| [OpenTitan Earlgrey on CW310](opentitan/earlgrey-cw310/README.md) | RISC-V RV32IMC   | EarlGrey       | custom     | custom                      | Yes (5.1)     |

Virtual hardware platforms that are regularly tested as part of the CI.

| Board                                                             | Architecture     | MCU            | Interface  | App deployment              | QEMU Support? |
|-------------------------------------------------------------------|------------------|----------------|------------|-----------------------------|---------------|
| [QEMU RISC-V 32 bit `virt` platform](qemu_rv32_virt/README.md)    | RISC-V RV32IMAC  | QEMU           | custom     | custom                      | Yes (7.2.0)   |
| [LiteX on Digilent Arty A-7](litex/arty/README.md)                | RISC-V RV32IMC   | LiteX+VexRiscV | custom     | tockloader (flash-file)[^1] | No            |
| [Verilated LiteX Simulation](litex/sim/README.md)                 | RISC-V RV32IMC   | LiteX+VexRiscv | custom     | tockloader (flash-file)[^1] | No            |
| [VeeR EL2 simulation](veer_el2_sim/README.md)                     | RISC-V RV32IMC   | VeeR EL2       | custom     | custom                      | No            |
| [QEMU i486 Q53](qemu_i486_q35/README.md)                          | i468             | Q35            | custom     | custom                      | Yes           |

[^1]: Tockloader is not able to interact with this board directly, but
      can be used to work on a flash-image of the board, which can in
      turn be flashed onto / read from the board. For more specific
      information, visit the board's README.

# Out of Tree Boards

Some projects that use Tock maintain their own board definitions outside the
Tock repository.

| Project                                                  | Boards                                     | Architecture   | MCU      | Build System  |
|----------------------------------------------------------|--------------------------------------------|----------------|----------|---------------|
| [OpenSK](https://github.com/google/opensk)               | nRF52840-DK, nRF52840-Dongle, nRF52840-MDK | ARM Cortex-M4  | nRF52840 | Python script |
| [OpenTitan](https://github.com/lowrisc/opentitan)        | OpenTitan                                  | RISC-V RV32IMC | EarlGrey | Meson         |
| [Tock-on-Titan](https://github.com/google/tock-on-titan) | golf2, papa                                | ARM Cortex-M3  | H1       | Makefiles     |
