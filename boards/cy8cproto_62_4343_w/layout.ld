/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Oxidos Automotive 2025 SRL.                              */

MEMORY
{
  	rom (rx)  : ORIGIN = 0x10000000, LENGTH = 1M
	prog (rx) : ORIGIN = 0x10100000, LENGTH = 1M
  	ram (rwx) : ORIGIN = 0x08000000, LENGTH = 1M
}

ENTRY(initialize_ram_jump_to_main)

INCLUDE tock_kernel_layout.ld
