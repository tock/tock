use crate::rcc::Rcc;
use crate::rcc::PLLP;

use kernel::debug;
use kernel::ErrorCode;

#[derive(Debug, PartialEq)]
struct PllConfig {
    p: PLLP,
    n: usize,
}

impl Default for PllConfig {
    fn default() -> Self {
        Self {
            p: PLLP::DivideBy2,
            n: 100,
        }
    }
}

impl PllConfig {
    fn get_p(&self) -> PLLP {
        self.p
    }

    fn set_p(&mut self, p: PLLP) {
        self.p = p;
    }

    fn get_n(&self) -> usize {
        self.n
    }

    fn set_n(&mut self, n: usize) -> Result<(), ErrorCode> {
        if n < 50 || n >= 432 {
            return Result::from(ErrorCode::INVAL);
        }
        self.n = n;

        Ok(())
    }
}

/// Main PLL clock.

// At the moment, only HSI is supported as the source clock.
pub struct Pll<'a> {
    rcc: &'a Rcc,
}

impl<'a> Pll<'a> {
    pub fn new(rcc: &'a Rcc) -> Self {
        Self {
            rcc,
        }
    }

    // **NOTE**: It assumes a value of 8 for PLLM
    // **TODO**: Change this function so it can adapt to changes of the PLLM
    fn get_pll_config_for_frequency_using_hsi(desired_frequency_mhz: usize) -> Option<PllConfig> {
        if desired_frequency_mhz < 13 || desired_frequency_mhz > 216 {
            return None;
        }
        let mut pll_config = PllConfig::default();
        // As the documentation says, selecting a frequency of 2MHz for the VCO input frequency
        // limits the PLL jitter. Since the HSI frequency is 16MHz, M must be configured
        // accordingly.
        pll_config.set_p(
            if desired_frequency_mhz < 55 {
                PLLP::DivideBy8
            } else if desired_frequency_mhz < 73 {
                PLLP::DivideBy6
            } else if desired_frequency_mhz < 109 {
                PLLP::DivideBy4
            } else {
                PLLP::DivideBy2
            }
        );
        if let Err(_) = pll_config.set_n(match pll_config.get_p() {
            PLLP::DivideBy8 => desired_frequency_mhz * 4,
            PLLP::DivideBy6 => desired_frequency_mhz * 3,
            PLLP::DivideBy4 => desired_frequency_mhz * 2,
            _ => desired_frequency_mhz * 1,
        }) {
            return None;
        }

        Some(pll_config)
    }

    /// Start PLL clock. It supports only HSI as source at the moment.
    /// Returns:
    /// + Err(ErrorCode::INVAL) if the desired frequency can't be achieved
    /// + Err(ErrorCode::FAIL) if any PLL clock is already enabled. They must be disabled before
    /// configuring them again.
    /// + Err(ErrorCode::BUSY) starting the PLL clock took too long. Retry.
    /// + Ok(()) everything went OK
    pub fn start(&self, desired_frequency_mhz: usize) -> Result<(), ErrorCode> {
        // Check whether the PLL clock is running or not
        if self.rcc.is_enabled_pll_clock() {
            return Result::from(ErrorCode::FAIL);
        }
        // Config the PLL
        let pll_config = Self::get_pll_config_for_frequency_using_hsi(desired_frequency_mhz);
        if let None = pll_config {
            return Result::from(ErrorCode::INVAL);
        }
        let pll_config = pll_config.unwrap();
        self.rcc.set_pll_clock_n_multiplier(pll_config.get_n());
        self.rcc.set_pll_clock_p_divider(pll_config.get_p());

        // Enable PLL clock
        self.rcc.enable_pll_clock()
    }

    /// Stop PLL clock.
    /// Returns:
    /// + Err(ErrorCode::FAIL) if the PLL clock is configured as the system clock.
    /// + Err(ErrorCode::BUSY) stoping the PLL clock took to long. Retry.
    /// + Ok(()) everything went alright
    pub fn stop(&self) -> Result<(), ErrorCode> {
        self.rcc.disable_pll_clock()
    }
}

pub mod unit_tests {
    use super::*;

    fn test_get_pll_config_for_frequency_using_hsi() {
        debug!("Testing PLL config...");

        // Desired frequency can't be achieved
        assert_eq!(None, Pll::get_pll_config_for_frequency_using_hsi(12));
        assert_eq!(None, Pll::get_pll_config_for_frequency_using_hsi(217));

        // Reachable frequencies
        // 13MHz --> minimum value
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(13).unwrap();
        assert_eq!(52, pll_config.get_n());
        assert_eq!(PLLP::DivideBy8, pll_config.get_p());

        // 25MHz --> minimum required value for Ethernet devices
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(25).unwrap();
        assert_eq!(100, pll_config.get_n());
        assert_eq!(PLLP::DivideBy8, pll_config.get_p());

        // 55MHz --> PLLP becomes DivideBy6
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(55).unwrap();
        assert_eq!(165, pll_config.get_n());
        assert_eq!(PLLP::DivideBy6, pll_config.get_p());

        // 70MHz --> Another value for PLLP::DivideBy6
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(70).unwrap();
        assert_eq!(210, pll_config.get_n());
        assert_eq!(PLLP::DivideBy6, pll_config.get_p());

        // 73MHz --> PLLP becomes DivideBy4
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(73).unwrap();
        assert_eq!(146, pll_config.get_n());
        assert_eq!(PLLP::DivideBy4, pll_config.get_p());

        // 100MHz --> Another value for PLLP::DivideBy4
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(100).unwrap();
        assert_eq!(200, pll_config.get_n());
        assert_eq!(PLLP::DivideBy4, pll_config.get_p());

        // 109MHz --> PLLP becomes DivideBy2
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(109).unwrap();
        assert_eq!(109, pll_config.get_n());
        assert_eq!(PLLP::DivideBy2, pll_config.get_p());

        // 125MHz --> Another value for PLLP::DivideBy2
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(125).unwrap();
        assert_eq!(125, pll_config.get_n());
        assert_eq!(PLLP::DivideBy2, pll_config.get_p());

        // 180MHz --> Max frequency for the CPU
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(180).unwrap();
        assert_eq!(180, pll_config.get_n());
        assert_eq!(PLLP::DivideBy2, pll_config.get_p());

        // 216MHz --> Max frequency for the CPU due to the VCO output frequency limit
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(216).unwrap();
        assert_eq!(216, pll_config.get_n());
        assert_eq!(PLLP::DivideBy2, pll_config.get_p());

        debug!("Finished testing PLL config.");
    }

    fn test_pll_start_stop<'a>(pll: &'a Pll<'a>) {
        debug!("Testing start/stop PLL...");
        // If the pll is already stop, nothing should happen
        assert_eq!(Ok(()), pll.stop());

        // Attempting to start PLL with either too high or too low frequency
        assert_eq!(Err(ErrorCode::INVAL), pll.start(12));
        assert_eq!(Err(ErrorCode::INVAL), pll.start(217));

        // Start the PLL with 25MHz
        assert_eq!(Ok(()), pll.start(25));

        // Impossible to start the PLL if it is already started
        assert_eq!(Err(ErrorCode::FAIL), pll.start(50));

        // Stop PLL
        assert_eq!(Ok(()), pll.stop());

        // Now, it can be configured to run at 50MHz
        assert_eq!(Ok(()), pll.start(50));
        debug!("Finished testing start/stop PLL.");
    }

    pub fn run<'a>(pll: &'a Pll<'a>) {
        debug!("");
        debug!("===============================================");
        debug!("Testing PLL...");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        test_get_pll_config_for_frequency_using_hsi();
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        test_pll_start_stop(pll);
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Finished testing PLL. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
