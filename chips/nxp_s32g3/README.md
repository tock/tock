NXP S32G3 Chip Crate
====================

This crate provides peripheral drivers for the NXP S32G3 Cortex-M7 core,
targeting the M7_0 subsystem of the S32G3 SoC.

Supporting an APP that writes to the console and measure the time taken to do so, this crate has been tested on S32G3 board.

Peripherals implemented, with testing level:

- LINFlexD UART
  - Fully tested and functional. Missing DMA support.
- STM (System Timer Module)
  - Fully tested and functional.
- SIUL2 (System Integration Unit Lite2) for pinmux
  - Tested as an enabler for other peripherals, no full testing of all features.
- MC_ME Reset Management
 - Tested as an enabler for other peripherals, no full testing of all features.
- MS_CM (Miscellaneous System Control Module)
 - Tested as an enabler for other peripherals, no full testing of all features.
- XRDC (External Resource Domain Controller)
 - Tested as an enabler for other peripherals, no full testing of all features.
- ssram (Standby SRAM)
 - Fully tested and functional.
- swt (Software Watchdog Timer)
 - Only disabled, not supporting actual watchdog functionality.
