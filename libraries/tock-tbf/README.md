Tock Binary Format (TBF) Parsing Library
========================================

This crate contains code the kernel uses to parse TBF headers for processes on a
board. It is split into a library because other code besides the kernel (for
example elf2tab) may want to use this shared library code.

This code was originally at `kernel/src/tbfheader.rs`.
