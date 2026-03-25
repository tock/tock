use crate::srss_registers::*;
use kernel::utilities::{
    registers::{
        interfaces::{ReadWriteable, Readable, Writeable},
        ReadOnly, ReadWrite,
    },
    StaticRef,
};

const SRSS_BASE: StaticRef<SrssRegisters> =
    unsafe { StaticRef::new(0x42200000 as *const SrssRegisters) };

fn delay_rough_us(us: u32) {
    const FREQ_MHZ: u32 = 180;
    let cycles = us * FREQ_MHZ;
    for _ in 0..(cycles) {
        core::hint::black_box(());
    }
}

struct DpllLpConfig {
    pub feedback_div: u32,
    pub reference_div: u32,
    pub output_div: u32,
    pub pll_dco_mode: bool,
    pub output_mode: u32,
    pub frac_div: u32,
    pub frac_dither_en: bool,
    pub frac_en: bool,
    pub sscg_depth: u32,
    pub sscg_rate: u32,
    pub sscg_dither_en: bool,
    pub sscg_mode: u32,
    pub sscg_en: bool,
    pub dco_code: u32,
    pub acc_mode: u32,
    pub tdc_mode: u32,
    pub pll_tg: u32,
    pub acc_cnt_lock: bool,
    pub ki_int: u32,
    pub kp_int: u32,
    pub ki_acc_int: u32,
    pub kp_acc_int: u32,
    pub ki_frac: u32,
    pub kp_frac: u32,
    pub ki_acc_frac: u32,
    pub kp_acc_frac: u32,
    pub ki_sscg: u32,
    pub kp_sscg: u32,
    pub ki_acc_sscg: u32,
    pub kp_acc_sscg: u32,
}
const DPLL_LP_CONFIG_0: DpllLpConfig = DpllLpConfig {
    feedback_div: 45,
    reference_div: 6,
    output_div: 2,
    pll_dco_mode: true,
    output_mode: 0, // CY_SYSCLK_FLLPLL_OUTPUT_AUTO
    frac_div: 0,
    frac_dither_en: false,
    frac_en: true,
    sscg_depth: 0x0,
    sscg_rate: 0x0,
    sscg_dither_en: false,
    sscg_mode: 0x0,
    sscg_en: false,
    dco_code: 0x0,
    acc_mode: 0x1,
    tdc_mode: 0x1,
    pll_tg: 0x0,
    acc_cnt_lock: false,
    ki_int: 0x24,
    kp_int: 0x1C,
    ki_acc_int: 0x23,
    kp_acc_int: 0x1A,
    ki_frac: 0x24,
    kp_frac: 0x20,
    ki_acc_frac: 0x23,
    kp_acc_frac: 0x1A,
    ki_sscg: 0x18,
    kp_sscg: 0x18,
    ki_acc_sscg: 0x16,
    kp_acc_sscg: 0x14,
};
const DPLL_LP_CONFIG_1: DpllLpConfig = DpllLpConfig {
    feedback_div: 60,
    reference_div: 6,
    output_div: 2,
    pll_dco_mode: true,
    output_mode: 0, // CY_SYSCLK_FLLPLL_OUTPUT_AUTO
    frac_div: 0,
    frac_dither_en: false,
    frac_en: true,
    sscg_depth: 0x0,
    sscg_rate: 0x0,
    sscg_dither_en: false,
    sscg_mode: 0x0,
    sscg_en: false,
    dco_code: 0x0,
    acc_mode: 0x1,
    tdc_mode: 0x1,
    pll_tg: 0x0,
    acc_cnt_lock: false,
    ki_int: 0x24,
    kp_int: 0x1C,
    ki_acc_int: 0x23,
    kp_acc_int: 0x1A,
    ki_frac: 0x24,
    kp_frac: 0x20,
    ki_acc_frac: 0x23,
    kp_acc_frac: 0x1A,
    ki_sscg: 0x18,
    kp_sscg: 0x18,
    ki_acc_sscg: 0x16,
    kp_acc_sscg: 0x14,
};
struct FllManualConfig {
    pub fll_mult: u32,
    pub ref_div: u32,
    pub cco_range: u32, // or an enum if you have one
    pub enable_output_div: bool,
    pub lock_tolerance: u32,
    pub igain: u32,
    pub pgain: u32,
    pub settling_count: u32,
    pub output_mode: u32, // corresponds to BYPASS_SEL in your C snippet
    pub cco_freq: u32,
}

const SRSS_0_CLOCK_0_FLL_0_FLL_CONFIG: FllManualConfig = FllManualConfig {
    fll_mult: 500,
    ref_div: 120,
    cco_range: CLK_FLL_CONFIG4::CCO_RANGE::RANGE4_150200MHz.value,
    enable_output_div: true,
    lock_tolerance: 10,
    igain: 9,
    pgain: 5,
    settling_count: 48,
    output_mode: CLK_FLL_CONFIG3::BYPASS_SEL::FLL_OUT.value,
    cco_freq: 355,
};

pub struct Srss {
    registers: StaticRef<SrssRegisters>,
}

impl Srss {
    pub const fn new() -> Srss {
        Srss {
            registers: SRSS_BASE,
        }
    }

    pub fn wdt_unlock(&self) {
        // Write 1 to bit to clear it
        self.registers.wdt_ctl.modify(WDT_CTL::WDT_LOCK::ClearsBit0);
        self.registers.wdt_ctl.modify(WDT_CTL::WDT_LOCK::ClearsBit1);
    }

    pub fn init_clock_paths(&self) {
        [
            &self.registers.clk_path_select1,
            &self.registers.clk_path_select2,
            &self.registers.clk_path_select3,
            &self.registers.clk_path_select4,
            &self.registers.clk_path_select5,
        ]
        .iter()
        .for_each(|clk_path_select| {
            clk_path_select.modify(CLK_PATH_SELECT::PATH_MUX::IHO);
        });
        self.registers
            .clk_path_select6
            .modify(CLK_PATH_SELECT::PATH_MUX::IMO);
    }

    pub fn sys_init_enable_clocks(&self) {
        // set source
        self.registers
            .clk_root_select2
            .modify(CLK_ROOT_SELECT::ROOT_MUX::PATH0);
        // set divider
        self.registers
            .clk_root_select2
            .modify(CLK_ROOT_SELECT::ROOT_DIV_INT::NO_DIV);
        // enable
        self.registers
            .clk_root_select2
            .modify(CLK_ROOT_SELECT::ENABLE::SET);

        self.registers
            .clk_root_select3
            .modify(CLK_ROOT_SELECT::ROOT_MUX::PATH0);
        self.registers
            .clk_root_select3
            .modify(CLK_ROOT_SELECT::ROOT_DIV_INT::NO_DIV);
        self.registers
            .clk_root_select3
            .modify(CLK_ROOT_SELECT::ENABLE::SET);

        self.registers
            .clk_root_select4
            .modify(CLK_ROOT_SELECT::ROOT_MUX::PATH0);
        self.registers
            .clk_root_select4
            .modify(CLK_ROOT_SELECT::ROOT_DIV_INT::NO_DIV);
        self.registers
            .clk_root_select4
            .modify(CLK_ROOT_SELECT::ENABLE::SET);
    }

    pub fn disable_fll(&self) {
        const MAX_DELAY_US: u32 = 100;
        self.registers
            .clk_fll_config3
            .modify(CLK_FLL_CONFIG3::BYPASS_SEL::FLL_REF);

        let mut success = false;
        for _ in 0..MAX_DELAY_US {
            if self
                .registers
                .clk_fll_config3
                .any_matching_bits_set(CLK_FLL_CONFIG3::BYPASS_SEL::FLL_REF)
            {
                success = true;
                break;
            }
            delay_rough_us(1);
        }
        if success {
            delay_rough_us(2);
            self.registers
                .clk_fll_config
                .modify(CLK_FLL_CONFIG::FLL_ENABLE::CLEAR);
            self.registers
                .clk_fll_config4
                .modify(CLK_FLL_CONFIG4::CCO_ENABLE::CLEAR);
        }
    }
    pub fn enable_iho(&self) {
        self.registers
            .clk_iho_config
            .modify(CLK_IHO_CONFIG::ENABLE::SET);
    }

    pub fn init_dpll_lp(&self) -> Result<(), ()> {
        [
            (
                &self.registers.clk_dpll_lp0_config,
                &self.registers.clk_dpll_lp0_status,
                0,
                &DPLL_LP_CONFIG_0,
            ),
            (
                &self.registers.clk_dpll_lp1_config,
                &self.registers.clk_dpll_lp1_status,
                1,
                &DPLL_LP_CONFIG_1,
            ),
        ]
        .iter()
        .try_for_each(|&(config_reg, status_reg, pll_num, config)| {
            config_reg.modify(CLK_DPLL_LP_CONFIG::BYPASS_SEL::PLL_BYPASS);
            delay_rough_us(1);
            config_reg.modify(CLK_DPLL_LP_CONFIG::ENABLE::CLEAR);

            self.configure_dpll_lp(pll_num, config);

            self.enable_dpll_lp(config_reg, status_reg)?;

            Ok(())
        })
    }

    /// Enable PLL and wait for lock/output to stabilize
    fn enable_dpll_lp(
        &self,
        config_reg: &ReadWrite<u32, CLK_DPLL_LP_CONFIG::Register>,
        status_reg: &ReadOnly<u32, CLK_DPLL_LP_STATUS::Register>,
    ) -> Result<(), ()> {
        const MAX_DELAY_US: u32 = 10_000;

        config_reg.modify(CLK_DPLL_LP_CONFIG::ENABLE::SET);

        let mut locked = false;
        for _ in 0..MAX_DELAY_US {
            if status_reg.any_matching_bits_set(CLK_DPLL_LP_STATUS::LOCKED::SET) {
                locked = true;
                break;
            }
            delay_rough_us(1);
        }

        if locked {
            if config_reg.any_matching_bits_set(CLK_DPLL_LP_CONFIG::BYPASS_SEL::PLL_BYPASS) {
                config_reg.modify(CLK_DPLL_LP_CONFIG::BYPASS_SEL::PLL_OUT);
            }
            Ok(())
        } else {
            // Switch bypass back to PLL output
            config_reg.modify(CLK_DPLL_LP_CONFIG::BYPASS_SEL::PLL_BYPASS);

            delay_rough_us(1);

            config_reg.modify(CLK_DPLL_LP_CONFIG::ENABLE::CLEAR);

            Err(())
        }
    }

    /// Configure DPLL LP registers for the given PLL (0 or 1) using the provided config.
    fn configure_dpll_lp(&self, pll_num: usize, config: &DpllLpConfig) {
        // Select correct register set for PLL0 or PLL1
        let (
            config_reg,
            config2_reg,
            config3_reg,
            config4_reg,
            config5_reg,
            config6_reg,
            config7_reg,
        ) = match pll_num {
            0 => (
                &self.registers.clk_dpll_lp0_config,
                &self.registers.clk_dpll_lp0_config2,
                &self.registers.clk_dpll_lp0_config3,
                &self.registers.clk_dpll_lp0_config4,
                &self.registers.clk_dpll_lp0_config5,
                &self.registers.clk_dpll_lp0_config6,
                &self.registers.clk_dpll_lp0_config7,
            ),
            1 => (
                &self.registers.clk_dpll_lp1_config,
                &self.registers.clk_dpll_lp1_config2,
                &self.registers.clk_dpll_lp1_config3,
                &self.registers.clk_dpll_lp1_config4,
                &self.registers.clk_dpll_lp1_config5,
                &self.registers.clk_dpll_lp1_config6,
                &self.registers.clk_dpll_lp1_config7,
            ),
            _ => return, // Invalid PLL number
        };

        // Only configure if output_mode != 2 (CY_SYSCLK_FLLPLL_OUTPUT_INPUT)
        if config.output_mode != 2 {
            config_reg.write(
                CLK_DPLL_LP_CONFIG::FEEDBACK_DIV.val(config.feedback_div)
                    + CLK_DPLL_LP_CONFIG::REFERENCE_DIV.val(config.reference_div)
                    + CLK_DPLL_LP_CONFIG::OUTPUT_DIV.val(config.output_div)
                    + CLK_DPLL_LP_CONFIG::PLL_DCO_CODE_MULT.val(config.pll_dco_mode as u32),
            );

            config2_reg.write(
                CLK_DPLL_LP_CONFIG2::FRAC_DIV.val(config.frac_div)
                    + CLK_DPLL_LP_CONFIG2::FRAC_DITHER_EN.val(config.frac_dither_en as u32)
                    + CLK_DPLL_LP_CONFIG2::FRAC_EN.val(config.frac_en as u32),
            );

            config3_reg.write(
                CLK_DPLL_LP_CONFIG3::SSCG_DEPTH.val(config.sscg_depth)
                    + CLK_DPLL_LP_CONFIG3::SSCG_RATE.val(config.sscg_rate)
                    + CLK_DPLL_LP_CONFIG3::SSCG_DITHER_EN.val(config.sscg_dither_en as u32)
                    + CLK_DPLL_LP_CONFIG3::SSCG_MODE.val(config.sscg_mode)
                    + CLK_DPLL_LP_CONFIG3::SSCG_EN.val(config.sscg_en as u32),
            );

            config4_reg.write(
                CLK_DPLL_LP_CONFIG4::DCO_CODE.val(config.dco_code)
                    + CLK_DPLL_LP_CONFIG4::ACC_MODE.val(config.acc_mode)
                    + CLK_DPLL_LP_CONFIG4::TDC_MODE.val(config.tdc_mode)
                    + CLK_DPLL_LP_CONFIG4::PLL_TG.val(config.pll_tg)
                    + CLK_DPLL_LP_CONFIG4::ACC_CNT_LOCK.val(config.acc_cnt_lock as u32),
            );

            config5_reg.write(
                CLK_DPLL_LP_CONFIG5::KI_INT.val(config.ki_int)
                    + CLK_DPLL_LP_CONFIG5::KP_INT.val(config.kp_int)
                    + CLK_DPLL_LP_CONFIG5::KI_ACC_INT.val(config.ki_acc_int)
                    + CLK_DPLL_LP_CONFIG5::KP_ACC_INT.val(config.kp_acc_int),
            );

            config6_reg.write(
                CLK_DPLL_LP_CONFIG6::KI_FRACT.val(config.ki_frac)
                    + CLK_DPLL_LP_CONFIG6::KP_FRACT.val(config.kp_frac)
                    + CLK_DPLL_LP_CONFIG6::KI_ACC_FRACT.val(config.ki_acc_frac)
                    + CLK_DPLL_LP_CONFIG6::KP_ACC_FRACT.val(config.kp_acc_frac),
            );

            config7_reg.write(
                CLK_DPLL_LP_CONFIG7::KI_SSCG.val(config.ki_sscg)
                    + CLK_DPLL_LP_CONFIG7::KP_SSCG.val(config.kp_sscg)
                    + CLK_DPLL_LP_CONFIG7::KI_ACC_SSCG.val(config.ki_acc_sscg)
                    + CLK_DPLL_LP_CONFIG7::KP_ACC_SSCG.val(config.kp_acc_sscg),
            );
        }

        // Always set BYPASS_SEL to output_mode
        config_reg.modify(CLK_DPLL_LP_CONFIG::BYPASS_SEL.val(config.output_mode));
    }

    pub fn init_clk_hf(&self) {
        // 1
        self.registers
            .clk_root_select1
            .modify(CLK_ROOT_SELECT::ROOT_MUX::PATH1);
        self.registers
            .clk_root_select1
            .modify(CLK_ROOT_SELECT::ROOT_DIV_INT::NO_DIV);
        self.registers
            .clk_root_select1
            .modify(CLK_ROOT_SELECT::ENABLE::SET);

        // 2
        self.registers
            .clk_root_select2
            .modify(CLK_ROOT_SELECT::ROOT_MUX::PATH0);
        self.registers
            .clk_root_select2
            .modify(CLK_ROOT_SELECT::ROOT_DIV_INT::NO_DIV);
        self.registers
            .clk_root_select2
            .modify(CLK_ROOT_SELECT::ENABLE::SET);

        // 3
        self.registers
            .clk_root_select3
            .modify(CLK_ROOT_SELECT::ROOT_MUX::PATH2);
        self.registers
            .clk_root_select3
            .modify(CLK_ROOT_SELECT::ROOT_DIV_INT::NO_DIV);
        self.registers
            .clk_root_select3
            .modify(CLK_ROOT_SELECT::ENABLE::SET);

        // 4
        self.registers
            .clk_root_select4
            .modify(CLK_ROOT_SELECT::ROOT_MUX::PATH0);
        self.registers
            .clk_root_select4
            .modify(CLK_ROOT_SELECT::ROOT_DIV_INT::NO_DIV);
        self.registers
            .clk_root_select4
            .modify(CLK_ROOT_SELECT::ENABLE::SET);
    }

    pub fn init_clk_hf0(&self) {
        self.registers
            .clk_root_select0
            .modify(CLK_ROOT_SELECT::ROOT_MUX::PATH1);
        self.registers
            .clk_root_select0
            .modify(CLK_ROOT_SELECT::ROOT_DIV_INT::NO_DIV);
    }

    pub fn init_clk_path0(&self) {
        self.registers
            .clk_path_select0
            .modify(CLK_PATH_SELECT::PATH_MUX::IHO);
    }

    pub fn init_fll(&self) -> Result<(), ()> {
        const MAX_DELAY_US: u32 = 20_000;
        self.fll_manual_configure(&SRSS_0_CLOCK_0_FLL_0_FLL_CONFIG);

        // Enable
        self.registers
            .clk_fll_config4
            .modify(CLK_FLL_CONFIG4::CCO_ENABLE::SET);

        let mut cc0_ready = false;
        for _ in 0..MAX_DELAY_US {
            if self
                .registers
                .clk_fll_status
                .any_matching_bits_set(CLK_FLL_STATUS::CCO_READY::SET)
            {
                cc0_ready = true;
                break;
            }
            delay_rough_us(1);
        }
        // Strangly setting this in manual config doesn't seem to work, so set it here after enabling the FLL
        // This work in MTB though (0_o)
        self.registers
            .clk_fll_config4
            .modify(CLK_FLL_CONFIG4::CCO_RANGE::RANGE4_150200MHz);
        self.registers
            .clk_fll_config3
            .modify(CLK_FLL_CONFIG3::BYPASS_SEL::FLL_REF);
        if cc0_ready {
            self.registers
                .clk_fll_config
                .modify(CLK_FLL_CONFIG::FLL_ENABLE::SET);
        }

        let mut locked = false;
        for _ in 0..MAX_DELAY_US {
            if self
                .registers
                .clk_fll_status
                .any_matching_bits_set(CLK_FLL_STATUS::LOCKED::SET)
            {
                locked = true;
                break;
            }
            delay_rough_us(1);
        }

        if locked {
            self.registers
                .clk_fll_config3
                .modify(CLK_FLL_CONFIG3::BYPASS_SEL::FLL_OUT);
            Ok(())
        } else {
            /* If lock doesn't occur, FLL is stopped */
            self.disable_fll();
            return Err(());
        }
    }

    fn fll_manual_configure(&self, config: &FllManualConfig) {
        self.registers.clk_fll_config.write(
            CLK_FLL_CONFIG::FLL_MULT.val(config.fll_mult)
                + CLK_FLL_CONFIG::FLL_OUTPUT_DIV.val(config.enable_output_div as u32),
        );

        self.registers.clk_fll_config2.write(
            CLK_FLL_CONFIG2::FLL_REF_DIV.val(config.ref_div)
                + CLK_FLL_CONFIG2::LOCK_TOL.val(config.lock_tolerance),
        );

        self.registers.clk_fll_config3.write(
            CLK_FLL_CONFIG3::FLL_LF_IGAIN.val(config.igain)
                + CLK_FLL_CONFIG3::FLL_LF_PGAIN.val(config.pgain)
                + CLK_FLL_CONFIG3::SETTLING_COUNT.val(config.settling_count)
                + CLK_FLL_CONFIG3::BYPASS_SEL.val(config.output_mode),
        );

        self.registers
            .clk_fll_config4
            .modify(CLK_FLL_CONFIG4::CCO_RANGE.val(config.cco_range));
        self.registers
            .clk_fll_config4
            .modify(CLK_FLL_CONFIG4::CCO_FREQ.val(config.cco_freq));
    }
}
