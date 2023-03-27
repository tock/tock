use crate::clk::pll::Pll;
use crate::clk::hsi::Hsi;
use crate::rcc::Rcc;
use crate::rcc::SysClockSource;
use crate::rcc::APB1Prescaler;
use crate::flash::Flash;
use crate::flash::LatencyValue;

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

    fn set_flash_latency_according_to_frequency(&self, frequency_mhz: usize) -> Result<(), ErrorCode> {
        let latency_value = if frequency_mhz < 30 {
            LatencyValue::Latency0
        } else if frequency_mhz < 60 {
            LatencyValue::Latency1
        } else if frequency_mhz < 90 {
            LatencyValue::Latency2
        } else if frequency_mhz < 120 {
            LatencyValue::Latency3
        } else if frequency_mhz < 150 {
            LatencyValue::Latency4
        // STM32F42xx and STM32F43xx support system clock frequencies up to 180MHz
        } else if frequency_mhz < 168 {
            LatencyValue::Latency5
        } else {
            return Err(ErrorCode::SIZE);
        };

        self.flash.unwrap_or_panic().set_latency(latency_value);

        for _ in 0..100 {
            if self.flash.unwrap_or_panic().get_latency() == latency_value {
                return Ok(());
            }
        }

        return Err(ErrorCode::BUSY);
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

        if alternate_frequency > current_frequency {
            self.set_flash_latency_according_to_frequency(alternate_frequency)?;
        }
        self.rcc.set_sys_clock_source(source);
        if alternate_frequency < current_frequency {
            self.set_flash_latency_according_to_frequency(alternate_frequency)?;
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

    pub fn run(clocks: &Clocks) {
        debug!("");
        debug!("===============================================");
        debug!("Testing clocks...");

        // By default, the HSI clock is the system clock
        assert_eq!(SysClockSource::HSI, clocks.get_sys_clock_source());

        // HSI frequency is 16MHz
        assert_eq!(16, clocks.get_sys_clock_frequency());

        // Attempting to change the system clock source with a disabled source
        assert_eq!(Err(ErrorCode::FAIL), clocks.set_sys_clock_source(SysClockSource::PLLCLK));

        // Attempting to set twice the same system clock source is fine
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));

        // Change the system clock source
        assert_eq!(Ok(()), clocks.pll.enable());
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::PLLCLK));
        assert_eq!(SysClockSource::PLLCLK, clocks.get_sys_clock_source());

        // Now the system clock frequency is equal to 96MHz (default PLL frequency)
        assert_eq!(96, clocks.get_sys_clock_frequency());

        // Attempting to disable PLL when it is configured as the system clock must fail
        assert_eq!(Err(ErrorCode::FAIL), clocks.pll.disable());
        // Same for the HSI since it is used indirectly as a system clock through PLL
        assert_eq!(Err(ErrorCode::FAIL), clocks.hsi.disable());

        // Revert to default system clock configuration
        assert_eq!(Ok(()), clocks.set_sys_clock_source(SysClockSource::HSI));
        assert_eq!(16, clocks.get_sys_clock_frequency());

        debug!("Finished testing clocks. Everything is alright!");
        debug!("===============================================");
        debug!("");
    }
}
