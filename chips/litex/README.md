LiteX SoC Peripherals
=====================

[LiteX is a Migen based Core / SoC
builder](https://github.com/enjoy-digital/litex/). It allows
developers to combine peripherals and CPUs into a custom SoC
easily. One of the supported CPUs is the VexRiscv processor described
in SpinalHDL, which is implemented in
[`litex_vexriscv`](../litex_vexriscv).

This crate is a collection of helpers and drivers for LiteX cores used
in generated SoCs. The following cores are supported:

- [Uart](src/uart.rs)
- [Timer](src/timer.rs)
- [Led (LedChaser)](src/led_controller.rs)
- [LiteEth](src/liteeth.rs)
  ([Repository](https://github.com/enjoy-digital/liteeth))
