# SHAKTI C-Class Chip

Chip crate for the open-source
[SHAKTI C-Class](https://gitlab.com/shaktiproject/cores/c-class) (RV64IMAC) test
SoC, targeting the Verilator simulation used by the `shakti_c_sim` board.

Provides a 64-bit-access CLINT machine-timer / alarm driver (the SoC's CLINT
mishandles 32-bit reads of the 64-bit `mtime`, so it is accessed as a single
64-bit register), a polled UART driver, and chip setup and interrupt handling.

Built on the shared `riscv` / `rv64i` architecture crates.
