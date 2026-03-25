/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Oxidos Automotive 2025 SRL.                              */

MEMORY
{
  	rom (rx)  : ORIGIN = 0x32000000, LENGTH = 0x40000
	prog (rx) : ORIGIN = 0x32020000, LENGTH = 0x20000 /* dynamically/after-compilation inserted apps */
  	ram (rwx) : ORIGIN = 0x34000000, LENGTH = 0x10000
}

PAGE_SIZE = 4K;

INCLUDE tock_kernel_layout.ld
