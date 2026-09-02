ARM MPS2 "Chip" Family
======================

The MPS2 is an FPGA board from ARM designed for hardware/software co-design.
Its `mps2-an*` configurations share a peripheral suite and differ mainly in the
CPU core they attach; the name refers to the ARM Application Note defining the
FPGA image. For more on the platform, see the QEMU documentation:
https://www.qemu.org/docs/master/system/arm/mps2.html

This crate holds the peripherals shared across those images. What differs per
image -- the core and its vector table -- lives in `qemu_arm_mps2_an385` and
`qemu_arm_mps2_an386`.

QEMU's `hw/arm/mps2.c` implements four of these configurations:

 - mps2-an385, a Cortex-M3
 - mps2-an386, a Cortex-M4
 - mps2-an500, a Cortex-M7 [not implemented here; PSRAM is at a different
   base and there is no block RAM]
 - mps2-an511, the "DesignStart" variant of the M3 [not implemented here;
   different hardware mappings]

The TrustZone-enabled MPS2 images (`mps2-an505`, `mps2-an521`) are a separate
QEMU machine family in `hw/arm/mps2-tz.c`, built on the IoTKit/SSE-200 rather
than the peripheral layout above, and are out of scope for this crate.
