# Cortex-R52 Architecture Support

This crate provides architecture support for ARM Cortex-R52 processors.

It only works with the QEMU virtual implementation of the MPS3 board with 
XCKU115 FPGA loaded with AN536 code provided by ARM as reference design.

The Cortex-R52 is based on the ARMv8-R architecture, AARCH32 variant only.
There is currently (2026) no other ARMv8-R aarch32 CPU (except the nearly
identical R52+), so this arch implementation has been put in only one
crate. The ARMv8-R is however close to the ARMv8-A.

This implemetnation only uses:
- EL1/EL0 secure level
- USR/SVC/FIQ PE modes
- A32 instruction set

It would be possible to also use T32 (thumb), but QEMU does not allow T32 boot and
it would probabl require much more work to define instruction set for user apps.

Base peripherals used here are:
- Generic Interrupt Controller v3 (GIC).
- PMSAv8 MPU.
- One Generic Timer EL1/Physical/Non secure.

This crate requires to be configured for some specific chip by using a mandatory 
feature (check lib.rs) which must provide clocks and HW modules base addresses
as constants.


TODO:
 - Watchdog
 - FP context saving
...

...

...

Still some work to do !

