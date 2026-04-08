/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Infineon Technologies AG 2026.                           */

/**
 * This layout currently still triggers CYBOOT_APP_LAUNCH_ADDR_INVALID in openocd/gdb. This error
 * should be cause because msp, pc aren't on valid addresses but could also been caused by this
 * layout as the error does not occur when using ModusToolbox's HelloWorld
 * Maybe the file size of tock is too big.
**/

MEMORY
{
  	rom (rx)  : ORIGIN = 0x32000000, LENGTH = 0x20000
	prog (rx) : ORIGIN = 0x32020000, LENGTH = 0x20000 /* dynamically/after-compilation inserted apps */
  	ram (rwx) : ORIGIN = 0x34000000, LENGTH = 0x10000
}

PAGE_SIZE = 4K;

INCLUDE tock_kernel_layout.ld
