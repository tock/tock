use crate::rcc::Rcc;
use crate::rcc::PLLP;

use kernel::debug;
use kernel::ErrorCode;
use kernel::utilities::cells::OptionalCell;

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
    frequency: OptionalCell<usize>,
}

impl<'a> Pll<'a> {
    /// Create a new instance of the PLL clock.
    ///
    /// ## Panics
    ///
    /// This constructor may panic if an internal error occurred.
    pub fn new(rcc: &'a Rcc) -> Self {
        let pll = Self {
            rcc,
            frequency: OptionalCell::empty(),
        };
        if let Err(_) = pll.set_frequency(16) {
            panic!("Something went wrong when creating the PLL clock structure");
        }
        pll
    }

    // **NOTE**: It assumes a value of 8 for PLLM (check rcc::init_pll_clocks() method)
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

    /// Start the PLL clock.
    pub fn enable(&self) -> Result<(), ErrorCode> {
        // Enable PLL clock
        self.rcc.enable_pll_clock()
    }

    /// Stop the PLL clock.
    /// Returns:
    /// + Err(ErrorCode::FAIL) if the PLL clock is configured as the system clock.
    /// + Err(ErrorCode::BUSY) disabling the PLL clock took to long. Retry.
    /// + Ok(()) everything went alright
    pub fn disable(&self) -> Result<(), ErrorCode> {
        self.rcc.disable_pll_clock()
    }

    /// Check whether the PLL clock is enabled or not.
    ///
    /// Returns true if the PLL clock is enabled, otherwise false.
    pub fn is_enabled(&self) -> bool {
        self.rcc.is_enabled_pll_clock()
    }

    /// Set the frequency of the PLL clock. The PLL clock must be disabled.
    ///
    /// frequency must be in MHz
    ///
    /// Returns:
    /// + Err(ErrorCode::INVAL) if the desired frequency can't be achieved
    /// + Err(ErrorCode::FAIL) if the PLL clock is already enabled. It must be disabled before
    /// configuring it.
    /// + Err(ErrorCode::BUSY) starting the PLL clock took too long. Retry.
    /// + Ok(()) everything went OK
    pub fn set_frequency(&self, desired_frequency_mhz: usize) -> Result<(), ErrorCode> {
        // Check whether the PLL clock is running or not
        if self.rcc.is_enabled_pll_clock() {
            return Result::from(ErrorCode::FAIL);
        }
        // Configure the PLL
        let pll_config = Self::get_pll_config_for_frequency_using_hsi(desired_frequency_mhz);
        if let None = pll_config {
            return Result::from(ErrorCode::INVAL);
        }
        let pll_config = pll_config.unwrap();
        self.rcc.set_pll_clock_n_multiplier(pll_config.get_n());
        self.rcc.set_pll_clock_p_divider(pll_config.get_p());

        self.frequency.set(desired_frequency_mhz);

        Ok(())
    }

    /// Get the frequency of the PLL clock.
    ///
    /// Returns the frequency in MHz if the clock is enabled, or None if it is disabled.
    pub fn get_frequency(&self) -> Option<usize> {
        if self.is_enabled() {
            self.frequency.extract()
        } else {
            None
        }
    }
}

pub mod unit_tests {
    use super::*;

    fn test_get_pll_config_for_frequency_using_hsi() {
        debug!("Testing PLL configuration...");

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

        // 216MHz --> Max frequency for the PLL due to the VCO output frequency limit
        let pll_config = Pll::get_pll_config_for_frequency_using_hsi(216).unwrap();
        assert_eq!(216, pll_config.get_n());
        assert_eq!(PLLP::DivideBy2, pll_config.get_p());

        debug!("Finished testing PLL configuration.");
    }

    fn test_pll_start_stop<'a>(pll: &'a Pll<'a>) {
        debug!("Testing PLL struct...");
        // Make sure the PLL clock is disabled
        assert_eq!(Ok(()), pll.disable());
        assert_eq!(false, pll.is_enabled());

        // Attempting to configure the PLL with either too high or too low frequency
        assert_eq!(Err(ErrorCode::INVAL), pll.set_frequency(12));
        assert_eq!(Err(ErrorCode::INVAL), pll.set_frequency(217));

        // Start the PLL with the default configuration.
        assert_eq!(Ok(()), pll.enable());

        // Make sure the PLL is enabled.
        assert_eq!(true, pll.is_enabled());

        // By default, the PLL clock is set to 16MHz
        assert_eq!(Some(16), pll.get_frequency());

        // Impossible to configure the PLL clock once it is enabled.
        assert_eq!(Err(ErrorCode::FAIL), pll.set_frequency(50));

        // Stop the PLL in order to reconfigure it.
        assert_eq!(Ok(()), pll.disable());

        // Configure the PLL clock to run at 25MHz
        assert_eq!(Ok(()), pll.set_frequency(25));

        // Start the PLL with the new configuration
        assert_eq!(Ok(()), pll.enable());

        // get_frequency() method should reflect the new change
        assert_eq!(Some(25), pll.get_frequency());

        // Stop the PLL clock
        assert_eq!(Ok(()), pll.disable());

        // Attempting to get the frequency of the PLL clock when it is disabled should return None.
        assert_eq!(None, pll.get_frequency());

        debug!("Finished testing PLL struct.");
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
