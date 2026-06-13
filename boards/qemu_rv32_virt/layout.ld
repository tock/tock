/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/**
 * QEMU emulated DRAM region. Tock is currently designed to be placed
 * at the start of DRAM, using the `-bios` option in qemu-system-riscv32.
 *
 *   0x80000000..0x80100000  rom    — kernel text / rodata (1 MB)
 *   0x80100000..0x80200000  prog   — app flash (1 MB)
 *   0x80200000..0x80800000  (gap — unused DRAM)
 *   0x80800000..0x80C00000  ram    — hart 0 stack / data / bss / appmem (4 MB, NAPOT-aligned)
 *   0x80C00000..0x81000000          — hart 1 private data / bss / stack (4 MB, NAPOT-aligned)
 *
 * The two RAM regions are adjacent, so their union 0x80800000..0x81000000
 * forms a single 8 MB NAPOT-aligned region.  Hart 1's ePMP uses this 8 MB
 * region as its RAMRegion so it can access both harts' data (shared large
 * statics live at absolute hart-0 BSS addresses) and its own private stack.
 *
 * Hart 0 stack grows down from _estack (bottom of ram + stack_size).
 * Hart 1 stack grows down from _estack_h1 (= 0x81000000, top of hart 1 region).
 *
 * Hart 1's .data and .bss are independent copies of hart 0's, placed at the
 * same offsets within the hart 1 region.  The stride between the two regions
 * (_stride_h1 = _ssram_h1 - _ssram) is used to derive all hart-1 symbols
 * from the corresponding hart-0 symbols after the shared linker script runs.
 */

MEMORY
{
  rom  (rx)  : ORIGIN = 0x80000000, LENGTH = 0x100000
  prog (rx)  : ORIGIN = 0x80100000, LENGTH = 0x100000
  ram  (rwx) : ORIGIN = 0x80800000, LENGTH = 0x400000
}

SECTIONS {
    _sflash = ORIGIN(rom);
    _eflash = ORIGIN(prog) + LENGTH(prog);

    _ssram  = ORIGIN(ram);
    _esram  = ORIGIN(ram) + LENGTH(ram);

    /* Hart 1 gets its own private RAM region, adjacent to hart 0's. */
    _ssram_h1  = 0x80C00000;
    _esram_h1  = 0x81000000;
    _estack_h1 = _esram_h1;
}

INCLUDE tock_kernel_layout.ld

/* Pin _sappmem to a stable address so app TBFs can use fixed RAM addresses
 * without recompiling whenever the kernel binary changes in size.
 * Must be >= end of kernel BSS (currently ~0x8081b11c) and within ram. */
SECTIONS {
    _sappmem = 0x80880000;
}

/* Derive hart 1's per-hart .data / .bss symbols by adding the stride between
 * the two RAM bases to the corresponding hart-0 symbols.  These are used by
 * the _hart1_entry assembly to copy / zero hart 1's private data region and
 * to set the GP register to the correct midpoint of that region. */
SECTIONS {
    _stride_h1    = _ssram_h1 - _ssram;
    _gp_h1        = __global_pointer$ + _stride_h1;
    _srelocate_h1 = _srelocate + _stride_h1;
    _erelocate_h1 = _erelocate + _stride_h1;
    _szero_h1     = _szero     + _stride_h1;
    _ezero_h1     = _ezero     + _stride_h1;

    /* Hart 1 process replica memory: from end of hart 1 BSS to the base of
     * hart 1's kernel stack.  The kernel stack for hart 1 grows downward from
     * _estack_h1 (= _esram_h1), so we leave the same stack_size headroom that
     * the primary hart leaves at the bottom of its own ram region. */
    _sappmem_h1 = _ezero_h1;
    _eappmem_h1 = _esram_h1 - (_estack - _sstack);
}
