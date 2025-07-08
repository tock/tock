// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Support for the 32-bit RISC-V architecture.

#![no_std]

pub mod clic;
pub mod machine_timer;

// Re-export shared libraries so that dependent crates do not have to have
// both rv32i and riscv as dependencies.
pub use riscv::csr;
pub use riscv::pmp;
pub use riscv::print_riscv_state;
pub use riscv::support;
pub use riscv::syscall;
pub use riscv::PermissionMode;
pub use riscv::_start;
pub use riscv::_start_trap;
pub use riscv::configure_trap_handler;
pub use riscv::print_mcause;
pub use riscv::semihost_command;
