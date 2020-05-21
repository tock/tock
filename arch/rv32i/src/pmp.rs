//! Implementation of the physical memory protection unit (PMP).
//!
//! Different PMP implementations support different numbers of PMP entries. To
//! reduce memory overhead, the PMP implementation supports customizing the
//! number of supported entries to match the hardware. Ideally we would do this
//! by templating the implementation on the number of entries, but that is an
//! experimental feature in Rust. To mimic that functionality, we wrap this
//! entire PMP implementation in a macro, and then require that each chip with
//! an PMP instantiate their own PMP implementation with the size customized for
//! the chip's hardware.
//!
//! ## Implementation
//!
//! We use the PMP Top of Region (TOR) alignment as there are alignment issues
//! with NAPOT. NAPOT would allow us to protect more memory regions (with NAPOT
//! each PMP region can be a memory region), but the problem with NAPOT is the
//! address must be aligned to the size, which results in wasted memory. To
//! avoid this wasted memory we use TOR and each memory region uses two physical
//! PMP regions.

/// Instantiate a PMP configuration.
///
/// `$x` is the number of PMP entries the hardware supports.
///
/// Since we use TOR, we will use two PMP entries for each region. So the actual
/// number of regions we can protect is `$x/2`.
#[macro_export]
macro_rules! PMPConfigMacro {
    ( $x:expr ) => {

use core::cell::Cell;
use core::cmp;
use core::fmt;
use kernel::common::cells::OptionalCell;

use rv32i::csr;
use kernel::common::cells::MapCell;
use kernel::common::registers;
use kernel::common::registers::register_bitfields;
use kernel::mpu;
use kernel::AppId;

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
pub struct PMP {
    /// The application that the MPU was last configured for. Used (along with
    /// the `is_dirty` flag) to determine if MPU can skip writing the
    /// configuration to hardware.
    last_configured_for: MapCell<AppId>,
}

impl PMP {
    pub const unsafe fn new() -> PMP {
        PMP {
            last_configured_for: MapCell::empty(),
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
            "addr={:p}, size={:#X}, cfg={:#X} ({}{}{})",
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
pub struct PMPConfig {
    /// Array of PMP regions. Each region requires two physical entries.
    regions: [Option<PMPRegion>; $x / 2],
    /// Indicates if the configuration has changed since the last time it was
    /// written to hardware.
    is_dirty: Cell<bool>,
    /// Which region index is used for app memory (if it has been configured).
    app_memory_region: OptionalCell<usize>,
}

impl Default for PMPConfig {
    fn default() -> Self {
        PMPConfig {
            regions: [None; $x / 2],
            is_dirty: Cell::new(true),
            app_memory_region: OptionalCell::empty(),
        }
    }
}

impl fmt::Display for PMPConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "PMP regions:")?;
        for (n, region) in self.regions.iter().enumerate() {
            match region {
                None => writeln!(f, "<unset>")?,
                Some(region) => writeln!(f, " [{}]: {}", n, region)?,
            }
        }
        Ok(())
    }
}

impl PMPConfig {
    fn unused_region_number(&self) -> Option<usize> {
        for (number, region) in self.regions.iter().enumerate() {
            if self.app_memory_region.contains(&number) {
                continue;
            }
            if region.is_none() {
                return Some(number);
            }
        }
        None
    }

    fn sort_regions(&mut self) {
        // Get the app region address
        let app_addres = if self.app_memory_region.is_some() {
            Some(
                self.regions[self.app_memory_region.unwrap_or(0)]
                    .unwrap()
                    .location
                    .0,
            )
        } else {
            None
        };

        // Sort the regions
        self.regions.sort_unstable_by(|a, b| {
            let (a_start, _a_size) = match a {
                Some(region) => (region.location().0 as usize, region.location().1),
                None => (0xFFFF_FFFF, 0xFFFF_FFFF),
            };
            let (b_start, _b_size) = match b {
                Some(region) => (region.location().0 as usize, region.location().1),
                None => (0xFFFF_FFFF, 0xFFFF_FFFF),
            };
            a_start.cmp(&b_start)
        });

        // Update the app region after the sort
        if app_addres.is_some() {
            for (i, region) in self.regions.iter().enumerate() {
                match region {
                    Some(reg) => {
                        if reg.location.0 == app_addres.unwrap() {
                            self.app_memory_region.set(i);
                        }
                    }
                    None => {}
                }
            }
        }
    }
}

impl kernel::mpu::MPU for PMP {
    type MpuConfig = PMPConfig;

    fn clear_mpu(&self) {
        // We want to disable all of the hardware entries, so we use `$x` here,
        // and not `$x / 2`.
        for x in 0..$x {
            match x % 4 {
                0 => {
                    csr::CSR.pmpcfg[x / 4].modify(
                        csr::pmpconfig::pmpcfg::r0::CLEAR
                            + csr::pmpconfig::pmpcfg::w0::CLEAR
                            + csr::pmpconfig::pmpcfg::x0::CLEAR
                            + csr::pmpconfig::pmpcfg::a0::OFF
                            + csr::pmpconfig::pmpcfg::l0::CLEAR,
                    );
                }
                1 => {
                    csr::CSR.pmpcfg[x / 4].modify(
                        csr::pmpconfig::pmpcfg::r1::CLEAR
                            + csr::pmpconfig::pmpcfg::w1::CLEAR
                            + csr::pmpconfig::pmpcfg::x1::CLEAR
                            + csr::pmpconfig::pmpcfg::a1::OFF
                            + csr::pmpconfig::pmpcfg::l1::CLEAR,
                    );
                }
                2 => {
                    csr::CSR.pmpcfg[x / 4].modify(
                        csr::pmpconfig::pmpcfg::r2::CLEAR
                            + csr::pmpconfig::pmpcfg::w2::CLEAR
                            + csr::pmpconfig::pmpcfg::x2::CLEAR
                            + csr::pmpconfig::pmpcfg::a2::OFF
                            + csr::pmpconfig::pmpcfg::l2::CLEAR,
                    );
                }
                3 => {
                    csr::CSR.pmpcfg[x / 4].modify(
                        csr::pmpconfig::pmpcfg::r3::CLEAR
                            + csr::pmpconfig::pmpcfg::w3::CLEAR
                            + csr::pmpconfig::pmpcfg::x3::CLEAR
                            + csr::pmpconfig::pmpcfg::a3::OFF
                            + csr::pmpconfig::pmpcfg::l3::CLEAR,
                    );
                }
                _ => unreachable!(),
            }
            csr::CSR.pmpaddr[x].set(0x0);
        }

        //set first PMP to have permissions to entire space
        csr::CSR.pmpaddr[0].set(0xFFFF_FFFF);
        //enable R W X fields
        csr::CSR.pmpcfg[0].modify(csr::pmpconfig::pmpcfg::r0::SET);
        csr::CSR.pmpcfg[0].modify(csr::pmpconfig::pmpcfg::w0::SET);
        csr::CSR.pmpcfg[0].modify(csr::pmpconfig::pmpcfg::x0::SET);
        csr::CSR.pmpcfg[0].modify(csr::pmpconfig::pmpcfg::a0::TOR);
        // PMP is not configured for any process now
        self.last_configured_for.take();
    }

    fn enable_app_mpu(&self) {}

    fn disable_app_mpu(&self) {
        // PMP is not enabled for machine mode, so we don't have to do
        // anything
    }

    fn number_total_regions(&self) -> usize {
        $x / 2
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

        let region_num = config.unused_region_number()?;

        // Logical region
        let mut start = unallocated_memory_start as usize;
        let mut size = min_region_size;

        // Region start always has to align to 4 bytes
        if start % 4 != 0 {
            start += 4 - (start % 4);
        }

        // RISC-V PMP is not inclusive of the final address, while Tock is, increase the size by 1
        size += 1;

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

        config.sort_regions();

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
            config.unused_region_number()?
        };

        // Make sure there is enough memory for app memory and kernel memory.
        let memory_size = cmp::max(
            min_memory_size,
            initial_app_memory_size + initial_kernel_memory_size,
        );

        // RISC-V PMP is not inclusive of the final address, while Tock is, increase the memory_size by 1
        let mut region_size = memory_size as usize + 1;

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

        let region = PMPRegion::new(region_start as *const u8, region_size, permissions);

        config.regions[region_num] = Some(region);
        config.is_dirty.set(true);

        config.app_memory_region.set(region_num);

        config.sort_regions();

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

        let (region_start, region_size) = match config.regions[region_num] {
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

        let region = PMPRegion::new(region_start as *const u8, region_size, permissions);

        config.regions[region_num] = Some(region);
        config.is_dirty.set(true);

        config.sort_regions();

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig, app_id: &AppId) {
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
                        let cfg_val = r.cfg.value as u32;
                        let start = r.location.0 as usize;
                        let size = r.location.1;

                        match x % 2 {
                            0 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg[x / 2].modify(
                                    csr::pmpconfig::pmpcfg::r0::CLEAR
                                        + csr::pmpconfig::pmpcfg::w0::CLEAR
                                        + csr::pmpconfig::pmpcfg::x0::CLEAR
                                        + csr::pmpconfig::pmpcfg::a0::TOR,
                                );
                                csr::CSR.pmpaddr[x * 2].set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg[x / 2]
                                    .set(cfg_val << 8 | csr::CSR.pmpcfg[x / 2].get());
                                csr::CSR.pmpaddr[(x * 2) + 1]
                                    .set((start as u32 + size as u32) >> 2);
                            }
                            1 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg[x / 2].modify(
                                    csr::pmpconfig::pmpcfg::r2::CLEAR
                                        + csr::pmpconfig::pmpcfg::w2::CLEAR
                                        + csr::pmpconfig::pmpcfg::x2::CLEAR
                                        + csr::pmpconfig::pmpcfg::a2::TOR,
                                );
                                csr::CSR.pmpaddr[x * 2].set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg[x / 2]
                                    .set(cfg_val << 24 | csr::CSR.pmpcfg[x / 2].get());
                                csr::CSR.pmpaddr[(x * 2) + 1]
                                    .set((start as u32 + size as u32) >> 2);
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
};
}
