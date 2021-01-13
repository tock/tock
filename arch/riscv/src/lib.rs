//! Shared support for RISC-V architectures.

#![crate_name = "riscv"]
#![crate_type = "rlib"]
#![no_std]

pub mod csr;

#[cfg(target_arch = "riscv32")]
pub const XLEN: usize = 32;
#[cfg(target_arch = "riscv64")]
pub const XLEN: usize = 64;

// Default to 32 bit if no architecture is specified of if this is being
// compiled for testing on a different architecture.
#[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64", target_os = "none")))]
pub const XLEN: usize = 32;
