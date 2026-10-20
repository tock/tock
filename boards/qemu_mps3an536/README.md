# MPS3-AN536 Board

This board crate targets the **MPS3** FPGA reference design, which
implements a configurable core.

The only available implementation is the MPS3-AN536 which contains an ARM Cortex R52 CPU.


The MPS3 can be run on:
- The MPS3 FPGA development board.
- The **Arm Fixed Virtual Platform (FVP)** for Cortex-R52
  (`FVP_BaseR_AEMv8R`), available as part of the Arm FVP tooling.

## Memory Map (provisional)
xxx NO xxx

| Region         | Origin       | Length |
|----------------|--------------|--------|
| ITCM (rom)     | 0x0000_0000  | 512 KB |
| DTCM (ram)     | 0x2000_0000  | 512 KB |
| SRAM (sram)    | 0x2100_0000  | 4 MB   |

Verify all addresses against the AN536 Technical Reference Manual before use.

## Building

```shell
make -C boards/mps3```

> **Note:** The `armv8r-none-eabihf` Rust target may need to be added via a
> custom target JSON file until it is available in the upstream Rust toolchain.
> Check `rustup target list` for availability.

<!---
## Running on the FVP

```shell
make -C boards/mps3an536 run
```

This requires the Arm FVP for Cortex-R52 (`FVP_BaseR_AEMv8R`) to be installed
and available on `PATH`.
-->

## TODO

- Implement GIC driver in `chips/mps3an536`.
- Implement SP804 timer driver.
- Complete `UserspaceKernelBoundary` and `MPU` for Cortex-R52 in `arch/cortex-r52`.
- Wire up console capsule.
- Verify memory map addresses against AN536 TRM.
