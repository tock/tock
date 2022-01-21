//! RISC-V CSR Library
//!
//! Uses the Tock Register Interface to control RISC-V CSRs.

#![feature(asm, asm_const)]
#![feature(const_fn_trait_bound)]
#![no_std]

pub mod csr;
