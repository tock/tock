use crate::clk::pll::Pll;
use crate::clk::hsi::Hsi;
use crate::rcc::Rcc;
use crate::rcc::SysClockSource;
use crate::rcc::APB1Prescaler;
use crate::flash::Flash;
use crate::flash::FlashLatency;

use kernel::debug;
use kernel::ErrorCode;
use kernel::utilities::cells::OptionalCell;

pub struct Clocks<'a> {
    rcc: &'a Rcc,
    flash: OptionalCell<&'a Flash>,
    pub hsi: Hsi<'a>,
    pub pll: Pll<'a>,
}

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

    fn sys_clock_frequency_to_flash_latency(frequency_mhz: usize) -> Result<FlashLatency, ErrorCode> {
        if frequency_mhz <= 30 {
            Ok(FlashLatency::Latency0)
        } else if frequency_mhz <= 60 {
            Ok(FlashLatency::Latency1)
        } else if frequency_mhz <= 90 {
            Ok(FlashLatency::Latency2)
        } else if frequency_mhz <= 120 {
            Ok(FlashLatency::Latency3)
        } else if frequency_mhz <= 150 {
            Ok(FlashLatency::Latency4)
        // HELP: STM32F42xx and STM32F43xx support system clock frequencies up to 180MHz
        } else if frequency_mhz <= 168 {
            Ok(FlashLatency::Latency5)
        } else {
            Err(ErrorCode::SIZE)
        }
    }

    fn set_flash_latency_according_to_sys_clock_freq(&self, frequency_mhz: usize) -> Result<(), ErrorCode> {
        let latency_value = Self::sys_clock_frequency_to_flash_latency(frequency_mhz)?;

        self.flash.unwrap_or_panic().set_latency(latency_value);

        for _ in 0..100 {
            if self.flash.unwrap_or_panic().get_latency() == latency_value {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    fn check_apb1_frequency_limit(&self, frequency_mhz: usize) -> bool {
        match self.rcc.get_apb1_prescaler()  {
            APB1Prescaler::DivideBy1 => frequency_mhz <= 45,
            APB1Prescaler::DivideBy2 => frequency_mhz <= 90,
            // Maximum system clock frequency is 168MHz < 45MHz * 4, which means that a value equal
            // or higher than 4 guarantees the APB1 frequency domain limit.
            _ => true,
        }
    }

    pub fn set_apb1_prescaler(&self, prescaler: APB1Prescaler) -> Result<(), ErrorCode> {
        self.rcc.set_apb1_prescaler(prescaler);

        for _ in 0..16 {
            if self.rcc.get_apb1_prescaler() == prescaler {
                return Ok(());
            }
        }

        Err(ErrorCode::BUSY)
    }

    pub fn get_apb1_prescaler(&self) -> APB1Prescaler {
        self.rcc.get_apb1_prescaler()
    }

    pub fn get_apb1_frequency(&self) -> usize {
        // Every enum variant can be converted into a usize
        let divider: usize = self.rcc.get_apb1_prescaler().try_into().unwrap();
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

        if let false = self.check_apb1_frequency_limit(alternate_frequency) {
            return Err(ErrorCode::SIZE);
        }

        if alternate_frequency > current_frequency {
            self.set_flash_latency_according_to_sys_clock_freq(alternate_frequency)?;
        }
        self.rcc.set_sys_clock_source(source);
        if alternate_frequency < current_frequency {
            self.set_flash_latency_according_to_sys_clock_freq(alternate_frequency)?;
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

    pub fn test_sys_clock_frequency_to_flash_latency() {
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing flash latency value according to the system clock frequency...");

        // HSI frequency
        assert_eq!(Ok(FlashLatency::Latency0), Clocks::sys_clock_frequency_to_flash_latency(16));

        // AHB Ethernet minimal frequency
        assert_eq!(Ok(FlashLatency::Latency0), Clocks::sys_clock_frequency_to_flash_latency(25));

        // Maximum APB1 frequency
        assert_eq!(Ok(FlashLatency::Latency1), Clocks::sys_clock_frequency_to_flash_latency(45));

        // Maximum APB2 frequency
        assert_eq!(Ok(FlashLatency::Latency2), Clocks::sys_clock_frequency_to_flash_latency(90));

        // Default PLL frequency
        assert_eq!(Ok(FlashLatency::Latency3), Clocks::sys_clock_frequency_to_flash_latency(96));

        // Maximum CPU frequency for all STM32F4xx models
        assert_eq!(Ok(FlashLatency::Latency5), Clocks::sys_clock_frequency_to_flash_latency(168));

        // Maximum PLL frequency
        assert_eq!(Err(ErrorCode::SIZE), Clocks::sys_clock_frequency_to_flash_latency(216));

        debug!("Finished testing sys_clock_frequency_to_flash_latency(). Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    pub fn test_set_flash_latency_according_to_sys_clock_freq(clocks: &Clocks) {
        debug!("");
        debug!("===============================================");
        debug!("Testing clocks...");

        // HSI frequency
        assert_eq!(Ok(()), clocks.set_flash_latency_according_to_sys_clock_freq(16));

        // Minimal Ethernet frequency
        assert_eq!(Ok(()), clocks.set_flash_latency_according_to_sys_clock_freq(25));

        // Maximum APB1 frequency
        assert_eq!(Ok(()), clocks.set_flash_latency_according_to_sys_clock_freq(45));

        // Maximum APB2 frequency
        assert_eq!(Ok(()), clocks.set_flash_latency_according_to_sys_clock_freq(90));

        // Default PLL frequency
        assert_eq!(Ok(()), clocks.set_flash_latency_according_to_sys_clock_freq(96));

        // Maximum CPU frequency
        assert_eq!(Ok(()), clocks.set_flash_latency_according_to_sys_clock_freq(168));

        // Maximum PLL frequency
        assert_eq!(Err(ErrorCode::SIZE), clocks.set_flash_latency_according_to_sys_clock_freq(216));
        
        // Revert to default settings
        assert_eq!(Ok(()), clocks.set_flash_latency_according_to_sys_clock_freq(16));

        debug!("Finished testing clocks. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }

    pub fn test_clocks_struct(clocks: &Clocks) {
        debug!("");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("Testing clocks struct...");

        // By default, the HSI clock is the system clock
        assert_eq!(SysClockSource::HSI, clocks.get_sys_clock_source());

        // HSI frequency is 16MHz
        assert_eq!(16, clocks.get_sys_clock_frequency());

        // APB1 default prescaler is 1
        assert_eq!(APB1Prescaler::DivideBy1, clocks.get_apb1_prescaler());

        // APB1 default frequency is 16MHz
        assert_eq!(16, clocks.get_apb1_frequency());

        // Attempting to change the system clock source with a disabled source
        assert_eq!(Err(ErrorCode::FAIL), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Attempting to set twice the same system clock source is fine
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));

        // Change the system clock source
        assert_eq!(Ok(()), clocks.pll.set_frequency(25));
        assert_eq!(Ok(()), clocks.pll.enable());
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::PLLCLK));
        assert_eq!(SysClockSource::PLLCLK, clocks.get_sys_clock_source());

        // Now the system clock frequency is equal to 25MHz
        assert_eq!(25, clocks.get_sys_clock_frequency());

        // APB1 frequency must also be 25MHz
        assert_eq!(25, clocks.get_apb1_frequency());

        // Attempting to disable PLL when it is configured as the system clock must fail
        assert_eq!(Err(ErrorCode::FAIL), clocks.pll.disable());
        // Same for the HSI since it is used indirectly as a system clock through PLL
        assert_eq!(Err(ErrorCode::FAIL), clocks.hsi.disable());

        // Revert to default system clock configuration
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));
        assert_eq!(16, clocks.get_sys_clock_frequency());
        assert_eq!(16, clocks.get_apb1_frequency());

        // Attempting to change system clock frequency without correctly configuring the APB1
        // prescaler (freq_APB1 <= 45MHz) must fail
        assert_eq!(Ok(()), clocks.pll.disable());
        assert_eq!(Ok(()), clocks.pll.set_frequency(100));
        assert_eq!(Ok(()), clocks.pll.enable());
        assert_eq!(Err(ErrorCode::SIZE), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Even if the APB1 prescaler is changed to 2, it must fail (100 / 2 > 45)
        assert_eq!(Ok(()), clocks.set_apb1_prescaler(APB1Prescaler::DivideBy2));
        assert_eq!(Err(ErrorCode::SIZE), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Configure APB1 prescaler to 4 and enabling the PLL clock should work now
        assert_eq!(Ok(()), clocks.set_apb1_prescaler(APB1Prescaler::DivideBy4));
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::PLLCLK));
        assert_eq!(25, clocks.get_apb1_frequency());

        // Revert to default system clock configuration
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));
        assert_eq!(16, clocks.get_sys_clock_frequency());
        assert_eq!(Ok(()), clocks.set_apb1_prescaler(APB1Prescaler::DivideBy1));
        assert_eq!(16, clocks.get_apb1_frequency());

        debug!("Finished testing clocks struct. Everything is alright!");
        debug!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        debug!("");
    }

    pub fn run_all(clocks: &Clocks) {
        debug!("");
        debug!("===============================================");
        debug!("Testing clocks...");

        test_sys_clock_frequency_to_flash_latency();
        test_set_flash_latency_according_to_sys_clock_freq(clocks);
        test_clocks_struct(clocks);

        debug!("Finished testing clocks. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
