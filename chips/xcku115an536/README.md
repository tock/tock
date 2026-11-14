# MPS3-AN536 Chip Support

This crate provides chip-level support for the QEMU virtual implementation 
of the MPS3 board with XCKU115 FPGA loaded with AN536 code, which implements
 he Cortex-R52 processor.

## Peripherals

The AN536 subsystem implemented here provide the following peripherals:
 - Many UART based on APB UART
 - Dual Timer, probably a SP804 but the doc is unclear (poorly implemented)


Check "Example Cortex-R52x2 Subsystem for MPS3 Application Note AN536" 
for more details.

Check also "Arm Cortex-M System Design Kit Technical Reference Manual"
rev r1p1 which contains details for peripherals.
