// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! The EarlGrey SoC ePMP implementation.
//!
//! Refer to the main [`EarlGreyEPMP`] struct documentation.

use core::cell::Cell;
use core::fmt;
use core::marker::PhantomData;
use kernel::platform::mpu;
use kernel::utilities::registers::FieldValue;
use rv32i::csr;
use rv32i::pmp::{
    format_pmp_entries, pmpcfg_octet, NAPOTRegionSpec, TORRegionSpec, TORUserPMP, TORUserPMPCFG,
};

// ---------- EarlGrey ePMP implementation named constants ---------------------
//
// The ePMP implementation (in part) relies on these constant values. Simply
// changing them here may break the implementation below.
const PMP_ENTRIES: usize = 16;
const PMP_ENTRIES_OVER_TWO: usize = 8;
const TOR_USER_REGIONS_DEBUG_ENABLE: usize = 4;
const TOR_USER_REGIONS_DEBUG_DISABLE: usize = 4;
const TOR_USER_ENTRIES_OFFSET_DEBUG_ENABLE: usize = 0;
const TOR_USER_ENTRIES_OFFSET_DEBUG_DISABLE: usize = 4;

// ---------- EarlGrey ePMP memory region wrapper types ------------------------
//
// These types exist primarily to avoid argument confusion in the
// [`EarlGreyEPMP`] constructor, which accepts the addresses of these memory
// regions as arguments. They further encode whether a region must adhere to the
// `NAPOT` or `TOR` addressing mode constraints:

/// The EarlGrey SOC's flash memory region address range.
///
/// Configured in the PMP as a `NAPOT` region.
#[derive(Copy, Clone, Debug)]
pub struct FlashRegion(pub NAPOTRegionSpec);

/// The EarlGrey SOC's RAM region address range.
///
/// Configured in the PMP as a `NAPOT` region.
#[derive(Copy, Clone, Debug)]
pub struct RAMRegion(pub NAPOTRegionSpec);

/// The EarlGrey SOC's MMIO region address range.
///
/// Configured in the PMP as a `NAPOT` region.
#[derive(Copy, Clone, Debug)]
pub struct MMIORegion(pub NAPOTRegionSpec);

/// The EarlGrey SOC's PMP region specification for the kernel `.text` section.
///
/// This is to be made accessible to machine-mode as read-execute. Configured in
/// the PMP as a `TOR` region.
#[derive(Copy, Clone, Debug)]
pub struct KernelTextRegion(pub TORRegionSpec);

/// The EarlGrey SOC's RISC-V Debug Manager memory region.
///
/// Configured in the PMP as a read/write/execute `NAPOT` region. Because R/W/X
/// regions are not supported in machine-mode lockdown (MML) mode, to enable
/// JTAG debugging, the generic [`EPMPDebugConfig`] argument must be set to
/// [`EPMPDebugEnable`], which will configure the ePMP to operate in non
/// machine-mode lockdown (MML), but still machine-mode whitelist policy (MMWP),
/// instead.
#[derive(Copy, Clone, Debug)]
pub struct RVDMRegion(pub NAPOTRegionSpec);

// ---------- EarlGrey SoC ePMP JTAG Debugging Configuration -------------------

/// EarlGrey SoC ePMP JTAG Debugging Configuration
///
/// The EarlGrey SoC includes a RISC-V Debug Manager mapped to a NAPOT-aligned
/// memory region. To use a JTAG-debugger with the EarlGrey SoC, this region
/// needs to be allowed as R/W/X in the ePMP, at least for machine-mode.
/// However, the RISC-V ePMP does not support R/W/X regions when in machine-mode
/// lockdown (MML) mode. Furthermore, with the machine-mode whitelist policy
/// (MMWP) enabled, machine-mode (the kernel) must be given explicit access for
/// any memory regions to be accessed.
///
/// Thus, to enable debugger access, the following changes have to be made in
/// the EarlGrey ePMP from its default locked-down configuration:
///
/// - Machine-Mode Lockdown (MML) must not be enabled
///
/// - A locked (machine-mode) PMP memory region must be allocated for the RISC-V
///   Debug Manager (RVDM) allocated, and be given R/W/X permissions.
///
/// - Locked regions are enforced & locked for both machine-mode and
///   user-mode. This means that we can no longer use locked regions in
///   combination with the machine-mode whitelist policy to take away access
///   permissions from user-mode. This means that we need to place all user-mode
///   regions as non-locked regions _in front of_ all locked machine-mode
///   regions, and insert a "deny-all" non-locked fallback user-mode region in
///   between to achieve our desired isolation properties.
///
/// As a consequence, because of this "deny-all" user-mode region, we have one
/// fewer memory regions available to be used as a userspace MPU.
///
/// Because all of this is much too complex to implement at runtime (and can't
/// be reconfigured at runtime once MML is configured), we define a new trait
/// [`EPMPDebugConfig`] with two implementations [`EPMPDebugEnable`] and
/// [`EPMPDebugDisable`]. The EPMP implementation is generic over those traits
/// and can, for instance, advertise a different number of MPU regions available
/// for userspace. It further contains a method to retrieve the RVDM memory
/// region's NAPOT address specification irrespective of whether the debug
/// memory is enabled, and an associated constant to use in the configuration
/// code (such that the branches not taken can be optimized out).
pub trait EPMPDebugConfig {
    /// Whether the debug port shall be enabled or not.
    const DEBUG_ENABLE: bool;

    /// How many userspace MPU (TOR) regions are available under this
    /// configuration.
    const TOR_USER_REGIONS: usize;

    /// The offset where the user-mode TOR PMP entries start. This counts
    /// "entries", meaning `pmpaddrX` registers. A single "TOR region" uses two
    /// consecutive "entries".
    const TOR_USER_ENTRIES_OFFSET: usize;
}

pub enum EPMPDebugEnable {}
impl EPMPDebugConfig for EPMPDebugEnable {
    const DEBUG_ENABLE: bool = true;
    const TOR_USER_REGIONS: usize = TOR_USER_REGIONS_DEBUG_ENABLE;
    const TOR_USER_ENTRIES_OFFSET: usize = TOR_USER_ENTRIES_OFFSET_DEBUG_ENABLE;
}

pub enum EPMPDebugDisable {}
impl EPMPDebugConfig for EPMPDebugDisable {
    const DEBUG_ENABLE: bool = false;
    const TOR_USER_REGIONS: usize = TOR_USER_REGIONS_DEBUG_DISABLE;
    const TOR_USER_ENTRIES_OFFSET: usize = TOR_USER_ENTRIES_OFFSET_DEBUG_DISABLE;
}

/// EarlGrey ePMP Configuration Errors
#[derive(Debug, Copy, Clone)]
pub enum EarlGreyEPMPError {
    /// The ePMP driver cannot be instantiated because of an unexpected
    /// `mseccfg` register value.
    InvalidInitialMseccfgValue,
    /// The ePMP driver cannot be instantiated because of an unexpected `pmpcfg`
    /// register value (where the `usize` value contains the index of the
    /// `pmpcfg` register).
    InvalidInitialPmpcfgValue(usize),
    /// The ePMP registers do not match their expected values after
    /// configuration. The system cannot be assumed to be in a secure state.
    SanityCheckFail,
}

/// RISC-V ePMP memory protection implementation for the EarlGrey SoC.
///
/// The EarlGrey ePMP implementation hard-codes many assumptions about the
/// behavior and state of the underlying hardware, to reduce complexity of this
/// codebase, and improve its security, reliability and auditability.
///
/// Namely, it makes and checks assumptions about the machine security policy
/// prior to its initialization routine, locks down the hardware through a
/// static set of PMP configuration steps, and then exposes a subset of regions
/// for user-mode protection through the `PMPUserMPU` trait.
///
/// The EarlGrey ePMP implementation supports JTAG debug-port access through the
/// integrated RISC-V Debug Manger (RVDM) core, which requires R/W/X-access to a
/// given region of memory in machine-mode and user-mode. The [`EarlGreyEPMP`]
/// struct accepts a generic [`EPMPDebugConfig`] implementation, which either
/// enables (in the case of [`EPMPDebugEnable`]) or disables
/// ([`EPMPDebugDisable`]) the debug-port access. However, enabling debug-port
/// access can potentially weaken the system's security by not enabling
/// machine-mode lockdown (MML), and uses an additional PMP region otherwise
/// available to userspace. See the documentation of [`EPMPDebugConfig`] for
/// more information on this.
///
/// ## ePMP Region Layout & Configuration (`EPMPDebugDisable` mode)
///
/// Because of the machine-mode lockdown (MML) mode, no region can have R/W/X
/// permissions. The machine-mode whitelist policy (MMWP) further requires all
/// memory accessed by machine-mode to have a corresponding locked PMP entry
/// defined. Lower-indexed PMP entires have precedence over entries with higher
/// indices. Under MML mode, a non-locked (user-mode) entry prevents
/// machine-mode access to that memory. Thus, the ePMP is to be configured in a
/// "sandwiched" layout (with decreasing precedence):
///
/// 1. High-priority machine-mode "lockdown" entries.
///
///    These entries are only accessible to machine mode. Once locked, they can
///    only be changed through a hart reset. Examples for such memory sections
///    can be the kernel's `.text` or certain RAM (e.g. stack) sections.
///
/// 2. Tock's user-mode "MPU"
///
///    This section defines entries corresponding to memory sections made
///    accessible to user-mode. These entires are exposed through the
///    implementation of the `TORUserPMP` trait.
///
///    **Effectively, this is Tock's "MPU" sandwiched in between the
///    high-priority and low-priority PMP sections.**
///
///    These entires are not locked and must be turned off prior to the kernel
///    being able to access them.
///
///    This section must take precende over the lower kernel-mode entries, as
///    these entries are aliased by the lower kernel-mode entries. Having a
///    locked machine-mode entry take precende over an alias a user-space one
///    prevents user-mode from accessing the aliased memory.
///
/// 3. Low-priority machine-mode "accessability" entires.
///
///    These entires provide the kernel access to memory regions which are
///    (partially) aliased by user-mode regions above. This allows for
///    implementing memory sharing between userspace and the kernel (moving
///    acccess to user-mode by turning on a region above, and falling back onto
///    these rules when turning the user-mode region off).
///
///    These regions can be granular (e.g. grant R/W on the entire RAM), but
///    should not provide any excess permissions where not required (e.g.  avoid
///    granting R/X on flash-memory where only R is required, because the
///    kernel-text is already marked as R/X in the high-priority regions above.
///
/// Because the ROM_EXT and test ROM set up different ePMP configs, there are
/// separate initialization routines (`new` and `new_test_rom`) for those
/// environments.
///
/// `new` (only available when the debug-port is disabled) attempts to set up
/// the following memory protection rules and layout:
///
/// - `msseccfg` CSR:
///
///   ```text
///   |-----+-----------------------------------------------------------+-------|
///   | BIT | LABEL                                                     | STATE |
///   |-----+-----------------------------------------------------------+-------|
///   |   0 | Machine-Mode Lockdown (MML)                               |     1 |
///   |   1 | Machine-Mode Whitelist Policy (MMWP)                      |     1 |
///   |   2 | Rule-Lock Bypass (RLB)                                    |     0 |
///   |-----+-----------------------------------------------------------+-------|
///   ```
///
/// - `pmpcfgX` / `pmpaddrX` CSRs:
///
///   ```text
///   |-------+----------------------------------------+-----------+---+-------|
///   | ENTRY | REGION / ADDR                          | MODE      | L | PERMS |
///   |-------+----------------------------------------+-----------+---+-------|
///   |     0 | Locked by the ROM_EXT or unused        | NAPOT/OFF | X |       |
///   |       |                                        |           |   |       |
///   |     1 | Locked by the ROM_EXT or unused        | NAPOT/OFF | X |       |
///   |       |                                        |           |   |       |
///   |     2 | -------------------------------------- | OFF       | X | ----- |
///   |     3 | Kernel .text section                   | TOR       | X | R/X   |
///   |       |                                        |           |   |       |
///   |     4 | /                                    \ | OFF       |   |       |
///   |     5 | \ Userspace TOR region #0            / | TOR       |   | ????? |
///   |       |                                        |           |   |       |
///   |     6 | /                                    \ | OFF       |   |       |
///   |     7 | \ Userspace TOR region #1            / | TOR       |   | ????? |
///   |       |                                        |           |   |       |
///   |     8 | /                                    \ | OFF       |   |       |
///   |     9 | \ Userspace TOR region #2            / | TOR       |   | ????? |
///   |       |                                        |           |   |       |
///   |    10 | /                                    \ | OFF       |   |       |
///   |    11 | \ Userspace TOR region #3            / | TOR       |   | ????? |
///   |       |                                        |           |   |       |
///   |    12 | FLASH (spanning kernel & apps)         | NAPOT     | X | R     |
///   |       |                                        |           |   |       |
///   |    13 | -------------------------------------- | OFF       | X | ----- |
///   |       |                                        |           |   |       |
///   |    14 | RAM (spanning kernel & apps)           | NAPOT     | X | R/W   |
///   |       |                                        |           |   |       |
///   |    15 | MMIO                                   | NAPOT     | X | R/W   |
///   |-------+----------------------------------------+-----------+---+-------|
///   ```
///
/// `new_test_rom` (only available when the debug-port is disabled) attempts to
/// set up the following memory protection rules and layout:
///
/// - `msseccfg` CSR:
///
///   ```text
///   |-----+-----------------------------------------------------------+-------|
///   | BIT | LABEL                                                     | STATE |
///   |-----+-----------------------------------------------------------+-------|
///   |   0 | Machine-Mode Lockdown (MML)                               |     1 |
///   |   1 | Machine-Mode Whitelist Policy (MMWP)                      |     1 |
///   |   2 | Rule-Lock Bypass (RLB)                                    |     0 |
///   |-----+-----------------------------------------------------------+-------|
///   ```
///
/// - `pmpcfgX` / `pmpaddrX` CSRs:
///
///   ```text
///   |-------+---------------------------------------------+-------+---+-------|
///   | ENTRY | REGION / ADDR                               | MODE  | L | PERMS |
///   |-------+---------------------------------------------+-------+---+-------|
///   |     0 | ------------------------------------------- | OFF   | X | ----- |
///   |     1 | Kernel .text section                        | TOR   | X | R/X   |
///   |       |                                             |       |   |       |
///   |     2 | ------------------------------------------- | OFF   | X |       |
///   |       |                                             |       |   |       |
///   |     3 | ------------------------------------------- | OFF   | X |       |
///   |       |                                             |       |   |       |
///   |     4 | /                                         \ | OFF   |   |       |
///   |     5 | \ Userspace TOR region #0                 / | TOR   |   | ????? |
///   |       |                                             |       |   |       |
///   |     6 | /                                         \ | OFF   |   |       |
///   |     7 | \ Userspace TOR region #1                 / | TOR   |   | ????? |
///   |       |                                             |       |   |       |
///   |     8 | /                                         \ | OFF   |   |       |
///   |     9 | \ Userspace TOR region #2                 / | TOR   |   | ????? |
///   |       |                                             |       |   |       |
///   |    10 | /                                         \ | OFF   |   |       |
///   |    11 | \ Userspace TOR region #3                 / | TOR   |   | ????? |
///   |       |                                             |       |   |       |
///   |    12 | ------------------------------------------- | OFF   | X | ----- |
///   |       |                                             |       |   |       |
///   |    13 | FLASH (spanning kernel & apps)              | NAPOT | X | R     |
///   |       |                                             |       |   |       |
///   |    14 | RAM (spanning kernel & apps)                | NAPOT | X | R/W   |
///   |       |                                             |       |   |       |
///   |    15 | MMIO                                        | NAPOT | X | R/W   |
///   |-------+---------------------------------------------+-------+---+-------|
///   ```
///
/// ## ePMP Region Layout & Configuration (`EPMPDebugEnable` mode)
///
/// When enabling the RISC-V Debug Manager (JTAG debug port), the ePMP must be
/// configured differently. This is because the `RVDM` requires a memory section
/// to be mapped with read-write-execute privileges, which is not possible under
/// the machine-mode lockdown (MML) mode. However, when simply disabling MML in
/// the above policy, it would grant userspace access to kernel memory through
/// the locked PMP entires. We still need to define locked PMP entries to grant
/// the kernel (machine-mode) access to its required memory regions, as the
/// machine-mode whitelist policy (MMWP) is enabled.
///
/// Thus we split the PMP entires into three parts, as outlined in the
/// following:
///
/// 1. Tock's user-mode "MPU"
///
///    This section defines entries corresponding to memory sections made
///    accessible to user-mode. These entires are exposed through the
///    implementation of the `TORUserPMP` trait.
///
///    These entires are not locked. Because the machine-mode lockdown (MML)
///    mode is not enabled, non-locked regions are ignored in machine-mode. The
///    kernel does not have to disable these entires prior to being able to
///    access them.
///
///    This section must take precende over the lower kernel-mode entries, as
///    these entries are aliased by the lower kernel-mode entries. Having a
///    locked machine-mode entry take precende over an alias a user-space one
///    prevents user-mode from accessing the aliased memory.
///
/// 2. User-mode "deny-all" rule.
///
///    Without machine-mode lockdown (MML) mode, locked regions apply to both
///    user- and kernel-mode. Because the machine-mode whitelist policy (MMWP)
///    is enabled, the kernel must be granted explicit permission to access
///    memory (default-deny policy). This means that we must prevent any
///    user-mode access from "falling through" to kernel-mode regions. For this
///    purpose, we insert a non-locked "deny-all" rule which disallows all
///    user-mode accesses to the entire address space, if no other
///    higher-priority user-mode rule matches.
///
/// 3. Machine-mode "accessability" entires.
///
///    These entires provide the kernel access to certain memory regions, as
///    required by the machine-mode whitelist policy (MMWP).
///
/// `new_debug` (only available when the debug-port is enabled) attempts to set
/// up the following memory protection rules and layout:
///
/// - `msseccfg` CSR:
///
///   ```text
///   |-----+-----------------------------------------------------------+-------|
///   | BIT | LABEL                                                     | STATE |
///   |-----+-----------------------------------------------------------+-------|
///   |   0 | Machine-Mode Lockdown (MML)                               |     0 |
///   |   1 | Machine-Mode Whitelist Policy (MMWP)                      |     1 |
///   |   2 | Rule-Lock Bypass (RLB)                                    |     0 |
///   |-----+-----------------------------------------------------------+-------|
///   ```
///
/// - `pmpcfgX` / `pmpaddrX` CSRs:
///
///   ```text
///   |-------+---------------------------------------------+-------+---+-------|
///   | ENTRY | REGION / ADDR                               | MODE  | L | PERMS |
///   |-------+---------------------------------------------+-------+---+-------|
///   |     0 | /                                         \ | OFF   |   |       |
///   |     1 | \ Userspace TOR region #0                 / | TOR   |   | ????? |
///   |       |                                             |       |   |       |
///   |     2 | /                                         \ | OFF   |   |       |
///   |     3 | \ Userspace TOR region #1                 / | TOR   |   | ????? |
///   |       |                                             |       |   |       |
///   |     4 | /                                         \ | OFF   |   |       |
///   |     5 | \ Userspace TOR region #2                 / | TOR   |   | ????? |
///   |       |                                             |       |   |       |
///   |     6 | /                                         \ | OFF   |   |       |
///   |     7 | \ Userspace TOR region #3                 / | TOR   |   | ????? |
///   |       |                                             |       |   |       |
///   |     8 | ------------------------------------------- | OFF   |   | ----- |
///   |       |                                             |       |   |       |
///   |     9 | "Deny-all" user-mode rule (all memory)      | NAPOT |   | ----- |
///   |       |                                             |       |   |       |
///   |    10 | ------------------------------------------- | OFF   | X | ----- |
///   |    11 | Kernel .text section                        | TOR   | X | R/X   |
///   |       |                                             |       |   |       |
///   |    12 | RVDM Debug Core Memory                      | NAPOT | X | R/W/X |
///   |       |                                             |       |   |       |
///   |    13 | FLASH (spanning kernel & apps)              | NAPOT | X | R     |
///   |       |                                             |       |   |       |
///   |    14 | RAM (spanning kernel & apps)                | NAPOT | X | R/W   |
///   |       |                                             |       |   |       |
///   |    15 | MMIO                                        | NAPOT | X | R/W   |
///   |-------+---------------------------------------------+-------+---+-------|
///   ```
pub struct EarlGreyEPMP<const HANDOVER_CONFIG_CHECK: bool, DBG: EPMPDebugConfig> {
    user_pmp_enabled: Cell<bool>,
    // We can't use our generic parameter to determine the length of the
    // TORUserPMPCFG array (missing `generic_const_exprs` feature). Thus we
    // always assume that the debug-port is disabled and we can fit
    // `TOR_USER_REGIONS_DEBUG_DISABLE` user-mode TOR regions.
    shadow_user_pmpcfgs: [Cell<TORUserPMPCFG>; TOR_USER_REGIONS_DEBUG_DISABLE],
    _pd: PhantomData<DBG>,
}

impl<const HANDOVER_CONFIG_CHECK: bool> EarlGreyEPMP<{ HANDOVER_CONFIG_CHECK }, EPMPDebugDisable> {
    pub unsafe fn new(
        flash: FlashRegion,
        ram: RAMRegion,
        mmio: MMIORegion,
        kernel_text: KernelTextRegion,
    ) -> Result<Self, EarlGreyEPMPError> {
        use kernel::utilities::registers::interfaces::{Readable, Writeable};

        // --> We start with the "high-priority" ("lockdown") section of the
        // ePMP configuration:

        // Provide R/X access to the kernel .text as passed to us above.
        // Allocate a TOR region in PMP entries 2 and 3:
        csr::CSR.pmpaddr2.set(kernel_text.0.pmpaddr_a());
        csr::CSR.pmpaddr3.set(kernel_text.0.pmpaddr_b());

        // Set the appropriate `pmpcfg0` register value:
        //
        // 0x80 = 0b10000000, for start the address of the kernel .text TOR
        //        entry as well as entries 0 and 1.
        //        setting L(7) = 1, A(4-3) = OFF, X(2) = 0, W(1) = 0, R(0) = 0
        //
        // 0x8d = 0b10001101, for kernel .text TOR region
        //        setting L(7) = 1, A(4-3) = TOR,   X(2) = 1, W(1) = 0, R(0) = 1
        //
        // Note that we try to lock entries 0 and 1 into OFF mode. If the
        // ROM_EXT set these up and locked them, this will do nothing, otherwise
        // it will permanently disable these entries (preventing them from being
        // misused later).
        csr::CSR.pmpcfg0.set(0x8d_80_80_80);

        // --> Continue with the "low-priority" ("accessibility") section of the
        // ePMP configuration:

        // Configure a Read-Only NAPOT region for the entire flash (spanning
        // kernel & apps, but overlayed by the R/X kernel text TOR section)
        csr::CSR.pmpaddr12.set(flash.0.pmpaddr());

        // Configure a Read-Write NAPOT region for MMIO.
        csr::CSR.pmpaddr14.set(mmio.0.pmpaddr());

        // Configure a Read-Write NAPOT region for the entire RAM (spanning
        // kernel & apps)
        csr::CSR.pmpaddr15.set(ram.0.pmpaddr());

        // With the FLASH, RAM and MMIO configured in separate regions, we can
        // activate this new configuration, and further adjust the permissions
        // of the (currently all-capable) last PMP entry `pmpaddr15` to be R/W,
        // as required for MMIO:
        //
        // 0x99 = 0b10011001, for FLASH NAPOT region
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 0, R(0) = 1
        //
        // 0x80 = 0b10000000, for the unused region
        //        setting L(7) = 1, A(4-3) = OFF, X(2) = 0, W(1) = 0, R(0) = 0
        //
        // 0x9B = 0b10011011, for RAM & MMIO NAPOT regions
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 1, R(0) = 1
        csr::CSR.pmpcfg3.set(0x9B_9B_80_99);

        // Ensure that the other pmpcfgX CSRs are cleared:
        csr::CSR.pmpcfg1.set(0x00000000);
        csr::CSR.pmpcfg2.set(0x00000000);

        // ---------- PMP machine CSRs configured, lock down the system

        // Finally, enable machine-mode lockdown.
        // Set RLB(2) = 0, MMWP(1) = 1, MML(0) = 1
        csr::CSR.mseccfg.set(0x00000003);

        // ---------- System locked down, cross-check config

        // Now, cross-check that the CSRs have the expected values. This acts as
        // a sanity check, and can also help to protect against some set of
        // fault-injection attacks. These checks can't be optimized out by the
        // compiler, as they invoke assembly underneath which is not marked as
        // ["pure"](https://doc.rust-lang.org/reference/inline-assembly.html).
        //
        // Note that different ROM_EXT versions configure entries 0 and 1
        // differently, so we only confirm they are locked here.
        if csr::CSR.mseccfg.get() != 0x00000003
            || (csr::CSR.pmpcfg0.get() & 0xFFFF8080) != 0x8d808080
            || csr::CSR.pmpcfg1.get() != 0x00000000
            || csr::CSR.pmpcfg2.get() != 0x00000000
            || csr::CSR.pmpcfg3.get() != 0x9B9B8099
            || csr::CSR.pmpaddr2.get() != kernel_text.0.pmpaddr_a()
            || csr::CSR.pmpaddr3.get() != kernel_text.0.pmpaddr_b()
            || csr::CSR.pmpaddr12.get() != flash.0.pmpaddr()
            || csr::CSR.pmpaddr14.get() != mmio.0.pmpaddr()
            || csr::CSR.pmpaddr15.get() != ram.0.pmpaddr()
        {
            return Err(EarlGreyEPMPError::SanityCheckFail);
        }

        // The ePMP hardware was correctly configured, build the ePMP struct:
        const DEFAULT_USER_PMPCFG_OCTET: Cell<TORUserPMPCFG> = Cell::new(TORUserPMPCFG::OFF);
        Ok(EarlGreyEPMP {
            user_pmp_enabled: Cell::new(false),
            shadow_user_pmpcfgs: [DEFAULT_USER_PMPCFG_OCTET; TOR_USER_REGIONS_DEBUG_DISABLE],
            _pd: PhantomData,
        })
    }

    pub unsafe fn new_test_rom(
        flash: FlashRegion,
        ram: RAMRegion,
        mmio: MMIORegion,
        kernel_text: KernelTextRegion,
    ) -> Result<Self, EarlGreyEPMPError> {
        use kernel::utilities::registers::interfaces::{Readable, Writeable};

        if HANDOVER_CONFIG_CHECK {
            Self::check_initial_hardware_config()?;
        } else {
            // We aren't supposed to run a handover configuration check. This is
            // useful for environments which don't replicate the OpenTitan
            // EarlGrey chip behavior entirely accurately, such as
            // QEMU. However, in those environments, we cannot guarantee that
            // this configuration is actually going to work, and not break the
            // system in the meantime.
            //
            // We perform a best-effort configuration, starting by setting rule-lock
            // bypass...
            csr::CSR.mseccfg.set(0x00000004);
            // ...adding our required kernel-mode mode memory access rule...
            csr::CSR.pmpaddr15.set(0x7FFFFFFF);
            csr::CSR.pmpcfg3.set(0x9F000000);
            // ...and enabling the machine-mode whitelist policy:
            csr::CSR.mseccfg.set(0x00000006);
        }

        // ---------- HW configured as expected, start setting PMP CSRs

        // The below instructions are an intricate dance to achieve our desired
        // ePMP configuration. For correctness sake, we -- at no intermediate
        // point -- want to lose access to RAM, FLASH or MMIO.
        //
        // This is challenging, as the last section currently provides us access
        // to all of these regions, and we can't atomically change both its
        // pmpaddrX and pmpcfgX CSRs to limit it to a subset of its address
        // range and permissions. Thus, before changing the `pmpcfg3` /
        // `pmpaddr15` region, we first utilize another higher-priority CSR to
        // provide us access to one of the memory regions we'd lose access to,
        // namely we use the PMP entry 12 to provide us access to MMIO.

        // --> We start with the "high-priority" ("lockdown") section of the
        // ePMP configuration:

        // Provide R/X access to the kernel .text as passed to us above.
        // Allocate a TOR region in PMP entries 0 and 1:
        csr::CSR.pmpaddr0.set(kernel_text.0.pmpaddr_a());
        csr::CSR.pmpaddr1.set(kernel_text.0.pmpaddr_b());

        // Set the appropriate `pmpcfg0` register value:
        //
        // 0x80 = 0b10000000, for start address of the kernel .text TOR entry
        //        and to disable regions 2 & 3 (to be compatible with the
        //        non-test-rom constructor).
        //        setting L(7) = 1, A(4-3) = OFF, X(2) = 0, W(1) = 0, R(0) = 0
        //
        // 0x8d = 0b10001101, for kernel .text TOR region
        //        setting L(7) = 1, A(4-3) = TOR,   X(2) = 1, W(1) = 0, R(0) = 1
        csr::CSR.pmpcfg0.set(0x80808d80);

        // --> Continue with the "low-priority" ("accessability") section of the
        // ePMP configuration:

        // Now, onto `pmpcfg3`. As discussed above, we want to use a temporary
        // region to retain MMIO access while reconfiguring the `pmpcfg3` /
        // `pmpaddr15` register. Thus, write the MMIO region access into
        // `pmpaddr12`:
        csr::CSR.pmpaddr12.set(mmio.0.pmpaddr());

        // Configure a Read-Only NAPOT region for the entire flash (spanning
        // kernel & apps, but overlayed by the R/X kernel text TOR section)
        csr::CSR.pmpaddr13.set(flash.0.pmpaddr());

        // Configure a Read-Write NAPOT region for the entire RAM (spanning
        // kernel & apps)
        csr::CSR.pmpaddr14.set(ram.0.pmpaddr());

        // With the FLASH, RAM and MMIO configured in separate regions, we can
        // activate this new configuration, and further adjust the permissions
        // of the (currently all-capable) last PMP entry `pmpaddr15` to be R/W,
        // as required for MMIO:
        //
        // 0x99 = 0b10011001, for FLASH NAPOT region
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 0, R(0) = 1
        //
        // 0x9B = 0b10011011, for RAM & MMIO NAPOT regions
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 1, R(0) = 1
        csr::CSR.pmpcfg3.set(0x9B9B999B);

        // With the new configuration in place, we can adjust the last region's
        // address to be limited to the MMIO region, ...
        csr::CSR.pmpaddr15.set(mmio.0.pmpaddr());

        // ...and then deactivate the `pmpaddr12` fallback MMIO region
        //
        // Remove the temporary MMIO region permissions from `pmpaddr12`:
        //
        // 0x80 = 0b10000000
        //        setting L(7) = 1, A(4-3) = OFF, X(2) = 0, W(1) = 0, R(0) = 0
        //
        // 0x99 = 0b10011001, for FLASH NAPOT region
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 0, R(0) = 1
        //
        // 0x9B = 0b10011011, for RAM & MMIO NAPOT regions
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 1, R(0) = 1
        csr::CSR.pmpcfg3.set(0x9B9B9980);

        // Ensure that the other pmpcfgX CSRs are cleared:
        csr::CSR.pmpcfg1.set(0x00000000);
        csr::CSR.pmpcfg2.set(0x00000000);

        // ---------- PMP machine CSRs configured, lock down the system

        // Finally, unset the rule-lock bypass (RLB) bit. If we don't have a
        // debug memory region provided, further set machine-mode lockdown (we
        // can't enable MML and also have a R/W/X region). We also set MMWP for
        // good measure, but that shouldn't make a difference -- it can't be
        // cleared anyways as it is a sticky bit.
        //
        // Unsetting RLB with at least one locked region will mean that we can't
        // set it again, thus actually enforcing the region lock bits.
        //
        // Set RLB(2) = 0, MMWP(1) = 1, MML(0) = 1
        csr::CSR.mseccfg.set(0x00000003);

        // ---------- System locked down, cross-check config

        // Now, cross-check that the CSRs have the expected values. This acts as
        // a sanity check, and can also help to protect against some set of
        // fault-injection attacks. These checks can't be optimized out by the
        // compiler, as they invoke assembly underneath which is not marked as
        // ["pure"](https://doc.rust-lang.org/reference/inline-assembly.html).
        if csr::CSR.mseccfg.get() != 0x00000003
            || csr::CSR.pmpcfg0.get() != 0x00008d80
            || csr::CSR.pmpcfg1.get() != 0x00000000
            || csr::CSR.pmpcfg2.get() != 0x00000000
            || csr::CSR.pmpcfg3.get() != 0x9B9B9980
            || csr::CSR.pmpaddr0.get() != kernel_text.0.pmpaddr_a()
            || csr::CSR.pmpaddr1.get() != kernel_text.0.pmpaddr_b()
            || csr::CSR.pmpaddr13.get() != flash.0.pmpaddr()
            || csr::CSR.pmpaddr14.get() != ram.0.pmpaddr()
            || csr::CSR.pmpaddr15.get() != mmio.0.pmpaddr()
        {
            return Err(EarlGreyEPMPError::SanityCheckFail);
        }

        // The ePMP hardware was correctly configured, build the ePMP struct:
        const DEFAULT_USER_PMPCFG_OCTET: Cell<TORUserPMPCFG> = Cell::new(TORUserPMPCFG::OFF);
        Ok(EarlGreyEPMP {
            user_pmp_enabled: Cell::new(false),
            shadow_user_pmpcfgs: [DEFAULT_USER_PMPCFG_OCTET; TOR_USER_REGIONS_DEBUG_DISABLE],
            _pd: PhantomData,
        })
    }
}

impl<const HANDOVER_CONFIG_CHECK: bool> EarlGreyEPMP<{ HANDOVER_CONFIG_CHECK }, EPMPDebugEnable> {
    pub unsafe fn new_debug(
        flash: FlashRegion,
        ram: RAMRegion,
        mmio: MMIORegion,
        kernel_text: KernelTextRegion,
        debug_memory: RVDMRegion,
    ) -> Result<Self, EarlGreyEPMPError> {
        use kernel::utilities::registers::interfaces::{Readable, Writeable};

        if HANDOVER_CONFIG_CHECK {
            Self::check_initial_hardware_config()?;
        } else {
            // We aren't supposed to run a handover configuration check. This is
            // useful for environments which don't replicate the OpenTitan
            // EarlGrey chip behavior entirely accurately, such as
            // QEMU. However, in those environments, we cannot guarantee that
            // this configuration is actually going to work, and not break the
            // system in the meantime.
            //
            // We perform a best-effort configuration, starting by setting rule-lock
            // bypass...
            csr::CSR.mseccfg.set(0x00000004);
            // ...adding our required kernel-mode mode memory access rule...
            csr::CSR.pmpaddr15.set(0x7FFFFFFF);
            csr::CSR.pmpcfg3.set(0x9F000000);
            // ...and enabling the machine-mode whitelist policy:
            csr::CSR.mseccfg.set(0x00000006);
        }

        // ---------- HW configured as expected, start setting PMP CSRs

        // The below instructions are an intricate dance to achieve our desired
        // ePMP configuration. For correctness sake, we -- at no intermediate
        // point -- want to lose access to RAM, FLASH or MMIO.
        //
        // This is challenging, as the last section currently provides us access
        // to all of these regions, and we can't atomically change both its
        // pmpaddrX and pmpcfgX CSRs to limit it to a subset of its address
        // range and permissions. Thus, before changing the `pmpcfg3` /
        // `pmpaddr15` region, we first utilize another higher-priority CSR to
        // provide us access to one of the memory regions we'd lose access to,
        // namely we use the PMP entry 12 to provide us access to MMIO.

        // Provide R/X access to the kernel .text as passed to us above.
        // Allocate a TOR region in PMP entries 10 and 11:
        csr::CSR.pmpaddr10.set(kernel_text.0.pmpaddr_a());
        csr::CSR.pmpaddr11.set(kernel_text.0.pmpaddr_b());

        // Set the appropriate `pmpcfg2` register value:
        //
        // 0x80 = 0b10000000, for start address of the kernel .text TOR entry
        //        setting L(7) = 1, A(4-3) = OFF, X(2) = 0, W(1) = 0, R(0) = 0
        //
        // 0x8d = 0b10001101, for kernel .text TOR region
        //        setting L(7) = 1, A(4-3) = TOR,   X(2) = 1, W(1) = 0, R(0) = 1
        csr::CSR.pmpcfg2.set(0x8d800000);

        // Now, onto `pmpcfg3`. As discussed above, we want to use a temporary
        // region to retain MMIO access while reconfiguring the `pmpcfg3` /
        // `pmpaddr15` register. Thus, write the MMIO region access into
        // `pmpaddr12`:
        csr::CSR.pmpaddr12.set(mmio.0.pmpaddr());

        // Configure a Read-Only NAPOT region for the entire flash (spanning
        // kernel & apps, but overlayed by the R/X kernel text TOR section)
        csr::CSR.pmpaddr13.set(flash.0.pmpaddr());

        // Configure a Read-Write NAPOT region for the entire RAM (spanning
        // kernel & apps)
        csr::CSR.pmpaddr14.set(ram.0.pmpaddr());

        // With the FLASH, RAM and MMIO configured in separate regions, we can
        // activate this new configuration, and further adjust the permissions
        // of the (currently all-capable) last PMP entry `pmpaddr15` to be R/W,
        // as required for MMIO:
        //
        // 0x99 = 0b10011001, for FLASH NAPOT region
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 0, R(0) = 1
        //
        // 0x9B = 0b10011011, for RAM & MMIO NAPOT regions
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 1, R(0) = 1
        csr::CSR.pmpcfg3.set(0x9B9B999B);

        // With the new configuration in place, we can adjust the last region's
        // address to be limited to the MMIO region, ...
        csr::CSR.pmpaddr15.set(mmio.0.pmpaddr());

        // ...and then repurpose `pmpaddr12` for the debug port:
        csr::CSR.pmpaddr12.set(debug_memory.0.pmpaddr());

        // 0x9F = 0b10011111, for RVDM R/W/X memory region
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 1, W(1) = 1, R(0) = 1
        //
        // 0x99 = 0b10011001, for FLASH NAPOT region
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 0, R(0) = 1
        //
        // 0x9B = 0b10011011, for RAM & MMIO NAPOT regions
        //        setting L(7) = 1, A(4-3) = NAPOT, X(2) = 0, W(1) = 1, R(0) = 1
        csr::CSR.pmpcfg3.set(0x9B9B999F);

        // Ensure that the other pmpcfgX CSRs are cleared:
        csr::CSR.pmpcfg0.set(0x00000000);
        csr::CSR.pmpcfg1.set(0x00000000);

        // ---------- PMP machine CSRs configured, lock down the system

        // Finally, unset the rule-lock bypass (RLB) bit. If we don't have a
        // debug memory region provided, further set machine-mode lockdown (we
        // can't enable MML and also have a R/W/X region). We also set MMWP for
        // good measure, but that shouldn't make a difference -- it can't be
        // cleared anyways as it is a sticky bit.
        //
        // Unsetting RLB with at least one locked region will mean that we can't
        // set it again, thus actually enforcing the region lock bits.
        //
        // Set RLB(2) = 0, MMWP(1) = 1, MML(0) = 0
        csr::CSR.mseccfg.set(0x00000002);

        // ---------- System locked down, cross-check config

        // Now, cross-check that the CSRs have the expected values. This acts as
        // a sanity check, and can also help to protect against some set of
        // fault-injection attacks. These checks can't be optimized out by the
        // compiler, as they invoke assembly underneath which is not marked as
        // ["pure"](https://doc.rust-lang.org/reference/inline-assembly.html).
        if csr::CSR.mseccfg.get() != 0x00000002
            || csr::CSR.pmpcfg0.get() != 0x00000000
            || csr::CSR.pmpcfg1.get() != 0x00000000
            || csr::CSR.pmpcfg2.get() != 0x8d800000
            || csr::CSR.pmpcfg3.get() != 0x9B9B999F
            || csr::CSR.pmpaddr10.get() != kernel_text.0.pmpaddr_a()
            || csr::CSR.pmpaddr11.get() != kernel_text.0.pmpaddr_b()
            || csr::CSR.pmpaddr12.get() != debug_memory.0.pmpaddr()
            || csr::CSR.pmpaddr13.get() != flash.0.pmpaddr()
            || csr::CSR.pmpaddr14.get() != ram.0.pmpaddr()
            || csr::CSR.pmpaddr15.get() != mmio.0.pmpaddr()
        {
            return Err(EarlGreyEPMPError::SanityCheckFail);
        }

        // Now, as we're not in the machine-mode lockdown (MML) mode, locked PMP
        // regions will still be accessible to userspace. To prevent our
        // kernel-mode access regions from being accessible to user-mode, we use
        // the last user-mode TOR region (`pmpaddr9`) to configure a
        // "protection" region which disallows access to all memory that has not
        // otherwise been granted access to.
        csr::CSR.pmpaddr9.set(0x7FFFFFFF); // the entire address space

        // And finally apply this configuration to the `pmpcfg2` CSR. For good
        // measure, we also include the locked regions (which we can no longer
        // modify thanks to RLB = 0).
        //
        // 0x18 = 0b00011000, to revoke user-mode perms to all memory
        //        setting L(7) = 0, A(4-3) = NAPOT, X(2) = 0, W(1) = 0, R(0) = 0
        //
        // 0x80 = 0b10000000, for start address of the kernel .text TOR entry
        //        setting L(7) = 1, A(4-3) = OFF, X(2) = 0, W(1) = 0, R(0) = 0
        //
        // 0x8d = 0b10001101, for kernel .text TOR region
        //        setting L(7) = 1, A(4-3) = TOR,   X(2) = 1, W(1) = 0, R(0) = 1
        csr::CSR.pmpcfg2.set(0x8d81800);

        // The ePMP hardware was correctly configured, build the ePMP struct:
        const DEFAULT_USER_PMPCFG_OCTET: Cell<TORUserPMPCFG> = Cell::new(TORUserPMPCFG::OFF);
        let epmp = EarlGreyEPMP {
            user_pmp_enabled: Cell::new(false),
            shadow_user_pmpcfgs: [DEFAULT_USER_PMPCFG_OCTET; TOR_USER_REGIONS_DEBUG_DISABLE],
            _pd: PhantomData,
        };

        Ok(epmp)
    }
}

impl<const HANDOVER_CONFIG_CHECK: bool, DBG: EPMPDebugConfig>
    EarlGreyEPMP<{ HANDOVER_CONFIG_CHECK }, DBG>
{
    fn check_initial_hardware_config() -> Result<(), EarlGreyEPMPError> {
        use kernel::utilities::registers::interfaces::Readable;

        // This initialization code is written to work with 16 PMP entries. Add
        // an explicit assertion such that things break when the constant above
        // is changed:
        #[allow(clippy::assertions_on_constants)]
        const _: () = assert!(
            PMP_ENTRIES_OVER_TWO == 8,
            "EarlGrey ePMP initialization is written for 16 PMP entries.",
        );

        // ---------- Check current HW config

        // Ensure that the `mseccfg` CSR has the expected value, namely that
        // we're in "machine-mode whitelist policy" and have "rule-lock bypass"
        // enabled. If this register has an unexpected value, we risk
        // accidentally revoking important permissions for the Tock kernel
        // itself.
        if csr::CSR.mseccfg.get() != 0x00000006 {
            return Err(EarlGreyEPMPError::InvalidInitialMseccfgValue);
        }

        // We assume the very last PMP region is set to provide us RXW access to
        // the entirety of memory, and all other regions are disabled. Check the
        // CSRs to make sure that this is indeed the case.
        for i in 0..(PMP_ENTRIES_OVER_TWO / 2 - 1) {
            // 0x98 = 0b10011000, extracting L(7) and A(4-3) bits.
            if csr::CSR.pmpconfig_get(i) & 0x98989898 != 0x00000000 {
                return Err(EarlGreyEPMPError::InvalidInitialPmpcfgValue(i));
            }
        }

        // The last CSR is special, as we expect it to contain the NAPOT region
        // which currently gives us memory access.
        //
        // 0x98 = 0b10011000, extracting L(7) and A(4-3) bits.
        // 0x9F = 0b10011111, extracing L(7), A(4-3), X(2), W(1), R(0) bits.
        if csr::CSR.pmpconfig_get(PMP_ENTRIES_OVER_TWO / 2 - 1) & 0x9F989898 != 0x9F000000 {
            return Err(EarlGreyEPMPError::InvalidInitialPmpcfgValue(
                PMP_ENTRIES_OVER_TWO / 2 - 1,
            ));
        }

        Ok(())
    }

    // ---------- Backing functions for the TORUserPMP implementations ---------
    //
    // The EarlGrey ePMP implementations of `TORUserPMP` differ between
    // `EPMPDebugEnable` and `EPMPDebugDisable` configurations. These backing
    // functions here are applicable to both, and called by those trait
    // implementations respectively:

    fn user_available_regions<const TOR_USER_REGIONS: usize>(&self) -> usize {
        // Always assume to have `TOR_USER_REGIONS` usable TOR regions. We have a
        // fixed number of kernel memory protection regions, and a fixed mapping
        // of user regions to hardware PMP entries.
        TOR_USER_REGIONS
    }

    fn user_configure_pmp<const TOR_USER_REGIONS: usize>(
        &self,
        regions: &[(TORUserPMPCFG, *const u8, *const u8); TOR_USER_REGIONS],
    ) -> Result<(), ()> {
        // Configure all of the regions' addresses and store their pmpcfg octets
        // in our shadow storage. If the user PMP is already enabled, we further
        // apply this configuration (set the pmpcfgX CSRs) by running
        // `enable_user_pmp`:
        for (i, (region, shadow_user_pmpcfg)) in regions
            .iter()
            .zip(self.shadow_user_pmpcfgs.iter())
            .enumerate()
        {
            // The ePMP in MML mode does not support read-write-execute
            // regions. If such a region is to be configured, abort. As this
            // loop here only modifies the shadow state, we can simply abort and
            // return an error. We don't make any promises about the ePMP state
            // if the configuration files, but it is still being activated with
            // `enable_user_pmp`:
            if region.0.get()
                == <TORUserPMPCFG as From<mpu::Permissions>>::from(
                    mpu::Permissions::ReadWriteExecute,
                )
                .get()
            {
                return Err(());
            }

            // Set the CSR addresses for this region (if its not OFF, in which
            // case the hardware-configured addresses are irrelevant):
            if region.0 != TORUserPMPCFG::OFF {
                csr::CSR.pmpaddr_set(
                    DBG::TOR_USER_ENTRIES_OFFSET + (i * 2) + 0,
                    (region.1 as usize).overflowing_shr(2).0,
                );
                csr::CSR.pmpaddr_set(
                    DBG::TOR_USER_ENTRIES_OFFSET + (i * 2) + 1,
                    (region.2 as usize).overflowing_shr(2).0,
                );
            }

            // Store the region's pmpcfg octet:
            shadow_user_pmpcfg.set(region.0);
        }

        // If the PMP is currently active, apply the changes to the CSRs:
        if self.user_pmp_enabled.get() {
            self.user_enable_user_pmp()?;
        }

        Ok(())
    }

    fn user_enable_user_pmp(&self) -> Result<(), ()> {
        // Currently, this code requires the TOR regions to start at an even PMP
        // region index. Assert that this is indeed the case:
        #[allow(clippy::let_unit_value)]
        let _: () = assert!(DBG::TOR_USER_ENTRIES_OFFSET % 2 == 0);

        // We store the "enabled" PMPCFG octets of user regions in the
        // `shadow_user_pmpcfg` field, such that we can re-enable the PMP
        // without a call to `configure_pmp` (where the `TORUserPMPCFG`s are
        // provided by the caller).

        // Could use `iter_array_chunks` once that's stable.
        //
        // Limit iteration to `DBG::TOR_USER_REGIONS` to avoid overwriting any
        // configured debug regions in the last user-mode TOR region.
        let mut shadow_user_pmpcfgs_iter = self.shadow_user_pmpcfgs[..DBG::TOR_USER_REGIONS].iter();
        let mut i = DBG::TOR_USER_ENTRIES_OFFSET / 2;

        while let Some(first_region_pmpcfg) = shadow_user_pmpcfgs_iter.next() {
            // If we're at a "region" offset divisible by two (where "region" =
            // 2 PMP "entries"), then we can configure an entire `pmpcfgX` CSR
            // in one operation. As CSR writes are expensive, this is an
            // operation worth making:
            let second_region_opt = if i % 2 == 0 {
                shadow_user_pmpcfgs_iter.next()
            } else {
                None
            };

            if let Some(second_region_pmpcfg) = second_region_opt {
                // We're at an even index and have two regions to configure, so
                // do that with a single CSR write:
                csr::CSR.pmpconfig_set(
                    i / 2,
                    u32::from_be_bytes([
                        second_region_pmpcfg.get().get(),
                        TORUserPMPCFG::OFF.get(),
                        first_region_pmpcfg.get().get(),
                        TORUserPMPCFG::OFF.get(),
                    ]) as usize,
                );

                i += 2;
            } else if i % 2 == 0 {
                // This is a single region at an even index. Thus, modify the
                // first two pmpcfgX octets for this region.
                csr::CSR.pmpconfig_modify(
                    i / 2,
                    FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                        0x0000FFFF,
                        0, // lower two octets
                        u32::from_be_bytes([
                            0,
                            0,
                            first_region_pmpcfg.get().get(),
                            TORUserPMPCFG::OFF.get(),
                        ]) as usize,
                    ),
                );

                i += 1;
            } else {
                // This is a single region at an odd index. Thus, modify the
                // latter two pmpcfgX octets for this region.
                csr::CSR.pmpconfig_modify(
                    i / 2,
                    FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                        0x0000FFFF,
                        16, // higher two octets
                        u32::from_be_bytes([
                            0,
                            0,
                            first_region_pmpcfg.get().get(),
                            TORUserPMPCFG::OFF.get(),
                        ]) as usize,
                    ),
                );

                i += 1;
            }
        }

        self.user_pmp_enabled.set(true);

        Ok(())
    }

    fn user_disable_user_pmp(&self) {
        // Simply set all of the user-region pmpcfg octets to OFF:
        let mut user_region_pmpcfg_octet_pairs = (DBG::TOR_USER_ENTRIES_OFFSET / 2)
            ..((DBG::TOR_USER_ENTRIES_OFFSET / 2) + DBG::TOR_USER_REGIONS);

        while let Some(first_region_idx) = user_region_pmpcfg_octet_pairs.next() {
            let second_region_opt = if first_region_idx % 2 == 0 {
                user_region_pmpcfg_octet_pairs.next()
            } else {
                None
            };

            if let Some(_second_region_idx) = second_region_opt {
                // We're at an even index and have two regions to configure, so
                // do that with a single CSR write:
                csr::CSR.pmpconfig_set(
                    first_region_idx / 2,
                    u32::from_be_bytes([
                        TORUserPMPCFG::OFF.get(),
                        TORUserPMPCFG::OFF.get(),
                        TORUserPMPCFG::OFF.get(),
                        TORUserPMPCFG::OFF.get(),
                    ]) as usize,
                );
            } else if first_region_idx % 2 == 0 {
                // This is a single region at an even index. Thus, modify the
                // first two pmpcfgX octets for this region.
                csr::CSR.pmpconfig_modify(
                    first_region_idx / 2,
                    FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                        0x0000FFFF,
                        0, // lower two octets
                        u32::from_be_bytes([
                            0,
                            0,
                            TORUserPMPCFG::OFF.get(),
                            TORUserPMPCFG::OFF.get(),
                        ]) as usize,
                    ),
                );
            } else {
                // This is a single region at an odd index. Thus, modify the
                // latter two pmpcfgX octets for this region.
                csr::CSR.pmpconfig_modify(
                    first_region_idx / 2,
                    FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                        0x0000FFFF,
                        16, // higher two octets
                        u32::from_be_bytes([
                            0,
                            0,
                            TORUserPMPCFG::OFF.get(),
                            TORUserPMPCFG::OFF.get(),
                        ]) as usize,
                    ),
                );
            }
        }

        self.user_pmp_enabled.set(false);
    }
}

impl<const HANDOVER_CONFIG_CHECK: bool> TORUserPMP<{ TOR_USER_REGIONS_DEBUG_ENABLE }>
    for EarlGreyEPMP<{ HANDOVER_CONFIG_CHECK }, EPMPDebugEnable>
{
    // Don't require any const-assertions in the EarlGreyEPMP.
    const CONST_ASSERT_CHECK: () = ();

    fn available_regions(&self) -> usize {
        self.user_available_regions::<TOR_USER_REGIONS_DEBUG_ENABLE>()
    }

    fn configure_pmp(
        &self,
        regions: &[(TORUserPMPCFG, *const u8, *const u8); TOR_USER_REGIONS_DEBUG_ENABLE],
    ) -> Result<(), ()> {
        self.user_configure_pmp::<TOR_USER_REGIONS_DEBUG_ENABLE>(regions)
    }

    fn enable_user_pmp(&self) -> Result<(), ()> {
        self.user_enable_user_pmp()
    }

    fn disable_user_pmp(&self) {
        // Technically, the `disable_user_pmp` can be implemented as a no-op in
        // the debug-mode ePMP, as machine-mode lockdown (MML) is not enabled.
        // However, we still execercise these routines to stay as close to the
        // non-debug ePMP configuration as possible:
        self.user_disable_user_pmp()
    }
}

impl<const HANDOVER_CONFIG_CHECK: bool> TORUserPMP<{ TOR_USER_REGIONS_DEBUG_DISABLE }>
    for EarlGreyEPMP<{ HANDOVER_CONFIG_CHECK }, EPMPDebugDisable>
{
    // Don't require any const-assertions in the EarlGreyEPMP.
    const CONST_ASSERT_CHECK: () = ();

    fn available_regions(&self) -> usize {
        self.user_available_regions::<TOR_USER_REGIONS_DEBUG_DISABLE>()
    }

    fn configure_pmp(
        &self,
        regions: &[(TORUserPMPCFG, *const u8, *const u8); TOR_USER_REGIONS_DEBUG_DISABLE],
    ) -> Result<(), ()> {
        self.user_configure_pmp::<TOR_USER_REGIONS_DEBUG_DISABLE>(regions)
    }

    fn enable_user_pmp(&self) -> Result<(), ()> {
        self.user_enable_user_pmp()
    }

    fn disable_user_pmp(&self) {
        self.user_disable_user_pmp()
    }
}

impl<const HANDOVER_CONFIG_CHECK: bool, DBG: EPMPDebugConfig> fmt::Display
    for EarlGreyEPMP<{ HANDOVER_CONFIG_CHECK }, DBG>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use kernel::utilities::registers::interfaces::Readable;

        write!(f, " EarlGrey ePMP configuration:\r\n")?;
        write!(
            f,
            "  mseccfg: {:#08X}, user-mode PMP active: {:?}\r\n",
            csr::CSR.mseccfg.get(),
            self.user_pmp_enabled.get()
        )?;
        unsafe { format_pmp_entries::<PMP_ENTRIES>(f) }?;

        write!(f, "  Shadow PMP entries for user-mode:\r\n")?;
        for (i, shadowed_pmpcfg) in self.shadow_user_pmpcfgs[..DBG::TOR_USER_REGIONS]
            .iter()
            .enumerate()
        {
            let (start_pmpaddr_label, startaddr_pmpaddr, endaddr, mode) =
                if shadowed_pmpcfg.get() == TORUserPMPCFG::OFF {
                    (
                        "pmpaddr",
                        csr::CSR.pmpaddr_get(DBG::TOR_USER_ENTRIES_OFFSET + (i * 2)),
                        0,
                        "OFF",
                    )
                } else {
                    (
                        "  start",
                        csr::CSR
                            .pmpaddr_get(DBG::TOR_USER_ENTRIES_OFFSET + (i * 2))
                            .overflowing_shl(2)
                            .0,
                        csr::CSR
                            .pmpaddr_get(DBG::TOR_USER_ENTRIES_OFFSET + (i * 2) + 1)
                            .overflowing_shl(2)
                            .0
                            | 0b11,
                        "TOR",
                    )
                };

            write!(
                f,
                "  [{:02}]: {}={:#010X}, end={:#010X}, cfg={:#04X} ({}) ({}{}{}{})\r\n",
                DBG::TOR_USER_ENTRIES_OFFSET + (i * 2) + 1,
                start_pmpaddr_label,
                startaddr_pmpaddr,
                endaddr,
                shadowed_pmpcfg.get().get(),
                mode,
                if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::l) {
                    "l"
                } else {
                    "-"
                },
                if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::r) {
                    "r"
                } else {
                    "-"
                },
                if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::w) {
                    "w"
                } else {
                    "-"
                },
                if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::x) {
                    "x"
                } else {
                    "-"
                },
            )?;
        }

        Ok(())
    }
}
