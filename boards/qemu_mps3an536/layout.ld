/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

/**
 * Memory layout for the ARM MPS3-AN536 (Cortex-R52).
 * based on ARM Application Note AN536
 * and qemu/hw/arm/mps3r.c
 *
 * Some region are defined in Cortex R52 spec
 */

/*
 * QEMU on 32 bits arch only supports 1G of DDR, but 3G on real HW and 64bits QEMU
 * MPS3_DDR_SIZE = 0xC0000000;
 */
MPS3_DDR_SIZE = 0x40000000;

MEMORY
{
  ATCM   (rwx)  : ORIGIN = 0x00000000, LENGTH = 0x00008000  /* boot and irq vector */
  QSPI_K (rx)   : ORIGIN = 0x08000000, LENGTH = 0x00200000  /* kernel text */
  QSPI_A (rx)   : ORIGIN = 0x08200000, LENGTH = 0x00600000  /* apps text */
  BRAM   (rwx)  : ORIGIN = 0x10000000, LENGTH = 0x00080000  /* ram for kernel and apps */
  DDR    (rwx)  : ORIGIN = 0x20000000, LENGTH = MPS3_DDR_SIZE
  ATCM0  (rwx)  : ORIGIN = 0xee000000, LENGTH = 0x00008000  /* aliased to 0 for core 0 */
  BTCM0  (rwx)  : ORIGIN = 0xee100000, LENGTH = 0x00008000
  CTCM0  (rwx)  : ORIGIN = 0xee200000, LENGTH = 0x00008000
  ATCM1  (rwx)  : ORIGIN = 0xee400000, LENGTH = 0x00008000  /* aliased to 0 for core 1 */
  BTCM1  (rwx)  : ORIGIN = 0xee500000, LENGTH = 0x00008000
  CTCM1  (rwx)  : ORIGIN = 0xee600000, LENGTH = 0x00008000
}

/* Additional section specific to this board */
SECTIONS
{
.text :
    {
        /* 
         * Vectors table with boot / irq / svc handlers
         * required for vector, and faster on real HW
         */
        KEEP(*(.vectors_atcm))
        KEEP(*(.atcm))
    } > ATCM
}

/* For generic tock layout */
REGION_ALIAS("rom",  QSPI_K);
REGION_ALIAS("prog", QSPI_A);
REGION_ALIAS("ram",  BRAM);

SECTIONS {
    /* Might be usefull for MPU setup. */
    _srom  = ORIGIN(rom);
    _erom  = ORIGIN(prog) + LENGTH(prog);

    _sram  = ORIGIN(ram);
    _eram  = ORIGIN(ram) + LENGTH(ram);
}

INCLUDE tock_kernel_layout.ld
