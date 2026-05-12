use crate::cpuss_ppu;
use crate::flashc;
use crate::peri;
use crate::pwrmode;
use crate::ramc_ppu;
use crate::srss;

/// Pre-initialize peripherals that are required for further system initialization.
/// Activates essential clocks.
/// Without this step, some peripherals do not work and abort the debugger connection.
pub fn preinit_peripherals() {
    srss::sys_init_enable_clocks();
    peri::sys_init_enable_peri();
}

/// Initialize system PPUs and set them to the default power mode.
fn init_pwr() {
    pwrmode::ppu_init();
    cpuss_ppu::init_ppu();
    ramc_ppu::init_ppu();

    /* Set Default mode to DEEPSLEEP */
    pwrmode::ppu_dynamic_enable(pwrmode::PwrPolicy::FullRetention);
    cpuss_ppu::ppu_dynamic_enable(cpuss_ppu::PwrPolicy::FullRetention);
    ramc_ppu::ppu_dynamic_enable(ramc_ppu::PwrPolicy::MemoryRetention);
}

/// Initialize system clocks, unlock watchdog and set flash wait states.
pub fn init_system() {
    flashc::set_waitstates(false, 180);

    /* Unlock WDT to be able to modify LFCLK registers */
    srss::wdt_unlock();

    init_pwr();

    srss::disable_fll();
    srss::enable_iho();

    srss::init_clock_paths();

    srss::init_dpll_lp().unwrap();

    srss::init_clk_hf();
    srss::init_clk_path0();

    srss::init_fll().unwrap();
    srss::init_clk_hf0();
}
