//! Implementation of the physical memory protection unit (PMP)
//! for the arty board
use crate::csr;
use kernel;
use kernel::mpu;

/// Struct storing configuration for a RISCV PMP region.
/// In accordance with the priviliged ISA 1.10
/// https://content.riscv.org/wp-content/uploads/2017/05/riscv-privileged-v1.10.pdf

/// Struct storing region configuration for RISCV PMP.
#[derive(Copy, Clone)]
pub struct PMPConfig {
    regions: usize,
}

impl Default for PMPConfig {
    /// number of regions on the arty chip
    fn default() -> PMPConfig {
        PMPConfig { regions: 4 }
    }
}

impl PMPConfig {
    pub const fn new(num_regions: usize) -> PMPConfig {
        PMPConfig {
            regions: num_regions,
        }
    }
}

impl kernel::mpu::MPU for PMPConfig {
    fn enable_mpu(&self) {}

    fn disable_mpu(&self) {
        for x in 0..self.regions {
            // disable everything
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
            //set first PMP to have permissions to entire space
            csr::CSR.pmpaddr0.set(0xFFFF_FFFF);
            //enable R W X fields
            csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r0::SET);
            csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w0::SET);
            csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x0::SET);
            csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a0::OFF);
        }
    }

    fn number_total_regions(&self) -> usize {
        self.regions
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
