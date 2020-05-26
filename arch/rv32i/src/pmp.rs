//! Implementation of the physical memory protection unit (PMP).

use core::cell::Cell;
use core::cmp;
use core::fmt;

use crate::csr;
use kernel;
use kernel::common::cells::MapCell;
use kernel::common::registers::register_bitfields;
use kernel::mpu;
use kernel::AppId;

// This is the RISC-V PMP support for Tock
// We use the PMP TOR alignment as there are alignment issues with NAPOT
// NAPOT would allow us to use more regions (each PMP region can be a
//     memory region) but the problem with NAPOT is the address must be
//     alignment to the size, which results in wasted memory.
// To avoid this wasted memory we use TOR and each memory region uses two
//     physical PMP regions.

// Generic PMP config
register_bitfields![u32,
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

/// Struct storing configuration for a RISC-V PMP region.
#[derive(Copy, Clone)]
pub struct PMPRegion {
    location: Option<(*const u8, usize)>,
    cfg: tock_registers::registers::FieldValue<u32, pmpcfg::Register>,
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
            location: Some((start, size)),
            cfg: pmpcfg,
        }
    }

    fn empty(_region_num: usize) -> PMPRegion {
        PMPRegion {
            location: None,
            cfg: pmpcfg::r::CLEAR + pmpcfg::w::CLEAR + pmpcfg::x::CLEAR + pmpcfg::a::OFF,
        }
    }

    fn location(&self) -> Option<(*const u8, usize)> {
        self.location
    }

    fn overlaps(&self, other_start: *const u8, other_size: usize) -> bool {
        let other_start = other_start as usize;
        let other_end = other_start + other_size;

        let (region_start, region_end) = match self.location {
            Some((region_start, region_size)) => {
                let region_start = region_start as usize;
                let region_end = region_start + region_size;
                (region_start, region_end)
            }
            None => return false,
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
    regions: [PMPRegion; 8],
    total_regions: usize,
    is_dirty: Cell<bool>,
    last_configured_for: MapCell<AppId>,
}

const APP_MEMORY_REGION_NUM: usize = 0;

impl Default for PMPConfig {
    /// number of regions on the arty chip
    fn default() -> PMPConfig {
        PMPConfig {
            regions: [
                PMPRegion::empty(0),
                PMPRegion::empty(1),
                PMPRegion::empty(2),
                PMPRegion::empty(3),
                PMPRegion::empty(4),
                PMPRegion::empty(5),
                PMPRegion::empty(6),
                PMPRegion::empty(7),
            ],
            total_regions: 8,
            is_dirty: Cell::new(true),
            last_configured_for: MapCell::empty(),
        }
    }
}

impl fmt::Display for PMPConfig {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl PMPConfig {
    pub fn new(pmp_regions: usize) -> PMPConfig {
        if pmp_regions > 16 {
            panic!("There is an ISA maximum of 16 PMP regions");
        }
        if pmp_regions < 4 {
            panic!("Tock requires at least 4 PMP regions");
        }
        PMPConfig {
            regions: [
                PMPRegion::empty(0),
                PMPRegion::empty(1),
                PMPRegion::empty(2),
                PMPRegion::empty(3),
                PMPRegion::empty(4),
                PMPRegion::empty(5),
                PMPRegion::empty(6),
                PMPRegion::empty(7),
            ],
            // As we use the PMP TOR setup we only support half the number
            // of regions as hardware supports
            total_regions: pmp_regions / 2,
            is_dirty: Cell::new(true),
            last_configured_for: MapCell::empty(),
        }
    }

    fn unused_region_number(&self) -> Option<usize> {
        for (number, region) in self.regions.iter().enumerate() {
            if number == APP_MEMORY_REGION_NUM {
                continue;
            }
            if let None = region.location() {
                if number < self.total_regions {
                    return Some(number);
                }
            }
        }
        None
    }

    fn clear_mpu_registers(&self) {
        for x in 0..self.total_regions {
            // If PMP is supported by the core then all 16 register sets must exist
            // They don't all have to do anything, but let's zero them all just in case.
            match x {
                0 => {
                    csr::CSR.pmpcfg0.modify(
                        csr::pmpconfig::pmpcfg::r0::CLEAR
                            + csr::pmpconfig::pmpcfg::w0::CLEAR
                            + csr::pmpconfig::pmpcfg::x0::CLEAR
                            + csr::pmpconfig::pmpcfg::a0::OFF
                            + csr::pmpconfig::pmpcfg::l0::CLEAR,
                    );
                    csr::CSR.pmpaddr0.set(0x0);
                }
                1 => {
                    csr::CSR.pmpcfg0.modify(
                        csr::pmpconfig::pmpcfg::r1::CLEAR
                            + csr::pmpconfig::pmpcfg::w1::CLEAR
                            + csr::pmpconfig::pmpcfg::x1::CLEAR
                            + csr::pmpconfig::pmpcfg::a1::OFF
                            + csr::pmpconfig::pmpcfg::l1::CLEAR,
                    );
                    csr::CSR.pmpaddr1.set(0x0);
                }
                2 => {
                    csr::CSR.pmpcfg0.modify(
                        csr::pmpconfig::pmpcfg::r2::CLEAR
                            + csr::pmpconfig::pmpcfg::w2::CLEAR
                            + csr::pmpconfig::pmpcfg::x2::CLEAR
                            + csr::pmpconfig::pmpcfg::a2::OFF
                            + csr::pmpconfig::pmpcfg::l2::CLEAR,
                    );
                    csr::CSR.pmpaddr2.set(0x0);
                }
                3 => {
                    csr::CSR.pmpcfg0.modify(
                        csr::pmpconfig::pmpcfg::r3::CLEAR
                            + csr::pmpconfig::pmpcfg::w3::CLEAR
                            + csr::pmpconfig::pmpcfg::x3::CLEAR
                            + csr::pmpconfig::pmpcfg::a3::OFF
                            + csr::pmpconfig::pmpcfg::l3::CLEAR,
                    );
                    csr::CSR.pmpaddr3.set(0x0);
                }
                4 => {
                    csr::CSR.pmpcfg1.modify(
                        csr::pmpconfig::pmpcfg::r0::CLEAR
                            + csr::pmpconfig::pmpcfg::w0::CLEAR
                            + csr::pmpconfig::pmpcfg::x0::CLEAR
                            + csr::pmpconfig::pmpcfg::a0::OFF
                            + csr::pmpconfig::pmpcfg::l0::CLEAR,
                    );
                    csr::CSR.pmpaddr4.set(0x0);
                }
                5 => {
                    csr::CSR.pmpcfg1.modify(
                        csr::pmpconfig::pmpcfg::r1::CLEAR
                            + csr::pmpconfig::pmpcfg::w1::CLEAR
                            + csr::pmpconfig::pmpcfg::x1::CLEAR
                            + csr::pmpconfig::pmpcfg::a1::OFF
                            + csr::pmpconfig::pmpcfg::l1::CLEAR,
                    );
                    csr::CSR.pmpaddr5.set(0x0);
                }
                6 => {
                    csr::CSR.pmpcfg1.modify(
                        csr::pmpconfig::pmpcfg::r2::CLEAR
                            + csr::pmpconfig::pmpcfg::w2::CLEAR
                            + csr::pmpconfig::pmpcfg::x2::CLEAR
                            + csr::pmpconfig::pmpcfg::a2::OFF
                            + csr::pmpconfig::pmpcfg::l2::CLEAR,
                    );
                    csr::CSR.pmpaddr6.set(0x0);
                }
                7 => {
                    csr::CSR.pmpcfg1.modify(
                        csr::pmpconfig::pmpcfg::r3::CLEAR
                            + csr::pmpconfig::pmpcfg::w3::CLEAR
                            + csr::pmpconfig::pmpcfg::x3::CLEAR
                            + csr::pmpconfig::pmpcfg::a3::OFF
                            + csr::pmpconfig::pmpcfg::l3::CLEAR,
                    );
                    csr::CSR.pmpaddr7.set(0x0);
                }
                8 => {
                    csr::CSR.pmpcfg2.modify(
                        csr::pmpconfig::pmpcfg::r0::CLEAR
                            + csr::pmpconfig::pmpcfg::w0::CLEAR
                            + csr::pmpconfig::pmpcfg::x0::CLEAR
                            + csr::pmpconfig::pmpcfg::a0::OFF
                            + csr::pmpconfig::pmpcfg::l0::CLEAR,
                    );
                    csr::CSR.pmpaddr8.set(0x0);
                }
                9 => {
                    csr::CSR.pmpcfg2.modify(
                        csr::pmpconfig::pmpcfg::r1::CLEAR
                            + csr::pmpconfig::pmpcfg::w1::CLEAR
                            + csr::pmpconfig::pmpcfg::x1::CLEAR
                            + csr::pmpconfig::pmpcfg::a1::OFF
                            + csr::pmpconfig::pmpcfg::l1::CLEAR,
                    );
                    csr::CSR.pmpaddr9.set(0x0);
                }
                10 => {
                    csr::CSR.pmpcfg2.modify(
                        csr::pmpconfig::pmpcfg::r2::CLEAR
                            + csr::pmpconfig::pmpcfg::w2::CLEAR
                            + csr::pmpconfig::pmpcfg::x2::CLEAR
                            + csr::pmpconfig::pmpcfg::a2::OFF
                            + csr::pmpconfig::pmpcfg::l2::CLEAR,
                    );
                    csr::CSR.pmpaddr10.set(0x0);
                }
                11 => {
                    csr::CSR.pmpcfg2.modify(
                        csr::pmpconfig::pmpcfg::r3::CLEAR
                            + csr::pmpconfig::pmpcfg::w3::CLEAR
                            + csr::pmpconfig::pmpcfg::x3::CLEAR
                            + csr::pmpconfig::pmpcfg::a3::OFF
                            + csr::pmpconfig::pmpcfg::l3::CLEAR,
                    );
                    csr::CSR.pmpaddr11.set(0x0);
                }
                12 => {
                    csr::CSR.pmpcfg3.modify(
                        csr::pmpconfig::pmpcfg::r0::CLEAR
                            + csr::pmpconfig::pmpcfg::w0::CLEAR
                            + csr::pmpconfig::pmpcfg::x0::CLEAR
                            + csr::pmpconfig::pmpcfg::a0::OFF
                            + csr::pmpconfig::pmpcfg::l0::CLEAR,
                    );
                    csr::CSR.pmpaddr12.set(0x0);
                }
                13 => {
                    csr::CSR.pmpcfg3.modify(
                        csr::pmpconfig::pmpcfg::r1::CLEAR
                            + csr::pmpconfig::pmpcfg::w1::CLEAR
                            + csr::pmpconfig::pmpcfg::x1::CLEAR
                            + csr::pmpconfig::pmpcfg::a1::OFF
                            + csr::pmpconfig::pmpcfg::l1::CLEAR,
                    );
                    csr::CSR.pmpaddr13.set(0x0);
                }
                14 => {
                    csr::CSR.pmpcfg3.modify(
                        csr::pmpconfig::pmpcfg::r2::CLEAR
                            + csr::pmpconfig::pmpcfg::w2::CLEAR
                            + csr::pmpconfig::pmpcfg::x2::CLEAR
                            + csr::pmpconfig::pmpcfg::a2::OFF
                            + csr::pmpconfig::pmpcfg::l2::CLEAR,
                    );
                    csr::CSR.pmpaddr14.set(0x0);
                }
                15 => {
                    csr::CSR.pmpcfg3.modify(
                        csr::pmpconfig::pmpcfg::r3::CLEAR
                            + csr::pmpconfig::pmpcfg::w3::CLEAR
                            + csr::pmpconfig::pmpcfg::x3::CLEAR
                            + csr::pmpconfig::pmpcfg::a3::OFF
                            + csr::pmpconfig::pmpcfg::l3::CLEAR,
                    );
                    csr::CSR.pmpaddr15.set(0x0);
                }
                // spec 1.10 only goes to 15
                _ => break,
            }
        }
        //set first PMP to have permissions to entire space
        csr::CSR.pmpaddr0.set(0xFFFF_FFFF);
        //enable R W X fields
        csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r0::SET);
        csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w0::SET);
        csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x0::SET);
        csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a0::TOR);
        // Set the dirty flag
        self.is_dirty.set(true);
    }
}

impl kernel::mpu::MPU for PMPConfig {
    type MpuConfig = PMPConfig;

    fn enable_mpu(&self) {}

    fn disable_mpu(&self) {}

    fn number_total_regions(&self) -> usize {
        self.total_regions
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
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return None;
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

        config.regions[region_num] = region;
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
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return None;
            }
        }

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

        config.regions[APP_MEMORY_REGION_NUM] = region;
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
        let (region_start, region_size) = match config.regions[APP_MEMORY_REGION_NUM].location() {
            Some((start, size)) => (start as usize, size),
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

        config.regions[APP_MEMORY_REGION_NUM] = region;
        config.is_dirty.set(true);

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig, app_id: &AppId) {
        let last_configured_for_this_app = self
            .last_configured_for
            .map_or(false, |last_app_id| last_app_id == app_id);

        // Skip PMP configuration if it is already configured for this app and the MPU
        // configuration of this app has not changed.
        if !last_configured_for_this_app || config.is_dirty.get() {
            // tock/tock#1860
            let mut regions_sorted = config.regions.clone();
            regions_sorted.sort_unstable_by(|a, b| {
                let (a_start, _a_size) = match a.location() {
                    Some((start, size)) => (start as usize, size),
                    None => (0xFFFF_FFFF, 0xFFFF_FFFF),
                };
                let (b_start, _b_size) = match b.location() {
                    Some((start, size)) => (start as usize, size),
                    None => (0xFFFF_FFFF, 0xFFFF_FFFF),
                };
                a_start.cmp(&b_start)
            });

            // tock/tock#1860
            self.clear_mpu_registers();

            for x in 0..self.total_regions {
                let region = regions_sorted[x];
                match region.location() {
                    Some((start, size)) => {
                        let cfg_val = region.cfg.value;

                        match x {
                            0 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg0.modify(
                                    csr::pmpconfig::pmpcfg::r0::CLEAR
                                        + csr::pmpconfig::pmpcfg::w0::CLEAR
                                        + csr::pmpconfig::pmpcfg::x0::CLEAR
                                        + csr::pmpconfig::pmpcfg::a0::TOR,
                                );
                                csr::CSR.pmpaddr0.set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg0.set(cfg_val << 8 | csr::CSR.pmpcfg0.get());
                                csr::CSR.pmpaddr1.set((start as u32 + size as u32) >> 2);
                            }
                            1 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg0.modify(
                                    csr::pmpconfig::pmpcfg::r2::CLEAR
                                        + csr::pmpconfig::pmpcfg::w2::CLEAR
                                        + csr::pmpconfig::pmpcfg::x2::CLEAR
                                        + csr::pmpconfig::pmpcfg::a2::TOR,
                                );
                                csr::CSR.pmpaddr2.set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg0.set(cfg_val << 24 | csr::CSR.pmpcfg0.get());
                                csr::CSR.pmpaddr3.set((start as u32 + size as u32) >> 2);
                            }
                            2 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg1.modify(
                                    csr::pmpconfig::pmpcfg::r0::CLEAR
                                        + csr::pmpconfig::pmpcfg::w0::CLEAR
                                        + csr::pmpconfig::pmpcfg::x0::CLEAR
                                        + csr::pmpconfig::pmpcfg::a0::TOR,
                                );
                                csr::CSR.pmpaddr4.set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg1.set(cfg_val << 8 | csr::CSR.pmpcfg0.get());
                                csr::CSR.pmpaddr5.set((start as u32 + size as u32) >> 2);
                            }
                            3 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg1.modify(
                                    csr::pmpconfig::pmpcfg::r3::CLEAR
                                        + csr::pmpconfig::pmpcfg::w3::CLEAR
                                        + csr::pmpconfig::pmpcfg::x3::CLEAR
                                        + csr::pmpconfig::pmpcfg::a3::TOR,
                                );
                                csr::CSR.pmpaddr6.set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg1.set(cfg_val << 24 | csr::CSR.pmpcfg0.get());
                                csr::CSR.pmpaddr7.set((start as u32 + size as u32) >> 2);
                            }
                            4 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg2.modify(
                                    csr::pmpconfig::pmpcfg::r0::CLEAR
                                        + csr::pmpconfig::pmpcfg::w0::CLEAR
                                        + csr::pmpconfig::pmpcfg::x0::CLEAR
                                        + csr::pmpconfig::pmpcfg::a0::TOR,
                                );
                                csr::CSR.pmpaddr8.set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg2.set(cfg_val << 8 | csr::CSR.pmpcfg0.get());
                                csr::CSR.pmpaddr9.set((start as u32 + size as u32) >> 2);
                            }
                            5 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg2.modify(
                                    csr::pmpconfig::pmpcfg::r3::CLEAR
                                        + csr::pmpconfig::pmpcfg::w3::CLEAR
                                        + csr::pmpconfig::pmpcfg::x3::CLEAR
                                        + csr::pmpconfig::pmpcfg::a3::TOR,
                                );
                                csr::CSR.pmpaddr10.set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg2.set(cfg_val << 24 | csr::CSR.pmpcfg0.get());
                                csr::CSR.pmpaddr11.set((start as u32 + size as u32) >> 2);
                            }
                            6 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg3.modify(
                                    csr::pmpconfig::pmpcfg::r0::CLEAR
                                        + csr::pmpconfig::pmpcfg::w0::CLEAR
                                        + csr::pmpconfig::pmpcfg::x0::CLEAR
                                        + csr::pmpconfig::pmpcfg::a0::TOR,
                                );
                                csr::CSR.pmpaddr12.set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg3.set(cfg_val << 8 | csr::CSR.pmpcfg0.get());
                                csr::CSR.pmpaddr13.set((start as u32 + size as u32) >> 2);
                            }
                            7 => {
                                // Disable access up to the start address
                                csr::CSR.pmpcfg3.modify(
                                    csr::pmpconfig::pmpcfg::r3::CLEAR
                                        + csr::pmpconfig::pmpcfg::w3::CLEAR
                                        + csr::pmpconfig::pmpcfg::x3::CLEAR
                                        + csr::pmpconfig::pmpcfg::a3::TOR,
                                );
                                csr::CSR.pmpaddr14.set((start as u32) >> 2);

                                // Set access to end address
                                csr::CSR.pmpcfg3.set(cfg_val << 24 | csr::CSR.pmpcfg0.get());
                                csr::CSR.pmpaddr15.set((start as u32 + size as u32) >> 2);
                            }
                            _ => break,
                        }
                    }
                    None => {}
                };
            }
            // Clear the dirty flag
            config.is_dirty.set(false);
            self.last_configured_for.put(*app_id);
        }
    }
}
