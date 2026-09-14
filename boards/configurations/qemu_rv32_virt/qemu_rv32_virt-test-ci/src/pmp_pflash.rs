// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Grants the kernel direct ePMP access to the qemu-rv32-virt `pflash0`
//! device.
//!
//! `qemu_rv32_virt_lib::start()` configures the shared
//! `rv32i::pmp::kernel_protection_mml_epmp::KernelProtectionMMLEPMP` with
//! only the standard kernel `.text`, flash, RAM and MMIO regions. QEMU's
//! "virt" machine also exposes a `pflash` device outside of that MMIO
//! window, which this board uses as backing storage for isolated userspace
//! nonvolatile storage; that requires the kernel to read and write it
//! directly.
//!
//! Rather than teaching the shared `KernelProtectionMMLEPMP` about `pflash`
//! (which would grant every board built on `qemu_rv32_virt_lib` kernel
//! access to it, whether or not they use it), this board configures the
//! extra PMP entry itself, reusing the entry that `KernelProtectionMMLEPMP`
//! otherwise leaves unused between its userspace TOR regions and its own
//! kernel regions.

use kernel::utilities::registers::FieldValue;
use rv32i::csr;
use rv32i::pmp::{NAPOTRegionSpec, pmpcfg_octet};

/// Number of hardware PMP entries implemented by this platform.
///
/// Must match the `AVAILABLE_ENTRIES` const generic parameter used to
/// instantiate `KernelProtectionMMLEPMP` in
/// `chips/qemu_rv32_virt_chip/src/chip.rs`.
const AVAILABLE_ENTRIES: usize = 16;

/// Number of userspace MPU regions `KernelProtectionMMLEPMP` is configured
/// for. Must match the `MPU_REGIONS` const generic parameter used in
/// `chips/qemu_rv32_virt_chip/src/chip.rs`.
const MPU_REGIONS: usize = 5;

/// The PMP entry `KernelProtectionMMLEPMP` leaves unused, repurposed here for
/// `pflash0`.
///
/// `KernelProtectionMMLEPMP<AVAILABLE_ENTRIES, MPU_REGIONS>` uses 2 PMP
/// entries (0 and 1) for the kernel `.text` TOR region, `2 * MPU_REGIONS`
/// entries (starting at entry 2) for userspace TOR regions, and the last 3
/// entries (`AVAILABLE_ENTRIES - 3..AVAILABLE_ENTRIES`) for the FLASH, RAM
/// and MMIO regions. With `MPU_REGIONS == 5` the userspace TOR regions
/// occupy entries 2 through 11, leaving entry `AVAILABLE_ENTRIES - 4` (here,
/// 12) configured `OFF` and unlocked after `KernelProtectionMMLEPMP::new()`
/// runs -- free for this board to repurpose.
const PFLASH_PMP_ENTRY: usize = AVAILABLE_ENTRIES - 4;

const _: () = assert!(PFLASH_PMP_ENTRY >= 2 + 2 * MPU_REGIONS);

/// Base address of the first `pflash` device provided by qemu's "virt"
/// machine.
const PFLASH0_BASE: *const u8 = 0x2000_0000 as *const u8;
/// Size of the first `pflash` device (qemu unconditionally reserves 32 MiB
/// per device).
const PFLASH0_SIZE: usize = 0x0200_0000;

/// Configure PMP entry [`PFLASH_PMP_ENTRY`] to give the kernel read/write
/// (but not execute) access to `pflash0`.
///
/// Must be called after `qemu_rv32_virt_lib::start()` has set up the ePMP
/// (so that entry [`PFLASH_PMP_ENTRY`] is still `OFF` and unlocked), and
/// before any code relies on the ePMP to deny kernel access to `pflash0`.
pub fn allow_kernel_pflash_access() {
    let region = NAPOTRegionSpec::from_start_size(PFLASH0_BASE, PFLASH0_SIZE).unwrap();

    let pmpcfg: u8 = (pmpcfg_octet::a::NAPOT
        + pmpcfg_octet::r::SET
        + pmpcfg_octet::w::SET
        + pmpcfg_octet::x::CLEAR
        + pmpcfg_octet::l::SET)
        .into();

    // Important to set the address first: locking the pmpcfg register (`l`
    // bit, set above) also locks the address register.
    csr::CSR.pmpaddr_set(PFLASH_PMP_ENTRY, region.pmpaddr());
    csr::CSR.pmpconfig_modify(
        PFLASH_PMP_ENTRY / 4,
        FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
            0x0000_00FF_usize,
            (PFLASH_PMP_ENTRY % 4) * 8,
            u32::from_be_bytes([0, 0, 0, pmpcfg]) as usize,
        ),
    );
}
