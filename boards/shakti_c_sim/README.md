# SHAKTI C-Class Simulation

This board runs Tock on the open-source
[SHAKTI C-Class](https://gitlab.com/shaktiproject/cores/c-class) (RV64IMAC) core
under a Verilator simulation. It boots one process and exercises the full RV64
userspace round-trip (context switch, `ecall`, upcalls) driving the Alarm
capsule for a time-based syscall. Boot progress and panic/process dumps use the
standard `debug!()` mechanism over the SoC UART; the board ends the simulation
once the test process completes.

Test-SoC memory map: RAM `0x8000_0000`, CLINT `0x0200_0000`, UART `0x0001_1300`,
sim-control register `0x0002_000C` (write `1` to end the sim).

## Building

Run `make` in this directory to build the kernel:

- ELF: `target/riscv64imac-unknown-none-elf/release/shakti_c_sim`
- BIN: `target/riscv64imac-unknown-none-elf/release/shakti_c_sim.bin`

The board loads a single process from the app flash region at `_sapps`
(`0x8010_0000`). The test app is a hand-written RV64 assembly TBF (there is no
libtock-rs RV64 target yet).

## Running

1. Build the kernel with `make` (above).
2. Produce the memory image your SHAKTI C-Class Verilator build loads: convert
   the kernel ELF to hex (e.g. `<elf2hex ...>`) and place the app TBF at the app
   region base `0x8010_0000`.
3. Start the Verilator simulation with that image (`<your sim invocation>`).
   Kernel output appears on the SoC UART; on `*** STAGE 5 PASS ***` the board
   writes `1` to `0x0002_000C` and the simulation self-exits.

If a process faults or the kernel panics, the standard Tock panic handler prints
the panic banner, kernel version, RISC-V CPU state, and a per-process dump over
the same UART, then ends the simulation.

## Notes

- The SoC UART is polled (no interrupt line in the sim), so output is synchronous.
- No PLIC in this Test-SoC; the only interrupt source is the CLINT (machine
  timer / software), which drives the Alarm capsule.
