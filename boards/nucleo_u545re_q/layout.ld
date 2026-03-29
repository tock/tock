/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2024.                                  */

INCLUDE ./chip_layout.ld
INCLUDE tock_kernel_layout.ld

/* 
 * Override _estack after including the generic layout.
 * Tock's generic layout usually sets _estack to the START of RAM.
 * We force it to the END of RAM (ORIGIN + LENGTH) to prevent HardFault.
 */
_estack = ORIGIN(ram) + LENGTH(ram);
