# SHAKTI C-Class Simulation

This board runs Tock on the open-source
[SHAKTI C-Class](https://gitlab.com/shaktiproject/cores/c-class) (RV64IMAC) core
under a Verilator simulation. It boots a single process and exercises the full
RV64 userspace round-trip (context switch, `ecall`, upcalls) driving the Alarm
capsule for a time-based syscall.

Test-SoC memory map: RAM at `0x8000_0000`, CLINT at `0x0200_0000`, UART at
`0x0001_1300`. The CLINT `mtime` is accessed as a single 64-bit register (see
`chips/shakti_c`).

## Building

Run `make` in this directory to produce
`target/riscv64imac-unknown-none-elf/release/shakti_c_sim.bin`, which is loaded
into the SHAKTI C-Class Verilator simulation. Boot progress is printed over the
SoC UART, and the board ends the simulation once the test process completes.
