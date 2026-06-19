/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/**
 * QEMU emulated DRAM region. Tock is currently designed to be placed
 * at the start of DRAM, using the `-bios` option in qemu-system-riscv32.
 *
 *   0x80000000..0x80100000  rom      — kernel text / rodata (1 MB)
 *   0x80100000..0x80200000  prog     — hart 0 app flash (1 MB)
 *   0x80200000..0x80300000  prog_h1  — hart 1 app flash (1 MB)
 *   0x80300000..0x80800000  (gap — unused DRAM)
 *   0x80800000..0x80C00000  ram      — hart 0 stack / data / bss / appmem (4 MB, NAPOT-aligned)
 *   0x80C00000..0x81000000           — hart 1 private data / bss / stack / appmem (4 MB, NAPOT-aligned)
 *
 * Hart 1 does not share hart 0's process replicas: since libtock-c has no
 * PIC support for RISC-V (only Cortex-M), an app's compiled memory image
 * embeds absolute addresses for whatever RAM_START it was linked against,
 * so it can't simply be byte-copied to a different base. Instead each app is
 * built twice, once per hart's fixed flash/RAM addresses, and hart 1 loads
 * its own copy via the normal load_processes() path, just like hart 0.
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
  rom     (rx)  : ORIGIN = 0x80000000, LENGTH = 0x100000
  prog    (rx)  : ORIGIN = 0x80100000, LENGTH = 0x100000
  prog_h1 (rx)  : ORIGIN = 0x80200000, LENGTH = 0x100000
  ram     (rwx) : ORIGIN = 0x80800000, LENGTH = 0x400000
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

/* Hart 1's own app flash region -- see prog_h1 in MEMORY above. Apps built
 * for hart 1 are linked against this fixed flash base (+0x80 for the TBF
 * header offset, by convention, same as hart 0's prog region). */
SECTIONS {
    _sapps_h1 = ORIGIN(prog_h1);
    _eapps_h1 = ORIGIN(prog_h1) + LENGTH(prog_h1);
}

/* NAPOT-valid upper bound for hart 1's kernel-protection FlashRegion.
 * Hart 1's load_processes() reads from prog_h1, so its FlashRegion must
 * extend past hart 0's _eflash (0x80200000) to cover it. Under the
 * KernelProtectionMMLEPMP's MML mode even M-mode (kernel) accesses are
 * restricted to configured regions, so this isn't optional. NAPOT regions
 * must be power-of-two sized starting at _sflash (0x80000000); rom + prog +
 * prog_h1 spans 3 MB, which isn't a power of two, so round up to 4 MB. */
SECTIONS {
    _eflash_h1 = ORIGIN(rom) + 0x400000;
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

    /* Hart 1 app memory: pinned to a fixed address (same +0x80000 offset
     * hart 0 uses for _sappmem above), not derived from _ezero_h1. Apps
     * built for hart 1 are linked against this fixed RAM address, so it
     * must be a build-time constant, not something that floats with kernel
     * BSS size. Must be >= end of hart 1's kernel BSS and leave room before
     * hart 1's kernel stack (which grows downward from _estack_h1).
     * _eappmem_h1 leaves the same stack_size headroom hart 0 leaves at the
     * bottom of its own ram region. */
    _sappmem_h1 = 0x80C80000;
    _eappmem_h1 = _esram_h1 - (_estack - _sstack);
}
