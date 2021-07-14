//! Implementation of the physical memory protection unit (PMP).
//!
//! ## Implementation
//!
//! We use the PMP Top of Region (TOR) alignment as there are alignment issues
//! with NAPOT. NAPOT would allow us to protect more memory regions (with NAPOT
//! each PMP region can be a memory region), but the problem with NAPOT is the
//! address must be aligned to the size, which results in wasted memory. To
//! avoid this wasted memory we use TOR and each memory region uses two physical
//! PMP regions.

use core::cell::Cell;
use core::cmp;
use core::fmt;
use kernel::utilities::cells::OptionalCell;

use crate::csr;
use kernel::platform::mpu;
use kernel::utilities::cells::MapCell;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{self, register_bitfields};
use kernel::ProcessId;

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
///  there are more than 0 implemented. So this value should be either 8 or 32.
///
///  If however you know the exact number of PMP regions implemented by your
///  platform and it's not going to change you can just specify the number.
///  This means that Tock won't be able to dynamically handle more regions,
///  but it will reduce runtime space requirements.
///  Note: that this does not mean all PMP regions are connected.
///  Some of the regions can be WARL (Write Any Read Legal). All this means
///  is that accessing `NUM_REGIONS` won't cause a fault.
pub struct PMP<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> {
    /// The application that the MPU was last configured for. Used (along with
    /// the `is_dirty` flag) to determine if MPU can skip writing the
    /// configuration to hardware.
    last_configured_for: MapCell<ProcessId>,
    /// This is a 64-bit mask of locked regions.
    /// Each bit that is set in this mask indicates that the region is locked
    /// and cannot be used by Tock.
    locked_region_mask: Cell<u64>,
    /// This is the total number of avaliable regions.
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

        for i in 0..(MAX_AVAILABLE_REGIONS_OVER_TWO * 2) {
            // Read the current value
            let pmpcfg_og = csr::CSR.pmpconfig_get(i / 4);

            // Flip R, W, X bits
            let pmpcfg_new = pmpcfg_og ^ (3 << ((i % 4) * 8));
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

        Self {
            last_configured_for: MapCell::empty(),
            num_regions,
            locked_region_mask: Cell::new(locked_region_mask),
        }
    }
}

/// Struct storing configuration for a RISC-V PMP region.
#[derive(Copy, Clone)]
pub struct PMPRegion {
    location: (*const u8, usize),
    cfg: registers::FieldValue<u8, pmpcfg::Register>,
}

impl fmt::Display for PMPRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn bit_str<'a>(reg: &PMPRegion, bit: u8, on_str: &'a str, off_str: &'a str) -> &'a str {
            match reg.cfg.value & bit as u8 {
                0 => off_str,
                _ => on_str,
            }
        }

        write!(
            f,
            "addr={:p}, size={:#010X}, cfg={:#X} ({}{}{})",
            self.location.0,
            self.location.1,
            u8::from(self.cfg),
            bit_str(self, pmpcfg::r::SET.value, "r", "-"),
            bit_str(self, pmpcfg::w::SET.value, "w", "-"),
            bit_str(self, pmpcfg::x::SET.value, "x", "-"),
        )
    }
}

impl PMPRegion {
    fn new(start: *const u8, size: usize, permissions: mpu::Permissions) -> PMPRegion {
        // Determine access and execute permissions
        let pmpcfg = match permissions {
            mpu::Permissions::ReadWriteExecute => {
                pmpcfg::r::SET + pmpcfg::w::SET + pmpcfg::x::SET + pmpcfg::a::TOR
            }
            mpu::Permissions::ReadWriteOnly => {
                pmpcfg::r::SET + pmpcfg::w::SET + pmpcfg::x::CLEAR + pmpcfg::a::TOR
            }
            mpu::Permissions::ReadExecuteOnly => {
                pmpcfg::r::SET + pmpcfg::w::CLEAR + pmpcfg::x::SET + pmpcfg::a::TOR
            }
            mpu::Permissions::ReadOnly => {
                pmpcfg::r::SET + pmpcfg::w::CLEAR + pmpcfg::x::CLEAR + pmpcfg::a::TOR
            }
            mpu::Permissions::ExecuteOnly => {
                pmpcfg::r::CLEAR + pmpcfg::w::CLEAR + pmpcfg::x::SET + pmpcfg::a::TOR
            }
        };

        PMPRegion {
            location: (start, size),
            cfg: pmpcfg,
        }
    }

    fn location(&self) -> (*const u8, usize) {
        self.location
    }

    fn overlaps(&self, other_start: *const u8, other_size: usize) -> bool {
        let other_start = other_start as usize;
        let other_end = other_start + other_size;

        let (region_start, region_size) = self.location;

        let (region_start, region_end) = {
            let region_start = region_start as usize;
            let region_end = region_start + region_size;
            (region_start, region_end)
        };

        if region_start < other_end && other_start < region_end {
            true
        } else {
            false
        }
    }
}

/// Struct storing region configuration for RISCV PMP.
pub struct PMPConfig<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> {
    /// Array of PMP regions. Each region requires two physical entries.
    regions: [Option<PMPRegion>; MAX_AVAILABLE_REGIONS_OVER_TWO],
    /// Indicates if the configuration has changed since the last time it was
    /// written to hardware.
    is_dirty: Cell<bool>,
    /// Which region index is used for app memory (if it has been configured).
    app_memory_region: OptionalCell<usize>,
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> Default
    for PMPConfig<MAX_AVAILABLE_REGIONS_OVER_TWO>
{
    /// `NUM_REGIONS` is the number of PMP entries the hardware supports.
    ///
    /// Since we use TOR, we will use two PMP entries for each region. So the actual
    /// number of regions we can protect is `NUM_REGIONS/2`. Limitations of min_const_generics
    /// require us to pass both of these values as separate generic consts.
    fn default() -> Self {
        PMPConfig {
            regions: [None; MAX_AVAILABLE_REGIONS_OVER_TWO],
            is_dirty: Cell::new(true),
            app_memory_region: OptionalCell::empty(),
        }
    }
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> fmt::Display
    for PMPConfig<MAX_AVAILABLE_REGIONS_OVER_TWO>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " PMP regions:\r\n")?;
        for (n, region) in self.regions.iter().enumerate() {
            match region {
                None => write!(f, "  <unset>\r\n")?,
                Some(region) => write!(f, "  [{}]: {}\r\n", n, region)?,
            }
        }
        Ok(())
    }
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> PMPConfig<MAX_AVAILABLE_REGIONS_OVER_TWO> {
    /// Get the first unused region
    fn unused_region_number(&self, locked_region_mask: u64) -> Option<usize> {
        for (number, region) in self.regions.iter().enumerate() {
            if self.app_memory_region.contains(&number) {
                continue;
            }
            // This region exists, but is locked
            if locked_region_mask & (1 << number) > 0 {
                continue;
            }
            if region.is_none() {
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
            if self.app_memory_region.contains(&number) {
                continue;
            }
            // This region exists, but is locked
            if locked_region_mask & (1 << number) > 0 {
                continue;
            }
            if region.is_none() {
                return Some(number);
            }
        }
        None
    }
}

impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> kernel::platform::mpu::MPU
    for PMP<MAX_AVAILABLE_REGIONS_OVER_TWO>
{
    type MpuConfig = PMPConfig<MAX_AVAILABLE_REGIONS_OVER_TWO>;

    fn clear_mpu(&self) {
        // We want to disable all of the hardware entries, so we use `NUM_REGIONS` here,
        // and not `NUM_REGIONS / 2`.
        for x in 0..(MAX_AVAILABLE_REGIONS_OVER_TWO * 2) {
            match x % 4 {
                0 => {
                    csr::CSR.pmpconfig_modify(
                        x / 4,
                        csr::pmpconfig::pmpcfg::r0::CLEAR
                            + csr::pmpconfig::pmpcfg::w0::CLEAR
                            + csr::pmpconfig::pmpcfg::x0::CLEAR
                            + csr::pmpconfig::pmpcfg::a0::OFF
                            + csr::pmpconfig::pmpcfg::l0::CLEAR,
                    );
                }
                1 => {
                    csr::CSR.pmpconfig_modify(
                        x / 4,
                        csr::pmpconfig::pmpcfg::r1::CLEAR
                            + csr::pmpconfig::pmpcfg::w1::CLEAR
                            + csr::pmpconfig::pmpcfg::x1::CLEAR
                            + csr::pmpconfig::pmpcfg::a1::OFF
                            + csr::pmpconfig::pmpcfg::l1::CLEAR,
                    );
                }
                2 => {
                    csr::CSR.pmpconfig_modify(
                        x / 4,
                        csr::pmpconfig::pmpcfg::r2::CLEAR
                            + csr::pmpconfig::pmpcfg::w2::CLEAR
                            + csr::pmpconfig::pmpcfg::x2::CLEAR
                            + csr::pmpconfig::pmpcfg::a2::OFF
                            + csr::pmpconfig::pmpcfg::l2::CLEAR,
                    );
                }
                3 => {
                    csr::CSR.pmpconfig_modify(
                        x / 4,
                        csr::pmpconfig::pmpcfg::r3::CLEAR
                            + csr::pmpconfig::pmpcfg::w3::CLEAR
                            + csr::pmpconfig::pmpcfg::x3::CLEAR
                            + csr::pmpconfig::pmpcfg::a3::OFF
                            + csr::pmpconfig::pmpcfg::l3::CLEAR,
                    );
                }
                _ => unreachable!(),
            }
            csr::CSR.pmpaddr_set(x, 0x0);
        }

        //set first PMP to have permissions to entire space
        csr::CSR.pmpaddr0.set(0xFFFF_FFFF);
        //enable R W X fields
        csr::CSR.pmpconfig_modify(0, csr::pmpconfig::pmpcfg::r0::SET);
        csr::CSR.pmpconfig_modify(0, csr::pmpconfig::pmpcfg::w0::SET);
        csr::CSR.pmpconfig_modify(0, csr::pmpconfig::pmpcfg::x0::SET);
        csr::CSR.pmpconfig_modify(0, csr::pmpconfig::pmpcfg::a0::TOR);
        // PMP is not configured for any process now
        self.last_configured_for.take();
    }

    fn enable_app_mpu(&self) {}

    fn disable_app_mpu(&self) {
        // PMP is not enabled for machine mode, so we don't have to do
        // anything
    }

    fn number_total_regions(&self) -> usize {
        self.num_regions / 2
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

        let region = PMPRegion::new(start as *const u8, size, permissions);

        config.regions[region_num] = Some(region);
        config.is_dirty.set(true);

        Some(mpu::Region::new(start as *const u8, size))
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

        let region = PMPRegion::new(
            region_start as *const u8,
            initial_app_memory_size,
            permissions,
        );

        config.regions[region_num] = Some(region);
        config.is_dirty.set(true);

        config.app_memory_region.set(region_num);

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

        let region = PMPRegion::new(region_start as *const u8, region_size, permissions);

        config.regions[region_num] = Some(region);
        config.is_dirty.set(true);

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig, app_id: &ProcessId) {
        // Is the PMP already configured for this app?
        let last_configured_for_this_app = self
            .last_configured_for
            .map_or(false, |last_app_id| last_app_id == app_id);

        // Skip PMP configuration if it is already configured for this app and the MPU
        // configuration of this app has not changed.
        if !last_configured_for_this_app || config.is_dirty.get() {
            for (x, region) in config.regions.iter().enumerate() {
                match region {
                    Some(r) => {
                        let cfg_val = r.cfg.value as usize;
                        let start = r.location.0 as usize;
                        let size = r.location.1;

                        match x % 2 {
                            0 => {
                                // Disable access up to the start address
                                csr::CSR.pmpconfig_modify(
                                    x / 2,
                                    csr::pmpconfig::pmpcfg::r0::CLEAR
                                        + csr::pmpconfig::pmpcfg::w0::CLEAR
                                        + csr::pmpconfig::pmpcfg::x0::CLEAR
                                        + csr::pmpconfig::pmpcfg::a0::OFF,
                                );
                                csr::CSR.pmpaddr_set(x * 2, start >> 2);

                                // Set access to end address
                                csr::CSR.pmpaddr_set((x * 2) + 1, (start + size) >> 2);
                                csr::CSR.pmpconfig_set(
                                    x / 2,
                                    cfg_val << 8 | csr::CSR.pmpconfig_get(x / 2),
                                );
                            }
                            1 => {
                                // Disable access up to the start address
                                csr::CSR.pmpconfig_modify(
                                    x / 2,
                                    csr::pmpconfig::pmpcfg::r2::CLEAR
                                        + csr::pmpconfig::pmpcfg::w2::CLEAR
                                        + csr::pmpconfig::pmpcfg::x2::CLEAR
                                        + csr::pmpconfig::pmpcfg::a2::OFF,
                                );
                                csr::CSR.pmpaddr_set(x * 2, start >> 2);

                                // Set access to end address
                                csr::CSR.pmpaddr_set((x * 2) + 1, (start + size) >> 2);
                                csr::CSR.pmpconfig_set(
                                    x / 2,
                                    cfg_val << 24 | csr::CSR.pmpconfig_get(x / 2),
                                );
                            }
                            _ => break,
                        }
                    }
                    None => {}
                };
            }
            config.is_dirty.set(false);
            self.last_configured_for.put(*app_id);
        }
    }
}

/// This is PMP support for kernel regions
/// PMP does not allow a deny by default option, so all regions not marked
/// with the below commands will have full access.
/// This is still a useful implementation as it can be used to limit the
/// kernels access, for example removing execute permission from regions
/// we don't need to execute from and removing write permissions from
/// executable reions.
impl<const MAX_AVAILABLE_REGIONS_OVER_TWO: usize> kernel::platform::mpu::KernelMPU
    for PMP<MAX_AVAILABLE_REGIONS_OVER_TWO>
{
    type KernelMpuConfig = PMPConfig<MAX_AVAILABLE_REGIONS_OVER_TWO>;

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

        let region = PMPRegion::new(start as *const u8, size, permissions);

        config.regions[region_num] = Some(region);

        // Mark the region as locked so that the app PMP doesn't use it.
        let mut mask = self.locked_region_mask.get();
        mask |= 1 << region_num;
        self.locked_region_mask.set(mask);

        Some(mpu::Region::new(start as *const u8, size))
    }

    fn enable_kernel_mpu(&self, config: &mut Self::KernelMpuConfig) {
        for (i, region) in config.regions.iter().rev().enumerate() {
            let x = MAX_AVAILABLE_REGIONS_OVER_TWO - i - 1;
            match region {
                Some(r) => {
                    let cfg_val = r.cfg.value as usize;
                    let start = r.location.0 as usize;
                    let size = r.location.1;

                    match x % 2 {
                        0 => {
                            csr::CSR.pmpaddr_set((x * 2) + 1, (start + size) >> 2);
                            // Disable access up to the start address
                            csr::CSR.pmpconfig_modify(
                                x / 2,
                                csr::pmpconfig::pmpcfg::r0::CLEAR
                                    + csr::pmpconfig::pmpcfg::w0::CLEAR
                                    + csr::pmpconfig::pmpcfg::x0::CLEAR
                                    + csr::pmpconfig::pmpcfg::a0::CLEAR,
                            );
                            csr::CSR.pmpaddr_set(x * 2, start >> 2);

                            // Set access to end address
                            csr::CSR
                                .pmpconfig_set(x / 2, cfg_val << 8 | csr::CSR.pmpconfig_get(x / 2));
                            // Lock the CSR
                            csr::CSR.pmpconfig_modify(x / 2, csr::pmpconfig::pmpcfg::l1::SET);
                        }
                        1 => {
                            csr::CSR.pmpaddr_set((x * 2) + 1, (start + size) >> 2);
                            // Disable access up to the start address
                            csr::CSR.pmpconfig_modify(
                                x / 2,
                                csr::pmpconfig::pmpcfg::r2::CLEAR
                                    + csr::pmpconfig::pmpcfg::w2::CLEAR
                                    + csr::pmpconfig::pmpcfg::x2::CLEAR
                                    + csr::pmpconfig::pmpcfg::a2::CLEAR,
                            );
                            csr::CSR.pmpaddr_set(x * 2, start >> 2);

                            // Set access to end address
                            csr::CSR.pmpconfig_set(
                                x / 2,
                                cfg_val << 24 | csr::CSR.pmpconfig_get(x / 2),
                            );
                            // Lock the CSR
                            csr::CSR.pmpconfig_modify(x / 2, csr::pmpconfig::pmpcfg::l3::SET);
                        }
                        _ => break,
                    }
                }
                None => {}
            };
        }
    }
}
