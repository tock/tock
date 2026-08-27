/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

INCLUDE ./chip_layout.ld
INCLUDE tock_kernel_layout.ld

/* The vector table only reaches the image as a side effect of `main.rs`
 * naming its core through `qemu_arm_mps2_an385`; check that still holds. */
ASSERT(DEFINED(mps2_vector_table), "AN385 vector table missing: does an385 still name its core through qemu_arm_mps2_an385?")
