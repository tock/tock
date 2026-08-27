# ARM MPS2 "Chip" Family

The MPS2 is an FPGA board from ARM designed for hardware/software co-design.

The `mps2-an*` family of boards all use the same peripheral hardware, they just
swap in different CPU cores. The naming scheme refers to the Application Note
that defines the full FPGA image, and pragmatically which CPU core is attached.

For more details on the platform, see the QEMU documentation on the MPS2 family:
https://www.qemu.org/docs/master/system/arm/mps2.html

As the only difference is the underlying core, and all that amounts to is the
vector table, this crate holds the shared peripherals and each image has its
own crate for the rest: `qemu_arm_mps2_an385` and `qemu_arm_mps2_an386`.

The upstream MPS2 family supports the following configurations (as of Aug 2026):
 - mps2-an385, a Cortex-M3
 - mps2-an386, a Cortex-M4
 - mps2-an500, a Cortex-M7 [not yet implemented here]
 - mps2-an505, a Cortex-M33 [not yet implemented here]
 - mps2-an511, the "DesignStart" variant of the M3 [not supported here; different hardware mappings]
 - mps2-an521, dual Cortex-M33 [not supported here]
