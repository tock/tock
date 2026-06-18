NXP S32G3 SAIL Tock board
=========================

This board target is the minimal build-time scaffold for running Tock on the
M7_0 core of an S32G3 SAIL deployment.

It is intentionally limited to the build milestone:

- SRAM-linked kernel image at `0x34000000`
- Cortex-M7 chip scaffolding
- uart driver: linflexd
- timer driver: stm (System Timer Module)
- SIUL2 (System Integration Unit Lite2) driver for pinmux (gpio not implemented yet)
- mc_me driver for clock configuration
- workspace integration for `cargo build`


Build with:

```bash
cd boards/nxp_s32g3_sail && cargo build --release --target thumbv7em-none-eabihf
objcopy -O binary target/thumbv7em-none-eabihf/release/tock target/thumbv7em-none-eabihf/release/tock.bin
```

You will need a bootloader or jtag controller to load the resulting `tock.bin` onto the SAIL platform, and jump in thumb mode to the entry point at `0x34000000`. 
The bootloader is outside the scope of this repository.
Future release will include a target that can be flashed directly into NOR and booted by the HSE.
