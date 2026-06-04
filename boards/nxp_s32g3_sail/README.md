NXP S32G3 SAIL Tock board
=========================

S32G3 is a SoC by NXP targetting automotive and industrial applications. It has a quad-core Cortex-M7 subsystem, and eight-core A53 subsystem.
The SAIL (Safety Island) is a separate subsystem that is designed to run safety-critical code, tock being a perfect candidate for such applications.

Hardware can be acquired through NXP sales.

This board target is the minimal setup for running Tock on the M7_0 core of an S32G3 Safety Island.
It is intended to be used as a starting point for building a Tock-based application on the S32G3 SoC.

It is intentionally limited to the following feature:

- SRAM-linked kernel image at `0x34200000`
- Large 2MiB SRAM for stack, BSS, and data at `0x34000000`
- The smaller zero wait state 56KiB DTCM is not used to simplify app deployment.

- Cortex-M7 chip initialization and clock setup
- uart driver: linflexd
- timer driver: stm (System Timer Module)

Other more complex drivers are included to the best of our ability, but are not fully tested (see chips/nxp_s32g3/README.md for details).

Build with:

```bash
make -C boards/nxp_s32g3_sail
```

You can append to the binary the TBF file of your application, and the resulting binary can be loaded into the S32G3 SRAM.

```
cat "${KERNEL_BIN}" "${APP_TBF}" > "${COMBINED_BIN}"
```

The resulting binary can be loaded into SRAM via a debugger or JTAG probe, or
flashed into NOR and booted by the SoC boot ROM.  See the NXP S32G3 Reference
Manual for boot options.
