/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

INCLUDE ./chip_layout.ld
INCLUDE tock_kernel_layout.ld

ASSERT(DEFINED(mps2_an386_vector_table), "AN386 vector table missing: does an386 still name its core through qemu_arm_mps2_an386?")
