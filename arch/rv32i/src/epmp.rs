// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Implementation of the enhanced physical memory protection unit (ePMP).
//!
//! ## Implementation
//!
//! We use the PMP Top of Region (TOR) alignment as there are alignment issues
//! with NAPOT. NAPOT would allow us to protect more memory regions (with NAPOT
//! each PMP region can be a memory region), but the problem with NAPOT is the
//! address must be aligned to the size, which results in wasted memory. To
//! avoid this wasted memory we use TOR and each memory region uses two physical
//! PMP regions.

use crate::csr;
use core::cell::Cell;
use core::num::NonZeroUsize;
use core::{cmp, fmt};
use kernel::platform::mpu;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable};
use kernel::utilities::registers::{self, register_bitfields};

// Generic PMP config
register_bitfields![u8,
    pub pmpcfg [
        r OFFSET(0) NUMBITS(1) [],
        w OFFSET(1) NUMBITS(1) [],
        x OFFSET(2) NUMBITS(1) [],
        a OFFSET(3) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l OFFSET(7) NUMBITS(1) []
    ]
];

/// Main PMP struct.
///
/// Tock will ignore locked PMP regions. Note that Tock will not make any
/// attempt to avoid access faults from locked regions.
///
/// `MAX_AVAILABLE_REGIONS_OVER_TWO`: The number of PMP regions divided by 2.
///  The RISC-V spec mandates that there must be either 0, 16 or 64 PMP
///  regions implemented. If you are using this PMP struct we are assuming
///  there is more then 0 implemented. So this value should be either 8 or 32.
///
///  If however you know the exact number of PMP regions implemented by your
///  platform and it's not going to change you can just specify the number.
///  This means that Tock won't be able to dynamically handle more regions,
///  but it will reduce runtime space requirements.
///  Note: that this does not mean all PMP regions are connected.
///  Some of the regions can be WARL (Write Any Read Legal). All this means
///  is that accessing `NUM_REGIONS` won't cause a fault.
pub struct PMP<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> {
    /// Monotonically increasing counter for allocated regions. This is used to
    /// determine whether the PMP has already been configured for the supplied
    /// configuration:
    config_count: Cell<NonZeroUsize>,
    /// The configuration that the PMP was last configured for. Used (along with
    /// the `is_dirty` flag) to determine if PMP can skip writing the
    /// configuration to hardware.
    last_configured_for: OptionalCell<NonZeroUsize>,
    /// This is a 64-bit mask of locked regions.
    /// Each bit that is set in this mask indicates that the region is locked
    /// and cannot be used by Tock.
    locked_region_mask: Cell<u64>,
    /// This is the total number of available regions.
    /// This will be between 0 and MAX_AVAILABLE_REGIONS_OVER_TWO * 2 depending
    /// on the hardware and previous boot stages.
    num_regions: usize,
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> PMP<MAX_AVAILABLE_REGIONS_OVER_TWO> {
    pub unsafe fn new() -> Self {
        // RISC-V PMP can support from 0 to 64 PMP regions
        // Let's figure out how many are supported.
        // We count any regions that are locked as unsupported
        let mut num_regions = 0;
        let mut locked_region_mask = 0;

        if csr::CSR.mseccfg.is_set(csr::mseccfg::mseccfg::mmwp) {
            // The MMWP bit is set, we need to be very careful about modifying
            // PMP configs as that might break us
            if csr::CSR.mseccfg.is_set(csr::mseccfg::mseccfg::rlb) {
                // Rule Locking Bypass (RLB) is set, we can do whatever we want
                // so let's just say all regions are modifiable and avoid the
                // auto-detect to avoid locking ourself out.
                num_regions = MAX_AVAILABLE_REGIONS_OVER_TWO * 2;
            } else {
                // We can't probe the registers by writing to them, so let's
                // just assume all `MAX_AVAILABLE_REGIONS_OVER_TWO * 2` are
                // accessible if they aren't locked

                for i in 0..(MAX_AVAILABLE_REGIONS_OVER_TWO * 2) {
                    // Read the current value
                    let pmpcfg_og = csr::CSR.pmpconfig_get(i / 4);

                    // Check if the locked bit is set
                    if pmpcfg_og & ((1 << 7) << ((i % 4) * 8)) > 0 {
                        // The bit is locked. Mark this regions as not usable
                        locked_region_mask |= 1 << i;
                    } else {
                        // Found a working region
                        num_regions += 1;
                    }
                }
            }
        } else {
            for i in 0..(MAX_AVAILABLE_REGIONS_OVER_TWO * 2) {
                // Read the current value
                let pmpcfg_og = csr::CSR.pmpconfig_get(i / 4);

                // Flip R, W, X bits and set config to off
                let cfg_offset = (i % 4) * 8;
                let flipped_bits = pmpcfg_og ^ (5 << cfg_offset);
                let pmpcfg_new = flipped_bits & !(3 << (cfg_offset + 3));
                csr::CSR.pmpconfig_set(i / 4, pmpcfg_new);

                // Check if the bits are set
                let pmpcfg_check = csr::CSR.pmpconfig_get(i / 4);

                // Check if the changes stuck
                if pmpcfg_check == pmpcfg_og {
                    // If we get here then our changes didn't stick, let's figure
                    // out why

                    // Check if the locked bit is set
                    if pmpcfg_og & ((1 << 7) << ((i % 4) * 8)) > 0 {
                        // The bit is locked. Mark this regions as not usable
                        locked_region_mask |= 1 << i;
                    } else {
                        // The locked bit isn't set
                        // This region must not be connected, which means we have run out
                        // of usable regions, break the loop
                        break;
                    }
                } else {
                    // Found a working region
                    num_regions += 1;
                }

                // Reset back to how we found it
                csr::CSR.pmpconfig_set(i / 4, pmpcfg_og);
            }
        }

        Self {
            config_count: Cell::new(NonZeroUsize::MIN),
            last_configured_for: OptionalCell::empty(),
            num_regions,
            locked_region_mask: Cell::new(locked_region_mask),
        }
    }
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> fmt::Display
    for PMP<MAX_AVAILABLE_REGIONS_OVER_TWO>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn bit_str<'a>(cfg: u8, bit: u8, on_str: &'a str, off_str: &'a str) -> &'a str {
            match cfg & bit {
                0 => off_str,
                _ => on_str,
            }
        }

        fn enabled_str<'a>(cfg: u8) -> &'a str {
            if cfg & pmpcfg::a::OFF.mask() == pmpcfg::a::OFF.value {
                "OFF"
            } else if cfg & pmpcfg::a::TOR.mask() == pmpcfg::a::TOR.value {
                "TOR"
            } else if cfg & pmpcfg::a::NA4.mask() == pmpcfg::a::NA4.value {
                "NA4"
            } else if cfg & pmpcfg::a::NAPOT.mask() == pmpcfg::a::NAPOT.value {
                "NAPOT"
            } else {
                unreachable!()
            }
        }

        write!(f, " ePMP regions:\r\n")?;

        for i in 0..(MAX_AVAILABLE_REGIONS_OVER_TWO * 2) {
            // Read the current value
            let pmpcfg = (csr::CSR.pmpconfig_get(i / 4) >> ((i % 4) * 8)) as u8;
            let pmpaddr0 = if i > 0 {
                csr::CSR.pmpaddr_get(i - 1) << 2
            } else {
                0
            };
            let pmpaddr1 = csr::CSR.pmpaddr_get(i) << 2;

            write!(
                f,
                "  [{}]: addr={:#010X}, end={:#010X}, cfg={:#X} ({}) ({}{}{}{})\r\n",
                i,
                pmpaddr0,
                pmpaddr1,
                pmpcfg,
                enabled_str(pmpcfg),
                bit_str(pmpcfg, pmpcfg::l::SET.value, "l", "-"),
                bit_str(pmpcfg, pmpcfg::r::SET.value, "r", "-"),
                bit_str(pmpcfg, pmpcfg::w::SET.value, "w", "-"),
                bit_str(pmpcfg, pmpcfg::x::SET.value, "x", "-"),
            )?;
        }

        Ok(())
    }
}

/// Struct storing configuration for a RISC-V PMP region.
#[derive(Copy, Clone)]
pub struct PMPRegion {
    location: (*const u8, usize),
    cfg: registers::FieldValue<u8, pmpcfg::Register>,
}

impl PartialEq<mpu::Region> for PMPRegion {
    fn eq(&self, other: &mpu::Region) -> bool {
        self.location.0 == other.start_address() && self.location.1 == other.size()
    }
}

impl fmt::Display for PMPRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn bit_str<'a>(reg: &PMPRegion, bit: u8, on_str: &'a str, off_str: &'a str) -> &'a str {
            match reg.cfg.value & bit {
                0 => off_str,
                _ => on_str,
            }
        }

        write!(
            f,
            "addr={:p}, size={:#010X}, cfg={:#X} ({}{}{}{})",
            self.location.0,
            self.location.1,
            u8::from(self.cfg),
            bit_str(self, pmpcfg::l::SET.value, "l", "-"),
            bit_str(self, pmpcfg::r::SET.value, "r", "-"),
            bit_str(self, pmpcfg::w::SET.value, "w", "-"),
            bit_str(self, pmpcfg::x::SET.value, "x", "-"),
        )
    }
}

impl PMPRegion {
    /// Create a new PMPRegion for use by apps
    fn new_app(start: *const u8, size: usize, permissions: mpu::Permissions) -> Option<PMPRegion> {
        // Determine access and execute permissions
        let pmpcfg = match permissions {
            mpu::Permissions::ReadWriteExecute => {
                // App has read/write/execute, kernel can't access
                pmpcfg::l::CLEAR + pmpcfg::r::SET + pmpcfg::w::SET + pmpcfg::x::SET + pmpcfg::a::TOR
            }
            mpu::Permissions::ReadWriteOnly => {
                // App and kernel can both read/write
                pmpcfg::l::CLEAR
                    + pmpcfg::r::SET
                    + pmpcfg::w::SET
                    + pmpcfg::x::CLEAR
                    + pmpcfg::a::TOR
            }
            mpu::Permissions::ReadExecuteOnly => {
                // App has read/execute, kernel can't access
                pmpcfg::l::CLEAR
                    + pmpcfg::r::SET
                    + pmpcfg::w::CLEAR
                    + pmpcfg::x::SET
                    + pmpcfg::a::TOR
            }
            mpu::Permissions::ReadOnly => {
                // App has read, kernel can't access
                pmpcfg::l::CLEAR
                    + pmpcfg::r::SET
                    + pmpcfg::w::CLEAR
                    + pmpcfg::x::CLEAR
                    + pmpcfg::a::TOR
            }
            mpu::Permissions::ExecuteOnly => {
                // App has execute only, kernel can't access
                pmpcfg::l::CLEAR
                    + pmpcfg::r::CLEAR
                    + pmpcfg::w::CLEAR
                    + pmpcfg::x::SET
                    + pmpcfg::a::TOR
            }
        };

        Some(PMPRegion {
            location: (start, size),
            cfg: pmpcfg,
        })
    }

    /// Create a new PMPRegion for use by the kernel
    fn new_kernel(
        start: *const u8,
        size: usize,
        permissions: mpu::Permissions,
    ) -> Option<PMPRegion> {
        // Determine access and execute permissions
        let pmpcfg = match permissions {
            mpu::Permissions::ReadWriteExecute => {
                // Not supported
                return None;
            }
            mpu::Permissions::ReadWriteOnly => {
                // Kernel can read/write, app can't access
                pmpcfg::l::SET + pmpcfg::r::SET + pmpcfg::w::SET + pmpcfg::x::CLEAR + pmpcfg::a::TOR
            }
            mpu::Permissions::ReadExecuteOnly => {
                // Kernel can read/execute, app can't access
                pmpcfg::l::SET + pmpcfg::r::SET + pmpcfg::w::CLEAR + pmpcfg::x::SET + pmpcfg::a::TOR
            }
            mpu::Permissions::ReadOnly => {
                // Kernel can read, app can't access
                pmpcfg::l::SET
                    + pmpcfg::r::SET
                    + pmpcfg::w::CLEAR
                    + pmpcfg::x::CLEAR
                    + pmpcfg::a::TOR
            }
            mpu::Permissions::ExecuteOnly => {
                // Kernel can execute, app can't access
                pmpcfg::l::SET
                    + pmpcfg::r::CLEAR
                    + pmpcfg::w::CLEAR
                    + pmpcfg::x::SET
                    + pmpcfg::a::TOR
            }
        };

        Some(PMPRegion {
            location: (start, size),
            cfg: pmpcfg,
        })
    }

    fn location(&self) -> (*const u8, usize) {
        self.location
    }

    /// Check if the PMP regions specified by `other_start` and `other_size`
    /// overlaps with the current region.
    /// Matching the RISC-V spec this checks pmpaddr[i-i] <= y < pmpaddr[i] for
    /// TOR ranges.
    fn overlaps(&self, other_start: *const u8, other_size: usize) -> bool {
        let other_start = other_start as usize;
        let other_end = other_start + other_size;

        let (region_start, region_size) = self.location;

        let (region_start, region_end) = {
            let region_start = region_start as usize;
            let region_end = region_start + region_size;
            (region_start, region_end)
        };

        if region_start == 0 && region_end == 0 {
            return false;
        }

        // PMP addresses are not inclusive on the high end, that is
        //     pmpaddr[i-i] <= y < pmpaddr[i]
        region_start < (other_end - 4) && other_start < (region_end - 4)
    }
}

/// Struct storing region configuration for RISCV PMP.
pub struct PMPConfig<ID, const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> {
    /// PMP config identifier, as generated by the issuing PMP implementation.
    id: ID,
    /// Array of PMP regions. Each region requires two physical entries.
    regions: [Option<PMPRegion>; MAX_AVAILABLE_REGIONS_OVER_TWO],
    /// Indicates if the configuration has changed since the last time it was
    /// written to hardware.
    is_dirty: Cell<bool>,
    /// Which region index is used for app memory (if it has been configured).
    app_memory_region: OptionalCell<usize>,
}

impl<ID, const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> fmt::Display
    for PMPConfig<ID, MAX_AVAILABLE_REGIONS_OVER_TWO>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " App ePMP regions:\r\n")?;
        for (n, region) in self.regions.iter().enumerate() {
            match region {
                None => write!(f, "  <unset>\r\n")?,
                Some(region) => write!(f, "  [{}]: {}\r\n", n, region)?,
            }
        }
        Ok(())
    }
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> PMPConfig<(), MAX_AVAILABLE_REGIONS_OVER_TWO> {
    /// Generate the default `PMPConfig` to be used when generating kernel regions.
    /// This should be called to generate a config before calling
    /// `allocate_kernel_region()` and `enable_kernel_mpu()`.
    /// This generally should only be called once, in `main()`.
    pub fn kernel_default() -> Self {
        let mut regions = [None; MAX_AVAILABLE_REGIONS_OVER_TWO];

        // This is a little challenging, so let's describe what's going on.
        // A previous boot stage has enabled Machine Mode Whitelist Policy
        // (mseccfg.MMWP), which we can't disable. That means that Tock will
        // be denied a memory access if it doesn't match a PMP region
        // This eventually won't matter, as Tock configures it's own regions,
        // but it makes it difficult to setup.
        //
        // The fact that we can run at all means that the previous stage has
        // set some rules that allows us to execute. When we configure our
        // rules we might overwrite them and lock ourselves out.
        //
        // To allow us to setup the ePMP regions without locking ourselves out
        // we create two special regions.
        //
        // We create an allow all region as the first and last region.
        // This way no matter what the previous stage did, we should have a
        // working fallback while we modify the rules.
        // If the previous stage has an allow all in the first or last region,
        // we won't get locked out. If the previous stage created a range of
        // regions we also won't get locked out.
        //
        // We make sure to remove these special regions later in
        // `enable_kernel_mpu()`
        if csr::CSR.mseccfg.is_set(csr::mseccfg::mseccfg::mmwp) {
            *(regions.last_mut().unwrap()) = Some(PMPRegion {
                // Set the size to zero so we don't get overlap errors latter
                location: (core::ptr::null::<u8>(), 0x00000000),
                cfg: pmpcfg::l::CLEAR
                    + pmpcfg::r::SET
                    + pmpcfg::w::SET
                    + pmpcfg::x::SET
                    + pmpcfg::a::TOR,
            });

            *(regions.first_mut().unwrap()) = Some(PMPRegion {
                // Set the size to zero so we don't get overlap errors latter
                location: (core::ptr::null::<u8>(), 0x00000000),
                cfg: pmpcfg::l::CLEAR
                    + pmpcfg::r::SET
                    + pmpcfg::w::SET
                    + pmpcfg::x::SET
                    + pmpcfg::a::TOR,
            });
        }

        PMPConfig {
            id: (),
            regions,
            is_dirty: Cell::new(true),
            app_memory_region: OptionalCell::empty(),
        }
    }
}

impl<ID, const MAX_AVAILABLE_REGIONS_OVER_TWO: usize>
    PMPConfig<ID, MAX_AVAILABLE_REGIONS_OVER_TWO>
{
    /// Get the first unused region
    fn unused_region_number(&self, locked_region_mask: u64) -> Option<usize> {
        for (number, region) in self.regions.iter().enumerate() {
            if !self.is_index_locked_or_app(locked_region_mask, number) && region.is_none() {
                return Some(number);
            }
        }
        None
    }

    /// Get the last unused region
    /// The app regions need to be lower then the kernel to ensure they
    /// match before the kernel ones.
    fn unused_kernel_region_number(&self, locked_region_mask: u64) -> Option<usize> {
        for (num, region) in self.regions.iter().rev().enumerate() {
            let number = MAX_AVAILABLE_REGIONS_OVER_TWO - num - 1;
            if !self.is_index_locked_or_app(locked_region_mask, number) && region.is_none() {
                return Some(number);
            }
        }
        None
    }

    /// Returns true is the specified index is either locked or corresponds to the app region
    fn is_index_locked_or_app(&self, locked_region_mask: u64, number: usize) -> bool {
        locked_region_mask & (1 << number) > 0 || self.app_memory_region.contains(&number)
    }
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> kernel::platform::mpu::MPU
    for PMP<MAX_AVAILABLE_REGIONS_OVER_TWO>
{
    type MpuConfig = PMPConfig<NonZeroUsize, MAX_AVAILABLE_REGIONS_OVER_TWO>;

    fn clear_mpu(&self) {
        // We want to disable all of the hardware entries, so we use `NUM_REGIONS` here,
        // and not `NUM_REGIONS / 2`.
        //
        // We want to keep the first region configured, so it is excluded from the loops and
        // set separately.
        for x in 1..(MAX_AVAILABLE_REGIONS_OVER_TWO * 2) {
            csr::CSR.pmpaddr_set(x, 0x0);
        }
        for x in 1..(MAX_AVAILABLE_REGIONS_OVER_TWO * 2 / 4) {
            csr::CSR.pmpconfig_set(x, 0);
        }
        csr::CSR.pmpaddr_set(0, 0xFFFF_FFFF);
        // enable R W X fields
        csr::CSR.pmpconfig_set(
            0,
            (csr::pmpconfig::pmpcfg::r0::SET
                + csr::pmpconfig::pmpcfg::w0::SET
                + csr::pmpconfig::pmpcfg::x0::SET
                + csr::pmpconfig::pmpcfg::a0::TOR)
                .value,
        );
        // PMP is not configured for any process now
        self.last_configured_for.take();
    }

    fn enable_app_mpu(&self) {}

    fn disable_app_mpu(&self) {
        for i in 0..self.number_total_regions() {
            if self.locked_region_mask.get() & (1 << i) > 0 {
                continue;
            }
            match i % 2 {
                0 => {
                    csr::CSR.pmpconfig_modify(i / 2, csr::pmpconfig::pmpcfg::a1::OFF);
                }
                1 => {
                    csr::CSR.pmpconfig_modify(i / 2, csr::pmpconfig::pmpcfg::a3::OFF);
                }
                _ => break,
            };
        }
    }

    fn number_total_regions(&self) -> usize {
        self.num_regions / 2
    }

    fn new_config(&self) -> Option<Self::MpuConfig> {
        let id = self.config_count.get();
        self.config_count.set(id.checked_add(1)?);

        Some(PMPConfig {
            id,
            regions: [None; MAX_AVAILABLE_REGIONS_OVER_TWO],
            is_dirty: Cell::new(true),
            app_memory_region: OptionalCell::empty(),
        })
    }

    fn reset_config(&self, config: &mut Self::MpuConfig) {
        config.regions.iter_mut().for_each(|region| *region = None);
        config.app_memory_region.clear();
        config.is_dirty.set(true);
    }

    fn allocate_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<mpu::Region> {
        for region in config.regions.iter() {
            if region.is_some() {
                if region
                    .unwrap()
                    .overlaps(unallocated_memory_start, unallocated_memory_size)
                {
                    return None;
                }
            }
        }

        let region_num = config.unused_region_number(self.locked_region_mask.get())?;

        // Logical region
        let mut start = unallocated_memory_start as usize;
        let mut size = min_region_size;

        // Region start always has to align to 4 bytes
        if start % 4 != 0 {
            start += 4 - (start % 4);
        }

        // Region size always has to align to 4 bytes
        if size % 4 != 0 {
            size += 4 - (size % 4);
        }

        // Regions must be at least 8 bytes
        if size < 8 {
            size = 8;
        }

        let region = PMPRegion::new_app(start as *const u8, size, permissions);

        region?;

        config.regions[region_num] = region;
        config.is_dirty.set(true);

        Some(mpu::Region::new(start as *const u8, size))
    }

    fn remove_memory_region(
        &self,
        region: mpu::Region,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        let (index, _r) = config
            .regions
            .iter()
            .enumerate()
            .find(|(_idx, r)| r.map_or(false, |r| r == region))
            .ok_or(())?;

        if config.is_index_locked_or_app(self.locked_region_mask.get(), index) {
            return Err(());
        }

        config.regions[index] = None;
        config.is_dirty.set(true);

        Ok(())
    }

    fn allocate_app_memory_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        // Check that no previously allocated regions overlap the unallocated memory.
        for region in config.regions.iter() {
            if region.is_some() {
                if region
                    .unwrap()
                    .overlaps(unallocated_memory_start, unallocated_memory_size)
                {
                    return None;
                }
            }
        }

        let region_num = if config.app_memory_region.is_some() {
            config.app_memory_region.unwrap_or(0)
        } else {
            config.unused_region_number(self.locked_region_mask.get())?
        };

        // App memory size is what we actual set the region to. So this region
        // has to be aligned to 4 bytes.
        let mut initial_app_memory_size: usize = initial_app_memory_size;
        if initial_app_memory_size % 4 != 0 {
            initial_app_memory_size += 4 - (initial_app_memory_size % 4);
        }

        // Make sure there is enough memory for app memory and kernel memory.
        let mut region_size = cmp::max(
            min_memory_size,
            initial_app_memory_size + initial_kernel_memory_size,
        ) as usize;

        // Region size always has to align to 4 bytes
        if region_size % 4 != 0 {
            region_size += 4 - (region_size % 4);
        }

        // The region should start as close as possible to the start of the unallocated memory.
        let region_start = unallocated_memory_start as usize;

        // Make sure the region fits in the unallocated memory.
        if region_start + region_size
            > (unallocated_memory_start as usize) + unallocated_memory_size
        {
            return None;
        }

        let region = PMPRegion::new_app(
            region_start as *const u8,
            initial_app_memory_size,
            permissions,
        );

        region?;

        config.regions[region_num] = region;

        config.app_memory_region.set(region_num);
        config.is_dirty.set(true);

        Some((region_start as *const u8, region_size))
    }

    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        let region_num = config.app_memory_region.unwrap_or(0);

        let (region_start, _) = match config.regions[region_num] {
            Some(region) => region.location(),
            None => {
                // Error: Process tried to update app memory MPU region before it was created.
                return Err(());
            }
        };

        let app_memory_break = app_memory_break as usize;
        let kernel_memory_break = kernel_memory_break as usize;

        // Out of memory
        if app_memory_break > kernel_memory_break {
            return Err(());
        }

        // Get size of updated region
        let region_size = app_memory_break - region_start as usize;

        let region = PMPRegion::new_app(region_start, region_size, permissions);

        if region.is_none() {
            return Err(());
        }

        config.regions[region_num] = region;
        config.is_dirty.set(true);

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig) {
        // Is the PMP already configured for this app?
        let last_configured_for_this_app = self
            .last_configured_for
            .map_or(false, |last_id| last_id == config.id);

        if !last_configured_for_this_app || config.is_dirty.get() {
            for (x, region) in config.regions.iter().enumerate() {
                match region {
                    Some(r) => {
                        let cfg_val = r.cfg.value as usize;
                        let start = r.location.0 as usize;
                        let size = r.location.1;

                        let disable_val = (csr::pmpconfig::pmpcfg::r0::CLEAR
                            + csr::pmpconfig::pmpcfg::w0::CLEAR
                            + csr::pmpconfig::pmpcfg::x0::CLEAR
                            + csr::pmpconfig::pmpcfg::a0::CLEAR)
                            .value;
                        let (region_shift, other_region_mask) = if x % 2 == 0 {
                            (0, 0xFFFF_0000)
                        } else {
                            (16, 0x0000_FFFF)
                        };
                        // Fully clear the PMP config
                        csr::CSR.pmpconfig_set(
                            x / 2,
                            (disable_val << region_shift)
                                | (csr::CSR.pmpconfig_get(x / 2) & other_region_mask),
                        );

                        // Set the address *before* we enable the config
                        // Otherwise this could take effect and block the kernel from running
                        csr::CSR.pmpaddr_set(x * 2, (start) >> 2);
                        csr::CSR.pmpaddr_set((x * 2) + 1, (start + size) >> 2);

                        // Enable the configs
                        csr::CSR.pmpconfig_set(
                            x / 2,
                            (cfg_val << 8) << region_shift | (csr::CSR.pmpconfig_get(x / 2)),
                        );
                    }
                    None => {
                        // Invalidate other regions not used in this PMPConfig.
                        match x % 2 {
                            0 => {
                                csr::CSR.pmpconfig_modify(x / 2, csr::pmpconfig::pmpcfg::a1::OFF);
                            }
                            1 => {
                                csr::CSR.pmpconfig_modify(x / 2, csr::pmpconfig::pmpcfg::a3::OFF);
                            }
                            // unreachable, but don't insert a panic
                            _ => (),
                        };
                    }
                };
            }
        } else {
            // We were last configured for this app, just re-enable
            for (x, region) in config.regions.iter().enumerate() {
                match region {
                    Some(_r) => {
                        match x % 2 {
                            0 => {
                                csr::CSR.pmpconfig_modify(x / 2, csr::pmpconfig::pmpcfg::a1::TOR);
                            }
                            1 => {
                                csr::CSR.pmpconfig_modify(x / 2, csr::pmpconfig::pmpcfg::a3::TOR);
                            }
                            // unreachable, but don't insert a panic
                            _ => (),
                        };
                    }
                    None => {}
                };
            }
        }

        config.is_dirty.set(false);
        self.last_configured_for.replace(config.id);
    }
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> PMP<MAX_AVAILABLE_REGIONS_OVER_TWO> {
    fn write_kernel_regions(&self, config: &PMPConfig<(), MAX_AVAILABLE_REGIONS_OVER_TWO>) {
        for (i, region) in config.regions.iter().enumerate() {
            match region {
                Some(r) => {
                    let cfg_val = r.cfg.value as usize;
                    let start = r.location.0 as usize;
                    let size = r.location.1;

                    match i % 2 {
                        0 => {
                            csr::CSR.pmpaddr_set((i * 2) + 1, (start + size) >> 2);
                            // Disable access up to the start address
                            csr::CSR.pmpconfig_modify(
                                i / 2,
                                csr::pmpconfig::pmpcfg::r0::CLEAR
                                    + csr::pmpconfig::pmpcfg::w0::CLEAR
                                    + csr::pmpconfig::pmpcfg::x0::CLEAR
                                    + csr::pmpconfig::pmpcfg::a0::CLEAR,
                            );
                            csr::CSR.pmpaddr_set(i * 2, start >> 2);

                            // Set access to end address
                            let new_cfg =
                                cfg_val << 8 | (csr::CSR.pmpconfig_get(i / 2) & 0xFFFF_00FF);
                            csr::CSR.pmpconfig_set(i / 2, new_cfg);
                        }
                        1 => {
                            csr::CSR.pmpaddr_set((i * 2) + 1, (start + size) >> 2);
                            // Disable access up to the start address
                            csr::CSR.pmpconfig_modify(
                                i / 2,
                                csr::pmpconfig::pmpcfg::r2::CLEAR
                                    + csr::pmpconfig::pmpcfg::w2::CLEAR
                                    + csr::pmpconfig::pmpcfg::x2::CLEAR
                                    + csr::pmpconfig::pmpcfg::a2::CLEAR,
                            );
                            csr::CSR.pmpaddr_set(i * 2, start >> 2);

                            // Set access to end address
                            let new_cfg =
                                cfg_val << 24 | (csr::CSR.pmpconfig_get(i / 2) & 0x00FF_FFFF);
                            csr::CSR.pmpconfig_set(i / 2, new_cfg);
                        }
                        _ => break,
                    }
                }
                None => match i % 2 {
                    0 => {
                        csr::CSR.pmpaddr_set(i * 2, 0);
                        csr::CSR.pmpaddr_set((i * 2) + 1, 0);

                        let new_cfg = 0 << 8 | (csr::CSR.pmpconfig_get(i / 2) & 0xFFFF_00FF);
                        csr::CSR.pmpconfig_set(i / 2, new_cfg);
                    }
                    1 => {
                        csr::CSR.pmpaddr_set(i * 2, 0);
                        csr::CSR.pmpaddr_set((i * 2) + 1, 0);

                        let new_cfg = 0 << 24 | (csr::CSR.pmpconfig_get(i / 2) & 0x00FF_FFFF);
                        csr::CSR.pmpconfig_set(i / 2, new_cfg);
                    }
                    _ => break,
                },
            };
        }
    }
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> kernel::platform::mpu::KernelMPU
    for PMP<MAX_AVAILABLE_REGIONS_OVER_TWO>
{
    type KernelMpuConfig = PMPConfig<(), MAX_AVAILABLE_REGIONS_OVER_TWO>;

    fn new_kernel_config(&self) -> Option<Self::KernelMpuConfig> {
        Some(PMPConfig {
            id: (),
            regions: [None; MAX_AVAILABLE_REGIONS_OVER_TWO],
            is_dirty: Cell::new(true),
            app_memory_region: OptionalCell::empty(),
        })
    }

    fn allocate_kernel_region(
        &self,
        memory_start: *const u8,
        memory_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::KernelMpuConfig,
    ) -> Option<mpu::Region> {
        for region in config.regions.iter() {
            if region.is_some() {
                if region.unwrap().overlaps(memory_start, memory_size) {
                    return None;
                }
            }
        }

        let region_num = config.unused_kernel_region_number(self.locked_region_mask.get())?;

        // Logical region
        let mut start = memory_start as usize;
        let mut size = memory_size;

        // Region start always has to align to 4 bytes
        if start % 4 != 0 {
            start += 4 - (start % 4);
        }

        // Region size always has to align to 4 bytes
        if size % 4 != 0 {
            size += 4 - (size % 4);
        }

        // Regions must be at least 8 bytes
        if size < 8 {
            size = 8;
        }

        let region = PMPRegion::new_kernel(start as *const u8, size, permissions);

        region?;

        config.regions[region_num] = region;

        // Mark the region as locked so that the app PMP doesn't use it.
        let mut mask = self.locked_region_mask.get();
        mask |= 1 << region_num;
        self.locked_region_mask.set(mask);

        Some(mpu::Region::new(start as *const u8, size))
    }

    fn enable_kernel_mpu(&self, config: &mut Self::KernelMpuConfig) {
        if csr::CSR.mseccfg.is_set(csr::mseccfg::mseccfg::mmwp) {
            // MMWP is set, so let's edit the size of the last and first
            // regions to allow all
            if let Some(last) = config.regions.last_mut() {
                if let Some(last_region) = last {
                    last_region.location = (core::ptr::null::<u8>(), 0xFFFF_FFFF);
                }
            }

            if let Some(first) = config.regions.first_mut() {
                if let Some(first_region) = first {
                    first_region.location = (core::ptr::null::<u8>(), 0xFFFF_FFFF);
                }
            }
        }

        self.write_kernel_regions(config);

        if csr::CSR.mseccfg.is_set(csr::mseccfg::mseccfg::mmwp) {
            // Now that we have written an initial copy, we can remove our
            // allow all regions. We want to do this one at a time though.

            // Remove the last region and rotate the entries
            // This maintains the first allow all region, so that we
            // don't get locked out while writing the data.
            if let Some(last) = config.regions.last_mut() {
                *last = None;
            }
            config.regions.rotate_right(1);
            self.write_kernel_regions(config);

            // Now we can remove the first region (which is now the second)
            if let Some(first) = config.regions.get_mut(1) {
                *first = None;
            }
            self.write_kernel_regions(config);

            // At this point we have configured the ePMP, we can disable debug
            // access (if it was enabled)
            csr::CSR.mseccfg.modify(csr::mseccfg::mseccfg::rlb::CLEAR);
        }

        // Set the Machine Mode Lockdown (mseccfg.MML) bit.
        // This is a sticky bit, meaning that once set it cannot be unset
        // until a hard reset.
        csr::CSR.mseccfg.modify(csr::mseccfg::mseccfg::mml::SET);
    }
}
