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

Two things live outside this tree and have to be pointed at:

- **`elf2hex`** — the Berkeley/fesvr-style tool, invoked as
  `elf2hex <bytes-per-row> <rows> <elf> [base]`, which is the form used
  throughout the C-Class documentation. SiFive's
  [elf2hex](https://github.com/sifive/elf2hex) takes `--bit-width`/`--input`
  instead and is **not** a drop-in substitute; set `ELF2HEX` to whichever
  binary provides the positional form.
- **the simulator** — `out`, `boot.MSB` and `boot.LSB`, built out of the
  c-class repository (see its `docs/source/simulating.rst`). Point
  `SHAKTI_BIN` at the directory holding them.

The SoC loads a single flat image, `code.mem`, into main memory at
`0x8000_0000`. Since the app region is inside that same memory, the kernel and
the app TBF are combined into that one image — `tools/splice_tbf.py` rewrites
the rows covering the app region.

```shell
# Build code.mem only
$ make APP=/path/to/app.tbf hex

# Build it and run the simulation
$ make APP=/path/to/app.tbf SHAKTI_BIN=/path/to/c-class/bin sim
```

Both targets write into `target/riscv64imac-unknown-none-elf/sim/`. The
simulator's own output goes to `sim.log` there, and the SoC UART output — the
kernel's `debug!()` lines — to `app_log`, which `make sim` prints. On
`*** STAGE 5 PASS ***` the board writes `1` to `0x0002_000C` and the simulation
self-exits.

Overridable variables: `ELF2HEX`, `ELF2HEX_WIDTH` (default 8),
`ELF2HEX_ROWS` (4194304), `RAM_BASE` (`0x80000000`), `APP_BASE`
(`0x80100000`), `SIM_TIMEOUT` (250s).

If a process faults or the kernel panics, the standard Tock panic handler prints
the panic banner, kernel version, RISC-V CPU state, and a per-process dump over
the same UART, then ends the simulation.

## Notes

- The SoC UART is polled (no interrupt line in the sim), so output is synchronous.
- No PLIC in this Test-SoC; the only interrupt source is the CLINT (machine
  timer / software), which drives the Alarm capsule.
- The chip crate reads `mtime` as a single 64-bit register, which is the natural
  access on RV64. (A 32-bit read of the upper half was also broken on this SoC
  when this board was written; that has since been fixed upstream in
  [`shaktiproject/uncore/devices`](https://gitlab.com/shaktiproject/uncore/devices).)
