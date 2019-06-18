use kernel;
use kernel::mpu;
use riscvregs;

/// Struct storing configuration for a Cortex-M MPU region.
#[derive(Copy, Clone)]
pub struct PMPRegion {
    base: usize,
    length: usize,
    r_val: bool,
    w_val: bool,
    x_val: bool,
    a_val: riscvregs::register::PmpAField,
    l_val: bool,
}

/// Struct storing region configuration for the Cortex-M MPU.
#[derive(Copy, Clone)]
pub struct PMPConfig {
    regions: usize,
}

impl Default for PMPConfig {
    // the default for arty chip
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
        unsafe {
            for x in 0..self.regions {
                // disable everything
                match x {
                    0 => {
                        riscvregs::register::pmpcfg0::clear_r0();
                        riscvregs::register::pmpcfg0::clear_w0();
                        riscvregs::register::pmpcfg0::clear_x0();
                        riscvregs::register::pmpcfg0::set_a0(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg0::clear_l0();
                        riscvregs::register::pmpaddr0::write(0);
                    }
                    1 => {
                        riscvregs::register::pmpcfg0::clear_r1();
                        riscvregs::register::pmpcfg0::clear_w1();
                        riscvregs::register::pmpcfg0::clear_x1();
                        riscvregs::register::pmpcfg0::set_a1(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg0::clear_l1();
                        riscvregs::register::pmpaddr1::write(0);
                    }
                    2 => {
                        riscvregs::register::pmpcfg0::clear_r2();
                        riscvregs::register::pmpcfg0::clear_w2();
                        riscvregs::register::pmpcfg0::clear_x2();
                        riscvregs::register::pmpcfg0::set_a2(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg0::clear_l2();
                        riscvregs::register::pmpaddr2::write(0);
                    }
                    3 => {
                        riscvregs::register::pmpcfg0::clear_r3();
                        riscvregs::register::pmpcfg0::clear_w3();
                        riscvregs::register::pmpcfg0::clear_x3();
                        riscvregs::register::pmpcfg0::set_a3(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg0::clear_l3();
                        riscvregs::register::pmpaddr3::write(0);
                    }
                    4 => {
                        riscvregs::register::pmpcfg1::clear_r4();
                        riscvregs::register::pmpcfg1::clear_w4();
                        riscvregs::register::pmpcfg1::clear_x4();
                        riscvregs::register::pmpcfg1::set_a4(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg1::clear_l4();
                        riscvregs::register::pmpaddr4::write(0);
                    }
                    5 => {
                        riscvregs::register::pmpcfg1::clear_r5();
                        riscvregs::register::pmpcfg1::clear_w5();
                        riscvregs::register::pmpcfg1::clear_x5();
                        riscvregs::register::pmpcfg1::set_a5(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg1::clear_l5();
                        riscvregs::register::pmpaddr5::write(0);
                    }
                    6 => {
                        riscvregs::register::pmpcfg1::clear_r6();
                        riscvregs::register::pmpcfg1::clear_w6();
                        riscvregs::register::pmpcfg1::clear_x6();
                        riscvregs::register::pmpcfg1::set_a6(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg1::clear_l6();
                        riscvregs::register::pmpaddr6::write(0);
                    }
                    7 => {
                        riscvregs::register::pmpcfg1::clear_r7();
                        riscvregs::register::pmpcfg1::clear_w7();
                        riscvregs::register::pmpcfg1::clear_x7();
                        riscvregs::register::pmpcfg1::set_a7(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg1::clear_l7();
                        riscvregs::register::pmpaddr7::write(0);
                    }
                    8 => {
                        riscvregs::register::pmpcfg2::clear_r8();
                        riscvregs::register::pmpcfg2::clear_w8();
                        riscvregs::register::pmpcfg2::clear_x8();
                        riscvregs::register::pmpcfg2::set_a8(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg2::clear_l8();
                        riscvregs::register::pmpaddr8::write(1);
                    }
                    9 => {
                        riscvregs::register::pmpcfg2::clear_r9();
                        riscvregs::register::pmpcfg2::clear_w9();
                        riscvregs::register::pmpcfg2::clear_x9();
                        riscvregs::register::pmpcfg2::set_a9(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg2::clear_l9();
                        riscvregs::register::pmpaddr9::write(1);
                    }
                    10 => {
                        riscvregs::register::pmpcfg2::clear_r10();
                        riscvregs::register::pmpcfg2::clear_w10();
                        riscvregs::register::pmpcfg2::clear_x10();
                        riscvregs::register::pmpcfg2::set_a10(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg2::clear_l10();
                        riscvregs::register::pmpaddr10::write(1);
                    }
                    11 => {
                        riscvregs::register::pmpcfg2::clear_r11();
                        riscvregs::register::pmpcfg2::clear_w11();
                        riscvregs::register::pmpcfg2::clear_x11();
                        riscvregs::register::pmpcfg2::set_a11(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg2::clear_l11();
                        riscvregs::register::pmpaddr11::write(1);
                    }
                    12 => {
                        riscvregs::register::pmpcfg3::clear_r12();
                        riscvregs::register::pmpcfg3::clear_w12();
                        riscvregs::register::pmpcfg3::clear_x12();
                        riscvregs::register::pmpcfg3::set_a12(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg3::clear_l12();
                        riscvregs::register::pmpaddr12::write(1);
                    }
                    13 => {
                        riscvregs::register::pmpcfg3::clear_r13();
                        riscvregs::register::pmpcfg3::clear_w13();
                        riscvregs::register::pmpcfg3::clear_x13();
                        riscvregs::register::pmpcfg3::set_a13(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg3::clear_l13();
                        riscvregs::register::pmpaddr13::write(1);
                    }
                    14 => {
                        riscvregs::register::pmpcfg3::clear_r14();
                        riscvregs::register::pmpcfg3::clear_w14();
                        riscvregs::register::pmpcfg3::clear_x14();
                        riscvregs::register::pmpcfg3::set_a14(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg3::clear_l14();
                        riscvregs::register::pmpaddr14::write(1);
                    }
                    15 => {
                        riscvregs::register::pmpcfg3::clear_r15();
                        riscvregs::register::pmpcfg3::clear_w15();
                        riscvregs::register::pmpcfg3::clear_x15();
                        riscvregs::register::pmpcfg3::set_a15(riscvregs::register::PmpAField::OFF);
                        riscvregs::register::pmpcfg3::clear_l15();
                        riscvregs::register::pmpaddr15::write(1);
                    }
                    // spec 1.10 only goes to 15
                    _ => break,
                }
                //set first PMP to have permissions to entire space
                riscvregs::register::pmpaddr0::write(0xFFFF_FFFF);
                // enable R W X fields
                riscvregs::register::pmpcfg0::set_r0();
                riscvregs::register::pmpcfg0::set_w0();
                riscvregs::register::pmpcfg0::set_x0();
                riscvregs::register::pmpcfg0::set_a0(riscvregs::register::PmpAField::OFF);
            }
        }
    }

    fn number_total_regions(&self) -> usize {
        self.regions
    }

    fn allocate_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<mpu::Region> {
        None
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
        None
    }

    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        Err(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig) {}
}
