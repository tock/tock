# SHAKTI C-Class Simulation

This board runs Tock on the open-source
[SHAKTI C-Class](https://gitlab.com/shaktiproject/cores/c-class) (RV64IMAC) core
under a Verilator simulation. It boots a single process and exercises the full
RV64 userspace round-trip (context switch, `ecall`, upcalls) driving the Alarm
capsule for a time-based syscall.

Test-SoC memory map: RAM at `0x8000_0000`, CLINT at `0x0200_0000`, UART at
`0x0001_1300`, and a simulation-control register at `0x0002_000C` (writing `1`
ends the sim). The CLINT `mtime` is accessed as a single 64-bit register (see
`chips/shakti_c`).

Kernel boot progress and the process/panic dumps are emitted with the standard
Tock `debug!()` mechanism over the SoC UART; the board self-terminates the
simulation once the test process completes.

## Building

Build the kernel from this directory:

```
cd boards/shakti_c_sim
make
```

This produces the kernel ELF and a raw binary:

- ELF: `target/riscv64imac-unknown-none-elf/release/shakti_c_sim`
- BIN: `target/riscv64imac-unknown-none-elf/release/shakti_c_sim.bin`

The board loads exactly one process. The app image is a hand-written RV64
assembly TBF (there is no libtock-rs RV64 target yet) spliced into the flash
region at `_sapps` (`0x8010_0000`). Concatenate the kernel and the app TBF into
the image the simulator loads:

```
make
cat target/riscv64imac-unknown-none-elf/release/shakti_c_sim.bin app.tbf > shakti_c_sim_with_app.bin
```

## Running

1. Build the SHAKTI C-Class Verilator model (from the
   [c-class](https://gitlab.com/shaktiproject/cores/c-class) project) for the
   Test-SoC configuration used here (RAM `0x8000_0000`, UART `0x0001_1300`,
   CLINT `0x0200_0000`).

2. Load the combined kernel+app binary at the boot address `0x8000_0000` and
   start the simulation, pointing the harness's boot memory image at
   `shakti_c_sim_with_app.bin`.

3. Watch the UART output. The testbench captures each byte written to the UART TX
   register and writes it to `app_log`/stdout. Expected output is approximately:

   ```
   === Tock OS on SHAKTI C-Class (RV64IMAC) ===
   [shakti_c_sim] real TBF process, time-based syscall via Alarm
   calling load_processes from _sapps=0x0000000080100000
   load_processes Ok
   processes loaded = 1
   alarm wired (driver 0); mtimer IRQ enabled; entering kernel_loop
   t_before  mtime=0x00000000........
   process resumed after alarm-fired upcall
   t_after   mtime=0x00000000........
   elapsed ticks (10 MHz) = 0x00000000........
   *** STAGE 5 PASS ***
   ```

4. On `*** STAGE 5 PASS ***` the board writes `1` to the sim-control register at
   `0x0002_000C`, which ends the Verilator run and flushes `app_log`. The board
   self-exits; you do not need to kill the simulator manually.

If a process faults or the kernel panics, the standard Tock panic handler prints
the panic banner, kernel version, RISC-V CPU register state, and a per-process
dump over the same UART, then ends the simulation the same way.

## Notes

- The SoC UART is polled (no interrupt line in the sim), so kernel output is
  synchronous.
- There is no PLIC in this Test-SoC; the only interrupt source is the CLINT
  (machine timer / machine software), which drives the Alarm capsule.
