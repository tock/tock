use crate::clk::pll::Pll;
use crate::clk::hsi::Hsi;
use crate::clk::hsi::HSI_FREQUENCY_MHZ;
use crate::rcc::Rcc;
use crate::rcc::SysClockSource;
use crate::rcc::APBPrescaler;
use crate::flash::Flash;

use kernel::debug;
use kernel::ErrorCode;
use kernel::utilities::cells::OptionalCell;

pub struct Clocks<'a> {
    rcc: &'a Rcc,
    flash: OptionalCell<&'a Flash>,
    pub hsi: Hsi<'a>,
    pub pll: Pll<'a>,
}

const APB1_FREQUENCY_LIMIT_MHZ: usize =
if cfg!(stm32f410) || cfg!(stm32f411) || cfg!(stm32f412) || cfg!(stm32f413) || cfg!(stm32f423) {
    50
} else if cfg!(stm32f42x) || cfg!(stm32f43x) || cfg!(stm32f446) || cfg!(stm32f469) || cfg!(stm32f479) {
    45
} else if cfg!(stm32f405) || cfg!(stm32f407) || cfg!(stm32f415) || cfg!(stm32f417) || cfg!(stm32f401) {
    42
} else {
    panic!("stm32f4xx flag not defined");
};

// APB2 frequency limit is twice the APB1 frequency limit
const APB2_FREQUENCY_LIMIT_MHZ: usize = APB1_FREQUENCY_LIMIT_MHZ << 1;

// TODO: Ensure that PLL clock never outputs a frequency higher than this
const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize =
if cfg!(stm32f410) || cfg!(stm32f411) || cfg!(stm32f412) || cfg!(stm32f413) || cfg!(stm32f423) {
    100
} else if cfg!(stm32f42x) || cfg!(stm32f43x) || cfg!(stm32f446) || cfg!(stm32f469) || cfg!(stm32f479) { 
    180
} else if cfg!(stm32f405) || cfg!(stm32f407) || cfg!(stm32f415) || cfg!(stm32f417) {
    168
} else if cfg!(stm32f401) {
    84
} else {
    panic!("stm32f4xx flag not defined");
};

impl<'a> Clocks<'a> {
    pub(crate) fn new(rcc: &'a Rcc) -> Self {
        Self {
            rcc,
            flash: OptionalCell::empty(),
            hsi: Hsi::new(rcc),
            pll: Pll::new(rcc),
        }
    }

    pub(crate) fn set_flash(&self, flash: &'a Flash) {
        self.flash.set(flash);
    }

    fn check_apb1_frequency_limit(&self, sys_clk_frequency_mhz: usize) -> bool {
        match self.rcc.get_apb1_prescaler()  {
            APBPrescaler::DivideBy1 => sys_clk_frequency_mhz <= APB1_FREQUENCY_LIMIT_MHZ,
            APBPrescaler::DivideBy2 => sys_clk_frequency_mhz <= APB1_FREQUENCY_LIMIT_MHZ * 2,
            // Maximum system clock frequency is 168MHz < 45MHz * 4, which means that a value equal
            // or higher than 4 guarantees the APB1 frequency domain limit.
            _ => true,
        }
    }

    pub fn set_apb1_prescaler(&self, prescaler: APBPrescaler) -> Result<(), ErrorCode> {
        self.rcc.set_apb1_prescaler(prescaler);

        for _ in 0..16 {
            if self.rcc.get_apb1_prescaler() == prescaler {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    pub fn get_apb1_prescaler(&self) -> APBPrescaler {
        self.rcc.get_apb1_prescaler()
    }

    pub fn get_apb1_frequency(&self) -> usize {
        // Every enum variant can be converted into a usize
        let divider: usize = self.rcc.get_apb1_prescaler().try_into().unwrap();
        self.get_sys_clock_frequency() / divider
    }

    fn check_apb2_frequency_limit(&self, sys_clk_frequency_mhz: usize) -> bool {
        match self.rcc.get_apb2_prescaler() {
            APBPrescaler::DivideBy1 => sys_clk_frequency_mhz <= APB2_FREQUENCY_LIMIT_MHZ,
            // Maximum system clock frequency is 168MHz < 90MHz * 2, which means that a value equal
            // or higher than 2 for the APB2 prescaler guarantees the APB2 frequency domain limit.
            _ => true,
        }
    }

    pub fn set_apb2_prescaler(&self, prescaler: APBPrescaler) -> Result<(), ErrorCode> {
        self.rcc.set_apb2_prescaler(prescaler);

        for _ in 0..16 {
            if self.rcc.get_apb2_prescaler() == prescaler {
                return  Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    pub fn get_apb2_prescaler(&self) -> APBPrescaler {
        self.rcc.get_apb2_prescaler()
    }

    pub fn get_apb2_frequency(&self) -> usize {
        // Every enum variant can be converted into a usize
        let divider: usize = self.rcc.get_apb2_prescaler().try_into().unwrap();
        self.get_sys_clock_frequency() / divider
    }

    pub fn set_sys_clock_source(&self, source: SysClockSource) -> Result<(), ErrorCode> {
        if source == self.get_sys_clock_source() {
            return Ok(());
        }

        if let false = match source {
            SysClockSource::HSI => self.hsi.is_enabled(),
            SysClockSource::PLLCLK => self.pll.is_enabled(),
        } {
            return Err(ErrorCode::FAIL);
        }

        let current_frequency = self.get_sys_clock_frequency();
        let alternate_frequency = match source {
            // The unwrap can't failed because the source clock status was checked before
            SysClockSource::HSI => self.hsi.get_frequency().unwrap(),
            SysClockSource::PLLCLK => self.pll.get_frequency().unwrap(),
        };

        if alternate_frequency > SYS_CLOCK_FREQUENCY_LIMIT_MHZ {
            return Err(ErrorCode::SIZE);
        }

        // APB1 frequency must not exceed APB1_FREQUENCY_LIMIT_MHZ
        if let false = self.check_apb1_frequency_limit(alternate_frequency) {
            return Err(ErrorCode::SIZE);
        }

        // APB2 frequency must not exceed APB2_FREQUENCY_LIMIT_MHZ
        if let false = self.check_apb2_frequency_limit(alternate_frequency) {
            return Err(ErrorCode::SIZE);
        }

        if alternate_frequency > current_frequency {
            self.flash.unwrap_or_panic().set_latency(alternate_frequency);
        }
        self.rcc.set_sys_clock_source(source);
        if alternate_frequency < current_frequency {
            self.flash.unwrap_or_panic().set_latency(alternate_frequency);
        }

        Ok(())
    }

    pub fn get_sys_clock_source(&self) -> SysClockSource {
        self.rcc.get_sys_clock_source()
    }

    pub fn get_sys_clock_frequency(&self) -> usize {
        match self.get_sys_clock_source() {
            // These unwraps can't panic because set_sys_clock_frequency ensures that the source is
            // enabled. Also, Hsi and Pll structs ensure that the clocks can't be disabled when
            // they are configured as the system clock
            SysClockSource::HSI => self.hsi.get_frequency().unwrap(),
            SysClockSource::PLLCLK => self.pll.get_frequency().unwrap(),
        }
    }
}

pub mod tests {
    use super::*;

    #[cfg(any(stm32f401, stm32f410, stm32f411, stm32f412, stm32f413, stm32f423))]
    pub fn test_clocks_struct(clocks: &Clocks) {
        const LOW_FREQUENCY: usize = 25;
        const HIGH_FREQUENCY: usize = 80;
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing clocks struct...");

        // By default, the HSI clock is the system clock
        assert_eq!(SysClockSource::HSI, clocks.get_sys_clock_source());

        // HSI frequency is 16MHz
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_sys_clock_frequency());

        // APB1 default prescaler is 1
        assert_eq!(APBPrescaler::DivideBy1, clocks.get_apb1_prescaler());

        // APB1 default frequency is 16MHz
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb1_frequency());

        // APB2 default prescaler is 1
        assert_eq!(APBPrescaler::DivideBy1, clocks.get_apb1_prescaler());

        // APB2 default frequency is 16MHz
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb2_frequency());

        // Attempting to change the system clock source with a disabled source
        assert_eq!(Err(ErrorCode::FAIL), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Attempting to set twice the same system clock source is fine
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));

        // Change the system clock source to a low frequency so that APB prescalers don't need to be
        // changed
        assert_eq!(Ok(()), clocks.pll.set_frequency(LOW_FREQUENCY));
        assert_eq!(Ok(()), clocks.pll.enable());
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::PLLCLK));
        assert_eq!(SysClockSource::PLLCLK, clocks.get_sys_clock_source());

        // Now the system clock frequency is equal to 25MHz
        assert_eq!(LOW_FREQUENCY, clocks.get_sys_clock_frequency());

        // APB1 and APB2 frequencies must also be 25MHz
        assert_eq!(LOW_FREQUENCY, clocks.get_apb1_frequency());
        assert_eq!(LOW_FREQUENCY, clocks.get_apb2_frequency());

        // Attempting to disable PLL when it is configured as the system clock must fail
        assert_eq!(Err(ErrorCode::FAIL), clocks.pll.disable());
        // Same for the HSI since it is used indirectly as a system clock through PLL
        assert_eq!(Err(ErrorCode::FAIL), clocks.hsi.disable());

        // Revert to default system clock configuration
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_sys_clock_frequency());
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb1_frequency());
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb2_frequency());

        // Trying to configure a high frequency for the system clock without configuring the APB1
        // prescaler must fail
        assert_eq!(Ok(()), clocks.pll.disable());
        assert_eq!(Ok(()), clocks.pll.set_frequency(HIGH_FREQUENCY));
        assert_eq!(Ok(()), clocks.pll.enable());
        assert_eq!(Err(ErrorCode::SIZE), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Configuring APB1 prescaler to 4
        assert_eq!(Ok(()), clocks.set_apb1_prescaler(APBPrescaler::DivideBy4));

        // Now, PLL can be set as the system clock source
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Configuring APB2 prescaler to 2
        assert_eq!(Ok(()), clocks.set_apb2_prescaler(APBPrescaler::DivideBy2));

        // Check new APB frequencies
        assert_eq!(HIGH_FREQUENCY / 4, clocks.get_apb1_frequency());
        assert_eq!(HIGH_FREQUENCY / 2, clocks.get_apb2_frequency());

        // Revert to default system clock configuration
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_sys_clock_frequency());
        assert_eq!(Ok(()), clocks.set_apb1_prescaler(APBPrescaler::DivideBy1));
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb1_frequency());
        assert_eq!(Ok(()), clocks.set_apb2_prescaler(APBPrescaler::DivideBy1));
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb2_frequency());

        debug!("Finished testing clocks struct. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    #[cfg(not(any(stm32f401, stm32f410, stm32f411, stm32f412, stm32f413, stm32f423)))]
    pub fn test_clocks_struct(clocks: &Clocks) {
        const LOW_FREQUENCY: usize = 25;
        const HIGH_FREQUENCY: usize = 112;
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing clocks struct...");

        // By default, the HSI clock is the system clock
        assert_eq!(SysClockSource::HSI, clocks.get_sys_clock_source());

        // HSI frequency is 16MHz
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_sys_clock_frequency());

        // APB1 default prescaler is 1
        assert_eq!(APBPrescaler::DivideBy1, clocks.get_apb1_prescaler());

        // APB1 default frequency is 16MHz
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb1_frequency());

        // APB2 default prescaler is 1
        assert_eq!(APBPrescaler::DivideBy1, clocks.get_apb1_prescaler());

        // APB2 default frequency is 16MHz
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb2_frequency());

        // Attempting to change the system clock source with a disabled source
        assert_eq!(Err(ErrorCode::FAIL), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Attempting to set twice the same system clock source is fine
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));

        // Change the system clock source to a low frequency so that APB prescalers don't need to be
        // changed
        assert_eq!(Ok(()), clocks.pll.set_frequency(LOW_FREQUENCY));
        assert_eq!(Ok(()), clocks.pll.enable());
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::PLLCLK));
        assert_eq!(SysClockSource::PLLCLK, clocks.get_sys_clock_source());

        // Now the system clock frequency is equal to 25MHz
        assert_eq!(LOW_FREQUENCY, clocks.get_sys_clock_frequency());

        // APB1 and APB2 frequencies must also be 25MHz
        assert_eq!(LOW_FREQUENCY, clocks.get_apb1_frequency());
        assert_eq!(LOW_FREQUENCY, clocks.get_apb2_frequency());

        // Attempting to disable PLL when it is configured as the system clock must fail
        assert_eq!(Err(ErrorCode::FAIL), clocks.pll.disable());
        // Same for the HSI since it is used indirectly as a system clock through PLL
        assert_eq!(Err(ErrorCode::FAIL), clocks.hsi.disable());

        // Revert to default system clock configuration
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_sys_clock_frequency());
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb1_frequency());
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb2_frequency());

        // Attempting to change the system clock frequency without correctly configuring the APB1
        // prescaler (freq_APB1 <= APB1_FREQUENCY_LIMIT_MHZ) and APB2 prescaler
        // (freq_APB2 <= APB2_FREQUENCY_LIMIT_MHZ) must fail
        assert_eq!(Ok(()), clocks.pll.disable());
        assert_eq!(Ok(()), clocks.pll.set_frequency(HIGH_FREQUENCY));
        assert_eq!(Ok(()), clocks.pll.enable());
        assert_eq!(Err(ErrorCode::SIZE), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Even if the APB1 prescaler is changed to 2, it must fail
        // (HIGH_FREQUENCY / 2 > APB1_FREQUENCY_LIMIT_MHZ)
        assert_eq!(Ok(()), clocks.set_apb1_prescaler(APBPrescaler::DivideBy2));
        assert_eq!(Err(ErrorCode::SIZE), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Configuring APB1 prescaler to 4 is fine, but APB2 prescaler is still wrong
        assert_eq!(Ok(()), clocks.set_apb1_prescaler(APBPrescaler::DivideBy4));
        assert_eq!(Err(ErrorCode::SIZE), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Configuring APB2 prescaler to 2
        assert_eq!(Ok(()), clocks.set_apb2_prescaler(APBPrescaler::DivideBy2));

        // Now the system clock source can be changed
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::PLLCLK));
        assert_eq!(HIGH_FREQUENCY / 4, clocks.get_apb1_frequency());
        assert_eq!(HIGH_FREQUENCY / 2, clocks.get_apb2_frequency());

        // Revert to default system clock configuration
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_sys_clock_frequency());
        assert_eq!(Ok(()), clocks.set_apb1_prescaler(APBPrescaler::DivideBy1));
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb1_frequency());
        assert_eq!(Ok(()), clocks.set_apb2_prescaler(APBPrescaler::DivideBy1));
        assert_eq!(HSI_FREQUENCY_MHZ, clocks.get_apb2_frequency());

        debug!("Finished testing clocks struct. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    pub fn run_all(clocks: &Clocks) {
        debug!("");
        debug!("===============================================");
        debug!("Testing clocks...");

        test_clocks_struct(clocks);

        debug!("Finished testing clocks. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
