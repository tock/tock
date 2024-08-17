// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! RISC-V CSR Library
//!
//! Uses the Tock Register Interface to control RISC-V CSRs.
#![cfg_attr(target_feature = "xcheri", feature(asm_const))]
#![no_std]

pub mod csr;
