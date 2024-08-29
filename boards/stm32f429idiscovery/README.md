STM32F429I Discovery development board with STM32F429ZI MCU
======================================================

Note: This board layout is based on the nucleo_f429zi board layout.

For more details [visit the STM32F429I Discovery website](https://www.st.com/en/evaluation-tools/32f429idiscovery.html).

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/std32f429idiscovery`
directory and run:

```bash
$ make flash

(or)

$ make flash-debug
```
