# Cortex-R52 Architecture Support

TODO: NEED MORE DETAILS!

This crate provides shared architecture support for ARM Cortex-R52 processors.

It only works with the QEMU virtual implementation of the MPS3 board with 
XCKU115 FPGA loaded with AN536 code provided by ARM as reference design.

The Cortex-R52 is based on the ARMv8-R architecture, AARCH32 variant only. 

This implemetnation only uses:
- EL1/EL0 secure level
- USR/SVC/FIQ PE modes
- A32 instruction set

Base peripherals used here are:
- Generic Interrupt Controller v3 (GIC).
- PMSAv8 MPU and is NOT done yet.
- One Generic Timer EL1/Physical/Non secure.

Still some work to do !

