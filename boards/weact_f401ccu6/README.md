WeAct STM32F401CCU6 Core Development Board
======================================================

For more details [visit WeActTC/MiniF4-STM32F4x1
Github repo](https://github.com/WeActTC/MiniF4-STM32F4x1).

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/weact_f401ccu6`
directory and run:

```bash
$ make flash

(or)

$ make flash-debug
```

Expects ST-LINK V2-1, if using V2 you can change lines 4-5 in
`openocd.cfg` to:
```bash
hla_device_desc "ST-LINK/V2"
hla_vid_pid 0x0483 0x3748
```

> **Note:** Unlike other Tock platforms, the default kernel image for this
> board will clear flashed apps when the kernel is loaded. This is to support
> the non-tockloader based app flash procedure below.

## Flashing app

**TODO:** Update this section once support for this board in
Tockloader is added.
