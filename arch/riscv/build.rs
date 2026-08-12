// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Defines two cfg aliases for gating RISC-V specific code.
//!
//! 1. `riscv_bare_metal`: True when building for bare metal RISC-V platforms,
//!    32 or 64 bit.
//! 2. `riscv`: True when building for any RISC-V platform, 32 or 64 bit.
//!
//! `riscv_bare_metal` can be used to gate code that is only valid on bare metal
//! systems where there is no other OS present. `riscv` can be used to gate code
//! that is valid on any RISC-V platform, with or without an OS.
//!
//! Commonly, code that is Tock-specific and only makes sense on bare-metal
//! platforms will be gated by `riscv_bare_metal`. For example:
//!
//! ```
//! #[cfg(riscv_bare_metal)]
//! pub extern "C" fn _start_trap() -> ! {
//!     // trap handler code...
//! }
//! ```
//!
//! This ensures that this code is only compiled and tested when the target is
//! bare metal RISC-V. However, cargo often builds Tock code targeted for the
//! host machine (which is likely not RISC-V), for example when running
//! tests. Therefore, we also need to include a mock implementation so the
//! symbol still exists:
//!
//! ```
//! #[cfg(not(riscv_bare_metal))]
//! pub extern "C" fn _start_trap() -> ! {
//!     unimplemented!()
//! }
//! ```
//!
//! By convention we just negate the check for the actual code.
//!
//! There may also be code that RISC-V specific, but is valid on bare metal or
//! if targeting a platform with an OS. That can be gated with `riscv`. For
//! example, a simple NOP instruction:
//!
//! ```
//! #[cfg(riscv)]
//! pub fn nop() {
//!     unsafe { asm!("nop"); }
//! }
//! ```
//!
//! Finally, there is some code that is rv32i- or rv64i-specific.
//!
//! That can be gated on the specific `target_arch` like this:
//!
//! ```
//! #[cfg(target_arch = "riscv32")]
//! pub const XLEN: usize = 32;
//! #[cfg(target_arch = "riscv64")]
//! pub const XLEN: usize = 64;
//! ```
//!
//! However, to make non-cross-compiled compilations still work (like `cargo
//! test`), we still need the mock implementation:
//!
//! ```
//! #[cfg(not(riscv))]
//! pub const XLEN: usize = 32;
//! ```
//!

fn main() {
    // Get build variables to determine how we are building the crate.
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    println!("cargo::rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo::rerun-if-env-changed=CARGO_CFG_TARGET_OS");

    // `riscv_bare_metal`
    println!("cargo::rustc-check-cfg=cfg(riscv_bare_metal)");
    if (arch == "riscv32" || arch == "riscv64") && os == "none" {
        println!("cargo::rustc-cfg=riscv_bare_metal");
    }

    // `riscv`
    println!("cargo::rustc-check-cfg=cfg(riscv)");
    if arch == "riscv32" || arch == "riscv64" {
        println!("cargo::rustc-cfg=riscv");
    }
}
