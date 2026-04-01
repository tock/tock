/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2024.                                  */

INCLUDE ./chip_layout.ld
INCLUDE tock_kernel_layout.ld

/* 
 * Define the application memory symbols so the Tock kernel
 * knows where to look for processes at boot.
 */
_sappmem = ORIGIN(prog);
_eappmem = ORIGIN(prog) + LENGTH(prog);

/* 
 * Override _estack after including the generic layout.
 */
_estack = ORIGIN(ram) + LENGTH(ram);
