//! Implementation of the physical memory protection unit (PMP).

use core::fmt;

use crate::csr;
use kernel;
use kernel::common::registers::register_bitfields;
use kernel::mpu;

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
    _cfg: tock_registers::registers::FieldValue<u32, pmpcfg::Register>,
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
            _cfg: pmpcfg,
        }
    }

    fn empty(_region_num: usize) -> PMPRegion {
        PMPRegion {
            location: None,
            _cfg: pmpcfg::r::CLEAR + pmpcfg::w::CLEAR + pmpcfg::x::CLEAR + pmpcfg::a::OFF,
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
#[derive(Copy, Clone)]
pub struct PMPConfig {
    regions: [PMPRegion; 8],
    total_regions: usize,
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
}

impl kernel::mpu::MPU for PMPConfig {
    type MpuConfig = PMPConfig;

    fn enable_mpu(&self) {}

    fn disable_mpu(&self) {
        for x in 0..16 {
            // If PMP is supported by the core then all 16 register sets must exist
            // They don't all have to do anything, but let's zero them all just in case.
            match x {
                0 => {
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r0::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w0::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x0::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a0::OFF);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::l0::CLEAR);
                    csr::CSR.pmpaddr0.set(0x0);
                }
                1 => {
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r1::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w1::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x1::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a1::OFF);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::l1::CLEAR);
                    csr::CSR.pmpaddr1.set(0x0);
                }
                2 => {
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r2::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w2::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x2::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a2::OFF);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::l2::CLEAR);
                    csr::CSR.pmpaddr2.set(0x0);
                }
                3 => {
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r3::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w3::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x3::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a3::OFF);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::l3::CLEAR);
                    csr::CSR.pmpaddr3.set(0x0);
                }
                4 => {
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::r0::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::w0::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::x0::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::a0::OFF);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::l0::CLEAR);
                    csr::CSR.pmpaddr4.set(0x0);
                }
                5 => {
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::r1::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::w1::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::x1::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::a1::OFF);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::l1::CLEAR);
                    csr::CSR.pmpaddr5.set(0x0);
                }
                6 => {
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::r2::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::w2::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::x2::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::a2::OFF);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::l2::CLEAR);
                    csr::CSR.pmpaddr6.set(0x0);
                }
                7 => {
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::r3::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::w3::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::x3::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::a3::OFF);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::l3::CLEAR);
                    csr::CSR.pmpaddr7.set(0x0);
                }
                8 => {
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::r0::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::w0::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::x0::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::a0::OFF);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::l0::CLEAR);
                    csr::CSR.pmpaddr8.set(0x0);
                }
                9 => {
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::r1::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::w1::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::x1::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::a1::OFF);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::l1::CLEAR);
                    csr::CSR.pmpaddr9.set(0x0);
                }
                10 => {
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::r2::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::w2::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::x2::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::a2::OFF);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::l2::CLEAR);
                    csr::CSR.pmpaddr10.set(0x0);
                }
                11 => {
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::r3::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::w3::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::x3::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::a3::OFF);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::l3::CLEAR);
                    csr::CSR.pmpaddr11.set(0x0);
                }
                12 => {
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::r0::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::w0::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::x0::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::a0::OFF);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::l0::CLEAR);
                    csr::CSR.pmpaddr12.set(0x0);
                }
                13 => {
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::r1::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::w1::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::x1::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::a1::OFF);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::l1::CLEAR);
                    csr::CSR.pmpaddr13.set(0x0);
                }
                14 => {
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::r2::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::w2::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::x2::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::a2::OFF);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::l2::CLEAR);
                    csr::CSR.pmpaddr14.set(0x0);
                }
                15 => {
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::r3::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::w3::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::x3::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::a3::OFF);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::l3::CLEAR);
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
        csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a0::OFF)
    }

    fn number_total_regions(&self) -> usize {
        self.total_regions
    }

    fn allocate_region(
        &self,
        _unallocated_memory_start: *const u8,
        _unallocated_memory_size: usize,
        _min_region_size: usize,
        _permissions: mpu::Permissions,
        _config: &mut Self::MpuConfig,
    ) -> Option<mpu::Region> {
        None
    }

    fn allocate_app_memory_region(
        &self,
        _unallocated_memory_start: *const u8,
        _unallocated_memory_size: usize,
        _min_memory_size: usize,
        _initial_app_memory_size: usize,
        _initial_kernel_memory_size: usize,
        _permissions: mpu::Permissions,
        _config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        None
    }

    fn update_app_memory_region(
        &self,
        _app_memory_break: *const u8,
        _kernel_memory_break: *const u8,
        _permissions: mpu::Permissions,
        _config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        Err(())
    }

    fn configure_mpu(&self, _config: &Self::MpuConfig) {}
}
