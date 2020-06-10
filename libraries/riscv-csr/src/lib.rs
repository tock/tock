//! RISC-V CSR Library
//!
//! Uses the Tock Register Interface to control RISC-V CSRs.

#![feature(asm)]
#![feature(const_fn)]
#![no_std]

pub mod csr;
